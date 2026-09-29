"""Measure a player against other players.

The player plays one seat per round; everyone else is the same kind of opponent: counting
bots, random bots, or a fixed reference network. Deals are **duplicated**: each deal is replayed
with the player in every seat, so it holds exactly the cards its opponents held and luck cancels
out.

Every table plays the same fixed number of deals (so `rounds` is approximate), which keeps the
mix of round sizes fair: stopping once "enough rounds" have finished would over-count the small
rounds, which finish first.

The headline number is its **edge**: per round, its score minus its opponents' average score,
averaged over rounds (each deal counted once it has been replayed in every seat). Two copies of
the same player have an edge of exactly 0.
"""

from __future__ import annotations

from typing import Callable

import numpy as np
import torch

from . import WizardEnv
from .net import QNet, pick_actions

# act(obs [B, F] tensor on device, legal [B, A] bool tensor on device) -> actions [B]
ActFn = Callable[[torch.Tensor, torch.Tensor], torch.Tensor]


def greedy(net: QNet) -> ActFn:
    return lambda o, lg: pick_actions(net(o), lg, 0.0)


@torch.no_grad()
def evaluate_fn(act: ActFn, device: torch.device, rounds: int = 20_000, players=(3, 4, 5, 6), opponents: str = "counting",
                reference: ActFn | None = None, seed: int = 12345, tables: int = 256, simultaneous: bool = True,
                game: bool = False) -> dict:
    """With `game`, full games are played and `rounds` means learner games; the edge is then in
    points per game, and `win_rate` / `other_win_rate` / `fair_win_rate` are filled in."""
    if opponents == "nets":
        assert reference is not None, "a reference player is needed to play against"
    env = WizardEnv(tables, list(players), opponents, seed, True, simultaneous, False, game)
    if reference is not None:
        env.set_nets(1)
    # Every table plays the same number of deals (each replayed in every seat); rounds are
    # counted only for those, so quick small rounds aren't over-represented.
    mean_players = float(np.mean(list(players)))
    env.set_quota(max(1, int(np.ceil(rounds / (tables * mean_players)))))
    env.stats()
    tot = dict(learner_rounds=0, learner_score=0, learner_bids_made=0, other_rounds=0, other_score=0, other_bids_made=0,
               edge_sum=0.0, edge_rounds=0, learner_games=0, learner_wins=0.0, other_games=0, other_wins=0.0)
    while env.quota_left() > 0:
        obs, legal, owner = env.observe()
        o = torch.from_numpy(obs).to(device)
        lg = torch.from_numpy(legal).to(device)
        acts = torch.zeros(len(obs), dtype=torch.int64)
        mine = torch.from_numpy(owner == 0)
        if mine.any():
            acts[mine] = act(o[mine.to(device)], lg[mine.to(device)]).cpu()
        if (~mine).any():
            acts[~mine] = reference(o[(~mine).to(device)], lg[(~mine).to(device)]).cpu()
        env.step(acts.numpy())
        env.drain()
        for k, v in env.stats().items():
            if k in tot:
                tot[k] += v
    lr, orr = tot["learner_rounds"], max(1, tot["other_rounds"])
    learner = tot["learner_score"] / lr
    others = tot["other_score"] / orr
    edge = tot["edge_sum"] / max(1, tot["edge_rounds"])
    out = dict(rounds=lr, learner_avg=learner, other_avg=others, edge=edge,
               learner_bid_rate=tot["learner_bids_made"] / lr, other_bid_rate=tot["other_bids_made"] / orr)
    if game:
        out.update(games=tot["learner_games"], win_rate=tot["learner_wins"] / max(1, tot["learner_games"]),
                   other_win_rate=tot["other_wins"] / max(1, tot["other_games"]),
                   fair_win_rate=len(list(players)) / sum(players))
    return out


def evaluate(net: QNet, device: torch.device, rounds: int = 20_000, players=(3, 4, 5, 6), opponents: str = "counting",
             reference: QNet | None = None, seed: int = 12345, tables: int = 256, simultaneous: bool = True,
             game: bool = False) -> dict:
    """`evaluate_fn` for a DMC network (greedy), optionally against a reference DMC network."""
    ref = greedy(reference.to(device).eval()) if reference is not None else None
    return evaluate_fn(greedy(net), device, rounds, players, opponents, ref, seed, tables, simultaneous, game)


def load_player(path: str, device: torch.device, sample: bool = False) -> ActFn:
    """A player from a checkpoint: a DMC network (plays its best move) or a PPO policy
    (samples its probabilities, or plays its most likely move with `sample=False`)."""
    ck = torch.load(path, map_location="cpu", weights_only=False)
    if "pi.weight" in ck["model"]:
        from .ppo import PolicyNet, policy_act
        pol = PolicyNet(**ck["net"])
        pol.load_state_dict(ck["model"])
        pol.to(device).eval()
        gen = torch.Generator().manual_seed(99)
        return lambda o, lg: policy_act(pol, o, lg, sample, gen)[0]
    from .net import load_qnet
    return greedy(load_qnet(path).to(device).eval())


def main(argv=None) -> None:
    import argparse
    p = argparse.ArgumentParser(
        description="Duplicate-deal edge of one player over others: "
                    "python -m wizard_rl.evaluate runs/a/best.pt --vs counting (or random, or another checkpoint)")
    p.add_argument("model")
    p.add_argument("--vs", default="counting", help="counting, random, or a checkpoint (.pt)")
    p.add_argument("--players", default="3,4,5,6", help="table sizes (each reported separately too)")
    p.add_argument("--rounds", type=int, default=40_000, help="rounds per table size")
    p.add_argument("--chunks", type=int, default=8, help="independent batches, for the error bar")
    p.add_argument("--sample", action="store_true", help="PPO models: sample moves (default: most likely move)")
    p.add_argument("--vs-sample", action="store_true", help="the same, for a PPO opponent")
    p.add_argument("--vs-style", default=None,
                   help="give the --vs network a habit: overbid, underbid, early-wizard, wild (see styles.py)")
    p.add_argument("--in-turn", action="store_true", help="bid in turn instead of all at once")
    p.add_argument("--game", action="store_true", help="full games: edge in points per game, and win rates (--rounds = games)")
    p.add_argument("--device", default="cpu")
    a = p.parse_args(argv)
    device = torch.device(a.device)
    me = load_player(a.model, device, a.sample)
    if a.vs in ("counting", "random"):
        opp, ref = a.vs, None
    else:
        opp, ref = "nets", load_player(a.vs, device, a.vs_sample)
        if a.vs_style:
            from .styles import apply_style
            base, gen = ref, torch.Generator().manual_seed(5)
            ref = lambda o, lg: apply_style(a.vs_style, base(o, lg).cpu(), o, lg, gen)  # noqa: E731
    sizes = [int(x) for x in a.players.split(",")]
    unit = "game" if a.game else "round"
    print(f"{a.model} vs {a.vs}{' (' + a.vs_style + ')' if a.vs_style else ''}, duplicate {'games' if a.game else 'deals'}, {'bids in turn' if a.in_turn else 'bids all at once'}")
    extra = f" {'wins':>9} {'theirs':>7}" if a.game else ""
    print(f"{'players':>8} {'edge/' + unit:>16} {'its avg':>8} {'theirs':>8} {'its bids made':>14} {'theirs':>7}{extra}")
    all_edges = []
    for n in sizes:
        res = [evaluate_fn(me, device, a.rounds // a.chunks, [n], opp, ref, seed=1000 * n + c,
                           simultaneous=not a.in_turn, game=a.game) for c in range(a.chunks)]
        e = np.array([r["edge"] for r in res])
        se = e.std(ddof=1) / np.sqrt(len(e)) if len(e) > 1 else float("nan")
        all_edges.append(e)
        avg = lambda k: float(np.mean([r[k] for r in res]))  # noqa: E731
        wins = f" {avg('win_rate'):>9.1%} {avg('other_win_rate'):>7.1%}" if a.game else ""
        print(f"{n:>8} {e.mean():+8.2f} ± {2 * se:5.2f} {avg('learner_avg'):8.1f} {avg('other_avg'):8.1f} "
              f"{avg('learner_bid_rate'):>14.0%} {avg('other_bid_rate'):>7.0%}{wins}", flush=True)
    if len(sizes) > 1:
        m = np.mean([e.mean() for e in all_edges])
        se = np.sqrt(sum(e.var(ddof=1) / len(e) for e in all_edges)) / len(all_edges)
        print(f"{'all':>8} {m:+8.2f} ± {2 * se:5.2f}   (± is two standard errors; averages are per round)")


if __name__ == "__main__":
    main()
