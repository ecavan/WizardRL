# Wizard mistake chart

What departing from `ppo5.pt`'s choice costs, for a human-like player (picks with probabilities softmax(expected points / 3)), in full games with everyone bidding at once. **Points**: expected points lost that round, as `simul1.pt` (a round-points network) sees it. **Win chance**: chance of winning the game lost (a rough reading, see `mistakes.py`). **Per game**: what it costs the human-like player over a whole game (how likely they are to pick it × what it costs, summed over the game); the habit lists are sorted by it. Every option at every decision is costed, so rare blunders show up too.

## 3 players

About 1,034 games. A player like this gives up about **7 points a game in bidding** and **82 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 8-12 cards | bid 1 fewer than the bot | −8.8 | −1.91% | −1.2 | 14,770 |
| 4-7 cards | bid 1 more than the bot | −9.7 | −2.02% | −0.8 | 11,370 |
| 4-7 cards | bid 1 fewer than the bot | −9.7 | −2.02% | −0.8 | 7,388 |
| 13+ cards | bid 1 fewer than the bot | −8.2 | −1.62% | −0.8 | 24,561 |
| 13+ cards | bid 1 more than the bot | −11.0 | −2.33% | −0.7 | 24,576 |
| 8-12 cards | bid 1 more than the bot | −9.1 | −1.98% | −0.7 | 15,341 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −14.2 | −1.98% | −0.3 | 4,789 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −17.7 | −2.76% | −0.3 | 4,197 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −14.1 | −2.93% | −0.3 | 3,986 |
| 1-3 cards | bid 1 more than the bot | −14.3 | −2.57% | −0.2 | 3,475 |
| 13+ cards | bid 2+ fewer than the bot | −68.4 | −13.01% | −0.1 | 87,637 |
| 1-3 cards | bid 1 fewer than the bot | −11.8 | −2.14% | −0.1 | 806 |
| 8-12 cards | bid 2+ more than the bot | −70.8 | −14.03% | −0.0 | 86,755 |
| 8-12 cards | bid 2+ fewer than the bot | −41.8 | −8.75% | −0.0 | 21,231 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −12.2 | −2.59% | −0.0 | 571 |
| 4-7 cards | bid 2+ more than the bot | −44.5 | −9.04% | −0.0 | 30,023 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −3.9 | −0.79% | −0.0 | 914 |
| 13+ cards | bid 2+ more than the bot | −110.3 | −19.65% | −0.0 | 244,154 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −29.5 | −5.64% | −0.0 | 3,321 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −39.8 | −8.19% | −0.0 | 7,388 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −38.9 | −7.76% | −0.0 | 3,340 |
| 4-7 cards | bid 2+ fewer than the bot | −33.4 | −7.03% | −0.0 | 3,175 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −55.4 | −11.38% | −0.0 | 14,770 |
| 1-3 cards | bid 2+ fewer than the bot (you bid 0) | −36.6 | −6.94% | −0.0 | 806 |
| 1-3 cards | bid 2+ more than the bot | −30.0 | −5.74% | −0.0 | 1,599 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −3.3 | −0.74% | −8.1 | 130,565 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −2.0 | −0.40% | −3.0 | 94,222 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −8.0 | −1.60% | −1.9 | 21,596 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −1.6 | −0.34% | −1.8 | 84,804 |
| need more tricks, you lead | bot: lead off-suit low / you: Jester | −8.0 | −1.64% | −1.8 | 62,383 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −8.7 | −1.85% | −1.8 | 21,856 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead trump low | −4.9 | −1.06% | −1.6 | 25,973 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −6.9 | −1.39% | −1.6 | 13,764 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −7.3 | −1.45% | −1.6 | 16,030 |
| need more tricks, you lead | bot: lead off-suit low / you: Wizard | −7.5 | −1.51% | −1.5 | 65,723 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −9.0 | −1.83% | −1.5 | 14,455 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −8.0 | −1.60% | −1.4 | 12,276 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck high | −5.5 | −1.21% | −1.4 | 14,974 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −2.5 | −0.49% | −1.4 | 41,070 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −3.9 | −0.80% | −1.2 | 17,341 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.6 | −0.55% | −1.2 | 17,481 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −12.6 | −2.49% | −1.1 | 18,454 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −2.8 | −0.55% | −1.0 | 13,763 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Jester | −12.5 | −2.51% | −1.0 | 18,172 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −8.2 | −1.67% | −1.0 | 16,393 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: Wizard | −6.4 | −1.32% | −1.0 | 13,547 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: Jester | −6.8 | −1.37% | −0.9 | 12,682 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck low | −6.3 | −1.28% | −0.9 | 11,668 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.2 | −0.40% | −0.8 | 13,546 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: Wizard | −6.5 | −1.22% | −0.8 | 10,426 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 2,564 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −28.8 | −5.94% | −0.0 | 2,735 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −27.0 | −5.33% | −0.0 | 2,576 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −17.2 | −3.60% | −0.1 | 4,776 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −15.5 | −3.28% | −0.2 | 3,922 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −15.4 | −3.22% | −0.1 | 5,136 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win big / you: Wizard | −13.2 | −2.88% | −0.2 | 6,021 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −12.6 | −2.49% | −1.1 | 18,454 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win big / you: Jester | −12.6 | −2.63% | −0.2 | 6,146 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Jester | −12.5 | −2.51% | −1.0 | 18,172 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −12.0 | −2.47% | −0.3 | 7,637 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −11.7 | −2.41% | −0.3 | 4,341 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −11.1 | −2.29% | −0.2 | 4,715 |
| need more tricks, last to play, off-suit is winning, can follow | bot: duck high / you: Jester | −11.1 | −2.32% | −0.2 | 4,930 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −10.8 | −2.28% | −0.6 | 19,128 |
| need more tricks, last to play, trump is winning, can follow | bot: win cheaply / you: Jester | −10.1 | −2.04% | −0.3 | 4,590 |
| made your bid, you lead | bot: Jester / you: lead off-suit low | −10.0 | −1.96% | −0.7 | 8,620 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Wizard | −9.9 | −2.14% | −0.6 | 19,381 |
| need more tricks, last to play, trump is winning, can follow | bot: win cheaply / you: Wizard | −9.7 | −2.02% | −0.4 | 4,374 |
| need more tricks, you lead | bot: lead trump low / you: Jester | −9.4 | −2.00% | −0.6 | 17,513 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: duck high / you: Jester | −9.3 | −1.84% | −0.3 | 6,586 |
| need more tricks, last to play, trump is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −9.2 | −1.90% | −0.3 | 3,462 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −9.0 | −1.83% | −1.5 | 14,455 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Jester | −9.0 | −1.77% | −0.6 | 12,291 |
| need more tricks, last to play, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck low / you: Wizard | −9.0 | −1.72% | −0.0 | 3,650 |
| need more tricks, mid-trick, only Jesters so far, no suit to follow | bot: win big / you: Wizard | −9.0 | −1.71% | −0.1 | 2,587 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 11 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

## 4 players

About 1,034 games. A player like this gives up about **5 points a game in bidding** and **50 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 8-12 cards | bid 1 more than the bot | −11.3 | −3.33% | −1.1 | 20,165 |
| 8-12 cards | bid 1 fewer than the bot | −8.8 | −2.60% | −1.0 | 17,082 |
| 4-7 cards | bid 1 more than the bot | −12.6 | −3.68% | −0.8 | 13,672 |
| 4-7 cards | bid 1 fewer than the bot | −8.9 | −2.71% | −0.5 | 6,751 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −13.6 | −3.94% | −0.4 | 6,922 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −17.4 | −4.38% | −0.3 | 7,872 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −6.7 | −1.95% | −0.2 | 2,711 |
| 13+ cards | bid 1 fewer than the bot | −9.0 | −2.55% | −0.2 | 11,738 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −17.5 | −4.51% | −0.2 | 4,387 |
| 13+ cards | bid 1 more than the bot | −11.0 | −3.84% | −0.2 | 12,244 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −13.0 | −3.71% | −0.2 | 3,083 |
| 1-3 cards | bid 1 more than the bot | −18.5 | −4.93% | −0.1 | 3,768 |
| 1-3 cards | bid 1 fewer than the bot | −11.0 | −3.11% | −0.0 | 622 |
| 8-12 cards | bid 2+ fewer than the bot | −40.3 | −11.46% | −0.0 | 15,207 |
| 13+ cards | bid 2+ fewer than the bot | −50.1 | −15.23% | −0.0 | 19,975 |
| 8-12 cards | bid 2+ more than the bot | −74.9 | −18.19% | −0.0 | 129,313 |
| 13+ cards | bid 1 fewer than the bot (you bid 0) | −14.1 | −4.81% | −0.0 | 506 |
| 4-7 cards | bid 2+ more than the bot | −47.9 | −12.40% | −0.0 | 40,943 |
| 13+ cards | bid 2+ more than the bot | −99.3 | −24.48% | −0.0 | 115,209 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −32.7 | −8.43% | −0.0 | 6,039 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −43.1 | −10.97% | −0.0 | 10,569 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −48.7 | −13.18% | −0.0 | 17,082 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −37.0 | −10.35% | −0.0 | 6,751 |
| 4-7 cards | bid 2+ fewer than the bot | −33.0 | −9.58% | −0.0 | 1,793 |
| 1-3 cards | bid 2+ more than the bot | −33.3 | −8.66% | −0.0 | 1,841 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −3.5 | −1.06% | −3.7 | 66,852 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −6.2 | −1.93% | −1.4 | 14,368 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −7.6 | −2.32% | −1.2 | 15,478 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −5.5 | −1.70% | −1.2 | 10,487 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −7.3 | −2.19% | −1.1 | 14,132 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −4.3 | −1.34% | −1.1 | 17,121 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −1.7 | −0.52% | −1.0 | 31,254 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −3.4 | −0.95% | −0.9 | 11,965 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −1.4 | −0.43% | −0.7 | 31,666 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead trump low | −4.4 | −1.41% | −0.7 | 12,011 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: win cheaply | −1.8 | −0.56% | −0.7 | 12,488 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −6.4 | −2.02% | −0.7 | 7,798 |
| need more tricks, you lead | bot: lead off-suit low / you: Jester | −5.9 | −1.97% | −0.7 | 20,011 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −1.9 | −0.53% | −0.6 | 13,057 |
| need more tricks, you lead | bot: lead off-suit low / you: Wizard | −6.1 | −1.82% | −0.6 | 25,022 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −6.2 | −1.91% | −0.6 | 5,924 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit low | −2.3 | −0.70% | −0.6 | 11,988 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −2.7 | −0.83% | −0.6 | 15,545 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −6.1 | −1.83% | −0.5 | 9,190 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: Jester | −4.6 | −1.35% | −0.5 | 6,828 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: Wizard | −6.3 | −2.01% | −0.5 | 8,072 |
| need more tricks, you lead | bot: Wizard / you: lead off-suit low | −4.4 | −1.48% | −0.5 | 5,398 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck high | −7.2 | −2.20% | −0.4 | 5,571 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck low | −4.3 | −1.22% | −0.4 | 6,162 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.2 | −0.71% | −0.4 | 8,226 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 1,367 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −29.7 | −8.72% | −0.0 | 1,480 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −27.8 | −7.84% | −0.0 | 2,901 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −18.8 | −5.55% | −0.1 | 3,069 |
| need more tricks, mid-trick, a Wizard is winning, can follow | bot: duck low / you: Wizard | −18.7 | −5.40% | −0.0 | 1,460 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −18.3 | −5.87% | −0.0 | 1,959 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −16.9 | −5.29% | −0.0 | 1,989 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: Jester / you: win cheaply | −13.7 | −4.64% | −0.1 | 1,530 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Jester | −12.9 | −4.18% | −0.2 | 5,110 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −12.7 | −3.87% | −0.1 | 3,638 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −12.5 | −3.82% | −0.3 | 5,644 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win cheaply | −11.9 | −3.89% | −0.1 | 1,766 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −11.3 | −3.56% | −0.1 | 4,492 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −10.3 | −3.20% | −0.3 | 6,129 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −10.3 | −3.16% | −0.0 | 2,611 |
| need more tricks, last to play, trump is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −10.1 | −2.98% | −0.1 | 1,594 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −9.7 | −3.01% | −0.3 | 7,205 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −9.3 | −3.15% | −0.3 | 8,254 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Wizard | −8.9 | −2.82% | −0.3 | 8,841 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Wizard | −8.6 | −2.74% | −0.3 | 6,958 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win big (trump in) / you: Wizard | −8.5 | −2.72% | −0.1 | 2,246 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck low | −8.3 | −2.59% | −0.2 | 3,647 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −8.3 | −2.56% | −0.2 | 4,770 |
| need more tricks, mid-trick, trump is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −8.2 | −2.49% | −0.0 | 1,909 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: duck high / you: Wizard | −8.0 | −2.56% | −0.1 | 1,551 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Jester | −8.0 | −2.44% | −0.3 | 6,243 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 8 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

## 5 players

About 1,033 games. A player like this gives up about **4 points a game in bidding** and **28 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 8-12 cards | bid 1 more than the bot | −12.6 | −5.33% | −0.8 | 24,056 |
| 8-12 cards | bid 1 fewer than the bot | −9.1 | −3.63% | −0.7 | 17,273 |
| 4-7 cards | bid 1 more than the bot | −14.1 | −5.08% | −0.7 | 15,401 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −13.1 | −4.73% | −0.4 | 9,379 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −9.2 | −3.29% | −0.4 | 5,079 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −12.2 | −4.62% | −0.3 | 6,783 |
| 4-7 cards | bid 1 fewer than the bot | −9.3 | −3.46% | −0.3 | 6,022 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −19.1 | −6.09% | −0.3 | 10,868 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −17.5 | −6.10% | −0.2 | 4,566 |
| 8-12 cards | bid 1 more than the bot (bot bids 0) | −4.9 | −1.82% | −0.1 | 1,544 |
| 1-3 cards | bid 1 more than the bot | −20.6 | −7.17% | −0.1 | 3,900 |
| 1-3 cards | bid 1 fewer than the bot | −10.3 | −4.11% | −0.0 | 478 |
| 8-12 cards | bid 2+ fewer than the bot | −40.4 | −16.65% | −0.0 | 11,632 |
| 8-12 cards | bid 2+ more than the bot | −77.7 | −22.77% | −0.0 | 164,212 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −45.9 | −17.04% | −0.0 | 17,273 |
| 4-7 cards | bid 2+ more than the bot | −50.3 | −15.36% | −0.0 | 48,976 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −46.2 | −13.57% | −0.0 | 20,626 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −34.8 | −10.94% | −0.0 | 8,972 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −36.4 | −12.41% | −0.0 | 6,022 |
| 8-12 cards | bid 2+ more than the bot (bot bids 0) | −70.0 | −19.30% | −0.0 | 13,227 |
| 1-3 cards | bid 2+ fewer than the bot (you bid 0) | −35.8 | −12.36% | −0.0 | 478 |
| 4-7 cards | bid 2+ fewer than the bot | −33.8 | −12.05% | −0.0 | 1,135 |
| 1-3 cards | bid 2+ more than the bot | −36.3 | −12.16% | −0.0 | 2,002 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −3.7 | −1.48% | −1.3 | 29,555 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −5.3 | −2.25% | −0.9 | 11,567 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −4.3 | −1.78% | −0.7 | 7,283 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −6.9 | −2.83% | −0.7 | 9,867 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −4.6 | −1.82% | −0.6 | 11,080 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −7.0 | −2.86% | −0.6 | 7,884 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −4.1 | −1.68% | −0.5 | 7,884 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit low | −2.0 | −0.80% | −0.5 | 12,541 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: win cheaply | −1.9 | −0.76% | −0.4 | 7,494 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −1.6 | −0.67% | −0.4 | 12,205 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −5.9 | −2.32% | −0.3 | 6,920 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −1.4 | −0.57% | −0.3 | 13,541 |
| made your bid, mid-trick, a Wizard is winning, no suit to follow | bot: duck high / you: duck low | −2.3 | −0.99% | −0.3 | 6,260 |
| made your bid, mid-trick, trump is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.0 | −0.40% | −0.3 | 9,878 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −1.7 | −0.81% | −0.3 | 8,168 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −5.6 | −2.33% | −0.3 | 3,903 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck low / you: duck high | −5.8 | −2.41% | −0.3 | 7,852 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.6 | −3.39% | −0.3 | 6,643 |
| need more tricks, you lead | bot: Wizard / you: lead off-suit low | −3.7 | −1.83% | −0.3 | 4,026 |
| made your bid, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −0.9 | −0.29% | −0.3 | 10,017 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −3.1 | −1.25% | −0.3 | 6,948 |
| need more tricks, you lead | bot: lead off-suit low / you: Jester | −4.6 | −1.95% | −0.3 | 8,004 |
| need more tricks, you lead | bot: lead off-suit low / you: Wizard | −5.0 | −2.09% | −0.2 | 12,172 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead trump low | −3.8 | −1.61% | −0.2 | 4,592 |
| made your bid, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −3.5 | −1.41% | −0.2 | 2,888 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 852 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −29.4 | −11.91% | −0.0 | 866 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −28.9 | −11.39% | −0.0 | 2,554 |
| need more tricks, last to play, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −26.8 | −10.56% | −0.0 | 947 |
| need more tricks, mid-trick, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −26.4 | −9.50% | −0.0 | 951 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −21.2 | −8.36% | −0.0 | 2,417 |
| need more tricks, mid-trick, a Wizard is winning, can follow | bot: duck low / you: Wizard | −20.3 | −8.36% | −0.0 | 2,287 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Jester | −14.2 | −6.12% | −0.1 | 1,516 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −12.8 | −4.96% | −0.0 | 1,588 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −12.5 | −5.08% | −0.1 | 1,909 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: Jester / you: win cheaply | −11.9 | −4.32% | −0.1 | 1,844 |
| need more tricks, last to play, trump is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −11.3 | −4.61% | −0.0 | 1,004 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −11.2 | −4.59% | −0.1 | 2,970 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −11.0 | −4.29% | −0.1 | 2,620 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win cheaply | −10.7 | −3.73% | −0.2 | 2,771 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck low | −9.8 | −3.93% | −0.1 | 1,156 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.6 | −3.39% | −0.3 | 6,643 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −9.1 | −3.58% | −0.1 | 3,168 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −9.1 | −4.02% | −0.1 | 2,580 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck high | −8.7 | −3.16% | −0.1 | 2,160 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win big (trump in) / you: duck high | −8.5 | −3.50% | −0.0 | 891 |
| made your bid, mid-trick, off-suit is winning, can't follow | bot: duck high / you: win cheaply (trump in) | −8.3 | −2.98% | −0.1 | 1,830 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −8.3 | −3.35% | −0.1 | 3,150 |
| need more tricks, mid-trick, trump is winning, can follow, nothing but a Wizard wins | bot: Jester / you: duck high | −8.2 | −2.93% | −0.0 | 902 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Wizard | −8.0 | −3.57% | −0.1 | 3,236 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit high (K+) | −7.9 | −3.05% | −0.1 | 1,630 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 5 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

## 6 players

About 1,031 games. A player like this gives up about **3 points a game in bidding** and **16 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 4-7 cards | bid 1 more than the bot | −16.3 | −8.32% | −0.5 | 16,705 |
| 8-12 cards | bid 1 more than the bot | −14.6 | −8.10% | −0.4 | 15,922 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −12.0 | −5.97% | −0.4 | 7,871 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −12.5 | −6.54% | −0.3 | 11,147 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −11.4 | −5.50% | −0.3 | 6,675 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −21.0 | −11.45% | −0.2 | 13,580 |
| 8-12 cards | bid 1 more than the bot (bot bids 0) | −8.0 | −3.21% | −0.2 | 2,510 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −17.3 | −9.72% | −0.1 | 4,888 |
| 4-7 cards | bid 1 fewer than the bot | −8.8 | −4.99% | −0.1 | 5,558 |
| 8-12 cards | bid 1 fewer than the bot | −8.3 | −4.47% | −0.1 | 9,247 |
| 1-3 cards | bid 1 more than the bot | −23.7 | −12.84% | −0.0 | 4,186 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −40.5 | −19.58% | −0.0 | 9,247 |
| 1-3 cards | bid 1 fewer than the bot | −9.0 | −5.26% | −0.0 | 403 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −35.7 | −17.14% | −0.0 | 5,558 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −49.0 | −18.13% | −0.0 | 32,641 |
| 8-12 cards | bid 2+ fewer than the bot | −36.4 | −20.28% | −0.0 | 4,263 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −36.8 | −17.20% | −0.0 | 11,655 |
| 8-12 cards | bid 2+ more than the bot | −71.4 | −24.76% | −0.0 | 97,954 |
| 4-7 cards | bid 2+ more than the bot | −52.6 | −20.40% | −0.0 | 54,705 |
| 8-12 cards | bid 2+ more than the bot (bot bids 0) | −68.2 | −19.71% | −0.0 | 20,070 |
| 4-7 cards | bid 2+ fewer than the bot | −33.7 | −17.04% | −0.0 | 983 |
| 1-3 cards | bid 2+ fewer than the bot (you bid 0) | −34.5 | −17.91% | −0.0 | 403 |
| 1-3 cards | bid 2+ more than the bot | −39.3 | −18.78% | −0.0 | 2,181 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −4.7 | −2.48% | −0.5 | 8,332 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −4.2 | −2.51% | −0.5 | 12,689 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit low | −2.0 | −1.11% | −0.4 | 10,367 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −3.7 | −2.05% | −0.3 | 4,295 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −6.3 | −3.52% | −0.3 | 5,860 |
| made your bid, mid-trick, a Wizard is winning, no suit to follow | bot: duck high / you: duck low | −2.1 | −1.08% | −0.3 | 7,833 |
| made your bid, mid-trick, trump is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −0.9 | −0.48% | −0.3 | 11,488 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −4.5 | −2.41% | −0.3 | 5,733 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −7.1 | −3.86% | −0.2 | 4,213 |
| made your bid, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −0.8 | −0.51% | −0.2 | 11,920 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.0 | −4.97% | −0.2 | 6,197 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −4.5 | −1.95% | −0.2 | 4,691 |
| made your bid, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −3.0 | −1.72% | −0.2 | 3,232 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck low / you: duck high | −5.8 | −2.92% | −0.2 | 6,032 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: win cheaply | −2.0 | −1.10% | −0.2 | 4,096 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: win big | −4.7 | −2.18% | −0.2 | 2,507 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win cheaply | −10.5 | −4.87% | −0.2 | 3,364 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −1.4 | −0.80% | −0.2 | 6,857 |
| need more tricks, you lead | bot: Wizard / you: lead off-suit low | −3.4 | −2.48% | −0.1 | 2,954 |
| need more tricks, mid-trick, trump is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −6.1 | −3.45% | −0.1 | 2,219 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −3.5 | −1.92% | −0.1 | 3,669 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −5.2 | −2.81% | −0.1 | 4,384 |
| made your bid, mid-trick, a Wizard is winning, can't follow | bot: duck high / you: duck low | −1.7 | −1.08% | −0.1 | 4,201 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: duck high | −7.9 | −3.86% | −0.1 | 3,530 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −1.5 | −0.82% | −0.1 | 4,979 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 586 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −30.7 | −14.99% | −0.0 | 591 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −29.3 | −15.01% | −0.0 | 2,169 |
| need more tricks, mid-trick, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −28.2 | −14.89% | −0.0 | 1,376 |
| need more tricks, last to play, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −27.9 | −13.46% | −0.0 | 873 |
| need more tricks, mid-trick, a Wizard is winning, can follow | bot: duck low / you: Wizard | −23.3 | −12.49% | −0.0 | 2,862 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −23.1 | −12.02% | −0.0 | 1,840 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −13.9 | −7.13% | −0.0 | 677 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −13.2 | −7.49% | −0.0 | 729 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win big | −12.8 | −5.18% | −0.0 | 777 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −12.7 | −6.81% | −0.0 | 914 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: Jester / you: win cheaply | −11.4 | −6.25% | −0.1 | 2,063 |
| need more tricks, last to play, trump is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −11.3 | −6.44% | −0.0 | 625 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −10.7 | −5.97% | −0.0 | 1,679 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win cheaply | −10.5 | −4.87% | −0.2 | 3,364 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck high | −10.0 | −4.94% | −0.0 | 837 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −9.6 | −5.25% | −0.0 | 1,268 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.0 | −4.97% | −0.2 | 6,197 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win big (trump in) / you: duck low | −8.8 | −5.17% | −0.0 | 700 |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: duck high | −8.3 | −4.07% | −0.0 | 915 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Jester | −8.1 | −5.06% | −0.1 | 1,184 |
| made your bid, mid-trick, off-suit is winning, can't follow | bot: duck high / you: win cheaply (trump in) | −8.1 | −4.23% | −0.1 | 2,391 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: duck high | −7.9 | −3.86% | −0.1 | 3,530 |
| need more tricks, last to play, a Wizard is winning, can't follow | bot: duck low / you: duck high | −7.9 | −4.22% | −0.0 | 965 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −7.9 | −4.29% | −0.1 | 2,562 |
| need more tricks, mid-trick, a Wizard is winning, can't follow | bot: duck low / you: duck high | −7.8 | −4.02% | −0.1 | 1,609 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 3 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

