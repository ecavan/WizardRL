"""Export a checkpoint for the Rust engine (`wizard play --advisor ...`, `--bots net:...`).

    python -m wizard_rl.export runs/first/best.pt runs/first/best.wznet
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


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    export(sys.argv[1], sys.argv[2])
