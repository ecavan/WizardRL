//! What the network sees, and the numbering of its actions.
//!
//! Everything is from the acting seat's point of view: other seats are numbered by how far
//! to its left they sit (1 = next to act after it), so the same situation looks the same
//! whichever chair it happens in. Only information a player at the table has goes in.

use crate::card::{cards, Card, Suit, DECK_SIZE};
use crate::round::{led_suit, trick_winner, Action, Phase, TrumpSource};
use crate::rules::MAX_PLAYERS;
use crate::view::View;

const SEATS: usize = MAX_PLAYERS as usize;
const MAX_SIZE: f32 = 20.0; // most cards in a round (3 players)

// Feature layout. Each block is [start, start + len).
pub const PHASE: usize = 0; // 3: pick trump, bid, play
pub const PLAYERS: usize = PHASE + 3; // 4: one-hot 3..6
pub const SIZE: usize = PLAYERS + 4; // 1: cards dealt / 20
pub const TRICKS_LEFT: usize = SIZE + 1; // 1: tricks still to play / 20
pub const FROM_DEALER: usize = TRICKS_LEFT + 1; // 6: my seat's distance left of the dealer (0 = dealer)
pub const TRUMP: usize = FROM_DEALER + SEATS; // 5: clubs, diamonds, hearts, spades, none
pub const TURNED: usize = TRUMP + 5; // 3: Wizard turned, Jester turned, nothing turned
pub const HAND: usize = TURNED + 3; // 60
pub const PLAYED: usize = HAND + DECK_SIZE; // 60: every card played this round so far
pub const TRICK: usize = PLAYED + DECK_SIZE; // 5 x 60: this trick's cards, by relative seat 1..5
pub const LED: usize = TRICK + (SEATS - 1) * DECK_SIZE; // 5: suit to follow, or none
pub const WINNING: usize = LED + 5; // 6: relative seat currently winning the trick
pub const BIDS: usize = WINNING + SEATS; // 6: bid / 20 by relative seat
pub const HAS_BID: usize = BIDS + SEATS; // 6
pub const WON: usize = HAS_BID + SEATS; // 6: tricks won / 20
pub const NEED: usize = WON + SEATS; // 6: (bid - won) / 20, for seats that have bid
pub const VOIDS: usize = NEED + SEATS; // 24: known out-of-suit, relative seat x suit
pub const TRICK_POS: usize = VOIDS + SEATS * 4; // 1: cards already in this trick / 5
pub const GAME: usize = TRICK_POS + 1; // 11: the game so far (zeros for a lone round), see `observe`
pub const FEATURES: usize = GAME + 11;
/// Features of networks trained before the game features existed; the game features were
/// appended, so such a network reads the first `ROUND_FEATURES` numbers and nothing changes.
pub const ROUND_FEATURES: usize = GAME;

// Action numbering.
pub const ACT_TRUMP: usize = 0; // 4 suits
pub const ACT_BID: usize = 4; // bids 0..=20
pub const ACT_CARD: usize = ACT_BID + 21; // 60 cards
pub const ACTIONS: usize = ACT_CARD + DECK_SIZE;

pub fn action_index(a: Action) -> usize {
    match a {
        Action::PickTrump(s) => ACT_TRUMP + s.index() as usize,
        Action::Bid(b) => ACT_BID + b as usize,
        Action::Play(c) => ACT_CARD + c as usize,
    }
}

pub fn action_from_index(i: usize) -> Option<Action> {
    if i < ACT_BID {
        Some(Action::PickTrump(Suit::from_index(i as u8)))
    } else if i < ACT_CARD {
        Some(Action::Bid((i - ACT_BID) as u8))
    } else if i < ACTIONS {
        Some(Action::Play((i - ACT_CARD) as Card))
    } else {
        None
    }
}

/// Write the acting seat's observation into `out` (length `FEATURES`, overwritten).
pub fn observe(v: &View, out: &mut [f32]) {
    assert_eq!(out.len(), FEATURES);
    out.fill(0.0);
    let n = v.players() as usize;
    let me = v.seat() as usize;
    let rel = |s: u8| (s as usize + n - me) % n;

    match v.phase() {
        Phase::PickTrump { .. } => out[PHASE] = 1.0,
        Phase::Bid { .. } => out[PHASE + 1] = 1.0,
        Phase::Play { .. } => out[PHASE + 2] = 1.0,
        Phase::Done => {}
    }
    out[PLAYERS + n - 3] = 1.0;
    out[SIZE] = v.size() as f32 / MAX_SIZE;
    out[TRICKS_LEFT] = v.tricks_left() as f32 / MAX_SIZE;
    out[FROM_DEALER + (me + n - v.dealer() as usize) % n] = 1.0;
    match v.trump() {
        Some(s) => out[TRUMP + s.index() as usize] = 1.0,
        None => out[TRUMP + 4] = 1.0,
    }
    match v.trump_source() {
        TrumpSource::WizardTurned(_) => out[TURNED] = 1.0,
        TrumpSource::JesterTurned(_) => out[TURNED + 1] = 1.0,
        TrumpSource::NoCardTurned => out[TURNED + 2] = 1.0,
        TrumpSource::TurnedCard(_) => {}
    }
    for c in cards(v.hand()) {
        out[HAND + c as usize] = 1.0;
    }
    for c in cards(v.played()) {
        out[PLAYED + c as usize] = 1.0;
    }
    let trick = v.current_trick();
    for &(s, c) in trick {
        let r = rel(s);
        debug_assert!(r >= 1, "the acting seat hasn't played to this trick yet");
        out[TRICK + (r - 1) * DECK_SIZE + c as usize] = 1.0;
    }
    match led_suit(trick) {
        Some(s) => out[LED + s.index() as usize] = 1.0,
        None => out[LED + 4] = 1.0,
    }
    if !trick.is_empty() {
        out[WINNING + rel(trick[trick_winner(trick, v.trump())].0)] = 1.0;
    }
    let bids = v.bids();
    for s in 0..n as u8 {
        let r = rel(s);
        let won = v.tricks_won(s);
        out[WON + r] = won as f32 / MAX_SIZE;
        if let Some(b) = bids[s as usize] {
            out[BIDS + r] = b as f32 / MAX_SIZE;
            out[HAS_BID + r] = 1.0;
            out[NEED + r] = (b as f32 - won as f32) / MAX_SIZE;
        }
        let voids = v.known_voids(s);
        for suit in 0..4 {
            if voids & (1 << suit) != 0 {
                out[VOIDS + r * 4 + suit] = 1.0;
            }
        }
    }
    out[TRICK_POS] = trick.len() as f32 / (SEATS - 1) as f32;

    // The game so far (only inside a full game): scores before this round, by relative seat;
    // my margin over the best other player; rounds still to come after this one; my rank.
    let scores = v.scores();
    if scores.len() == n {
        out[GAME] = 1.0;
        for s in 0..n as u8 {
            out[GAME + 1 + rel(s)] = scores[s as usize] as f32 / SCORE_SCALE;
        }
        let mine = scores[me];
        let best_other = (0..n)
            .filter(|&s| s != me)
            .map(|s| scores[s])
            .max()
            .unwrap_or(mine);
        out[GAME + 7] = (mine - best_other) as f32 / SCORE_SCALE;
        let rounds = (DECK_SIZE / n) as f32;
        out[GAME + 8] = (rounds - v.size() as f32) / MAX_SIZE;
        out[GAME + 9] = (mine > best_other) as u8 as f32;
        let ahead = (0..n).filter(|&s| s != me && scores[s] > mine).count();
        out[GAME + 10] = ahead as f32 / (n - 1) as f32;
    }
}

/// Game scores are divided by this in the observation.
const SCORE_SCALE: f32 = 200.0;

/// Legal actions as a mask of length `ACTIONS`.
pub fn legal_mask(v: &View, out: &mut [bool]) {
    assert_eq!(out.len(), ACTIONS);
    out.fill(false);
    for a in v.legal_actions() {
        out[action_index(a)] = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::parse;
    use crate::rng::Rng;
    use crate::round::Round;
    use crate::rules::Rules;

    #[test]
    fn layout_is_contiguous() {
        assert_eq!(
            ROUND_FEATURES, 503,
            "networks from before the game features read 503"
        );
        assert_eq!(FEATURES, 514);
        assert_eq!(ACTIONS, 85);
        for i in 0..ACTIONS {
            assert_eq!(action_index(action_from_index(i).unwrap()), i);
        }
        assert!(action_from_index(ACTIONS).is_none());
    }

    #[test]
    fn game_features_show_the_score_race() {
        let mut rng = Rng::new(5);
        let r = Round::deal(Rules::simultaneous(4), 3, 0, &mut rng);
        let mut o = vec![0.0; FEATURES];
        // Lone round: no game features.
        observe(&View::new(&r, 1, &[]), &mut o);
        assert!(o[GAME..].iter().all(|&x| x == 0.0));
        // Seat 1 has 60, the leader (seat 3) 100: 40 behind, one player ahead, 12 rounds to come.
        observe(&View::new(&r, 1, &[20, 60, -10, 100]), &mut o);
        assert_eq!(o[GAME], 1.0);
        assert_eq!(o[GAME + 1], 60.0 / 200.0); // me
        assert_eq!(o[GAME + 1 + 2], 100.0 / 200.0); // seat 3 is two to my left
        assert_eq!(o[GAME + 7], -40.0 / 200.0);
        assert_eq!(o[GAME + 8], 12.0 / 20.0);
        assert_eq!(o[GAME + 9], 0.0);
        assert_eq!(o[GAME + 10], 1.0 / 3.0);
    }

    #[test]
    fn a_seat_sees_its_hand_and_the_table_but_not_other_hands() {
        let c = |s: &str| parse(s).unwrap();
        let hands = vec![
            vec![c("wiz1"), c("As")],
            vec![c("Kh"), c("3s")],
            vec![c("Ah"), c("Qs")],
        ];
        let mut r = Round::from_hands(Rules::official(3), 2, 2, &hands, Some(c("5h")));
        // Bids: seat 0 bids 2, seat 1 bids 0, seat 2 bids 1. Seat 0 leads As; seat 1 to act.
        for b in [2, 0, 1] {
            r.apply(Action::Bid(b)).unwrap();
        }
        r.apply(Action::Play(c("As"))).unwrap();
        let v = View::new(&r, 1, &[]);
        let mut o = vec![0.0; FEATURES];
        observe(&v, &mut o);
        assert_eq!(o[PHASE + 2], 1.0);
        assert_eq!(o[PLAYERS], 1.0); // 3 players
        assert_eq!(o[TRUMP + Suit::Hearts.index() as usize], 1.0);
        // Own hand only.
        let hand: Vec<usize> = (0..DECK_SIZE).filter(|&i| o[HAND + i] == 1.0).collect();
        assert_eq!(hand, vec![c("Kh") as usize, c("3s") as usize]);
        // Seat 0 is two to seat 1's left (relative seat 2), and led the ace of spades.
        assert_eq!(o[TRICK + DECK_SIZE + c("As") as usize], 1.0);
        assert_eq!(o[LED + Suit::Spades.index() as usize], 1.0);
        assert_eq!(o[WINNING + 2], 1.0);
        // Bids by relative seat: me 0, left neighbour (seat 2) 1, seat 0 bid 2.
        assert_eq!(o[BIDS], 0.0);
        assert_eq!(o[BIDS + 1], 1.0 / 20.0);
        assert_eq!(o[BIDS + 2], 2.0 / 20.0);
        assert_eq!(o[HAS_BID] + o[HAS_BID + 1] + o[HAS_BID + 2], 3.0);
        // Seat 1 sits two to the left of the dealer (seat 2 -> 0 -> 1).
        assert_eq!(o[FROM_DEALER + 2], 1.0);
        // Legal: must follow spades with 3s.
        let mut m = vec![false; ACTIONS];
        legal_mask(&v, &mut m);
        let legal: Vec<usize> = (0..ACTIONS).filter(|&i| m[i]).collect();
        assert_eq!(legal, vec![ACT_CARD + c("3s") as usize]);
    }

    #[test]
    fn every_observation_is_finite_and_in_range() {
        let mut rng = Rng::new(4);
        let mut o = vec![0.0; FEATURES];
        for n in 3..=6 {
            let rules = Rules::official(n);
            for size in 1..=rules.rounds() {
                let mut r = Round::deal(rules, size, 0, &mut rng);
                while let Some(s) = r.to_act() {
                    let v = View::new(&r, s, &[]);
                    observe(&v, &mut o);
                    assert!(o.iter().all(|x| x.is_finite() && (-1.0..=1.0).contains(x)));
                    let legal = r.legal_actions();
                    let a = legal[rng.below(legal.len() as u64) as usize];
                    r.apply(a).unwrap();
                }
            }
        }
    }
}
