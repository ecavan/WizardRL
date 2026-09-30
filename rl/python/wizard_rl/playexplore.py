"""Ask the bot about a card-play situation on the first trick of a round.

    python -m wizard_rl.playexplore models/simul1.pt            # the situations below
    python -m wizard_rl.playexplore models/simul1.pt --out charts/play_situations.md

For each situation it lists every card you could play with the bot's expected points for the
round and its chance of making your bid afterwards, best first. Edit SITUATIONS to ask your own.
Cards: Ah, 10s, wiz, jes. Positions: 1 = left of the dealer (leads the first trick) ...
players = the dealer. Bids are everyone's, from position 1 to the dealer.
"""

from __future__ import annotations

import argparse
import json
import os

import numpy as np
import torch

from . import ACT_CARD, action_name, play_scenario
from .net import load_qnet

# (title, players, trump, my hand, my position, everyone's bids by position, cards already in the trick)
SITUATIONS = [
    ("Clubs led on your right, you have no clubs: trump in with the A, the J, or throw off? (hearts trump, you bid 1)",
     4, "h", ["Ah", "Jh", "9s", "4d"], 2, [1, 1, 1, 1], ["5c"]),
    ("Same, but you bid 2", 4, "h", ["Ah", "Jh", "9s", "4d"], 2, [1, 2, 1, 1], ["5c"]),
    ("Same, but you bid 0", 4, "h", ["Ah", "Jh", "9s", "4d"], 2, [1, 0, 1, 1], ["5c"]),
    ("Clubs led, you hold the A and J of clubs (spades trump, you bid 1): take it now with the A, or play the J?",
     4, "s", ["Ac", "Jc", "9h", "4d"], 2, [1, 1, 1, 1], ["5c"]),
    ("Same, but you're last to play and the K of clubs is winning", 4, "s", ["Ac", "Jc", "9h", "4d"], 4, [1, 1, 1, 1],
     ["5c", "Kc", "8c"]),
    ("Trump led on your right (7 of hearts, hearts trump), you hold K and 3 of trump, you bid 1",
     4, "h", ["Kh", "3h", "9s", "Qd"], 2, [1, 1, 1, 1], ["7h"]),
    ("A Wizard is led, you bid 1 with a trump ace, a low club and a Jester (hearts trump)",
     4, "h", ["Ah", "2c", "jes", "9d"], 2, [2, 1, 1, 1], ["wiz"]),
    ("You bid 0, spades led: the J of spades is winning, you hold the Q and 3 of spades, you're last",
     4, "h", ["Qs", "3s", "8d", "5c"], 4, [1, 1, 1, 0], ["Js", "4s", "2s"]),
    ("You lead the first trick holding a Wizard, the A of trump and junk, you bid 2 (hearts trump)",
     4, "h", ["wiz", "Ah", "7c", "4s", "2d"], 1, [2, 1, 1, 1], []),
    ("You lead the first trick and bid 0 (hearts trump)", 4, "h", ["Kh", "9c", "5s", "3d", "jes"], 1, [0, 1, 2, 1], []),
]


@torch.no_grad()
def advise(net, players, trump, hand, position, bids, trick):
    obs, legal = play_scenario(players, hand, trump, position, bids, trick)
    q, make = net.both(torch.from_numpy(obs).unsqueeze(0))
    pts = q[0].numpy() * 100.0
    p = torch.sigmoid(make[0]).numpy() if make is not None else None
    rows = [(action_name(int(a)), float(pts[a]), float(p[a]) if p is not None else None)
            for a in np.flatnonzero(legal) if a >= ACT_CARD]
    return sorted(rows, key=lambda r: -r[1])


def main(argv=None) -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("model")
    ap.add_argument("--out", default=None, help="also write the answers to this markdown file")
    a = ap.parse_args(argv)
    net = load_qnet(a.model).eval()
    md = ["# Play situations", "",
          f"`{os.path.basename(a.model)}`'s view of some first-trick decisions: expected points for the round "
          "and the chance of making your bid, for each card you could play (best first).", ""]
    js = []
    for title, n, trump, hand, pos, bids, trick in SITUATIONS:
        rows = advise(net, n, trump, hand, pos, bids, trick)
        js.append(dict(title=title, players=n, trump=trump, hand=hand, position=pos, bids=bids, trick=trick,
                       plays=[dict(card=c, points=round(x, 1), make=None if pr is None else round(pr, 3)) for c, x, pr in rows]))
        best = rows[0][1]
        print(f"\n{title}\n  {n} players, trump {trump or 'none'}, your hand {' '.join(hand)}, "
              f"trick so far: {' '.join(trick) or '(you lead)'}")
        md += [f"### {title}", "",
               f"{n} players, trump: {trump or 'none'}. Your hand: {' '.join(hand)}. "
               f"Trick so far: {' '.join(trick) or '(you lead)'}. Bids: {bids}.", "",
               "| Play | Expected points | vs best | Chance to make your bid |", "| --- | ---: | ---: | ---: |"]
        for name, x, pr in rows:
            pr_s = f"{pr:.0%}" if pr is not None else ""
            print(f"    {name:<6} {x:+6.1f}  ({x - best:+5.1f})  {pr_s}")
            md.append(f"| {name} | {x:+.1f} | {x - best:+.1f} | {pr_s} |")
        md.append("")
    if a.out:
        with open(a.out, "w") as f:
            f.write("\n".join(md))
        with open(os.path.splitext(a.out)[0] + ".json", "w") as f:
            json.dump(dict(model=os.path.basename(a.model), situations=js), f, indent=1)
        print(f"\nwrote {a.out} (and .json)")


if __name__ == "__main__":
    main()
