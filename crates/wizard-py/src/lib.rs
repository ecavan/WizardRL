//! `wizard_rl._engine`: the Rust Wizard engine as a batch of tables Python can drive.

use numpy::{PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray1};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use wizard::encode::{self, ACTIONS, FEATURES};
use wizard::env::{EnvConfig, Opponents, VecEnv};
use wizard::net::Mlp;

type Obs<'py> = Bound<'py, PyArray2<f32>>;
type Legal<'py> = Bound<'py, PyArray2<bool>>;

/// A batch of Wizard tables. Each has one pending learner decision at all times.
///
/// - `observe()` -> `(obs[float32, B x FEATURES], legal[bool, B x ACTIONS])`
/// - `step(actions[int64, B])` applies one action index per table
/// - `drain()` -> `(obs[N x FEATURES], actions[N], returns[N])`: decisions from finished
///   rounds, each labelled with the round score its seat got
/// - `stats()` -> dict of totals since the last call
#[pyclass(unsendable)]
struct WizardEnv {
    inner: VecEnv,
    obs: Vec<f32>,
    mask: Vec<bool>,
}

#[pymethods]
impl WizardEnv {
    #[new]
    #[pyo3(signature = (num_tables, players = vec![3, 4, 5, 6], opponents = "selfplay", seed = 0))]
    fn new(num_tables: usize, players: Vec<u8>, opponents: &str, seed: u64) -> PyResult<Self> {
        let opponents = Opponents::parse(opponents).ok_or_else(|| {
            PyValueError::new_err("opponents must be 'selfplay', 'counting' or 'random'")
        })?;
        let inner = VecEnv::new(num_tables, EnvConfig { players, opponents }, seed)
            .map_err(PyValueError::new_err)?;
        Ok(WizardEnv {
            obs: vec![0.0; num_tables * FEATURES],
            mask: vec![false; num_tables * ACTIONS],
            inner,
        })
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn observe<'py>(&mut self, py: Python<'py>) -> PyResult<(Obs<'py>, Legal<'py>)> {
        self.inner.observe_into(&mut self.obs, &mut self.mask);
        let b = self.inner.len();
        let o = PyArray1::from_slice(py, &self.obs).reshape([b, FEATURES])?;
        let m = PyArray1::from_slice(py, &self.mask).reshape([b, ACTIONS])?;
        Ok((o, m))
    }

    fn step(&mut self, actions: PyReadonlyArray1<'_, i64>) -> PyResult<()> {
        let a: Vec<usize> = actions
            .as_slice()?
            .iter()
            .map(|&x| usize::try_from(x).map_err(|_| PyValueError::new_err("negative action")))
            .collect::<PyResult<_>>()?;
        self.inner.step(&a).map_err(PyValueError::new_err)
    }

    #[allow(clippy::type_complexity)]
    fn drain<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<(
        Bound<'py, PyArray2<f32>>,
        Bound<'py, PyArray1<i64>>,
        Bound<'py, PyArray1<f32>>,
    )> {
        let s = self.inner.drain();
        let n = s.len();
        let obs = PyArray1::from_vec(py, s.obs).reshape([n, FEATURES])?;
        let acts = PyArray1::from_vec(py, s.actions.into_iter().map(|a| a as i64).collect());
        let rets = PyArray1::from_vec(py, s.returns);
        Ok((obs, acts, rets))
    }

    fn stats<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let s = self.inner.take_stats();
        let d = PyDict::new(py);
        d.set_item("rounds", s.rounds)?;
        d.set_item("learner_rounds", s.learner_rounds)?;
        d.set_item("learner_score", s.learner_score)?;
        d.set_item("learner_bids_made", s.learner_bids_made)?;
        d.set_item("bot_rounds", s.bot_rounds)?;
        d.set_item("bot_score", s.bot_score)?;
        d.set_item("bot_bids_made", s.bot_bids_made)?;
        d.set_item("decisions", s.decisions)?;
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
/// Returns the raw outputs `[B, ACTIONS]`; used to check the export matches PyTorch.
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
    let mut out = Vec::with_capacity(x.nrows() * ACTIONS);
    for row in x.rows() {
        let v: Vec<f32> = row.iter().copied().collect();
        out.extend(net.forward(&v));
    }
    PyArray1::from_vec(py, out).reshape([x.nrows(), ACTIONS])
}

#[pymodule]
fn _engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<WizardEnv>()?;
    m.add_function(wrap_pyfunction!(action_name, m)?)?;
    m.add_function(wrap_pyfunction!(rust_forward, m)?)?;
    m.add("FEATURES", FEATURES)?;
    m.add("ACTIONS", ACTIONS)?;
    m.add("ACT_TRUMP", encode::ACT_TRUMP)?;
    m.add("ACT_BID", encode::ACT_BID)?;
    m.add("ACT_CARD", encode::ACT_CARD)?;
    m.add("PHASE", encode::PHASE)?;
    m.add("HAND", encode::HAND)?;
    Ok(())
}
