//! Player styles: a base player with a recognisable habit, for training a bot to spot and
//! exploit habits, and for playing against "someone like my sister".
//!
//! - `overbid`: bids one more than the base player would (70% of the time);
//! - `underbid`: bids one fewer (70%);
//! - `early-wizard`: throws a Wizard on the first trick whenever it can (90%);
//! - `wild`: a casual player, 20% of decisions random.

use crate::bots::Bot;
use crate::card::{cards, is_wizard};
use crate::rng::Rng;
use crate::round::{Action, Phase};
use crate::view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Overbid,
    Underbid,
    EarlyWizard,
    Wild,
}

impl Style {
    pub const ALL: [Style; 4] = [
        Style::Overbid,
        Style::Underbid,
        Style::EarlyWizard,
        Style::Wild,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Style::Overbid => "overbid",
            Style::Underbid => "underbid",
            Style::EarlyWizard => "early-wizard",
            Style::Wild => "wild",
        }
    }

    pub fn parse(s: &str) -> Option<Style> {
        Style::ALL.into_iter().find(|x| x.name() == s)
    }

    /// Turn the base player's choice into this style's.
    pub fn apply(self, base: Action, v: &View, rng: &mut Rng) -> Action {
        match (self, base) {
            (Style::Overbid, Action::Bid(b)) if rng.unit() < 0.7 => {
                Action::Bid((b + 1).min(v.size()))
            }
            (Style::Underbid, Action::Bid(b)) if rng.unit() < 0.7 => {
                Action::Bid(b.saturating_sub(1))
            }
            (Style::EarlyWizard, Action::Play(_))
                if v.completed_tricks().is_empty() && rng.unit() < 0.9 =>
            {
                match cards(v.legal_plays()).find(|&c| is_wizard(c)) {
                    Some(w) => Action::Play(w),
                    None => base,
                }
            }
            (Style::Wild, _) if rng.unit() < 0.2 => {
                let legal = v.legal_actions();
                legal[rng.below(legal.len() as u64) as usize]
            }
            _ => base,
        }
    }
}

/// A base player with a style.
pub struct StyleBot {
    pub style: Style,
    pub base: Box<dyn Bot>,
}

impl Bot for StyleBot {
    fn name(&self) -> String {
        format!("{}:{}", self.style.name(), self.base.name())
    }
    fn act(&mut self, v: &View, rng: &mut Rng) -> Action {
        let a = self.base.act(v, rng);
        if matches!(v.phase(), Phase::PickTrump { .. }) {
            return a;
        }
        self.style.apply(a, v, rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bots::CountingBot;
    use crate::game::play_game;
    use crate::rules::Rules;

    #[test]
    fn styled_players_play_legal_games_and_show_their_habits() {
        let mut rng = Rng::new(11);
        let mut over = StyleBot {
            style: Style::Overbid,
            base: Box::new(CountingBot),
        };
        let mut under = StyleBot {
            style: Style::Underbid,
            base: Box::new(CountingBot),
        };
        let mut wiz = StyleBot {
            style: Style::EarlyWizard,
            base: Box::new(CountingBot),
        };
        let mut wild = StyleBot {
            style: Style::Wild,
            base: Box::new(CountingBot),
        };
        let (mut bid_over, mut bid_under) = (0i64, 0i64);
        for _ in 0..30 {
            let mut seats: Vec<&mut dyn Bot> = vec![&mut over, &mut under, &mut wiz, &mut wild];
            let g = play_game(Rules::simultaneous(4), &mut seats, &mut rng);
            for r in &g.rounds {
                bid_over += r.bids[0] as i64;
                bid_under += r.bids[1] as i64;
            }
        }
        assert!(bid_over > bid_under + 100, "{bid_over} vs {bid_under}");
    }
}
