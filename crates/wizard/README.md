# wizard

A rules engine for Wizard, the trick-taking card game, built for self-play reinforcement
learning, playing the official rules. By default everyone bids at once (bids hidden until all
are in); `--in-turn` bids in turn as printed. Training, results and the full guide:
[`rl/README.md`](../../rl/README.md).

```sh
cargo run --release -p wizard -- play                 # you vs 3 counting bots
cargo run --release -p wizard -- play --players 5
cargo run --release -p wizard -- sim --games 20000 --bots counting,random,random,random
cargo run --release -p wizard -- play --advisor rl/models/simul1.wznet      # a trained network's advice
cargo run --release -p wizard -- sim --bots net:rl/models/simul1.wznet,counting,counting,counting
cargo run --release -p wizard -- sim --bots chart:rl/charts/bid_chart.csv,counting,counting,counting
cargo test --release -p wizard
```

## Rules (official)

| Rule | |
| --- | --- |
| Deck | 52 standard cards, 4 Wizards, 4 Jesters |
| Players | 3 to 6 |
| Rounds | round r deals r cards: 20 / 15 / 12 / 10 rounds for 3 / 4 / 5 / 6 players |
| Trump | the next undealt card; a Wizard: the dealer names trump; a Jester: no trump; last round (no card left): no trump |
| Bids | all at once by default (nobody sees another bid until all are in); `--in-turn`: left of the dealer first; 0 to r; may add up to anything |
| Following | follow the suit led if you can; Wizards and Jesters any time; Wizard led: anything; Jester led: the first standard card sets the suit |
| Trick | first Wizard; else highest trump; else highest of the suit led; all Jesters: the first |
| Score | exact: 20 + 10 per trick; missed: −10 per trick off |

## Layout

- `card.rs`: the deck (cards are `u8`, hands are `u64` bitmasks)
- `rules.rs`: the official rules (table size, and bids all at once or in turn)
- `round.rs`: one round as a state machine, the trick-winner and follow-suit rules, scoring
- `game.rs`: full games, the deal rotating left
- `view.rs`: what one seat may see (bots only get a `View`)
- `bots.rs`: `RandomBot` (the floor) and `CountingBot` (a casual player: counts likely tricks, plays greedily)
- `chart.rs`: `ChartBot`, which bids from a bid chart CSV and plays like `CountingBot`
- `encode.rs`: what the network sees (503 numbers from the acting seat's point of view) and its 85 actions
- `env.rs`: a batch of tables for training; the learner's seats wait for Python, bots play the rest
- `net.rs`: runs an exported network (`.wznet`) in Rust: `NetBot`, `--bots net:FILE`, `--advisor FILE`
- `scenario.rs`: builds a bidding situation (hand, trump, seat) to ask a network about
- `tests/rules.rs`: hand-worked tricks and rounds, and fuzzing of thousands of random rounds
  at every table size

Training lives in `../../rl` (Python, PyTorch), through the bindings in `../wizard-py`.

Speed: about 3 to 6 million decisions a second on one core, so the engine is never the
bottleneck for training.
