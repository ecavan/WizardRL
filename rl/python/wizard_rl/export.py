"""Export a checkpoint for the Rust engine (`wizard play --advisor ...`, `--bots net:...`).

    python -m wizard_rl.export runs/first/best.pt models/first.wznet   # for Rust
    python -m wizard_rl.export runs/first/best.pt models/first.pt      # slim PyTorch weights (3.5 MB)
    python -m wizard_rl.export models/ppo5.pt models/ppo5.wznet --evaluator models/arche2.pt

Training checkpoints carry the optimizer and the frozen pool (~38 MB); a `.pt` output keeps only
the network, which every Python command (evaluate, charts, bidchart, --init) accepts.

A PPO policy goes to Rust as a **two-brain file**: the policy (it chooses the moves, by its most
likely move) plus a DMC points network (`--evaluator`, normally the one it was anchored to),
which values every option for the advisor, the move-cost review and look-ahead search.
"""

from __future__ import annotations

import os
import struct
import sys

import torch

from .dmc import LoopConfig
from .net import load_qnet


def _block(f, linears, features: int, actions: int, scale: float) -> None:
    """One network in the Rust format (see crates/wizard/src/net.rs)."""
    f.write(b"WZNET001")
    f.write(struct.pack("<IIfI", features, actions, float(scale), len(linears)))
    for lin in linears:
        w = lin.weight.detach().to(torch.float32).contiguous()
        b = lin.bias.detach().to(torch.float32).contiguous()
        f.write(struct.pack("<II", w.shape[1], w.shape[0]))
        f.write(w.numpy().astype("<f4").tobytes())
        f.write(b.numpy().astype("<f4").tobytes())


def _dmc_block(f, path: str, scale: float):
    net = load_qnet(path)
    linears = [m for m in net.body if isinstance(m, torch.nn.Linear)]
    _block(f, linears, net.features, net.actions, scale)
    return net, linears


def is_policy(path: str) -> bool:
    ck = torch.load(path, map_location="cpu", weights_only=False)
    return "pi.weight" in ck["model"]


def export(ckpt_path: str, out_path: str, scale: float = LoopConfig.scale, evaluator: str | None = None) -> None:
    if is_policy(ckpt_path):
        if not evaluator:
            raise SystemExit(f"{ckpt_path} is a PPO policy: give --evaluator DMC.pt (a points network, "
                             "normally the one it was anchored to) for the advisor and search")
        from .ppo import PolicyNet
        ck = torch.load(ckpt_path, map_location="cpu", weights_only=False)
        pol = PolicyNet(**ck["net"])
        pol.load_state_dict(ck["model"])
        linears = [m for m in pol.body if isinstance(m, torch.nn.Linear)] + [pol.pi]
        with open(out_path, "wb") as f:
            f.write(b"WZDUO001")
            _block(f, linears, pol.features, pol.actions, 1.0)
            _dmc_block(f, evaluator, scale)
        print(f"wrote {out_path}: two-brain file, policy {os.path.basename(ckpt_path)} "
              f"+ points network {os.path.basename(evaluator)}")
        return
    with open(out_path, "wb") as f:
        net, linears = _dmc_block(f, ckpt_path, scale)
    head = " with the make-bid head" if net.make_head else ""
    print(f"wrote {out_path}: {len(linears)} layers, {sum(p.numel() for p in net.parameters()):,} parameters{head}")


def slim(ckpt_path: str, out_path: str) -> None:
    ck = torch.load(ckpt_path, map_location="cpu", weights_only=False)
    keep = {k: ck[k] for k in ("model", "net", "decisions", "edge") if k in ck}
    torch.save(keep, out_path)
    print(f"wrote {out_path}: network weights only")


if __name__ == "__main__":
    args = sys.argv[1:]
    ev = None
    if "--evaluator" in args:
        k = args.index("--evaluator")
        ev = args[k + 1] if k + 1 < len(args) else None
        args = args[:k] + args[k + 2:]
    if len(args) != 2 or ("--evaluator" in sys.argv and not ev):
        print(__doc__)
        sys.exit(2)
    os.makedirs(os.path.dirname(os.path.abspath(args[1])), exist_ok=True)
    if args[1].endswith(".pt"):
        slim(args[0], args[1])
    else:
        export(args[0], args[1], evaluator=ev)
