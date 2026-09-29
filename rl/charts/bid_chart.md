# Wizard bid chart

Made from `simul1.pt` playing 150,000 rounds against itself at each table size, everyone bidding at once.

**How to use it:** find your table size, whether there's trump, and how many cards you hold. Add up the values of your cards and round to the nearest whole number. That's your bid.

- Values are tricks: 0.5 means the card wins a trick about half the time.
- *average miss*: how far the chart total is from the tricks actually taken, on average.
- *chart bid made*: how often the rounded total was exactly right (the bid would have been made).
- *bot's own bid made*: the same for the bot's real bids, which also weigh everything else it sees.
- *Seat*: add this for your seat (it's small; the dealer usually gains a little from playing last to the first trick).
- Jesters can come out slightly negative: holding one leaves fewer cards that can win.

### 3 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards | 16-20 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.95 | 0.93 | 0.91 | 0.93 | 0.94 | 1.00 | 1.17 |
| Jester | 0.00 | -0.07 | -0.09 | -0.05 | -0.01 | -0.04 | -0.11 |
| trump A | 0.89 | 0.91 | 0.95 | 0.92 | 0.91 | 0.91 | 0.98 |
| trump K | 0.84 | 0.86 | 0.90 | 0.88 | 0.84 | 0.85 | 0.90 |
| trump Q | 0.82 | 0.85 | 0.88 | 0.85 | 0.77 | 0.78 | 0.82 |
| trump J | 0.74 | 0.82 | 0.83 | 0.80 | 0.77 | 0.73 | 0.76 |
| trump 10-7 | 0.69 | 0.74 | 0.76 | 0.72 | 0.66 | 0.62 | 0.63 |
| trump 6-2 | 0.61 | 0.63 | 0.60 | 0.55 | 0.51 | 0.48 | 0.47 |
| off-suit A | 0.31 | 0.35 | 0.40 | 0.52 | 0.60 | 0.68 | 0.79 |
| off-suit K | 0.26 | 0.30 | 0.34 | 0.41 | 0.45 | 0.51 | 0.60 |
| off-suit Q | 0.26 | 0.27 | 0.28 | 0.33 | 0.34 | 0.38 | 0.44 |
| off-suit J | 0.22 | 0.24 | 0.25 | 0.26 | 0.27 | 0.29 | 0.32 |
| off-suit 10-2 | 0.16 | 0.14 | 0.13 | 0.11 | 0.11 | 0.10 | 0.05 |
| *average miss (tricks)* | 0.29 | 0.41 | 0.52 | 0.57 | 0.58 | 0.57 | 0.53 |
| *chart bid made* | 81% | 72% | 60% | 57% | 58% | 59% | 61% |
| *bot's own bid made* | 80% | 73% | 66% | 63% | 65% | 67% | 73% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards | 16-20 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.94 | 1.09 | 1.15 | 1.13 | 1.03 | 1.02 | 1.35 |
| Jester | 0.00 | -0.17 | -0.31 | -0.25 | -0.17 | -0.20 | -0.40 |
| off-suit A | 0.39 | 0.64 | 0.79 | 0.82 | 0.78 | 0.79 | 0.99 |
| off-suit K | 0.45 | 0.54 | 0.62 | 0.68 | 0.72 | 0.69 | 0.85 |
| off-suit Q | 0.40 | 0.46 | 0.50 | 0.59 | 0.56 | 0.57 | 0.66 |
| off-suit J | 0.38 | 0.53 | 0.42 | 0.46 | 0.46 | 0.49 | 0.54 |
| off-suit 10-2 | 0.25 | 0.20 | 0.19 | 0.16 | 0.17 | 0.17 | 0.11 |
| *average miss (tricks)* | 0.37 | 0.62 | 0.78 | 0.73 | 0.69 | 0.65 | 0.56 |
| *chart bid made* | 73% | 53% | 37% | 47% | 51% | 54% | 55% |
| *bot's own bid made* | 82% | 70% | 60% | 57% | 61% | 65% | 81% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.00; seat 2 -0.04; dealer +0.04.

### 4 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.93 | 0.94 | 0.93 | 0.95 | 1.00 | 1.08 |
| Jester | 0.00 | -0.04 | -0.06 | -0.04 | -0.00 | 0.01 |
| trump A | 0.81 | 0.91 | 0.93 | 0.92 | 0.92 | 0.93 |
| trump K | 0.76 | 0.82 | 0.88 | 0.85 | 0.84 | 0.81 |
| trump Q | 0.73 | 0.78 | 0.83 | 0.79 | 0.75 | 0.71 |
| trump J | 0.68 | 0.76 | 0.78 | 0.74 | 0.70 | 0.65 |
| trump 10-7 | 0.59 | 0.67 | 0.67 | 0.62 | 0.57 | 0.53 |
| trump 6-2 | 0.45 | 0.39 | 0.40 | 0.41 | 0.40 | 0.39 |
| off-suit A | 0.21 | 0.19 | 0.25 | 0.37 | 0.52 | 0.65 |
| off-suit K | 0.17 | 0.18 | 0.21 | 0.25 | 0.30 | 0.37 |
| off-suit Q | 0.16 | 0.17 | 0.18 | 0.20 | 0.19 | 0.21 |
| off-suit J | 0.14 | 0.16 | 0.15 | 0.16 | 0.14 | 0.13 |
| off-suit 10-2 | 0.08 | 0.06 | 0.05 | 0.03 | 0.01 | -0.01 |
| *average miss (tricks)* | 0.22 | 0.32 | 0.43 | 0.50 | 0.50 | 0.46 |
| *chart bid made* | 85% | 78% | 69% | 63% | 64% | 67% |
| *bot's own bid made* | 85% | 79% | 73% | 69% | 70% | 74% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-15 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.93 | 1.05 | 1.11 | 1.09 | 1.11 | 1.26 |
| Jester | 0.00 | -0.12 | -0.25 | -0.18 | -0.16 | -0.17 |
| off-suit A | 0.43 | 0.47 | 0.65 | 0.78 | 0.81 | 0.85 |
| off-suit K | 0.30 | 0.40 | 0.47 | 0.55 | 0.61 | 0.69 |
| off-suit Q | 0.29 | 0.38 | 0.37 | 0.38 | 0.41 | 0.48 |
| off-suit J | 0.30 | 0.34 | 0.31 | 0.30 | 0.29 | 0.32 |
| off-suit 10-2 | 0.17 | 0.13 | 0.11 | 0.08 | 0.06 | 0.03 |
| *average miss (tricks)* | 0.29 | 0.50 | 0.62 | 0.61 | 0.56 | 0.45 |
| *chart bid made* | 81% | 61% | 52% | 54% | 59% | 66% |
| *bot's own bid made* | 85% | 77% | 67% | 64% | 68% | 80% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.02; seat 3 -0.01; dealer +0.01.

### 5 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-12 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.90 | 0.93 | 0.94 | 0.96 | 1.05 | 1.11 |
| Jester | 0.00 | -0.03 | -0.06 | -0.03 | 0.01 | 0.04 |
| trump A | 0.74 | 0.86 | 0.91 | 0.92 | 0.91 | 0.95 |
| trump K | 0.68 | 0.78 | 0.84 | 0.82 | 0.79 | 0.78 |
| trump Q | 0.67 | 0.74 | 0.77 | 0.75 | 0.69 | 0.67 |
| trump J | 0.60 | 0.66 | 0.73 | 0.69 | 0.63 | 0.60 |
| trump 10-7 | 0.48 | 0.52 | 0.53 | 0.52 | 0.48 | 0.46 |
| trump 6-2 | 0.35 | 0.21 | 0.23 | 0.29 | 0.32 | 0.32 |
| off-suit A | 0.13 | 0.14 | 0.16 | 0.27 | 0.45 | 0.54 |
| off-suit K | 0.11 | 0.12 | 0.14 | 0.16 | 0.20 | 0.22 |
| off-suit Q | 0.10 | 0.12 | 0.13 | 0.12 | 0.10 | 0.09 |
| off-suit J | 0.08 | 0.10 | 0.11 | 0.09 | 0.06 | 0.04 |
| off-suit 10-2 | 0.04 | 0.04 | 0.02 | -0.00 | -0.03 | -0.05 |
| *average miss (tricks)* | 0.18 | 0.27 | 0.36 | 0.43 | 0.43 | 0.40 |
| *chart bid made* | 88% | 82% | 75% | 69% | 70% | 73% |
| *bot's own bid made* | 88% | 83% | 77% | 74% | 75% | 79% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards | 11-12 cards |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.90 | 1.00 | 1.05 | 1.09 | 1.16 | 1.28 |
| Jester | 0.00 | -0.09 | -0.20 | -0.17 | -0.12 | -0.12 |
| off-suit A | 0.28 | 0.44 | 0.56 | 0.72 | 0.78 | 0.85 |
| off-suit K | 0.32 | 0.40 | 0.37 | 0.47 | 0.52 | 0.57 |
| off-suit Q | 0.22 | 0.23 | 0.29 | 0.28 | 0.29 | 0.33 |
| off-suit J | 0.21 | 0.17 | 0.25 | 0.20 | 0.18 | 0.19 |
| off-suit 10-2 | 0.12 | 0.09 | 0.07 | 0.03 | 0.01 | -0.01 |
| *average miss (tricks)* | 0.24 | 0.38 | 0.51 | 0.51 | 0.47 | 0.39 |
| *chart bid made* | 85% | 74% | 63% | 63% | 66% | 72% |
| *bot's own bid made* | 87% | 82% | 74% | 71% | 73% | 81% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.01; seat 3 -0.01; seat 4 -0.01; dealer +0.01.

### 6 players

**With trump**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.88 | 0.93 | 0.95 | 0.98 | 1.05 |
| Jester | 0.00 | -0.02 | -0.05 | -0.03 | 0.01 |
| trump A | 0.71 | 0.80 | 0.87 | 0.89 | 0.90 |
| trump K | 0.63 | 0.73 | 0.79 | 0.79 | 0.77 |
| trump Q | 0.57 | 0.66 | 0.72 | 0.72 | 0.67 |
| trump J | 0.54 | 0.59 | 0.65 | 0.63 | 0.58 |
| trump 10-7 | 0.41 | 0.35 | 0.35 | 0.40 | 0.41 |
| trump 6-2 | 0.27 | 0.16 | 0.17 | 0.21 | 0.25 |
| off-suit A | 0.08 | 0.11 | 0.11 | 0.18 | 0.32 |
| off-suit K | 0.08 | 0.10 | 0.10 | 0.11 | 0.13 |
| off-suit Q | 0.05 | 0.08 | 0.09 | 0.08 | 0.05 |
| off-suit J | 0.05 | 0.07 | 0.07 | 0.06 | 0.02 |
| off-suit 10-2 | 0.02 | 0.02 | 0.01 | -0.02 | -0.05 |
| *average miss (tricks)* | 0.15 | 0.23 | 0.31 | 0.38 | 0.39 |
| *chart bid made* | 90% | 85% | 80% | 75% | 75% |
| *bot's own bid made* | 90% | 85% | 81% | 78% | 80% |

**No trump (a Jester turned up, or the last round)**: tricks each card is worth, by cards in hand

| Card | 1 card | 2 cards | 3-4 cards | 5-7 cards | 8-10 cards |
| --- | ---: | ---: | ---: | ---: | ---: |
| Wizard | 0.87 | 0.96 | 1.02 | 1.08 | 1.23 |
| Jester | 0.00 | -0.08 | -0.17 | -0.14 | -0.11 |
| off-suit A | 0.23 | 0.35 | 0.44 | 0.70 | 0.81 |
| off-suit K | 0.21 | 0.26 | 0.31 | 0.34 | 0.44 |
| off-suit Q | 0.17 | 0.20 | 0.23 | 0.21 | 0.20 |
| off-suit J | 0.18 | 0.21 | 0.20 | 0.15 | 0.11 |
| off-suit 10-2 | 0.08 | 0.06 | 0.05 | 0.01 | -0.02 |
| *average miss (tricks)* | 0.19 | 0.32 | 0.45 | 0.44 | 0.36 |
| *chart bid made* | 89% | 82% | 69% | 69% | 76% |
| *bot's own bid made* | 89% | 84% | 77% | 76% | 83% |

**Seat**: tricks taken beyond what the cards say, on average: left of dealer (leads first) +0.02; seat 2 -0.01; seat 3 -0.00; seat 4 -0.01; seat 5 -0.00; dealer +0.00.
