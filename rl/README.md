# Wizard RL

A bot that teaches itself Wizard, the trick-taking card game, by playing millions of rounds
against copies of itself. It uses the official rules for 3 to 6 players, and by default everyone
bids at the same time. Once trained, you can:

- play against it, or have it whisper advice while you play, in the terminal;
- ask what it would bid with any hand;
- print **bid charts** (how many tricks each card is worth, by table size and round size);
- measure it against simple bots and against other versions of itself.

The game engine is Rust (`crates/wizard`). Training is Python and PyTorch (this folder). The two
talk through a small bridge (`crates/wizard-py`). A trained network is a 3.5 MB file that runs
in Rust with no Python needed.

**Contents:** [Quick start](#quick-start) · [The rules it plays](#the-rules-it-plays) ·
[How it learns](#how-it-learns) · [Commands](#commands) · [Results](#results) ·
[Bid charts](#bid-charts) · [Probabilities vs best move (PPO)](#a-learner-that-outputs-probabilities-ppo) ·
[Size and speed](#size-and-speed) · [Files](#files)

---

## Quick start

You need Rust (`rustup`) and Python 3.9+. From the repo root:

```sh
# 1. Play against the trained bot right away (Rust only, no Python needed)
cargo run --release -p wizard -- play --bots net:rl/models/simul1.wznet,net:rl/models/simul1.wznet,net:rl/models/simul1.wznet

# 2. Play against simple bots, with the trained bot advising you on every decision
cargo run --release -p wizard -- play --advisor rl/models/simul1.wznet
```

For training, evaluation and charts, set up Python once:

```sh
cd rl
python3 -m venv .venv && source .venv/bin/activate
pip install maturin numpy torch
maturin develop --release          # builds the Rust engine into the venv (rerun after any Rust change)
python tests/test_bridge.py        # checks the bridge, a few seconds
```

Every later session: `cd rl && source .venv/bin/activate`.

---

## The rules it plays

The official rules. `crates/wizard/README.md` has the full table. The short version:

- 60 cards: 52 standard, 4 Wizards, 4 Jesters. 3 to 6 players. Round *r* deals *r* cards, so a
  game is 20 / 15 / 12 / 10 rounds for 3 / 4 / 5 / 6 players.
- Trump: the next card turned up. A Wizard means the dealer picks trump. A Jester means no trump.
  In the last round no card is left, so there's no trump.
- **Bids: everyone bids at once** (hands out at the same time), so nobody sees anyone else's
  bid first. The printed rules have players bid in turn, starting left of the dealer; add
  `--in-turn` to any command to play that way instead. Bids may add up to anything.
- Follow suit if you can; Wizards and Jesters can be played any time. The first Wizard wins
  the trick, otherwise the highest trump, otherwise the highest card of the suit led.
- Score: bid made exactly = 20 + 10 per trick; missed = −10 per trick off.

---

## How it learns

### The idea (Deep Monte Carlo)

The bot is one neural network that plays every seat. Whenever it has to act (pick trump, bid, or
play a card), it looks at what that seat can see and **predicts the round score each legal move
would lead to**. Then it takes the move with the best prediction.

At first the predictions are random, so the play is too. After every round, each decision is
labelled with the score its seat actually got. The network is nudged so that next time it predicts
a bit closer to that score:

```
loss = (predicted score for the move it made − score it actually got)²
```

Do that across hundreds of millions of decisions and the predictions become good, which makes
the play good. That's all it is: no hand-written strategy, only "what score did this lead to?".
The method is called Deep Monte Carlo (DMC). It's the method behind DouZero, a strong bot for
the Chinese card game Dou Dizhu, which suits card games with hidden hands and several players.

A few details that matter:

- **Exploration.** A small share of moves (ε, 5% falling to 2%) are random, so it keeps trying
  things it currently thinks are worse and can find out it was wrong.
- **Rewards.** The reward is simply the round score: +20 plus 10 per trick for a made bid,
  −10 per trick off for a miss. Rounds are independent (the score doesn't depend on earlier
  rounds), so it learns round by round. It trains on all table sizes and round sizes at once.
- **Opponents.** If it only ever played itself, it could learn habits that only work against
  itself. So the other seats are the current network (70%), a frozen older copy of it from a pool
  of 8 (20%), or a simple counting bot (10%).
- **Chance of making the bid.** A second output learns the probability of making the bid from
  here. It doesn't change the moves; the advisor shows it ("bid 1: +15, 71% to make it").
- **What it sees.** 503 numbers from its own seat's point of view: its hand, trump, cards
  played so far, the current trick, bids and tricks won by everyone (bids are hidden until
  everyone has bid), which suits each player has shown they're out of, and more. See
  `crates/wizard/src/encode.rs`. It never sees other hands.
- **What it can do.** 85 actions: 4 trump suits, bids 0 to 20, or one of the 60 cards. Illegal
  moves are masked out.

### How it's measured: duplicate deals and "edge"

Card games are noisy: a great player can lose with bad cards. To measure skill we use
**duplicate** deals, as in duplicate bridge. Every deal is replayed once with the bot in each
seat, and the other seats are all the same opponent. Over a full cycle the bot has held exactly
the cards its opponents held, so luck cancels out.

The number to watch is the **edge**: the bot's round score minus its opponents' average, per
round. An edge of +10 means about 10 points a round better than them on the same cards. Over a
15-round game that's about 150 points. A bot against an exact copy of itself has an edge of 0.

Two baselines:

- **random**: plays random legal moves. It's the floor.
- **counting**: a casual player. It bids by adding rough card values and plays greedily: win
  tricks while it needs them, duck once it has enough.

---

## Commands

All Python commands run from `rl/` with the venv active. All Rust commands run from the repo
root. Add `--help` to any of them for every option.

### Train

```sh
python -m wizard_rl.train --hours 8 --out runs/first                          # from scratch
python -m wizard_rl.train --hours 8 --out runs/first --resume runs/first/latest.pt   # continue a run
python -m wizard_rl.train --hours 5 --out runs/next --init runs/first/best.pt \
    --reference runs/first/best.pt --eps-start 0.05 --lr 2e-4 --lr-final 3e-5 --lr-decay 4.5e8   # build on a run
```

- `--init` copies a network's weights into a new run (new optimizer, new log). `--resume`
  continues a run exactly where it stopped.
- `--reference` also measures each checkpoint head-to-head against that network, so you can see
  whether the new run is actually beating the old one.
- `--in-turn` trains with bids in turn instead of all at once.
- `--players 4` trains on 4-player tables only (the default is 3,4,5,6).
- `--mix 0.7,0.2,0.1` sets the other seats (current / frozen copies / counting bots).
- `--lr`, `--lr-final`, `--lr-decay`: learning rate, decaying linearly to `--lr-final` over
  `--lr-decay` decisions. `--eps-start`, `--eps-end`, `--eps-decay` do the same for exploration.
- `--device auto` uses the Apple GPU (`mps`) if there is one. The network is small, so the CPU can
  be as fast; try both for a few minutes (`--decisions 5e6 --eval-every 5e6`) and keep the faster.
- `--threads N` caps the CPU threads PyTorch uses.

Every `--eval-every` decisions (default 20M) it prints a line like

```
[ 420.0 min]    616.8M   23691/s  loss 0.027/0.295  eps 0.020  pool 8 | vs counting: +12.1/round, bids made 75.3% (theirs 59.1%) | vs random: +36.5 | vs reference:  +5.9
```

That's minutes, decisions so far, decisions per second, training loss (score / make-bid),
exploration, frozen copies in the pool, then edges: against counting bots (with how often each
side made its bid), random bots, and the reference network. It appends the same to `runs/<name>/metrics.csv` and saves:

- `latest.pt`: the newest checkpoint, with the frozen pool (use for `--resume`). About 38 MB.
- `best.pt`: the checkpoint with the best edge against the counting bots so far. Also about 38 MB;
  `python -m wizard_rl.export runs/first/best.pt models/first.pt` keeps just the network (3.5 MB).

On a Mac, keep it awake for long runs: `caffeinate -i python -m wizard_rl.train ...`.

### Evaluate

```sh
python -m wizard_rl.evaluate runs/first/best.pt                          # vs counting bots, every table size
python -m wizard_rl.evaluate runs/first/best.pt --vs random
python -m wizard_rl.evaluate runs/next/best.pt --vs runs/first/best.pt   # head-to-head: new vs old
python -m wizard_rl.evaluate runs/ppo1/best.pt --sample --vs runs/first/best.pt   # a PPO bot, sampling its moves
```

It prints the edge per table size and overall, with an error bar (± two standard errors: if the
bar doesn't reach 0, the difference is real), plus average scores and how often bids were made.
`--rounds` sets rounds per table size (default 40,000).

### Export a network for Rust

```sh
python -m wizard_rl.export runs/first/best.pt models/first.wznet   # for Rust
python -m wizard_rl.export runs/first/best.pt models/first.pt      # slim checkpoint for Python
```

The `.wznet` file is the weights plus a small header (3.5 MB). The Rust engine runs it directly,
so it can go into an app later without Python. A `.pt` output drops the optimizer and frozen
pool from a training checkpoint (38 MB → 3.5 MB); every Python command accepts either.

### Play, watch, and simulate (Rust)

```sh
cargo run --release -p wizard -- play                                   # you vs 3 counting bots
cargo run --release -p wizard -- play --players 5 --advisor rl/models/simul1.wznet
cargo run --release -p wizard -- play --bots net:rl/models/simul1.wznet,counting,counting
cargo run --release -p wizard -- sim --games 2000 --bots net:rl/models/simul1.wznet,counting,counting,counting
cargo run --release -p wizard -- sim --games 2000 --bots chart:rl/charts/bid_chart.csv,counting,counting,counting
```

- `play`: you play a full game in the terminal. `--bots` lists your opponents; `--advisor` shows
  the bot's expected points (and chance of making the bid) for each of your options.
- `sim`: bots play each other; prints average game score, win rate and bid rate for each. The
  bots rotate seats game to game.
- Bot names: `random`, `counting`, `net:FILE` (a trained network), `chart:FILE` (bids from a bid
  chart CSV, plays like the counting bot).
- `--players 3..6`, `--seed N`, `--in-turn`.

### Ask it about a hand

```sh
python -m wizard_rl.charts runs/first/best.pt
```

Prints, for a list of situations, every bid with the network's expected points and chance of
making it. For example, "one card, K♥, hearts trump, 4 players, leading". Edit `SITUATIONS` in
`python/wizard_rl/charts.py` to ask about your own hands (cards are written `Ah`, `10s`, `wiz`,
`jes`; the seat is 1 = left of the dealer ... n = the dealer).

### Bid charts

```sh
python -m wizard_rl.bidchart runs/first/best.pt --out charts          # 3, 4, 5 and 6 players
```

See [Bid charts](#bid-charts) below.

### The PPO learner

```sh
python -m wizard_rl.ppo --hours 4 --init runs/first/best.pt --reference runs/first/best.pt --out runs/ppo1
```

See [below](#a-learner-that-outputs-probabilities-ppo).

### Tests

```sh
cargo test --release -p wizard     # rules engine: hand-worked tricks, scoring, fuzzing of random rounds
python tests/test_bridge.py        # Rust <-> Python bridge, export parity, evaluator
python -m wizard_rl.kuhn           # learner check on Kuhn poker (a tiny game with known answers), ~1 min
```

The Kuhn check trains the same DMC learner on a three-card poker game where the right values are
known exactly, and checks it learns them. If you change the learner, run it first.

---

## Results

<!-- RESULTS -->

---

## Bid charts

`python -m wizard_rl.bidchart MODEL` has the bot play 150,000 rounds against itself at each table
size (everyone bidding at once). It records every hand and how many tricks it took. Then it fits
a value for each **kind of card**:

| With trump | No trump |
| --- | --- |
| Wizard, Jester | Wizard, Jester |
| trump A, K, Q, J, 10–7, 6–2 | A, K, Q, J, 10–2 (any suit) |
| off-suit A, K, Q, J, 10–2 | |

For each table size and range of round sizes (1, 2, 3–4, 5–7, 8–10, 11–15, 16–20 cards), it
finds the values that best predict tricks taken (least squares). To bid, **add up your cards and
round**. Values change with round size. For example, an off-suit ace is often a trick with 3
cards in hand, but in a 12-card round it's more likely to get trumped.

Rounds with no trump (a Jester turned up, or the last round) get their own table.

It writes four files to `--out`:

- `bid_chart.md`: full chart, two decimals, with accuracy rows: *average miss*, *chart bid made*
  (how often the rounded total was exactly the tricks taken), *bot's own bid made*, and a small
  per-seat adjustment.
- `bid_cheat_sheet.md`: the same, rounded to quarters (¼, ½, ¾), for use at the table.
- `bid_chart.csv` / `bid_chart_quarters.csv`: the values as data. `--bots chart:FILE` plays with them.

The charts for the current model are in `charts/`.

---

## A learner that outputs probabilities (PPO)

The DMC bot always plays its single best-scoring move. A **policy** learner instead outputs a
probability for every move. It can **mix**, for example bidding 1 seventy percent of the time
and 2 thirty percent. In poker, mixing is essential (a player who always bets strong hands is easy
to read). `ppo.py` trains such a learner with Proximal Policy Optimization:

- **Two heads on one network.** A policy (probabilities over legal moves) and a value (expected
  round score from here).
- **The update.** After each batch of rounds, moves whose round went better than the value
  expected become more likely, and worse ones less likely. The *clip* keeps each update small, so
  it can't wreck itself.
- **Entropy bonus.** A small bonus keeps it from collapsing onto one move too early.
- **Starting point.** It starts from the DMC network (probabilities = softmax of DMC scores / 5
  points), so the question is fair: can it improve on the DMC bot from where it stands?

<!-- PPO -->

---

## Size and speed

- **Network:** 3 hidden layers of 512 units, 870,570 parameters. `.wznet` export: 3.5 MB.
  Slim `.pt`: 3.5 MB. Training checkpoints (`best.pt`, `latest.pt`, with optimizer state and
  8 frozen copies for resuming): about 38 MB each.
- **Engine:** 3–6 million decisions a second on one core, so the network is the bottleneck.
- **Training:** about 20–25 thousand decisions a second on 2 CPU cores. Hours of training give a
  strong bot. More cores or the Apple GPU can speed it up; memory use is under 2 GB.
- `runs/` holds checkpoints and logs and is not committed. `models/` holds exported networks
  and is committed.

---

## Files

```
rl/
  README.md                this file
  pyproject.toml           Python package; builds the Rust bridge with maturin
  models/                  exported networks (.wznet) and slim checkpoints
  charts/                  bid charts for the current model
  python/wizard_rl/
    net.py                 the Q-network (score per move + chance of making the bid); move picking
    dmc.py                 the Deep Monte Carlo learner, frozen pool, updates
    train.py               training runs: logging, evaluation, checkpoints
    evaluate.py            duplicate-deal edge of one player over others (library + command)
    export.py              checkpoint -> .wznet for Rust
    charts.py              the network's view of set bidding situations
    bidchart.py            bid charts from self-play
    ppo.py                 the policy (probabilities) learner, for comparison
    kuhn.py                learner check on Kuhn poker
  tests/test_bridge.py     Rust <-> Python checks
crates/wizard/             rules engine, bots, encoding, batch environment, Rust network runner
crates/wizard-py/          Python bindings (PyO3)
```

The engine side (`crates/wizard/src`):

- `rules.rs` / `round.rs` / `game.rs`: the rules, one round as a state machine, full games.
- `view.rs`: what a seat may see (bots get only this; other bids are hidden until everyone has bid).
- `encode.rs`: the 503 numbers the network sees and its 85 actions.
- `env.rs`: many tables at once for training; frozen-network seats; duplicate deals; stats.
- `bots.rs`, `chart.rs`, `net.rs`: random and counting bots, the chart bot, a trained network.
- `scenario.rs`: builds a bidding situation to ask a network about.
