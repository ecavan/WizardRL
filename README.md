# Wizard RL

A bot that teaches itself Wizard, the trick-taking card game, by playing millions of rounds
against copies of itself. It uses the official rules for 3 to 6 players, and by default everyone
bids at the same time. Once trained, you can:

- play against it, or have it whisper advice while you play, in the terminal;
- ask what it would bid with any hand;
- print **bid charts** (how many tricks each card is worth, by table size and round size);
- measure it against simple bots and against other versions of itself.

The game engine is Rust (`crates/wizard`). Training is Python and PyTorch (`rl/`). The two talk
through a small bridge (`crates/wizard-py`). A trained network is a 3.5 MB file that runs
in Rust with no Python needed.

**Contents:** [Quick start](#quick-start) · [The rules it plays](#the-rules-it-plays) ·
[How it learns](#how-it-learns) · [Commands](#commands) · [Results](#results) ·
[Bid charts](#bid-charts) · [Probabilities vs best move (PPO)](#a-learner-that-outputs-probabilities-ppo) ·
[Playing to win](#playing-to-win-the-game) · [How beatable is it?](#how-beatable-is-it) ·
[Look-ahead search](#look-ahead-search) · [Player styles](#player-styles) · [Size and speed](#size-and-speed) · [Files](#files) ·
[What's next](#whats-next)

---

## Quick start

You need Rust (install with [rustup](https://rustup.rs)) and Python 3.9 or newer. From the repo root:

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

The official rules ([`crates/wizard/README.md`](crates/wizard/README.md) has the full table). The short
version:

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
- **What it sees.** 532 numbers from its own seat's point of view: its hand, trump, cards
  played so far, the current trick, bids and tricks won by everyone (bids are hidden until
  everyone has bid), which suits each player has shown they're out of, and in a full game the scores so far and
  each player's habits this game (see [Player styles](#player-styles)). See
  [`crates/wizard/src/encode.rs`](crates/wizard/src/encode.rs). It never sees other hands.
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
  continues a run exactly where it stopped; `latest.pt` is saved every `--save-every` decisions
  (default 5M, a few minutes), so a crash loses little.
- `--hours`, `--decisions` (more decisions) or `--until` (total decisions, counted across
  resumes) say when to stop.
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
- `--game`: play **full games** and reward **winning the game** instead of each round's score
  (see [Playing to win the game](#playing-to-win-the-game)). `--win-weight 0.8` mixes in 20%
  "share of opponents finished ahead of". Evaluation then reports points per game and win rates.
- `--styles overbid,underbid,early-wizard,wild,plain` (with `--game`): most other seats are
  `--style-net` (default: the `--init` network) playing with one of those habits, so the bot
  learns to spot and exploit them. See [Player styles](#player-styles).
- `--opponent FILE`: **exploiter** mode. Every other seat is that fixed network and nothing else,
  so the learner learns to beat it specifically (see [How beatable is it?](#how-beatable-is-it)).

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
python -m wizard_rl.evaluate runs/game1/best.pt --game --rounds 4000 --vs runs/first/best.pt   # full games: points per game, win %
python -m wizard_rl.evaluate runs/game1/best.pt --game --vs runs/first/best.pt --vs-style overbid   # vs a table of overbidders
```

It prints the edge per table size and overall, with an error bar (± two standard errors: if the
bar doesn't reach 0, the difference is real), plus average scores and how often bids were made.
`--rounds` sets rounds per table size (default 40,000).

### League table (strength against strong players)

```sh
python -m wizard_rl.league --rows models/simul1.pt runs/ppo1/best.pt@sample \
    --tables counting runs/night1/best.pt models/simul1.pt@soft10 models/simul1.pt --players 4
```

Each cell has one row player against a table where every other seat is the column player, in
duplicate full games. It shows the row player's win rate (a fair share is 1/players) and its
margin per game, with 95% intervals. Players are written `PATH[@style]`:

- `@sample`: a PPO policy that samples its probabilities.
- `@soft10`: a strong but imperfect, human-like player. It picks moves at random in proportion
  to exp(points / 10), so it usually takes the best move or a near-tie, and rarely a clearly bad
  one. `@soft5` plays closer to perfect, `@soft25` sloppier.
- `@overbid`, `@underbid`, `@early-wizard`, `@wild`: players with a habit.
- `counting`, `random`: the built-in bots.

The diagonal (a player against itself) should read exactly a fair share, which is a built-in
check.

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
- `sim`: bots play full games against each other as **duplicate games**. Each deal is played
  once in every seating, so every bot holds every hand and luck mostly cancels. For each bot it
  prints the average game score, its margin over the rest of the table, its win rate and how
  often it made its bid, with **95% confidence intervals**.
- Bot names:
  - `random`, `counting`;
  - `net:FILE`: a trained network;
  - `chart:FILE`: bids from a bid chart CSV, plays like the counting bot;
  - `search:FILE[:samples[:width]]`: a network that **looks ahead before bidding**. See
    [Look-ahead search](#look-ahead-search).
  - `style:NAME:BOT`: any bot with a habit (`overbid`, `underbid`, `early-wizard`, `wild`),
    e.g. `style:early-wizard:net:rl/models/simul1.wznet` for "plays like my sister".
- `--players 3..6`, `--seed N`, `--in-turn`.

### Ask it about a hand

```sh
python -m wizard_rl.charts runs/first/best.pt
```

Prints, for a list of situations, every bid with the network's expected points and chance of
making it. For example, "one card, K♥, hearts trump, 4 players, leading". Edit `SITUATIONS` in
`rl/python/wizard_rl/charts.py` to ask about your own hands (cards are written `Ah`, `10s`, `wiz`,
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

**How to read these numbers.** A win rate only means something next to the opponent it was
measured against. The counting bot is a weak rule-based player (at 3 players it makes a third of
its bids and averages a negative score), so results against it are a **floor**, not a claim about
real games. The meaningful numbers are against strong networks, against human-like imperfect
versions of them (`@soft10`), and the exploiter test. How strong a real family table is can only
be measured with real games.

Sanity checks that pass: the bot only ever sees its own cards (a test enforces it); against three
copies of itself it wins exactly 25.0% with a margin of exactly 0; the Python evaluation and the
Rust full-game simulator agree.

### The current network: `models/simul1` (bids all at once)

Built on `night1` with 290M more decisions of self-play with simultaneous bids.

Full games, 4 players, duplicate deals (± is a 95% interval):

| Opponents (3 seats) | Bot's win rate (fair share 25%) | Its margin per game | Games |
| --- | --- | --- | --- |
| itself (check) | 25.0% | 0 | 800 |
| `night1` (older network, trained bidding in turn) | 46 ± 5% | +88 ± 11 | 400 |
| 1 `night1` + 2 copies of itself with habits (overbid, wild) | 57 ± 4% | +139 ± 9 | 400 |
| counting bots (floor) | 87 ± 1% | +223 ± 4 | 2000 |

Against counting bots (the floor), by table size:

| Players | 3 | 4 | 5 | 6 |
| --- | --- | --- | --- | --- |
| Bot's win rate (fair share) | 99.9% (33%) | 87% (25%) | 63% (20%) | 51% (17%) |
| Margin per game | +731 | +223 | +106 | +74 |

Per round, duplicate deals (± two standard errors):

| | 3 players | 4 | 5 | 6 | all |
| --- | --- | --- | --- | --- | --- |
| vs `night1` | +11.1 | +6.1 | +3.3 | +2.0 | **+5.6 ± 0.2** |
| vs counting bots (floor) | +37.2 | +14.8 | +9.0 | +7.5 | +17.1 ± 0.3 |

The edge shrinks as the table grows. With more players there are more hands to beat, and each
decision matters less.

<!-- LEAGUE -->

### The bid chart on its own

A bot that bids by the chart and plays like the counting bot (`chart:rl/charts/bid_chart.csv`), against counting bots:

| Players | Chart bot's margin per game | Its win rate (fair share) |
| --- | --- | --- |
| 3 | +198 ± 6 | 69% (33%) |
| 4 | +21 ± 3 | 30% (25%) |
| 5 | +9 ± 2 | 23% (20%) |

So the chart's bids alone beat the counting bot's rough card values. Most of the network's edge
still comes from how it plays the cards.

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

It writes these files to `--out`:

- `bid_chart.md`: full chart, two decimals, with accuracy rows: *average miss*, *chart bid made*
  (how often the rounded total was exactly the tricks taken), *bot's own bid made*, and a small
  per-seat adjustment.
- `bid_cheat_sheet.md`: the same, rounded to tenths, for use at the table. (Quarters were too
  coarse: in a 15-card round, a low card worth 0.1 rounds to 0, and a dozen of them add up to
  more than a trick.)
- `bid_chart.html`: an interactive page. Pick the table size and trump, read the chart, and tap in
  your hand to get the bid. Open it in any browser.
- `bid_chart.csv` / `bid_chart_simple.csv` / `bid_chart.json`: the values as data.
  `--bots chart:FILE` plays with a CSV.

The charts for the current model are in [`rl/charts/`](rl/charts/).

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

## Playing to win the game

The main bot maximizes each round's score. In a real game, what matters is **winning**: a
player 80 points behind with two rounds left should gamble on a big bid, and a leader should
play safe. With `--game`:

- **Full games.** Rounds of 1, 2, 3 ... cards are played, the deal moves left, and scores add
  up.
- **More to see.** The network also sees the game so far: everyone's score, its margin over the
  best other player, how many players are ahead of it, and the rounds left.
- **The reward** for every decision is how the game ended for that seat: 100 for a win (shared on
  a tie), 0 otherwise.

A game-trained network starts from the round-score network: the new inputs start at zero weight,
so it begins by playing the same way.

<!-- GAME -->

## How beatable is it?

Against a copy of itself the bot's edge is exactly zero, so that says nothing. The real
question is how much a player could win **if they knew exactly how it plays**. That's the
exploiter test: train a new network (`--opponent`) whose only job is to beat the frozen main bot,
with every other seat being that bot. The exploiter's final edge measures how exploitable the
bot is, in the spirit of "distance from GTO" in poker. A small edge means there's no easy hole to
find.

<!-- EXPLOIT -->

## Look-ahead search

`search:FILE` bids the way a careful player thinks it through. For each of the network's
favourite bids (3 by default), it deals the cards it can't see at random many times (32 by
default). It plays each imagined round to the end with the network making every other decision,
then bids whatever scored best on average. Every candidate is tried on the same imagined deals,
so the comparison is sharp. The imagined rounds all advance together, so the network runs on
big batches. That takes about a second per game on one core. Play after the bid is the plain
network.

<!-- SEARCH -->

## Player styles

The original goal: notice how a particular person plays, and exploit it. Four habits are built
in, each on top of a strong network:

| Style | Habit |
| --- | --- |
| `overbid` | bids one more than it should (70% of the time) |
| `underbid` | bids one fewer (70%) |
| `early-wizard` | throws a Wizard on the round's first trick whenever it can (90%) |
| `wild` | 20% of its decisions are random |

To spot a habit, the bot sees, for every player, how they've played so far in the game: their
average (bid − tricks won), how often they made their bid, and what share of their Wizards went
on a first trick. Training with `--game --styles ...` fills most seats with styled players, so
the bot learns to read the table during a game and adjust. For example, against an overbidder it
can take tricks off them to push them further over. `--vs-style` measures it against a whole
table of one style.

<!-- STYLES -->

---

## Size and speed

- **Network:** 3 hidden layers of 512 units, 870,570 parameters. `.wznet` export: 3.5 MB.
  Slim `.pt`: 3.5 MB. Training checkpoints (`best.pt`, `latest.pt`, with optimizer state and
  8 frozen copies for resuming): about 38 MB each.
- **Engine:** 3–6 million decisions a second on one core, so the network is the bottleneck.
- **Training:** about 20–25 thousand decisions a second on 2 CPU cores. Hours of training give a
  strong bot. More cores or the Apple GPU can speed it up; memory use is under 2 GB.
- `rl/runs/` holds checkpoints and logs and is not committed. `rl/models/` holds exported
  networks and is committed.

---

## Files

```
README.md                  this guide
crates/wizard/             the game: rules engine, bots, what the network sees, the batch
                           environment, and a trained network running in Rust (see its README)
crates/wizard-py/          Python bindings for the engine (PyO3, built by maturin)
rl/
  pyproject.toml           the Python package; builds the Rust bridge with maturin
  models/                  exported networks (.wznet) and slim checkpoints (.pt)
  charts/                  bid charts for the current model
  python/wizard_rl/
    net.py                 the network (score per move + chance of making the bid); move picking
    dmc.py                 the Deep Monte Carlo learner, frozen pool, updates
    train.py               training runs: logging, evaluation, checkpoints
    evaluate.py            duplicate-deal edge of one player over others (library + command)
    export.py              checkpoint -> .wznet for Rust, or a slim .pt
    charts.py              the network's view of set bidding situations
    bidchart.py            bid charts from self-play
    chartpage.py/.html     the interactive bid chart page
    ppo.py                 the policy (probabilities) learner, for comparison
    styles.py              habits for opponent seats (overbid, underbid, early Wizards, wild)
    kuhn.py                learner check on Kuhn poker
  tests/test_bridge.py     Rust <-> Python checks
```

The engine side (`crates/wizard/src`):

- `rules.rs`, `round.rs`, `game.rs`: the rules, one round as a state machine, full games.
- `view.rs`: what a seat may see (bots get only this; other bids are hidden until everyone has bid).
- `encode.rs`: the 532 numbers the network sees (503 about the round, 11 about the game score,
  18 about each player's habits) and its 85 actions.
- `style.rs`: players with habits.
- `env.rs`: many tables at once for training; frozen-network seats; duplicate deals; stats.
- `bots.rs`, `chart.rs`, `net.rs`: the random and counting bots, the chart bot, a trained network.
- `search.rs`: the look-ahead bidder.
- `scenario.rs`: builds a bidding situation to ask a network about.

---

## What's next

- An app to play against it and get advice at the table.
- Longer runs and a bigger network, if the results keep improving with more training.

---

## Licence

MIT (see [LICENSE](LICENSE)).
