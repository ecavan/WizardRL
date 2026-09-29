"""Learner check on Kuhn poker, where the right answers are known exactly.

Kuhn poker: a deck of J, Q, K; each player antes 1 and gets one card. Player 1 checks or bets
1; if player 1 checks, player 2 checks (showdown for 1) or bets; facing a bet you fold or call
(showdown for 2).

The learner plays player 1 with the same network and Deep Monte Carlo loop used for Wizard,
against a fixed player 2. We compute player 1's exact best-response values by enumeration and
check that (a) the learned action values match them and (b) the learned greedy strategy wins
what a best response wins:

- against the textbook equilibrium player 2, the best player 1 can do is -1/18 of a chip a hand
  (the game's value);
- against an always-aggressive player 2 (bets whenever checked to, calls every bet), a best
  response exploits it for a clear profit.

"Exact" values assume the learner's own final exploration rate, since its training targets
include its random moves.

Run: python -m wizard_rl.kuhn
"""

from __future__ import annotations

import itertools
import sys

import numpy as np
import torch

from .dmc import Learner, LoopConfig
from .net import QNet, best_device

J, Q, K = 0, 1, 2
NAMES = "JQK"
# Player 1's two decision points: first action, and facing a bet after checking.
FIRST, FACING = 0, 1
FEATURES = 3 + 2  # card one-hot, decision point one-hot
ACTIONS = 2  # 0 = check/fold, 1 = bet/call
MIN_VISITS = 5_000

# Player 2 strategies: probability of betting after a check, and of calling a bet, by card.
EQUILIBRIUM = {"bet": {J: 1 / 3, Q: 0.0, K: 1.0}, "call": {J: 0.0, Q: 1 / 3, K: 1.0}}
AGGRESSIVE = {"bet": {J: 1.0, Q: 1.0, K: 1.0}, "call": {J: 1.0, Q: 1.0, K: 1.0}}


def showdown(c1: int, c2: int, pot_each: int) -> int:
    return pot_each if c1 > c2 else -pot_each


def exact_q(p2: dict, epsilon: float = 0.0) -> dict:
    """Player 1's action values at each (card, decision point) when it plays its best move later,
    except for a random move with probability `epsilon` (the learner's own exploration, which
    its Monte Carlo targets include)."""
    q = {}
    for c1 in (J, Q, K):
        others = [c for c in (J, Q, K) if c != c1]
        # Facing a bet after checking: condition on player 2 having bet.
        w = {c2: 0.5 * p2["bet"][c2] for c2 in others}
        tot = sum(w.values())
        if tot > 0:
            fold = -1.0
            call = sum(w[c2] * showdown(c1, c2, 2) for c2 in others) / tot
            q[(c1, FACING)] = (fold, call)
        f = q.get((c1, FACING), (-1.0, -1.0))
        facing_best = (1 - epsilon) * max(f) + epsilon * sum(f) / 2
        # First decision.
        check = 0.0
        bet = 0.0
        for c2 in others:
            pb = p2["bet"][c2]
            check += 0.5 * ((1 - pb) * showdown(c1, c2, 1) + pb * facing_best)
            pc = p2["call"][c2]
            bet += 0.5 * (pc * showdown(c1, c2, 2) + (1 - pc) * 1.0)
        q[(c1, FIRST)] = (check, bet)
    return q


def policy_value(policy: dict, p2: dict) -> float:
    """Exact expected chips a hand for player 1's deterministic `policy[(card, point)] -> action`."""
    v = 0.0
    for c1, c2 in itertools.permutations((J, Q, K), 2):
        p = 1 / 6
        if policy[(c1, FIRST)] == 1:
            pc = p2["call"][c2]
            v += p * (pc * showdown(c1, c2, 2) + (1 - pc) * 1.0)
        else:
            pb = p2["bet"][c2]
            facing = showdown(c1, c2, 2) if policy.get((c1, FACING), 0) == 1 else -1.0
            v += p * ((1 - pb) * showdown(c1, c2, 1) + pb * facing)
    return v


class KuhnEnv:
    """A batch of Kuhn hands with the same interface as the Wizard tables."""

    def __init__(self, n: int, p2: dict, seed: int = 0):
        self.n, self.p2 = n, p2
        self.rng = np.random.default_rng(seed)
        self.card1 = np.zeros(n, dtype=np.int64)
        self.card2 = np.zeros(n, dtype=np.int64)
        self.point = np.zeros(n, dtype=np.int64)
        self.first_obs = np.zeros((n, FEATURES), dtype=np.float32)
        self.first_act = np.zeros(n, dtype=np.int64)
        self.out = ([], [], [])
        self.visits: dict = {}
        for i in range(n):
            self._deal(i)

    def _deal(self, i):
        c = self.rng.permutation(3)
        self.card1[i], self.card2[i], self.point[i] = c[0], c[1], FIRST

    def _obs(self, i):
        o = np.zeros(FEATURES, dtype=np.float32)
        o[self.card1[i]] = 1.0
        o[3 + self.point[i]] = 1.0
        return o

    def observe(self):
        obs = np.stack([self._obs(i) for i in range(self.n)])
        return obs, np.ones((self.n, ACTIONS), dtype=bool)

    def _finish(self, samples, ret):
        for o, a in samples:
            key = (int(o[:3].argmax()), int(o[3:].argmax()), int(a))
            self.visits[key] = self.visits.get(key, 0) + 1
            self.out[0].append(o)
            self.out[1].append(a)
            self.out[2].append(ret)

    def step(self, actions):
        for i, a in enumerate(actions):
            c1, c2, o = self.card1[i], self.card2[i], self._obs(i)
            if self.point[i] == FIRST:
                if a == 1:  # bet
                    call = self.rng.random() < self.p2["call"][c2]
                    self._finish([(o, 1)], float(showdown(c1, c2, 2) if call else 1))
                    self._deal(i)
                elif self.rng.random() < self.p2["bet"][c2]:  # check, player 2 bets
                    self.first_obs[i], self.first_act[i] = o, 0
                    self.point[i] = FACING
                else:  # check, check
                    self._finish([(o, 0)], float(showdown(c1, c2, 1)))
                    self._deal(i)
            else:
                ret = float(showdown(c1, c2, 2)) if a == 1 else -1.0
                self._finish([(self.first_obs[i].copy(), 0), (o, int(a))], ret)
                self._deal(i)

    def drain(self):
        o, a, r = self.out
        self.out = ([], [], [])
        if not a:
            return np.zeros((0, FEATURES), np.float32), np.zeros(0, np.int64), np.zeros(0, np.float32)
        return np.stack(o).astype(np.float32), np.array(a, dtype=np.int64), np.array(r, dtype=np.float32)


def learned(net: QNet, device) -> tuple[dict, dict]:
    qs, policy = {}, {}
    with torch.no_grad():
        for c1 in (J, Q, K):
            for point in (FIRST, FACING):
                o = torch.zeros(1, FEATURES, device=device)
                o[0, c1] = 1.0
                o[0, 3 + point] = 1.0
                q = net(o)[0].cpu().numpy()
                qs[(c1, point)] = (float(q[0]), float(q[1]))
                policy[(c1, point)] = int(q.argmax())
    return qs, policy


def check(name: str, p2: dict, decisions: int = 2_000_000, seed: int = 0, verbose: bool = True) -> bool:
    torch.manual_seed(seed)
    device = best_device() if torch.cuda.is_available() else torch.device("cpu")
    env = KuhnEnv(256, p2, seed)
    net = QNet(FEATURES, ACTIONS, hidden=64, layers=2)
    cfg = LoopConfig(batch=1024, lr=1e-3, lr_final=1e-4, lr_decay=decisions, eps_start=0.3, eps_end=0.1, eps_decay=decisions // 2, scale=1.0)
    Learner(env, net, cfg, device, seed).run(decisions=decisions)
    exact = exact_q(p2, cfg.eps_end)
    got, policy = learned(net, device)
    best = max(policy_value(dict(zip(exact.keys(), choice)), p2) for choice in itertools.product((0, 1), repeat=len(exact)))
    value = policy_value(policy, p2)
    worst_err = 0.0
    if verbose:
        print(f"\n== {name}: player 2 {p2}")
        print(f"{'card':<5}{'point':<9}{'exact check/fold':>18}{'learned':>9}{'exact bet/call':>16}{'learned':>9}   picks")
    for (c1, point), (e0, e1) in sorted(exact.items()):
        g0, g1 = got[(c1, point)]
        # Only judge values the learner actually tried often: an action it (rightly) almost
        # never takes, like folding a king, gets too few samples to pin down, and that's fine.
        n0, n1 = (env.visits.get((c1, point, a), 0) for a in (0, 1))
        for e, g, nv in ((e0, g0, n0), (e1, g1, n1)):
            if nv >= MIN_VISITS:
                worst_err = max(worst_err, abs(g - e))
        if verbose:
            pick = ["check/fold", "bet/call"][policy[(c1, point)]]
            mark = lambda nv: " " if nv >= MIN_VISITS else "*"
            print(f"{NAMES[c1]:<5}{['first', 'facing'][point]:<9}{e0:>18.3f}{g0:>8.3f}{mark(n0)}{e1:>16.3f}{g1:>8.3f}{mark(n1)}   {pick}")
    # The strategy must be a best response, and the values it tried often must be close.
    ok = abs(value - best) < 1e-6 and worst_err < 0.1
    if verbose:
        print(f"learned strategy wins {value:+.4f} a hand; the best response wins {best:+.4f}")
        print(f"largest error in a value it tried {MIN_VISITS}+ times: {worst_err:.3f} chips (* = tried fewer times, not judged)")
        print("PASS" if ok else "FAIL")
    return ok


def main() -> int:
    a = check("Against the textbook equilibrium (best possible: -1/18 = -0.0556)", EQUILIBRIUM)
    b = check("Against an always-aggressive player 2 (exploit it)", AGGRESSIVE, seed=1)
    return 0 if a and b else 1


if __name__ == "__main__":
    sys.exit(main())
