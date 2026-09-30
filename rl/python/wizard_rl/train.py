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
           "edge_vs_counting", "bids_made", "bids_made_counting", "edge_vs_random", "edge_vs_reference",
           "wins_vs_counting", "wins_vs_reference"]  # the last two with --game only


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--out", default=f"runs/{time.strftime('%Y%m%d-%H%M%S')}")
    p.add_argument("--hours", type=float, default=None, help="stop after this long")
    p.add_argument("--decisions", type=float, default=None, help="stop after this many more decisions (e.g. 5e8)")
    p.add_argument("--until", type=float, default=None,
                   help="stop once the run has this many decisions in total (counts across --resume)")
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
    p.add_argument("--save-every", type=float, default=5e6, help="decisions between checkpoints of latest.pt")
    p.add_argument("--eval-rounds", type=int, default=20_000)
    p.add_argument("--reference", default=None, help="checkpoint to measure against (e.g. an earlier run's best.pt)")
    p.add_argument("--opponent", default=None,
                   help="exploiter mode: every other seat is this fixed network (no frozen copies of the learner)")
    p.add_argument("--styles", default=None,
                   help="opponents with habits, e.g. overbid,underbid,early-wizard,wild,plain (use with --game); "
                        "each is --style-net with that habit")
    p.add_argument("--style-net", default=None, help="the network the styled opponents play with (default: --init)")
    p.add_argument("--init", default=None, help="start from this checkpoint's weights (new optimizer, new run)")
    p.add_argument("--resume", default=None, help="continue this run from its checkpoint")
    p.add_argument("--in-turn", action="store_true", help="bid in turn (printed rules) instead of all at once")
    p.add_argument("--game", action="store_true", help="play full games and reward winning the game, not round scores")
    p.add_argument("--win-weight", type=float, default=1.0,
                   help="with --game: reward = win_weight x winning + the rest x share of opponents beaten")
    p.add_argument("--game-reward", choices=["win", "margin", "wpa"], default="win",
                   help="with --game: 'win' (see --win-weight); 'margin' = final score minus the best other "
                        "player's; 'wpa' = each round rewarded by how much it changed the chance of winning "
                        "(needs --winprob, see winprob.py)")
    p.add_argument("--winprob", default=None, help="win-probability model for --game-reward wpa")
    p.add_argument("--wpa-weight", type=float, default=None,
                   help="with --game-reward wpa: learn from round score + this x win probability added "
                        "(e.g. 2: +10%% chance of winning is worth 20 points) instead of WPA alone")
    p.add_argument("--eval-games", type=int, default=2000, help="with --game: learner games per evaluation")
    p.add_argument("--device", default="auto", help="auto, cpu, mps or cuda")
    p.add_argument("--threads", type=int, default=0, help="CPU threads for torch (0 = default)")
    p.add_argument("--seed", type=int, default=0)
    a = p.parse_args(argv)
    if a.hours is None and a.decisions is None and a.until is None:
        p.error("give --hours, --decisions or --until")
    if a.threads:
        torch.set_num_threads(a.threads)
    device = best_device() if a.device == "auto" else torch.device(a.device)
    players = [int(x) for x in a.players.split(",")]
    wl, wn, wc = (float(x) for x in a.mix.split(","))
    if a.opponent:
        wl, wn, wc = 0.0, 1.0, 0.0  # an exploiter faces only the target network
    styles = a.styles.split(",") if a.styles else []
    if styles and a.mix == p.get_default("mix"):
        wl, wn, wc = 0.25, 0.75, 0.0  # mostly styled opponents, some copies of itself
    os.makedirs(a.out, exist_ok=True)

    # A resumed run keeps the inputs it started with (older runs read fewer features).
    feats = torch.load(a.resume, map_location="cpu")["net"]["features"] if a.resume else FEATURES
    net = QNet(feats, ACTIONS, a.hidden, a.layers, make_head=True)
    if a.init:
        warm_start(net, a.init)
        print(f"initialised from {a.init}")
    cfg = LoopConfig(batch=a.batch, lr=a.lr, lr_final=a.lr_final, lr_decay=int(a.lr_decay), eps_start=a.eps_start,
                     eps_end=a.eps_end, eps_decay=int(a.eps_decay))
    sim = not a.in_turn
    env = WizardEnv(a.tables, players, dict(learner=wl, nets=wn, counting=wc), a.seed, False, sim, False, a.game,
                    -1.0 if a.game_reward == "margin" else a.win_weight)
    learner = Learner(env, net, cfg, device, a.seed)
    if a.game and a.game_reward == "wpa":
        from .winprob import load_winprob, wpa_returns
        if not a.winprob:
            p.error("--game-reward wpa needs --winprob")
        wp = load_winprob(a.winprob)
        learner.reward_fn = lambda d, ctx: wpa_returns(wp, ctx, a.wpa_weight)
    reference = load_qnet(a.reference) if a.reference else None
    if styles:
        from .styles import STYLES
        bad = [x for x in styles if x not in STYLES and not x.startswith("soft")]
        if bad:
            p.error(f"unknown styles {bad}; choose from {', '.join(STYLES)}")
        base = load_qnet(a.style_net or a.init).to(device).eval()
        for p_ in base.parameters():
            p_.requires_grad_(False)
        learner.frozen = [base] * len(styles)
        learner.styles = styles
        env.set_nets(len(styles))
    if a.opponent:
        target = load_qnet(a.opponent).to(device).eval()
        for p_ in target.parameters():
            p_.requires_grad_(False)
        learner.frozen = [target]
        env.set_nets(1)
    best_edge = float("-inf")
    if a.resume:
        ck = torch.load(a.resume, map_location="cpu")
        net.load_state_dict(ck["model"])  # (a run keeps the feature count it started with)
        learner.opt.load_state_dict(ck["opt"])
        learner.state.decisions = ck["decisions"]
        learner.state.updates = ck.get("updates", 0)
        best_edge = ck.get("best_edge", best_edge)
        for sd in ck.get("pool", []) if not (a.opponent or styles) else []:  # a fixed pool isn't restored
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
    last = dict(t=t0, d=learner.state.decisions, snap=learner.state.decisions, eval=learner.state.decisions)

    def save(name: str, edge: float) -> None:
        torch.save(dict(model=net.state_dict(), opt=learner.opt.state_dict(), decisions=learner.state.decisions,
                        updates=learner.state.updates, best_edge=best_edge, net=net.config(), edge=edge,
                        pool=[m.state_dict() for m in learner.frozen]),
                   os.path.join(a.out, name + ".tmp"))
        os.replace(os.path.join(a.out, name + ".tmp"), os.path.join(a.out, name))  # never a half-written file

    def tick(lr: Learner) -> None:
        nonlocal best_edge
        now = time.time()
        dps = (lr.state.decisions - last["d"]) / max(1e-9, now - last["t"])
        # (a small tolerance: ticks land a few decisions either side of their mark)
        if not (a.opponent or styles) and lr.state.decisions - last["snap"] >= 0.95 * a.snapshot_every:
            lr.freeze(a.pool)
            last["snap"] = lr.state.decisions
        if lr.state.decisions - last["eval"] < 0.95 * a.eval_every:
            save("latest.pt", float("nan"))  # a checkpoint between evaluations
            return
        last["eval"] = lr.state.decisions
        env.stats()
        net.eval()
        n_eval = a.eval_games if a.game else a.eval_rounds
        ec = evaluate(net, device, n_eval, players, "counting", simultaneous=sim, game=a.game)
        er = evaluate(net, device, n_eval // 4, players, "random", simultaneous=sim, game=a.game)
        eref = (evaluate(net, device, n_eval, players, "nets", reference, simultaneous=sim, game=a.game)
                if reference is not None else None)
        net.train()
        cur_lr = lr.opt.param_groups[0]["lr"]
        w.writerow([round(now - t0), lr.state.decisions, lr.state.updates, f"{lr.state.last_loss:.4f}",
                    f"{lr.state.last_make_loss:.4f}", f"{lr.epsilon():.3f}", f"{cur_lr:.2e}", round(dps), len(lr.frozen),
                    f"{ec['edge']:.2f}", f"{ec['learner_bid_rate']:.3f}", f"{ec['other_bid_rate']:.3f}", f"{er['edge']:.2f}",
                    f"{eref['edge']:.2f}" if eref else "",
                    f"{ec['win_rate']:.4f}" if a.game else "", f"{eref['win_rate']:.4f}" if a.game and eref else ""])
        log.flush()
        unit = "game" if a.game else "round"
        if a.game:
            fair = ec["fair_win_rate"]
            ref = f" | vs reference: {eref['edge']:+5.1f}/game, wins {eref['win_rate']:.1%}" if eref else ""
            summary = (f"vs counting: {ec['edge']:+6.1f}/game, wins {ec['win_rate']:.1%} (fair share {fair:.1%}), "
                       f"bids made {ec['learner_bid_rate']:.1%}{ref}")
            score = eref["win_rate"] if eref else ec["win_rate"]
        else:
            ref = f" | vs reference: {eref['edge']:+5.1f}" if eref else ""
            summary = (f"vs counting: {ec['edge']:+5.1f}/{unit}, bids made {ec['learner_bid_rate']:.1%} "
                       f"(theirs {ec['other_bid_rate']:.1%}) | vs random: {er['edge']:+5.1f}{ref}")
            score = ec["edge"]
        print(f"[{(now - t0) / 60:6.1f} min] {lr.state.decisions / 1e6:8.1f}M  {dps:6.0f}/s  loss {lr.state.last_loss:.3f}/"
              f"{lr.state.last_make_loss:.3f}  eps {lr.epsilon():.3f}  pool {len(lr.frozen)} | {summary}", flush=True)
        # best.pt: the best edge over the counting bots; with --game, the best win rate (against
        # the reference network if there is one)
        if score > best_edge:
            best_edge = score
            save("best.pt", score)
        save("latest.pt", score)
        last["t"], last["d"] = time.time(), lr.state.decisions

    print(f"training on {device}; {(('full games, reward: round score + ' + f'{a.wpa_weight:g}' + ' x win probability added' if a.wpa_weight is not None else 'full games, reward: win probability added per round') if a.game_reward == 'wpa' else 'full games, reward: margin over the best other player' if a.game_reward == 'margin' else 'full games, reward: win' + (f' {a.win_weight:g} + placement' if a.win_weight < 1 else '')) if a.game else 'lone rounds, reward: round score'}; "
          f"bids {'all at once' if sim else 'in turn'}; tables {a.tables}; players {players}; "
          f"{'exploiting ' + a.opponent + ' (every other seat)' if a.opponent else ('styled opponents ' + ','.join(styles) + f' ({wn:.0%} of seats)') if styles else 'seats learner/frozen/counting ' + a.mix}; "
          f"network {net.config()}; out {a.out}", flush=True)
    more = a.decisions
    if a.until is not None:
        more = max(0.0, a.until - learner.state.decisions) if more is None else min(more, max(0.0, a.until - learner.state.decisions))
    learner.run(decisions=int(more) if more is not None else None, seconds=a.hours * 3600 if a.hours else None,
                on_tick=tick, tick_every=int(min(a.eval_every, a.save_every)))
    if learner.state.decisions != last["d"]:
        last["eval"] = float("-inf")
        tick(learner)  # a final evaluation, unless one just ran
    log.close()


if __name__ == "__main__":
    main()
