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
use crate::view::{SeatHistory, View};

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
    /// Play full games (rounds 1, 2, 3 ... cards, the deal moving left, scores adding up)
    /// instead of lone rounds of random size. Every decision is then rewarded with how the
    /// *game* went for its seat (see `win_weight`), and the network sees the game scores.
    pub game: bool,
    /// Game reward = 100 x (win_weight x win + (1 - win_weight) x share of opponents beaten),
    /// ties split; or, with `win_weight` = -1, the seat's final margin over the best other
    /// player, in points (a dense signal that still points at winning).
    /// ties split. 1.0 = only winning counts.
    pub win_weight: f32,
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
    /// Full games only (`EnvConfig::game`): games finished, and learner / other seat-games with
    /// their win shares (ties split). `edge_sum / edge_rounds` is then per game, in points.
    pub games: u64,
    pub learner_games: u64,
    pub learner_wins: f64,
    pub other_games: u64,
    pub other_wins: f64,
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
    /// Full games only: the game situation around each decision's round, `CONTEXT` numbers per
    /// sample (zeros for lone rounds): players, rounds left after this round, then the seat's
    /// margin over the best and the second-best other player before the round, and after it.
    /// Enough to reward a round by how much it changed the seat's chance of winning.
    pub ctx: Vec<f32>,
}

/// Numbers per sample in `Samples::ctx`.
pub const CONTEXT: usize = 6;

/// A seat's margins over the best and second-best other player (0 when there is no second).
fn margins(totals: &[i32], seat: usize) -> (f32, f32) {
    let mut others: Vec<i32> = (0..totals.len())
        .filter(|&o| o != seat)
        .map(|o| totals[o])
        .collect();
    others.sort_unstable_by(|a, b| b.cmp(a));
    let me = totals[seat];
    let d1 = (me - others[0]) as f32;
    let d2 = others.get(1).map_or(0.0, |&o| (me - o) as f32);
    (d1, d2)
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
    /// Full games: scores so far by seat, the cards' random stream for the rest of the game,
    /// and the learner's decisions this game (seat, action, aux, made its bid that round).
    totals: [i32; SEATS],
    history: [SeatHistory; SEATS],
    deal_rng: Rng,
    game_traj: Vec<(u8, u32, f32, f32, [f32; CONTEXT])>,
    game_obs: Vec<f32>,
    game_mask: Vec<bool>,
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
        if !(0.0..=1.0).contains(&cfg.win_weight) && cfg.win_weight != -1.0 {
            return Err("win_weight must be between 0 and 1 (or -1 for the margin reward)".into());
        }
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
                totals: [0; SEATS],
                history: [SeatHistory::default(); SEATS],
                deal_rng: Rng::new(0),
                game_traj: Vec::new(),
                game_obs: Vec::new(),
                game_mask: Vec::new(),
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

    /// Deal table `i` a new round (or the next replay of its deal, in duplicate mode). With
    /// `game`, this starts a new game (or the next replay of one): round 1, scores at zero.
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
        let size = if cfg.game {
            1
        } else {
            1 + rng.below(rules.rounds() as u64) as u8
        };
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
        t.totals = [0; SEATS];
        t.history = [SeatHistory::default(); SEATS];
        t.deal_rng = rng; // the rest of the game's cards (duplicate replays get the same ones)
        t.game_traj.clear();
        t.game_obs.clear();
        t.game_mask.clear();
    }

    /// Full games: deal table `i` the game's next round (one more card, the deal moving left).
    fn next_round(&mut self, i: usize) {
        let t = &mut self.tables[i];
        let rules = *t.round.rules();
        let n = rules.players;
        let size = t.round.size() + 1;
        let dealer = (t.round.dealer() + 1) % n;
        t.round = Round::deal(rules, size, dealer, &mut t.deal_rng);
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
                    if self.cfg.game {
                        // Held until the game ends, when the reward is known.
                        let np = n as usize;
                        let before = t.totals;
                        let mut after = t.totals;
                        for s in 0..np {
                            after[s] += scores[s];
                        }
                        let left = (t.round.rules().rounds() - t.round.size()) as f32;
                        for &(seat, action, aux) in &t.traj {
                            let (b1, b2) = margins(&before[..np], seat as usize);
                            let (a1, a2) = margins(&after[..np], seat as usize);
                            t.game_traj.push((
                                seat,
                                action,
                                aux,
                                made[seat as usize] as u8 as f32,
                                [n as f32, left, b1, b2, a1, a2],
                            ));
                        }
                        t.game_obs.extend_from_slice(&t.traj_obs);
                        t.game_mask.extend_from_slice(&t.traj_mask);
                    } else {
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
                            self.out.ctx.extend_from_slice(&[0.0; CONTEXT]);
                        }
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
                    if ln > 0 && on > 0 && !self.cfg.game {
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
                    if self.cfg.game {
                        for s in 0..n as usize {
                            t.totals[s] += scores[s];
                            t.history[s].record(&t.round, s as u8);
                        }
                        if t.round.size() < t.round.rules().rounds() {
                            self.next_round(i);
                            continue;
                        }
                        // Game over: reward every decision of the game, and count the result.
                        let reward = game_rewards(&t.totals[..n as usize], self.cfg.win_weight);
                        for (k, &(seat, action, aux, made, ctx)) in t.game_traj.iter().enumerate() {
                            self.out.ctx.extend_from_slice(&ctx);
                            self.out
                                .obs
                                .extend_from_slice(&t.game_obs[k * FEATURES..(k + 1) * FEATURES]);
                            self.out.actions.push(action);
                            self.out.returns.push(reward[seat as usize]);
                            self.out.made.push(made);
                            self.out.aux.push(aux);
                            self.out
                                .legal
                                .extend_from_slice(&t.game_mask[k * ACTIONS..(k + 1) * ACTIONS]);
                        }
                        t.game_traj.clear();
                        t.game_obs.clear();
                        t.game_mask.clear();
                        let wins = win_shares(&t.totals[..n as usize]);
                        let st = &mut t.pending;
                        st.games += 1;
                        let (mut lt, mut ln, mut ot, mut on) = (0f64, 0f64, 0f64, 0f64);
                        for s in 0..n as usize {
                            if t.seats[s] == Seat::Learner {
                                st.learner_games += 1;
                                st.learner_wins += wins[s];
                                lt += t.totals[s] as f64;
                                ln += 1.0;
                            } else {
                                st.other_games += 1;
                                st.other_wins += wins[s];
                                ot += t.totals[s] as f64;
                                on += 1.0;
                            }
                        }
                        if ln > 0.0 && on > 0.0 {
                            st.edge_sum += lt / ln - ot / on;
                            st.edge_rounds += 1;
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
                        self.stats.games += p.games;
                        self.stats.learner_games += p.learner_games;
                        self.stats.learner_wins += p.learner_wins;
                        self.stats.other_games += p.other_games;
                        self.stats.other_wins += p.other_wins;
                    }
                    self.redeal(i);
                }
                Some(seat) => match t.seats[seat as usize] {
                    Seat::Learner | Seat::Net(_) => {
                        let np = t.round.players() as usize;
                        let (scores, hist): (&[i32], &[SeatHistory]) = if self.cfg.game {
                            (&t.totals[..np], &t.history[..np])
                        } else {
                            (&[], &[])
                        };
                        let v = View::with_history(&t.round, seat, scores, hist);
                        encode::observe(&v, &mut t.obs);
                        encode::legal_mask(&v, &mut t.mask);
                        return;
                    }
                    kind => {
                        let a = {
                            let np = t.round.players() as usize;
                            let (scores, hist): (&[i32], &[SeatHistory]) = if self.cfg.game {
                                (&t.totals[..np], &t.history[..np])
                            } else {
                                (&[], &[])
                            };
                            let v = View::with_history(&t.round, seat, scores, hist);
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

/// Each seat's share of the win (1 for a clear winner, split on a tie, 0 otherwise).
fn win_shares(totals: &[i32]) -> Vec<f64> {
    let best = *totals.iter().max().expect("players");
    let tied = totals.iter().filter(|&&t| t == best).count() as f64;
    totals
        .iter()
        .map(|&t| if t == best { 1.0 / tied } else { 0.0 })
        .collect()
}

/// The game reward per seat, in points (0 to 100): `win_weight` x winning plus the rest x the
/// share of opponents finished ahead of (ties count half).
fn game_rewards(totals: &[i32], win_weight: f32) -> Vec<f32> {
    let n = totals.len();
    if win_weight == -1.0 {
        // margin over the best other player
        return (0..n)
            .map(|s| {
                let best_other = (0..n)
                    .filter(|&o| o != s)
                    .map(|o| totals[o])
                    .max()
                    .unwrap_or(0);
                (totals[s] - best_other) as f32
            })
            .collect();
    }
    let wins = win_shares(totals);
    (0..n)
        .map(|s| {
            let beaten: f64 = (0..n)
                .filter(|&o| o != s)
                .map(|o| match totals[s].cmp(&totals[o]) {
                    std::cmp::Ordering::Greater => 1.0,
                    std::cmp::Ordering::Equal => 0.5,
                    std::cmp::Ordering::Less => 0.0,
                })
                .sum();
            let place = beaten / (n - 1) as f64;
            (100.0 * (win_weight as f64 * wins[s] + (1.0 - win_weight as f64) * place)) as f32
        })
        .collect()
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
            game: false,
            win_weight: 1.0,
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
                game: false,
                win_weight: 1.0,
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
                game: false,
                win_weight: 1.0,
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
                game: false,
                win_weight: 1.0,
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
                game: false,
                win_weight: 1.0,
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
                game: false,
                win_weight: 1.0,
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
    fn full_games_reward_every_decision_with_the_game_result() {
        let mut env = VecEnv::new(
            1,
            EnvConfig {
                players: vec![4],
                mix: SeatMix::SELF_PLAY,
                duplicate: false,
                simultaneous: true,
                log_rounds: true,
                game: true,
                win_weight: 1.0,
            },
            21,
        )
        .unwrap();
        let mut rng = Rng::new(4);
        run(&mut env, 8000, &mut rng);
        let st = env.take_stats();
        let s = env.drain();
        let log = env.drain_rounds();
        assert!(st.games >= 4, "{} games", st.games);
        assert_eq!(
            st.rounds,
            15 * st.games + (st.rounds % 15),
            "rounds come in games of 15"
        );
        // Win shares add up to one per game; with win_weight 1 a return is 100 / (winners) or 0.
        assert!((st.learner_wins + st.other_wins - st.games as f64).abs() < 1e-9);
        assert!(s.len() > 0);
        for &r in &s.returns {
            assert!(
                r == 0.0
                    || [100.0, 50.0, 100.0 / 3.0, 25.0]
                        .iter()
                        .any(|w| (r - w).abs() < 1e-3),
                "{r}"
            );
        }
        // Rounds are played in order, 1 to 15 cards.
        let sizes: Vec<u8> = log.iter().step_by(4).map(|r| r.size).take(15).collect();
        assert_eq!(sizes, (1..=15).collect::<Vec<u8>>());
    }

    #[test]
    fn game_rewards_split_ties_and_count_opponents_beaten() {
        let r = game_rewards(&[100, 50, 100, 0], 0.5);
        // Seats 0 and 2 share the win (0.5 each) and beat 1.5 of 3 opponents... plus the tie.
        assert!((r[0] - 100.0 * (0.5 * 0.5 + 0.5 * (2.5 / 3.0)) as f32).abs() < 1e-3);
        assert_eq!(r[0], r[2]);
        assert!((r[1] - 100.0 * 0.5 * (1.0 / 3.0) as f32).abs() < 1e-3);
        assert_eq!(r[3], 0.0);
        assert_eq!(game_rewards(&[10, 20, 30], 1.0), vec![0.0, 0.0, 100.0]);
        assert_eq!(game_rewards(&[10, 20, 30], -1.0), vec![-20.0, -10.0, 10.0]);
    }

    #[test]
    fn duplicate_games_of_a_player_against_itself_have_zero_edge() {
        let mut env = VecEnv::new(
            6,
            EnvConfig {
                players: vec![4],
                mix: SeatMix::COUNTING,
                duplicate: true,
                simultaneous: true,
                log_rounds: false,
                game: true,
                win_weight: 1.0,
            },
            8,
        )
        .unwrap();
        for _ in 0..20000 {
            let acts: Vec<usize> = (0..env.len())
                .map(|i| {
                    let t = &mut env.tables[i];
                    let seat = t.round.to_act().unwrap();
                    let v = View::new(&t.round, seat, &t.totals[..t.round.players() as usize]);
                    encode::action_index(CountingBot.act(&v, &mut t.rng))
                })
                .collect();
            env.step(&acts).unwrap();
        }
        let st = env.take_stats();
        assert!(st.edge_rounds > 0, "some games finished");
        assert!((st.edge_sum / st.edge_rounds as f64).abs() < 1e-9);
        let lw = st.learner_wins / st.learner_games as f64;
        let ow = st.other_wins / st.other_games as f64;
        assert!((lw - ow).abs() < 1e-9, "{lw} {ow}");
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
