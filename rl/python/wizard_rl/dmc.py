"""Deep Monte Carlo (the DouZero method): play, then pull each decision's predicted score
toward the score its seat actually got for the round.

    loss = (Q(s, a) - G)^2  +  w * BCE(P_make(s, a), made)

The second term trains the optional make-bid head; it doesn't change how moves are chosen.

Works with any batch environment that has `observe() -> (obs, legal[, owner])`,
`step(actions)` and `drain() -> (obs, actions, returns[, made])`: the Rust Wizard tables, or
the Kuhn poker check. `owner` > 0 marks a decision for frozen network `owner` (an older copy of
the learner), which plays greedily and isn't trained on.
"""

from __future__ import annotations

import copy
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
    make_weight: float = 0.25    # weight of the make-bid loss
    max_grad_norm: float = 10.0


@dataclass
class LoopState:
    decisions: int = 0
    samples: int = 0
    updates: int = 0
    last_loss: float = float("nan")
    last_make_loss: float = float("nan")
    buf: list = field(default_factory=list)
    buffered: int = 0


class Learner:
    def __init__(self, env, net: QNet, cfg: LoopConfig, device: torch.device, seed: int = 0):
        self.env, self.net, self.cfg, self.device = env, net.to(device), cfg, device
        self.opt = torch.optim.Adam(self.net.parameters(), lr=cfg.lr)
        self.state = LoopState()
        self.frozen: list[QNet] = []
        # Optional habit per frozen network (see styles.py); None = plays normally.
        self.styles: list[str | None] = []
        # Optional: turn (drained samples, their game context) into the returns to learn from.
        self.reward_fn = None
        self.freezes = 0
        self.gen = torch.Generator(device="cpu")
        self.gen.manual_seed(seed)

    # ------------------------------------------------------------------ frozen opponents

    def freeze(self, max_pool: int) -> int:
        """Add a frozen copy of the current network to the opponent pool (replacing the oldest
        when full). Returns the pool size."""
        snap = copy.deepcopy(self.net).eval()
        for p in snap.parameters():
            p.requires_grad_(False)
        if len(self.frozen) < max_pool:
            self.frozen.append(snap)
        else:
            self.frozen[self.freezes % max_pool] = snap  # round-robin: replaces the oldest
        self.freezes += 1
        if hasattr(self.env, "set_nets"):
            self.env.set_nets(len(self.frozen))
        return len(self.frozen)

    # ------------------------------------------------------------------ playing

    def epsilon(self) -> float:
        c = self.cfg
        f = min(1.0, self.state.decisions / max(1, c.eps_decay))
        return c.eps_start + f * (c.eps_end - c.eps_start)

    @torch.no_grad()
    def act(self) -> None:
        got = self.env.observe()
        obs, legal = got[0], got[1]
        owner = got[2] if len(got) > 2 else np.zeros(len(obs), dtype=np.int64)
        o = torch.from_numpy(obs).to(self.device)
        lg = torch.from_numpy(legal)
        actions = torch.zeros(len(obs), dtype=torch.int64)
        mine = owner == 0
        if mine.any():
            idx = torch.from_numpy(np.flatnonzero(mine))
            q = self.net(o[idx.to(self.device)]).cpu()
            actions[idx] = pick_actions(q, lg[idx], self.epsilon(), self.gen)
            self.state.decisions += int(mine.sum())
        for k in np.unique(owner[~mine]):
            idx = torch.from_numpy(np.flatnonzero(owner == k))
            q = self.frozen[int(k) - 1](o[idx.to(self.device)]).cpu()
            style = self.styles[int(k) - 1] if int(k) <= len(self.styles) else None
            if style and style.startswith("soft"):
                # a strong but imperfect player: moves drawn in proportion to exp(points / T)
                temp = float(style[4:] or 10)
                logits = (q * self.cfg.scale / temp).masked_fill(~lg[idx], float("-inf"))
                actions[idx] = torch.multinomial(torch.softmax(logits, 1), 1, generator=self.gen).squeeze(1)
                continue
            actions[idx] = pick_actions(q, lg[idx], 0.0)
            if style:
                from .styles import apply_style
                actions[idx] = apply_style(style, actions[idx], o[idx.to(self.device)], lg[idx], self.gen)
        self.env.step(actions.numpy())
        d = self.env.drain()
        if len(d[1]):
            made = d[3] if len(d) > 3 else np.zeros(len(d[1]), dtype=np.float32)
            ret = d[2] if self.reward_fn is None else self.reward_fn(d, self.env.last_context())
            self.state.buf.append((d[0], d[1], ret, made))
            self.state.buffered += len(d[1])

    # ------------------------------------------------------------------ learning

    def learn(self) -> None:
        s, c = self.state, self.cfg
        while s.buffered >= c.batch:
            obs, act, ret, made = (np.concatenate([b[i] for b in s.buf]) for i in range(4))
            # Shuffle so a batch mixes many rounds and tables.
            perm = np.random.default_rng(s.updates).permutation(len(act))
            obs, act, ret, made = obs[perm], act[perm], ret[perm], made[perm]
            take = c.batch
            self._update(obs[:take], act[:take], ret[:take], made[:take])
            s.buf = [(obs[take:], act[take:], ret[take:], made[take:])]
            s.buffered = len(act) - take

    def _set_lr(self) -> None:
        c = self.cfg
        if c.lr_final is None or c.lr_decay <= 0:
            return
        f = min(1.0, self.state.decisions / c.lr_decay)
        for g in self.opt.param_groups:
            g["lr"] = c.lr + f * (c.lr_final - c.lr)

    def _update(self, obs, act, ret, made) -> None:
        self._set_lr()
        o = torch.from_numpy(obs).to(self.device)
        a = torch.from_numpy(act).to(self.device).unsqueeze(1)
        g = torch.from_numpy(ret).to(self.device) / self.cfg.scale
        q, make_logits = self.net.both(o)
        loss = torch.nn.functional.mse_loss(q.gather(1, a).squeeze(1), g)
        total = loss
        if make_logits is not None and self.cfg.make_weight > 0:
            m = torch.from_numpy(made).to(self.device)
            make_loss = torch.nn.functional.binary_cross_entropy_with_logits(make_logits.gather(1, a).squeeze(1), m)
            total = loss + self.cfg.make_weight * make_loss
            self.state.last_make_loss = float(make_loss.detach().cpu())
        self.opt.zero_grad(set_to_none=True)
        total.backward()
        torch.nn.utils.clip_grad_norm_(self.net.parameters(), self.cfg.max_grad_norm)
        self.opt.step()
        self.state.updates += 1
        self.state.samples += len(act)
        self.state.last_loss = float(loss.detach().cpu())

    def run(self, decisions: int | None = None, seconds: float | None = None, on_tick=None, tick_every: int = 1_000_000) -> None:
        """Play and learn until `decisions` more learner decisions or `seconds` pass.
        `on_tick(learner)` is called about every `tick_every` decisions."""
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
