//! A full game: rounds of 1, 2, 3 … cards, the deal rotating left each round.

use crate::bots::Bot;
use crate::card::Suit;
use crate::rng::Rng;
use crate::round::{Action, Phase, Round};
use crate::rules::Rules;
use crate::view::{SeatHistory, View};

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
    play_round_with(round, bots, scores, &[], rng)
}

/// `play_round` inside a game: the bots also see how everyone has played so far.
pub fn play_round_with(
    round: &mut Round,
    bots: &mut [&mut dyn Bot],
    scores: &[i32],
    history: &[SeatHistory],
    rng: &mut Rng,
) -> u64 {
    assert_eq!(bots.len(), round.players() as usize);
    let mut decisions = 0;
    while let Some(seat) = round.to_act() {
        let a: Action = {
            let v = View::with_history(round, seat, scores, history);
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
    run_game(rules, bots, rng, None)
}

/// Play a full game with the cards (and first dealer) drawn from `deals` and the bots' own
/// randomness from `rng`, so the same `deals` seed deals the same cards whatever the bots do:
/// the basis of duplicate games.
pub fn play_game_dealt(
    rules: Rules,
    bots: &mut [&mut dyn Bot],
    deals: &mut Rng,
    rng: &mut Rng,
) -> GameResult {
    run_game(rules, bots, rng, Some(deals))
}

fn run_game(
    rules: Rules,
    bots: &mut [&mut dyn Bot],
    rng: &mut Rng,
    mut deals: Option<&mut Rng>,
) -> GameResult {
    rules.validate().expect("valid rules");
    let n = rules.players;
    let mut totals = vec![0i32; n as usize];
    let mut history = vec![SeatHistory::default(); n as usize];
    let mut rounds = Vec::with_capacity(rules.rounds() as usize);
    let mut decisions = 0;
    let mut dealer = match deals.as_deref_mut() {
        Some(d) => d.below(n as u64) as u8,
        None => rng.below(n as u64) as u8,
    };
    for size in 1..=rules.rounds() {
        let mut round = match deals.as_deref_mut() {
            Some(d) => Round::deal(rules, size, dealer, d),
            None => Round::deal(rules, size, dealer, rng),
        };
        decisions += play_round_with(&mut round, bots, &totals, &history, rng);
        for (s, h) in history.iter_mut().enumerate() {
            h.record(&round, s as u8);
        }
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
