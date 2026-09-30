"""The mistake chart: which departures from the bot's choice cost the most, in points and in
chance of winning the game.

    python -m wizard_rl.mistakes models/simul1.pt --winprob models/winprob.pt --out charts

Full games are played by a human-like player: every seat picks among its options with
probabilities softmax(expected points / T) from the network (`--temp`, default 3; the strength
curve in the README shows what T means). At every decision, each option other than the
network's favourite is described the way a player would put it,

    bids:  "bid 1 more than the bot" (by hand size)
    plays: situation + "bot: duck high, you: trump in" (the play-chart words, see playchart.py)

and costed two ways: expected points lost this round (the network's own estimate), and chance of
winning the game lost (your margins with each option's expected points, through the
win-probability model; the other players' rounds taken as even, so it's a rough reading).
The chart lists the habits that cost a human-like player the most over a game (how likely they
pick it x what it costs), and the biggest mistakes whether or not people make them.
"""

from __future__ import annotations

import argparse
import json
import os
from collections import defaultdict

import numpy as np
import torch

from . import ACT_BID, GAME, PHASE, WizardEnv
from .net import load_qnet
from .playchart import SIZE, describe
from .winprob import load_winprob

SCORE_SCALE = 200.0


def margins(o: np.ndarray, n: int):
    """(rounds left after this one, margin over the best other, over the second best) from the
    game features of one observation."""
    scores = np.rint(o[GAME + 1:GAME + 1 + n] * SCORE_SCALE)
    others = np.sort(scores[1:])[::-1]
    d1 = scores[0] - others[0]
    d2 = scores[0] - (others[1] if len(others) > 1 else others[0])
    left = int(round(o[GAME + 8] * 20))
    return left, d1, d2


def size_band(size: int) -> str:
    return "1-3 cards" if size <= 3 else "4-7 cards" if size <= 7 else "8-12 cards" if size <= 12 else "13+ cards"


def bid_mistake(o, best: int, chosen: int):
    size = int(round(o[SIZE] * 20))
    b, c = best - ACT_BID, chosen - ACT_BID
    d = c - b
    what = f"bid {abs(d)} {'more' if d > 0 else 'fewer'} than the bot" if abs(d) < 2 else \
        f"bid 2+ {'more' if d > 0 else 'fewer'} than the bot"
    if b == 0:
        what += " (bot bids 0)"
    elif c == 0:
        what += " (you bid 0)"
    return ("bid", size_band(size)), what


def situation(r) -> str:
    if r["kind"] == "bid":
        return r["situation"][0]
    need, seat, trick, follow = r["situation"]
    need = {"need all": "need every trick left", "need more": "need more tricks", "made it": "made your bid",
            "over": "already over"}[need]
    if seat == "lead":
        return f"{need}, you lead"
    return f"{need}, {'last to play' if seat == 'last' else 'mid-trick'}, {trick}, {follow}"


def section(title, rows, top):
    lines = [f"### {title}", "", "| Situation | Mistake | Points | Win chance | Per game | Times it was an option |",
             "| --- | --- | ---: | ---: | ---: | ---: |"]
    for r in rows[:top]:
        lines.append(f"| {situation(r)} | {r['mistake']} | −{r['points']:.1f} | −{r['win']:.2f}% | "
                     f"−{r['per_game']:.1f} | {r['count']:,} |")
    return lines + [""]


@torch.no_grad()
def collect(net, wp, players: int, games: int, temp: float, seed: int = 21):
    """Every option at every decision of human-like full games, against the bot's favourite.
    Per (situation, mistake): [times it was an option, points lost, win chance lost, and the same
    two weighted by the chance the human-like player picks it]."""
    env = WizardEnv(256, [players], "selfplay", seed, False, True, False, True, 1.0)
    rows = defaultdict(lambda: np.zeros(5))
    decisions = {"bid": 0, "play": 0}
    lost = {"bid": np.zeros(2), "play": np.zeros(2)}
    gen = torch.Generator().manual_seed(seed)
    done = 0
    while done < games:
        obs, legal, _ = env.observe()
        o_t, l_t = torch.from_numpy(obs), torch.from_numpy(legal)
        pts = (net(o_t) * 100.0).masked_fill(~l_t, float("-inf"))
        prob = torch.softmax(pts / temp, 1)
        act = torch.multinomial(prob, 1, generator=gen).squeeze(1)
        best = pts.argmax(1)
        pts_np, prob_np = pts.numpy(), prob.numpy()
        for i in np.flatnonzero(legal.sum(1) > 1):
            o = obs[i]
            if o[PHASE + 1] == 1:
                kind = "bid"
            elif o[PHASE + 2] == 1:
                kind = "play"
            else:
                continue
            decisions[kind] += 1
            b = int(best[i])
            alts = [int(c) for c in np.flatnonzero(legal[i]) if c != b]
            cost = pts_np[i, b] - pts_np[i, alts]
            left, d1, d2 = margins(o, players)
            vals = np.concatenate([[pts_np[i, b]], pts_np[i, alts]])
            k = len(vals)
            pw = wp.prob([players] * k, [left] * k, d1 + vals, d2 + vals).numpy()
            wcost = 100.0 * (pw[0] - pw[1:])
            w = prob_np[i, alts]
            if kind == "play":
                sit, bot = describe(o, b, legal[i])
            for c, x, y, q in zip(alts, cost, wcost, w):
                if kind == "bid":
                    key, what = bid_mistake(o, b, c)
                else:
                    you = describe(o, c, legal[i])[1]
                    key = ("play", sit["need"], sit["seat"], sit["trick"], sit["follow"])
                    what = f"bot: {bot} / you: {you}" if bot != you else f"{bot}, a different card than the bot"
                rows[(key, what)] += (1, x, y, q * x, q * y)
            lost[kind] += (float((w * cost).sum()), float((w * wcost).sum()))
        env.step(act.numpy())
        env.drain()
        done += env.stats()["games"]  # taken (reset) on each read
    return rows, decisions, lost, done


def to_markdown(data: dict, top: int = 25) -> str:
    md = ["# Wizard mistake chart", "",
          f"What departing from `{data['model']}`'s choice costs, for a human-like player "
          f"(picks with probabilities softmax(expected points / {data['temp']:g})), in full games with everyone "
          "bidding at once. **Points**: expected points lost that round, as the network sees it. "
          "**Win chance**: chance of winning the game lost (a rough reading, see `mistakes.py`). "
          "**Per game**: what it costs the human-like player over a whole game (how likely they are "
          "to pick it × what it costs, summed over the game); the habit lists are sorted by it. Every "
          "option at every decision is costed, so rare blunders show up too.", ""]
    for n, t in data["tables"].items():
        lp, table = t["lost_per_game"], t["mistakes"]
        md += [f"## {n} players", "",
               f"About {t['games']:,} games. A player like this gives up about **{lp['bid']['points']:.0f} points a "
               f"game in bidding** and **{lp['play']['points']:.0f} in card play**, against playing the bot's "
               "choice every time.", ""]
        bids = [r for r in table if r["kind"] == "bid" and r["count"] >= 200]
        md += section("Bidding (sorted by what it costs per game)", bids, top)
        plays = [r for r in table if r["kind"] == "play"]
        same = [r for r in plays if "a different card" in r["mistake"]]
        md += section("Card play: the costliest habits", [r for r in plays if r not in same], top)
        common = max(50, sum(r["count"] for r in plays) // 1000)
        big = sorted((r for r in plays if r["count"] >= common and r not in same), key=lambda r: -r["points"])
        md += section(f"Card play: the biggest mistakes, whether or not people make them (an option at "
                      f"least {common:,} times)", big, top)
        sp = sum(r["per_game"] for r in same)
        md += [f"Picking a different card of the same kind as the bot (another low off-suit lead, another "
               f"low card to duck with) adds up to {sp:.0f} points a game. Those are mostly small differences "
               "(one or two points each), where the network's own estimates are least sure.", ""]
    return "\n".join(md) + "\n"


def main(argv=None) -> None:
    import sys
    argv = sys.argv[1:] if argv is None else argv
    if argv and argv[0] == "render":  # rewrite the markdown from an existing mistake_chart.json
        with open(argv[1]) as f:
            data = json.load(f)
        with open(os.path.splitext(argv[1])[0] + ".md", "w") as f:
            f.write(to_markdown(data))
        return
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("model")
    p.add_argument("--winprob", default="models/winprob.pt")
    p.add_argument("--players", default="4")
    p.add_argument("--games", type=int, default=1000, help="full games per table size")
    p.add_argument("--temp", type=float, default=3.0, help="how loosely the human-like player picks")
    p.add_argument("--top", type=int, default=25)
    p.add_argument("--out", default="charts")
    a = p.parse_args(argv)
    net = load_qnet(a.model).eval()
    wp = load_winprob(a.winprob)
    os.makedirs(a.out, exist_ok=True)
    out = {}
    for n in (int(x) for x in a.players.split(",")):
        rows, decisions, lost, games = collect(net, wp, n, a.games, a.temp)
        seat_games = decisions["bid"] / (60 // n)  # one bid per seat and round
        table = []
        for (key, what), (cnt, pts, wc, wpts, wwc) in rows.items():
            table.append(dict(kind=key[0], situation=list(key[1:]), mistake=what, count=int(cnt),
                              points=pts / cnt, win=wc / cnt, per_game=wpts / seat_games,
                              win_per_game=wwc / seat_games))
        # Rare (situation, mistake) pairs are noise; keep the rest, rounded.
        table = [{k: round(v, 3) if isinstance(v, float) else v for k, v in r.items()}
                 for r in table if r["count"] >= 100]
        table.sort(key=lambda r: -r["per_game"])
        out[str(n)] = dict(games=round(seat_games / n), decisions=decisions,
                           lost_per_game={k: dict(points=float(v[0]) / seat_games, win=float(v[1]) / seat_games)
                                          for k, v in lost.items()},
                           mistakes=table)
        print(f"{n} players: {games:,} games; lost per game {out[str(n)]['lost_per_game']}", flush=True)
    data = dict(model=os.path.basename(a.model), temp=a.temp, tables=out)
    with open(os.path.join(a.out, "mistake_chart.json"), "w") as f:
        json.dump(data, f, indent=1)
    with open(os.path.join(a.out, "mistake_chart.md"), "w") as f:
        f.write(to_markdown(data, a.top))
    print(f"wrote {a.out}/mistake_chart.md and .json")


if __name__ == "__main__":
    main()
