# wizard

A rules engine for Wizard, the trick-taking card game, built for self-play reinforcement
learning. Official rules by default; house rules are options.

```sh
cargo run --release -p wizard -- play                 # you vs 3 counting bots, official rules
cargo run --release -p wizard -- play --players 5 --house
cargo run --release -p wizard -- sim --games 20000 --bots counting,random,random,random
cargo test --release -p wizard
```

## Rules

| Rule | Official (default) | House option (`--house`) |
| --- | --- | --- |
| Deck | 52 standard cards, 4 Wizards, 4 Jesters | |
| Players | 3 to 6 | up to 8 |
| Rounds | round r deals r cards: 20 / 15 / 12 / 10 rounds for 3 / 4 / 5 / 6 players | |
| Trump | the next undealt card; none if no card is left | last round never has trump |
| Wizard turned up | dealer names trump | player on the dealer's right names it (or a random suit) |
| Jester turned up | no trump | player on the dealer's right names it (or a random suit) |
| Bids | left of the dealer first; 0 to r; may add up to anything | |
| Following | follow the suit led if you can; Wizards and Jesters any time; Wizard led: anything; Jester led: the first standard card sets the suit | |
| Trick | first Wizard; else highest trump; else highest of the suit led; all Jesters: the first | |
| Score | exact: 20 + 10 per trick; missed: −10 per trick off | |

## Layout

- `card.rs`: the deck (cards are `u8`, hands are `u64` bitmasks)
- `rules.rs`: official rules and house options
- `round.rs`: one round as a state machine, the trick-winner and follow-suit rules, scoring
- `game.rs`: full games, the deal rotating left
- `view.rs`: what one seat may see (bots only get a `View`)
- `bots.rs`: `RandomBot` (the floor) and `CountingBot` (a casual player: counts likely tricks, plays greedily)
- `tests/rules.rs`: hand-worked tricks and rounds, and fuzzing of thousands of random rounds
  under every rule set

Speed: about 3 to 6 million decisions a second on one core, so the engine is never the
bottleneck for training.
