# Wizard bid chart

Made from `simul1.pt` playing 200,000 rounds against itself at each table size, everyone bidding at once.

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
| Wizard | 0.95 | 0.93 | 0.91 | 0.93 | 0.94 | 1.00 | 1.17 |
| Jester | -0.00 | -0.07 | -0.09 | -0.05 | -0.01 | -0.04 | -0.11 |
| trump A | 0.89 | 0.91 | 0.95 | 0.92 | 0.89 | 0.90 | 0.98 |
| trump K | 0.84 | 0.85 | 0.90 | 0.88 | 0.84 | 0.85 | 0.90 |
| trump Q | 0.84 | 0.86 | 0.88 | 0.84 | 0.77 | 0.78 | 0.82 |
| trump J | 0.76 | 0.81 | 0.84 | 0.81 | 0.76 | 0.73 | 0.76 |
| trump 10 | 0.69 | 0.77 | 0.81 | 0.76 | 0.70 | 0.68 | 0.69 |
| trump 9 | 0.71 | 0.75 | 0.77 | 0.73 | 0.68 | 0.65 | 0.65 |
| trump 8 | 0.66 | 0.73 | 0.75 | 0.70 | 0.65 | 0.61 | 0.60 |
| trump 7 | 0.68 | 0.74 | 0.71 | 0.66 | 0.62 | 0.55 | 0.56 |
| trump 6 | 0.65 | 0.67 | 0.68 | 0.62 | 0.58 | 0.54 | 0.55 |
| trump 5 | 0.61 | 0.66 | 0.64 | 0.58 | 0.53 | 0.50 | 0.49 |
| trump 4 | 0.60 | 0.64 | 0.62 | 0.55 | 0.50 | 0.48 | 0.46 |
| trump 3 | 0.56 | 0.62 | 0.58 | 0.53 | 0.48 | 0.45 | 0.43 |
| trump 2 | 0.58 | 0.60 | 0.50 | 0.48 | 0.45 | 0.43 | 0.40 |
| off-suit A | 0.29 | 0.34 | 0.40 | 0.52 | 0.60 | 0.68 | 0.79 |
| off-suit K | 0.26 | 0.30 | 0.34 | 0.41 | 0.45 | 0.51 | 0.60 |
| off-suit Q | 0.26 | 0.27 | 0.29 | 0.33 | 0.34 | 0.38 | 0.44 |
| off-suit J | 0.22 | 0.24 | 0.25 | 0.27 | 0.28 | 0.29 | 0.32 |
| off-suit 10 | 0.22 | 0.22 | 0.22 | 0.23 | 0.22 | 0.22 | 0.21 |
| off-suit 9 | 0.20 | 0.18 | 0.21 | 0.19 | 0.18 | 0.18 | 0.15 |
| off-suit 8 | 0.18 | 0.18 | 0.19 | 0.16 | 0.16 | 0.14 | 0.10 |
| off-suit 7 | 0.17 | 0.18 | 0.16 | 0.14 | 0.13 | 0.10 | 0.06 |
| off-suit 6 | 0.14 | 0.15 | 0.12 | 0.11 | 0.11 | 0.10 | 0.05 |
| off-suit 5 | 0.13 | 0.11 | 0.11 | 0.08 | 0.09 | 0.07 | 0.01 |
| off-suit 4 | 0.13 | 0.13 | 0.08 | 0.06 | 0.07 | 0.05 | -0.01 |
| off-suit 3 | 0.13 | 0.07 | 0.06 | 0.04 | 0.05 | 0.03 | -0.04 |
| off-suit 2 | 0.12 | 0.04 | 0.01 | 0.01 | 0.01 | 0.00 | -0.07 |
| *average miss (tricks)* | 0.29 | 0.41 | 0.51 | 0.56 | 0.57 | 0.56 | 0.49 |
| *chart bid made* | 81% | 72% | 61% | 59% | 60% | 61% | 65% |
| *bot's own bid made* | 81% | 73% | 66% | 63% | 64% | 67% | 73% |
| *each Jester: chance to make your bid* | +26% | +29% | +16% | +13% | +12% | +11% | +12% |
| *each Wizard: chance to make your bid* | +21% | +20% | +11% | +11% | +10% | +8% | +7% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards | 16-20 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.93 | 1.10 | 1.17 | 1.12 | 1.04 | 1.03 | 1.36 |
| Jester | 0.00 | -0.17 | -0.31 | -0.25 | -0.17 | -0.20 | -0.39 |
| off-suit A | 0.42 | 0.62 | 0.76 | 0.80 | 0.79 | 0.78 | 0.98 |
| off-suit K | 0.44 | 0.57 | 0.60 | 0.68 | 0.71 | 0.70 | 0.84 |
| off-suit Q | 0.40 | 0.44 | 0.51 | 0.60 | 0.56 | 0.57 | 0.66 |
| off-suit J | 0.39 | 0.51 | 0.41 | 0.46 | 0.46 | 0.48 | 0.54 |
| off-suit 10 | 0.35 | 0.38 | 0.37 | 0.34 | 0.37 | 0.34 | 0.38 |
| off-suit 9 | 0.33 | 0.28 | 0.29 | 0.26 | 0.27 | 0.27 | 0.28 |
| off-suit 8 | 0.27 | 0.24 | 0.30 | 0.24 | 0.24 | 0.23 | 0.19 |
| off-suit 7 | 0.27 | 0.30 | 0.25 | 0.21 | 0.20 | 0.16 | 0.12 |
| off-suit 6 | 0.23 | 0.12 | 0.17 | 0.14 | 0.13 | 0.16 | 0.11 |
| off-suit 5 | 0.25 | 0.17 | 0.12 | 0.15 | 0.14 | 0.14 | 0.05 |
| off-suit 4 | 0.19 | 0.16 | 0.14 | 0.08 | 0.07 | 0.10 | 0.01 |
| off-suit 3 | 0.21 | 0.08 | 0.04 | 0.02 | 0.07 | 0.06 | -0.05 |
| off-suit 2 | 0.16 | 0.09 | 0.03 | 0.00 | 0.01 | 0.03 | -0.11 |
| *average miss (tricks)* | 0.37 | 0.61 | 0.76 | 0.70 | 0.66 | 0.61 | 0.43 |
| *chart bid made* | 73% | 49% | 41% | 51% | 55% | 59% | 70% |
| *bot's own bid made* | 84% | 70% | 60% | 58% | 61% | 66% | 81% |
| *each Jester: chance to make your bid* | +22% | +32% | +25% | +17% | +16% | +13% | +9% |
| *each Wizard: chance to make your bid* | +14% | +18% | +8% | +12% | +13% | +8% | +7% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.00; seat 2 -0.04; dealer +0.04.

### 4 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.93 | 0.94 | 0.93 | 0.95 | 1.00 | 1.08 |
| Jester | -0.00 | -0.04 | -0.06 | -0.04 | -0.00 | 0.02 |
| trump A | 0.80 | 0.91 | 0.94 | 0.92 | 0.92 | 0.92 |
| trump K | 0.77 | 0.82 | 0.88 | 0.85 | 0.84 | 0.82 |
| trump Q | 0.73 | 0.78 | 0.83 | 0.79 | 0.75 | 0.71 |
| trump J | 0.68 | 0.75 | 0.77 | 0.74 | 0.70 | 0.65 |
| trump 10 | 0.65 | 0.70 | 0.72 | 0.68 | 0.64 | 0.58 |
| trump 9 | 0.62 | 0.68 | 0.68 | 0.66 | 0.59 | 0.55 |
| trump 8 | 0.57 | 0.66 | 0.66 | 0.60 | 0.55 | 0.52 |
| trump 7 | 0.54 | 0.60 | 0.59 | 0.55 | 0.50 | 0.47 |
| trump 6 | 0.51 | 0.58 | 0.55 | 0.50 | 0.46 | 0.44 |
| trump 5 | 0.50 | 0.47 | 0.46 | 0.45 | 0.42 | 0.41 |
| trump 4 | 0.45 | 0.37 | 0.39 | 0.40 | 0.40 | 0.39 |
| trump 3 | 0.46 | 0.29 | 0.32 | 0.36 | 0.37 | 0.37 |
| trump 2 | 0.39 | 0.25 | 0.25 | 0.33 | 0.33 | 0.34 |
| off-suit A | 0.20 | 0.19 | 0.25 | 0.38 | 0.52 | 0.65 |
| off-suit K | 0.17 | 0.18 | 0.20 | 0.26 | 0.30 | 0.37 |
| off-suit Q | 0.15 | 0.17 | 0.18 | 0.20 | 0.19 | 0.21 |
| off-suit J | 0.14 | 0.16 | 0.16 | 0.15 | 0.14 | 0.13 |
| off-suit 10 | 0.12 | 0.13 | 0.13 | 0.13 | 0.10 | 0.07 |
| off-suit 9 | 0.11 | 0.10 | 0.13 | 0.10 | 0.07 | 0.03 |
| off-suit 8 | 0.10 | 0.10 | 0.10 | 0.08 | 0.05 | 0.02 |
| off-suit 7 | 0.09 | 0.08 | 0.07 | 0.05 | 0.03 | -0.01 |
| off-suit 6 | 0.07 | 0.06 | 0.05 | 0.03 | 0.01 | -0.02 |
| off-suit 5 | 0.07 | 0.05 | 0.04 | 0.01 | -0.01 | -0.03 |
| off-suit 4 | 0.06 | 0.03 | 0.01 | -0.02 | -0.02 | -0.04 |
| off-suit 3 | 0.05 | 0.02 | -0.02 | -0.03 | -0.03 | -0.05 |
| off-suit 2 | 0.03 | 0.00 | -0.03 | -0.05 | -0.06 | -0.07 |
| *average miss (tricks)* | 0.22 | 0.32 | 0.43 | 0.49 | 0.50 | 0.45 |
| *chart bid made* | 85% | 79% | 70% | 65% | 65% | 68% |
| *bot's own bid made* | 85% | 79% | 73% | 69% | 70% | 74% |
| *each Jester: chance to make your bid* | +26% | +24% | +13% | +12% | +11% | +10% |
| *each Wizard: chance to make your bid* | +18% | +19% | +10% | +11% | +10% | +8% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.95 | 1.04 | 1.12 | 1.09 | 1.11 | 1.26 |
| Jester | 0.00 | -0.13 | -0.26 | -0.18 | -0.17 | -0.17 |
| off-suit A | 0.40 | 0.46 | 0.65 | 0.79 | 0.80 | 0.85 |
| off-suit K | 0.29 | 0.43 | 0.48 | 0.55 | 0.62 | 0.69 |
| off-suit Q | 0.29 | 0.37 | 0.38 | 0.39 | 0.41 | 0.48 |
| off-suit J | 0.29 | 0.34 | 0.32 | 0.32 | 0.29 | 0.32 |
| off-suit 10 | 0.21 | 0.22 | 0.28 | 0.23 | 0.19 | 0.19 |
| off-suit 9 | 0.27 | 0.17 | 0.20 | 0.20 | 0.18 | 0.13 |
| off-suit 8 | 0.18 | 0.19 | 0.20 | 0.11 | 0.10 | 0.07 |
| off-suit 7 | 0.24 | 0.15 | 0.15 | 0.12 | 0.07 | 0.04 |
| off-suit 6 | 0.16 | 0.14 | 0.12 | 0.07 | 0.07 | 0.03 |
| off-suit 5 | 0.16 | 0.11 | 0.05 | 0.05 | 0.02 | -0.00 |
| off-suit 4 | 0.13 | 0.08 | 0.02 | 0.01 | 0.01 | -0.03 |
| off-suit 3 | 0.08 | 0.09 | -0.02 | -0.04 | -0.01 | -0.05 |
| off-suit 2 | 0.08 | 0.05 | -0.06 | -0.05 | -0.04 | -0.07 |
| *average miss (tricks)* | 0.29 | 0.50 | 0.60 | 0.59 | 0.55 | 0.41 |
| *chart bid made* | 81% | 62% | 53% | 57% | 62% | 71% |
| *bot's own bid made* | 85% | 77% | 68% | 65% | 67% | 80% |
| *each Jester: chance to make your bid* | +20% | +29% | +18% | +16% | +14% | +8% |
| *each Wizard: chance to make your bid* | +15% | +13% | +4% | +12% | +11% | +8% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.02; seat 3 -0.01; dealer +0.01.

### 5 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-12 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.91 | 0.92 | 0.94 | 0.96 | 1.05 | 1.11 |
| Jester | -0.00 | -0.03 | -0.06 | -0.04 | 0.01 | 0.04 |
| trump A | 0.74 | 0.87 | 0.90 | 0.92 | 0.91 | 0.94 |
| trump K | 0.69 | 0.78 | 0.84 | 0.82 | 0.79 | 0.78 |
| trump Q | 0.67 | 0.74 | 0.78 | 0.75 | 0.69 | 0.67 |
| trump J | 0.60 | 0.67 | 0.73 | 0.69 | 0.62 | 0.60 |
| trump 10 | 0.53 | 0.61 | 0.66 | 0.62 | 0.55 | 0.52 |
| trump 9 | 0.51 | 0.60 | 0.61 | 0.57 | 0.51 | 0.49 |
| trump 8 | 0.47 | 0.52 | 0.50 | 0.50 | 0.46 | 0.44 |
| trump 7 | 0.43 | 0.35 | 0.36 | 0.41 | 0.41 | 0.40 |
| trump 6 | 0.41 | 0.28 | 0.29 | 0.36 | 0.37 | 0.38 |
| trump 5 | 0.37 | 0.27 | 0.25 | 0.32 | 0.34 | 0.32 |
| trump 4 | 0.35 | 0.19 | 0.23 | 0.28 | 0.32 | 0.32 |
| trump 3 | 0.32 | 0.18 | 0.19 | 0.26 | 0.29 | 0.30 |
| trump 2 | 0.29 | 0.15 | 0.17 | 0.24 | 0.28 | 0.28 |
| off-suit A | 0.13 | 0.14 | 0.17 | 0.28 | 0.45 | 0.54 |
| off-suit K | 0.11 | 0.12 | 0.14 | 0.16 | 0.20 | 0.22 |
| off-suit Q | 0.10 | 0.11 | 0.12 | 0.12 | 0.10 | 0.09 |
| off-suit J | 0.08 | 0.10 | 0.11 | 0.09 | 0.06 | 0.04 |
| off-suit 10 | 0.07 | 0.09 | 0.09 | 0.07 | 0.03 | 0.00 |
| off-suit 9 | 0.06 | 0.07 | 0.08 | 0.05 | 0.01 | -0.03 |
| off-suit 8 | 0.05 | 0.07 | 0.06 | 0.03 | -0.00 | -0.04 |
| off-suit 7 | 0.04 | 0.04 | 0.04 | 0.01 | -0.02 | -0.05 |
| off-suit 6 | 0.04 | 0.03 | 0.02 | -0.01 | -0.04 | -0.05 |
| off-suit 5 | 0.03 | 0.02 | 0.01 | -0.03 | -0.05 | -0.07 |
| off-suit 4 | 0.03 | 0.01 | -0.01 | -0.04 | -0.06 | -0.07 |
| off-suit 3 | 0.02 | 0.00 | -0.02 | -0.05 | -0.06 | -0.08 |
| off-suit 2 | 0.01 | -0.00 | -0.04 | -0.06 | -0.07 | -0.08 |
| *average miss (tricks)* | 0.18 | 0.27 | 0.36 | 0.43 | 0.43 | 0.39 |
| *chart bid made* | 88% | 83% | 76% | 71% | 70% | 73% |
| *bot's own bid made* | 88% | 83% | 77% | 74% | 75% | 79% |
| *each Jester: chance to make your bid* | +24% | +22% | +10% | +11% | +10% | +12% |
| *each Wizard: chance to make your bid* | +14% | +17% | +8% | +10% | +9% | +12% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-12 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.89 | 0.99 | 1.05 | 1.09 | 1.16 | 1.28 |
| Jester | 0.00 | -0.10 | -0.21 | -0.16 | -0.12 | -0.12 |
| off-suit A | 0.29 | 0.44 | 0.55 | 0.72 | 0.78 | 0.85 |
| off-suit K | 0.29 | 0.39 | 0.38 | 0.46 | 0.52 | 0.58 |
| off-suit Q | 0.23 | 0.26 | 0.29 | 0.28 | 0.29 | 0.33 |
| off-suit J | 0.19 | 0.17 | 0.24 | 0.20 | 0.18 | 0.19 |
| off-suit 10 | 0.21 | 0.20 | 0.18 | 0.15 | 0.11 | 0.09 |
| off-suit 9 | 0.17 | 0.17 | 0.18 | 0.12 | 0.08 | 0.05 |
| off-suit 8 | 0.14 | 0.16 | 0.13 | 0.08 | 0.04 | 0.01 |
| off-suit 7 | 0.14 | 0.10 | 0.11 | 0.06 | 0.03 | -0.01 |
| off-suit 6 | 0.10 | 0.07 | 0.06 | 0.03 | 0.01 | -0.02 |
| off-suit 5 | 0.11 | 0.06 | 0.05 | 0.00 | -0.01 | -0.03 |
| off-suit 4 | 0.07 | 0.04 | 0.00 | -0.02 | -0.03 | -0.05 |
| off-suit 3 | 0.08 | 0.01 | -0.05 | -0.05 | -0.03 | -0.06 |
| off-suit 2 | 0.05 | -0.03 | -0.06 | -0.07 | -0.07 | -0.07 |
| *average miss (tricks)* | 0.24 | 0.38 | 0.50 | 0.50 | 0.46 | 0.38 |
| *chart bid made* | 85% | 75% | 63% | 64% | 68% | 74% |
| *bot's own bid made* | 87% | 82% | 74% | 71% | 73% | 81% |
| *each Jester: chance to make your bid* | +18% | +25% | +16% | +14% | +12% | +8% |
| *each Wizard: chance to make your bid* | +8% | +13% | +4% | +9% | +9% | +8% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.01; seat 3 -0.01; seat 4 -0.01; dealer +0.01.

### 6 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.88 | 0.93 | 0.95 | 0.98 | 1.05 |
| Jester | -0.00 | -0.02 | -0.05 | -0.03 | 0.01 |
| trump A | 0.71 | 0.81 | 0.87 | 0.89 | 0.89 |
| trump K | 0.63 | 0.72 | 0.79 | 0.79 | 0.76 |
| trump Q | 0.57 | 0.66 | 0.71 | 0.72 | 0.66 |
| trump J | 0.54 | 0.59 | 0.65 | 0.63 | 0.58 |
| trump 10 | 0.48 | 0.52 | 0.50 | 0.53 | 0.50 |
| trump 9 | 0.43 | 0.35 | 0.36 | 0.43 | 0.43 |
| trump 8 | 0.39 | 0.29 | 0.29 | 0.35 | 0.38 |
| trump 7 | 0.38 | 0.25 | 0.24 | 0.29 | 0.33 |
| trump 6 | 0.33 | 0.21 | 0.22 | 0.25 | 0.29 |
| trump 5 | 0.29 | 0.18 | 0.18 | 0.23 | 0.26 |
| trump 4 | 0.26 | 0.15 | 0.16 | 0.20 | 0.25 |
| trump 3 | 0.25 | 0.13 | 0.14 | 0.20 | 0.22 |
| trump 2 | 0.21 | 0.11 | 0.12 | 0.18 | 0.22 |
| off-suit A | 0.08 | 0.11 | 0.11 | 0.18 | 0.32 |
| off-suit K | 0.07 | 0.10 | 0.10 | 0.11 | 0.12 |
| off-suit Q | 0.06 | 0.08 | 0.09 | 0.08 | 0.05 |
| off-suit J | 0.05 | 0.06 | 0.07 | 0.06 | 0.02 |
| off-suit 10 | 0.04 | 0.06 | 0.06 | 0.04 | 0.00 |
| off-suit 9 | 0.03 | 0.05 | 0.05 | 0.02 | -0.02 |
| off-suit 8 | 0.03 | 0.04 | 0.03 | 0.01 | -0.03 |
| off-suit 7 | 0.03 | 0.03 | 0.02 | -0.01 | -0.04 |
| off-suit 6 | 0.02 | 0.02 | 0.01 | -0.02 | -0.06 |
| off-suit 5 | 0.02 | 0.01 | -0.00 | -0.03 | -0.06 |
| off-suit 4 | 0.01 | 0.00 | -0.02 | -0.04 | -0.07 |
| off-suit 3 | 0.01 | -0.00 | -0.02 | -0.05 | -0.07 |
| off-suit 2 | 0.01 | -0.01 | -0.03 | -0.05 | -0.07 |
| *average miss (tricks)* | 0.15 | 0.23 | 0.31 | 0.37 | 0.38 |
| *chart bid made* | 90% | 85% | 80% | 75% | 75% |
| *bot's own bid made* | 90% | 85% | 81% | 78% | 80% |
| *each Jester: chance to make your bid* | +21% | +20% | +9% | +10% | +9% |
| *each Wizard: chance to make your bid* | +9% | +15% | +6% | +9% | +8% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.88 | 0.95 | 1.03 | 1.09 | 1.23 |
| Jester | 0.00 | -0.08 | -0.17 | -0.14 | -0.11 |
| off-suit A | 0.24 | 0.37 | 0.44 | 0.70 | 0.81 |
| off-suit K | 0.23 | 0.25 | 0.30 | 0.34 | 0.44 |
| off-suit Q | 0.16 | 0.22 | 0.23 | 0.20 | 0.20 |
| off-suit J | 0.18 | 0.19 | 0.20 | 0.15 | 0.11 |
| off-suit 10 | 0.15 | 0.16 | 0.16 | 0.09 | 0.04 |
| off-suit 9 | 0.13 | 0.14 | 0.14 | 0.08 | 0.02 |
| off-suit 8 | 0.10 | 0.10 | 0.09 | 0.05 | -0.01 |
| off-suit 7 | 0.08 | 0.08 | 0.06 | 0.02 | -0.02 |
| off-suit 6 | 0.05 | 0.03 | 0.04 | 0.01 | -0.03 |
| off-suit 5 | 0.06 | 0.03 | 0.01 | -0.01 | -0.04 |
| off-suit 4 | 0.07 | 0.03 | -0.02 | -0.05 | -0.05 |
| off-suit 3 | 0.04 | 0.02 | -0.03 | -0.05 | -0.06 |
| off-suit 2 | 0.03 | -0.03 | -0.04 | -0.05 | -0.06 |
| *average miss (tricks)* | 0.19 | 0.32 | 0.44 | 0.43 | 0.36 |
| *chart bid made* | 89% | 81% | 70% | 70% | 77% |
| *bot's own bid made* | 89% | 84% | 77% | 76% | 83% |
| *each Jester: chance to make your bid* | +16% | +24% | +15% | +12% | +9% |
| *each Wizard: chance to make your bid* | +4% | +9% | +4% | +8% | +8% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.01; seat 3 -0.00; seat 4 -0.01; seat 5 -0.00; dealer +0.00.
