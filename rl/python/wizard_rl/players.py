"""Players for evaluation, written as `PATH[@style]`:

- `runs/x/best.pt`: a DMC network (plays its best move) or a PPO policy (its most likely move)
- `...@sample`: a PPO policy sampling its probabilities
- `...@soft10`: a DMC network that picks moves at random in proportion to exp(points / 10), so it
  often takes a move nearly as good as its best and rarely a bad one: a strong but imperfect,
  human-like player. A lower number plays closer to perfect (`@soft5`), a higher one sloppier
  (`@soft25`).
- `...@overbid`, `@underbid`, `@early-wizard`, `@wild`: habits (see styles.py)
- `counting`, `random`: the built-in bots
"""

from __future__ import annotations

import torch

from .evaluate import ActFn, load_player


def make_player(spec: str, device: torch.device, seed: int = 99) -> ActFn | str:
    """An action function, or the name of a built-in bot."""
    if spec in ("counting", "random"):
        return spec
    path, _, style = spec.partition("@")
    gen = torch.Generator().manual_seed(seed)
    if style == "sample":
        return load_player(path, device, sample=True)
    if style.startswith("soft"):
        temp = float(style[4:] or 10)
        ck = torch.load(path, map_location="cpu", weights_only=False)
        if "pi.weight" in ck["model"]:
            raise ValueError("@soft is for DMC networks; use @sample for a PPO policy")
        from .net import load_qnet
        net = load_qnet(path).to(device).eval()

        @torch.no_grad()
        def soft(o, lg):
            points = net(o) * 100.0  # the network's units are 100 points
            logits = (points / temp).masked_fill(~lg, float("-inf"))
            return torch.multinomial(torch.softmax(logits, 1).cpu(), 1, generator=gen).squeeze(1)

        return soft
    base = load_player(path, device)
    if not style:
        return base
    from .styles import apply_style
    return lambda o, lg: apply_style(style, base(o, lg).cpu(), o, lg, gen)


def describe(spec: str) -> str:
    path, _, style = spec.partition("@")
    name = path.rsplit("/", 1)[-1].removesuffix(".pt")
    if name in ("best", "latest"):  # runs/<name>/best.pt -> <name>
        name = path.rstrip("/").rsplit("/", 2)[-2]
    return f"{name} ({style})" if style else name
