"""Distill a trained network into a smaller one (for an app, or just to see how small it can go).

    python -m wizard_rl.distill models/simul1.pt --hidden 256 --layers 2 --out models/simul1-small.pt

The teacher plays itself; the student learns to predict the teacher's expected points for every
legal move (and its chance of making the bid), so it plays the same moves with far fewer
parameters. Then it's measured head-to-head against the teacher on duplicate deals.

    loss = mean over legal moves of (student points - teacher points)^2   (+ the make-bid head)
"""

from __future__ import annotations

import argparse
import time

import numpy as np
import torch
from torch import nn

from . import ACTIONS, FEATURES, WizardEnv
from .evaluate import evaluate_fn, greedy
from .net import QNet, load_qnet


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("teacher")
    p.add_argument("--hidden", type=int, default=256)
    p.add_argument("--layers", type=int, default=2)
    p.add_argument("--minutes", type=float, default=60)
    p.add_argument("--tables", type=int, default=512)
    p.add_argument("--batch", type=int, default=4096)
    p.add_argument("--lr", type=float, default=1e-3)
    p.add_argument("--eps", type=float, default=0.05, help="random moves while collecting, for coverage")
    p.add_argument("--out", required=True)
    p.add_argument("--threads", type=int, default=0)
    a = p.parse_args(argv)
    if a.threads:
        torch.set_num_threads(a.threads)
    teacher = load_qnet(a.teacher).eval()
    student = QNet(FEATURES, ACTIONS, a.hidden, a.layers, make_head=teacher.make_head)
    print(f"teacher {sum(x.numel() for x in teacher.parameters()):,} parameters -> "
          f"student {sum(x.numel() for x in student.parameters()):,}", flush=True)
    opt = torch.optim.Adam(student.parameters(), lr=a.lr)
    env = WizardEnv(a.tables, [3, 4, 5, 6], "selfplay", 1)
    gen = torch.Generator().manual_seed(1)
    t0, steps, buf = time.time(), 0, []
    while time.time() - t0 < a.minutes * 60:
        obs, legal, _ = env.observe()
        o, lg = torch.from_numpy(obs), torch.from_numpy(legal)
        with torch.no_grad():
            tq, tm = teacher.both(o)
        buf.append((o, lg, tq, tm))
        # the teacher plays (with a little randomness so the student sees varied positions)
        act = tq.masked_fill(~lg, float("-inf")).argmax(1)
        rnd = torch.rand(len(act), generator=gen) < a.eps
        noise = torch.rand(lg.shape, generator=gen).masked_fill(~lg, -1).argmax(1)
        env.step(torch.where(rnd, noise, act).numpy())
        env.drain()
        if sum(len(b[0]) for b in buf) >= a.batch:
            o, lg, tq, tm = (torch.cat([b[i] for b in buf]) if buf[0][i] is not None else None for i in range(4))
            buf = []
            sq, sm = student.both(o)
            m = lg.float()
            loss = (((sq - tq) ** 2) * m).sum() / m.sum()
            if tm is not None and sm is not None:
                loss = loss + 0.25 * (nn.functional.binary_cross_entropy_with_logits(sm, torch.sigmoid(tm), reduction="none") * m).sum() / m.sum()
            opt.zero_grad()
            loss.backward()
            opt.step()
            steps += 1
            if steps % 500 == 0:
                print(f"[{(time.time() - t0) / 60:5.1f} min] step {steps}  loss {loss.item():.5f}", flush=True)
    torch.save(dict(model=student.state_dict(), net=student.config()), a.out)
    r = evaluate_fn(greedy(student.eval()), torch.device("cpu"), 20000, [3, 4, 5, 6], "nets", greedy(teacher))
    print(f"wrote {a.out}; head-to-head vs the teacher: {r['edge']:+.2f} points per round "
          f"(0 = plays as well as the teacher)")


if __name__ == "__main__":
    main()
