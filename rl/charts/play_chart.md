# Wizard play chart

How `simul1.pt` plays its cards after bidding, from self-play with everyone bidding at once.

- **win cheaply**: the lowest card that takes the trick; **win big**: a stronger winner than needed.
- **duck high**: the highest card that still loses (getting rid of danger); **duck low**: a lower loser.
- **trump in**: winning with trump when you can't follow the suit led.

## 4 players

300,122 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 45%, lead trump high (J+) 26%, lead off-suit high (K+) 11%, lead off-suit low 11%, lead trump low 7% | 1,392 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 81%, Wizard 11%, win big (trump in) 8% | 1,146 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 89%, duck low 7%, duck high 4% | 524 |
| middle | trump is winning | can follow | **win cheaply** 43%, Wizard 39%, win big 17% | 460 |
| middle | a Wizard is winning | no suit to follow | **duck low** 80%, Wizard 9%, duck high 8% | 411 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 88%, win big (trump in) 6%, Wizard 5% | 523 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 52%, lead off-suit high (K+) 21%, lead trump low 11%, Wizard 10%, lead trump high (J+) 4% | 56,097 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 50%, duck low 42%, Wizard 4%, Jester 4% | 29,246 |
| middle | off-suit is winning | can follow | **win cheaply** 48%, win big 26%, duck high 13%, Wizard 6%, duck low 4%, Jester 3% | 29,116 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 47%, win big (trump in) 22%, duck high 13%, duck low 11%, Jester 3% | 13,621 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 56%, duck high 23%, Wizard 16%, Jester 5% | 8,959 |
| middle | a Wizard is winning | no suit to follow | **duck low** 89%, Jester 9% | 8,055 |
| middle | trump is winning | can follow | **win cheaply** 44%, win big 18%, duck high 14%, Jester 13%, Wizard 7%, duck low 4% | 7,742 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 39%, duck low 35%, Jester 15%, Wizard 11% | 6,914 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 54%, duck high 29%, Wizard 12%, Jester 5% | 4,823 |
| middle | only Jesters so far | no suit to follow | **win big** 51%, win cheaply 25%, Wizard 18%, Jester 7% | 3,569 |
| middle | a Wizard is winning | can follow | **duck low** 72%, Jester 16%, duck high 11% | 1,417 |
| middle | a Wizard is winning | can't follow | **duck low** 86%, Jester 13% | 937 |
| middle | trump is winning | can't follow | **win cheaply (trump in)** 57%, win big (trump in) 16%, duck low 13%, duck high 10% | 751 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 47%, duck low 45%, Wizard 5%, Jester 4% | 16,470 |
| last | off-suit is winning | can follow | **win cheaply** 57%, win big 15%, duck high 15%, Wizard 7%, duck low 4% | 8,598 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 42%, duck low 37%, Wizard 12%, Jester 9% | 7,326 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 48%, win big (trump in) 24%, duck high 15%, duck low 9% | 5,075 |
| last | a Wizard is winning | no suit to follow | **duck low** 89%, Jester 9% | 4,252 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 62%, duck high 22%, Wizard 12%, Jester 4% | 3,635 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 54%, duck high 27%, Wizard 14%, Jester 5% | 3,061 |
| last | a Wizard is winning | can follow | **duck low** 73%, Jester 17%, duck high 10% | 2,754 |
| last | trump is winning | can follow | **win cheaply** 62%, win big 15%, duck high 15%, duck low 4% | 2,728 |
| last | a Wizard is winning | can't follow | **duck low** 87%, Jester 10% | 1,607 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 64%, win big (trump in) 17%, duck high 9%, duck low 7% | 1,462 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 60%, Jester 20%, lead trump low 17% | 19,662 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 52%, duck low 41%, Jester 6% | 5,812 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 55%, duck low 40%, Jester 5% | 5,175 |
| middle | a Wizard is winning | no suit to follow | **duck high** 65%, duck low 30%, Jester 3% | 4,708 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 66%, duck low 28%, Jester 7% | 3,494 |
| middle | off-suit is winning | can follow | **Jester** 40%, duck high 30%, win cheaply 21%, duck low 6% | 3,480 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 44%, Jester 31%, win big 24% | 1,693 |
| middle | off-suit is winning | can't follow | **duck high** 52%, duck low 27%, Jester 14%, win cheaply (trump in) 6% | 952 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 71%, duck low 23%, Jester 6% | 805 |
| middle | a Wizard is winning | can't follow | **duck high** 65%, duck low 29%, Jester 3% | 769 |
| middle | a Wizard is winning | can follow | **duck high** 72%, duck low 24% | 335 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 55%, duck low 38%, Jester 6% | 2,952 |
| last | a Wizard is winning | no suit to follow | **duck high** 65%, duck low 30% | 2,436 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 51%, duck low 43%, Jester 7% | 2,020 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 67%, duck low 26%, Jester 7% | 1,603 |
| last | a Wizard is winning | can't follow | **duck high** 65%, duck low 29%, Wizard 3% | 1,538 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 66%, duck low 27%, Jester 7% | 1,067 |
| last | off-suit is winning | can follow | **Jester** 42%, duck high 33%, win cheaply 9%, duck low 9%, win big 6% | 893 |
| last | a Wizard is winning | can follow | **duck high** 72%, duck low 19%, Jester 5%, Wizard 4% | 658 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 81%, lead trump low 6%, Jester 5%, lead off-suit high (K+) 5%, lead trump high (J+) 3% | 2,258 |
## 3 players

300,180 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 41%, lead trump high (J+) 21%, lead off-suit low 18%, lead off-suit high (K+) 11%, lead trump low 9% | 2,803 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 77%, win big (trump in) 12%, Wizard 10% | 914 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 46%, win big 29%, Wizard 25% | 556 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 82%, duck low 10%, duck high 7% | 426 |
| middle | a Wizard is winning | no suit to follow | **duck low** 81%, duck high 12%, Wizard 5% | 419 |
| middle | trump is winning | can follow | **win cheaply** 45%, Wizard 41%, win big 13% | 350 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 83%, win big (trump in) 9%, Wizard 7% | 1,054 |
| last | a Wizard is winning | no suit to follow | **duck low** 83%, duck high 12%, Wizard 3% | 626 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 81%, duck low 12%, duck high 6% | 415 |
| last | trump is winning | can follow | **win cheaply** 79%, Wizard 11%, win big 9% | 412 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **Wizard** 78%, duck low 13%, duck high 9% | 340 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 58%, lead off-suit high (K+) 19%, lead trump low 10%, Wizard 7%, lead trump high (J+) 4% | 85,421 |
| middle | off-suit is winning | can follow | **win cheaply** 45%, win big 28%, duck high 14%, duck low 6%, Wizard 3%, Jester 3% | 26,414 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 47%, duck low 46%, Jester 4%, Wizard 3% | 20,881 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 43%, win big (trump in) 21%, duck low 19%, duck high 12%, Jester 4% | 10,434 |
| middle | trump is winning | can follow | **win cheaply** 42%, win big 23%, duck high 12%, Jester 12%, Wizard 6%, duck low 5% | 6,032 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 60%, duck high 21%, Wizard 12%, Jester 7% | 5,611 |
| middle | a Wizard is winning | no suit to follow | **duck low** 84%, Jester 15% | 4,165 |
| middle | only Jesters so far | no suit to follow | **win big** 51%, win cheaply 24%, Wizard 15%, Jester 11% | 4,042 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 34%, duck low 30%, Jester 22%, Wizard 14% | 3,170 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 60%, duck high 26%, Wizard 8%, Jester 6% | 3,052 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck low** 47%, duck high 43%, Wizard 5%, Jester 4% | 24,086 |
| last | off-suit is winning | can follow | **win cheaply** 53%, win big 18%, duck high 15%, duck low 6%, Wizard 4% | 19,465 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 49%, win big (trump in) 22%, duck low 14%, duck high 12% | 9,133 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 36%, duck low 35%, Wizard 15%, Jester 13% | 7,619 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 60%, duck high 22%, Wizard 12%, Jester 6% | 5,282 |
| last | trump is winning | can follow | **win cheaply** 59%, win big 18%, duck high 12%, duck low 5% | 4,729 |
| last | a Wizard is winning | no suit to follow | **duck low** 85%, Jester 13% | 4,487 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 57%, duck high 23%, Wizard 14%, Jester 6% | 3,756 |
| last | a Wizard is winning | can follow | **duck low** 72%, Jester 19%, duck high 8% | 1,874 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 59%, win big (trump in) 21%, duck high 9%, duck low 8% | 1,293 |
| last | a Wizard is winning | can't follow | **duck low** 79%, Jester 17%, duck high 3% | 1,013 |
| last | only Jesters so far | no suit to follow | **win big** 50%, Wizard 23%, win cheaply 14%, Jester 13% | 684 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 50%, Jester 29%, lead trump low 16%, lead trump high (J+) 3% | 13,278 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 57%, duck low 37%, Jester 6% | 2,680 |
| middle | a Wizard is winning | no suit to follow | **duck high** 69%, duck low 25%, Wizard 4% | 2,185 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 64%, duck low 32%, Jester 4% | 1,765 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 73%, duck low 23%, Jester 4% | 1,247 |
| middle | off-suit is winning | can follow | **Jester** 48%, duck high 28%, win cheaply 13%, duck low 8% | 1,213 |
| middle | only Jesters so far | no suit to follow | **Jester** 43%, win cheaply 29%, win big 26% | 1,018 |
| middle | off-suit is winning | can't follow | **duck high** 46%, duck low 31%, Jester 15%, win cheaply (trump in) 7% | 452 |
| last | a Wizard is winning | no suit to follow | **duck high** 68%, duck low 25%, Wizard 5% | 2,365 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 63%, duck low 31%, Jester 6% | 2,240 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 59%, duck low 37%, Jester 5% | 2,009 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 74%, duck low 21%, Jester 4% | 1,177 |
| last | off-suit is winning | can follow | **Jester** 47%, duck high 30%, duck low 10%, win cheaply 8%, Wizard 3% | 722 |
| last | a Wizard is winning | can't follow | **duck high** 68%, duck low 24%, Wizard 6% | 715 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 77%, duck low 21% | 502 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 74%, Jester 9%, lead off-suit high (K+) 8%, lead trump low 5%, lead trump high (J+) 4% | 1,759 |
## 5 players

300,137 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 49%, lead trump high (J+) 26%, lead off-suit high (K+) 12%, lead off-suit low 8%, lead trump low 5% | 901 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 81%, Wizard 11%, win big (trump in) 8% | 1,108 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 89%, duck low 5%, duck high 4% | 623 |
| middle | trump is winning | can follow | **win cheaply** 52%, Wizard 34%, win big 13% | 479 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 49%, lead off-suit high (K+) 20%, Wizard 15%, lead trump low 12%, lead trump high (J+) 4% | 37,302 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 54%, duck low 36%, Wizard 6%, Jester 4% | 27,141 |
| middle | off-suit is winning | can follow | **win cheaply** 49%, win big 23%, duck high 12%, Wizard 9%, Jester 3% | 26,601 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 45%, win big (trump in) 24%, duck high 16%, duck low 9%, Wizard 4%, Jester 3% | 13,049 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 55%, duck high 23%, Wizard 19%, Jester 4% | 11,000 |
| middle | a Wizard is winning | no suit to follow | **duck low** 93%, Jester 6% | 10,535 |
| middle | trump is winning | can follow | **win cheaply** 46%, duck high 16%, win big 16%, Jester 12%, Wizard 6%, duck low 3% | 8,572 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 45%, duck low 35%, Jester 11%, Wizard 9% | 8,236 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 49%, duck high 34%, Wizard 13%, Jester 4% | 5,666 |
| middle | a Wizard is winning | can follow | **duck low** 75%, duck high 14%, Jester 11% | 3,206 |
| middle | only Jesters so far | no suit to follow | **win big** 46%, win cheaply 29%, Wizard 20%, Jester 5% | 2,558 |
| middle | a Wizard is winning | can't follow | **duck low** 87%, Jester 11% | 2,065 |
| middle | trump is winning | can't follow | **win cheaply (trump in)** 55%, win big (trump in) 17%, duck high 12%, duck low 12% | 1,161 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 49%, duck low 40%, Wizard 7%, Jester 5% | 9,983 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 45%, duck low 35%, Wizard 13%, Jester 7% | 5,493 |
| last | off-suit is winning | can follow | **win cheaply** 56%, duck high 15%, win big 13%, Wizard 10%, Jester 4% | 4,395 |
| last | a Wizard is winning | no suit to follow | **duck low** 93%, Jester 5% | 3,722 |
| last | a Wizard is winning | can follow | **duck low** 74%, duck high 14%, Jester 11% | 3,078 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 60%, duck high 21%, Wizard 16%, Jester 4% | 2,936 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 46%, win big (trump in) 25%, duck high 17%, duck low 7% | 2,886 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 52%, duck high 29%, Wizard 14%, Jester 5% | 2,398 |
| last | a Wizard is winning | can't follow | **duck low** 89%, Jester 9% | 1,871 |
| last | trump is winning | can follow | **win cheaply** 62%, duck high 18%, win big 12%, Wizard 3% | 1,775 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 68%, win big (trump in) 14%, duck high 10%, duck low 6% | 1,020 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 64%, lead trump low 18%, Jester 15% | 24,302 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 48%, duck low 45%, Jester 7% | 10,299 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 52%, duck low 42%, Jester 5% | 9,509 |
| middle | a Wizard is winning | no suit to follow | **duck high** 59%, duck low 36%, Jester 4% | 7,660 |
| middle | off-suit is winning | can follow | **Jester** 35%, duck high 32%, win cheaply 23%, duck low 6% | 6,970 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 62%, duck low 30%, Jester 8% | 6,750 |
| middle | a Wizard is winning | can't follow | **duck high** 59%, duck low 34%, Jester 5% | 2,312 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 68%, duck low 27%, Jester 5% | 2,279 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 51%, win big 26%, Jester 23% | 2,002 |
| middle | off-suit is winning | can't follow | **duck high** 53%, duck low 29%, Jester 15% | 1,868 |
| middle | a Wizard is winning | can follow | **duck high** 68%, duck low 24%, Jester 6% | 1,275 |
| middle | trump is winning | can follow | **Jester** 56%, win cheaply 31%, duck high 11% | 427 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 53%, duck low 40%, Jester 7% | 3,519 |
| last | a Wizard is winning | no suit to follow | **duck high** 58%, duck low 36%, Jester 4% | 2,505 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 47%, duck low 45%, Jester 8% | 2,275 |
| last | a Wizard is winning | can't follow | **duck high** 60%, duck low 34%, Jester 4% | 2,254 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 60%, duck low 31%, Jester 9% | 2,217 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 62%, duck low 32%, Jester 6% | 1,566 |
| last | a Wizard is winning | can follow | **duck high** 66%, duck low 24%, Jester 8% | 1,143 |
| last | off-suit is winning | can follow | **duck high** 42%, Jester 33%, win cheaply 9%, duck low 9%, win big 6% | 989 |
| last | off-suit is winning | can't follow | **duck high** 60%, duck low 27%, Jester 13% | 349 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 82%, lead trump low 8%, lead off-suit high (K+) 4% | 2,932 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 52%, duck high 48% | 357 |
| middle | a Wizard is winning | no suit to follow | **duck high** 52%, duck low 45% | 312 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 48%, duck low 48%, Jester 5% | 301 |
## 6 players

300,130 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 50%, lead trump high (J+) 26%, lead off-suit high (K+) 13%, lead off-suit low 8%, lead trump low 4% | 599 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 85%, Wizard 9%, win big (trump in) 6% | 847 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 93%, duck low 4% | 623 |
| middle | trump is winning | can follow | **win cheaply** 56%, Wizard 34%, win big 9% | 415 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 46%, lead off-suit high (K+) 18%, Wizard 18%, lead trump low 13%, lead trump high (J+) 3% | 25,751 |
| middle | off-suit is winning | can follow | **win cheaply** 47%, win big 20%, Wizard 16%, duck high 12%, Jester 4% | 22,376 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 57%, duck low 29%, Wizard 10%, Jester 4% | 21,305 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 50%, duck high 25%, Wizard 22% | 12,087 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 40%, win big (trump in) 24%, duck high 19%, duck low 9%, Wizard 6%, Jester 3% | 11,866 |
| middle | a Wizard is winning | no suit to follow | **duck low** 94%, Jester 5% | 10,256 |
| middle | trump is winning | can follow | **win cheaply** 47%, duck high 19%, win big 14%, Jester 11%, Wizard 6% | 8,615 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 48%, duck low 34%, Wizard 9%, Jester 9% | 7,971 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 41%, duck high 39%, Wizard 17% | 6,121 |
| middle | a Wizard is winning | can follow | **duck low** 76%, duck high 16%, Jester 8% | 5,495 |
| middle | a Wizard is winning | can't follow | **duck low** 91%, Jester 8% | 3,667 |
| middle | only Jesters so far | no suit to follow | **win big** 41%, win cheaply 33%, Wizard 22%, Jester 3% | 1,789 |
| middle | trump is winning | can't follow | **win cheaply (trump in)** 54%, duck high 15%, win big (trump in) 14%, duck low 12% | 1,255 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 52%, duck low 33%, Wizard 10%, Jester 5% | 5,718 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 46%, duck low 34%, Wizard 13%, Jester 6% | 3,596 |
| last | a Wizard is winning | can follow | **duck low** 77%, duck high 15%, Jester 8% | 3,376 |
| last | a Wizard is winning | no suit to follow | **duck low** 94%, Jester 4% | 2,729 |
| last | off-suit is winning | can follow | **win cheaply** 50%, Wizard 18%, duck high 14%, win big 11%, Jester 5% | 2,470 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 55%, duck high 23%, Wizard 19% | 2,346 |
| last | a Wizard is winning | can't follow | **duck low** 90%, Jester 8% | 2,093 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 46%, duck high 33%, Wizard 18%, Jester 4% | 1,842 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 41%, win big (trump in) 26%, duck high 20%, duck low 7%, Wizard 4% | 1,752 |
| last | trump is winning | can follow | **win cheaply** 60%, duck high 19%, win big 11%, duck low 4%, Wizard 4% | 1,368 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 66%, win big (trump in) 16%, duck high 9%, duck low 6% | 686 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 66%, lead trump low 19%, Jester 12% | 26,362 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 47%, duck high 46%, Jester 7% | 14,940 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 50%, duck low 44%, Jester 6% | 13,633 |
| middle | a Wizard is winning | no suit to follow | **duck high** 56%, duck low 38%, Jester 5% | 10,652 |
| middle | off-suit is winning | can follow | **duck high** 33%, Jester 32%, win cheaply 26%, duck low 5%, win big 3% | 10,611 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 62%, duck low 30%, Jester 8% | 9,980 |
| middle | a Wizard is winning | can't follow | **duck high** 58%, duck low 36%, Jester 5% | 4,976 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 67%, duck low 29%, Jester 5% | 3,861 |
| middle | a Wizard is winning | can follow | **duck high** 62%, duck low 28%, Jester 9% | 3,157 |
| middle | off-suit is winning | can't follow | **duck high** 52%, duck low 29%, Jester 14%, win cheaply (trump in) 3% | 3,098 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 55%, win big 27%, Jester 18% | 2,016 |
| middle | trump is winning | can follow | **Jester** 48%, win cheaply 31%, duck high 17%, win big 3% | 620 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 50%, duck low 43%, Jester 7% | 3,663 |
| last | a Wizard is winning | can't follow | **duck high** 56%, duck low 38%, Jester 5% | 2,976 |
| last | a Wizard is winning | no suit to follow | **duck high** 58%, duck low 37%, Jester 4% | 2,602 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 59%, duck low 30%, Jester 10% | 2,431 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 45%, duck low 45%, Jester 9% | 2,319 |
| last | a Wizard is winning | can follow | **duck high** 65%, duck low 26%, Jester 8% | 1,857 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 61%, duck low 33%, Jester 6% | 1,814 |
| last | off-suit is winning | can follow | **duck high** 43%, Jester 31%, win cheaply 12%, duck low 7%, win big 6% | 948 |
| last | off-suit is winning | can't follow | **duck high** 59%, duck low 24%, Jester 14% | 422 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 83%, lead trump low 10%, lead off-suit high (K+) 3% | 3,339 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 51%, duck high 49% | 472 |
| middle | a Wizard is winning | no suit to follow | **duck high** 53%, duck low 42%, Wizard 4% | 402 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 59%, duck high 38%, Jester 3% | 394 |
