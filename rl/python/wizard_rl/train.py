"""Train a Wizard network by self-play (Deep Monte Carlo).

    python -m wizard_rl.train --hours 8 --out runs/first

Seats: one seat per table is always the learner; the others are mostly the learner too, with
some frozen older copies of it (added to a pool every `--snapshot-every` decisions) and a few
counting bots, so it can't drift into habits that only work against itself.

Every `--eval-every` decisions it measures the network on duplicate deals against counting
bots, random bots and (with `--reference`) a fixed earlier network, logs to `metrics.csv`, and
saves `latest.pt` and `best.pt` (best edge over the counting bots). Resume with
`--resume runs/first/latest.pt`; start from an earlier network's weights with `--init`.
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
from .net import QNet, best_device, load_qnet, warm_start

COLUMNS = ["time_s", "decisions", "updates", "loss", "make_loss", "epsilon", "lr", "decisions_per_s", "pool",
           "edge_vs_counting", "bids_made", "bids_made_counting", "edge_vs_random", "edge_vs_reference"]


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
    p.add_argument("--lr-final", type=float, default=None, help="decay the learning rate linearly to this ...")
    p.add_argument("--lr-decay", type=float, default=0, help="... over this many decisions")
    p.add_argument("--batch", type=int, default=4096)
    p.add_argument("--eps-start", type=float, default=0.2)
    p.add_argument("--eps-end", type=float, default=0.02)
    p.add_argument("--eps-decay", type=float, default=2e7, help="decisions over which exploration decays")
    p.add_argument("--mix", default="0.7,0.2,0.1", help="other seats: learner,frozen,counting weights")
    p.add_argument("--snapshot-every", type=float, default=2e7, help="decisions between frozen copies")
    p.add_argument("--pool", type=int, default=8, help="frozen copies kept")
    p.add_argument("--eval-every", type=float, default=2e7, help="decisions between evaluations")
    p.add_argument("--eval-rounds", type=int, default=20_000)
    p.add_argument("--reference", default=None, help="checkpoint to measure against (e.g. an earlier run's best.pt)")
    p.add_argument("--init", default=None, help="start from this checkpoint's weights (new optimizer, new run)")
    p.add_argument("--resume", default=None, help="continue this run from its checkpoint")
    p.add_argument("--device", default="auto", help="auto, cpu, mps or cuda")
    p.add_argument("--threads", type=int, default=0, help="CPU threads for torch (0 = default)")
    p.add_argument("--seed", type=int, default=0)
    a = p.parse_args(argv)
    if a.hours is None and a.decisions is None:
        p.error("give --hours or --decisions")
    if a.threads:
        torch.set_num_threads(a.threads)
    device = best_device() if a.device == "auto" else torch.device(a.device)
    players = [int(x) for x in a.players.split(",")]
    wl, wn, wc = (float(x) for x in a.mix.split(","))
    os.makedirs(a.out, exist_ok=True)

    net = QNet(FEATURES, ACTIONS, a.hidden, a.layers, make_head=True)
    if a.init:
        warm_start(net, a.init)
        print(f"initialised from {a.init}")
    cfg = LoopConfig(batch=a.batch, lr=a.lr, lr_final=a.lr_final, lr_decay=int(a.lr_decay), eps_start=a.eps_start,
                     eps_end=a.eps_end, eps_decay=int(a.eps_decay))
    env = WizardEnv(a.tables, players, dict(learner=wl, nets=wn, counting=wc), a.seed)
    learner = Learner(env, net, cfg, device, a.seed)
    reference = load_qnet(a.reference) if a.reference else None
    best_edge = float("-inf")
    if a.resume:
        ck = torch.load(a.resume, map_location="cpu")
        net.load_state_dict(ck["model"])
        learner.opt.load_state_dict(ck["opt"])
        learner.state.decisions = ck["decisions"]
        learner.state.updates = ck.get("updates", 0)
        best_edge = ck.get("best_edge", best_edge)
        for sd in ck.get("pool", []):
            learner.freeze(a.pool)
            learner.frozen[-1].load_state_dict(sd)
        print(f"resumed from {a.resume} at {learner.state.decisions:,} decisions, pool {len(learner.frozen)}")
    with open(os.path.join(a.out, "config.json"), "w") as f:
        json.dump(dict(vars(a), net=net.config(), device=str(device)), f, indent=2)

    log_path = os.path.join(a.out, "metrics.csv")
    new_log = not os.path.exists(log_path)
    log = open(log_path, "a", newline="")
    w = csv.writer(log)
    if new_log:
        w.writerow(COLUMNS)
    t0 = time.time()
    last = dict(t=t0, d=learner.state.decisions, snap=learner.state.decisions)

    def save(name: str, edge: float) -> None:
        torch.save(dict(model=net.state_dict(), opt=learner.opt.state_dict(), decisions=learner.state.decisions,
                        updates=learner.state.updates, best_edge=best_edge, net=net.config(), edge=edge,
                        pool=[m.state_dict() for m in learner.frozen]),
                   os.path.join(a.out, name))

    def tick(lr: Learner) -> None:
        nonlocal best_edge
        now = time.time()
        dps = (lr.state.decisions - last["d"]) / max(1e-9, now - last["t"])
        if lr.state.decisions - last["snap"] >= a.snapshot_every:
            lr.freeze(a.pool)
            last["snap"] = lr.state.decisions
        env.stats()
        net.eval()
        ec = evaluate(net, device, a.eval_rounds, players, "counting")
        er = evaluate(net, device, a.eval_rounds // 4, players, "random")
        eref = evaluate(net, device, a.eval_rounds, players, "nets", reference) if reference is not None else None
        net.train()
        cur_lr = lr.opt.param_groups[0]["lr"]
        w.writerow([round(now - t0), lr.state.decisions, lr.state.updates, f"{lr.state.last_loss:.4f}",
                    f"{lr.state.last_make_loss:.4f}", f"{lr.epsilon():.3f}", f"{cur_lr:.2e}", round(dps), len(lr.frozen),
                    f"{ec['edge']:.2f}", f"{ec['learner_bid_rate']:.3f}", f"{ec['other_bid_rate']:.3f}", f"{er['edge']:.2f}",
                    f"{eref['edge']:.2f}" if eref else ""])
        log.flush()
        ref = f" | vs reference: {eref['edge']:+5.1f}" if eref else ""
        print(f"[{(now - t0) / 60:6.1f} min] {lr.state.decisions / 1e6:8.1f}M  {dps:6.0f}/s  loss {lr.state.last_loss:.3f}/"
              f"{lr.state.last_make_loss:.3f}  eps {lr.epsilon():.3f}  pool {len(lr.frozen)} | vs counting: {ec['edge']:+5.1f}/round, "
              f"bids made {ec['learner_bid_rate']:.1%} (theirs {ec['other_bid_rate']:.1%}) | vs random: {er['edge']:+5.1f}{ref}",
              flush=True)
        if ec["edge"] > best_edge:
            best_edge = ec["edge"]
            save("best.pt", ec["edge"])
        save("latest.pt", ec["edge"])
        last["t"], last["d"] = time.time(), lr.state.decisions

    print(f"training on {device}; tables {a.tables}; players {players}; seats learner/frozen/counting {a.mix}; "
          f"network {net.config()}; out {a.out}", flush=True)
    learner.run(decisions=int(a.decisions) if a.decisions else None, seconds=a.hours * 3600 if a.hours else None,
                on_tick=tick, tick_every=int(a.eval_every))
    if learner.state.decisions != last["d"]:
        tick(learner)  # a final evaluation, unless one just ran
    log.close()


if __name__ == "__main__":
    main()
