"""Deep Monte Carlo (the DouZero method): play, then pull each decision's predicted score
toward the score its seat actually got for the round.

    loss = (Q(s, a) - G)^2

Works with any batch environment that has `observe() -> (obs, legal)`, `step(actions)` and
`drain() -> (obs, actions, returns)`: the Rust Wizard tables, or the Kuhn poker check.
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field

import numpy as np
import torch

from .net import QNet, pick_actions


@dataclass
class LoopConfig:
    batch: int = 4096            # samples per gradient step
    lr: float = 3e-4
    lr_final: float | None = None  # if set, decay linearly to this ...
    lr_decay: int = 0              # ... over this many decisions
    eps_start: float = 0.2       # exploration at the start ...
    eps_end: float = 0.02        # ... decaying to this
    eps_decay: int = 20_000_000  # over this many decisions
    scale: float = 100.0         # returns are divided by this before fitting
    max_grad_norm: float = 10.0


@dataclass
class LoopState:
    decisions: int = 0
    samples: int = 0
    updates: int = 0
    last_loss: float = float("nan")
    buf_obs: list = field(default_factory=list)
    buf_act: list = field(default_factory=list)
    buf_ret: list = field(default_factory=list)
    buffered: int = 0


class Learner:
    def __init__(self, env, net: QNet, cfg: LoopConfig, device: torch.device, seed: int = 0):
        self.env, self.net, self.cfg, self.device = env, net.to(device), cfg, device
        self.opt = torch.optim.Adam(self.net.parameters(), lr=cfg.lr)
        self.state = LoopState()
        self.gen = torch.Generator(device="cpu")
        self.gen.manual_seed(seed)

    def epsilon(self) -> float:
        c = self.cfg
        f = min(1.0, self.state.decisions / max(1, c.eps_decay))
        return c.eps_start + f * (c.eps_end - c.eps_start)

    @torch.no_grad()
    def act(self) -> None:
        obs, legal = self.env.observe()
        q = self.net(torch.from_numpy(obs).to(self.device)).cpu()
        a = pick_actions(q, torch.from_numpy(legal), self.epsilon(), self.gen)
        self.env.step(a.numpy().astype(np.int64))
        self.state.decisions += len(a)
        o, act, ret = self.env.drain()
        if len(act):
            s = self.state
            s.buf_obs.append(o)
            s.buf_act.append(act)
            s.buf_ret.append(ret)
            s.buffered += len(act)

    def learn(self) -> None:
        s, c = self.state, self.cfg
        while s.buffered >= c.batch:
            obs = np.concatenate(s.buf_obs)
            act = np.concatenate(s.buf_act)
            ret = np.concatenate(s.buf_ret)
            # Shuffle so a batch mixes many rounds and tables.
            perm = np.random.default_rng(s.updates).permutation(len(act))
            obs, act, ret = obs[perm], act[perm], ret[perm]
            take = c.batch
            self._update(obs[:take], act[:take], ret[:take])
            s.buf_obs, s.buf_act, s.buf_ret = [obs[take:]], [act[take:]], [ret[take:]]
            s.buffered = len(act) - take

    def _set_lr(self) -> None:
        c = self.cfg
        if c.lr_final is None or c.lr_decay <= 0:
            return
        f = min(1.0, self.state.decisions / c.lr_decay)
        for g in self.opt.param_groups:
            g["lr"] = c.lr + f * (c.lr_final - c.lr)

    def _update(self, obs, act, ret) -> None:
        self._set_lr()
        o = torch.from_numpy(obs).to(self.device)
        a = torch.from_numpy(act).to(self.device)
        g = torch.from_numpy(ret).to(self.device) / self.cfg.scale
        q = self.net(o).gather(1, a.unsqueeze(1)).squeeze(1)
        loss = torch.nn.functional.mse_loss(q, g)
        self.opt.zero_grad(set_to_none=True)
        loss.backward()
        torch.nn.utils.clip_grad_norm_(self.net.parameters(), self.cfg.max_grad_norm)
        self.opt.step()
        self.state.updates += 1
        self.state.samples += len(act)
        self.state.last_loss = float(loss.detach().cpu())

    def run(self, decisions: int | None = None, seconds: float | None = None, on_tick=None, tick_every: int = 1_000_000) -> None:
        """Play and learn until `decisions` more decisions or `seconds` pass. `on_tick(learner)`
        is called about every `tick_every` decisions."""
        start_d, start_t = self.state.decisions, time.time()
        next_tick = self.state.decisions + tick_every
        while True:
            self.act()
            self.learn()
            if on_tick and self.state.decisions >= next_tick:
                on_tick(self)
                next_tick += tick_every
            if decisions is not None and self.state.decisions - start_d >= decisions:
                break
            if seconds is not None and time.time() - start_t >= seconds:
                break
