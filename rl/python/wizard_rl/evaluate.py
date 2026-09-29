"""Measure a trained network against other players.

The network plays one seat per round (greedy, no exploration); everyone else is the same kind
of opponent: counting bots, random bots, or a fixed reference network. Deals are **duplicated**:
each deal is replayed with the network in every seat, so it holds exactly the cards its
opponents held and luck cancels out.

The headline number is its **edge**: its average round score minus its opponents' average, over
the same deals. Two copies of the same player have an edge of 0.
"""

from __future__ import annotations

import numpy as np
import torch

from . import WizardEnv
from .net import QNet, pick_actions


@torch.no_grad()
def evaluate(net: QNet, device: torch.device, rounds: int = 20_000, players=(3, 4, 5, 6), opponents: str = "counting",
             reference: QNet | None = None, seed: int = 12345, tables: int = 256) -> dict:
    if opponents == "nets":
        assert reference is not None, "a reference network is needed to play against"
    env = WizardEnv(tables, list(players), opponents, seed, True)
    if reference is not None:
        env.set_nets(1)
        reference = reference.to(device).eval()
    env.stats()
    tot = dict(learner_rounds=0, learner_score=0, learner_bids_made=0, other_rounds=0, other_score=0, other_bids_made=0)
    while tot["learner_rounds"] < rounds:
        obs, legal, owner = env.observe()
        o = torch.from_numpy(obs).to(device)
        q = torch.empty(len(obs), legal.shape[1])
        mine = owner == 0
        if mine.any():
            q[torch.from_numpy(mine)] = net(o[torch.from_numpy(mine).to(device)]).cpu()
        if (~mine).any():
            q[torch.from_numpy(~mine)] = reference(o[torch.from_numpy(~mine).to(device)]).cpu()
        env.step(pick_actions(q, torch.from_numpy(legal), 0.0).numpy().astype(np.int64))
        env.drain()
        for k, v in env.stats().items():
            if k in tot:
                tot[k] += v
    lr, orr = tot["learner_rounds"], max(1, tot["other_rounds"])
    learner = tot["learner_score"] / lr
    others = tot["other_score"] / orr
    return dict(
        rounds=lr,
        learner_avg=learner,
        other_avg=others,
        edge=learner - others,
        learner_bid_rate=tot["learner_bids_made"] / lr,
        other_bid_rate=tot["other_bids_made"] / orr,
    )
