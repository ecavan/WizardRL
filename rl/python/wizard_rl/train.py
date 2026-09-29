"""Train a Wizard network by self-play (Deep Monte Carlo).

    python -m wizard_rl.train --hours 8 --out runs/first

Every `--eval-every` decisions it plays the current network against counting bots and
random bots, logs the results to `metrics.csv`, and saves `latest.pt` (and `best.pt`, the
checkpoint with the best edge over the counting bots). Resume with `--resume runs/first/latest.pt`.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import time

import torch

from . import ACTIONS, FEATURES, WizardEnv
from .dmc import Learner, LoopConfig
from .evaluate import evaluate
from .net import QNet, best_device


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--out", default=f"runs/{time.strftime('%Y%m%d-%H%M%S')}")
    p.add_argument("--hours", type=float, default=None, help="stop after this long")
    p.add_argument("--decisions", type=float, default=None, help="stop after this many decisions (e.g. 5e8)")
    p.add_argument("--players", default="3,4,5,6", help="table sizes to train on")
    p.add_argument("--tables", type=int, default=512, help="tables played in parallel")
    p.add_argument("--hidden", type=int, default=512)
    p.add_argument("--layers", type=int, default=3)
    p.add_argument("--lr", type=float, default=3e-4)
    p.add_argument("--batch", type=int, default=4096)
    p.add_argument("--eps-start", type=float, default=0.2)
    p.add_argument("--eps-end", type=float, default=0.02)
    p.add_argument("--eps-decay", type=float, default=2e7, help="decisions over which exploration decays")
    p.add_argument("--eval-every", type=float, default=2e6, help="decisions between evaluations")
    p.add_argument("--eval-rounds", type=int, default=20_000)
    p.add_argument("--device", default="auto", help="auto, cpu, mps or cuda")
    p.add_argument("--threads", type=int, default=0, help="CPU threads for torch (0 = default)")
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--resume", default=None, help="checkpoint to continue from")
    a = p.parse_args(argv)
    if a.hours is None and a.decisions is None:
        p.error("give --hours or --decisions")
    if a.threads:
        torch.set_num_threads(a.threads)
    device = best_device() if a.device == "auto" else torch.device(a.device)
    players = [int(x) for x in a.players.split(",")]
    os.makedirs(a.out, exist_ok=True)

    net = QNet(FEATURES, ACTIONS, a.hidden, a.layers)
    cfg = LoopConfig(batch=a.batch, lr=a.lr, eps_start=a.eps_start, eps_end=a.eps_end, eps_decay=int(a.eps_decay))
    env = WizardEnv(a.tables, players, "selfplay", a.seed)
    learner = Learner(env, net, cfg, device, a.seed)
    best_edge = float("-inf")
    if a.resume:
        ck = torch.load(a.resume, map_location="cpu")
        net.load_state_dict(ck["model"])
        learner.opt.load_state_dict(ck["opt"])
        learner.state.decisions = ck["decisions"]
        learner.state.updates = ck.get("updates", 0)
        best_edge = ck.get("best_edge", best_edge)
        print(f"resumed from {a.resume} at {learner.state.decisions:,} decisions")
    with open(os.path.join(a.out, "config.json"), "w") as f:
        json.dump(dict(vars(a), net=net.config(), device=str(device)), f, indent=2)

    log_path = os.path.join(a.out, "metrics.csv")
    new_log = not os.path.exists(log_path)
    log = open(log_path, "a", newline="")
    w = csv.writer(log)
    if new_log:
        w.writerow(["time_s", "decisions", "updates", "loss", "epsilon", "decisions_per_s", "selfplay_avg", "selfplay_bid_rate",
                    "edge_vs_counting", "avg_vs_counting", "bid_rate_vs_counting", "edge_vs_random"])
    t0 = time.time()
    last = dict(t=t0, d=learner.state.decisions)

    def save(name: str, edge: float) -> None:
        torch.save(dict(model=net.state_dict(), opt=learner.opt.state_dict(), decisions=learner.state.decisions,
                        updates=learner.state.updates, best_edge=best_edge, net=net.config(), edge=edge),
                   os.path.join(a.out, name))

    def tick(lr: Learner) -> None:
        nonlocal best_edge
        now = time.time()
        dps = (lr.state.decisions - last["d"]) / max(1e-9, now - last["t"])
        sp = env.stats()
        sp_avg = sp["learner_score"] / max(1, sp["learner_rounds"])
        sp_bid = sp["learner_bids_made"] / max(1, sp["learner_rounds"])
        net.eval()
        ec = evaluate(net, device, a.eval_rounds, players, "counting")
        er = evaluate(net, device, a.eval_rounds // 4, players, "random")
        net.train()
        w.writerow([round(now - t0), lr.state.decisions, lr.state.updates, f"{lr.state.last_loss:.4f}", f"{lr.epsilon():.3f}",
                    round(dps), f"{sp_avg:.2f}", f"{sp_bid:.3f}", f"{ec['edge']:.2f}", f"{ec['learner_avg']:.2f}",
                    f"{ec['learner_bid_rate']:.3f}", f"{er['edge']:.2f}"])
        log.flush()
        print(f"[{(now - t0) / 60:6.1f} min] {lr.state.decisions / 1e6:8.1f}M decisions  {dps:8.0f}/s  loss {lr.state.last_loss:.3f}  "
              f"eps {lr.epsilon():.3f} | vs counting: edge {ec['edge']:+6.1f}/round, bids made {ec['learner_bid_rate']:.1%} "
              f"(bots {ec['bot_bid_rate']:.1%}) | vs random: edge {er['edge']:+6.1f}", flush=True)
        if ec["edge"] > best_edge:
            best_edge = ec["edge"]
            save("best.pt", ec["edge"])
        save("latest.pt", ec["edge"])
        # Don't count evaluation time against training speed.
        last["t"], last["d"] = time.time(), lr.state.decisions

    print(f"training on {device}; tables {a.tables}; players {players}; network {net.config()}; out {a.out}", flush=True)
    learner.run(decisions=int(a.decisions) if a.decisions else None, seconds=a.hours * 3600 if a.hours else None,
                on_tick=tick, tick_every=int(a.eval_every))
    if learner.state.decisions != last["d"]:
        tick(learner)  # a final evaluation, unless one just ran
    log.close()


if __name__ == "__main__":
    main()
