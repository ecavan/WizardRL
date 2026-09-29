//! Look-ahead search for bids: before bidding, imagine the round many times and see how each bid
//! works out, the way a strong player thinks "if I bid 2 here, how often do I make it?".
//!
//! For a bid decision the searcher:
//!
//! 1. deals the cards it can't see at random (the other hands), `samples` times; with bids in
//!    turn, the bids already made are kept; nothing else is known at bid time;
//! 2. for each of the network's `width` favourite bids, plays every imagined round to the end
//!    with the network making every other decision, for everyone;
//! 3. bids whatever scored best on average over the imagined rounds.
//!
//! Every candidate bid is tried on the same imagined deals, so the comparison between bids is
//! much less noisy than the scores themselves. All imagined rounds advance in lockstep so the
//! network runs on big batches. Everything but bids is played by the network directly.

use crate::bots::Bot;
use crate::card::{bit, cards, Card, ALL_CARDS};
use crate::encode::{self, ACTIONS, FEATURES};
use crate::net::Mlp;
use crate::rng::Rng;
use crate::round::{Action, Phase, Round, TrumpSource};
use crate::view::View;

pub struct SearchBot {
    net: Mlp,
    name: String,
    /// Imagined deals per bid decision.
    pub samples: usize,
    /// How many of the network's top bids to try.
    pub width: usize,
    obs: Vec<f32>,
    mask: Vec<bool>,
}

impl SearchBot {
    pub fn new(net: Mlp, name: impl Into<String>, samples: usize, width: usize) -> SearchBot {
        SearchBot {
            net,
            name: name.into(),
            samples: samples.max(1),
            width: width.max(1),
            obs: vec![0.0; FEATURES],
            mask: vec![false; ACTIONS],
        }
    }

    /// The network's choice (no search): best predicted score among legal actions.
    fn greedy(&mut self, v: &View) -> Action {
        encode::observe(v, &mut self.obs);
        encode::legal_mask(v, &mut self.mask);
        let q = self.net.forward(&self.obs);
        best_legal(&q, &self.mask)
    }

    /// Candidate bids: the network's `width` favourites.
    fn candidates(&mut self, v: &View) -> Vec<u8> {
        encode::observe(v, &mut self.obs);
        encode::legal_mask(v, &mut self.mask);
        let q = self.net.forward(&self.obs);
        let mut bids: Vec<(usize, f32)> = (0..ACTIONS)
            .filter(|&i| self.mask[i])
            .map(|i| (i, q[i]))
            .collect();
        bids.sort_by(|a, b| b.1.total_cmp(&a.1));
        bids.iter()
            .take(self.width)
            .map(|&(i, _)| match encode::action_from_index(i) {
                Some(Action::Bid(b)) => b,
                other => panic!("expected a bid, got {other:?}"),
            })
            .collect()
    }

    /// Average score of each candidate bid over the same imagined deals.
    pub fn evaluate_bids(&mut self, v: &View, rng: &mut Rng) -> Vec<(u8, f64)> {
        let cands = self.candidates(v);
        if cands.len() == 1 {
            return vec![(cands[0], 0.0)];
        }
        let me = v.seat();
        let n = v.players();
        let size = v.size();
        let rules = *v_rules(v);
        let turned = match v.trump_source() {
            TrumpSource::TurnedCard(c)
            | TrumpSource::WizardTurned(c)
            | TrumpSource::JesterTurned(c) => Some(c),
            TrumpSource::NoCardTurned => None,
        };
        let known = v.hand() | turned.map_or(0, bit);
        let unseen: Vec<Card> = cards(ALL_CARDS & !known).collect();
        let visible_bids = v.bids();
        let scores = v.scores().to_vec();
        let history = v.history().to_vec();
        // One world per (sample, candidate), candidates sharing each sample's deal.
        let mut worlds: Vec<(Round, u8)> = Vec::with_capacity(self.samples * cands.len());
        for _ in 0..self.samples {
            let mut deck = unseen.clone();
            rng.shuffle(&mut deck);
            let mut it = deck.into_iter();
            let hands: Vec<Vec<Card>> = (0..n)
                .map(|s| {
                    if s == me {
                        cards(v.hand()).collect()
                    } else {
                        (&mut it).take(size as usize).collect()
                    }
                })
                .collect();
            let mut base = Round::from_hands(rules, size, v.dealer(), &hands, turned);
            if let Phase::PickTrump { .. } = base.phase() {
                let t = v.trump().expect("trump was named before bidding");
                base.apply(Action::PickTrump(t)).expect("legal trump pick");
            }
            // Bids already on the table (bidding in turn) stay as they were.
            while let Some(s) = base.to_act() {
                if s == me {
                    break;
                }
                match visible_bids[s as usize] {
                    Some(b) => {
                        base.apply(Action::Bid(b)).expect("a bid that was legal");
                    }
                    None => break, // hidden (all bid at once): the network bids for them below
                }
            }
            for &b in &cands {
                worlds.push((base.clone(), b));
            }
        }
        // Play every world to the end in lockstep.
        let mut obs = Vec::new();
        let mut masks = Vec::new();
        let mut who = Vec::new();
        loop {
            obs.clear();
            masks.clear();
            who.clear();
            for (w, (round, my_bid)) in worlds.iter_mut().enumerate() {
                while let Some(s) = round.to_act() {
                    if s == me && matches!(round.phase(), Phase::Bid { .. }) {
                        round.apply(Action::Bid(*my_bid)).expect("a legal bid");
                        continue;
                    }
                    let view = View::with_history(round, s, &scores, &history);
                    let start = obs.len();
                    obs.resize(start + FEATURES, 0.0);
                    encode::observe(&view, &mut obs[start..]);
                    let m = masks.len();
                    masks.resize(m + ACTIONS, false);
                    encode::legal_mask(&view, &mut masks[m..]);
                    who.push(w);
                    break;
                }
            }
            if who.is_empty() {
                break;
            }
            let q = self.net.forward_batch(&obs, who.len(), FEATURES);
            let out = self.net.outputs();
            for (k, &w) in who.iter().enumerate() {
                let a = best_legal(
                    &q[k * out..k * out + ACTIONS],
                    &masks[k * ACTIONS..(k + 1) * ACTIONS],
                );
                worlds[w].0.apply(a).expect("legal");
            }
        }
        cands
            .iter()
            .enumerate()
            .map(|(c, &b)| {
                let total: i64 = (0..self.samples)
                    .map(|s| {
                        worlds[s * cands.len() + c].0.scores().expect("done")[me as usize] as i64
                    })
                    .sum();
                (b, total as f64 / self.samples as f64)
            })
            .collect()
    }
}

fn v_rules<'a>(v: &View<'a>) -> &'a crate::rules::Rules {
    v.rules()
}

fn best_legal(q: &[f32], mask: &[bool]) -> Action {
    let i = (0..ACTIONS)
        .filter(|&i| mask[i])
        .max_by(|&a, &b| q[a].total_cmp(&q[b]))
        .expect("a legal action");
    encode::action_from_index(i).expect("valid index")
}

impl Bot for SearchBot {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn act(&mut self, v: &View, rng: &mut Rng) -> Action {
        match v.phase() {
            Phase::Bid { .. } => {
                let scored = self.evaluate_bids(v, rng);
                let best = scored
                    .iter()
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .expect("a candidate");
                Action::Bid(best.0)
            }
            _ => self.greedy(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rules;

    fn tiny_net() -> Mlp {
        // A random small network is enough to exercise the machinery.
        let mut rng = Rng::new(3);
        let mut bytes = b"WZNET001".to_vec();
        let hidden = 16u32;
        bytes.extend((FEATURES as u32).to_le_bytes());
        bytes.extend((ACTIONS as u32).to_le_bytes());
        bytes.extend(100f32.to_le_bytes());
        bytes.extend(2u32.to_le_bytes());
        for (i, o) in [(FEATURES as u32, hidden), (hidden, ACTIONS as u32)] {
            bytes.extend(i.to_le_bytes());
            bytes.extend(o.to_le_bytes());
            for _ in 0..(i * o + o) {
                bytes.extend(((rng.unit() as f32 - 0.5) * 0.2).to_le_bytes());
            }
        }
        Mlp::from_bytes(&bytes).unwrap()
    }

    #[test]
    fn batch_forward_matches_single_rows() {
        let net = tiny_net();
        let mut rng = Rng::new(1);
        let x: Vec<f32> = (0..3 * FEATURES).map(|_| rng.unit() as f32).collect();
        let y = net.forward_batch(&x, 3, FEATURES);
        for r in 0..3 {
            let one = net.forward(&x[r * FEATURES..(r + 1) * FEATURES]);
            for (a, b) in one.iter().zip(&y[r * ACTIONS..(r + 1) * ACTIONS]) {
                assert!((a - b).abs() < 1e-4, "{a} {b}");
            }
        }
    }

    #[test]
    fn search_bids_legally_and_plays_whole_rounds() {
        let mut bot = SearchBot::new(tiny_net(), "search", 4, 3);
        let mut rng = Rng::new(7);
        for (simul, players, size) in [(true, 4, 5), (false, 3, 7), (true, 6, 10), (false, 5, 12)] {
            let rules = if simul {
                Rules::simultaneous(players)
            } else {
                Rules::official(players)
            };
            let mut r = Round::deal(rules, size, 1, &mut rng);
            while let Some(s) = r.to_act() {
                let a = bot.act(&View::new(&r, s, &[]), &mut rng);
                r.apply(a).expect("the search bot plays legal moves");
            }
            assert!(r.is_done());
        }
    }
}
