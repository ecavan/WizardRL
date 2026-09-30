"""Checks on the Rust <-> Python bridge. Run: python tests/test_bridge.py (or pytest)."""

import numpy as np
import torch

import os
import tempfile

from wizard_rl import ACT_BID, ACT_CARD, ACTIONS, FEATURES, GAME, HAND, PHASE, ROUND_FEATURES, WizardEnv, action_name, rust_forward
from wizard_rl.export import export
from wizard_rl.net import QNet, pick_actions


def test_shapes_and_legality():
    env = WizardEnv(32, [3, 4, 5, 6], "selfplay", 1)
    assert len(env) == 32
    obs, legal, owner = env.observe()
    assert (owner == 0).all()
    assert obs.shape == (32, FEATURES) and obs.dtype == np.float32
    assert legal.shape == (32, ACTIONS) and legal.dtype == np.bool_
    assert legal.any(axis=1).all()
    net = QNet(FEATURES, ACTIONS, 64, 2)
    total = 0
    for _ in range(3000):
        obs, legal, _ = env.observe()
        with torch.no_grad():
            a = pick_actions(net(torch.from_numpy(obs)), torch.from_numpy(legal), 0.1)
        assert legal[np.arange(32), a.numpy()].all()
        env.step(a.numpy().astype(np.int64))
        o, act, ret, made, aux, lgl = env.drain()
        assert lgl.shape == (len(act), ACTIONS) and lgl[np.arange(len(act)), act].all()
        assert o.shape == (len(act), FEATURES) and ret.shape == (len(act),) and made.shape == (len(act),)
        assert ((ret >= 20) == (made == 1.0)).all()
        # A card action always matches a card in the hand block; bids come in the bid phase.
        cards = act >= ACT_CARD
        assert (o[cards, HAND + act[cards] - ACT_CARD] == 1.0).all()
        bids = (act >= ACT_BID) & ~cards
        assert (o[bids, PHASE + 1] == 1.0).all()
        total += len(act)
    st = env.stats()
    assert total > 0 and st["rounds"] > 0 and st["other_rounds"] == 0


def test_illegal_action_is_refused():
    env = WizardEnv(1, [4], "counting", 2)
    _, legal, _ = env.observe()
    bad = int(np.flatnonzero(~legal[0])[0])
    try:
        env.step(np.array([bad], dtype=np.int64))
    except ValueError:
        pass
    else:
        raise AssertionError("illegal action accepted")


def test_action_names():
    assert action_name(ACT_BID + 2) == "bid 2"
    assert action_name(ACT_CARD + 51) == "A♠"


def test_export_matches_pytorch():
    torch.manual_seed(0)
    net = QNet(FEATURES, ACTIONS, 96, 3, make_head=True)
    with tempfile.TemporaryDirectory() as d:
        ck = os.path.join(d, "m.pt")
        torch.save(dict(model=net.state_dict(), net=net.config()), ck)
        out = os.path.join(d, "m.wznet")
        export(ck, out)
        env = WizardEnv(64, [3, 4, 5, 6], "selfplay", 3)
        obs, _, _ = env.observe()
        with torch.no_grad():
            want = net.body(torch.from_numpy(obs)).numpy()
        got = rust_forward(out, obs)
        assert got.shape == want.shape == (64, 2 * ACTIONS)
        assert np.abs(got - want).max() < 1e-4, np.abs(got - want).max()


def test_two_brain_export_matches_pytorch():
    """A PPO policy exports with its points network; Rust runs both like PyTorch."""
    from wizard_rl.ppo import PolicyNet
    torch.manual_seed(1)
    pol = PolicyNet(FEATURES, ACTIONS, hidden=48, layers=2)
    dmc = QNet(FEATURES, ACTIONS, 64, 2, make_head=True)
    with tempfile.TemporaryDirectory() as d:
        pp, dp, out = (os.path.join(d, x) for x in ("pol.pt", "dmc.pt", "duo.wznet"))
        torch.save(dict(model=pol.state_dict(), net=pol.config()), pp)
        torch.save(dict(model=dmc.state_dict(), net=dmc.config()), dp)
        export(pp, out, evaluator=dp)
        obs, _, _ = WizardEnv(32, [3, 4, 5, 6], "selfplay", 5).observe()
        with torch.no_grad():
            logits, _ = pol(torch.from_numpy(obs))
            points = dmc.body(torch.from_numpy(obs))
        assert np.abs(rust_forward(out, obs, policy=True) - logits.numpy()).max() < 1e-4
        assert np.abs(rust_forward(out, obs) - points.numpy()).max() < 1e-4


def test_frozen_nets_and_duplicate():
    env = WizardEnv(16, [4], dict(learner=0.5, nets=0.5), 4)
    env.set_nets(2)
    owners = set()
    for _ in range(500):
        obs, legal, owner = env.observe()
        owners |= set(owner.tolist())
        env.step(np.array([np.flatnonzero(l)[0] for l in legal], dtype=np.int64))
    assert owners == {0, 1, 2}
    try:
        WizardEnv(4, [4], "train", 1, True)
    except ValueError:
        pass
    else:
        raise AssertionError("duplicate needs a single kind of opponent")
    WizardEnv(4, [4], "counting", 1, True)


def test_evaluation_with_a_quota():
    """The evaluator plays about the rounds asked for (a fixed number of deals per table) and
    finishes; random against random has a small edge."""
    from wizard_rl.evaluate import evaluate_fn

    rng = torch.Generator().manual_seed(0)

    def rand(o, lg):
        return torch.multinomial(lg.float(), 1, generator=rng).squeeze(1)

    r = evaluate_fn(rand, torch.device("cpu"), 2000, [3, 5], "nets", rand, tables=32)
    assert 1500 <= r["rounds"] <= 3000, r
    assert abs(r["edge"]) < 5, r  # random vs random: small, noisy


def test_full_games_and_older_networks():
    """Game mode: returns are game rewards (0..100) and the game features are filled in. A network
    from before the game features (503 inputs) still runs, in PyTorch and in Rust."""
    env = WizardEnv(8, [4], "selfplay", 3, False, True, False, True, 1.0)
    rng = np.random.default_rng(0)
    seen_game = False
    for _ in range(3000):
        obs, legal, _ = env.observe()
        seen_game |= bool((obs[:, GAME] == 1).all())
        acts = np.array([rng.choice(np.flatnonzero(m)) for m in legal], dtype=np.int64)
        env.step(acts)
    assert seen_game
    _, _, ret, *_ = env.drain()
    assert len(ret) > 0 and ret.min() >= 0 and ret.max() <= 100
    st = env.stats()
    assert st["games"] > 0 and abs(st["learner_wins"] + st["other_wins"] - st["games"]) < 1e-6

    old = QNet(ROUND_FEATURES, ACTIONS, hidden=32, layers=2)
    x = torch.from_numpy(obs)
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "old.pt")
        torch.save(dict(model=old.state_dict(), net=old.config()), path)
        export(path, os.path.join(d, "old.wznet"))
        rust = rust_forward(os.path.join(d, "old.wznet"), obs)
    with torch.no_grad():
        ours = old.body(x[:, :ROUND_FEATURES]).numpy()
        assert np.allclose(old(x).numpy(), ours[:, :ACTIONS])
    assert np.allclose(rust, ours, atol=1e-4)


def test_styles_are_legal_and_have_their_habit():
    from wizard_rl.styles import STYLES, apply_style

    env = WizardEnv(64, [3, 4, 5, 6], "selfplay", 9, False, True, False, True)
    gen = torch.Generator().manual_seed(1)
    rng = np.random.default_rng(1)
    shifted = {"overbid": 0, "underbid": 0}
    for _ in range(400):
        obs, legal, _ = env.observe()
        o, lg = torch.from_numpy(obs), torch.from_numpy(legal)
        base = torch.tensor([rng.choice(np.flatnonzero(m)) for m in legal])
        for style in STYLES:
            a = apply_style(style, base, o, lg, gen)
            assert bool(lg[torch.arange(len(a)), a].all()), style
            if style in shifted:
                shifted[style] += int((a - base).sum())
        env.step(base.numpy())
    assert shifted["overbid"] > 0 > shifted["underbid"], shifted


if __name__ == "__main__":
    test_styles_are_legal_and_have_their_habit()
    test_full_games_and_older_networks()
    test_evaluation_with_a_quota()
    test_frozen_nets_and_duplicate()
    test_export_matches_pytorch()
    test_shapes_and_legality()
    test_illegal_action_is_refused()
    test_action_names()
    print("bridge ok")
