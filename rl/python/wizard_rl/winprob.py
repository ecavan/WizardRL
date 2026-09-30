"""Win probability from the score situation, and "win probability added" as a reward.

    python -m wizard_rl.winprob models/simul1.pt --out models/winprob.pt     # fit W
    python -m wizard_rl.winprob export models/winprob.pt models/winprob.wzwp # for `wizard watch/play`

W(players, rounds left, my margin over the best other player, over the second best) = chance of
winning the game from here, fitted on self-play games of a trained network (a small network,
cross-entropy). It's the same idea as a live win-probability chart in sports.

Why: rewarding every decision with "did you win the game?" is right but very noisy: one
decision barely moves a whole game's result, so the learner drowns in luck from the other 14
rounds. Instead each round is rewarded with how much it changed the seat's chance of winning:

    reward(round) = 100 x ( W(after the round) - W(before the round) )

These add up over the game to (final result - starting chance), so maximizing them is maximizing
the chance of winning, but each round's reward only carries that round's luck. A round where you
were 80 points behind with two rounds left and you played safe to +30 earns almost nothing; a
gamble that catches the leader earns a lot. That is exactly the behaviour a round-score bot can't
learn.
"""

from __future__ import annotations

import argparse

import numpy as np
import torch
from torch import nn

from . import PHASE, WizardEnv
from .net import load_qnet

PLAYERS = (3, 4, 5, 6)


class WinProb(nn.Module):
    def __init__(self, hidden: int = 64):
        super().__init__()
        self.body = nn.Sequential(nn.Linear(4 + 3, hidden), nn.ReLU(), nn.Linear(hidden, hidden), nn.ReLU(),
                                  nn.Linear(hidden, 1))

    @staticmethod
    def inputs(players, left, d1, d2) -> torch.Tensor:
        players = torch.as_tensor(players, dtype=torch.float32)
        onehot = torch.stack([(players == n).float() for n in PLAYERS], 1)
        cols = [torch.as_tensor(x, dtype=torch.float32) for x in (left, d1, d2)]
        return torch.cat([onehot, (cols[0] / 20).unsqueeze(1), (cols[1] / 100).clamp(-6, 6).unsqueeze(1),
                          (cols[2] / 100).clamp(-6, 6).unsqueeze(1)], 1)

    def forward(self, x):
        return self.body(x).squeeze(1)

    @torch.no_grad()
    def prob(self, players, left, d1, d2) -> torch.Tensor:
        """Chance of winning; with no rounds left, the actual result (a tie for the lead counts
        half, whatever the number of players tied)."""
        p = torch.sigmoid(self(self.inputs(players, left, d1, d2)))
        d1 = torch.as_tensor(d1, dtype=torch.float32)
        done = torch.as_tensor(left) == 0
        final = torch.where(d1 > 0, 1.0, torch.where(d1 == 0, 0.5, 0.0))
        return torch.where(done, final, p)


def wpa_returns(model: WinProb, ctx: np.ndarray, weight: float | None = None) -> np.ndarray:
    """100 x (W after - W before) for each sample, from `env.last_context()` rows (players,
    rounds left after the round, margins before, margins after, own round score). With `weight`,
    the round score plus weight x that: points as usual, plus a bonus for moving the chance of
    winning (weight 2: +10% win chance is worth 20 points)."""
    n, left, b1, b2, a1, a2 = (ctx[:, i] for i in range(6))
    before = model.prob(n, left + 1, b1, b2)
    after = model.prob(n, left, a1, a2)
    wpa = (100.0 * (after - before)).numpy().astype(np.float32)
    if weight is None:
        return wpa
    return (ctx[:, 6] + weight * wpa).astype(np.float32)


def load_winprob(path: str) -> WinProb:
    m = WinProb()
    m.load_state_dict(torch.load(path, map_location="cpu")["model"])
    return m.eval()


@torch.no_grad()
def collect(net, games: int, seed: int = 3):
    """(players, rounds left before the round, margins before the round, won) for every seat and
    round of self-play full games."""
    env = WizardEnv(256, list(PLAYERS), "selfplay", seed, False, True, False, True, 1.0)
    rows, done = [], 0
    while done < games:
        obs, legal, _ = env.observe()
        q = net(torch.from_numpy(obs))
        env.step(q.masked_fill(~torch.from_numpy(legal), float("-inf")).argmax(1).numpy())
        d = env.drain()
        if len(d[1]):
            ctx = env.last_context()
            bid = d[0][:, PHASE + 1] == 1  # one row per seat and round: its bid decision
            c = ctx[bid]
            rows.append(np.column_stack([c[:, 0], c[:, 1] + 1, c[:, 2], c[:, 3], d[2][bid] / 100.0]))
            done += int((c[:, 1] == 0).sum() / 1)  # last-round bids: one per seat per game
    data = np.concatenate(rows)
    return data


def export(pt_path: str, out_path: str) -> None:
    """Write W for the Rust engine (`wizard watch` and `wizard play` show win chances with it):

        b"WZWINP01"  u32 layers,  per layer: u32 in  u32 out  f32[out * in] weights  f32[out] bias

    little-endian, ReLU between layers; inputs as in `WinProb.inputs`, output a logit."""
    import struct
    m = load_winprob(pt_path)
    linears = [x for x in m.body if isinstance(x, nn.Linear)]
    with open(out_path, "wb") as f:
        f.write(b"WZWINP01")
        f.write(struct.pack("<I", len(linears)))
        for lin in linears:
            w = lin.weight.detach().float().numpy()
            f.write(struct.pack("<II", w.shape[1], w.shape[0]))
            f.write(w.astype("<f4").tobytes())
            f.write(lin.bias.detach().float().numpy().astype("<f4").tobytes())
    print(f"wrote {out_path}")
    for left, d1 in ((10, -50), (3, -50), (1, 30)):
        print(f"  check: 4 players, {left} rounds left, {d1:+d}: {float(m.prob([4], [left], [d1], [d1])[0]):.4f}")


def main(argv=None) -> None:
    import sys
    argv = sys.argv[1:] if argv is None else argv
    if argv and argv[0] == "export":
        if len(argv) != 3:
            raise SystemExit("usage: python -m wizard_rl.winprob export WINPROB.pt OUT.wzwp")
        return export(argv[1], argv[2])
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("model", help="network whose self-play games define the win chances")
    p.add_argument("--games", type=int, default=40_000, help="seat-games of self-play")
    p.add_argument("--out", required=True)
    a = p.parse_args(argv)
    net = load_qnet(a.model).eval()
    data = collect(net, a.games)
    rng = np.random.default_rng(0)
    test = rng.random(len(data)) < 0.1
    x = WinProb.inputs(data[:, 0], data[:, 1], data[:, 2], data[:, 3])
    y = torch.as_tensor(data[:, 4], dtype=torch.float32)
    m = WinProb()
    opt = torch.optim.Adam(m.parameters(), lr=3e-3)
    tr = torch.from_numpy(~test)
    for epoch in range(300):
        perm = torch.randperm(int(tr.sum()))
        xs, ys = x[tr][perm], y[tr][perm]
        for i in range(0, len(xs), 8192):
            loss = nn.functional.binary_cross_entropy_with_logits(m(xs[i:i + 8192]), ys[i:i + 8192])
            opt.zero_grad()
            loss.backward()
            opt.step()
    with torch.no_grad():
        te = torch.from_numpy(test)
        pt = torch.sigmoid(m(x[te]))
        ll = nn.functional.binary_cross_entropy(pt, y[te]).item()
        base = nn.functional.binary_cross_entropy(torch.full_like(pt, float(y[tr].mean())), y[te]).item()
    torch.save(dict(model=m.state_dict()), a.out)
    print(f"{len(data):,} seat-rounds; held-out log loss {ll:.4f} (knowing nothing: {base:.4f}); wrote {a.out}")
    # A few readings, 4 players:
    for left, d1 in ((10, -50), (3, -50), (3, 0), (1, -30), (1, 30)):
        print(f"  4 players, {left} rounds left, {d1:+d} vs the leader (2nd-best other level): "
              f"{float(m.prob([4], [left], [d1], [d1])[0]):.0%} to win")


if __name__ == "__main__":
    main()
