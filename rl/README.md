# wizard_rl

The Python half of Wizard RL: training, evaluation, export and bid charts, in PyTorch, driving the
Rust engine in `../crates/wizard` through the bindings in `../crates/wizard-py`.

**The full guide (setup, every command, how it learns, results) is the [top-level README](../README.md).**

Setup, from this folder:

```sh
python3 -m venv .venv && source .venv/bin/activate
pip install maturin numpy torch
maturin develop --release          # rerun after any Rust change
python tests/test_bridge.py
```

The commands, each with `--help`:

| Command | What it does |
| --- | --- |
| `python -m wizard_rl.train` | train (or resume, or build on) a network by self-play |
| `python -m wizard_rl.evaluate` | a network's edge over counting or random bots, or over another network |
| `python -m wizard_rl.export` | a checkpoint as a `.wznet` for Rust, or a slim `.pt` |
| `python -m wizard_rl.charts` | the network's expected points for every bid in set situations |
| `python -m wizard_rl.bidchart` | bid charts for 3 to 6 players |
| `python -m wizard_rl.chartpage` | rebuild the interactive chart page from `bid_chart.json` |
| `python -m wizard_rl.ppo` | the probabilities (PPO) learner, for comparison |
| `python -m wizard_rl.kuhn` | learner check on Kuhn poker |
