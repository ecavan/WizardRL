# Wizard play chart

How `ppo5.pt` plays its cards after bidding, from self-play with everyone bidding at once.

- **win cheaply**: the lowest card that takes the trick; **win big**: a stronger winner than needed.
- **duck high**: the highest card that still loses (getting rid of danger); **duck low**: a lower loser.
- **trump in**: winning with trump when you can't follow the suit led.

## 4 players

300,042 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 34%, lead off-suit low 23%, lead trump high (J+) 23%, lead off-suit high (K+) 11%, lead trump low 8% | 1,416 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 90%, win big (trump in) 6%, Wizard 4% | 1,222 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 81%, duck low 14%, duck high 5% | 633 |
| middle | a Wizard is winning | no suit to follow | **duck low** 83%, duck high 7%, Wizard 5%, Jester 4% | 493 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 60%, win big 30%, Wizard 10% | 392 |
| middle | trump is winning | can follow | **win cheaply** 51%, Wizard 30%, win big 18% | 383 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **Wizard** 85%, duck low 8%, duck high 6% | 363 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 93%, win big (trump in) 5% | 494 |
| last | a Wizard is winning | no suit to follow | **duck low** 89%, Wizard 6%, duck high 3% | 365 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **Wizard** 78%, duck low 11%, duck high 8%, Jester 3% | 300 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 56%, lead off-suit high (K+) 21%, lead trump low 11%, Wizard 6%, lead trump high (J+) 4% | 57,897 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 48%, duck low 45%, Wizard 5% | 30,429 |
| middle | off-suit is winning | can follow | **win cheaply** 52%, win big 28%, duck high 11%, Wizard 4% | 29,232 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 57%, win big (trump in) 23%, duck high 11%, duck low 6% | 13,937 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 57%, duck high 24%, Wizard 15%, Jester 4% | 10,598 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 36%, duck low 34%, Wizard 16%, Jester 15% | 7,504 |
| middle | trump is winning | can follow | **win cheaply** 48%, win big 20%, Jester 14%, duck high 10%, Wizard 5% | 7,412 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 52%, duck high 32%, Wizard 11%, Jester 5% | 5,771 |
| middle | a Wizard is winning | no suit to follow | **duck low** 86%, Jester 12% | 5,005 |
| middle | only Jesters so far | no suit to follow | **win big** 46%, win cheaply 29%, Wizard 19%, Jester 6% | 3,015 |
| middle | a Wizard is winning | can follow | **duck low** 70%, Jester 19%, duck high 11% | 1,263 |
| middle | trump is winning | can't follow | **win cheaply (trump in)** 61%, win big (trump in) 23%, duck low 8%, duck high 6% | 931 |
| middle | a Wizard is winning | can't follow | **duck low** 82%, Jester 16% | 855 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck low** 47%, duck high 43%, Wizard 8% | 16,436 |
| last | off-suit is winning | can follow | **win cheaply** 66%, win big 15%, duck high 11%, Wizard 5% | 8,479 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck low** 38%, duck high 35%, Wizard 20%, Jester 7% | 8,188 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 57%, win big (trump in) 25%, duck high 12%, duck low 5% | 4,989 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 57%, duck high 23%, Wizard 16%, Jester 4% | 4,352 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 49%, duck high 25%, Wizard 22%, Jester 4% | 3,802 |
| last | a Wizard is winning | can follow | **duck low** 70%, Jester 18%, duck high 12% | 2,727 |
| last | a Wizard is winning | no suit to follow | **duck low** 86%, Jester 11% | 2,714 |
| last | trump is winning | can follow | **win cheaply** 68%, win big 15%, duck high 12% | 2,563 |
| last | a Wizard is winning | can't follow | **duck low** 84%, Jester 13% | 1,901 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 68%, win big (trump in) 22%, duck high 5%, duck low 3% | 1,627 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 60%, Jester 20%, lead trump low 18% | 17,889 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 67%, duck low 28%, Jester 5% | 6,202 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 70%, duck low 27% | 4,794 |
| middle | off-suit is winning | can follow | **Jester** 37%, duck high 33%, win cheaply 20%, duck low 8% | 3,637 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 77%, duck low 19%, Jester 3% | 3,578 |
| middle | a Wizard is winning | no suit to follow | **duck high** 74%, duck low 22% | 3,274 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 51%, Jester 32%, win big 16% | 1,589 |
| middle | off-suit is winning | can't follow | **duck high** 55%, duck low 28%, Jester 12%, win cheaply (trump in) 4% | 966 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 77%, duck low 20%, Jester 3% | 860 |
| middle | a Wizard is winning | can't follow | **duck high** 79%, duck low 16%, Wizard 3% | 671 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 70%, duck low 28% | 2,807 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 66%, duck low 29%, Jester 4% | 2,113 |
| last | a Wizard is winning | no suit to follow | **duck high** 76%, duck low 19% | 1,735 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 78%, duck low 20% | 1,621 |
| last | a Wizard is winning | can't follow | **duck high** 75%, duck low 19%, Wizard 4% | 1,507 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 71%, duck low 24%, Jester 5% | 1,021 |
| last | off-suit is winning | can follow | **Jester** 38%, duck high 37%, duck low 10%, win cheaply 7%, win big 6% | 862 |
| last | a Wizard is winning | can follow | **duck high** 82%, duck low 15% | 604 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 79%, lead trump low 6%, lead off-suit high (K+) 5%, Jester 5%, lead trump high (J+) 4% | 2,145 |
## 3 players

300,115 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 30%, lead off-suit low 27%, lead trump high (J+) 21%, lead off-suit high (K+) 12%, lead trump low 10% | 2,848 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 86%, win big (trump in) 8%, Wizard 4% | 919 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 60%, win big 34%, Wizard 6% | 865 |
| middle | a Wizard is winning | no suit to follow | **duck low** 86%, duck high 9%, Wizard 3% | 598 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 74%, duck low 19%, duck high 6% | 482 |
| middle | trump is winning | can follow | **win cheaply** 50%, Wizard 30%, win big 18% | 352 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 91%, win big (trump in) 5% | 1,107 |
| last | a Wizard is winning | no suit to follow | **duck low** 87%, duck high 6%, Wizard 3%, Jester 3% | 812 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 70%, duck low 18%, duck high 9% | 550 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **Wizard** 75%, duck high 12%, duck low 11% | 395 |
| last | trump is winning | can follow | **win cheaply** 86%, win big 9%, Wizard 4% | 382 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 62%, lead off-suit high (K+) 18%, lead trump low 9%, Wizard 4%, lead trump high (J+) 4% | 87,434 |
| middle | off-suit is winning | can follow | **win cheaply** 50%, win big 31%, duck high 11%, duck low 3% | 27,172 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck low** 49%, duck high 42%, Wizard 6%, Jester 4% | 21,342 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 55%, win big (trump in) 21%, duck low 12%, duck high 9% | 11,837 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 58%, duck high 21%, Wizard 13%, Jester 8% | 7,023 |
| middle | trump is winning | can follow | **win cheaply** 44%, win big 22%, Jester 16%, duck high 9%, Wizard 6% | 4,991 |
| middle | only Jesters so far | no suit to follow | **win big** 53%, win cheaply 23%, Wizard 15%, Jester 9% | 3,478 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 55%, duck high 27%, Wizard 11%, Jester 7% | 3,177 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 29%, duck low 28%, Jester 23%, Wizard 20% | 3,009 |
| middle | a Wizard is winning | no suit to follow | **duck low** 76%, Jester 21% | 2,814 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck low** 50%, duck high 39%, Wizard 8%, Jester 3% | 23,667 |
| last | off-suit is winning | can follow | **win cheaply** 64%, win big 17%, duck high 12%, duck low 3% | 19,386 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 61%, win big (trump in) 22%, duck high 8%, duck low 7% | 9,616 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck low** 35%, duck high 32%, Wizard 22%, Jester 11% | 8,358 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 59%, duck high 21%, Wizard 14%, Jester 6% | 6,065 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 53%, duck high 24%, Wizard 18%, Jester 5% | 4,459 |
| last | trump is winning | can follow | **win cheaply** 65%, win big 17%, duck high 11%, duck low 3% | 3,809 |
| last | a Wizard is winning | no suit to follow | **duck low** 79%, Jester 18%, duck high 3% | 3,025 |
| last | a Wizard is winning | can follow | **duck low** 67%, Jester 24%, duck high 8% | 2,122 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 62%, win big (trump in) 28%, duck low 5%, duck high 4% | 2,071 |
| last | a Wizard is winning | can't follow | **duck low** 77%, Jester 21% | 1,243 |
| last | only Jesters so far | no suit to follow | **win big** 56%, Wizard 20%, win cheaply 15%, Jester 8% | 499 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 50%, Jester 31%, lead trump low 15% | 11,492 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 66%, duck low 26%, Jester 9% | 2,600 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 70%, duck low 26%, Jester 5% | 1,482 |
| middle | a Wizard is winning | no suit to follow | **duck high** 75%, duck low 18%, Wizard 4%, Jester 3% | 1,303 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 81%, duck low 15%, Jester 4% | 1,185 |
| middle | off-suit is winning | can follow | **Jester** 48%, duck high 29%, win cheaply 14%, duck low 7% | 1,167 |
| middle | only Jesters so far | no suit to follow | **Jester** 46%, win cheaply 37%, win big 16% | 936 |
| middle | off-suit is winning | can't follow | **duck high** 46%, duck low 29%, Jester 15%, win cheaply (trump in) 8% | 331 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 72%, duck low 22%, Jester 5% | 1,969 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 71%, duck low 26%, Jester 4% | 1,894 |
| last | a Wizard is winning | no suit to follow | **duck high** 77%, duck low 16%, Wizard 5% | 1,592 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 84%, duck low 14% | 1,018 |
| last | off-suit is winning | can follow | **Jester** 45%, duck high 32%, duck low 10%, win cheaply 6%, win big 6% | 739 |
| last | a Wizard is winning | can't follow | **duck high** 75%, duck low 15%, Wizard 5%, Jester 5% | 693 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 74%, duck low 22%, Jester 4% | 418 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 76%, Jester 9%, lead off-suit high (K+) 6%, lead trump low 4% | 1,487 |
## 5 players

300,027 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 43%, lead trump high (J+) 21%, lead off-suit low 17%, lead off-suit high (K+) 13%, lead trump low 5% | 823 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 90%, Wizard 5%, win big (trump in) 4% | 1,066 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 87%, duck low 8%, duck high 5% | 693 |
| middle | a Wizard is winning | no suit to follow | **duck low** 88%, Wizard 7% | 380 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **Wizard** 88%, duck low 8%, duck high 3% | 352 |
| middle | trump is winning | can follow | **Wizard** 45%, win cheaply 40%, win big 14% | 332 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 52%, lead off-suit high (K+) 22%, lead trump low 12%, Wizard 8%, lead trump high (J+) 4% | 39,757 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 53%, duck low 37%, Wizard 7% | 29,544 |
| middle | off-suit is winning | can follow | **win cheaply** 53%, win big 26%, duck high 10%, Wizard 8% | 26,458 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 53%, win big (trump in) 27%, duck high 12%, duck low 4% | 13,161 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 55%, duck high 26%, Wizard 16%, Jester 3% | 13,068 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 38%, duck low 35%, Wizard 15%, Jester 12% | 9,977 |
| middle | trump is winning | can follow | **win cheaply** 47%, win big 18%, Jester 15%, duck high 12%, Wizard 6% | 8,479 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 46%, duck high 37%, Wizard 13%, Jester 4% | 7,339 |
| middle | a Wizard is winning | no suit to follow | **duck low** 89%, Jester 9% | 6,060 |
| middle | a Wizard is winning | can follow | **duck low** 72%, duck high 15%, Jester 13% | 3,257 |
| middle | a Wizard is winning | can't follow | **duck low** 88%, Jester 10% | 2,193 |
| middle | only Jesters so far | no suit to follow | **win big** 45%, win cheaply 33%, Wizard 17%, Jester 4% | 2,137 |
| middle | trump is winning | can't follow | **win cheaply (trump in)** 66%, win big (trump in) 18%, duck high 8%, duck low 6% | 1,324 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 46%, duck low 40%, Wizard 11% | 10,475 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 37%, duck low 36%, Wizard 21%, Jester 6% | 6,288 |
| last | off-suit is winning | can follow | **win cheaply** 65%, win big 15%, duck high 9%, Wizard 7% | 4,240 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 56%, duck high 22%, Wizard 19%, Jester 3% | 3,454 |
| last | a Wizard is winning | can follow | **duck low** 72%, duck high 15%, Jester 13% | 3,398 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 46%, duck high 29%, Wizard 22%, Jester 3% | 3,048 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 54%, win big (trump in) 29%, duck high 13% | 2,704 |
| last | a Wizard is winning | can't follow | **duck low** 87%, Jester 11% | 2,262 |
| last | a Wizard is winning | no suit to follow | **duck low** 90%, Jester 8% | 2,059 |
| last | trump is winning | can follow | **win cheaply** 70%, win big 13%, duck high 13% | 1,746 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 72%, win big (trump in) 18%, duck high 6%, duck low 3% | 1,097 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 64%, lead trump low 20%, Jester 14% | 22,397 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 68%, duck low 27%, Jester 5% | 10,799 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 71%, duck low 27% | 8,991 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 77%, duck low 19%, Jester 4% | 7,071 |
| middle | off-suit is winning | can follow | **duck high** 34%, Jester 33%, win cheaply 25%, duck low 7% | 6,394 |
| middle | a Wizard is winning | no suit to follow | **duck high** 75%, duck low 22% | 5,295 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 78%, duck low 19% | 2,367 |
| middle | a Wizard is winning | can't follow | **duck high** 79%, duck low 17% | 2,147 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 55%, Jester 25%, win big 20% | 1,758 |
| middle | off-suit is winning | can't follow | **duck high** 63%, duck low 22%, Jester 10% | 1,717 |
| middle | a Wizard is winning | can follow | **duck high** 82%, duck low 16% | 1,107 |
| middle | trump is winning | can follow | **Jester** 51%, win cheaply 39%, duck high 9% | 384 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 73%, duck low 26% | 3,203 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 68%, duck low 28%, Jester 5% | 2,326 |
| last | a Wizard is winning | can't follow | **duck high** 74%, duck low 22% | 2,165 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 75%, duck low 22%, Jester 3% | 2,113 |
| last | a Wizard is winning | no suit to follow | **duck high** 76%, duck low 20% | 1,861 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 75%, duck low 22% | 1,463 |
| last | a Wizard is winning | can follow | **duck high** 83%, duck low 15% | 1,199 |
| last | off-suit is winning | can follow | **duck high** 42%, Jester 30%, win cheaply 13%, duck low 10%, win big 5% | 893 |
| last | off-suit is winning | can't follow | **duck high** 64%, duck low 20%, Jester 14% | 325 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 81%, lead trump low 7%, lead off-suit high (K+) 5%, Jester 4% | 2,518 |
## 6 players

300,015 real choices (more than one legal card). Each row: a situation, and how often the bot makes each kind of play there (the most common first).


### You need every trick that's left

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **Wizard** 42%, lead trump high (J+) 23%, lead off-suit low 17%, lead off-suit high (K+) 14%, lead trump low 4% | 554 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 91%, Wizard 4%, win big (trump in) 4% | 971 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **Wizard** 89%, duck low 6%, duck high 4% | 733 |
| middle | trump is winning | can follow | **win cheaply** 51%, Wizard 39%, win big 9% | 318 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **Wizard** 91%, duck high 4%, duck low 3% | 308 |

### You still need tricks

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 51%, lead off-suit high (K+) 21%, lead trump low 13%, Wizard 10%, lead trump high (J+) 4% | 28,401 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 58%, duck low 28%, Wizard 11% | 24,309 |
| middle | off-suit is winning | can follow | **win cheaply** 53%, win big 23%, Wizard 12%, duck high 9% | 22,016 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 50%, duck high 28%, Wizard 19%, Jester 3% | 13,824 |
| middle | off-suit is winning | can't follow | **win cheaply (trump in)** 49%, win big (trump in) 28%, duck high 14%, Wizard 4%, duck low 4% | 11,903 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 40%, duck low 33%, Wizard 17%, Jester 10% | 9,959 |
| middle | trump is winning | can follow | **win cheaply** 49%, win big 15%, duck high 14%, Jester 13%, Wizard 7% | 8,849 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 41%, duck high 40%, Wizard 16%, Jester 4% | 7,796 |
| middle | a Wizard is winning | no suit to follow | **duck low** 91%, Jester 7% | 6,125 |
| middle | a Wizard is winning | can follow | **duck low** 73%, duck high 17%, Jester 10% | 5,417 |
| middle | a Wizard is winning | can't follow | **duck low** 91%, Jester 8% | 3,650 |
| middle | only Jesters so far | no suit to follow | **win big** 43%, win cheaply 36%, Wizard 18%, Jester 3% | 1,809 |
| middle | trump is winning | can't follow | **win cheaply (trump in)** 66%, win big (trump in) 17%, duck high 9%, duck low 6% | 1,521 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 48%, duck low 32%, Wizard 17% | 6,294 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 39%, duck low 31%, Wizard 24%, Jester 6% | 4,212 |
| last | a Wizard is winning | can follow | **duck low** 74%, duck high 16%, Jester 10% | 3,753 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck low** 51%, duck high 23%, Wizard 23%, Jester 3% | 2,662 |
| last | a Wizard is winning | can't follow | **duck low** 92%, Jester 7% | 2,369 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck low** 41%, duck high 31%, Wizard 25%, Jester 4% | 2,363 |
| last | off-suit is winning | can follow | **win cheaply** 64%, win big 13%, Wizard 11%, duck high 8% | 2,293 |
| last | off-suit is winning | can't follow | **win cheaply (trump in)** 50%, win big (trump in) 30%, duck high 14%, duck low 3% | 1,733 |
| last | a Wizard is winning | no suit to follow | **duck low** 92%, Jester 6% | 1,594 |
| last | trump is winning | can follow | **win cheaply** 66%, duck high 18%, win big 10% | 1,255 |
| last | trump is winning | can't follow | **win cheaply (trump in)** 72%, win big (trump in) 16%, duck high 7% | 763 |

### You've made your bid exactly (every trick from here costs you)

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 64%, lead trump low 21%, Jester 13% | 24,601 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 68%, duck low 27%, Jester 6% | 15,527 |
| middle | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 72%, duck low 26% | 13,381 |
| middle | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 78%, duck low 18%, Jester 5% | 11,131 |
| middle | off-suit is winning | can follow | **duck high** 35%, Jester 30%, win cheaply 27%, duck low 5% | 9,444 |
| middle | a Wizard is winning | no suit to follow | **duck high** 74%, duck low 23% | 7,161 |
| middle | a Wizard is winning | can't follow | **duck high** 78%, duck low 19% | 4,515 |
| middle | trump is winning | can follow, nothing but a Wizard wins | **duck high** 77%, duck low 20% | 4,070 |
| middle | a Wizard is winning | can follow | **duck high** 82%, duck low 15% | 2,777 |
| middle | off-suit is winning | can't follow | **duck high** 67%, duck low 19%, Jester 10%, win cheaply (trump in) 3% | 2,550 |
| middle | only Jesters so far | no suit to follow | **win cheaply** 59%, Jester 24%, win big 17% | 2,046 |
| middle | trump is winning | can follow | **Jester** 44%, win cheaply 41%, duck high 13% | 502 |
| last | trump is winning | can't follow, nothing but a Wizard wins | **duck high** 72%, duck low 25% | 3,572 |
| last | a Wizard is winning | can't follow | **duck high** 75%, duck low 22% | 2,974 |
| last | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 65%, duck low 30%, Jester 5% | 2,469 |
| last | off-suit is winning | can follow, nothing but a Wizard wins | **duck high** 75%, duck low 21%, Jester 4% | 2,283 |
| last | a Wizard is winning | can follow | **duck high** 83%, duck low 14% | 1,905 |
| last | a Wizard is winning | no suit to follow | **duck high** 77%, duck low 21% | 1,810 |
| last | trump is winning | can follow, nothing but a Wizard wins | **duck high** 75%, duck low 22%, Jester 3% | 1,773 |
| last | off-suit is winning | can follow | **duck high** 46%, Jester 28%, win cheaply 12%, duck low 10% | 776 |
| last | off-suit is winning | can't follow | **duck high** 67%, duck low 19%, Jester 12% | 352 |

### You're already over your bid

| Seat | Trick so far | Your cards | Bot plays (how often) | Seen |
| --- | --- | --- | --- | ---: |
| lead | — | — | **lead off-suit low** 82%, lead trump low 9%, lead off-suit high (K+) 4% | 2,591 |
| middle | off-suit is winning | can't follow, nothing but a Wizard wins | **duck high** 62%, duck low 38% | 306 |
