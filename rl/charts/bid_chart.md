# Wizard bid chart

Made from `ppo5.pt` playing 200,000 rounds against itself at each table size, everyone bidding at once.

**How to use it:** find your table size, whether there's trump, and how many cards you hold. Add up the values of your cards and round to the nearest whole number. That's your bid.

- Values are tricks: 0.5 means the card wins a trick about half the time.
- *average miss*: how far the chart total is from the tricks actually taken, on average.
- *chart bid made*: how often the rounded total was exactly right (the bid would have been made).
- *bot's own bid made*: the same for the bot's real bids, which also weigh everything else it sees.
- *Seat*: add this for your seat (it's small; the dealer usually gains a little from playing last to the first trick).
- Jesters are worth about 0 tricks, but they are far from useless: *each Jester* shows how much one raises your chance of making your bid (you can always duck a trick with it). Wizards do the same the other way (you can always take one).

### 3 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards | 16-20 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.95 | 0.93 | 0.92 | 0.97 | 1.00 | 1.09 | 1.19 |
| Jester | -0.00 | -0.07 | -0.10 | -0.06 | -0.02 | -0.02 | -0.04 |
| trump A | 0.89 | 0.92 | 0.98 | 0.95 | 0.92 | 0.95 | 0.97 |
| trump K | 0.84 | 0.84 | 0.90 | 0.88 | 0.86 | 0.87 | 0.90 |
| trump Q | 0.84 | 0.86 | 0.87 | 0.87 | 0.79 | 0.82 | 0.85 |
| trump J | 0.76 | 0.80 | 0.84 | 0.82 | 0.78 | 0.76 | 0.78 |
| trump 10 | 0.69 | 0.74 | 0.79 | 0.76 | 0.71 | 0.71 | 0.71 |
| trump 9 | 0.71 | 0.71 | 0.75 | 0.72 | 0.68 | 0.68 | 0.67 |
| trump 8 | 0.67 | 0.71 | 0.73 | 0.69 | 0.65 | 0.62 | 0.61 |
| trump 7 | 0.68 | 0.71 | 0.69 | 0.67 | 0.63 | 0.60 | 0.59 |
| trump 6 | 0.65 | 0.64 | 0.66 | 0.60 | 0.58 | 0.57 | 0.55 |
| trump 5 | 0.61 | 0.63 | 0.63 | 0.57 | 0.56 | 0.54 | 0.51 |
| trump 4 | 0.61 | 0.60 | 0.61 | 0.55 | 0.52 | 0.52 | 0.49 |
| trump 3 | 0.55 | 0.59 | 0.56 | 0.52 | 0.50 | 0.53 | 0.49 |
| trump 2 | 0.58 | 0.57 | 0.50 | 0.47 | 0.47 | 0.48 | 0.46 |
| off-suit A | 0.29 | 0.34 | 0.40 | 0.54 | 0.61 | 0.71 | 0.79 |
| off-suit K | 0.26 | 0.30 | 0.34 | 0.42 | 0.46 | 0.52 | 0.60 |
| off-suit Q | 0.26 | 0.28 | 0.30 | 0.34 | 0.35 | 0.39 | 0.44 |
| off-suit J | 0.22 | 0.25 | 0.26 | 0.27 | 0.28 | 0.28 | 0.30 |
| off-suit 10 | 0.22 | 0.24 | 0.24 | 0.23 | 0.21 | 0.20 | 0.19 |
| off-suit 9 | 0.20 | 0.19 | 0.21 | 0.19 | 0.18 | 0.15 | 0.13 |
| off-suit 8 | 0.18 | 0.19 | 0.19 | 0.16 | 0.14 | 0.10 | 0.06 |
| off-suit 7 | 0.17 | 0.18 | 0.16 | 0.14 | 0.12 | 0.07 | 0.03 |
| off-suit 6 | 0.14 | 0.16 | 0.14 | 0.10 | 0.09 | 0.05 | 0.00 |
| off-suit 5 | 0.13 | 0.11 | 0.10 | 0.08 | 0.07 | 0.03 | -0.02 |
| off-suit 4 | 0.12 | 0.12 | 0.08 | 0.05 | 0.04 | 0.00 | -0.04 |
| off-suit 3 | 0.13 | 0.07 | 0.04 | 0.02 | 0.03 | 0.00 | -0.04 |
| off-suit 2 | 0.12 | 0.04 | 0.01 | -0.02 | -0.02 | -0.03 | -0.07 |
| *average miss (tricks)* | 0.29 | 0.42 | 0.53 | 0.59 | 0.60 | 0.61 | 0.55 |
| *chart bid made* | 81% | 70% | 59% | 56% | 57% | 55% | 57% |
| *bot's own bid made* | 80% | 72% | 65% | 61% | 61% | 61% | 64% |
| *each Jester: chance to make your bid* | +26% | +30% | +17% | +15% | +12% | +11% | +11% |
| *each Wizard: chance to make your bid* | +21% | +19% | +8% | +9% | +8% | +6% | +6% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards | 16-20 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.93 | 1.13 | 1.17 | 1.15 | 1.08 | 1.15 | 1.38 |
| Jester | 0.00 | -0.15 | -0.29 | -0.22 | -0.17 | -0.24 | -0.33 |
| off-suit A | 0.42 | 0.60 | 0.73 | 0.79 | 0.84 | 0.85 | 0.96 |
| off-suit K | 0.44 | 0.55 | 0.62 | 0.71 | 0.74 | 0.74 | 0.83 |
| off-suit Q | 0.40 | 0.42 | 0.48 | 0.58 | 0.60 | 0.61 | 0.70 |
| off-suit J | 0.39 | 0.49 | 0.42 | 0.48 | 0.43 | 0.49 | 0.57 |
| off-suit 10 | 0.35 | 0.33 | 0.40 | 0.33 | 0.36 | 0.38 | 0.39 |
| off-suit 9 | 0.33 | 0.30 | 0.28 | 0.28 | 0.28 | 0.28 | 0.30 |
| off-suit 8 | 0.27 | 0.27 | 0.30 | 0.24 | 0.20 | 0.21 | 0.16 |
| off-suit 7 | 0.27 | 0.32 | 0.27 | 0.18 | 0.17 | 0.15 | 0.11 |
| off-suit 6 | 0.23 | 0.13 | 0.20 | 0.13 | 0.11 | 0.11 | 0.05 |
| off-suit 5 | 0.25 | 0.13 | 0.12 | 0.10 | 0.14 | 0.09 | 0.02 |
| off-suit 4 | 0.19 | 0.17 | 0.09 | 0.08 | 0.08 | 0.06 | -0.03 |
| off-suit 3 | 0.21 | 0.10 | 0.05 | 0.04 | 0.02 | 0.01 | -0.04 |
| off-suit 2 | 0.16 | 0.10 | 0.01 | -0.01 | 0.00 | -0.03 | -0.09 |
| *average miss (tricks)* | 0.37 | 0.61 | 0.76 | 0.72 | 0.70 | 0.67 | 0.57 |
| *chart bid made* | 73% | 50% | 40% | 47% | 50% | 51% | 55% |
| *bot's own bid made* | 84% | 72% | 59% | 56% | 56% | 59% | 64% |
| *each Jester: chance to make your bid* | +22% | +30% | +24% | +20% | +17% | +12% | +8% |
| *each Wizard: chance to make your bid* | +14% | +19% | +4% | +8% | +9% | +7% | +4% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.00; seat 2 -0.04; dealer +0.04.

### 4 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.93 | 0.94 | 0.94 | 0.99 | 1.06 | 1.16 |
| Jester | -0.00 | -0.04 | -0.07 | -0.05 | 0.00 | 0.04 |
| trump A | 0.80 | 0.91 | 0.95 | 0.93 | 0.91 | 0.94 |
| trump K | 0.76 | 0.82 | 0.88 | 0.85 | 0.81 | 0.83 |
| trump Q | 0.72 | 0.77 | 0.82 | 0.80 | 0.76 | 0.73 |
| trump J | 0.69 | 0.72 | 0.75 | 0.73 | 0.70 | 0.67 |
| trump 10 | 0.66 | 0.68 | 0.72 | 0.69 | 0.64 | 0.61 |
| trump 9 | 0.61 | 0.67 | 0.68 | 0.66 | 0.60 | 0.58 |
| trump 8 | 0.57 | 0.64 | 0.64 | 0.60 | 0.54 | 0.52 |
| trump 7 | 0.54 | 0.60 | 0.58 | 0.56 | 0.51 | 0.49 |
| trump 6 | 0.52 | 0.57 | 0.53 | 0.51 | 0.48 | 0.46 |
| trump 5 | 0.50 | 0.42 | 0.45 | 0.46 | 0.45 | 0.44 |
| trump 4 | 0.45 | 0.36 | 0.38 | 0.41 | 0.42 | 0.42 |
| trump 3 | 0.46 | 0.29 | 0.31 | 0.38 | 0.41 | 0.41 |
| trump 2 | 0.38 | 0.24 | 0.25 | 0.34 | 0.37 | 0.39 |
| off-suit A | 0.20 | 0.20 | 0.25 | 0.39 | 0.53 | 0.68 |
| off-suit K | 0.17 | 0.19 | 0.21 | 0.25 | 0.29 | 0.36 |
| off-suit Q | 0.16 | 0.18 | 0.19 | 0.20 | 0.19 | 0.20 |
| off-suit J | 0.14 | 0.17 | 0.16 | 0.15 | 0.12 | 0.10 |
| off-suit 10 | 0.12 | 0.13 | 0.14 | 0.13 | 0.09 | 0.04 |
| off-suit 9 | 0.11 | 0.11 | 0.13 | 0.09 | 0.06 | 0.01 |
| off-suit 8 | 0.10 | 0.11 | 0.10 | 0.07 | 0.03 | -0.02 |
| off-suit 7 | 0.09 | 0.09 | 0.07 | 0.05 | 0.01 | -0.03 |
| off-suit 6 | 0.07 | 0.06 | 0.06 | 0.03 | -0.00 | -0.04 |
| off-suit 5 | 0.07 | 0.05 | 0.04 | -0.00 | -0.02 | -0.05 |
| off-suit 4 | 0.06 | 0.03 | 0.00 | -0.02 | -0.04 | -0.07 |
| off-suit 3 | 0.05 | 0.02 | -0.02 | -0.04 | -0.04 | -0.06 |
| off-suit 2 | 0.04 | -0.00 | -0.04 | -0.06 | -0.07 | -0.08 |
| *average miss (tricks)* | 0.22 | 0.34 | 0.45 | 0.51 | 0.52 | 0.49 |
| *chart bid made* | 85% | 77% | 68% | 62% | 62% | 62% |
| *bot's own bid made* | 85% | 78% | 71% | 67% | 67% | 68% |
| *each Jester: chance to make your bid* | +26% | +25% | +14% | +13% | +10% | +9% |
| *each Wizard: chance to make your bid* | +18% | +18% | +8% | +9% | +7% | +5% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.95 | 1.05 | 1.07 | 1.11 | 1.16 | 1.35 |
| Jester | 0.00 | -0.13 | -0.26 | -0.21 | -0.16 | -0.18 |
| off-suit A | 0.40 | 0.48 | 0.71 | 0.82 | 0.80 | 0.88 |
| off-suit K | 0.29 | 0.42 | 0.48 | 0.59 | 0.63 | 0.69 |
| off-suit Q | 0.29 | 0.38 | 0.39 | 0.41 | 0.42 | 0.51 |
| off-suit J | 0.29 | 0.33 | 0.31 | 0.33 | 0.27 | 0.33 |
| off-suit 10 | 0.21 | 0.24 | 0.29 | 0.24 | 0.21 | 0.20 |
| off-suit 9 | 0.27 | 0.18 | 0.20 | 0.18 | 0.15 | 0.12 |
| off-suit 8 | 0.18 | 0.19 | 0.22 | 0.10 | 0.10 | 0.04 |
| off-suit 7 | 0.24 | 0.14 | 0.16 | 0.09 | 0.06 | 0.01 |
| off-suit 6 | 0.16 | 0.15 | 0.11 | 0.07 | 0.05 | -0.01 |
| off-suit 5 | 0.16 | 0.09 | 0.04 | 0.02 | 0.02 | -0.03 |
| off-suit 4 | 0.13 | 0.08 | 0.02 | 0.00 | 0.01 | -0.05 |
| off-suit 3 | 0.08 | 0.06 | -0.02 | -0.03 | -0.02 | -0.06 |
| off-suit 2 | 0.08 | 0.04 | -0.07 | -0.09 | -0.05 | -0.07 |
| *average miss (tricks)* | 0.29 | 0.50 | 0.61 | 0.60 | 0.57 | 0.50 |
| *chart bid made* | 81% | 62% | 52% | 56% | 58% | 61% |
| *bot's own bid made* | 85% | 77% | 67% | 64% | 64% | 70% |
| *each Jester: chance to make your bid* | +21% | +28% | +19% | +18% | +14% | +7% |
| *each Wizard: chance to make your bid* | +16% | +12% | +0% | +8% | +7% | +6% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.03; seat 3 -0.02; dealer +0.03.

### 5 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-12 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.91 | 0.92 | 0.94 | 1.00 | 1.10 | 1.17 |
| Jester | -0.00 | -0.03 | -0.06 | -0.03 | 0.04 | 0.07 |
| trump A | 0.74 | 0.85 | 0.88 | 0.90 | 0.88 | 0.88 |
| trump K | 0.69 | 0.76 | 0.83 | 0.79 | 0.75 | 0.74 |
| trump Q | 0.66 | 0.73 | 0.76 | 0.74 | 0.67 | 0.65 |
| trump J | 0.60 | 0.66 | 0.70 | 0.67 | 0.61 | 0.58 |
| trump 10 | 0.53 | 0.60 | 0.65 | 0.62 | 0.54 | 0.53 |
| trump 9 | 0.51 | 0.59 | 0.59 | 0.57 | 0.51 | 0.48 |
| trump 8 | 0.47 | 0.50 | 0.49 | 0.49 | 0.46 | 0.44 |
| trump 7 | 0.43 | 0.37 | 0.40 | 0.43 | 0.43 | 0.41 |
| trump 6 | 0.41 | 0.28 | 0.31 | 0.38 | 0.39 | 0.38 |
| trump 5 | 0.37 | 0.28 | 0.25 | 0.33 | 0.35 | 0.35 |
| trump 4 | 0.35 | 0.19 | 0.21 | 0.30 | 0.34 | 0.34 |
| trump 3 | 0.32 | 0.17 | 0.19 | 0.27 | 0.32 | 0.34 |
| trump 2 | 0.29 | 0.15 | 0.15 | 0.26 | 0.31 | 0.30 |
| off-suit A | 0.12 | 0.15 | 0.18 | 0.28 | 0.46 | 0.57 |
| off-suit K | 0.11 | 0.13 | 0.14 | 0.16 | 0.18 | 0.20 |
| off-suit Q | 0.10 | 0.12 | 0.13 | 0.12 | 0.09 | 0.08 |
| off-suit J | 0.08 | 0.11 | 0.11 | 0.09 | 0.04 | 0.01 |
| off-suit 10 | 0.07 | 0.09 | 0.09 | 0.06 | 0.01 | -0.01 |
| off-suit 9 | 0.06 | 0.08 | 0.07 | 0.04 | -0.01 | -0.03 |
| off-suit 8 | 0.05 | 0.07 | 0.06 | 0.02 | -0.02 | -0.05 |
| off-suit 7 | 0.04 | 0.04 | 0.04 | 0.01 | -0.04 | -0.07 |
| off-suit 6 | 0.04 | 0.03 | 0.02 | -0.01 | -0.04 | -0.07 |
| off-suit 5 | 0.03 | 0.02 | 0.01 | -0.03 | -0.06 | -0.08 |
| off-suit 4 | 0.03 | 0.01 | -0.01 | -0.04 | -0.07 | -0.08 |
| off-suit 3 | 0.02 | 0.00 | -0.02 | -0.06 | -0.07 | -0.08 |
| off-suit 2 | 0.01 | -0.00 | -0.04 | -0.06 | -0.07 | -0.08 |
| *average miss (tricks)* | 0.18 | 0.28 | 0.37 | 0.45 | 0.45 | 0.43 |
| *chart bid made* | 88% | 81% | 74% | 68% | 66% | 67% |
| *bot's own bid made* | 87% | 82% | 76% | 72% | 71% | 72% |
| *each Jester: chance to make your bid* | +24% | +23% | +10% | +11% | +8% | +12% |
| *each Wizard: chance to make your bid* | +15% | +16% | +6% | +8% | +6% | +10% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-12 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.89 | 0.97 | 1.04 | 1.09 | 1.19 | 1.32 |
| Jester | 0.00 | -0.09 | -0.21 | -0.16 | -0.11 | -0.12 |
| off-suit A | 0.29 | 0.47 | 0.58 | 0.74 | 0.80 | 0.83 |
| off-suit K | 0.29 | 0.42 | 0.37 | 0.46 | 0.52 | 0.57 |
| off-suit Q | 0.23 | 0.25 | 0.29 | 0.29 | 0.31 | 0.34 |
| off-suit J | 0.19 | 0.17 | 0.22 | 0.20 | 0.16 | 0.18 |
| off-suit 10 | 0.21 | 0.20 | 0.19 | 0.15 | 0.12 | 0.09 |
| off-suit 9 | 0.17 | 0.16 | 0.18 | 0.12 | 0.07 | 0.04 |
| off-suit 8 | 0.14 | 0.18 | 0.14 | 0.08 | 0.03 | 0.00 |
| off-suit 7 | 0.14 | 0.11 | 0.12 | 0.05 | 0.02 | -0.02 |
| off-suit 6 | 0.10 | 0.05 | 0.06 | 0.03 | -0.00 | -0.04 |
| off-suit 5 | 0.11 | 0.04 | 0.04 | -0.01 | -0.03 | -0.04 |
| off-suit 4 | 0.07 | 0.03 | -0.00 | -0.03 | -0.04 | -0.05 |
| off-suit 3 | 0.08 | -0.00 | -0.06 | -0.04 | -0.05 | -0.05 |
| off-suit 2 | 0.05 | -0.03 | -0.06 | -0.05 | -0.06 | -0.07 |
| *average miss (tricks)* | 0.24 | 0.39 | 0.51 | 0.51 | 0.48 | 0.44 |
| *chart bid made* | 85% | 73% | 62% | 63% | 65% | 67% |
| *bot's own bid made* | 87% | 80% | 73% | 69% | 71% | 73% |
| *each Jester: chance to make your bid* | +18% | +27% | +17% | +15% | +12% | +7% |
| *each Wizard: chance to make your bid* | +8% | +9% | +0% | +6% | +7% | +5% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.02; seat 3 -0.01; seat 4 -0.01; dealer +0.02.

### 6 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.88 | 0.92 | 0.94 | 1.01 | 1.11 |
| Jester | -0.00 | -0.02 | -0.04 | -0.01 | 0.04 |
| trump A | 0.71 | 0.80 | 0.84 | 0.85 | 0.83 |
| trump K | 0.64 | 0.71 | 0.76 | 0.75 | 0.71 |
| trump Q | 0.56 | 0.64 | 0.69 | 0.69 | 0.62 |
| trump J | 0.54 | 0.58 | 0.63 | 0.62 | 0.55 |
| trump 10 | 0.48 | 0.53 | 0.54 | 0.54 | 0.49 |
| trump 9 | 0.43 | 0.38 | 0.41 | 0.45 | 0.44 |
| trump 8 | 0.39 | 0.31 | 0.31 | 0.37 | 0.39 |
| trump 7 | 0.37 | 0.25 | 0.25 | 0.32 | 0.34 |
| trump 6 | 0.33 | 0.21 | 0.21 | 0.27 | 0.31 |
| trump 5 | 0.29 | 0.19 | 0.18 | 0.24 | 0.29 |
| trump 4 | 0.26 | 0.16 | 0.16 | 0.22 | 0.27 |
| trump 3 | 0.25 | 0.12 | 0.14 | 0.20 | 0.26 |
| trump 2 | 0.21 | 0.10 | 0.12 | 0.19 | 0.24 |
| off-suit A | 0.08 | 0.11 | 0.12 | 0.20 | 0.34 |
| off-suit K | 0.07 | 0.10 | 0.10 | 0.10 | 0.11 |
| off-suit Q | 0.06 | 0.08 | 0.09 | 0.07 | 0.04 |
| off-suit J | 0.05 | 0.06 | 0.07 | 0.04 | 0.00 |
| off-suit 10 | 0.04 | 0.06 | 0.06 | 0.03 | -0.02 |
| off-suit 9 | 0.03 | 0.05 | 0.04 | 0.01 | -0.04 |
| off-suit 8 | 0.03 | 0.04 | 0.03 | -0.01 | -0.05 |
| off-suit 7 | 0.03 | 0.03 | 0.02 | -0.02 | -0.06 |
| off-suit 6 | 0.02 | 0.02 | 0.01 | -0.03 | -0.06 |
| off-suit 5 | 0.01 | 0.01 | -0.00 | -0.04 | -0.07 |
| off-suit 4 | 0.01 | 0.00 | -0.01 | -0.04 | -0.07 |
| off-suit 3 | 0.01 | -0.00 | -0.02 | -0.05 | -0.07 |
| off-suit 2 | 0.01 | -0.01 | -0.03 | -0.05 | -0.07 |
| *average miss (tricks)* | 0.15 | 0.24 | 0.32 | 0.39 | 0.41 |
| *chart bid made* | 90% | 84% | 78% | 73% | 70% |
| *bot's own bid made* | 89% | 84% | 79% | 76% | 75% |
| *each Jester: chance to make your bid* | +22% | +20% | +9% | +9% | +7% |
| *each Wizard: chance to make your bid* | +9% | +14% | +5% | +7% | +5% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.88 | 0.93 | 0.99 | 1.09 | 1.28 |
| Jester | 0.00 | -0.07 | -0.18 | -0.14 | -0.09 |
| off-suit A | 0.24 | 0.39 | 0.48 | 0.68 | 0.76 |
| off-suit K | 0.23 | 0.28 | 0.30 | 0.35 | 0.43 |
| off-suit Q | 0.16 | 0.22 | 0.23 | 0.21 | 0.22 |
| off-suit J | 0.18 | 0.18 | 0.19 | 0.15 | 0.10 |
| off-suit 10 | 0.15 | 0.15 | 0.17 | 0.11 | 0.05 |
| off-suit 9 | 0.13 | 0.14 | 0.14 | 0.07 | 0.01 |
| off-suit 8 | 0.10 | 0.09 | 0.09 | 0.04 | -0.02 |
| off-suit 7 | 0.08 | 0.08 | 0.05 | 0.01 | -0.03 |
| off-suit 6 | 0.05 | 0.03 | 0.05 | 0.01 | -0.04 |
| off-suit 5 | 0.06 | 0.02 | 0.01 | -0.02 | -0.05 |
| off-suit 4 | 0.07 | 0.01 | -0.01 | -0.05 | -0.04 |
| off-suit 3 | 0.04 | 0.02 | -0.04 | -0.05 | -0.05 |
| off-suit 2 | 0.03 | -0.03 | -0.04 | -0.05 | -0.06 |
| *average miss (tricks)* | 0.19 | 0.33 | 0.44 | 0.45 | 0.40 |
| *chart bid made* | 89% | 80% | 69% | 68% | 72% |
| *bot's own bid made* | 89% | 83% | 76% | 74% | 77% |
| *each Jester: chance to make your bid* | +16% | +25% | +17% | +13% | +8% |
| *each Wizard: chance to make your bid* | +4% | +4% | +2% | +5% | +3% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.01; seat 3 -0.01; seat 4 -0.01; seat 5 -0.01; dealer +0.02.
