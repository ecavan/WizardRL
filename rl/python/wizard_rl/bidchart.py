"""Bid charts: how many tricks each kind of card is worth, by table size and round size.

    python -m wizard_rl.bidchart runs/simul1/best.pt --out charts/

How it's made: the trained bot plays hundreds of thousands of rounds against itself at each
table size (everyone bidding at once). For every seat we record the hand it was dealt and how
many tricks it actually took. Then, separately for each table size and range of round sizes, we
fit

    tricks won  ~  sum over cards of (value of that card's kind)

by least squares. The fitted values are the chart: an off-suit ace might be worth 0.6 of a trick
with 4 players in a 3-card round, but only 0.3 in a 12-card round, where it's more likely to get
trumped. To bid, add up the values of your cards and round to the nearest whole number.

Card kinds: Wizard, Jester, and every rank of trump and of the other suits (A, K, ... 2). In
no-trump rounds (a Jester turned up, or the last round) every suit is "off-suit", so those rounds
get their own chart.
"""

from __future__ import annotations

import argparse
import csv
import json
import os

import numpy as np
import torch

from . import WizardEnv
from .net import load_qnet, pick_actions

RANKS = ["A", "K", "Q", "J", "10", "9", "8", "7", "6", "5", "4", "3", "2"]  # high to low
# One value per rank (not buckets): tested on held-out hands, per-rank values predict tricks
# better (4 players: 68.9% vs 67.5% of chart bids exactly right) and match the bot's own bid more
# often (88.7% vs 85.8%). Bonus rows for trump length or voids added almost nothing.
KINDS = ["Wizard", "Jester"] + [f"trump {r}" for r in RANKS] + [f"off-suit {r}" for r in RANKS]
NO_TRUMP_KINDS = ["Wizard", "Jester"] + [f"off-suit {r}" for r in RANKS]


def size_bands(rounds: int) -> list[tuple[int, int]]:
    bands = [(1, 1), (2, 2), (3, 4), (5, 7), (8, 10), (11, 15), (16, 20)]
    return [(lo, min(hi, rounds)) for lo, hi in bands if lo <= rounds]


def features(hand: np.ndarray, trump: np.ndarray) -> np.ndarray:
    """Count of each card kind in each hand. `hand` is a uint64 bitmask per row; `trump` is the
    suit index, or 4 for no trump."""
    n = len(hand)
    bits = ((hand[:, None] >> np.arange(60, dtype=np.uint64)[None, :]) & np.uint64(1)).astype(bool)
    x = np.zeros((n, len(KINDS)), dtype=np.float32)
    x[:, 0] = bits[:, 52:56].sum(1)
    x[:, 1] = bits[:, 56:60].sum(1)
    for s in range(4):
        cards = bits[:, 13 * s:13 * s + 13][:, ::-1]  # column 0 = ace ... 12 = deuce, as RANKS
        is_trump = (trump == s)[:, None]
        x[:, 2:15] += np.where(is_trump, cards, 0)
        x[:, 15:28] += np.where(~is_trump, cards, 0)
    return x


@torch.no_grad()
def collect(net, players: int, rounds: int, seed: int = 7, tables: int = 256) -> dict:
    """Self-play with the network in every seat (greedy); returns the round log."""
    env = WizardEnv(tables, [players], "selfplay", seed, False, True, True)
    logs = []
    got = 0
    while got < rounds * players:
        obs, legal, _ = env.observe()
        q = net(torch.from_numpy(obs))
        env.step(pick_actions(q, torch.from_numpy(legal), 0.0).numpy().astype(np.int64))
        env.drain()
        r = env.drain_rounds()
        if len(r["hand"]):
            logs.append(r)
            got += len(r["hand"])
    return {k: np.concatenate([x[k] for x in logs]) for k in logs[0]}


def fit(x: np.ndarray, y: np.ndarray, cols: list[int]) -> tuple[np.ndarray, float, int]:
    """Least squares, no intercept (an empty hand takes no tricks). Returns values, mean
    absolute error and the number of seat-rounds."""
    xs = x[:, cols]
    coef, *_ = np.linalg.lstsq(xs, y, rcond=None)
    mae = float(np.abs(xs @ coef - y).mean())
    return coef, mae, len(y)


def simple(v: np.ndarray) -> np.ndarray:
    """Values rounded to tenths, for adding up at the table. (Quarters were tried: in big hands
    they turn a low card worth 0.1 into 0, and twelve of those cost more than a trick.)"""
    return np.round(np.asarray(v) * 10) / 10


def score(x: np.ndarray, coef: np.ndarray, y: np.ndarray) -> tuple[float, float]:
    """Average miss in tricks, and how often the rounded total is exactly the tricks taken (the
    bid would have been made)."""
    est = x @ coef
    return float(np.abs(est - y).mean()), float((np.clip(np.rint(est), 0, None) == y).mean())


def build(net, players: int, rounds: int, logs: str | None = None) -> dict:
    path = os.path.join(logs, f"selfplay_{players}.npz") if logs else None
    if path and os.path.exists(path):
        log = dict(np.load(path))  # hands saved by an earlier run (--save-logs)
    else:
        log = collect(net, players, rounds)
        if logs:
            os.makedirs(logs, exist_ok=True)
            np.savez_compressed(path, **log)
    x = features(log["hand"], log["trump"].astype(np.int64))
    y = log["won"].astype(np.float32)
    bid = log["bid"].astype(np.int64)
    size = log["size"].astype(np.int64)
    pos = log["position"].astype(np.int64)
    no_trump = log["trump"] == 4
    max_rounds = 60 // players
    out = {"trump": [], "no_trump": []}
    resid = np.full(len(y), np.nan)
    for lo, hi in size_bands(max_rounds):
        band = (size >= lo) & (size <= hi)
        for key, mask, kinds in (("trump", ~no_trump, KINDS), ("no_trump", no_trump, NO_TRUMP_KINDS)):
            sel = band & mask
            if sel.sum() < 500:
                continue
            cols = [KINDS.index(k) for k in kinds]
            xs = x[sel][:, cols]
            coef, mae, n = fit(x[sel], y[sel], cols)
            _, hit = score(xs, coef, y[sel])
            q_mae, q_hit = score(xs, simple(coef), y[sel])
            bot_hit = float((bid[sel] == y[sel]).mean())
            resid[sel] = y[sel] - xs @ coef
            # Jesters and Wizards also help you *make* your bid (dump or grab a trick at will),
            # which a trick count can't show: the change in the chance of making the bid per card
            # held, compared with the same hand holding another card instead.
            made = (bid[sel] == y[sel]).astype(np.float64)
            mc, *_ = np.linalg.lstsq(np.column_stack([np.ones(len(xs)), xs]), made, rcond=None)
            make = {k: float(mc[1 + kinds.index(k)]) for k in ("Wizard", "Jester")}
            out[key].append(dict(band=(lo, hi), values=dict(zip(kinds, coef)), mae=mae, n=n, hit=hit,
                                 q_mae=q_mae, q_hit=q_hit, bot_hit=bot_hit, make=make))
    # Seat: does leading first (seat 1) or dealing (seat n) take more tricks than the cards say?
    ok = ~np.isnan(resid)
    out["seat"] = {int(p): float(resid[ok & (pos == p)].mean()) for p in range(1, players + 1) if (ok & (pos == p)).any()}
    out["bid_made"] = float((bid == log["won"]).mean())
    out["rounds"] = int(len(y) // players)
    return out


def jsonable(res: dict) -> dict:
    def row(r):
        return dict(lo=r["band"][0], hi=r["band"][1], name=band_name(r["band"]), n=r["n"],
                    values={k: round(float(v), 4) for k, v in r["values"].items()},
                    mae=round(r["mae"], 3), hit=round(r["hit"], 3), q_hit=round(r["q_hit"], 3), bot_hit=round(r["bot_hit"], 3),
                    make={k: round(v, 3) for k, v in r["make"].items()})
    return dict(trump=[row(r) for r in res["trump"]], no_trump=[row(r) for r in res["no_trump"]],
                seat={str(k): round(v, 3) for k, v in res["seat"].items()}, bid_made=round(res["bid_made"], 3), rounds=res["rounds"])


def band_name(b: tuple[int, int]) -> str:
    lo, hi = b
    return f"{lo} card{'s' if hi > 1 else ''}" if lo == hi else f"{lo}-{hi} cards"


def to_markdown(players: int, res: dict) -> str:
    lines = [f"### {players} players", ""]
    for key, kinds, title in (("trump", KINDS, "With trump"), ("no_trump", NO_TRUMP_KINDS, "No trump (a Jester turned up, or the last round)")):
        rows = res[key]
        if not rows:
            continue
        lines.append(f"**{title}**: tricks each card is worth, by cards in hand")
        lines.append("")
        lines.append("| Card | " + " | ".join(band_name(r["band"]) for r in rows) + " |")
        lines.append("| --- |" + " ---: |" * len(rows))
        for k in kinds:
            lines.append(f"| {k} | " + " | ".join(f"{r['values'][k]:.2f}" for r in rows) + " |")
        lines.append("| *average miss (tricks)* | " + " | ".join(f"{r['mae']:.2f}" for r in rows) + " |")
        lines.append("| *chart bid made* | " + " | ".join(f"{r['hit']:.0%}" for r in rows) + " |")
        lines.append("| *bot's own bid made* | " + " | ".join(f"{r['bot_hit']:.0%}" for r in rows) + " |")
        lines.append("| *each Jester: chance to make your bid* | " + " | ".join(f"{r['make']['Jester']:+.0%}" for r in rows) + " |")
        lines.append("| *each Wizard: chance to make your bid* | " + " | ".join(f"{r['make']['Wizard']:+.0%}" for r in rows) + " |")
        lines.append("")
    seat = res["seat"]
    names = {1: "left of dealer (leads first)", players: "dealer"}
    lines.append("**Seat**: tricks taken beyond what the cards say, on average: " + "; ".join(
        f"{names.get(p, f'seat {p}')} {v:+.2f}" for p, v in seat.items()) + ".")
    lines.append("")
    return "\n".join(lines)


def cheat_sheet(results: dict[int, dict]) -> str:
    """One compact table per table size, values rounded to tenths."""
    lines = ["# Wizard bid cheat sheet", "",
             "Card values in tricks, rounded to tenths (from the full chart in `bid_chart.md`). "
             "Add up your cards and round to the nearest whole number.", ""]
    for n, res in results.items():
        lines += [f"## {n} players", ""]
        for key, kinds, title in (("trump", KINDS, "With trump"), ("no_trump", NO_TRUMP_KINDS, "No trump")):
            rows = res[key]
            if not rows:
                continue
            lines.append(f"**{title}**")
            lines.append("")
            lines.append("| Card | " + " | ".join(band_name(r["band"]) for r in rows) + " |")
            lines.append("| --- |" + " ---: |" * len(rows))
            for k in kinds:
                lines.append(f"| {k} | " + " | ".join(fmt_q(r["values"][k]) for r in rows) + " |")
            lines.append("| *chart bid made* | " + " | ".join(f"{r['q_hit']:.0%}" for r in rows) + " |")
            lines.append("")
    return "\n".join(lines)


def fmt_q(v: float) -> str:
    q = float(simple(v))
    t = f"{abs(q):.1f}".rstrip("0").rstrip(".") or "0"
    return ("−" if q < 0 else "") + t


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("model")
    p.add_argument("--out", default="charts")
    p.add_argument("--players", default="3,4,5,6")
    p.add_argument("--rounds", type=int, default=150_000, help="rounds of self-play per table size")
    p.add_argument("--logs", default=None, help="folder to save the self-play hands in (and reuse them from next time)")
    a = p.parse_args(argv)
    net = load_qnet(a.model)
    os.makedirs(a.out, exist_ok=True)
    md = ["# Wizard bid chart", "",
          f"Made from `{os.path.basename(a.model)}` playing {a.rounds:,} rounds against itself at each table size, "
          "everyone bidding at once.", "",
          "**How to use it:** find your table size, whether there's trump, and how many cards you hold. "
          "Add up the values of your cards and round to the nearest whole number. That's your bid.", "",
          "- Values are tricks: 0.5 means the card wins a trick about half the time.",
          "- *average miss*: how far the chart total is from the tricks actually taken, on average.",
          "- *chart bid made*: how often the rounded total was exactly right (the bid would have been made).",
          "- *bot's own bid made*: the same for the bot's real bids, which also weigh everything else it sees.",
          "- *Seat*: add this for your seat (it's small; the dealer usually gains a little from playing last "
          "to the first trick).",
          "- Jesters are worth about 0 tricks, but they are far from useless: *each Jester* shows how much one "
          "raises your chance of making your bid (you can always duck a trick with it). Wizards do the same "
          "the other way (you can always take one).", ""]
    rows_csv, rows_q = [], []
    results = {}
    for n in (int(x) for x in a.players.split(",")):
        res = build(net, n, a.rounds, a.logs)
        results[n] = res
        md.append(to_markdown(n, res))
        for key in ("trump", "no_trump"):
            for r in res[key]:
                for k, v in r["values"].items():
                    row = dict(players=n, trump=key, cards=band_name(r["band"]), lo=r["band"][0], hi=r["band"][1], kind=k)
                    rows_csv.append(dict(row, value=round(float(v), 4)))
                    rows_q.append(dict(row, value=float(simple(v))))
        print(f"{n} players done: {res['rounds']:,} rounds, bot made {res['bid_made']:.0%} of its bids", flush=True)
    with open(os.path.join(a.out, "bid_chart.md"), "w") as f:
        f.write("\n".join(md))
    with open(os.path.join(a.out, "bid_cheat_sheet.md"), "w") as f:
        f.write(cheat_sheet(results))
    with open(os.path.join(a.out, "bid_chart.json"), "w") as f:
        json.dump(dict(model=os.path.basename(a.model), rounds=a.rounds, kinds=KINDS, no_trump_kinds=NO_TRUMP_KINDS,
                       tables={str(n): jsonable(r) for n, r in results.items()}), f, indent=1)
    page = os.path.join(os.path.dirname(__file__), "chartpage.html")
    if os.path.exists(page):
        from .chartpage import render
        with open(os.path.join(a.out, "bid_chart.html"), "w") as f:
            f.write(render(os.path.join(a.out, "bid_chart.json"), full=True))
    for name, rows in (("bid_chart.csv", rows_csv), ("bid_chart_simple.csv", rows_q)):
        with open(os.path.join(a.out, name), "w", newline="") as f:
            w = csv.DictWriter(f, fieldnames=["players", "trump", "cards", "lo", "hi", "kind", "value"])
            w.writeheader()
            w.writerows(rows)
    print(f"wrote bid_chart.md / .html / .json / .csv, bid_cheat_sheet.md and bid_chart_simple.csv to {a.out}")


if __name__ == "__main__":
    main()
