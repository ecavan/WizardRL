//! A batch of Wizard tables for training: the learner (the Python side) decides for its seats
//! in every table at once; built-in bots play any other seats inside Rust.
//!
//! Each table plays single rounds (every round is scored on its own): random table size from
//! the config, random round size, random dealer. When a round ends, every decision a learner
//! seat made in it becomes a training sample labelled with that seat's round score.

use crate::bots::{Bot, CountingBot, RandomBot};
use crate::encode::{self, ACTIONS, FEATURES};
use crate::rng::Rng;
use crate::round::Round;
use crate::rules::{Rules, MAX_PLAYERS, MIN_PLAYERS};
use crate::view::View;

const SEATS: usize = MAX_PLAYERS as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opponents {
    /// The learner plays every seat.
    SelfPlay,
    /// The learner plays one random seat; counting bots play the rest.
    Counting,
    /// The learner plays one random seat; random bots play the rest.
    Random,
}

impl Opponents {
    pub fn parse(s: &str) -> Option<Opponents> {
        match s {
            "selfplay" | "self" => Some(Opponents::SelfPlay),
            "counting" => Some(Opponents::Counting),
            "random" => Some(Opponents::Random),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnvConfig {
    /// Table sizes to draw from, uniformly.
    pub players: Vec<u8>,
    pub opponents: Opponents,
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
    /// Bot seat-rounds and their scores (versus modes only).
    pub bot_rounds: u64,
    pub bot_score: i64,
    pub bot_bids_made: u64,
    /// Learner decisions taken.
    pub decisions: u64,
}

/// Training samples: `obs` is `len * FEATURES` long.
#[derive(Clone, Debug, Default)]
pub struct Samples {
    pub obs: Vec<f32>,
    pub actions: Vec<u32>,
    pub returns: Vec<f32>,
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
    learner: [bool; SEATS],
    rng: Rng,
    /// Observation and legal mask for the pending learner decision.
    obs: Vec<f32>,
    mask: Vec<bool>,
    /// This round's learner decisions: (seat, action), with observations in `traj_obs`.
    traj: Vec<(u8, u32)>,
    traj_obs: Vec<f32>,
}

pub struct VecEnv {
    cfg: EnvConfig,
    tables: Vec<Table>,
    out: Samples,
    stats: Stats,
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
        let mut seeder = Rng::new(seed);
        let mut env = VecEnv {
            cfg,
            tables: Vec::with_capacity(num_tables),
            out: Samples::default(),
            stats: Stats::default(),
        };
        for _ in 0..num_tables {
            let mut rng = seeder.fork();
            let (round, learner) = deal(&env.cfg, &mut rng);
            env.tables.push(Table {
                round,
                learner,
                rng,
                obs: vec![0.0; FEATURES],
                mask: vec![false; ACTIONS],
                traj: Vec::new(),
                traj_obs: Vec::new(),
            });
        }
        for i in 0..num_tables {
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

    /// Observations `[tables, FEATURES]` and legal masks `[tables, ACTIONS]` for every table's
    /// pending learner decision.
    pub fn observe_into(&self, obs: &mut [f32], mask: &mut [bool]) {
        assert_eq!(obs.len(), self.tables.len() * FEATURES);
        assert_eq!(mask.len(), self.tables.len() * ACTIONS);
        for (i, t) in self.tables.iter().enumerate() {
            obs[i * FEATURES..(i + 1) * FEATURES].copy_from_slice(&t.obs);
            mask[i * ACTIONS..(i + 1) * ACTIONS].copy_from_slice(&t.mask);
        }
    }

    /// Apply one action index per table (see `encode`). Illegal actions are an error and
    /// leave every table unchanged.
    pub fn step(&mut self, actions: &[usize]) -> Result<(), String> {
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
            t.traj.push((seat, a as u32));
            t.traj_obs.extend_from_slice(&t.obs);
            t.round
                .apply(encode::action_from_index(a).unwrap())
                .expect("checked legal");
            self.stats.decisions += 1;
            self.advance(i);
        }
        Ok(())
    }

    /// Take the samples from rounds finished since the last call.
    pub fn drain(&mut self) -> Samples {
        std::mem::take(&mut self.out)
    }

    pub fn take_stats(&mut self) -> Stats {
        std::mem::take(&mut self.stats)
    }

    /// Play bot seats and finished rounds until table `i` waits on a learner decision.
    fn advance(&mut self, i: usize) {
        loop {
            let t = &mut self.tables[i];
            match t.round.to_act() {
                None => {
                    let scores = t.round.scores().expect("round over");
                    let n = t.round.players();
                    for (k, &(seat, action)) in t.traj.iter().enumerate() {
                        self.out
                            .obs
                            .extend_from_slice(&t.traj_obs[k * FEATURES..(k + 1) * FEATURES]);
                        self.out.actions.push(action);
                        self.out.returns.push(scores[seat as usize] as f32);
                    }
                    t.traj.clear();
                    t.traj_obs.clear();
                    self.stats.rounds += 1;
                    for s in 0..n {
                        let made = (t.round.bid(s) == Some(t.round.tricks_won(s))) as u64;
                        if t.learner[s as usize] {
                            self.stats.learner_rounds += 1;
                            self.stats.learner_score += scores[s as usize] as i64;
                            self.stats.learner_bids_made += made;
                        } else {
                            self.stats.bot_rounds += 1;
                            self.stats.bot_score += scores[s as usize] as i64;
                            self.stats.bot_bids_made += made;
                        }
                    }
                    let (round, learner) = deal(&self.cfg, &mut t.rng);
                    t.round = round;
                    t.learner = learner;
                }
                Some(seat) if t.learner[seat as usize] => {
                    let v = View::new(&t.round, seat, &[]);
                    encode::observe(&v, &mut t.obs);
                    encode::legal_mask(&v, &mut t.mask);
                    return;
                }
                Some(seat) => {
                    let a = {
                        let v = View::new(&t.round, seat, &[]);
                        match self.cfg.opponents {
                            Opponents::Counting => CountingBot.act(&v, &mut t.rng),
                            Opponents::Random => RandomBot.act(&v, &mut t.rng),
                            Opponents::SelfPlay => unreachable!("every seat is the learner's"),
                        }
                    };
                    t.round.apply(a).expect("bots play legal moves");
                }
            }
        }
    }
}

fn deal(cfg: &EnvConfig, rng: &mut Rng) -> (Round, [bool; SEATS]) {
    let n = cfg.players[rng.below(cfg.players.len() as u64) as usize];
    let rules = Rules::official(n);
    let size = 1 + rng.below(rules.rounds() as u64) as u8;
    let dealer = rng.below(n as u64) as u8;
    let round = Round::deal(rules, size, dealer, rng);
    let mut learner = [false; SEATS];
    match cfg.opponents {
        Opponents::SelfPlay => learner[..n as usize].fill(true),
        _ => learner[rng.below(n as u64) as usize] = true,
    }
    (round, learner)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(env: &mut VecEnv, steps: usize, rng: &mut Rng) {
        let b = env.len();
        let mut obs = vec![0.0; b * FEATURES];
        let mut mask = vec![false; b * ACTIONS];
        for _ in 0..steps {
            env.observe_into(&mut obs, &mut mask);
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
    }

    #[test]
    fn self_play_samples_carry_the_round_score() {
        let mut env = VecEnv::new(
            16,
            EnvConfig {
                players: vec![3, 4, 5, 6],
                opponents: Opponents::SelfPlay,
            },
            1,
        )
        .unwrap();
        let mut rng = Rng::new(2);
        run(&mut env, 2000, &mut rng);
        let s = env.drain();
        let stats = env.take_stats();
        assert!(stats.rounds > 50);
        assert_eq!(stats.bot_rounds, 0);
        assert_eq!(s.obs.len(), s.len() * FEATURES);
        assert_eq!(s.returns.len(), s.len());
        // Every sample from a finished round; every decision eventually lands in a sample.
        assert!(s.len() as u64 <= stats.decisions);
        // Returns are real Wizard scores: 20 + 10k, or a negative multiple of 10.
        for &r in &s.returns {
            let r = r as i32;
            assert!(r % 10 == 0 && !(0..20).contains(&r), "{r}");
        }
        // The chosen action was legal for the observation it's paired with: a card must be in
        // the hand block; a bid must be at most the round size.
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
        let mut env = VecEnv::new(
            8,
            EnvConfig {
                players: vec![4],
                opponents: Opponents::Counting,
            },
            3,
        )
        .unwrap();
        let mut rng = Rng::new(4);
        run(&mut env, 3000, &mut rng);
        let st = env.take_stats();
        assert_eq!(st.bot_rounds, 3 * st.learner_rounds);
        assert_eq!(st.learner_rounds, st.rounds);
        // A random learner should do much worse than the counting bots.
        assert!(
            (st.learner_score as f64 / st.learner_rounds as f64)
                < (st.bot_score as f64 / st.bot_rounds as f64)
        );
    }

    #[test]
    fn illegal_actions_are_refused_and_change_nothing() {
        let mut env = VecEnv::new(
            2,
            EnvConfig {
                players: vec![3],
                opponents: Opponents::SelfPlay,
            },
            5,
        )
        .unwrap();
        let mut obs = vec![0.0; 2 * FEATURES];
        let mut mask = vec![false; 2 * ACTIONS];
        env.observe_into(&mut obs, &mut mask);
        let illegal = (0..ACTIONS).find(|&a| !mask[a]).unwrap();
        let legal = (0..ACTIONS).find(|&a| mask[ACTIONS + a]).unwrap();
        assert!(env.step(&[illegal, legal]).is_err());
        let mut obs2 = obs.clone();
        let mut mask2 = mask.clone();
        env.observe_into(&mut obs2, &mut mask2);
        assert_eq!(obs, obs2);
        assert_eq!(mask, mask2);
        assert!(env.step(&[0]).is_err(), "one action per table");
    }

    #[test]
    fn reproducible() {
        let mk = || {
            VecEnv::new(
                4,
                EnvConfig {
                    players: vec![3, 6],
                    opponents: Opponents::Random,
                },
                9,
            )
            .unwrap()
        };
        let (mut a, mut b) = (mk(), mk());
        run(&mut a, 500, &mut Rng::new(1));
        run(&mut b, 500, &mut Rng::new(1));
        assert_eq!(a.drain().returns, b.drain().returns);
    }
}
