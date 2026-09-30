"""Where do real players sit on the strength curve? Score sheets in, comparison out.

Keep a score sheet of real games as a CSV, one row per player per round:

    game,players,cards,player,bid,won
    1,4,1,Eli,0,0
    1,4,1,Sister,1,1
    ...

(`players` = table size, `cards` = cards dealt that round). Then

    python -m wizard_rl.family scores.csv

prints each player's bid-made rate and average points per round, next to the same numbers for
the bot and for imperfect copies of it (the `@softT` players from the strength curve) at that table
size, and the closest match. Bids made and points per round are what a score sheet shows; they
place a player on the curve roughly (a single evening is a small sample, so the output says how
sure it is).

    python -m wizard_rl.family --reference models/simul1.pt     # (re)build the reference numbers

The reference is saved in rl/charts/soft_reference.json.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
from collections import defaultdict

import numpy as np
import torch

from . import WizardEnv
from .net import load_qnet

LEVELS = [0.0, 1.0, 2.0, 4.0, 6.0, 10.0, 20.0]  # 0 = the bot itself (always its best move)
REF_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "charts", "soft_reference.json")


def score(bid: int, won: int) -> int:
    return 20 + 10 * won if bid == won else -10 * abs(bid - won)


@torch.no_grad()
def reference(model: str, sizes=(3, 4, 5, 6), games: int = 300, seed: int = 5) -> dict:
    """Bid-made rate and points per round for a table where everyone is `model@softT`."""
    net = load_qnet(model).eval()
    gen = torch.Generator().manual_seed(seed)
    out = {}
    for n in sizes:
        for t in LEVELS:
            # Lone rounds of every size, equally often (as in a game), a fixed number per table
            # so small rounds (which finish first) aren't over-counted.
            tables = 64
            env = WizardEnv(tables, [n], "selfplay", seed, False, True)
            env.set_quota(max(1, round(games * (60 // n) / tables)))
            env.stats()
            made = pts = rounds = 0
            while env.quota_left() > 0:
                obs, legal, _ = env.observe()
                q = net(torch.from_numpy(obs)) * 100.0
                lg = torch.from_numpy(legal)
                if t == 0:
                    a = q.masked_fill(~lg, float("-inf")).argmax(1)
                else:
                    a = torch.multinomial(torch.softmax((q / t).masked_fill(~lg, float("-inf")), 1), 1, generator=gen).squeeze(1)
                env.step(a.numpy())
                env.drain()
                st = env.stats()
                made += st["learner_bids_made"]
                pts += st["learner_score"]
                rounds += st["learner_rounds"]
            out[f"{n}/{t:g}"] = dict(made=made / rounds, points=pts / rounds, seat_rounds=rounds)
            print(f"  {n} players, soft{t:g}: made {made / rounds:.0%}, {pts / rounds:.1f} points per round", flush=True)
    return out


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("sheet", nargs="?", help="score sheet CSV")
    p.add_argument("--reference", default=None, help="(re)build the reference numbers from this model")
    a = p.parse_args(argv)
    if a.reference:
        ref = reference(a.reference)
        with open(REF_PATH, "w") as f:
            json.dump(dict(model=os.path.basename(a.reference), levels=LEVELS, table=ref), f, indent=1)
        print(f"wrote {os.path.normpath(REF_PATH)}")
    if not a.sheet:
        return
    with open(REF_PATH) as f:
        ref = json.load(f)["table"]
    rows = defaultdict(list)
    with open(a.sheet) as f:
        for r in csv.DictReader(f):
            rows[(r["player"], int(r["players"]))].append((int(r["bid"]), int(r["won"])))
    print(f"{'player':<12} {'table':>5} {'rounds':>6} {'made':>11} {'points/round':>13}   closest")
    for (name, n), rs in sorted(rows.items()):
        b = np.array([x for x, _ in rs])
        w = np.array([y for _, y in rs])
        made = (b == w).mean()
        pts = np.mean([score(x, y) for x, y in rs])
        se = np.sqrt(made * (1 - made) / len(rs))
        # closest reference level by bid-made rate (the steadier of the two numbers)
        levels = [(t, ref.get(f"{n}/{t:g}")) for t in LEVELS]
        levels = [(t, v) for t, v in levels if v]
        best = min(levels, key=lambda tv: abs(tv[1]["made"] - made)) if levels else None
        near = (f"bot{'' if best[0] == 0 else f'@soft{best[0]:g}'} "
                f"(made {best[1]['made']:.0%}, {best[1]['points']:.1f} pts)") if best else "no reference"
        print(f"{name:<12} {n:>5} {len(rs):>6} {made:>6.0%} ± {1.96 * se:.0%} {pts:>13.1f}   {near}")
    print("\n± is a 95% interval: with a few dozen rounds a player's made-rate is only known to within "
          "±10-15 points, so read the match as a rough placement.")


if __name__ == "__main__":
    main()
