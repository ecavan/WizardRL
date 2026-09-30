# Wizard mistake chart

What departing from `simul1.pt`'s choice costs, for a human-like player (picks with probabilities softmax(expected points / 3)), in full games with everyone bidding at once. **Points**: expected points lost that round, as the network sees it. **Win chance**: chance of winning the game lost (a rough reading, see `mistakes.py`). **Per game**: what it costs the human-like player over a whole game (how likely they are to pick it × what it costs, summed over the game); the habit lists are sorted by it. Every option at every decision is costed, so rare blunders show up too.

## 3 players

About 1,034 games. A player like this gives up about **14 points a game in bidding** and **201 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 13+ cards | bid 1 fewer than the bot | −9.0 | −1.83% | −3.2 | 24,572 |
| 13+ cards | bid 1 more than the bot | −10.8 | −2.25% | −2.6 | 24,576 |
| 8-12 cards | bid 1 fewer than the bot | −8.3 | −1.79% | −2.0 | 14,995 |
| 8-12 cards | bid 1 more than the bot | −10.6 | −2.29% | −1.7 | 15,351 |
| 4-7 cards | bid 1 more than the bot | −11.0 | −2.30% | −1.2 | 11,569 |
| 4-7 cards | bid 1 fewer than the bot | −9.0 | −1.86% | −1.0 | 7,933 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −14.5 | −2.03% | −0.4 | 4,693 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −12.9 | −2.67% | −0.3 | 3,642 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −17.3 | −2.69% | −0.3 | 4,232 |
| 1-3 cards | bid 1 more than the bot | −14.6 | −2.63% | −0.3 | 3,557 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −5.5 | −1.12% | −0.1 | 713 |
| 1-3 cards | bid 1 fewer than the bot | −11.2 | −2.01% | −0.1 | 867 |
| 13+ cards | bid 2+ fewer than the bot | −69.4 | −13.22% | −0.1 | 87,123 |
| 8-12 cards | bid 2+ fewer than the bot | −41.2 | −8.63% | −0.1 | 22,108 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −10.5 | −2.21% | −0.0 | 356 |
| 4-7 cards | bid 2+ more than the bot | −45.3 | −9.21% | −0.0 | 29,931 |
| 13+ cards | bid 2+ more than the bot | −110.7 | −19.75% | −0.0 | 244,657 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −30.0 | −5.74% | −0.0 | 3,169 |
| 8-12 cards | bid 2+ more than the bot | −71.9 | −14.26% | −0.0 | 85,721 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −38.5 | −7.93% | −0.0 | 7,933 |
| 4-7 cards | bid 2+ fewer than the bot | −32.9 | −6.93% | −0.0 | 3,328 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −55.1 | −11.34% | −0.0 | 14,995 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −40.3 | −7.99% | −0.0 | 2,535 |
| 1-3 cards | bid 2+ fewer than the bot (you bid 0) | −35.6 | −6.75% | −0.0 | 867 |
| 1-3 cards | bid 2+ more than the bot | −30.4 | −5.83% | −0.0 | 1,611 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −3.8 | −0.83% | −13.2 | 171,128 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −3.2 | −0.64% | −6.1 | 89,731 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −2.9 | −0.60% | −4.5 | 70,056 |
| need more tricks, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −3.0 | −0.57% | −3.8 | 69,107 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −3.7 | −0.73% | −2.9 | 42,639 |
| need more tricks, you lead | bot: lead off-suit low / you: Wizard | −8.9 | −1.76% | −2.6 | 63,281 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead trump low | −5.1 | −1.06% | −2.5 | 35,936 |
| need more tricks, you lead | bot: lead off-suit low / you: Jester | −9.6 | −1.95% | −2.4 | 61,953 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: win cheaply | −2.3 | −0.46% | −2.2 | 17,456 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −8.5 | −1.69% | −2.1 | 20,479 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −3.3 | −0.69% | −2.1 | 17,240 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −7.8 | −1.56% | −2.0 | 15,759 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −9.5 | −1.99% | −1.9 | 20,651 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck high | −6.0 | −1.29% | −1.9 | 14,746 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −7.4 | −1.47% | −1.9 | 13,769 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −3.6 | −0.69% | −1.8 | 13,123 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.9 | −0.55% | −1.6 | 13,287 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −9.3 | −1.89% | −1.6 | 14,379 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −8.3 | −1.66% | −1.6 | 12,315 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −4.8 | −0.99% | −1.5 | 14,945 |
| need more tricks, you lead | bot: lead trump high (J+) / you: lead off-suit low | −2.8 | −0.57% | −1.5 | 24,619 |
| need more tricks, you lead | bot: Wizard / you: lead off-suit low | −5.0 | −1.05% | −1.4 | 12,824 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −12.9 | −2.59% | −1.3 | 18,000 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: duck low | −2.0 | −0.36% | −1.3 | 10,102 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: win big | −1.8 | −0.31% | −1.2 | 11,390 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 2,564 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −29.5 | −5.95% | −0.0 | 2,734 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −17.8 | −3.70% | −0.1 | 4,836 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −16.2 | −3.36% | −0.2 | 5,074 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −16.0 | −3.36% | −0.2 | 3,920 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win big / you: Wizard | −13.6 | −2.90% | −0.4 | 6,819 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Jester | −13.2 | −2.67% | −1.1 | 17,603 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win big / you: Jester | −13.1 | −2.74% | −0.3 | 6,752 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −12.9 | −2.59% | −1.3 | 18,000 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −12.6 | −2.56% | −0.5 | 7,618 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −12.1 | −2.51% | −0.2 | 4,393 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −12.0 | −2.44% | −0.3 | 4,329 |
| need more tricks, last to play, off-suit is winning, can follow | bot: duck high / you: Jester | −11.5 | −2.35% | −0.2 | 4,293 |
| need more tricks, last to play, trump is winning, can follow | bot: win cheaply / you: Jester | −11.0 | −2.28% | −0.4 | 4,479 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −10.9 | −2.30% | −0.8 | 24,376 |
| need more tricks, you lead | bot: lead trump high (J+) / you: Jester | −10.4 | −2.14% | −0.2 | 4,174 |
| need more tricks, you lead | bot: lead trump low / you: Jester | −10.3 | −2.13% | −0.6 | 13,983 |
| need more tricks, last to play, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck low / you: Wizard | −10.3 | −1.92% | −0.2 | 3,689 |
| need more tricks, last to play, trump is winning, can follow | bot: win cheaply / you: Wizard | −10.2 | −2.12% | −0.4 | 4,348 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Wizard | −10.2 | −2.13% | −0.9 | 24,526 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Jester | −10.0 | −2.06% | −0.8 | 13,030 |
| need more tricks, last to play, trump is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −9.7 | −2.00% | −0.4 | 3,434 |
| made your bid, you lead | bot: Jester / you: lead off-suit low | −9.7 | −2.03% | −1.0 | 10,274 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −9.7 | −2.04% | −0.3 | 4,901 |
| need more tricks, you lead | bot: lead off-suit low / you: Jester | −9.6 | −1.95% | −2.4 | 61,953 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Wizard | −9.5 | −1.96% | −0.8 | 13,057 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 55 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

## 4 players

About 1,034 games. A player like this gives up about **9 points a game in bidding** and **92 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 8-12 cards | bid 1 fewer than the bot | −9.0 | −2.70% | −1.7 | 17,416 |
| 8-12 cards | bid 1 more than the bot | −11.6 | −3.44% | −1.5 | 20,247 |
| 13+ cards | bid 1 fewer than the bot | −9.3 | −2.85% | −1.1 | 11,853 |
| 13+ cards | bid 1 more than the bot | −11.9 | −3.84% | −0.9 | 12,271 |
| 4-7 cards | bid 1 more than the bot | −12.6 | −3.67% | −0.9 | 13,878 |
| 4-7 cards | bid 1 fewer than the bot | −9.3 | −2.84% | −0.7 | 6,800 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −13.2 | −3.85% | −0.5 | 7,080 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −17.7 | −4.46% | −0.3 | 7,753 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −7.4 | −2.16% | −0.3 | 2,504 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −17.1 | −4.41% | −0.2 | 4,482 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −12.2 | −3.40% | −0.2 | 2,831 |
| 1-3 cards | bid 1 more than the bot | −18.6 | −4.96% | −0.1 | 3,849 |
| 1-3 cards | bid 1 fewer than the bot | −10.9 | −3.05% | −0.1 | 646 |
| 8-12 cards | bid 1 more than the bot (bot bids 0) | −3.9 | −1.18% | −0.0 | 233 |
| 13+ cards | bid 1 fewer than the bot (you bid 0) | −12.5 | −3.45% | −0.0 | 418 |
| 13+ cards | bid 2+ fewer than the bot | −50.7 | −15.30% | −0.0 | 20,142 |
| 8-12 cards | bid 2+ fewer than the bot | −41.1 | −11.68% | −0.0 | 15,081 |
| 4-7 cards | bid 2+ more than the bot | −47.9 | −12.41% | −0.0 | 41,685 |
| 8-12 cards | bid 2+ more than the bot | −75.2 | −18.30% | −0.0 | 129,697 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −48.4 | −13.12% | −0.0 | 17,416 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −33.0 | −8.48% | −0.0 | 5,925 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −37.0 | −10.35% | −0.0 | 6,800 |
| 13+ cards | bid 2+ more than the bot | −100.0 | −24.68% | −0.0 | 115,257 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −43.7 | −11.10% | −0.0 | 9,684 |
| 4-7 cards | bid 2+ fewer than the bot | −34.2 | −9.94% | −0.0 | 1,681 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −3.8 | −1.14% | −5.0 | 77,835 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −2.6 | −0.78% | −1.8 | 31,260 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −6.6 | −2.14% | −1.5 | 14,010 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −2.3 | −0.70% | −1.3 | 26,209 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −8.1 | −2.50% | −1.3 | 14,987 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −5.7 | −1.77% | −1.3 | 10,588 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −5.1 | −1.58% | −1.2 | 15,267 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −7.7 | −2.39% | −1.2 | 13,482 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −4.0 | −1.17% | −1.2 | 11,411 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: win cheaply | −2.4 | −0.75% | −1.2 | 12,313 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.4 | −0.68% | −1.2 | 12,947 |
| need more tricks, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −2.9 | −0.94% | −1.0 | 23,574 |
| need more tricks, you lead | bot: lead off-suit low / you: Wizard | −7.3 | −2.17% | −1.0 | 24,452 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −3.5 | −1.07% | −1.0 | 16,437 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead trump low | −4.5 | −1.41% | −0.9 | 13,772 |
| need more tricks, you lead | bot: Wizard / you: lead off-suit low | −3.6 | −1.23% | −0.9 | 10,725 |
| need more tricks, you lead | bot: lead off-suit low / you: Jester | −7.1 | −2.29% | −0.9 | 20,156 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −6.8 | −2.12% | −0.8 | 7,824 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit low | −3.0 | −0.90% | −0.8 | 9,926 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.8 | −0.90% | −0.7 | 7,937 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −6.7 | −2.00% | −0.7 | 8,704 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.3 | −0.40% | −0.7 | 8,305 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −6.5 | −1.96% | −0.6 | 6,003 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: Wizard | −6.7 | −2.06% | −0.6 | 8,007 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: win big (trump in) | −2.9 | −0.86% | −0.6 | 6,864 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 1,366 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −30.1 | −8.84% | −0.0 | 1,485 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −28.2 | −8.00% | −0.0 | 2,893 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −19.0 | −5.50% | −0.1 | 3,055 |
| need more tricks, mid-trick, a Wizard is winning, can follow | bot: duck low / you: Wizard | −18.8 | −5.29% | −0.0 | 1,463 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −18.8 | −5.96% | −0.0 | 1,997 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −17.6 | −5.46% | −0.0 | 1,943 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win big / you: Wizard | −13.8 | −4.60% | −0.1 | 1,400 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: Jester / you: win cheaply | −13.7 | −4.63% | −0.1 | 1,613 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Jester | −13.6 | −4.30% | −0.3 | 4,860 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −13.4 | −4.08% | −0.2 | 3,648 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −12.8 | −3.81% | −0.3 | 5,518 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win cheaply | −12.4 | −3.94% | −0.1 | 1,666 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −12.2 | −3.93% | −0.1 | 2,249 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −12.0 | −3.72% | −0.2 | 4,221 |
| need more tricks, last to play, trump is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −10.9 | −3.17% | −0.1 | 1,494 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −10.6 | −3.21% | −0.3 | 6,147 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −10.5 | −3.27% | −0.4 | 7,087 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win big (trump in) / you: Wizard | −9.6 | −3.04% | −0.1 | 1,885 |
| need more tricks, mid-trick, trump is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −9.5 | −2.87% | −0.1 | 1,578 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −9.4 | −3.13% | −0.3 | 9,179 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −9.3 | −2.87% | −0.2 | 4,312 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck low | −9.2 | −2.80% | −0.2 | 3,380 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Wizard | −9.1 | −2.82% | −0.3 | 9,863 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: Wizard | −9.1 | −2.85% | −0.4 | 6,793 |
| need more tricks, mid-trick, trump is winning, can follow, nothing but a Wizard wins | bot: Jester / you: duck high | −9.0 | −2.69% | −0.1 | 1,369 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 22 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

## 5 players

About 1,033 games. A player like this gives up about **6 points a game in bidding** and **49 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 8-12 cards | bid 1 more than the bot | −12.6 | −5.13% | −1.3 | 24,148 |
| 8-12 cards | bid 1 fewer than the bot | −9.5 | −4.09% | −1.3 | 17,240 |
| 4-7 cards | bid 1 more than the bot | −13.6 | −4.92% | −0.7 | 15,270 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −13.5 | −4.87% | −0.5 | 9,460 |
| 4-7 cards | bid 1 fewer than the bot | −10.2 | −3.78% | −0.4 | 5,810 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −9.1 | −3.24% | −0.4 | 5,210 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −12.2 | −4.61% | −0.4 | 6,908 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −19.1 | −6.07% | −0.3 | 10,903 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −17.7 | −6.17% | −0.2 | 4,551 |
| 8-12 cards | bid 1 more than the bot (bot bids 0) | −5.5 | −2.03% | −0.2 | 1,452 |
| 1-3 cards | bid 1 more than the bot | −20.5 | −7.14% | −0.1 | 3,900 |
| 1-3 cards | bid 1 fewer than the bot | −10.9 | −4.35% | −0.0 | 458 |
| 8-12 cards | bid 2+ fewer than the bot | −41.5 | −16.77% | −0.0 | 11,487 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −46.3 | −17.18% | −0.0 | 17,240 |
| 8-12 cards | bid 2+ more than the bot | −77.9 | −22.90% | −0.0 | 165,297 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −46.1 | −13.53% | −0.0 | 21,177 |
| 4-7 cards | bid 2+ more than the bot | −50.2 | −15.37% | −0.0 | 48,866 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −34.8 | −10.93% | −0.0 | 9,012 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −37.2 | −12.66% | −0.0 | 5,810 |
| 8-12 cards | bid 2+ more than the bot (bot bids 0) | −69.7 | −18.91% | −0.0 | 12,228 |
| 4-7 cards | bid 2+ fewer than the bot | −35.5 | −12.67% | −0.0 | 1,037 |
| 1-3 cards | bid 2+ fewer than the bot (you bid 0) | −36.4 | −12.58% | −0.0 | 458 |
| 1-3 cards | bid 2+ more than the bot | −36.4 | −12.22% | −0.0 | 1,982 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −4.0 | −1.65% | −1.7 | 32,542 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −5.8 | −2.46% | −1.0 | 11,046 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −7.4 | −2.98% | −0.8 | 9,544 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −4.6 | −1.88% | −0.7 | 7,123 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −5.4 | −2.13% | −0.7 | 10,220 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −4.7 | −1.90% | −0.6 | 7,466 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −7.4 | −3.06% | −0.6 | 7,631 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −2.3 | −0.96% | −0.6 | 12,550 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.2 | −1.04% | −0.6 | 8,476 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: win cheaply | −2.5 | −0.95% | −0.6 | 7,118 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit low | −2.7 | −1.04% | −0.6 | 10,085 |
| need more tricks, you lead | bot: Wizard / you: lead off-suit low | −3.0 | −1.46% | −0.5 | 8,896 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −2.1 | −0.86% | −0.5 | 10,962 |
| need more tricks, you lead | bot: lead off-suit low / you: Wizard | −6.2 | −2.54% | −0.5 | 11,857 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −6.6 | −2.60% | −0.5 | 6,641 |
| made your bid, mid-trick, trump is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.6 | −0.63% | −0.5 | 7,599 |
| made your bid, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.5 | −0.73% | −0.4 | 7,306 |
| need more tricks, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −3.1 | −1.29% | −0.4 | 10,971 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −3.5 | −1.46% | −0.4 | 7,670 |
| made your bid, mid-trick, a Wizard is winning, no suit to follow | bot: duck high / you: duck low | −2.8 | −1.21% | −0.4 | 5,336 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck low / you: duck high | −6.4 | −2.56% | −0.4 | 7,738 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.1 | −0.45% | −0.4 | 5,952 |
| need more tricks, last to play, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −6.1 | −2.45% | −0.3 | 3,839 |
| need more tricks, you lead | bot: lead off-suit low / you: Jester | −5.8 | −2.44% | −0.3 | 7,698 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.9 | −3.60% | −0.3 | 6,798 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 852 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −29.8 | −12.04% | −0.0 | 860 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −29.3 | −11.60% | −0.0 | 2,558 |
| need more tricks, last to play, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −27.0 | −10.52% | −0.0 | 950 |
| need more tricks, mid-trick, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −26.9 | −9.48% | −0.0 | 949 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −21.4 | −8.32% | −0.0 | 2,406 |
| need more tricks, mid-trick, a Wizard is winning, can follow | bot: duck low / you: Wizard | −20.5 | −8.33% | −0.0 | 2,285 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Jester | −15.0 | −6.66% | −0.1 | 1,462 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −13.8 | −5.34% | −0.1 | 1,603 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −13.0 | −5.18% | −0.1 | 1,874 |
| need more tricks, last to play, trump is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −12.1 | −4.82% | −0.0 | 945 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −11.8 | −4.87% | −0.1 | 2,802 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −11.7 | −4.62% | −0.1 | 2,669 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: Jester / you: win cheaply | −11.6 | −4.33% | −0.1 | 1,990 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck low | −11.0 | −4.65% | −0.1 | 1,070 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win cheaply | −11.0 | −3.82% | −0.2 | 2,684 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −10.5 | −4.73% | −0.1 | 2,256 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.9 | −3.60% | −0.3 | 6,798 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck high | −9.6 | −3.50% | −0.1 | 2,040 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −9.1 | −3.64% | −0.1 | 2,894 |
| need more tricks, mid-trick, trump is winning, can follow, nothing but a Wizard wins | bot: Jester / you: duck high | −9.1 | −3.28% | −0.1 | 919 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −8.9 | −3.56% | −0.1 | 3,539 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win big (trump in) / you: Wizard | −8.8 | −3.77% | −0.0 | 1,253 |
| made your bid, mid-trick, off-suit is winning, can't follow | bot: duck high / you: win cheaply (trump in) | −8.7 | −3.09% | −0.1 | 1,636 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win big (trump in) / you: duck low | −8.5 | −3.60% | −0.1 | 1,253 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit high (K+) | −8.5 | −3.14% | −0.1 | 1,338 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 11 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

## 6 players

About 1,031 games. A player like this gives up about **4 points a game in bidding** and **29 in card play**, against playing the bot's choice every time.

### Bidding (sorted by what it costs per game)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| 8-12 cards | bid 1 more than the bot | −13.4 | −7.26% | −0.7 | 15,737 |
| 4-7 cards | bid 1 more than the bot | −15.3 | −7.90% | −0.5 | 15,973 |
| 8-12 cards | bid 1 fewer than the bot | −9.9 | −5.67% | −0.5 | 8,754 |
| 4-7 cards | bid 1 more than the bot (bot bids 0) | −11.1 | −5.54% | −0.5 | 8,602 |
| 4-7 cards | bid 1 fewer than the bot (you bid 0) | −13.8 | −7.18% | −0.5 | 10,889 |
| 8-12 cards | bid 1 fewer than the bot (you bid 0) | −12.4 | −6.17% | −0.3 | 6,983 |
| 4-7 cards | bid 1 fewer than the bot | −10.7 | −6.04% | −0.3 | 5,085 |
| 1-3 cards | bid 1 more than the bot (bot bids 0) | −20.8 | −11.32% | −0.2 | 13,744 |
| 8-12 cards | bid 1 more than the bot (bot bids 0) | −7.7 | −3.09% | −0.2 | 2,695 |
| 1-3 cards | bid 1 fewer than the bot (you bid 0) | −18.0 | −10.13% | −0.2 | 4,780 |
| 1-3 cards | bid 1 more than the bot | −23.4 | −12.73% | −0.1 | 4,098 |
| 1-3 cards | bid 1 fewer than the bot | −11.0 | −6.40% | −0.0 | 347 |
| 4-7 cards | bid 2+ more than the bot (bot bids 0) | −48.7 | −17.86% | −0.0 | 35,998 |
| 8-12 cards | bid 2+ fewer than the bot (you bid 0) | −42.1 | −20.26% | −0.0 | 8,754 |
| 1-3 cards | bid 2+ more than the bot (bot bids 0) | −36.7 | −17.13% | −0.0 | 11,861 |
| 8-12 cards | bid 2+ more than the bot | −71.4 | −24.91% | −0.0 | 97,698 |
| 4-7 cards | bid 2+ more than the bot | −52.5 | −20.58% | −0.0 | 52,728 |
| 4-7 cards | bid 2+ fewer than the bot (you bid 0) | −37.5 | −18.04% | −0.0 | 5,085 |
| 8-12 cards | bid 2+ fewer than the bot | −39.1 | −21.10% | −0.0 | 3,911 |
| 8-12 cards | bid 2+ more than the bot (bot bids 0) | −67.7 | −19.73% | −0.0 | 21,356 |
| 4-7 cards | bid 2+ fewer than the bot | −37.4 | −18.83% | −0.0 | 808 |
| 1-3 cards | bid 2+ more than the bot | −39.4 | −18.96% | −0.0 | 2,121 |
| 1-3 cards | bid 2+ fewer than the bot (you bid 0) | −36.7 | −18.89% | −0.0 | 347 |

### Card play: the costliest habits

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: lead off-suit low | −4.4 | −2.68% | −0.6 | 13,833 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −5.2 | −2.79% | −0.6 | 7,949 |
| made your bid, you lead | bot: lead trump low / you: lead off-suit low | −2.5 | −1.37% | −0.4 | 8,724 |
| made your bid, mid-trick, trump is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.5 | −0.80% | −0.4 | 8,402 |
| made your bid, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.4 | −1.18% | −0.4 | 8,749 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −6.8 | −3.79% | −0.4 | 5,659 |
| made your bid, mid-trick, a Wizard is winning, no suit to follow | bot: duck high / you: duck low | −2.7 | −1.48% | −0.4 | 6,619 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −4.0 | −2.22% | −0.4 | 4,151 |
| need more tricks, you lead | bot: Wizard / you: lead off-suit low | −2.8 | −1.84% | −0.3 | 6,441 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: duck high | −5.1 | −2.26% | −0.3 | 4,412 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −5.4 | −2.90% | −0.3 | 5,410 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win cheaply / you: Jester | −7.6 | −4.16% | −0.3 | 4,056 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: win big / you: win cheaply | −2.5 | −1.39% | −0.3 | 3,840 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck low / you: duck high | −2.1 | −1.23% | −0.3 | 4,587 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −6.2 | −3.37% | −0.3 | 4,219 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.3 | −5.09% | −0.2 | 6,278 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck low / you: duck high | −6.2 | −3.04% | −0.2 | 6,027 |
| made your bid, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: Jester | −3.3 | −1.97% | −0.2 | 3,021 |
| need more tricks, you lead | bot: lead off-suit low / you: Wizard | −5.4 | −3.08% | −0.2 | 6,232 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump low | −2.1 | −1.15% | −0.2 | 5,226 |
| need more tricks, you lead | bot: lead trump low / you: lead off-suit low | −2.1 | −1.14% | −0.2 | 5,472 |
| made your bid, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.0 | −0.70% | −0.2 | 3,847 |
| need more tricks, you lead | bot: lead off-suit low / you: lead trump high (J+) | −3.9 | −2.13% | −0.2 | 4,066 |
| need more tricks, mid-trick, off-suit is winning, can follow, nothing but a Wizard wins | bot: duck high / you: duck low | −1.1 | −0.47% | −0.2 | 3,893 |
| need more tricks, mid-trick, off-suit is winning, can follow | bot: Wizard / you: win cheaply | −6.3 | −3.66% | −0.2 | 2,843 |

### Card play: the biggest mistakes, whether or not people make them (an option at least 587 times)

| Situation | Mistake | Points | Win chance | Per game | Times it was an option |
| --- | --- | ---: | ---: | ---: | ---: |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −31.3 | −14.98% | −0.0 | 600 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: Wizard | −29.7 | −15.29% | −0.0 | 2,170 |
| need more tricks, mid-trick, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −28.4 | −14.75% | −0.0 | 1,393 |
| need more tricks, last to play, a Wizard is winning, can't follow | bot: duck low / you: Wizard | −28.4 | −13.51% | −0.0 | 866 |
| need more tricks, mid-trick, a Wizard is winning, can follow | bot: duck low / you: Wizard | −23.3 | −12.51% | −0.0 | 2,868 |
| need more tricks, last to play, a Wizard is winning, can follow | bot: duck low / you: Wizard | −23.1 | −11.93% | −0.0 | 1,829 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck high | −14.8 | −7.64% | −0.0 | 736 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: Wizard | −13.6 | −7.70% | −0.0 | 732 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win big | −13.5 | −5.30% | −0.0 | 706 |
| need more tricks, last to play, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −13.4 | −7.11% | −0.0 | 1,004 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: Jester / you: win big | −13.0 | −5.79% | −0.0 | 605 |
| made your bid, you lead | bot: Jester / you: lead off-suit high (K+) | −12.5 | −7.29% | −0.0 | 600 |
| need more tricks, last to play, trump is winning, can't follow | bot: win cheaply (trump in) / you: duck low | −11.9 | −6.61% | −0.0 | 635 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Wizard | −11.6 | −6.53% | −0.0 | 1,618 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: Jester / you: win cheaply | −11.3 | −6.13% | −0.1 | 2,167 |
| made your bid, mid-trick, off-suit is winning, can follow | bot: duck high / you: win cheaply | −11.1 | −5.12% | −0.2 | 3,236 |
| need more tricks, last to play, off-suit is winning, can follow | bot: win cheaply / you: duck high | −10.4 | −5.17% | −0.0 | 833 |
| made your bid, you lead | bot: lead off-suit low / you: lead off-suit high (K+) | −9.3 | −5.09% | −0.2 | 6,278 |
| need more tricks, you lead | bot: lead off-suit high (K+) / you: Jester | −9.2 | −4.73% | −0.0 | 1,430 |
| need more tricks, mid-trick, off-suit is winning, can't follow, nothing but a Wizard wins | bot: duck high / you: Wizard | −9.0 | −4.97% | −0.1 | 2,320 |
| need more tricks, last to play, a Wizard is winning, no suit to follow | bot: duck low / you: duck high | −8.8 | −4.26% | −0.0 | 915 |
| need more tricks, mid-trick, off-suit is winning, can't follow | bot: win cheaply (trump in) / you: Jester | −8.6 | −4.54% | −0.1 | 1,717 |
| made your bid, mid-trick, off-suit is winning, can't follow | bot: duck high / you: win cheaply (trump in) | −8.6 | −4.49% | −0.1 | 2,063 |
| need more tricks, mid-trick, a Wizard is winning, no suit to follow | bot: duck low / you: duck high | −8.4 | −4.11% | −0.1 | 3,494 |
| need more tricks, last to play, a Wizard is winning, can't follow | bot: duck low / you: duck high | −8.3 | −4.49% | −0.0 | 955 |

Picking a different card of the same kind as the bot (another low off-suit lead, another low card to duck with) adds up to 6 points a game. Those are mostly small differences (one or two points each), where the network's own estimates are least sure.

