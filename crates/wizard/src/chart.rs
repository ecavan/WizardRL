//! A bot that bids from a bid chart (`rl/charts/bid_chart.csv`, made by
//! `python -m wizard_rl.bidchart`): add up the chart's value for each card and round. It picks
//! trump and plays its cards like the counting bot, so pitting it against counting bots measures
//! what the chart's bidding alone is worth.
//!
//! The CSV has one row per (table size, trump or no trump, range of round sizes, card kind):
//! `players,trump,lo,hi,kind,value`, with `trump` = `trump` | `no_trump` and kinds as in
//! [`kind_name`] (one value per rank: "trump 10", "off-suit A", ...).

use crate::bots::{Bot, CountingBot};
use crate::card::{cards, is_jester, is_wizard, rank_of, suit_of, Card, Suit};
use crate::rng::Rng;
use crate::round::{Action, Phase};
use crate::view::View;
use std::collections::HashMap;

/// The chart's name for a card's kind: "Wizard", "Jester", "trump A" ... "trump 2",
/// "off-suit A" ... "off-suit 2". In no-trump rounds every suit is off-suit.
pub fn kind_name(c: Card, trump: Option<Suit>) -> String {
    if is_wizard(c) {
        return "Wizard".into();
    }
    if is_jester(c) {
        return "Jester".into();
    }
    const RANKS: [&str; 13] = [
        "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A",
    ];
    let rank = RANKS[rank_of(c).expect("a standard card") as usize]; // 0 = deuce ... 12 = ace
    if suit_of(c) == trump {
        format!("trump {rank}")
    } else {
        format!("off-suit {rank}")
    }
}

struct Band {
    players: u8,
    no_trump: bool,
    lo: u8,
    hi: u8,
    values: HashMap<String, f64>,
}

pub struct ChartBot {
    bands: Vec<Band>,
    path: String,
}

impl ChartBot {
    pub fn load(path: &str) -> Result<ChartBot, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        ChartBot::parse(&text, path)
    }

    pub fn parse(text: &str, path: &str) -> Result<ChartBot, String> {
        let mut lines = text.lines();
        let header: Vec<&str> = lines
            .next()
            .ok_or(format!("{path}: empty"))?
            .split(',')
            .collect();
        let col = |name: &str| {
            header
                .iter()
                .position(|h| h.trim() == name)
                .ok_or(format!("{path}: no '{name}' column"))
        };
        let (cp, ct, cl, ch, ck, cv) = (
            col("players")?,
            col("trump")?,
            col("lo")?,
            col("hi")?,
            col("kind")?,
            col("value")?,
        );
        let mut bands: Vec<Band> = Vec::new();
        for (i, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            let bad = || format!("{path}: bad line {}: {line}", i + 2);
            let players: u8 = f.get(cp).and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let no_trump = f.get(ct).ok_or_else(bad)? == &"no_trump";
            let lo: u8 = f.get(cl).and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let hi: u8 = f.get(ch).and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            let kind = f.get(ck).ok_or_else(bad)?.to_string();
            let value: f64 = f.get(cv).and_then(|s| s.parse().ok()).ok_or_else(bad)?;
            match bands.iter_mut().find(|b| {
                b.players == players && b.no_trump == no_trump && b.lo == lo && b.hi == hi
            }) {
                Some(b) => {
                    b.values.insert(kind, value);
                }
                None => bands.push(Band {
                    players,
                    no_trump,
                    lo,
                    hi,
                    values: HashMap::from([(kind, value)]),
                }),
            }
        }
        if bands.is_empty() {
            return Err(format!("{path}: no chart rows"));
        }
        Ok(ChartBot {
            bands,
            path: path.to_string(),
        })
    }

    /// The chart's expected tricks for a hand, or `None` if the chart has no row for this table
    /// size, trump state and round size.
    pub fn expected_tricks(
        &self,
        hand: &[Card],
        trump: Option<Suit>,
        players: u8,
        size: u8,
    ) -> Option<f64> {
        let no_trump = trump.is_none();
        let band = self.bands.iter().find(|b| {
            b.players == players && b.no_trump == no_trump && (b.lo..=b.hi).contains(&size)
        })?;
        Some(
            hand.iter()
                .map(|&c| {
                    band.values
                        .get(&kind_name(c, trump))
                        .copied()
                        .unwrap_or(0.0)
                })
                .sum(),
        )
    }
}

impl Bot for ChartBot {
    fn name(&self) -> String {
        format!("chart:{}", self.path)
    }
    fn act(&mut self, v: &View, rng: &mut Rng) -> Action {
        match v.phase() {
            Phase::Bid { .. } => {
                let hand: Vec<Card> = cards(v.hand()).collect();
                let e = self
                    .expected_tricks(&hand, v.trump(), v.players(), v.size())
                    .unwrap_or_else(|| {
                        CountingBot::expected_tricks(v.hand(), v.trump(), v.players())
                    });
                Action::Bid((e.max(0.0).round() as u8).min(v.size()))
            }
            _ => CountingBot.act(v, rng),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::parse;

    const CSV: &str = "players,trump,cards,lo,hi,kind,value
4,trump,3-4 cards,3,4,Wizard,1.0
4,trump,3-4 cards,3,4,trump 3,0.4
4,trump,3-4 cards,3,4,off-suit A,0.5
4,no_trump,15 cards,15,15,off-suit K,0.3
";

    #[test]
    fn adds_up_card_values_for_the_right_band() {
        let bot = ChartBot::parse(CSV, "test").unwrap();
        let hand: Vec<Card> = ["wiz", "3h", "As"]
            .iter()
            .map(|s| parse(s).unwrap())
            .collect();
        let e = bot
            .expected_tricks(&hand, Some(Suit::Hearts), 4, 3)
            .unwrap();
        assert!((e - 1.9).abs() < 1e-9, "{e}");
        assert_eq!(bot.expected_tricks(&hand, Some(Suit::Hearts), 4, 5), None);
        assert_eq!(bot.expected_tricks(&hand, Some(Suit::Hearts), 5, 3), None);
        assert_eq!(
            kind_name(parse("10h").unwrap(), Some(Suit::Hearts)),
            "trump 10"
        );
        assert_eq!(
            kind_name(parse("6h").unwrap(), Some(Suit::Hearts)),
            "trump 6"
        );
        assert_eq!(kind_name(parse("Ah").unwrap(), None), "off-suit A");
        assert!(ChartBot::parse("players,trump\n", "x").is_err());
    }
}
