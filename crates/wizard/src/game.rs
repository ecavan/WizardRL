//! A full game: rounds of 1, 2, 3 … cards, the deal rotating left each round.

use crate::bots::Bot;
use crate::card::Suit;
use crate::rng::Rng;
use crate::round::{Action, Phase, Round};
use crate::rules::Rules;
use crate::view::View;

#[derive(Clone, Debug)]
pub struct RoundRecord {
    pub size: u8,
    pub dealer: u8,
    pub trump: Option<Suit>,
    pub bids: Vec<u8>,
    pub won: Vec<u8>,
    pub scores: Vec<i32>,
}

#[derive(Clone, Debug)]
pub struct GameResult {
    pub totals: Vec<i32>,
    pub rounds: Vec<RoundRecord>,
    /// Decisions taken (trump picks, bids, cards), for throughput numbers.
    pub decisions: u64,
}

/// Play one round to the end with the given bots (bot `i` sits in seat `i`).
pub fn play_round(
    round: &mut Round,
    bots: &mut [&mut dyn Bot],
    scores: &[i32],
    rng: &mut Rng,
) -> u64 {
    assert_eq!(bots.len(), round.players() as usize);
    let mut decisions = 0;
    while let Some(seat) = round.to_act() {
        let a: Action = {
            let v = View::new(round, seat, scores);
            bots[seat as usize].act(&v, rng)
        };
        if let Err(e) = round.apply(a) {
            panic!(
                "seat {seat} ({}) made an illegal move: {e}",
                bots[seat as usize].name()
            );
        }
        decisions += 1;
    }
    debug_assert_eq!(round.phase(), Phase::Done);
    decisions
}

/// Play a full game. The first dealer is drawn from `rng`; the deal moves one seat left each round.
pub fn play_game(rules: Rules, bots: &mut [&mut dyn Bot], rng: &mut Rng) -> GameResult {
    rules.validate().expect("valid rules");
    let n = rules.players;
    let mut totals = vec![0i32; n as usize];
    let mut rounds = Vec::with_capacity(rules.rounds() as usize);
    let mut decisions = 0;
    let mut dealer = rng.below(n as u64) as u8;
    for size in 1..=rules.rounds() {
        let mut round = Round::deal(rules, size, dealer, rng);
        decisions += play_round(&mut round, bots, &totals, rng);
        let scores = round.scores().expect("round finished");
        for (t, s) in totals.iter_mut().zip(&scores) {
            *t += s;
        }
        rounds.push(RoundRecord {
            size,
            dealer,
            trump: round.trump(),
            bids: round
                .bids()
                .iter()
                .map(|b| b.expect("everyone bid"))
                .collect(),
            won: (0..n).map(|i| round.tricks_won(i)).collect(),
            scores,
        });
        dealer = (dealer + 1) % n;
    }
    GameResult {
        totals,
        rounds,
        decisions,
    }
}
