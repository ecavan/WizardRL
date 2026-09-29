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
python -m wizard_rl.train --hours 8 --out runs/first --resume runs/first/latest.pt       # continue
python -m wizard_rl.train --hours 8 --out runs/second --init runs/first/best.pt \
    --reference runs/first/best.pt --eps-start 0.05                                        # build on a run
```

Who it plays: one seat per table is always the learner; each other seat is the learner too
(70%), a frozen older copy of it (20%, from a pool of 8 refreshed every 20M decisions), or a
counting bot (10%). Change with `--mix learner,frozen,counting`.

Every `--eval-every` decisions it measures the network on **duplicate deals** (each deal
replayed with the network in every seat, so luck cancels) against counting bots, random bots
and, with `--reference`, an earlier network. It prints a line, appends it to
`runs/<name>/metrics.csv`, and saves `latest.pt` and `best.pt`. The number to watch is **edge
vs counting**: its average round score minus theirs, on the same cards.

The network has two outputs per action: the expected round score (which picks the move) and the
chance of making its bid afterwards (trained alongside, shown by the advisor).

`--device auto` picks the Apple GPU (`mps`) when there is one. The network is small, so the CPU
can be as fast; try both for a minute (`--decisions 5e6 --eval-every 5e6`) and keep the faster.
Keep the Mac awake for long runs: `caffeinate -i python -m wizard_rl.train ...`.

## Play against it, or get its advice

Two trained networks come with the repo:

- `models/night1.wznet`: 7 hours on 2 CPU cores (617M decisions, built on the starter). About
  +12 points a round over the counting bots on duplicate deals; in full four-player games it
  averages 387 points to their ~190 and wins 80% of games. Shows the chance of making each bid.
- `models/starter.wznet`: the first 45-minute network (+5 a round; scores only).

`cargo run --release -p wizard -- play --advisor rl/models/night1.wznet`

## Ask it about bids

```sh
python -m wizard_rl.charts runs/first/best.pt      # edit SITUATIONS in charts.py for your own hands
```

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
- `python/wizard_rl/charts.py`: the network's expected score and make-chance for every bid in set situations
- `tests/test_bridge.py`: Rust <-> Python checks
