"""Export a checkpoint for the Rust engine (`wizard play --advisor ...`, `--bots net:...`).

    python -m wizard_rl.export runs/first/best.pt models/first.wznet   # for Rust
    python -m wizard_rl.export runs/first/best.pt models/first.pt      # slim PyTorch weights (3.5 MB)

Training checkpoints carry the optimizer and the frozen pool (~38 MB); a `.pt` output keeps only
the network, which every Python command (evaluate, charts, bidchart, --init) accepts.
"""

from __future__ import annotations

import struct
import sys

import torch

from .dmc import LoopConfig
from .net import load_qnet


def export(ckpt_path: str, out_path: str, scale: float = LoopConfig.scale) -> None:
    net = load_qnet(ckpt_path)
    linears = [m for m in net.body if isinstance(m, torch.nn.Linear)]
    with open(out_path, "wb") as f:
        f.write(b"WZNET001")
        f.write(struct.pack("<IIfI", net.features, net.actions, float(scale), len(linears)))
        for lin in linears:
            w = lin.weight.detach().to(torch.float32).contiguous()
            b = lin.bias.detach().to(torch.float32).contiguous()
            f.write(struct.pack("<II", w.shape[1], w.shape[0]))
            f.write(w.numpy().astype("<f4").tobytes())
            f.write(b.numpy().astype("<f4").tobytes())
    head = " with the make-bid head" if net.make_head else ""
    print(f"wrote {out_path}: {len(linears)} layers, {sum(p.numel() for p in net.parameters()):,} parameters{head}")


def slim(ckpt_path: str, out_path: str) -> None:
    ck = torch.load(ckpt_path, map_location="cpu", weights_only=False)
    keep = {k: ck[k] for k in ("model", "net", "decisions", "edge") if k in ck}
    torch.save(keep, out_path)
    print(f"wrote {out_path}: network weights only")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    (slim if sys.argv[2].endswith(".pt") else export)(sys.argv[1], sys.argv[2])
