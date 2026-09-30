"""The Q-network: for one seat's view of the table, a predicted round score for every action,
and (optionally) the chance of making its bid after each action."""

from __future__ import annotations

import torch
from torch import nn


class QNet(nn.Module):
    """A plain MLP. Output k is the expected round score (in units of `scale` points) of taking
    action k here and playing on as usual. With `make_head`, outputs `actions + k` are logits of
    the chance of making the bid after action k (a second, easier-to-read opinion of the same
    situation; it's trained alongside but doesn't choose moves)."""

    def __init__(self, features: int, actions: int, hidden: int = 512, layers: int = 3, make_head: bool = True):
        super().__init__()
        mods = []
        width = features
        for _ in range(layers):
            mods += [nn.Linear(width, hidden), nn.ReLU()]
            width = hidden
        mods.append(nn.Linear(width, actions * (2 if make_head else 1)))
        self.body = nn.Sequential(*mods)
        self.features, self.actions, self.hidden, self.layers, self.make_head = features, actions, hidden, layers, make_head

    def forward(self, obs: torch.Tensor) -> torch.Tensor:
        """Expected score of every action, `[B, actions]`. An older network (fewer features)
        reads the leading features only: the game features were appended at the end."""
        return self.body(obs[:, : self.features])[:, : self.actions]

    def both(self, obs: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor | None]:
        out = self.body(obs[:, : self.features])
        return out[:, : self.actions], (out[:, self.actions :] if self.make_head else None)

    def config(self) -> dict:
        return dict(features=self.features, actions=self.actions, hidden=self.hidden, layers=self.layers, make_head=self.make_head)


def load_qnet(path: str) -> QNet:
    """A network from a training checkpoint."""
    ck = torch.load(path, map_location="cpu")
    cfg = dict(ck["net"])
    cfg.setdefault("make_head", False)  # checkpoints from before the make-bid head
    net = QNet(**cfg)
    net.load_state_dict(ck["model"])
    net.eval()
    return net


def warm_start(net: QNet, path: str) -> None:
    """Copy weights from an earlier checkpoint with the same depth. Where shapes differ, the
    overlap is copied: the score half of the last layer carries over even if the old network had
    no make-bid head, and inputs the old network didn't have (the game features) start with zero
    weight, so the new network begins by playing exactly like the old one."""
    old = load_qnet(path)
    src, dst = old.body, net.body
    assert len(src) == len(dst), "different depth"
    with torch.no_grad():
        for i, (a, b) in enumerate(zip(src, dst)):
            if isinstance(a, nn.Linear):
                r, c = min(a.weight.shape[0], b.weight.shape[0]), min(a.weight.shape[1], b.weight.shape[1])
                if i == 0 and b.weight.shape[1] > a.weight.shape[1]:
                    b.weight[:, c:].zero_()
                b.weight[:r, :c].copy_(a.weight[:r, :c])
                b.bias[:r].copy_(a.bias[:r])


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


class TwoBrain(nn.Module):
    """A PPO policy that chooses plus the DMC points network that scores (as in a two-brain
    `.wznet`). Calling it gives the policy's logits, so "the best legal output" is the move the
    bot plays; `both` and `points` give the points network's scores, for anything that needs
    expected points (the advisor, move costs)."""

    def __init__(self, policy: nn.Module, evaluator: QNet):
        super().__init__()
        self.policy, self.evaluator = policy, evaluator
        self.features = max(policy.features, evaluator.features)

    def forward(self, obs: torch.Tensor) -> torch.Tensor:
        return self.policy(obs)[0]

    def both(self, obs: torch.Tensor):
        return self.evaluator.both(obs)

    def points(self, obs: torch.Tensor) -> torch.Tensor:
        return self.evaluator(obs)


def load_brain(path: str, evaluator: str | None = None) -> nn.Module:
    """A DMC network, or (for a PPO checkpoint) a TwoBrain of it and `evaluator`."""
    ck = torch.load(path, map_location="cpu", weights_only=False)
    if "pi.weight" not in ck["model"]:
        return load_qnet(path)
    if not evaluator:
        raise SystemExit(f"{path} is a PPO policy: give --evaluator DMC.pt (the points network it was anchored to)")
    from .ppo import PolicyNet
    pol = PolicyNet(**ck["net"])
    pol.load_state_dict(ck["model"])
    return TwoBrain(pol.eval(), load_qnet(evaluator)).eval()
