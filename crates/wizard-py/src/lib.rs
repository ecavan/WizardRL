//! `wizard_rl._engine`: the Rust Wizard engine as a batch of tables Python can drive.

use numpy::{PyArray1, PyArray2, PyArrayMethods, PyReadonlyArray1};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use wizard::encode::{self, ACTIONS, FEATURES};
use wizard::env::{EnvConfig, SeatMix, VecEnv};
use wizard::net::Mlp;

type Obs<'py> = Bound<'py, PyArray2<f32>>;
type Legal<'py> = Bound<'py, PyArray2<bool>>;
type Vec1<'py, T> = Bound<'py, PyArray1<T>>;

/// A batch of Wizard tables. Each has one pending decision for Python at all times: the
/// learner's, or a frozen network's.
///
/// - `WizardEnv(num_tables, players=[3,4,5,6], opponents="selfplay", seed=0, duplicate=False)`;
///   `opponents` is a preset (`selfplay`, `train`, `counting`, `random`, `nets`) or a dict of
///   weights `{learner, nets, counting, random}` for the seats other than the learner's own
/// - `observe()` -> `(obs[float32, B x FEATURES], legal[bool, B x ACTIONS], owner[int64, B])`,
///   owner 0 = the learner, k = frozen network k
/// - `step(actions[int64, B])` applies one action index per table
/// - `drain()` -> `(obs[N x FEATURES], actions[N], returns[N], made[N])`: the learner's
///   decisions from finished rounds, with its round score and whether it made its bid
/// - `set_nets(n)`: frozen networks 1..=n may be dealt into seats from now on
/// - `stats()` -> dict of totals since the last call
#[pyclass(unsendable)]
struct WizardEnv {
    inner: VecEnv,
    obs: Vec<f32>,
    mask: Vec<bool>,
    owner: Vec<u16>,
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
    #[pyo3(signature = (num_tables, players = vec![3, 4, 5, 6], opponents = None, seed = 0, duplicate = false))]
    fn new(
        num_tables: usize,
        players: Vec<u8>,
        opponents: Option<&Bound<'_, PyAny>>,
        seed: u64,
        duplicate: bool,
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
            },
            seed,
        )
        .map_err(PyValueError::new_err)?;
        Ok(WizardEnv {
            obs: vec![0.0; num_tables * FEATURES],
            mask: vec![false; num_tables * ACTIONS],
            owner: vec![0; num_tables],
            inner,
        })
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn set_nets(&mut self, n: u16) {
        self.inner.set_nets(n);
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
    ) -> PyResult<(Obs<'py>, Vec1<'py, i64>, Vec1<'py, f32>, Vec1<'py, f32>)> {
        let s = self.inner.drain();
        let n = s.len();
        let obs = PyArray1::from_vec(py, s.obs).reshape([n, FEATURES])?;
        let acts = PyArray1::from_vec(py, s.actions.into_iter().map(|a| a as i64).collect());
        let rets = PyArray1::from_vec(py, s.returns);
        let made = PyArray1::from_vec(py, s.made);
        Ok((obs, acts, rets, made))
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
