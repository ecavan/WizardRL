# Play situations

`simul1.pt`'s view of some first-trick decisions: expected points for the round and the chance of making your bid, for each card you could play (best first).

### Clubs led on your right, you have no clubs: trump in with the A, the J, or throw off? (hearts trump, you bid 1)

4 players, trump: h. Your hand: Ah Jh 9s 4d. Trick so far: 5c. Bids: [1, 1, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 9♠ | +14.6 | +0.0 | 62% |
| 4♦ | +14.1 | -0.5 | 62% |
| A♥ | +10.3 | -4.3 | 52% |
| J♥ | +9.8 | -4.8 | 51% |

### Same, but you bid 2

4 players, trump: h. Your hand: Ah Jh 9s 4d. Trick so far: 5c. Bids: [1, 2, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 9♠ | +31.2 | +0.0 | 80% |
| 4♦ | +30.0 | -1.2 | 79% |
| J♥ | +29.0 | -2.2 | 78% |
| A♥ | +27.8 | -3.4 | 76% |

### Same, but you bid 0

4 players, trump: h. Your hand: Ah Jh 9s 4d. Trick so far: 5c. Bids: [1, 0, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 9♠ | -10.5 | +0.0 | 9% |
| 4♦ | -14.1 | -3.6 | 7% |
| A♥ | -17.9 | -7.5 | 2% |
| J♥ | -19.2 | -8.8 | 3% |

### Clubs led, you hold the A and J of clubs (spades trump, you bid 1): take it now with the A, or play the J?

4 players, trump: s. Your hand: Ac Jc 9h 4d. Trick so far: 5c. Bids: [1, 1, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| A♣ | +19.8 | +0.0 | 73% |
| J♣ | +15.7 | -4.0 | 65% |

### Same, but you're last to play and the K of clubs is winning

4 players, trump: s. Your hand: Ac Jc 9h 4d. Trick so far: 5c Kc 8c. Bids: [1, 1, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| A♣ | +26.3 | +0.0 | 90% |
| J♣ | +18.5 | -7.9 | 70% |

### Trump led on your right (7 of hearts, hearts trump), you hold K and 3 of trump, you bid 1

4 players, trump: h. Your hand: Kh 3h 9s Qd. Trick so far: 7h. Bids: [1, 1, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 3♥ | +17.2 | +0.0 | 66% |
| K♥ | +14.9 | -2.3 | 62% |

### A Wizard is led, you bid 1 with a trump ace, a low club and a Jester (hearts trump)

4 players, trump: h. Your hand: Ah 2c jes 9d. Trick so far: wiz. Bids: [2, 1, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 2♣ | +28.4 | +0.0 | 92% |
| 9♦ | +27.3 | -1.1 | 91% |
| Jes | +23.4 | -5.0 | 84% |
| A♥ | +7.8 | -20.6 | 52% |

### You bid 0, spades led: the J of spades is winning, you hold the Q and 3 of spades, you're last

4 players, trump: h. Your hand: Qs 3s 8d 5c. Trick so far: Js 4s 2s. Bids: [1, 1, 1, 0].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 3♠ | +12.4 | +0.0 | 84% |
| Q♠ | -5.7 | -18.2 | 17% |

### You lead the first trick holding a Wizard, the A of trump and junk, you bid 2 (hearts trump)

4 players, trump: h. Your hand: wiz Ah 7c 4s 2d. Trick so far: (you lead). Bids: [2, 1, 1, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 2♦ | +33.5 | +0.0 | 87% |
| 4♠ | +32.9 | -0.6 | 87% |
| 7♣ | +31.1 | -2.4 | 86% |
| Wiz | +29.3 | -4.2 | 80% |
| A♥ | +27.8 | -5.7 | 81% |

### You lead the first trick and bid 0 (hearts trump)

4 players, trump: h. Your hand: Kh 9c 5s 3d jes. Trick so far: (you lead). Bids: [0, 1, 2, 1].

| Play | Expected points | vs best | Chance to make your bid |
| --- | ---: | ---: | ---: |
| 3♦ | +2.0 | +0.0 | 41% |
| 5♠ | +1.9 | -0.2 | 40% |
| 9♣ | -2.6 | -4.6 | 30% |
| Jes | -3.0 | -5.0 | 33% |
| K♥ | -6.3 | -8.3 | 19% |
