//! A batch of Wizard tables for training, driven from Python.
//!
//! Every seat at a table is one of:
//! - the **learner**: its decisions are passed to Python and become training samples;
//! - a **frozen network** (`Net(k)`): an older version of the learner, also run by Python
//!   (in a batch), but its decisions are not trained on;
//! - a **counting** or **random** bot, played here in Rust.
//!
//! One random seat is always the learner; the others are drawn from a [`SeatMix`]. Each table
//! plays single rounds (every round is scored on its own) with a random table size from the
//! config, round size and dealer. When a round ends, every learner decision in it becomes a
//! sample labelled with that seat's round score and whether it made its bid.
//!
//! **Duplicate mode** (for evaluation): each deal is replayed once per seat with the learner
//! moved one seat along each time, so the learner and the other players hold exactly the same
//! cards over a cycle and luck cancels out of the comparison.

use crate::bots::{Bot, CountingBot, RandomBot};
use crate::encode::{self, ACTIONS, FEATURES};
use crate::rng::Rng;
use crate::round::Round;
use crate::rules::{Rules, MAX_PLAYERS, MIN_PLAYERS};
use crate::view::View;

const SEATS: usize = MAX_PLAYERS as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seat {
    Learner,
    /// A frozen network held by Python, numbered from 1.
    Net(u16),
    Counting,
    Random,
}

/// Relative weights for the kinds of player in the seats other than the learner's anchor seat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeatMix {
    pub learner: f64,
    pub nets: f64,
    pub counting: f64,
    pub random: f64,
}

impl SeatMix {
    pub const SELF_PLAY: SeatMix = SeatMix {
        learner: 1.0,
        nets: 0.0,
        counting: 0.0,
        random: 0.0,
    };
    pub const COUNTING: SeatMix = SeatMix {
        learner: 0.0,
        nets: 0.0,
        counting: 1.0,
        random: 0.0,
    };
    pub const RANDOM: SeatMix = SeatMix {
        learner: 0.0,
        nets: 0.0,
        counting: 0.0,
        random: 1.0,
    };
    pub const NETS: SeatMix = SeatMix {
        learner: 0.0,
        nets: 1.0,
        counting: 0.0,
        random: 0.0,
    };

    /// `selfplay`, `counting`, `random`, `nets` (frozen networks only), or `train`
    /// (mostly the learner, some frozen networks, a few counting bots).
    pub fn preset(s: &str) -> Option<SeatMix> {
        match s {
            "selfplay" | "self" => Some(SeatMix::SELF_PLAY),
            "counting" => Some(SeatMix::COUNTING),
            "random" => Some(SeatMix::RANDOM),
            "nets" => Some(SeatMix::NETS),
            "train" => Some(SeatMix {
                learner: 0.7,
                nets: 0.2,
                counting: 0.1,
                random: 0.0,
            }),
            _ => None,
        }
    }

    fn validate(&self) -> Result<(), String> {
        let w = [self.learner, self.nets, self.counting, self.random];
        if w.iter().any(|x| !x.is_finite() || *x < 0.0) || w.iter().sum::<f64>() <= 0.0 {
            return Err("seat mix weights must be non-negative and not all zero".into());
        }
        Ok(())
    }

    /// Draw a kind for one seat. With no frozen networks available, their share goes to the
    /// learner.
    fn draw(&self, nets_available: u16, rng: &mut Rng) -> Seat {
        let nets = if nets_available > 0 { self.nets } else { 0.0 };
        let learner = if nets_available > 0 {
            self.learner
        } else {
            self.learner + self.nets
        };
        let total = learner + nets + self.counting + self.random;
        if total <= 0.0 {
            return Seat::Learner;
        }
        let mut x = rng.unit() * total;
        if x < learner {
            return Seat::Learner;
        }
        x -= learner;
        if x < nets {
            return Seat::Net(1 + rng.below(nets_available as u64) as u16);
        }
        x -= nets;
        if x < self.counting {
            Seat::Counting
        } else {
            Seat::Random
        }
    }

    /// True if every non-learner seat is the same fixed kind (needed for duplicate deals).
    fn is_single_kind(&self) -> bool {
        self.learner == 0.0
            && [self.nets, self.counting, self.random]
                .iter()
                .filter(|&&w| w > 0.0)
                .count()
                == 1
    }
}

#[derive(Clone, Debug)]
pub struct EnvConfig {
    /// Table sizes to draw from, uniformly.
    pub players: Vec<u8>,
    pub mix: SeatMix,
    /// Replay each deal with the learner in every seat (evaluation).
    pub duplicate: bool,
    /// Everyone bids at once (no one sees another bid while bidding).
    pub simultaneous: bool,
    /// Keep a record of every finished round (hands, bids, tricks), for bid charts.
    pub log_rounds: bool,
}

/// One seat's round, for bid charts: what it was dealt and how it went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeatRound {
    pub players: u8,
    pub size: u8,
    /// Trump suit index, or 4 for no trump.
    pub trump: u8,
    /// Seats left of the dealer: 1 = bids and leads first, `players` = the dealer.
    pub position: u8,
    /// The hand as dealt.
    pub hand: u64,
    pub bid: u8,
    pub won: u8,
    pub learner: bool,
}

/// Running totals since the last `take_stats`.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    /// Rounds finished.
    pub rounds: u64,
    /// Learner seat-rounds (one per learner seat per round).
    pub learner_rounds: u64,
    pub learner_score: i64,
    pub learner_bids_made: u64,
    /// Every other seat-round (frozen networks and bots).
    pub other_rounds: u64,
    pub other_score: i64,
    pub other_bids_made: u64,
    /// Learner decisions taken.
    pub decisions: u64,
    /// Sum over rounds of (learner seats' average score - other seats' average score), for rounds
    /// with both; `edge_sum / edge_rounds` is the edge. (Averaging per round keeps big and small
    /// tables on an equal footing; pooling seat-rounds would weight big tables more for "others".)
    pub edge_sum: f64,
    pub edge_rounds: u64,
}

/// Training samples: `obs` is `len * FEATURES` long.
#[derive(Clone, Debug, Default)]
pub struct Samples {
    pub obs: Vec<f32>,
    pub actions: Vec<u32>,
    /// The seat's score for the round.
    pub returns: Vec<f32>,
    /// 1.0 if the seat made its bid.
    pub made: Vec<f32>,
    /// Whatever Python passed with the action (`step_with`), e.g. its log-probability.
    pub aux: Vec<f32>,
    /// The legal-action mask at each decision, `len * ACTIONS` long.
    pub legal: Vec<bool>,
}

impl Samples {
    pub fn len(&self) -> usize {
        self.actions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

struct Table {
    round: Round,
    seats: [Seat; SEATS],
    rng: Rng,
    /// Duplicate mode: the deal being replayed and how many replays are left.
    dup_seed: u64,
    dup_left: u8,
    /// Observation and legal mask for the pending decision (learner or frozen network).
    obs: Vec<f32>,
    mask: Vec<bool>,
    /// This round's learner decisions: (seat, action), with observations in `traj_obs`.
    traj: Vec<(u8, u32, f32)>,
    /// Hands as dealt this round (for the round log).
    dealt: [u64; SEATS],
    traj_obs: Vec<f32>,
    traj_mask: Vec<bool>,
    /// Results of a duplicate cycle still being replayed.
    pending: Stats,
    /// Deals (duplicate cycles, or rounds) this table has counted in the stats.
    counted: u64,
    /// The deal in progress when the quota was set: not counted (it's more likely to be a long
    /// one, having been caught in progress).
    skip: bool,
}

pub struct VecEnv {
    cfg: EnvConfig,
    nets: u16,
    tables: Vec<Table>,
    out: Samples,
    stats: Stats,
    rounds_log: Vec<SeatRound>,
    /// Count at most this many deals per table (0 = no limit); see `set_quota`.
    quota: u64,
}

impl VecEnv {
    pub fn new(num_tables: usize, cfg: EnvConfig, seed: u64) -> Result<VecEnv, String> {
        if num_tables == 0 {
            return Err("need at least one table".into());
        }
        if cfg.players.is_empty()
            || cfg
                .players
                .iter()
                .any(|p| !(MIN_PLAYERS..=MAX_PLAYERS).contains(p))
        {
            return Err(format!(
                "players must be drawn from {MIN_PLAYERS}..={MAX_PLAYERS}"
            ));
        }
        cfg.mix.validate()?;
        if cfg.duplicate && !cfg.mix.is_single_kind() {
            return Err(
                "duplicate deals need every other seat to be the same kind of player".into(),
            );
        }
        let mut seeder = Rng::new(seed);
        let mut env = VecEnv {
            cfg,
            nets: 0,
            tables: Vec::with_capacity(num_tables),
            out: Samples::default(),
            stats: Stats::default(),
            rounds_log: Vec::new(),
            quota: 0,
        };
        for _ in 0..num_tables {
            let rng = seeder.fork();
            let placeholder = Round::deal(Rules::official(3), 1, 0, &mut Rng::new(0));
            env.tables.push(Table {
                round: placeholder,
                seats: [Seat::Learner; SEATS],
                rng,
                dup_seed: 0,
                dup_left: 0,
                obs: vec![0.0; FEATURES],
                mask: vec![false; ACTIONS],
                traj: Vec::new(),
                dealt: [0; SEATS],
                traj_obs: Vec::new(),
                traj_mask: Vec::new(),
                pending: Stats::default(),
                counted: 0,
                skip: false,
            });
        }
        for i in 0..num_tables {
            env.redeal(i);
            env.advance(i);
        }
        Ok(env)
    }

    pub fn len(&self) -> usize {
        self.tables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    /// How many frozen networks Python holds (seats `Net(1..=n)` may be dealt from now on).
    /// Tables already mid-round keep the networks they were dealt.
    pub fn set_nets(&mut self, n: u16) {
        self.nets = n;
    }

    pub fn nets(&self) -> u16 {
        self.nets
    }

    /// From now on, count only the next `deals` deals of every table in the stats (a deal is a
    /// full duplicate cycle, or one round without duplicates); 0 removes the limit.
    ///
    /// Why: stopping after "enough rounds" favours deals that finish quickly (small rounds), which
    /// skews averages. A fixed number of deals per table does not.
    pub fn set_quota(&mut self, deals: u64) {
        self.quota = deals;
        for t in &mut self.tables {
            t.counted = 0;
            t.skip = deals > 0;
        }
    }

    /// Tables that have not yet counted their quota of deals (0 with no quota).
    pub fn quota_left(&self) -> usize {
        if self.quota == 0 {
            return 0;
        }
        self.tables
            .iter()
            .filter(|t| t.counted < self.quota)
            .count()
    }

    /// Observations `[tables, FEATURES]`, legal masks `[tables, ACTIONS]`, and who decides
    /// (`0` = the learner, `k` = frozen network `k`) for every table's pending decision.
    pub fn observe_into(&self, obs: &mut [f32], mask: &mut [bool], owner: &mut [u16]) {
        assert_eq!(obs.len(), self.tables.len() * FEATURES);
        assert_eq!(mask.len(), self.tables.len() * ACTIONS);
        assert_eq!(owner.len(), self.tables.len());
        for (i, t) in self.tables.iter().enumerate() {
            obs[i * FEATURES..(i + 1) * FEATURES].copy_from_slice(&t.obs);
            mask[i * ACTIONS..(i + 1) * ACTIONS].copy_from_slice(&t.mask);
            let seat = t.round.to_act().expect("a pending decision");
            owner[i] = match t.seats[seat as usize] {
                Seat::Net(k) => k,
                _ => 0,
            };
        }
    }

    /// Apply one action index per table (see `encode`). Illegal actions are an error and
    /// leave every table unchanged.
    pub fn step(&mut self, actions: &[usize]) -> Result<(), String> {
        self.step_with(actions, None)
    }

    /// Like `step`, with a number per table to carry into that decision's training sample.
    pub fn step_with(&mut self, actions: &[usize], aux: Option<&[f32]>) -> Result<(), String> {
        if let Some(x) = aux {
            if x.len() != self.tables.len() {
                return Err(format!(
                    "{} aux values for {} tables",
                    x.len(),
                    self.tables.len()
                ));
            }
        }
        if actions.len() != self.tables.len() {
            return Err(format!(
                "{} actions for {} tables",
                actions.len(),
                self.tables.len()
            ));
        }
        for (i, &a) in actions.iter().enumerate() {
            if a >= ACTIONS || !self.tables[i].mask[a] {
                return Err(format!("table {i}: action {a} is not legal"));
            }
        }
        for (i, &a) in actions.iter().enumerate() {
            let t = &mut self.tables[i];
            let seat = t.round.to_act().expect("a pending decision");
            if t.seats[seat as usize] == Seat::Learner {
                t.traj.push((seat, a as u32, aux.map_or(0.0, |x| x[i])));
                t.traj_obs.extend_from_slice(&t.obs);
                t.traj_mask.extend_from_slice(&t.mask);
                self.stats.decisions += 1;
            }
            t.round
                .apply(encode::action_from_index(a).unwrap())
                .expect("checked legal");
            self.advance(i);
        }
        Ok(())
    }

    /// Take the samples from rounds finished since the last call.
    pub fn drain(&mut self) -> Samples {
        std::mem::take(&mut self.out)
    }

    /// Take the round records kept since the last call (only with `log_rounds`).
    pub fn drain_rounds(&mut self) -> Vec<SeatRound> {
        std::mem::take(&mut self.rounds_log)
    }

    pub fn take_stats(&mut self) -> Stats {
        std::mem::take(&mut self.stats)
    }

    /// Deal table `i` a new round (or the next replay of its deal, in duplicate mode).
    fn redeal(&mut self, i: usize) {
        let cfg = &self.cfg;
        let nets = self.nets;
        let t = &mut self.tables[i];
        let mut rng = if cfg.duplicate {
            if t.dup_left == 0 {
                t.dup_seed = t.rng.next_u64();
            }
            Rng::new(t.dup_seed)
        } else {
            t.rng.fork()
        };
        let n = cfg.players[rng.below(cfg.players.len() as u64) as usize];
        let rules = if cfg.simultaneous {
            Rules::simultaneous(n)
        } else {
            Rules::official(n)
        };
        let size = 1 + rng.below(rules.rounds() as u64) as u8;
        let dealer = rng.below(n as u64) as u8;
        t.round = Round::deal(rules, size, dealer, &mut rng);
        let mut seats = [Seat::Learner; SEATS];
        for s in seats.iter_mut().take(n as usize) {
            *s = cfg.mix.draw(nets, &mut rng);
        }
        let anchor = if cfg.duplicate {
            if t.dup_left == 0 {
                t.dup_left = n;
            }
            t.dup_left -= 1;
            (n - 1 - t.dup_left) as usize
        } else {
            t.rng.below(n as u64) as usize
        };
        seats[anchor] = Seat::Learner;
        t.seats = seats;
        for s in 0..n {
            t.dealt[s as usize] = t.round.hand(s);
        }
    }

    /// Play bot seats and finished rounds until table `i` waits on a decision from Python.
    fn advance(&mut self, i: usize) {
        loop {
            let t = &mut self.tables[i];
            match t.round.to_act() {
                None => {
                    let scores = t.round.scores().expect("round over");
                    let n = t.round.players();
                    let made: Vec<bool> = (0..n)
                        .map(|s| t.round.bid(s) == Some(t.round.tricks_won(s)))
                        .collect();
                    for (k, &(seat, action, aux)) in t.traj.iter().enumerate() {
                        self.out
                            .obs
                            .extend_from_slice(&t.traj_obs[k * FEATURES..(k + 1) * FEATURES]);
                        self.out.actions.push(action);
                        self.out.returns.push(scores[seat as usize] as f32);
                        self.out.made.push(made[seat as usize] as u8 as f32);
                        self.out.aux.push(aux);
                        self.out
                            .legal
                            .extend_from_slice(&t.traj_mask[k * ACTIONS..(k + 1) * ACTIONS]);
                    }
                    if self.cfg.log_rounds {
                        let trump = t.round.trump().map_or(4, |s| s.index());
                        for s in 0..n {
                            self.rounds_log.push(SeatRound {
                                players: n,
                                size: t.round.size(),
                                trump,
                                position: (s + n - t.round.dealer()) % n
                                    + if s == t.round.dealer() { n } else { 0 },
                                hand: t.dealt[s as usize],
                                bid: t.round.bid(s).expect("everyone bid"),
                                won: t.round.tricks_won(s),
                                learner: t.seats[s as usize] == Seat::Learner,
                            });
                        }
                    }
                    t.traj.clear();
                    t.traj_obs.clear();
                    t.traj_mask.clear();
                    // In duplicate mode, results count only once a deal has been replayed in
                    // every seat, so the learner and the others always hold the same cards.
                    let st = &mut t.pending;
                    st.rounds += 1;
                    let (mut ls, mut ln, mut os, mut on) = (0i64, 0i64, 0i64, 0i64);
                    for s in 0..n as usize {
                        if t.seats[s] == Seat::Learner {
                            ls += scores[s] as i64;
                            ln += 1;
                        } else {
                            os += scores[s] as i64;
                            on += 1;
                        }
                    }
                    if ln > 0 && on > 0 {
                        st.edge_sum += ls as f64 / ln as f64 - os as f64 / on as f64;
                        st.edge_rounds += 1;
                    }
                    for s in 0..n as usize {
                        if t.seats[s] == Seat::Learner {
                            st.learner_rounds += 1;
                            st.learner_score += scores[s] as i64;
                            st.learner_bids_made += made[s] as u64;
                        } else {
                            st.other_rounds += 1;
                            st.other_score += scores[s] as i64;
                            st.other_bids_made += made[s] as u64;
                        }
                    }
                    if !self.cfg.duplicate || t.dup_left == 0 {
                        let p = std::mem::take(&mut t.pending);
                        // With a quota, a deal counts only if it started after `set_quota` and
                        // is within this table's quota.
                        if self.quota > 0 && (t.skip || t.counted >= self.quota) {
                            t.skip = false;
                            self.redeal(i);
                            continue;
                        }
                        t.counted += 1;
                        self.stats.rounds += p.rounds;
                        self.stats.learner_rounds += p.learner_rounds;
                        self.stats.learner_score += p.learner_score;
                        self.stats.learner_bids_made += p.learner_bids_made;
                        self.stats.other_rounds += p.other_rounds;
                        self.stats.other_score += p.other_score;
                        self.stats.other_bids_made += p.other_bids_made;
                        self.stats.edge_sum += p.edge_sum;
                        self.stats.edge_rounds += p.edge_rounds;
                    }
                    self.redeal(i);
                }
                Some(seat) => match t.seats[seat as usize] {
                    Seat::Learner | Seat::Net(_) => {
                        let v = View::new(&t.round, seat, &[]);
                        encode::observe(&v, &mut t.obs);
                        encode::legal_mask(&v, &mut t.mask);
                        return;
                    }
                    kind => {
                        let a = {
                            let v = View::new(&t.round, seat, &[]);
                            match kind {
                                Seat::Counting => CountingBot.act(&v, &mut t.rng),
                                _ => RandomBot.act(&v, &mut t.rng),
                            }
                        };
                        t.round.apply(a).expect("bots play legal moves");
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(players: Vec<u8>, mix: SeatMix) -> EnvConfig {
        EnvConfig {
            players,
            mix,
            duplicate: false,
            simultaneous: true,
            log_rounds: false,
        }
    }

    /// Random legal actions for every pending decision; returns how many were frozen-net turns.
    fn run(env: &mut VecEnv, steps: usize, rng: &mut Rng) -> usize {
        let b = env.len();
        let mut obs = vec![0.0; b * FEATURES];
        let mut mask = vec![false; b * ACTIONS];
        let mut owner = vec![0u16; b];
        let mut net_turns = 0;
        for _ in 0..steps {
            env.observe_into(&mut obs, &mut mask, &mut owner);
            net_turns += owner.iter().filter(|&&o| o > 0).count();
            assert!(owner.iter().all(|&o| o <= env.nets()));
            let acts: Vec<usize> = (0..b)
                .map(|i| {
                    let legal: Vec<usize> =
                        (0..ACTIONS).filter(|&a| mask[i * ACTIONS + a]).collect();
                    assert!(!legal.is_empty());
                    legal[rng.below(legal.len() as u64) as usize]
                })
                .collect();
            env.step(&acts).unwrap();
        }
        net_turns
    }

    #[test]
    fn self_play_samples_carry_the_round_score() {
        let mut env = VecEnv::new(16, cfg(vec![3, 4, 5, 6], SeatMix::SELF_PLAY), 1).unwrap();
        let mut rng = Rng::new(2);
        run(&mut env, 2000, &mut rng);
        let s = env.drain();
        let stats = env.take_stats();
        assert!(stats.rounds > 50);
        assert_eq!(stats.other_rounds, 0);
        assert_eq!(s.obs.len(), s.len() * FEATURES);
        assert_eq!(s.returns.len(), s.len());
        assert_eq!(s.made.len(), s.len());
        assert!(s.len() as u64 <= stats.decisions);
        // Returns are real Wizard scores, and agree with the made-bid flag.
        for k in 0..s.len() {
            let r = s.returns[k] as i32;
            assert!(r % 10 == 0 && !(0..20).contains(&r), "{r}");
            assert_eq!(s.made[k] == 1.0, r >= 20);
        }
        // The stored legal mask includes the chosen action.
        assert_eq!(s.legal.len(), s.len() * ACTIONS);
        for k in 0..s.len() {
            assert!(s.legal[k * ACTIONS + s.actions[k] as usize]);
        }
        // The chosen action was legal for the observation it's paired with.
        for k in 0..s.len() {
            let o = &s.obs[k * FEATURES..(k + 1) * FEATURES];
            let a = s.actions[k] as usize;
            if a >= encode::ACT_CARD {
                assert_eq!(o[encode::HAND + a - encode::ACT_CARD], 1.0);
                assert_eq!(o[encode::PHASE + 2], 1.0);
            } else if a >= encode::ACT_BID {
                assert!((a - encode::ACT_BID) as f32 <= o[encode::SIZE] * 20.0 + 1e-4);
                assert_eq!(o[encode::PHASE + 1], 1.0);
            } else {
                assert_eq!(o[encode::PHASE], 1.0);
            }
        }
    }

    #[test]
    fn versus_mode_has_one_learner_seat() {
        let mut env = VecEnv::new(8, cfg(vec![4], SeatMix::COUNTING), 3).unwrap();
        let mut rng = Rng::new(4);
        run(&mut env, 3000, &mut rng);
        let st = env.take_stats();
        assert_eq!(st.other_rounds, 3 * st.learner_rounds);
        assert_eq!(st.learner_rounds, st.rounds);
        // A random learner should do much worse than the counting bots.
        assert!(
            (st.learner_score as f64 / st.learner_rounds as f64)
                < (st.other_score as f64 / st.other_rounds as f64)
        );
    }

    #[test]
    fn frozen_networks_get_turns_but_no_samples() {
        let mut env = VecEnv::new(8, cfg(vec![4], SeatMix::NETS), 6).unwrap();
        let mut rng = Rng::new(7);
        // No networks registered yet: their seats go to the learner.
        assert_eq!(run(&mut env, 200, &mut rng), 0);
        let before = env.take_stats();
        let mut samples = env.drain().len();
        env.set_nets(3);
        let steps = 3000;
        let net_turns = run(&mut env, steps, &mut rng);
        assert!(net_turns > 1000);
        let st = env.take_stats();
        samples += env.drain().len();
        // Only learner decisions become samples: every step was either a learner decision
        // (counted) or a frozen network's turn (not a sample).
        assert_eq!(
            before.decisions + st.decisions + net_turns as u64,
            (200 + steps) as u64 * 8
        );
        assert!(samples as u64 <= before.decisions + st.decisions);
        assert!(st.other_rounds > 0 && st.learner_rounds > 0);
    }

    #[test]
    fn duplicate_deals_rotate_the_learner_through_every_seat() {
        let mut env = VecEnv::new(
            1,
            EnvConfig {
                players: vec![4],
                mix: SeatMix::COUNTING,
                duplicate: true,
                simultaneous: true,
                log_rounds: false,
            },
            11,
        )
        .unwrap();
        let mut seen = Vec::new();
        let mut rng = Rng::new(1);
        let mut obs = vec![0.0; FEATURES];
        let mut mask = vec![false; ACTIONS];
        let mut owner = vec![0u16; 1];
        let mut last_hands: Option<Vec<u64>> = None;
        for _ in 0..4 {
            // Record this replay's deal and learner seat, then finish the round.
            let t = &env.tables[0];
            let hands: Vec<u64> = (0..4).map(|s| t.round.hand(s)).collect();
            if let Some(h) = &last_hands {
                assert_eq!(h, &hands, "same cards in every replay");
            }
            last_hands = Some(hands);
            seen.push(
                t.seats
                    .iter()
                    .take(4)
                    .position(|&s| s == Seat::Learner)
                    .unwrap(),
            );
            let done = |e: &VecEnv| e.stats.rounds + e.tables[0].pending.rounds;
            let before = done(&env);
            while done(&env) == before {
                env.observe_into(&mut obs, &mut mask, &mut owner);
                let legal: Vec<usize> = (0..ACTIONS).filter(|&a| mask[a]).collect();
                env.step(&[legal[rng.below(legal.len() as u64) as usize]])
                    .unwrap();
            }
        }
        assert_eq!(seen, vec![0, 1, 2, 3]);
        // The cycle is complete, so its results have been counted.
        assert_eq!(env.take_stats().rounds, 4);
        // A mixed field can't be duplicated.
        assert!(VecEnv::new(
            1,
            EnvConfig {
                players: vec![4],
                mix: SeatMix::preset("train").unwrap(),
                duplicate: true,
                simultaneous: true,
                log_rounds: false,
            },
            1
        )
        .is_err());
    }

    #[test]
    fn round_log_records_what_was_dealt_and_won() {
        let mut env = VecEnv::new(
            4,
            EnvConfig {
                players: vec![3, 5],
                mix: SeatMix::SELF_PLAY,
                duplicate: false,
                simultaneous: true,
                log_rounds: true,
            },
            21,
        )
        .unwrap();
        let mut rng = Rng::new(2);
        run(&mut env, 1500, &mut rng);
        let log = env.drain_rounds();
        assert!(log.len() > 100);
        for r in &log {
            assert!(r.players == 3 || r.players == 5);
            assert_eq!(r.hand.count_ones(), r.size as u32);
            assert!(r.won <= r.size && r.bid <= r.size);
            assert!((1..=r.players).contains(&r.position));
            assert!(r.trump <= 4);
            assert!(r.learner);
        }
        // Each round's seats hold different cards, and tricks add up to the round size.
        let mut i = 0;
        while i < log.len() {
            let n = log[i].players as usize;
            let seats = &log[i..i + n];
            assert_eq!(
                seats.iter().map(|r| r.won as u32).sum::<u32>(),
                seats[0].size as u32
            );
            let all = seats.iter().fold(0u64, |m, r| {
                assert_eq!(m & r.hand, 0);
                m | r.hand
            });
            assert_eq!(all.count_ones(), n as u32 * seats[0].size as u32);
            let mut pos: Vec<u8> = seats.iter().map(|r| r.position).collect();
            pos.sort();
            assert_eq!(pos, (1..=n as u8).collect::<Vec<_>>());
            i += n;
        }
    }

    #[test]
    fn duplicate_edge_of_a_player_against_itself_is_exactly_zero() {
        // The learner seat plays like a counting bot here (we pick its moves with the same
        // bot), so on duplicate deals its edge must be exactly zero.
        let mut env = VecEnv::new(
            8,
            EnvConfig {
                players: vec![3, 4, 5, 6],
                mix: SeatMix::COUNTING,
                duplicate: true,
                simultaneous: true,
                log_rounds: false,
            },
            13,
        )
        .unwrap();
        for _ in 0..4000 {
            let acts: Vec<usize> = (0..env.len())
                .map(|i| {
                    let t = &mut env.tables[i];
                    let seat = t.round.to_act().unwrap();
                    let v = View::new(&t.round, seat, &[]);
                    encode::action_index(CountingBot.act(&v, &mut t.rng))
                })
                .collect();
            env.step(&acts).unwrap();
        }
        let st = env.take_stats();
        assert!(st.learner_rounds > 100);
        let edge = st.edge_sum / st.edge_rounds as f64;
        assert!(edge.abs() < 1e-9, "edge {edge}");
    }

    #[test]
    fn a_quota_counts_the_same_number_of_deals_at_every_table() {
        // Random play everywhere; 20 tables, 5 duplicate cycles each.
        let mut env = VecEnv::new(
            20,
            EnvConfig {
                players: vec![4],
                mix: SeatMix::RANDOM,
                duplicate: true,
                simultaneous: true,
                log_rounds: false,
            },
            3,
        )
        .unwrap();
        let mut rng = Rng::new(9);
        // Run a while first, so the quota starts with rounds in progress.
        let random_step = |env: &mut VecEnv, rng: &mut Rng| {
            let acts: Vec<usize> = (0..env.len())
                .map(|i| {
                    let t = &env.tables[i];
                    let v = View::new(&t.round, t.round.to_act().unwrap(), &[]);
                    encode::action_index(RandomBot.act(&v, rng))
                })
                .collect();
            env.step(&acts).unwrap();
        };
        for _ in 0..777 {
            random_step(&mut env, &mut rng);
        }
        env.set_quota(5);
        env.take_stats();
        assert_eq!(env.quota_left(), 20);
        let mut steps = 0;
        while env.quota_left() > 0 {
            random_step(&mut env, &mut rng);
            steps += 1;
            assert!(steps < 1_000_000);
        }
        // Keep playing: nothing more is counted.
        for _ in 0..500 {
            random_step(&mut env, &mut rng);
        }
        let st = env.take_stats();
        assert_eq!(
            st.rounds,
            20 * 5 * 4,
            "100 deals, each replayed in all 4 seats"
        );
        assert_eq!(st.edge_rounds, 100 * 4);
        env.set_quota(0);
        assert_eq!(env.quota_left(), 0);
    }

    #[test]
    fn illegal_actions_are_refused_and_change_nothing() {
        let mut env = VecEnv::new(2, cfg(vec![3], SeatMix::SELF_PLAY), 5).unwrap();
        let mut obs = vec![0.0; 2 * FEATURES];
        let mut mask = vec![false; 2 * ACTIONS];
        let mut owner = vec![0u16; 2];
        env.observe_into(&mut obs, &mut mask, &mut owner);
        let illegal = (0..ACTIONS).find(|&a| !mask[a]).unwrap();
        let legal = (0..ACTIONS).find(|&a| mask[ACTIONS + a]).unwrap();
        assert!(env.step(&[illegal, legal]).is_err());
        let mut obs2 = obs.clone();
        let mut mask2 = mask.clone();
        env.observe_into(&mut obs2, &mut mask2, &mut owner);
        assert_eq!(obs, obs2);
        assert_eq!(mask, mask2);
        assert!(env.step(&[0]).is_err(), "one action per table");
    }

    #[test]
    fn reproducible() {
        let mk = || VecEnv::new(4, cfg(vec![3, 6], SeatMix::RANDOM), 9).unwrap();
        let (mut a, mut b) = (mk(), mk());
        run(&mut a, 500, &mut Rng::new(1));
        run(&mut b, 500, &mut Rng::new(1));
        assert_eq!(a.drain().returns, b.drain().returns);
    }

    #[test]
    fn seat_mix_draws_in_proportion() {
        let mix = SeatMix::preset("train").unwrap();
        let mut rng = Rng::new(3);
        let mut c = [0u32; 4];
        for _ in 0..40_000 {
            match mix.draw(5, &mut rng) {
                Seat::Learner => c[0] += 1,
                Seat::Net(k) => {
                    assert!((1..=5).contains(&k));
                    c[1] += 1
                }
                Seat::Counting => c[2] += 1,
                Seat::Random => c[3] += 1,
            }
        }
        let f = |x: u32| x as f64 / 40_000.0;
        assert!(
            (f(c[0]) - 0.7).abs() < 0.01
                && (f(c[1]) - 0.2).abs() < 0.01
                && (f(c[2]) - 0.1).abs() < 0.01
        );
        assert_eq!(c[3], 0);
        // Without networks, their share goes to the learner.
        assert!((0..1000).all(|_| !matches!(mix.draw(0, &mut rng), Seat::Net(_))));
    }
}
