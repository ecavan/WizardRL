//! What one seat is allowed to see. Bots only ever get a `View`, so they can't peek at other
//! hands by accident.

use crate::card::{Card, CardSet, Suit};
use crate::round::{Action, Phase, Round, Trick, TrumpSource};

#[derive(Clone, Copy)]
pub struct View<'a> {
    round: &'a Round,
    seat: u8,
    /// Game scores before this round, by seat (empty outside a full game).
    scores: &'a [i32],
}

impl<'a> View<'a> {
    pub fn new(round: &'a Round, seat: u8, scores: &'a [i32]) -> View<'a> {
        assert!(seat < round.players());
        View {
            round,
            seat,
            scores,
        }
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
