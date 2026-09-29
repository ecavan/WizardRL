"""Measure a trained network against the baseline bots.

The network plays one random seat per round (greedy, no exploration); bots play the rest.
The headline number is its **edge**: its average round score minus the bots' average round
score in the same rounds. Two counting bots have an edge of 0 against each other.
"""

from __future__ import annotations

import numpy as np
import torch

from . import WizardEnv
from .net import QNet, pick_actions


@torch.no_grad()
def evaluate(net: QNet, device: torch.device, rounds: int = 20_000, players=(3, 4, 5, 6), opponents: str = "counting", seed: int = 12345, tables: int = 256) -> dict:
    env = WizardEnv(tables, list(players), opponents, seed)
    env.stats()
    tot = dict(learner_rounds=0, learner_score=0, learner_bids_made=0, bot_rounds=0, bot_score=0, bot_bids_made=0)
    while tot["learner_rounds"] < rounds:
        obs, legal = env.observe()
        q = net(torch.from_numpy(obs).to(device)).cpu()
        env.step(pick_actions(q, torch.from_numpy(legal), 0.0).numpy().astype(np.int64))
        env.drain()
        for k, v in env.stats().items():
            if k in tot:
                tot[k] += v
    lr, br = tot["learner_rounds"], max(1, tot["bot_rounds"])
    learner = tot["learner_score"] / lr
    bots = tot["bot_score"] / br
    return dict(
        rounds=lr,
        learner_avg=learner,
        bot_avg=bots,
        edge=learner - bots,
        learner_bid_rate=tot["learner_bids_made"] / lr,
        bot_bid_rate=tot["bot_bids_made"] / br,
    )
