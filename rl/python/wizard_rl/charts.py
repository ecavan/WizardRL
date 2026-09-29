"""Ask a trained network about bidding situations.

    python -m wizard_rl.charts runs/first/best.pt

For each situation it prints every legal bid with the network's expected points for the round
and its chance of making that bid (everyone bidding at once, so no one has seen another bid). Edit SITUATIONS (or call `advise`) to ask your own.
"""

from __future__ import annotations

import sys

import numpy as np
import torch

from . import ACT_BID, bid_scenario
from .net import load_qnet

# (description, players, hand, trump, seat: 1 = left of the dealer (leads first) ... players = the dealer)
SITUATIONS = [
    ("Three trumps, one high: 10♥ 7♥ 3♥, hearts trump, leading", 4, ["10h", "7h", "3h"], "h", 1),
    ("Three trumps, one high: 10♥ 7♥ 3♥, hearts trump, dealer", 4, ["10h", "7h", "3h"], "h", 4),
    ("Ace of trump + two small: A♥ 9♥ 2♥, hearts trump, leading", 4, ["Ah", "9h", "2h"], "h", 1),
    ("Off-suit king: K♠ 5♦ 2♣, hearts trump, leading", 4, ["Ks", "5d", "2c"], "h", 1),
    ("A Wizard and junk: Wiz 4♣ 6♦, hearts trump, leading", 4, ["wiz", "4c", "6d"], "h", 1),
    ("A Jester, ace of trump, junk: Jes A♥ 8♣, hearts trump, leading", 4, ["jes", "Ah", "8c"], "h", 1),
    ("One card: K♥, hearts trump, leading", 4, ["Kh"], "h", 1),
    ("One card: K♥, hearts trump, dealer", 4, ["Kh"], "h", 4),
    ("One card: 7♥, hearts trump, leading", 4, ["7h"], "h", 1),
    ("One card: 7♥, hearts trump, leading, 6 players", 6, ["7h"], "h", 1),
    ("One card: A♠, no trump, leading", 4, ["As"], None, 1),
    ("Five cards: Wiz A♥ K♣ 9♦ 3♠, hearts trump, leading", 4, ["wiz", "Ah", "Kc", "9d", "3s"], "h", 1),
    ("Last round (15 cards, no trump), two off-suit kings and junk", 4,
     ["Ks", "Kd", "9c", "8c", "7c", "6c", "5c", "4c", "3c", "2c", "9d", "8d", "7d", "6d", "5d"], None, 1),
]


@torch.no_grad()
def advise(net, players, hand, trump, position, simultaneous: bool = True) -> list[tuple[int, float, float | None]]:
    obs, legal = bid_scenario(players, hand, trump, position, [], 0, simultaneous)
    q, make = net.both(torch.from_numpy(obs).unsqueeze(0))
    q = q[0].numpy() * 100.0
    p = torch.sigmoid(make[0]).numpy() if make is not None else None
    out = []
    for a in np.flatnonzero(legal):
        out.append((int(a - ACT_BID), float(q[a]), float(p[a]) if p is not None else None))
    return sorted(out, key=lambda t: -t[1])


def main(argv=None) -> None:
    argv = argv if argv is not None else sys.argv[1:]
    if len(argv) != 1:
        print(__doc__)
        sys.exit(2)
    net = load_qnet(argv[0])
    for title, players, hand, trump, pos in SITUATIONS:
        res = advise(net, players, hand, trump, pos)
        best = res[0][0]
        cells = []
        for bid, pts, p in sorted(res):
            mark = "*" if bid == best else " "
            chance = f" ({p:.0%})" if p is not None else ""
            cells.append(f"{mark}bid {bid}: {pts:+.0f}{chance}")
        print(f"{title} [{players} players]\n    " + "   ".join(cells))


if __name__ == "__main__":
    main()
