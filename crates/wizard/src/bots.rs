//! Baseline bots: a random player (the floor) and a simple counting player (a decent casual
//! player, the first real yardstick for the learning bot).

use crate::card::{self, cards, is_jester, is_wizard, rank_of, suit_of, Card, CardSet, Suit};
use crate::rng::Rng;
use crate::round::{trick_winner, Action, Phase};
use crate::view::View;

pub trait Bot {
    fn name(&self) -> String;
    fn act(&mut self, v: &View, rng: &mut Rng) -> Action;
}

/// Picks uniformly among legal actions.
#[derive(Default)]
pub struct RandomBot;

impl Bot for RandomBot {
    fn name(&self) -> String {
        "random".into()
    }
    fn act(&mut self, v: &View, rng: &mut Rng) -> Action {
        let legal = v.legal_actions();
        assert!(!legal.is_empty(), "asked to act when it isn't our turn");
        legal[rng.below(legal.len() as u64) as usize]
    }
}

/// Bids by adding up how likely each card is to win a trick, then plays greedily: win cheaply
/// while it still needs tricks, duck with its highest safe card once it has enough.
#[derive(Default)]
pub struct CountingBot;

impl CountingBot {
    /// Rough chance a card takes a trick, given trump and table size.
    pub fn card_value(c: Card, trump: Option<Suit>, players: u8) -> f64 {
        if is_wizard(c) {
            return 1.0;
        }
        if is_jester(c) {
            return 0.0;
        }
        let rank = rank_of(c).unwrap(); // 12 = ace
        let crowd = 1.0 - 0.06 * (players as f64 - 4.0); // more players, more competition
        let v = if Some(suit_of(c).unwrap()) == trump {
            match rank {
                12 => 0.95,
                11 => 0.85,
                10 => 0.7,
                9 => 0.55,
                8 => 0.45,
                _ => 0.3,
            }
        } else if trump.is_none() {
            match rank {
                12 => 0.75,
                11 => 0.45,
                10 => 0.2,
                _ => 0.0,
            }
        } else {
            match rank {
                12 => 0.6,
                11 => 0.3,
                _ => 0.0,
            }
        };
        (v * crowd).clamp(0.0, 1.0)
    }

    pub fn expected_tricks(hand: CardSet, trump: Option<Suit>, players: u8) -> f64 {
        cards(hand)
            .map(|c| Self::card_value(c, trump, players))
            .sum()
    }

    /// How strong a card is to hold on to: Jesters lowest, then plain cards, trumps, Wizards.
    fn strength(c: Card, trump: Option<Suit>) -> u32 {
        match card::kind(c) {
            card::Kind::Jester => 0,
            card::Kind::Wizard => 100,
            card::Kind::Normal { suit, rank } => {
                1 + rank as u32 + if Some(suit) == trump { 40 } else { 0 }
            }
        }
    }

    fn would_win(v: &View, c: Card) -> bool {
        let mut plays: Vec<(u8, Card)> = v.current_trick().to_vec();
        plays.push((v.seat(), c));
        plays[trick_winner(&plays, v.trump())].0 == v.seat()
    }

    fn pick_trump(v: &View) -> Suit {
        let hand = v.hand();
        *Suit::ALL
            .iter()
            .max_by_key(|s| {
                let mine: Vec<Card> = cards(hand & s.mask()).collect();
                10 * mine.len() as u32
                    + mine
                        .iter()
                        .map(|&c| rank_of(c).unwrap() as u32)
                        .sum::<u32>()
            })
            .unwrap()
    }

    fn choose_card(v: &View) -> Card {
        let legal: Vec<Card> = cards(v.legal_plays()).collect();
        let trump = v.trump();
        let bid = v.my_bid().unwrap_or(0) as i32;
        let need = bid - v.tricks_won(v.seat()) as i32;
        let by_strength = |a: &Card, b: &Card| {
            Self::strength(*a, trump)
                .cmp(&Self::strength(*b, trump))
                .then(a.cmp(b))
        };
        let lowest = |set: &[Card]| *set.iter().min_by(|a, b| by_strength(a, b)).unwrap();
        let highest = |set: &[Card]| *set.iter().max_by(|a, b| by_strength(a, b)).unwrap();

        if v.current_trick().is_empty() {
            // Leading.
            return if need > 0 {
                highest(&legal)
            } else {
                lowest(&legal)
            };
        }
        let winners: Vec<Card> = legal
            .iter()
            .copied()
            .filter(|&c| Self::would_win(v, c))
            .collect();
        let losers: Vec<Card> = legal
            .iter()
            .copied()
            .filter(|&c| !Self::would_win(v, c))
            .collect();
        if need > 0 {
            if !winners.is_empty() {
                lowest(&winners)
            } else {
                lowest(&legal)
            }
        } else if !losers.is_empty() {
            highest(&losers)
        } else {
            lowest(&legal)
        }
    }
}

impl Bot for CountingBot {
    fn name(&self) -> String {
        "counting".into()
    }
    fn act(&mut self, v: &View, _rng: &mut Rng) -> Action {
        match v.phase() {
            Phase::PickTrump { .. } => Action::PickTrump(Self::pick_trump(v)),
            Phase::Bid { .. } => {
                let e = Self::expected_tricks(v.hand(), v.trump(), v.players());
                Action::Bid((e.round() as u8).min(v.size()))
            }
            Phase::Play { .. } => Action::Play(Self::choose_card(v)),
            Phase::Done => panic!("asked to act after the round ended"),
        }
    }
}

/// Parse a bot name as used on the command line.
pub fn by_name(name: &str) -> Option<Box<dyn Bot>> {
    match name {
        "random" => Some(Box::new(RandomBot)),
        "counting" => Some(Box::new(CountingBot)),
        _ => None,
    }
}
