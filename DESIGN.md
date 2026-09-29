# Poker Study — design

A free, personal "chess.com for live poker": play rated bots, do puzzles, review your hands.
Built for live $1/$2–$2/$5 NLHE, aimed at getting to $5/$10 without donating.

> GTO is the baseline. Exploitation is the adaptation. Every answer is one action.

## 1. What it is

| Chess habit | Poker Study |
|---|---|
| Engine | `postflop-solver` (Rust, Discounted CFR, node locking) |
| Bots rated 250 → 2200 | Bot = GTO strategy bent by an opponent **profile** at some **intensity** λ |
| Puzzles | One spot, one villain, one hand, and a single correct answer |
| Game review ("you played like an 1100") | EV lost per decision, graded twice: vs GTO and vs the best exploit of *that* bot |
| Openings / theory | The Boot Camp curriculum (Track D: Exploit the Villain) |

Phone-first PWA (from LiveGTO), running off a precomputed library of solves. The laptop can
re-solve any spot live for deep review.

## 2. Principle: one action, never frequencies

Live, nobody randomises. Two facts make pure answers correct rather than a simplification:

1. **Against a known opponent the best response is pure.** With villain's strategy σ_V fixed,
   hero solves a one-player problem, BR(σ_V) = argmax_σH u_H(σ_H, σ_V), and a one-player problem
   always has a deterministic optimum. Every hand has exactly one best action (ties are real
   indifference: either is fine).
2. **Against GTO, mixing only happens where you're indifferent.** A hand that mixes has equal EV
   for its actions, so no single choice is a mistake. What matters is the *range*: if every
   bluff-catcher folds, a thinking opponent bluffs you off every pot. So the GTO playbook keeps the
   range-level frequencies and hands each action to the hands that lean towards it most: "call
   with the best 40% of your bluff-catchers, fold the rest". That's how strong live players stay
   balanced without dice.

The trainer reports how much never-mixing costs, so this stays a measured choice:

- `Exploit, one action per hand` vs the best exploit: ≈ 0 by (1). Verified by test.
- `GTO playbook vs a perfect opponent`: the worst case if someone knew your exact playbook, which
  no $1/$2 player does. So far: 2.5–3bb in a ~20bb river pot, but 8.6bb in a 16bb turn pot,
  where the playbook covers two streets. Improving the purification (for example choosing
  hands by EV threshold instead of greedy quotas) is an open item.

Answers are graded by EV, not by matching frequencies. Any action within `fine_pct` (default 1%) of
the pot of the best one counts as correct, and the grader shows it as "also fine".

## 3. Profiles: human leaks as edits to GTO

A profile is a TOML list of rules. Each rule says where it applies (street, facing a bet or not,
the size faced, hand class) and how it bends the GTO frequencies:

- `scale = { fold = 0.25 }`: "folds a quarter as often as a solver". The frequency of the
  matched actions is multiplied by m^λ (capped at 1), and the other actions absorb the difference
  in proportion to their GTO weight:
  σ'(S|h) = min(1, m^λ · σ(S|h))
- `set = { raise = 0.7, call = 0.3 }`: "raises his draws 70% of the time", whatever GTO does:
  σ' = (1 − λ)·σ + λ·target

Hand classes are what a live player sees: `monster, strong, medium, weak, draw, air`
(see `ps-core`). Action classes: `fold, check, call, bet, bet_small, bet_large, raise, allin,
aggressive, passive`. Facing sizes: `small` (< ½ pot), `large` (½–1 pot), `overbet`.

Shipped profiles: `gto`, `station`, `nit`, `maniac`, `whale`. **They are hypotheses, not facts.**
The exploit is only as good as the read. The first calibration mistake is instructive: a
"folds ¼ as often" rule, applied to a 5×-pot river shove, turned a station into someone who calls
off 90bb with second pair. That's why `facing_size` exists. Tune profiles against what you see at
the table.

## 4. What one spot analysis computes

Five copies of the same tree, lined up node-for-node by action history:

| game | hero | villain | answers |
|---|---|---|---|
| A | solved | solved | GTO baseline |
| B | solved | locked = profile | best exploit of this villain |
| C | locked = GTO | locked = profile | what GTO alone wins against him |
| P | locked = one-action exploit | locked = profile | exploit with no mixing |
| Q | locked = GTO playbook | free (best response) | worst case of never mixing |

Invariants (tested in `crates/ps-solve/tests/invariants.rs`). v = GTO value, ε = exploitability:

1. GTO guarantee: EV(GTO vs any villain) ≥ v − ε
2. Best response dominates: EV(BR vs P) ≥ EV(GTO vs P)
3. Convergence: the solved exploit matches the exact best-response value (`compute_mes_ev`)
4. Pure is enough: the one-action exploit ≈ the best exploit
5. No free lunch: a fixed playbook vs a perfect opponent ≤ v + ε
6. Identity: λ = 0 or the GTO profile changes nothing
7. Zero-sum (no rake); plus a hand-solvable toy game where the right answer can be checked by hand

A lesson already visible in the numbers: against a Station on the river, GTO alone gains about
0.1–0.5bb from his leaks, and *adjusting* gains 4–7bb. GTO doesn't punish leaks where the villain's
choices were indifferent. You only collect by changing your play.

## 5. Ratings

Poker isn't win/lose and it isn't transitive (two fish leak in different ways), so everything is
anchored to one yardstick: the GTO bot. Let L be a player's loss rate against the GTO bot, in
bb/100:

```
R = 2200 − 400 · log₂(1 + L / L₀)
```

Every 400 points roughly doubles how fast you bleed. With L₀ = 5 bb/100:

| L (bb/100 lost to GTO) | 0 | 2 | 5 | 15 | 35 | 75 | 155 |
|---|---|---|---|---|---|---|---|
| R | 2200 | ~2000 | 1800 | 1400 | 1000 | 600 | 200 |

So a 1000 bot bleeds about 17× faster than a 2000 bot.

- **Bots**: R(profile, λ) is measured by simulating hands against the GTO bot, not assumed. The
  ladder picks λ per profile to hit target ratings (Station 600, Station 1000, Nit 1400, …).
- **Your review rating**: the same formula, using your EV loss per decision (×100 hands) against
  the GTO reference. A second number shows loss against the *exploit* reference, because beating
  a 600 whale means playing unlike GTO, and the review should reward that.
- L₀ is set once from calibration so the bands feel right. It's a display scale, not physics.

## 6. Architecture

```
crates/
  ps-core    cards, exact evaluator, live-player hand classes          [done]
  ps-solve   spots, profiles, GTO vs node-locked exploit, pure answers [done]
  ps-cli     `ps spot <spot.toml> --profile <p.toml> [--hands --json]`   [done]
  ps-engine  full-hand HU NLHE engine (play mode + RL env)               [next]
  ps-bots    bots that play whole hands: preflop charts + library + profile
  ps-library batch-solve spot families → compact JSON for the app
app/         LiveGTO frontend folded in (PWA, drills, simulate, review)
rl/          self-play research track
spots/ profiles/
```

`postflop-solver` is AGPL-3.0, so this repo is AGPL-3.0 too. That's fine for personal use. If the
app is ever served to other people, its source has to be offered to them.

## 7. Roadmap

1. **Spot solver** (done). Turn/river spots solve in seconds.
2. **Library + puzzles.** Spot families (SRP BTN vs BB, SRP CO vs BTN, 3-bet pots), sampled boards,
   turn/river decision points, × profiles. A puzzle is a (spot, villain, hand) whose best action
   beats the next best by a clear EV margin: an "only move". Store per-hand answers + EV
   losses + one-line explanations drawn from the profile rule notes.
3. **Fold in LiveGTO.** Move its UI into `app/` and swap its bucketed CFR+ data for the
   library. Puzzle mode first (phone, offline).
4. **Engine + bots + ladder.** Deterministic HU engine with the invariants from the original plan
   (chip conservation, side pots, min-raise rules, seeded RNG). Bots play full hands. Calibrate
   ratings, then build Play with post-hand review.
5. **Flop + preflop.** Flop trees are too large to lock node-by-node through the interpreter
   (hundreds of thousands of nodes). Options: lock only the current street, then re-lock per
   runout, or fork the solver to transform strategies in its storage. Preflop starts as charts per
   profile plus a steal calculator for the "nobody's raising, open A8o" adjustment:
   EV(open) = P(all fold)·(blinds) + (1 − P(all fold))·EV(called).
6. **RL track** (for the science). Self-play agent in `rl/` on `ps-engine`. Validate the
   algorithm on Leduc against an exact CFR solution first. Then run heads-up NLHE with the small
   action set and put it on the same ladder: "rated 1350 after N hours of training".

## 8. Open questions

- Hand classes are coarse. Blockers and kickers matter for the "which bluff-catchers call" split.
  The per-hand rows already capture this; the classes are for explanations.
- Profile calibration from real observations (session notes → rule multipliers).
- Multiway pots: live $1/$2 is often multiway preflop. The first versions stay heads-up
  postflop, where most of the study value is.
