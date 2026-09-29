"""Checks on the Rust <-> Python bridge. Run: python tests/test_bridge.py (or pytest)."""

import numpy as np
import torch

import os
import tempfile

from wizard_rl import ACT_BID, ACT_CARD, ACTIONS, FEATURES, HAND, PHASE, WizardEnv, action_name, rust_forward
from wizard_rl.export import export
from wizard_rl.net import QNet, pick_actions


def test_shapes_and_legality():
    env = WizardEnv(32, [3, 4, 5, 6], "selfplay", 1)
    assert len(env) == 32
    obs, legal = env.observe()
    assert obs.shape == (32, FEATURES) and obs.dtype == np.float32
    assert legal.shape == (32, ACTIONS) and legal.dtype == np.bool_
    assert legal.any(axis=1).all()
    net = QNet(FEATURES, ACTIONS, 64, 2)
    total = 0
    for _ in range(3000):
        obs, legal = env.observe()
        with torch.no_grad():
            a = pick_actions(net(torch.from_numpy(obs)), torch.from_numpy(legal), 0.1)
        assert legal[np.arange(32), a.numpy()].all()
        env.step(a.numpy().astype(np.int64))
        o, act, ret = env.drain()
        assert o.shape == (len(act), FEATURES) and ret.shape == (len(act),)
        # A card action always matches a card in the hand block; bids come in the bid phase.
        cards = act >= ACT_CARD
        assert (o[cards, HAND + act[cards] - ACT_CARD] == 1.0).all()
        bids = (act >= ACT_BID) & ~cards
        assert (o[bids, PHASE + 1] == 1.0).all()
        total += len(act)
    st = env.stats()
    assert total > 0 and st["rounds"] > 0 and st["bot_rounds"] == 0


def test_illegal_action_is_refused():
    env = WizardEnv(1, [4], "counting", 2)
    _, legal = env.observe()
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
    net = QNet(FEATURES, ACTIONS, 96, 3)
    with tempfile.TemporaryDirectory() as d:
        ck = os.path.join(d, "m.pt")
        torch.save(dict(model=net.state_dict(), net=net.config()), ck)
        out = os.path.join(d, "m.wznet")
        export(ck, out)
        env = WizardEnv(64, [3, 4, 5, 6], "selfplay", 3)
        obs, _ = env.observe()
        with torch.no_grad():
            want = net(torch.from_numpy(obs)).numpy()
        got = rust_forward(out, obs)
        assert np.abs(got - want).max() < 1e-4, np.abs(got - want).max()


if __name__ == "__main__":
    test_export_matches_pytorch()
    test_shapes_and_legality()
    test_illegal_action_is_refused()
    test_action_names()
    print("bridge ok")
