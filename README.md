# Poker Study

A personal "chess.com for live poker": rated bots, puzzles and hand review for live $1/$2–$2/$5
NLHE. Built on the [postflop-solver](https://github.com/b-inary/postflop-solver) engine with node
locking. See [DESIGN.md](DESIGN.md) for the why and the roadmap.

**Every answer is one action.** No "call 55%". Against a known opponent the best play is pure. Against
GTO, mixed hands are indifferent, so the playbook keeps the range balanced by *which hands* take each
action instead.

## Quick start

```sh
cargo build --release
./target/release/ps spot spots/river_bluffcatch.toml --profile profiles/station.toml --hands
./target/release/ps spot spots/river_thin_value.toml --profile profiles/nit.toml
./target/release/ps spot spots/turn_barrel.toml --profile profiles/station.toml --intensity 0.5
cargo test --release      # includes the game-theory invariants
```

The output shows:

- **What the spot is worth**: GTO vs GTO, GTO vs this villain, best exploit, the exploit with one
  action per hand, and the worst case of a never-mix GTO playbook against a perfect opponent.
- **Your answer by hand class**: GTO playbook vs the exploit of this villain.
- **Villain's range when you decide**: GTO vs this profile. This is the "I have queens, but what
  does *he* have?" view.
- `--hands`: every hand type with its one answer, the alternatives that are also fine, and how
  much each other action costs.

## Layout

```
crates/ps-core    cards, evaluator, live-player hand classes
crates/ps-solve   spots, profiles, GTO vs node-locked exploit
crates/ps-cli     the `ps` command
crates/wizard     Wizard (the card game) engine for self-play RL; see its README
profiles/         opponent archetypes (TOML): gto, station, nit, maniac, whale
spots/            study spots (TOML)
```

Writing a profile or spot: see the doc comments in `crates/ps-solve/src/profile.rs` and
`crates/ps-solve/src/spot.rs`, and the examples in `profiles/` and `spots/`.

## Licence

AGPL-3.0-or-later, because the solver it builds on is AGPL.
