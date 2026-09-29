//! Every rule, checked by hand-worked examples and by fuzzing thousands of random rounds.

use wizard::bots::{Bot, CountingBot, RandomBot};
use wizard::card::{self, bit, cards, parse, Card, Suit, JESTER_BASE, WIZARD_BASE};
use wizard::game::{play_game, play_round};
use wizard::rng::Rng;
use wizard::round::{
    led_suit, legal_cards, round_score, trick_winner, Action, Event, Phase, Round, TrumpSource,
};
use wizard::rules::Rules;
use wizard::view::View;

fn c(s: &str) -> Card {
    parse(s).unwrap_or_else(|| panic!("bad card {s}"))
}
fn hand(s: &str) -> Vec<Card> {
    s.split_whitespace().map(c).collect()
}
fn plays(s: &str) -> Vec<(u8, Card)> {
    s.split_whitespace()
        .enumerate()
        .map(|(i, x)| (i as u8, c(x)))
        .collect()
}

// ------------------------------------------------------------------------------ trick winner

#[test]
fn first_wizard_wins() {
    assert_eq!(
        trick_winner(&plays("As wiz1 wiz2 Ks"), Some(Suit::Spades)),
        1
    );
    assert_eq!(trick_winner(&plays("wiz3 wiz1 As 2c"), None), 0);
    assert_eq!(trick_winner(&plays("jes1 5h wiz4"), Some(Suit::Hearts)), 2);
}

#[test]
fn highest_trump_beats_the_suit_led() {
    assert_eq!(trick_winner(&plays("As 2h Ks"), Some(Suit::Hearts)), 1);
    assert_eq!(trick_winner(&plays("As 2h 3h Ks"), Some(Suit::Hearts)), 2);
    // Trump led: highest trump.
    assert_eq!(trick_winner(&plays("5h Jh 10h"), Some(Suit::Hearts)), 1);
}

#[test]
fn off_suit_cards_never_win() {
    // Ace of clubs thrown on a spade lead with diamonds trump: the spade wins.
    assert_eq!(trick_winner(&plays("3s Ac 2s"), Some(Suit::Diamonds)), 0);
    assert_eq!(trick_winner(&plays("3s Ac Ks"), None), 2);
}

#[test]
fn jesters() {
    // All Jesters: the first one wins.
    assert_eq!(trick_winner(&plays("jes2 jes1 jes3"), Some(Suit::Clubs)), 0);
    // Jester led: the first standard card sets the suit.
    assert_eq!(trick_winner(&plays("jes1 7d Ad 9s"), Some(Suit::Hearts)), 2);
    assert_eq!(trick_winner(&plays("jes1 jes2 7d 9s"), None), 2);
    // Jesters never win against a standard card.
    assert_eq!(trick_winner(&plays("2c jes1"), None), 0);
}

#[test]
fn suit_to_follow() {
    assert_eq!(led_suit(&plays("")), None);
    assert_eq!(led_suit(&plays("Kh 2s")), Some(Suit::Hearts));
    assert_eq!(
        led_suit(&plays("wiz1 Kh")),
        None,
        "Wizard led: nothing to follow"
    );
    assert_eq!(
        led_suit(&plays("jes1 Kh 2s")),
        Some(Suit::Hearts),
        "Jester led: next card sets it"
    );
    assert_eq!(led_suit(&plays("jes1 jes2")), None);
    assert_eq!(
        led_suit(&plays("jes1 wiz2 Kh")),
        None,
        "Jester then Wizard: Wizard rules"
    );
}

#[test]
fn following_suit() {
    let h = hand("Kh 2h As wiz1 jes1")
        .iter()
        .fold(0, |m, &x| m | bit(x));
    let legal = |t: &str| {
        let mut v: Vec<String> = cards(legal_cards(h, &plays(t))).map(card::name).collect();
        v.sort();
        v.join(" ")
    };
    // Hearts led: hearts, or a Wizard/Jester any time.
    assert_eq!(legal("5h"), "2♥ Jes K♥ Wiz");
    // Clubs led and none held: anything.
    assert_eq!(legal("5c"), "2♥ A♠ Jes K♥ Wiz");
    // Wizard led: anything.
    assert_eq!(legal("wiz2"), "2♥ A♠ Jes K♥ Wiz");
    // Jester led, nothing else yet: anything.
    assert_eq!(legal("jes2"), "2♥ A♠ Jes K♥ Wiz");
    // Jester then a spade: follow spades.
    assert_eq!(legal("jes2 3s"), "A♠ Jes Wiz");
    // Leading: anything.
    assert_eq!(legal(""), "2♥ A♠ Jes K♥ Wiz");
}

#[test]
fn scoring() {
    assert_eq!(round_score(0, 0), 20);
    assert_eq!(round_score(2, 2), 40);
    assert_eq!(round_score(5, 5), 70);
    assert_eq!(round_score(2, 3), -10);
    assert_eq!(round_score(3, 0), -30);
    assert_eq!(round_score(0, 4), -40);
}

// ------------------------------------------------------------------------------ a full round, by hand

/// Four players, three cards each, seat 3 deals so seat 0 bids and leads first. Hearts trump.
#[test]
fn hand_worked_round() {
    let rules = Rules::official(4);
    let hands = vec![
        hand("wiz1 As 2c"),
        hand("Kh jes1 3s"),
        hand("Ah Qs 4d"),
        hand("7h 9c jes2"),
    ];
    let mut r = Round::from_hands(rules, 3, 3, &hands, Some(c("5h")));
    assert_eq!(r.trump(), Some(Suit::Hearts));
    assert_eq!(r.trump_source(), TrumpSource::TurnedCard(c("5h")));
    assert_eq!(r.phase(), Phase::Bid { seat: 0 });
    assert!(
        r.apply(Action::Bid(4)).is_err(),
        "can't bid more than the cards dealt"
    );
    for b in [2, 1, 1, 0] {
        r.apply(Action::Bid(b)).unwrap();
    }
    assert_eq!(r.phase(), Phase::Play { seat: 0 });

    // Trick 1: As led; seat 1 must follow with 3s (Kh refused; the Jester is allowed).
    r.apply(Action::Play(c("As"))).unwrap();
    assert!(
        r.apply(Action::Play(c("Kh"))).is_err(),
        "must follow spades"
    );
    assert!(r.is_legal(Action::Play(c("jes1"))));
    r.apply(Action::Play(c("3s"))).unwrap();
    r.apply(Action::Play(c("Qs"))).unwrap();
    assert_eq!(
        r.apply(Action::Play(c("7h"))).unwrap(),
        Event::TrickWon { winner: 3 },
        "seat 3 trumps in"
    );
    assert_eq!(r.known_voids(3), 1 << Suit::Spades.index());

    // Trick 2: seat 3 leads 9c; seat 0 plays the Wizard although it holds 2c.
    assert_eq!(r.phase(), Phase::Play { seat: 3 });
    r.apply(Action::Play(c("9c"))).unwrap();
    r.apply(Action::Play(c("wiz1"))).unwrap();
    r.apply(Action::Play(c("Kh"))).unwrap();
    assert_eq!(
        r.apply(Action::Play(c("Ah"))).unwrap(),
        Event::TrickWon { winner: 0 },
        "the Wizard beats the ace of trump"
    );
    assert_eq!(r.known_voids(0), 0, "a Wizard doesn't reveal a void");
    assert_eq!(r.known_voids(1), 1 << Suit::Clubs.index());

    // Trick 3: 2c led, two Jesters and an off-suit card: the deuce wins.
    r.apply(Action::Play(c("2c"))).unwrap();
    r.apply(Action::Play(c("jes1"))).unwrap();
    r.apply(Action::Play(c("4d"))).unwrap();
    assert_eq!(r.apply(Action::Play(c("jes2"))).unwrap(), Event::RoundOver);
    assert!(r.is_done());
    assert_eq!(
        (0..4).map(|s| r.tricks_won(s)).collect::<Vec<_>>(),
        vec![2, 0, 0, 1]
    );
    assert_eq!(r.scores().unwrap(), vec![40, -10, -10, -10]);
    assert!(r.apply(Action::Bid(0)).is_err());
    r.check_invariants().unwrap();
}

#[test]
fn bids_may_add_up_to_anything() {
    let rules = Rules::official(3);
    let hands = vec![hand("As"), hand("Ks"), hand("Qs")];
    let mut r = Round::from_hands(rules, 1, 0, &hands, Some(c("2c")));
    for _ in 0..3 {
        r.apply(Action::Bid(1)).unwrap();
    }
    assert!(matches!(r.phase(), Phase::Play { .. }));
}

#[test]
fn wrong_kind_of_action_is_refused() {
    let rules = Rules::official(3);
    let hands = vec![hand("As"), hand("Ks"), hand("Qs")];
    let mut r = Round::from_hands(rules, 1, 0, &hands, Some(c("2c")));
    assert!(r.apply(Action::Play(c("Ks"))).is_err());
    assert!(r.apply(Action::PickTrump(Suit::Clubs)).is_err());
    r.apply(Action::Bid(0)).unwrap();
    r.apply(Action::Bid(0)).unwrap();
    r.apply(Action::Bid(0)).unwrap();
    assert!(r.apply(Action::Bid(0)).is_err());
    // Seat 1 leads (left of dealer 0); seat 0 can't play out of turn.
    assert_eq!(r.phase(), Phase::Play { seat: 1 });
    assert!(
        r.apply(Action::Play(c("As"))).is_err(),
        "not seat 0's card to play now"
    );
}

// ------------------------------------------------------------------------------ trump

fn one_card_round(rules: Rules, turned: Card) -> Round {
    let hands: Vec<Vec<Card>> = (0..rules.players)
        .map(|i| vec![card::card(Suit::Clubs, i)])
        .collect();
    Round::from_hands(rules, 1, 1, &hands, Some(turned))
}

#[test]
fn official_trump_rules() {
    let rules = Rules::official(4);
    // Wizard turned: the dealer (seat 1) names trump, then bidding starts left of the dealer.
    let mut r = one_card_round(rules, WIZARD_BASE);
    assert_eq!(r.phase(), Phase::PickTrump { seat: 1 });
    assert_eq!(r.trump(), None);
    r.apply(Action::PickTrump(Suit::Spades)).unwrap();
    assert_eq!(r.trump(), Some(Suit::Spades));
    assert_eq!(r.phase(), Phase::Bid { seat: 2 });
    // Jester turned: no trump.
    let r = one_card_round(rules, JESTER_BASE);
    assert_eq!(r.trump(), None);
    assert_eq!(r.phase(), Phase::Bid { seat: 2 });
    assert_eq!(r.trump_source(), TrumpSource::JesterTurned(JESTER_BASE));
}

#[test]
fn last_round_has_no_trump() {
    // The last round deals the whole deck, so nothing is turned up.
    for n in 3..=6 {
        let rules = Rules::official(n);
        let r = Round::deal(rules, rules.rounds(), 0, &mut Rng::new(9));
        assert_eq!(r.trump_source(), TrumpSource::NoCardTurned);
        assert_eq!(r.trump(), None);
        assert!(matches!(r.phase(), Phase::Bid { .. }));
        // Every other round turns a card up.
        let r = Round::deal(rules, rules.rounds() - 1, 0, &mut Rng::new(9));
        assert_ne!(r.trump_source(), TrumpSource::NoCardTurned);
    }
}

// ------------------------------------------------------------------------------ fuzzing

fn all_rule_sets() -> Vec<Rules> {
    (3..=6).map(Rules::official).collect()
}

/// Random rounds of every size and table: invariants after every move, illegal moves refused
/// without changing anything, and the score matches the formula.
#[test]
fn fuzz_rounds() {
    let mut rng = Rng::new(2026);
    let mut rounds = 0;
    for rules in all_rule_sets() {
        for size in 1..=rules.rounds() {
            for _ in 0..150 {
                let dealer = rng.below(rules.players as u64) as u8;
                let mut r = Round::deal(rules, size, dealer, &mut rng);
                r.check_invariants().unwrap();
                // Everyone got `size` cards; no card twice; the turned card is not in a hand.
                let mut all = 0u64;
                for s in 0..rules.players {
                    assert_eq!(r.hand(s).count_ones(), size as u32);
                    assert_eq!(all & r.hand(s), 0);
                    all |= r.hand(s);
                }
                match r.trump_source() {
                    TrumpSource::TurnedCard(t)
                    | TrumpSource::WizardTurned(t)
                    | TrumpSource::JesterTurned(t) => assert_eq!(all & bit(t), 0),
                    TrumpSource::NoCardTurned => {}
                }
                while let Some(seat) = r.to_act() {
                    // An illegal card is refused and changes nothing.
                    if let Phase::Play { .. } = r.phase() {
                        let illegal =
                            cards(!r.legal_plays() & card::ALL_CARDS).nth(rng.below(10) as usize);
                        if let Some(x) = illegal {
                            let before = format!("{:?}", r);
                            assert!(r.apply(Action::Play(x)).is_err());
                            assert_eq!(before, format!("{:?}", r));
                        }
                        // Following suit: a legal standard card is of the suit led whenever the
                        // hand holds that suit.
                        if let Some(s) = led_suit(r.current_trick()) {
                            if r.hand(seat) & s.mask() != 0 {
                                for x in cards(r.legal_plays()) {
                                    assert!(card::suit_of(x).is_none_or(|xs| xs == s));
                                }
                            }
                        }
                    }
                    let legal = r.legal_actions();
                    assert!(!legal.is_empty());
                    let a = legal[rng.below(legal.len() as u64) as usize];
                    r.apply(a).unwrap();
                    r.check_invariants().unwrap();
                    // Known voids are true.
                    for s in 0..rules.players {
                        for suit in Suit::ALL {
                            if r.known_voids(s) & (1 << suit.index()) != 0 {
                                assert_eq!(r.hand(s) & suit.mask(), 0);
                            }
                        }
                    }
                }
                let won: u32 = (0..rules.players).map(|s| r.tricks_won(s) as u32).sum();
                assert_eq!(won, size as u32);
                let scores = r.scores().unwrap();
                for s in 0..rules.players {
                    assert_eq!(
                        scores[s as usize],
                        round_score(r.bid(s).unwrap(), r.tricks_won(s))
                    );
                }
                assert_eq!(r.played().count_ones(), size as u32 * rules.players as u32);
                rounds += 1;
            }
        }
    }
    assert!(rounds > 5_000);
}

#[test]
fn full_games_are_reproducible() {
    for rules in all_rule_sets() {
        let run = |seed| {
            let mut a = CountingBot;
            let mut b = RandomBot;
            let mut bots: Vec<&mut dyn Bot> = Vec::new();
            // Can't hold two &mut to one bot, so alternate two types across seats.
            let mut extra: Vec<Box<dyn Bot>> = (2..rules.players)
                .map(|i| {
                    if i % 2 == 0 {
                        Box::new(CountingBot) as Box<dyn Bot>
                    } else {
                        Box::new(RandomBot)
                    }
                })
                .collect();
            bots.push(&mut a);
            bots.push(&mut b);
            for e in extra.iter_mut() {
                bots.push(e.as_mut());
            }
            play_game(rules, &mut bots, &mut Rng::new(seed))
        };
        let x = run(5);
        let y = run(5);
        assert_eq!(x.totals, y.totals);
        assert_eq!(x.rounds.len(), rules.rounds() as usize);
        for (i, rec) in x.rounds.iter().enumerate() {
            assert_eq!(rec.size as usize, i + 1);
            assert_eq!(
                rec.won.iter().map(|&w| w as u32).sum::<u32>(),
                rec.size as u32
            );
        }
        // The deal rotates left each round.
        for w in x.rounds.windows(2) {
            assert_eq!(w[1].dealer, (w[0].dealer + 1) % rules.players);
        }
        let totals: Vec<i32> = (0..rules.players as usize)
            .map(|s| x.rounds.iter().map(|r| r.scores[s]).sum())
            .collect();
        assert_eq!(totals, x.totals);
    }
}

/// Bots only see their own hand: a bot that tries to act out of turn gets no legal actions.
#[test]
fn views_hide_other_hands() {
    let mut rng = Rng::new(11);
    let r = Round::deal(Rules::official(4), 5, 0, &mut rng);
    let to_act = r.to_act().unwrap();
    for s in 0..4 {
        let v = View::new(&r, s, &[]);
        assert_eq!(v.hand(), r.hand(s));
        assert_eq!(v.legal_actions().is_empty(), s != to_act);
    }
}

#[test]
fn counting_bot_beats_random() {
    let mut rng = Rng::new(1);
    let mut diff = 0i64;
    for _ in 0..300 {
        let mut round = Round::deal(
            Rules::official(4),
            1 + rng.below(15) as u8,
            rng.below(4) as u8,
            &mut rng,
        );
        let (mut c0, mut r1, mut r2, mut r3) = (CountingBot, RandomBot, RandomBot, RandomBot);
        let mut bots: Vec<&mut dyn Bot> = vec![&mut c0, &mut r1, &mut r2, &mut r3];
        play_round(&mut round, &mut bots, &[], &mut rng);
        let s = round.scores().unwrap();
        diff += (s[0] - (s[1] + s[2] + s[3]) / 3) as i64;
    }
    assert!(
        diff > 0,
        "counting bot should beat random players, diff {diff}"
    );
}

// ------------------------------------------------------------------------------ simultaneous bids

/// Everyone bids at once: while bidding, no seat sees another seat's bid (the engine collects
/// them one by one, but that order is invisible). Once all are in, everyone sees them all.
#[test]
fn simultaneous_bids_are_hidden_until_everyone_has_bid() {
    use wizard::encode::{self, FEATURES};
    let rules = Rules::simultaneous(4);
    let hands = vec![hand("As Kd"), hand("Ks Qd"), hand("Qs Jd"), hand("Js 10d")];
    let mut r = Round::from_hands(rules, 2, 3, &hands, Some(c("2h")));
    r.apply(Action::Bid(2)).unwrap(); // seat 0
    r.apply(Action::Bid(1)).unwrap(); // seat 1
                                      // Seat 2 is bidding: it sees no bids at all (not even how many are in).
    let v = View::new(&r, 2, &[]);
    assert!(v.bids().iter().all(|b| b.is_none()));
    let mut o = vec![0.0; FEATURES];
    encode::observe(&v, &mut o);
    assert!((0..4).all(|k| o[encode::HAS_BID + k] == 0.0 && o[encode::BIDS + k] == 0.0));
    // A seat that has bid sees only its own bid while others are still bidding.
    let v0 = View::new(&r, 0, &[]);
    assert_eq!(v0.bids(), vec![Some(2), None, None, None]);
    r.apply(Action::Bid(0)).unwrap();
    r.apply(Action::Bid(0)).unwrap();
    // Everyone has bid: all bids are visible to everyone.
    for s in 0..4 {
        assert_eq!(
            View::new(&r, s, &[]).bids(),
            vec![Some(2), Some(1), Some(0), Some(0)]
        );
    }
    // Bidding in turn (the printed rules), later bidders hear the earlier bids.
    let mut t = Round::from_hands(Rules::official(4), 2, 3, &hands, Some(c("2h")));
    t.apply(Action::Bid(2)).unwrap();
    assert_eq!(View::new(&t, 1, &[]).bids()[0], Some(2));
}
