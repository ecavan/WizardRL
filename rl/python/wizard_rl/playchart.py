"""Play charts: how the bot plays its cards after bidding (the "post-flop" of Wizard).

    python -m wizard_rl.playchart models/simul1.pt --players 4 --out charts

The bot plays itself; every card it plays is described the way a player would think of it:

    Wizard | Jester | lead: trump high/low, off-suit high/low
    win cheaply (the lowest card that takes the trick) | win big (a stronger winner than needed)
    duck high (the highest card that still loses) | duck low (a lower losing card)

and tagged with the situation: do you still need tricks (and do you need every remaining one),
have you made your bid exactly, or are you already over; are you leading, in the middle, or last
to play; what's winning the trick so far; can you follow suit. The chart is, for each situation,
how often the bot makes each kind of play.
"""

from __future__ import annotations

import argparse
import json
import os
from collections import Counter, defaultdict

import numpy as np
import torch

from . import ACT_CARD, HAND, PHASE, WizardEnv
from .net import load_qnet

# Observation layout (see crates/wizard/src/encode.rs)
SEATS, DECK = 6, 60
PLAYERS = PHASE + 3
SIZE = PLAYERS + 4
TRICKS_LEFT = SIZE + 1
FROM_DEALER = TRICKS_LEFT + 1
TRUMP = FROM_DEALER + SEATS
TURNED = TRUMP + 5
PLAYED = HAND + DECK
TRICK = PLAYED + DECK
LED = TRICK + (SEATS - 1) * DECK
WINNING = LED + 5
BIDS = WINNING + SEATS
HAS_BID = BIDS + SEATS
WON = HAS_BID + SEATS
NEED = WON + SEATS
VOIDS = NEED + SEATS
TRICK_POS = VOIDS + SEATS * 4
assert HAND == 23 and TRICK_POS == 502

WIZ, JES = range(52, 56), range(56, 60)
RANK = "23456789TJQKA"


def is_wiz(c): return 52 <= c < 56
def is_jes(c): return 56 <= c < 60
def suit(c): return c // 13 if c < 52 else None
def rank(c): return c % 13 if c < 52 else None


def strength(c, trump):
    if is_wiz(c):
        return 100
    if is_jes(c):
        return 0
    return 1 + rank(c) + (40 if suit(c) == trump else 0)


def beats(c, w, trump):
    """Would card c take the trick from the card w currently winning it?"""
    if is_wiz(w):
        return False
    if is_wiz(c):
        return True
    if is_jes(c):
        return False
    if is_jes(w):  # only Jesters so far: the first standard card wins
        return True
    if suit(c) == suit(w):
        return rank(c) > rank(w)
    return suit(c) == trump


def describe(o: np.ndarray, action: int, legal: np.ndarray):
    """(situation, play) for one card-play decision from its observation."""
    n = int(np.argmax(o[PLAYERS:PLAYERS + 4])) + 3
    t = int(np.argmax(o[TRUMP:TRUMP + 5]))
    trump = t if t < 4 else None
    size = int(round(o[SIZE] * 20))
    left = int(round(o[TRICKS_LEFT] * 20))
    bid = int(round(o[BIDS] * 20))
    won = int(round(o[WON] * 20))
    need = bid - won
    in_trick = int(round(o[TRICK_POS] * 5))
    card = action - ACT_CARD
    hand = np.flatnonzero(o[HAND:HAND + DECK] > 0)
    legal_cards = [a - ACT_CARD for a in np.flatnonzero(legal) if a >= ACT_CARD]

    if need > 0:
        ns = "need all" if need >= left else "need more"
    elif need == 0:
        ns = "made it"
    else:
        ns = "over"
    pos = "lead" if in_trick == 0 else ("last" if in_trick == n - 1 else "middle")

    if in_trick == 0:
        if is_wiz(card):
            play = "Wizard"
        elif is_jes(card):
            play = "Jester"
        elif suit(card) == trump:
            play = "lead trump high (J+)" if rank(card) >= 9 else "lead trump low"
        else:
            play = "lead off-suit high (K+)" if rank(card) >= 11 else "lead off-suit low"
        sit = dict(need=ns, seat=pos, trick="—", follow="—")
        return sit, play

    # The card currently winning: the card held by the winning relative seat.
    wseat = int(np.argmax(o[WINNING:WINNING + SEATS]))
    wcards = np.flatnonzero(o[TRICK + (wseat - 1) * DECK:TRICK + wseat * DECK] > 0)
    w = int(wcards[0]) if len(wcards) else 56
    led = int(np.argmax(o[LED:LED + 5]))
    if is_wiz(w):
        trick = "a Wizard is winning"
    elif is_jes(w):
        trick = "only Jesters so far"
    elif suit(w) == trump:
        trick = "trump is winning"
    else:
        trick = "off-suit is winning"
    can_follow = led < 4 and any(suit(c) == led for c in hand)
    follow = "can follow" if can_follow else ("no suit to follow" if led == 4 else "can't follow")

    winners = [c for c in legal_cards if beats(c, w, trump)]
    losers = [c for c in legal_cards if not beats(c, w, trump)]
    if is_wiz(card):
        play = "Wizard"
    elif is_jes(card):
        play = "Jester"
    elif card in winners:
        plain = [c for c in winners if not is_wiz(c)]
        play = "win cheaply" if strength(card, trump) == min(strength(c, trump) for c in plain) else "win big"
        if suit(card) == trump and led != trump and led < 4:
            play += " (trump in)"
    else:
        plain = [c for c in losers if not is_jes(c)]
        play = "duck high" if strength(card, trump) == max(strength(c, trump) for c in plain) else "duck low"
    has_winner = any(not is_wiz(c) for c in winners)
    sit = dict(need=ns, seat=pos, trick=trick, follow=follow + ("" if has_winner else ", nothing but a Wizard wins"))
    return sit, play


@torch.no_grad()
def collect(net, players: int, decisions: int, seed: int = 11):
    env = WizardEnv(256, [players], "selfplay", seed)
    out = []
    while len(out) < decisions:
        obs, legal, _ = env.observe()
        q = net(torch.from_numpy(obs)).masked_fill(~torch.from_numpy(legal), float("-inf"))
        act = q.argmax(1).numpy()
        playing = obs[:, PHASE + 2] == 1
        for i in np.flatnonzero(playing):
            if legal[i].sum() > 1:  # only real choices
                out.append(describe(obs[i], int(act[i]), legal[i]))
        env.step(act)
        env.drain()
    return out


def summarize(rows, min_count: int = 300):
    groups = defaultdict(Counter)
    for sit, play in rows:
        groups[tuple(sit.items())][play] += 1
    table = []
    for key, c in groups.items():
        total = sum(c.values())
        if total < min_count:
            continue
        table.append(dict(situation=dict(key), n=total, plays={k: v / total for k, v in c.most_common()}))
    order = {"need all": 0, "need more": 1, "made it": 2, "over": 3}
    seat = {"lead": 0, "middle": 1, "last": 2}
    table.sort(key=lambda r: (order[r["situation"]["need"]], seat[r["situation"]["seat"]], -r["n"]))
    return table


def to_markdown(players, table, total):
    lines = [f"## {players} players", "",
             f"{total:,} real choices (more than one legal card). Each row: a situation, and how often "
             "the bot makes each kind of play there (the most common first).", ""]
    cur = None
    for r in table:
        s = r["situation"]
        if s["need"] != cur:
            cur = s["need"]
            title = {"need all": "You need every trick that's left", "need more": "You still need tricks",
                     "made it": "You've made your bid exactly (every trick from here costs you)",
                     "over": "You're already over your bid"}[cur]
            lines += ["", f"### {title}", "", "| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |",
                      "| --- | --- | --- | --- | ---: |"]
        plays = ", ".join(f"**{k}** {v:.0%}" if i == 0 else f"{k} {v:.0%}" for i, (k, v) in enumerate(r["plays"].items()) if v >= 0.03)
        lines.append(f"| {s['seat']} | {s['trick']} | {s['follow']} | {plays} | {r['n']:,} |")
    return "\n".join(lines)


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("model")
    p.add_argument("--players", default="4")
    p.add_argument("--decisions", type=int, default=300_000, help="card plays to record per table size")
    p.add_argument("--out", default="charts")
    a = p.parse_args(argv)
    net = load_qnet(a.model).eval()
    os.makedirs(a.out, exist_ok=True)
    md = ["# Wizard play chart", "",
          f"How `{os.path.basename(a.model)}` plays its cards after bidding, from self-play with everyone "
          "bidding at once.", "",
          "- **win cheaply**: the lowest card that takes the trick; **win big**: a stronger winner than needed.",
          "- **duck high**: the highest card that still loses (getting rid of danger); **duck low**: a lower loser.",
          "- **trump in**: winning with trump when you can't follow the suit led.", ""]
    data = {}
    for n in (int(x) for x in a.players.split(",")):
        rows = collect(net, n, a.decisions)
        table = summarize(rows)
        data[str(n)] = table
        md.append(to_markdown(n, table, len(rows)))
        print(f"{n} players: {len(rows):,} choices, {len(table)} situations", flush=True)
    with open(os.path.join(a.out, "play_chart.md"), "w") as f:
        f.write("\n".join(md) + "\n")
    with open(os.path.join(a.out, "play_chart.json"), "w") as f:
        json.dump(dict(model=os.path.basename(a.model), tables=data), f, indent=1)
    print(f"wrote {a.out}/play_chart.md and play_chart.json")


if __name__ == "__main__":
    main()
