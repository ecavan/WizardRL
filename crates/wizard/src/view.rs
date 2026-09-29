//! What one seat is allowed to see. Bots only ever get a `View`, so they can't peek at other
//! hands by accident.

use crate::card::{Card, CardSet, Suit};
use crate::round::{Action, Phase, Round, Trick, TrumpSource};

/// How a player has played so far this game (everyone can see this at the table): how far their
/// bids were off, how often they made them, and whether they throw Wizards out early.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SeatHistory {
    /// Rounds finished this game.
    pub rounds: u16,
    /// Sum of (bid - tricks won): positive for an overbidder.
    pub over: i32,
    pub made: u16,
    /// Wizards played, and how many of those went on the round's first trick.
    pub wizards: u16,
    pub early_wizards: u16,
}

impl SeatHistory {
    /// Add seat `seat`'s finished round.
    pub fn record(&mut self, round: &Round, seat: u8) {
        let bid = round.bid(seat).expect("everyone bid") as i32;
        let won = round.tricks_won(seat) as i32;
        self.rounds += 1;
        self.over += bid - won;
        self.made += (bid == won) as u16;
        for (k, t) in round.completed_tricks().iter().enumerate() {
            for &(s, c) in &t.plays {
                if s == seat && crate::card::is_wizard(c) {
                    self.wizards += 1;
                    self.early_wizards += (k == 0) as u16;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct View<'a> {
    round: &'a Round,
    seat: u8,
    /// Game scores before this round, by seat (empty outside a full game).
    scores: &'a [i32],
    /// Each seat's play so far this game (empty outside a full game).
    history: &'a [SeatHistory],
}

impl<'a> View<'a> {
    pub fn new(round: &'a Round, seat: u8, scores: &'a [i32]) -> View<'a> {
        View::with_history(round, seat, scores, &[])
    }

    pub fn with_history(
        round: &'a Round,
        seat: u8,
        scores: &'a [i32],
        history: &'a [SeatHistory],
    ) -> View<'a> {
        assert!(seat < round.players());
        View {
            round,
            seat,
            scores,
            history,
        }
    }

    pub fn history(&self) -> &'a [SeatHistory] {
        self.history
    }

    pub fn rules(&self) -> &'a crate::rules::Rules {
        self.round.rules()
    }
    pub fn seat(&self) -> u8 {
        self.seat
    }
    pub fn players(&self) -> u8 {
        self.round.players()
    }
    pub fn size(&self) -> u8 {
        self.round.size()
    }
    pub fn dealer(&self) -> u8 {
        self.round.dealer()
    }
    pub fn phase(&self) -> Phase {
        self.round.phase()
    }
    pub fn trump(&self) -> Option<Suit> {
        self.round.trump()
    }
    pub fn trump_source(&self) -> TrumpSource {
        self.round.trump_source()
    }
    pub fn hand(&self) -> CardSet {
        self.round.hand(self.seat)
    }
    /// Bids this seat can see. With simultaneous bidding, other players' bids stay hidden
    /// until everyone has bid.
    pub fn bids(&self) -> Vec<Option<u8>> {
        let hidden = self.round.rules().simultaneous_bids
            && matches!(
                self.round.phase(),
                Phase::Bid { .. } | Phase::PickTrump { .. }
            );
        self.round
            .bids()
            .iter()
            .enumerate()
            .map(|(s, &b)| {
                if hidden && s as u8 != self.seat {
                    None
                } else {
                    b
                }
            })
            .collect()
    }
    pub fn my_bid(&self) -> Option<u8> {
        self.round.bid(self.seat)
    }
    pub fn tricks_won(&self, seat: u8) -> u8 {
        self.round.tricks_won(seat)
    }
    pub fn current_trick(&self) -> &'a [(u8, Card)] {
        self.round.current_trick()
    }
    pub fn completed_tricks(&self) -> &'a [Trick] {
        self.round.completed_tricks()
    }
    pub fn played(&self) -> CardSet {
        self.round.played()
    }
    pub fn known_voids(&self, seat: u8) -> u8 {
        self.round.known_voids(seat)
    }
    pub fn scores(&self) -> &'a [i32] {
        self.scores
    }
    pub fn simultaneous_bids(&self) -> bool {
        self.round.rules().simultaneous_bids
    }
    pub fn is_my_turn(&self) -> bool {
        self.round.to_act() == Some(self.seat)
    }
    pub fn legal_actions(&self) -> Vec<Action> {
        if self.is_my_turn() {
            self.round.legal_actions()
        } else {
            Vec::new()
        }
    }
    pub fn legal_plays(&self) -> CardSet {
        if self.is_my_turn() {
            self.round.legal_plays()
        } else {
            0
        }
    }
    /// Tricks still to be played this round, counting the one in progress.
    pub fn tricks_left(&self) -> u8 {
        self.round.size() - self.round.completed_tricks().len() as u8
    }
}
