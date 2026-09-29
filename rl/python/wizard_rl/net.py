"""The Q-network: for one seat's view of the table, a predicted round score for every action."""

from __future__ import annotations

import torch
from torch import nn


class QNet(nn.Module):
    """A plain MLP. Output k is the expected round score (in units of `scale` points) of taking
    action k here and playing on as usual."""

    def __init__(self, features: int, actions: int, hidden: int = 512, layers: int = 3):
        super().__init__()
        mods = []
        width = features
        for _ in range(layers):
            mods += [nn.Linear(width, hidden), nn.ReLU()]
            width = hidden
        mods.append(nn.Linear(width, actions))
        self.body = nn.Sequential(*mods)
        self.features, self.actions, self.hidden, self.layers = features, actions, hidden, layers

    def forward(self, obs: torch.Tensor) -> torch.Tensor:
        return self.body(obs)

    def config(self) -> dict:
        return dict(features=self.features, actions=self.actions, hidden=self.hidden, layers=self.layers)


def pick_actions(q: torch.Tensor, legal: torch.Tensor, epsilon: float, generator: torch.Generator | None = None) -> torch.Tensor:
    """Best legal action per row; with probability `epsilon` a uniformly random legal one."""
    masked = q.masked_fill(~legal, float("-inf"))
    greedy = masked.argmax(dim=1)
    if epsilon <= 0:
        return greedy
    noise = torch.rand(legal.shape, generator=generator, device=legal.device).masked_fill(~legal, -1.0)
    random_legal = noise.argmax(dim=1)
    explore = torch.rand(q.shape[0], generator=generator, device=q.device) < epsilon
    return torch.where(explore, random_legal, greedy)


def best_device() -> torch.device:
    if torch.cuda.is_available():
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")
