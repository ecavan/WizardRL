# wizard_rl

Self-play reinforcement learning for Wizard. The game runs in Rust (`crates/wizard`, bound to
Python by `crates/wizard-py`); the network and training loop are PyTorch.

## Setup (once)

```sh
cd rl
python3 -m venv .venv && source .venv/bin/activate
pip install maturin numpy torch
maturin develop --release          # builds the Rust engine into the venv; rerun after Rust changes
python tests/test_bridge.py        # bridge checks, a few seconds
python -m wizard_rl.kuhn           # learner check on Kuhn poker, about a minute
```

## Train

```sh
python -m wizard_rl.train --hours 8 --out runs/first
python -m wizard_rl.train --hours 8 --out runs/first --resume runs/first/latest.pt   # continue
```

Every `--eval-every` decisions (default 2M) it plays the network against counting bots and
random bots, prints a line and appends it to `runs/<name>/metrics.csv`, and saves `latest.pt`
and `best.pt`. The number to watch is **edge vs counting**: the network's average round score
minus the counting bots' in the same rounds. 0 means as good as the counting bot.

`--device auto` picks the Apple GPU (`mps`) when there is one. The network is small, so the CPU
can be as fast; try both for a minute (`--decisions 5e6 --eval-every 5e6`) and keep the faster.
Keep the Mac awake for long runs: `caffeinate -i python -m wizard_rl.train ...`.

## Play against it, or get its advice

`models/starter.wznet` is a first network (45 minutes of training on 2 CPU cores): it already
beats the counting bots by about 5 points a round. Try it before training your own:
`cargo run --release -p wizard -- play --advisor rl/models/starter.wznet`.

```sh
python -m wizard_rl.export runs/first/best.pt runs/first/best.wznet
cd .. && cargo run --release -p wizard -- play --advisor rl/runs/first/best.wznet     # its predicted score for each of your options
cargo run --release -p wizard -- play --bots net:rl/runs/first/best.wznet,counting,counting
cargo run --release -p wizard -- sim --games 2000 --bots net:rl/runs/first/best.wznet,counting,counting,counting
```

The exported file runs in Rust with no Python needed (the same file can later go into an app).

## How it learns (Deep Monte Carlo)

One network plays every seat. For each decision it predicts the round score of every legal
action and takes the best (with a little random exploration). When the round ends, every
decision is labelled with the score its seat got, and the prediction is pulled toward it:
`loss = (Q(s, a) - G)^2`. See `dmc.py`.

## Files

- `python/wizard_rl/net.py`: the Q-network and action picking
- `python/wizard_rl/dmc.py`: the Deep Monte Carlo loop, shared by Wizard and the Kuhn check
- `python/wizard_rl/train.py`: training runs, logging, checkpoints
- `python/wizard_rl/evaluate.py`: edge against the baseline bots
- `python/wizard_rl/kuhn.py`: the learner check (known answers)
- `python/wizard_rl/export.py`: checkpoint -> `.wznet` file for the Rust engine
- `tests/test_bridge.py`: Rust <-> Python checks
