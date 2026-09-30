"""A learner that outputs probabilities (PPO), to compare with the Deep Monte Carlo bot.

    python -m wizard_rl.ppo --hours 4 --init runs/simul1/best.pt --reference runs/simul1/best.pt --out runs/ppo1

The DMC bot always plays its single best-scoring move. This one learns a probability for every
move (a *policy*), so it can mix: bid 1 seventy percent of the time and 2 thirty percent, say.
That's what equilibrium play needs in games like poker, and it's harder to read.

With `--anchor DMC.pt --kl 0.1` it stays close to the DMC bot's choices (a KL penalty towards
softmax(DMC points / 5)) and only moves where that clearly pays: DMC decides, PPO fine-tunes.

How it learns (Proximal Policy Optimization, with whole-round returns):

    advantage  A = G/scale - V(s)                       how much better the round went than expected
    ratio      r = pi_new(a|s) / pi_old(a|s)
    loss       = -min(r A, clip(r, 1-e, 1+e) A)  +  c_v (V(s) - G/scale)^2  -  c_h entropy

In words: make moves that beat expectations more likely, the ones that fell short less likely,
but never move the policy too far in one update (the clip); keep a value estimate V to judge
"better than expected"; and a small entropy bonus keeps it from collapsing onto one move too early.

`--init` starts it from a DMC network: the policy begins as a softmax of the DMC scores (moves a
few points better get most of the probability), so the comparison is fair: can mixing improve
on the DMC bot from where it stands?
"""

from __future__ import annotations

import argparse
import copy
import csv
import json
import os
import time

import numpy as np
import torch
from torch import nn

from . import ACTIONS, FEATURES, WizardEnv
from .evaluate import evaluate_fn
from .net import best_device, load_qnet


class PolicyNet(nn.Module):
    """Shared MLP body; a policy head (logits over actions) and a value head (expected round
    score, in units of `scale` points)."""

    def __init__(self, features: int = FEATURES, actions: int = ACTIONS, hidden: int = 512, layers: int = 3):
        super().__init__()
        mods, width = [], features
        for _ in range(layers):
            mods += [nn.Linear(width, hidden), nn.ReLU()]
            width = hidden
        self.body = nn.Sequential(*mods)
        self.pi = nn.Linear(hidden, actions)
        self.v = nn.Linear(hidden, 1)
        self.features, self.actions, self.hidden, self.layers = features, actions, hidden, layers

    def forward(self, obs):
        h = self.body(obs[:, : self.features])  # an older network reads only the leading features
        return self.pi(h), self.v(h).squeeze(-1)

    def config(self) -> dict:
        return dict(features=self.features, actions=self.actions, hidden=self.hidden, layers=self.layers)


def init_from_dmc(pol: PolicyNet, dmc_path: str, temperature: float = 0.05) -> None:
    """Copy the DMC body; policy logits = DMC scores / temperature (0.05 = 5 points); value head
    starts from the average of the scores."""
    q = load_qnet(dmc_path)
    with torch.no_grad():
        src = [m for m in q.body if isinstance(m, nn.Linear)]
        dst = [m for m in pol.body if isinstance(m, nn.Linear)]
        assert len(src) == len(dst) + 1, "same depth expected"
        for i, (a, b) in enumerate(zip(src[:-1], dst)):
            c = min(a.weight.shape[1], b.weight.shape[1])
            if i == 0:
                b.weight[:, c:].zero_()  # inputs the DMC network didn't have start at zero weight
            b.weight[:, :c].copy_(a.weight[:, :c])
            b.bias.copy_(a.bias)
        head = src[-1]
        pol.pi.weight.copy_(head.weight[:ACTIONS] / temperature)
        pol.pi.bias.copy_(head.bias[:ACTIONS] / temperature)
        pol.v.weight.copy_(head.weight[:ACTIONS].mean(0, keepdim=True))
        pol.v.bias.copy_(head.bias[:ACTIONS].mean().reshape(1))


def masked_logits(logits, legal):
    return logits.masked_fill(~legal, float("-inf"))


@torch.no_grad()
def policy_act(pol: PolicyNet, obs, legal, sample: bool = True, gen=None):
    logits, _ = pol(obs)
    ml = masked_logits(logits, legal)
    if sample:
        probs = torch.softmax(ml, dim=1)
        a = torch.multinomial(probs, 1, generator=gen).squeeze(1)
    else:
        a = ml.argmax(1)
    logp = torch.log_softmax(ml, dim=1).gather(1, a.unsqueeze(1)).squeeze(1)
    return a, logp


def main(argv=None) -> None:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--out", default=f"runs/ppo-{time.strftime('%Y%m%d-%H%M%S')}")
    p.add_argument("--hours", type=float, default=None)
    p.add_argument("--decisions", type=float, default=None)
    p.add_argument("--init", default=None, help="DMC checkpoint to start from")
    p.add_argument("--resume", default=None, help="continue a PPO run from its latest.pt")
    p.add_argument("--save-every", type=float, default=5e6, help="decisions between checkpoints of latest.pt")
    p.add_argument("--reference", default=None, help="DMC checkpoint to play head-to-head against")
    p.add_argument("--players", default="3,4,5,6")
    p.add_argument("--tables", type=int, default=512)
    p.add_argument("--lr", type=float, default=5e-5)
    p.add_argument("--rollout", type=int, default=65536, help="samples per update")
    p.add_argument("--minibatch", type=int, default=4096)
    p.add_argument("--epochs", type=int, default=3)
    p.add_argument("--clip", type=float, default=0.2)
    p.add_argument("--entropy", type=float, default=0.003)
    p.add_argument("--anchor", default=None,
                   help="a DMC checkpoint to stay close to: adds kl x KL(policy || softmax(DMC points / anchor-temp)) "
                        "to the loss, so the policy only moves away from the DMC bot where it clearly pays")
    p.add_argument("--kl", type=float, default=0.1, help="weight of the anchor term")
    p.add_argument("--anchor-temp", type=float, default=5.0, help="points: how sharply the anchor prefers DMC's best move")
    p.add_argument("--value-coef", type=float, default=0.5)
    p.add_argument("--temperature", type=float, default=0.05, help="softmax temperature for --init")
    p.add_argument("--mix", default="0.7,0.2,0.1", help="other seats: learner,frozen,counting weights")
    p.add_argument("--game", action="store_true", help="full games, rewarded by winning (as in train --game)")
    p.add_argument("--win-weight", type=float, default=1.0)
    p.add_argument("--styles", default=None,
                   help="exploitation: most other seats are --style-net with these habits "
                        "(overbid,underbid,early-wizard,wild,softN,plain); no frozen copies of the policy")
    p.add_argument("--style-net", default=None, help="DMC network the styled opponents play with")
    p.add_argument("--snapshot-every", type=float, default=2.5e7)
    p.add_argument("--pool", type=int, default=8)
    p.add_argument("--eval-every", type=float, default=2.5e7)
    p.add_argument("--eval-rounds", type=int, default=20_000)
    p.add_argument("--threads", type=int, default=0)
    p.add_argument("--device", default="auto")
    p.add_argument("--seed", type=int, default=0)
    a = p.parse_args(argv)
    if a.hours is None and a.decisions is None:
        p.error("give --hours or --decisions")
    if a.threads:
        torch.set_num_threads(a.threads)
    device = best_device() if a.device == "auto" else torch.device(a.device)
    players = [int(x) for x in a.players.split(",")]
    wl, wn, wc = (float(x) for x in a.mix.split(","))
    styles = a.styles.split(",") if a.styles else []
    if styles and a.mix == p.get_default("mix"):
        wl, wn, wc = 0.25, 0.75, 0.0
    os.makedirs(a.out, exist_ok=True)
    scale = 100.0

    resumed = torch.load(a.resume, map_location="cpu", weights_only=False) if a.resume else None
    if resumed:
        pol = PolicyNet(**resumed["net"])
        pol.load_state_dict(resumed["model"])
    elif a.init:
        pol = PolicyNet(features=load_qnet(a.init).features)  # same inputs as the DMC network
        init_from_dmc(pol, a.init, a.temperature)
    else:
        pol = PolicyNet()
    pol.to(device)
    reference = load_qnet(a.reference).to(device) if a.reference else None
    anchor = load_qnet(a.anchor).to(device).eval() if a.anchor else None
    opt = torch.optim.Adam(pol.parameters(), lr=a.lr)
    if resumed:
        opt.load_state_dict(resumed["opt"])
    env = WizardEnv(a.tables, players, dict(learner=wl, nets=wn, counting=wc), a.seed, False, True, False,
                    a.game, a.win_weight)
    frozen: list[PolicyNet] = []
    # Styled opponents (exploitation runs): owner k plays styled_players[k - 1] instead of a frozen policy.
    styled_players = []
    if styles:
        from .players import make_player
        net_path = a.style_net or a.anchor or a.init
        styled_players = [make_player(f"{net_path}@{st}" if st != "plain" else net_path, device, 100 + i)
                          for i, st in enumerate(styles)]
        env.set_nets(len(styles))
    freezes = 0
    gen = torch.Generator().manual_seed(a.seed)
    with open(os.path.join(a.out, "config.json"), "w") as f:
        json.dump(dict(vars(a), net=pol.config(), device=str(device)), f, indent=2)
    log_path = os.path.join(a.out, "metrics.csv")
    new_log = not os.path.exists(log_path)
    log = open(log_path, "a", newline="")
    w = csv.writer(log)
    if new_log:
        w.writerow(["time_s", "decisions", "updates", "policy_loss", "value_loss", "entropy", "clip_frac",
                    "edge_vs_counting_sampled", "edge_vs_counting_greedy", "edge_vs_reference_sampled", "edge_vs_reference_greedy"])

    def freeze():
        nonlocal freezes
        snap = copy.deepcopy(pol).eval()
        if len(frozen) < a.pool:
            frozen.append(snap)
        else:
            frozen[freezes % a.pool] = snap
        freezes += 1
        env.set_nets(len(frozen))

    def evaluate_all():
        pol.eval()

        def sampled(o, lg):
            return policy_act(pol, o, lg, True, gen)[0]

        def greedy(o, lg):
            return policy_act(pol, o, lg, False)[0]

        r = {}
        g = dict(game=a.game)
        r["cs"] = evaluate_fn(sampled, device, a.eval_rounds, players, "counting", **g)["edge"]
        r["cg"] = evaluate_fn(greedy, device, a.eval_rounds, players, "counting", **g)["edge"]
        if reference is not None:
            ref = lambda o, lg: reference(o).masked_fill(~lg, float("-inf")).argmax(1)  # noqa: E731
            r["rs"] = evaluate_fn(sampled, device, a.eval_rounds, players, "nets", ref, **g)["edge"]
            r["rg"] = evaluate_fn(greedy, device, a.eval_rounds, players, "nets", ref, **g)["edge"]
        # against each styled opponent (a whole table of it), most-likely move
        for i, st in enumerate(styles):
            r[f"style:{st}"] = evaluate_fn(greedy, device, a.eval_rounds // 2, players, "nets", styled_players[i], **g)["edge"]
        pol.train()
        return r

    decisions, updates, t0 = 0, 0, time.time()
    best = float("-inf")
    if resumed:
        decisions, updates, best = resumed["decisions"], resumed["updates"], resumed["best"]
        for sd in resumed["pool"] if not styled_players else []:
            freeze()
            frozen[-1].load_state_dict(sd)
        print(f"resumed from {a.resume} at {decisions:,} decisions, pool {len(frozen)}", flush=True)
    buf = []
    buffered = 0
    next_eval, next_snap, next_save = decisions + a.eval_every, decisions + a.snapshot_every, decisions + a.save_every
    stats = dict(pl=float("nan"), vl=float("nan"), ent=float("nan"), cf=float("nan"))

    def save_latest(edge):
        path = os.path.join(a.out, "latest.pt")
        torch.save(dict(model=pol.state_dict(), opt=opt.state_dict(), net=pol.config(), decisions=decisions, updates=updates,
                        edge=edge, best=best, pool=[m.state_dict() for m in frozen]), path + ".tmp")
        os.replace(path + ".tmp", path)

    def tick():
        nonlocal best
        r = evaluate_all()
        w.writerow([round(time.time() - t0), decisions, updates, f"{stats['pl']:.4f}", f"{stats['vl']:.4f}", f"{stats['ent']:.3f}",
                    f"{stats['cf']:.3f}", f"{r['cs']:.2f}", f"{r['cg']:.2f}", f"{r.get('rs', float('nan')):.2f}", f"{r.get('rg', float('nan')):.2f}"])
        log.flush()
        ref = f" | vs DMC bot: sampled {r['rs']:+5.2f}, greedy {r['rg']:+5.2f}" if "rs" in r else ""
        sty = "".join(f" | vs {k[6:]}: {v:+.1f}" for k, v in r.items() if k.startswith("style:"))
        unit = "game" if a.game else "round"
        print(f"[{(time.time() - t0) / 60:6.1f} min] {decisions / 1e6:7.1f}M  entropy {stats['ent']:.3f}  clip {stats['cf']:.2f} | "
              f"per {unit}: vs counting: sampled {r['cs']:+5.1f}, greedy {r['cg']:+5.1f}{ref}{sty}", flush=True)
        score = r.get("rs", r["cs"])
        if score > best:
            best = score
            torch.save(dict(model=pol.state_dict(), net=pol.config(), decisions=decisions, edge=score), os.path.join(a.out, "best.pt"))
        save_latest(score)

    print(f"PPO on {device}; players {players}; out {a.out}", flush=True)
    if not resumed:
        tick()
    while True:
        # ---- play one step at every table
        with torch.no_grad():
            obs, legal, owner = env.observe()
            o = torch.from_numpy(obs).to(device)
            lg = torch.from_numpy(legal).to(device)
            acts = torch.zeros(len(obs), dtype=torch.int64)
            aux = torch.zeros(len(obs), dtype=torch.float32)
            mine = torch.from_numpy(owner == 0)
            if mine.any():
                act, logp = policy_act(pol, o[mine.to(device)], lg[mine.to(device)], True, gen)
                acts[mine], aux[mine] = act.cpu(), logp.cpu()
                decisions += int(mine.sum())
            for k in np.unique(owner[owner > 0]):
                sel = torch.from_numpy(owner == k)
                if styled_players:
                    act = styled_players[int(k) - 1](o[sel.to(device)], lg[sel.to(device)])
                else:
                    act, _ = policy_act(frozen[int(k) - 1], o[sel.to(device)], lg[sel.to(device)], True, gen)
                acts[sel] = act.cpu()
            env.step(acts.numpy(), aux.numpy())
            d = env.drain()
            if len(d[1]):
                buf.append((d[0], d[1], d[2], d[4], d[5]))
                buffered += len(d[1])
        # ---- learn from a full rollout
        if buffered >= a.rollout:
            ob, ac, re, lp, lm = (np.concatenate([b[i] for b in buf]) for i in range(5))
            buf, buffered = [], 0
            ob, ac = torch.from_numpy(ob).to(device), torch.from_numpy(ac).to(device)
            g = torch.from_numpy(re).to(device) / scale
            old = torch.from_numpy(lp).to(device)
            legal_m = torch.from_numpy(lm).to(device)
            with torch.no_grad():
                _, v = pol(ob)
                adv = g - v
                adv = (adv - adv.mean()) / (adv.std() + 1e-8)
            n = len(ac)
            pls, vls, ents, cfs = [], [], [], []
            for _ in range(a.epochs):
                perm = torch.randperm(n, generator=gen).to(device)
                for i in range(0, n, a.minibatch):
                    idx = perm[i:i + a.minibatch]
                    logits, v = pol(ob[idx])
                    logp_all = torch.log_softmax(masked_logits(logits, legal_m[idx]), dim=1)
                    logp = logp_all.gather(1, ac[idx].unsqueeze(1)).squeeze(1)
                    ratio = torch.exp(logp - old[idx])
                    s1 = ratio * adv[idx]
                    s2 = torch.clamp(ratio, 1 - a.clip, 1 + a.clip) * adv[idx]
                    pl = -torch.min(s1, s2).mean()
                    vl = torch.nn.functional.mse_loss(v, g[idx])
                    p_all = logp_all.exp()
                    ent = -(p_all * logp_all.masked_fill(~legal_m[idx], 0.0)).sum(1).mean()
                    loss = pl + a.value_coef * vl - a.entropy * ent
                    if anchor is not None:
                        with torch.no_grad():
                            pts = anchor(ob[idx]) * scale
                            logp_anchor = torch.log_softmax(masked_logits(pts / a.anchor_temp, legal_m[idx]), dim=1)
                        diff = (logp_all - logp_anchor).masked_fill(~legal_m[idx], 0.0)
                        loss = loss + a.kl * (p_all * diff).sum(1).mean()
                    opt.zero_grad(set_to_none=True)
                    loss.backward()
                    torch.nn.utils.clip_grad_norm_(pol.parameters(), 1.0)
                    opt.step()
                    updates += 1
                    pls.append(pl.item()); vls.append(vl.item()); ents.append(ent.item())
                    cfs.append(((ratio - 1).abs() > a.clip).float().mean().item())
            stats.update(pl=float(np.mean(pls)), vl=float(np.mean(vls)), ent=float(np.mean(ents)), cf=float(np.mean(cfs)))
        if decisions >= next_snap:
            if not styled_players:
                freeze()
            next_snap += a.snapshot_every
        if decisions >= next_eval:
            tick()
            next_eval += a.eval_every
        elif decisions >= next_save:
            save_latest(float("nan"))
        if decisions >= next_save:
            next_save = decisions + a.save_every
        if a.decisions and decisions >= a.decisions:
            break
        if a.hours and time.time() - t0 >= a.hours * 3600:
            break
    tick()
    log.close()


if __name__ == "__main__":
    main()
