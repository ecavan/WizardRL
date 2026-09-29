"""How a player does against a whole table of another player, in full games.

    python -m wizard_rl.league --rows models/simul1.pt runs/ppo1/best.pt@sample \\
        --tables counting models/night1.pt models/simul1.pt@soft10 models/simul1.pt --players 4

Each cell: one row player against a table where every other seat is the column player,
duplicate games (every deal played with the row player in every seat), full games scored as at
the table. It shows the row player's win rate (a fair share is 1/players) and its average margin
over the others per game, each with a 95% interval. Against itself a player wins exactly its fair
share, so the diagonal is a check.
"""

from __future__ import annotations

import argparse

import numpy as np
import torch

from .evaluate import evaluate_fn
from .players import describe, make_player


def cell(row, col, device, players, games, chunks, seed):
    ref = None if isinstance(col, str) else col
    opp = col if isinstance(col, str) else "nets"
    per = max(1, games // chunks)
    # one deal (replayed in every seat) per table: size the batch to the games wanted
    tables = max(4, min(256, round(per / np.mean(players))))
    rs = [evaluate_fn(row, device, per, players, opp, ref, seed=seed + c, tables=tables, game=True) for c in range(chunks)]
    w = np.array([r["win_rate"] for r in rs])
    m = np.array([r["edge"] for r in rs])
    ci = lambda x: 1.96 * x.std(ddof=1) / np.sqrt(len(x))  # noqa: E731
    return w.mean(), ci(w), m.mean(), ci(m)


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--rows", nargs="+", required=True, help="players to rate (PATH[@style])")
    p.add_argument("--tables", nargs="+", required=True, help="opponents, each filling every other seat")
    p.add_argument("--players", default="4", help="table size(s), e.g. 4 or 3,4,5,6 (games are split evenly)")
    p.add_argument("--games", type=int, default=2000, help="games for the row player per cell")
    p.add_argument("--chunks", type=int, default=5, help="independent batches, for the interval")
    p.add_argument("--out", default=None, help="also write the table to this markdown file")
    a = p.parse_args(argv)
    device = torch.device("cpu")
    sizes = [int(x) for x in a.players.split(",")]
    fair = len(sizes) / sum(sizes)
    rows = [(s, make_player(s, device, 1)) for s in a.rows]
    cols = [(s, make_player(s, device, 2)) for s in a.tables]
    head = f"Full games, {'/'.join(map(str, sizes))} players, bids all at once. Each cell: the row player's win rate " \
           f"(fair share {fair:.0%}) and average margin over the rest of the table per game, ± 95% interval."
    lines = [head, "", "| player \\ table of | " + " | ".join(describe(s) for s, _ in cols) + " |",
             "| --- |" + " --- |" * len(cols)]
    print(head, flush=True)
    for rs, rp in rows:
        cells = []
        for cs, cp in cols:
            w, wci, m, mci = cell(rp, cp, device, sizes, a.games, a.chunks, 777)
            txt = f"{w:.0%} ± {wci:.0%}, {m:+.0f} ± {mci:.0f}"
            cells.append(txt)
            print(f"  {describe(rs)} vs table of {describe(cs)}: {txt}", flush=True)
        lines.append(f"| **{describe(rs)}** | " + " | ".join(cells) + " |")
    table = "\n".join(lines)
    print("\n" + table)
    if a.out:
        with open(a.out, "w") as f:
            f.write(table + "\n")


if __name__ == "__main__":
    main()
