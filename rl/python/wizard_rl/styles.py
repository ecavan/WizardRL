"""Player styles for opponent seats (the same habits as the Rust `style:` bots).

A styled opponent is a network that plays normally except for one habit:

- `overbid`: bids one more than the network would (70% of the time)
- `underbid`: bids one fewer (70%)
- `early-wizard`: throws a Wizard on a round's first trick whenever it can (90%)
- `wild`: 20% of its decisions are random
- `plain`: no habit
- `soft10` (any number): no habit, but picks moves at random in proportion to exp(points / 10),
  so it often takes a nearly-best move and rarely a bad one: a strong, imperfect, human-like
  player (training only; for evaluation see players.py)

Training against a mix of them (`train --styles ...`, in full games) teaches the bot to notice a
habit from how a player has played so far this game, and to exploit it.
"""

from __future__ import annotations

import torch

from . import ACT_BID, ACT_CARD, PHASE, SIZE, TRICKS_LEFT, WIZARD_CARDS

STYLES = ("overbid", "underbid", "early-wizard", "wild", "plain")


def apply_style(style: str, actions: torch.Tensor, obs: torch.Tensor, legal: torch.Tensor,
                gen: torch.Generator | None = None) -> torch.Tensor:
    """The styled version of `actions` (a batch of chosen action indices, on the CPU)."""
    if style == "plain":
        return actions
    obs, legal = obs.cpu(), legal.cpu()
    a = actions.clone()
    u = torch.rand(len(a), generator=gen)
    bidding = obs[:, PHASE + 1] == 1
    size = torch.round(obs[:, SIZE] * 20).long()
    if style in ("overbid", "underbid"):
        is_bid = bidding & (a >= ACT_BID) & (a < ACT_CARD) & (u < 0.7)
        b = a - ACT_BID
        b = torch.minimum(b + 1, size) if style == "overbid" else torch.clamp(b - 1, min=0)
        a = torch.where(is_bid, ACT_BID + b, a)
    elif style == "early-wizard":
        playing = obs[:, PHASE + 2] == 1
        first_trick = torch.isclose(obs[:, TRICKS_LEFT], obs[:, SIZE])
        lo, hi = WIZARD_CARDS
        wiz = legal[:, ACT_CARD + lo:ACT_CARD + hi]
        has = wiz.any(1)
        pick = ACT_CARD + lo + wiz.float().argmax(1)
        a = torch.where(playing & first_trick & has & (u < 0.9), pick, a)
    elif style == "wild":
        noise = torch.rand(legal.shape, generator=gen).masked_fill(~legal, -1.0)
        a = torch.where(u < 0.2, noise.argmax(1), a)
    else:
        raise ValueError(f"unknown style {style!r}; choose from {', '.join(STYLES)}")
    assert bool(legal[torch.arange(len(a)), a].all()), "a style made an illegal move"
    return a
