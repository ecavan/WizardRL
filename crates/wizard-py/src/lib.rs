//! `wizard_rl._engine`: the Rust Wizard engine as a batch of tables Python can drive.

use numpy::{PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray1};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use wizard::encode::{self, ACTIONS, FEATURES};
use wizard::env::{EnvConfig, SeatMix, VecEnv, CONTEXT};
use wizard::net::Mlp;
use wizard::rng::Rng;
use wizard::view::View;

type Obs<'py> = Bound<'py, PyArray2<f32>>;
type Legal<'py> = Bound<'py, PyArray2<bool>>;
type Vec1<'py, T> = Bound<'py, PyArray1<T>>;

/// A batch of Wizard tables. Each has one pending decision for Python at all times: the
/// learner's, or a frozen network's.
///
/// - `WizardEnv(num_tables, players=[3,4,5,6], opponents="selfplay", seed=0, duplicate=False,
///   simultaneous=True, log_rounds=False, game=False, win_weight=1.0)`; with `game`, tables play
///   full games and every decision's return is the game reward (0-100: winning, and with
///   `win_weight` < 1 partly the share of opponents beaten);
///   `opponents` is a preset (`selfplay`, `train`, `counting`, `random`, `nets`) or a dict of
///   weights `{learner, nets, counting, random}` for the seats other than the learner's own
/// - `observe()` -> `(obs[float32, B x FEATURES], legal[bool, B x ACTIONS], owner[int64, B])`,
///   owner 0 = the learner, k = frozen network k
/// - `step(actions[int64, B])` applies one action index per table
/// - `drain()` -> `(obs[N x FEATURES], actions[N], returns[N], made[N], aux[N], legal[N x ACTIONS])`: the learner's
///   decisions from finished rounds, with its round score, whether it made its bid, and the
///   value passed with the action to `step(actions, aux)`
/// - `set_nets(n)`: frozen networks 1..=n may be dealt into seats from now on
/// - `stats()` -> dict of totals since the last call
#[pyclass(unsendable)]
struct WizardEnv {
    inner: VecEnv,
    obs: Vec<f32>,
    mask: Vec<bool>,
    owner: Vec<u16>,
    last_ctx: Vec<f32>,
}

fn mix_from(obj: &Bound<'_, PyAny>) -> PyResult<SeatMix> {
    if let Ok(name) = obj.extract::<String>() {
        return SeatMix::preset(&name).ok_or_else(|| {
            PyValueError::new_err(
                "opponents: 'selfplay', 'train', 'counting', 'random', 'nets' or a dict",
            )
        });
    }
    let d = obj
        .cast::<PyDict>()
        .map_err(|_| PyValueError::new_err("opponents must be a preset name or a dict"))?;
    let get = |k: &str| -> PyResult<f64> {
        match d.get_item(k)? {
            Some(v) => v.extract::<f64>(),
            None => Ok(0.0),
        }
    };
    for k in d.keys() {
        let k: String = k.extract()?;
        if !["learner", "nets", "counting", "random"].contains(&k.as_str()) {
            return Err(PyValueError::new_err(format!("unknown seat kind '{k}'")));
        }
    }
    Ok(SeatMix {
        learner: get("learner")?,
        nets: get("nets")?,
        counting: get("counting")?,
        random: get("random")?,
    })
}

#[pymethods]
impl WizardEnv {
    #[new]
    #[pyo3(signature = (num_tables, players = vec![3, 4, 5, 6], opponents = None, seed = 0, duplicate = false, simultaneous = true, log_rounds = false, game = false, win_weight = 1.0))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        num_tables: usize,
        players: Vec<u8>,
        opponents: Option<&Bound<'_, PyAny>>,
        seed: u64,
        duplicate: bool,
        simultaneous: bool,
        log_rounds: bool,
        game: bool,
        win_weight: f32,
    ) -> PyResult<Self> {
        let mix = match opponents {
            Some(o) => mix_from(o)?,
            None => SeatMix::SELF_PLAY,
        };
        let inner = VecEnv::new(
            num_tables,
            EnvConfig {
                players,
                mix,
                duplicate,
                simultaneous,
                log_rounds,
                game,
                win_weight,
            },
            seed,
        )
        .map_err(PyValueError::new_err)?;
        Ok(WizardEnv {
            obs: vec![0.0; num_tables * FEATURES],
            mask: vec![false; num_tables * ACTIONS],
            owner: vec![0; num_tables],
            last_ctx: Vec::new(),
            inner,
        })
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn set_nets(&mut self, n: u16) {
        self.inner.set_nets(n);
    }

    /// Count only the next `deals` deals of every table in `stats()` (0 = no limit). Evaluate
    /// with a quota: stopping after "enough rounds" over-counts quick, small rounds.
    fn set_quota(&mut self, deals: u64) {
        self.inner.set_quota(deals);
    }

    /// Tables still short of their quota.
    fn quota_left(&self) -> usize {
        self.inner.quota_left()
    }

    fn observe<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<(Obs<'py>, Legal<'py>, Vec1<'py, i64>)> {
        self.inner
            .observe_into(&mut self.obs, &mut self.mask, &mut self.owner);
        let b = self.inner.len();
        let o = PyArray1::from_slice(py, &self.obs).reshape([b, FEATURES])?;
        let m = PyArray1::from_slice(py, &self.mask).reshape([b, ACTIONS])?;
        let w = PyArray1::from_vec(py, self.owner.iter().map(|&x| x as i64).collect());
        Ok((o, m, w))
    }

    #[pyo3(signature = (actions, aux = None))]
    fn step(
        &mut self,
        actions: PyReadonlyArray1<'_, i64>,
        aux: Option<PyReadonlyArray1<'_, f32>>,
    ) -> PyResult<()> {
        let a: Vec<usize> = actions
            .as_slice()?
            .iter()
            .map(|&x| usize::try_from(x).map_err(|_| PyValueError::new_err("negative action")))
            .collect::<PyResult<_>>()?;
        match aux {
            Some(x) => self.inner.step_with(&a, Some(x.as_slice()?)),
            None => self.inner.step(&a),
        }
        .map_err(PyValueError::new_err)
    }

    /// Finished rounds since the last call, one row per seat (needs `log_rounds=True`): a dict
    /// of arrays `players, size, trump (0-3 = clubs, diamonds, hearts, spades; 4 = none),
    /// position (1 = left of the dealer ... players = the dealer), hand (uint64 bitmask of the
    /// cards dealt), bid, won, learner`.
    fn drain_rounds<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let log = self.inner.drain_rounds();
        let d = PyDict::new(py);
        d.set_item(
            "players",
            PyArray1::from_vec(py, log.iter().map(|r| r.players).collect()),
        )?;
        d.set_item(
            "size",
            PyArray1::from_vec(py, log.iter().map(|r| r.size).collect()),
        )?;
        d.set_item(
            "trump",
            PyArray1::from_vec(py, log.iter().map(|r| r.trump).collect()),
        )?;
        d.set_item(
            "position",
            PyArray1::from_vec(py, log.iter().map(|r| r.position).collect()),
        )?;
        d.set_item(
            "hand",
            PyArray1::from_vec(py, log.iter().map(|r| r.hand).collect()),
        )?;
        d.set_item(
            "bid",
            PyArray1::from_vec(py, log.iter().map(|r| r.bid).collect()),
        )?;
        d.set_item(
            "won",
            PyArray1::from_vec(py, log.iter().map(|r| r.won).collect()),
        )?;
        d.set_item(
            "learner",
            PyArray1::from_vec(py, log.iter().map(|r| r.learner).collect()),
        )?;
        Ok(d)
    }

    #[allow(clippy::type_complexity)]
    fn drain<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<(
        Obs<'py>,
        Vec1<'py, i64>,
        Vec1<'py, f32>,
        Vec1<'py, f32>,
        Vec1<'py, f32>,
        Legal<'py>,
    )> {
        let s = self.inner.drain();
        let n = s.len();
        self.last_ctx = s.ctx;
        let obs = PyArray1::from_vec(py, s.obs).reshape([n, FEATURES])?;
        let acts = PyArray1::from_vec(py, s.actions.into_iter().map(|a| a as i64).collect());
        let rets = PyArray1::from_vec(py, s.returns);
        let made = PyArray1::from_vec(py, s.made);
        let aux = PyArray1::from_vec(py, s.aux);
        let legal = PyArray1::from_vec(py, s.legal).reshape([n, ACTIONS])?;
        Ok((obs, acts, rets, made, aux, legal))
    }

    /// The game context of the samples from the last `drain()` (`[N x 6]`: players, rounds left
    /// after the round, margin over the best and second-best other player before the round,
    /// and after it; zeros outside full games).
    fn last_context<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f32>>> {
        let n = self.last_ctx.len() / CONTEXT;
        PyArray1::from_vec(py, self.last_ctx.clone()).reshape([n, CONTEXT])
    }

    fn stats<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let s = self.inner.take_stats();
        let d = PyDict::new(py);
        d.set_item("rounds", s.rounds)?;
        d.set_item("learner_rounds", s.learner_rounds)?;
        d.set_item("learner_score", s.learner_score)?;
        d.set_item("learner_bids_made", s.learner_bids_made)?;
        d.set_item("other_rounds", s.other_rounds)?;
        d.set_item("other_score", s.other_score)?;
        d.set_item("other_bids_made", s.other_bids_made)?;
        d.set_item("decisions", s.decisions)?;
        d.set_item("edge_sum", s.edge_sum)?;
        d.set_item("edge_rounds", s.edge_rounds)?;
        d.set_item("games", s.games)?;
        d.set_item("learner_games", s.learner_games)?;
        d.set_item("learner_wins", s.learner_wins)?;
        d.set_item("other_games", s.other_games)?;
        d.set_item("other_wins", s.other_wins)?;
        Ok(d)
    }
}

/// Human-readable name of an action index: "trump hearts", "bid 2", "A♠".
#[pyfunction]
fn action_name(i: usize) -> PyResult<String> {
    encode::action_from_index(i)
        .map(|a| a.to_string())
        .ok_or_else(|| PyValueError::new_err("no such action"))
}

/// Run an exported network file (`wizard_rl.export`) in Rust on a batch of observations.
/// Returns the raw outputs `[B, outputs]`; used to check the export matches PyTorch.
#[pyfunction]
fn rust_forward<'py>(
    py: Python<'py>,
    path: &str,
    obs: numpy::PyReadonlyArray2<'py, f32>,
) -> PyResult<Bound<'py, PyArray2<f32>>> {
    let net = Mlp::load(path).map_err(PyValueError::new_err)?;
    let x = obs.as_array();
    if x.ncols() != FEATURES {
        return Err(PyValueError::new_err(format!(
            "expected {FEATURES} features"
        )));
    }
    let mut out = Vec::with_capacity(x.nrows() * 2 * ACTIONS);
    for row in x.rows() {
        let v: Vec<f32> = row.iter().copied().collect();
        out.extend(net.forward(&v));
    }
    let width = if x.nrows() == 0 {
        ACTIONS
    } else {
        out.len() / x.nrows()
    };
    PyArray1::from_vec(py, out).reshape([x.nrows(), width])
}

/// The observation and legal moves for a first-trick card-play situation (see
/// `wizard::scenario::play_scenario`): `bids` for every seat from the left of the dealer to the
/// dealer, `trick` the cards already played to this trick by positions 1, 2, ...
#[pyfunction]
#[pyo3(signature = (players, hand, trump, position, bids, trick, seed = 0, simultaneous = true))]
#[allow(clippy::too_many_arguments)]
fn play_scenario<'py>(
    py: Python<'py>,
    players: u8,
    hand: Vec<String>,
    trump: Option<String>,
    position: u8,
    bids: Vec<u8>,
    trick: Vec<String>,
    seed: u64,
    simultaneous: bool,
) -> PyResult<(Vec1<'py, f32>, Vec1<'py, bool>)> {
    let parse_all = |v: &[String]| {
        v.iter()
            .map(|s| wizard::card::parse(s).ok_or_else(|| PyValueError::new_err(format!("bad card '{s}'"))))
            .collect::<PyResult<Vec<_>>>()
    };
    let (cards, played) = (parse_all(&hand)?, parse_all(&trick)?);
    let trump = match trump.as_deref() {
        None | Some("none") | Some("") => None,
        Some(t) => Some(
            t.chars()
                .next()
                .and_then(wizard::card::Suit::from_char)
                .ok_or_else(|| PyValueError::new_err("trump must be c, d, h, s or None"))?,
        ),
    };
    let (round, me) = wizard::scenario::play_scenario(
        simultaneous,
        players,
        &cards,
        trump,
        position,
        &bids,
        &played,
        &mut Rng::new(seed),
    )
    .map_err(PyValueError::new_err)?;
    let v = View::new(&round, me, &[]);
    let mut obs = vec![0.0; FEATURES];
    let mut mask = vec![false; ACTIONS];
    encode::observe(&v, &mut obs);
    encode::legal_mask(&v, &mut mask);
    Ok((PyArray1::from_vec(py, obs), PyArray1::from_vec(py, mask)))
}

/// The observation and legal bids for a bidding situation: `players`, the bidder's `hand`
/// (e.g. `["7h", "10h", "wiz"]`), `trump` (`"h"`, ..., or `None` for no trump), `position` in the
/// seat order (1 = left of the dealer, `players` = the dealer) and, when bidding in turn
/// (`simultaneous=False`), the `bids_before` it. Other hands are dealt at random from `seed`.
#[pyfunction]
#[pyo3(signature = (players, hand, trump, position, bids_before = vec![], seed = 0, simultaneous = true))]
#[allow(clippy::too_many_arguments)]
fn bid_scenario<'py>(
    py: Python<'py>,
    players: u8,
    hand: Vec<String>,
    trump: Option<String>,
    position: u8,
    bids_before: Vec<u8>,
    seed: u64,
    simultaneous: bool,
) -> PyResult<(Vec1<'py, f32>, Vec1<'py, bool>)> {
    let cards = hand
        .iter()
        .map(|s| {
            wizard::card::parse(s).ok_or_else(|| PyValueError::new_err(format!("bad card '{s}'")))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let trump = match trump.as_deref() {
        None | Some("none") | Some("") => None,
        Some(t) => Some(
            t.chars()
                .next()
                .and_then(wizard::card::Suit::from_char)
                .ok_or_else(|| PyValueError::new_err("trump must be c, d, h, s or None"))?,
        ),
    };
    let (round, me) = wizard::scenario::bid_scenario(
        simultaneous,
        players,
        &cards,
        trump,
        position,
        &bids_before,
        &mut Rng::new(seed),
    )
    .map_err(PyValueError::new_err)?;
    let v = View::new(&round, me, &[]);
    let mut obs = vec![0.0; FEATURES];
    let mut mask = vec![false; ACTIONS];
    encode::observe(&v, &mut obs);
    encode::legal_mask(&v, &mut mask);
    Ok((PyArray1::from_vec(py, obs), PyArray1::from_vec(py, mask)))
}

#[pymodule]
fn _engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<WizardEnv>()?;
    m.add_function(wrap_pyfunction!(action_name, m)?)?;
    m.add_function(wrap_pyfunction!(rust_forward, m)?)?;
    m.add_function(wrap_pyfunction!(bid_scenario, m)?)?;
    m.add_function(wrap_pyfunction!(play_scenario, m)?)?;
    m.add("FEATURES", FEATURES)?;
    m.add("ACTIONS", ACTIONS)?;
    m.add("ACT_TRUMP", encode::ACT_TRUMP)?;
    m.add("ACT_BID", encode::ACT_BID)?;
    m.add("ACT_CARD", encode::ACT_CARD)?;
    m.add("PHASE", encode::PHASE)?;
    m.add("HAND", encode::HAND)?;
    m.add("SIZE", encode::SIZE)?;
    m.add("TRICKS_LEFT", encode::TRICKS_LEFT)?;
    m.add("HISTORY", encode::HISTORY)?;
    m.add("WIZARD_CARDS", (wizard::card::WIZARD_BASE as usize, wizard::card::JESTER_BASE as usize))?;
    m.add("GAME", encode::GAME)?;
    m.add("ROUND_FEATURES", encode::ROUND_FEATURES)?;
    Ok(())
}
