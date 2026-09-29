//! Run a trained network inside Rust, so you can play against it (and later ship it in an
//! app) without Python.
//!
//! The file is written by `python -m wizard_rl.export`: a plain MLP, stored as
//!
//! ```text
//! b"WZNET001"  u32 features  u32 actions  f32 scale  u32 layers
//! per layer:   u32 in  u32 out  f32[out * in] weights (row-major)  f32[out] bias
//! ```
//!
//! all little-endian, with ReLU between layers. The first `actions` outputs times `scale` are
//! expected round scores in points; if there are `2 * actions` outputs, the rest are logits of the
//! chance the seat makes its bid after taking that action.

use crate::bots::Bot;
use crate::encode::{self, ACTIONS, FEATURES};
use crate::rng::Rng;
use crate::round::Action;
use crate::view::View;
use std::io::Read;
use std::path::Path;

const MAGIC: &[u8; 8] = b"WZNET001";

struct Layer {
    input: usize,
    output: usize,
    w: Vec<f32>,
    b: Vec<f32>,
}

pub struct Mlp {
    layers: Vec<Layer>,
    /// Multiply outputs by this to get points.
    pub scale: f32,
    /// Whether the network also predicts the chance of making the bid.
    pub make_head: bool,
}

fn read_u32(r: &mut impl Read) -> std::io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_f32s(r: &mut impl Read, n: usize) -> std::io::Result<Vec<f32>> {
    let mut bytes = vec![0u8; n * 4];
    r.read_exact(&mut bytes)?;
    Ok(bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

impl Mlp {
    pub fn load(path: impl AsRef<Path>) -> Result<Mlp, String> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Mlp::from_bytes(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Mlp, String> {
        let mut r = bytes;
        let mut magic = [0u8; 8];
        r.read_exact(&mut magic).map_err(|_| "too short")?;
        if &magic != MAGIC {
            return Err("not a Wizard network file (wrong header)".into());
        }
        let io = |e: std::io::Error| format!("truncated file: {e}");
        let features = read_u32(&mut r).map_err(io)? as usize;
        let actions = read_u32(&mut r).map_err(io)? as usize;
        if features != FEATURES || actions != ACTIONS {
            return Err(format!(
                "network is for {features} features / {actions} actions, this engine uses {FEATURES} / {ACTIONS}"
            ));
        }
        let scale = f32::from_bits(read_u32(&mut r).map_err(io)?);
        if !(scale.is_finite() && scale > 0.0) {
            return Err(format!("bad scale {scale}"));
        }
        let n = read_u32(&mut r).map_err(io)? as usize;
        let mut layers = Vec::with_capacity(n);
        let mut width = features;
        for i in 0..n {
            let input = read_u32(&mut r).map_err(io)? as usize;
            let output = read_u32(&mut r).map_err(io)? as usize;
            if input != width {
                return Err(format!(
                    "layer {i} expects {input} inputs, previous layer gives {width}"
                ));
            }
            let w = read_f32s(&mut r, input * output).map_err(io)?;
            let b = read_f32s(&mut r, output).map_err(io)?;
            layers.push(Layer {
                input,
                output,
                w,
                b,
            });
            width = output;
        }
        if width != actions && width != 2 * actions {
            return Err(format!(
                "last layer gives {width} outputs, expected {actions} or {}",
                2 * actions
            ));
        }
        if !r.is_empty() {
            return Err(format!("{} unexpected bytes at the end", r.len()));
        }
        Ok(Mlp {
            layers,
            scale,
            make_head: width == 2 * actions,
        })
    }

    /// Predicted round score (in the network's units) of every action.
    pub fn forward(&self, x: &[f32]) -> Vec<f32> {
        let mut cur = x.to_vec();
        let last = self.layers.len() - 1;
        for (k, l) in self.layers.iter().enumerate() {
            assert_eq!(cur.len(), l.input);
            let mut out = l.b.clone();
            for (o, acc) in out.iter_mut().enumerate() {
                let row = &l.w[o * l.input..(o + 1) * l.input];
                *acc += row.iter().zip(&cur).map(|(a, b)| a * b).sum::<f32>();
            }
            if k != last {
                for v in &mut out {
                    *v = v.max(0.0);
                }
            }
            debug_assert_eq!(out.len(), l.output);
            cur = out;
        }
        cur
    }
}

/// Plays the legal action with the highest predicted score.
pub struct NetBot {
    net: Mlp,
    name: String,
    obs: Vec<f32>,
    mask: Vec<bool>,
}

impl NetBot {
    pub fn new(net: Mlp, name: impl Into<String>) -> NetBot {
        NetBot {
            net,
            name: name.into(),
            obs: vec![0.0; FEATURES],
            mask: vec![false; ACTIONS],
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<NetBot, String> {
        let name = path
            .as_ref()
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "net".into());
        Ok(NetBot::new(Mlp::load(path)?, name))
    }

    /// Every legal action with its predicted round score (points) and, if the network has
    /// that head, the chance of making the bid; best first.
    pub fn values(&mut self, v: &View) -> Vec<(Action, f32, Option<f32>)> {
        encode::observe(v, &mut self.obs);
        encode::legal_mask(v, &mut self.mask);
        let q = self.net.forward(&self.obs);
        let head = self.net.make_head;
        let mut out: Vec<(Action, f32, Option<f32>)> = (0..ACTIONS)
            .filter(|&i| self.mask[i])
            .map(|i| {
                let p = head.then(|| 1.0 / (1.0 + (-q[ACTIONS + i]).exp()));
                (
                    encode::action_from_index(i).unwrap(),
                    q[i] * self.net.scale,
                    p,
                )
            })
            .collect();
        out.sort_by(|a, b| b.1.total_cmp(&a.1));
        out
    }
}

impl Bot for NetBot {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn act(&mut self, v: &View, _rng: &mut Rng) -> Action {
        self.values(v).first().expect("a legal action").0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny(features: usize, actions: usize) -> Vec<u8> {
        tiny_out(features, actions, actions)
    }

    fn tiny_out(features: usize, actions: usize, outputs: usize) -> Vec<u8> {
        let mut b = MAGIC.to_vec();
        b.extend((features as u32).to_le_bytes());
        b.extend((actions as u32).to_le_bytes());
        b.extend(100f32.to_le_bytes());
        b.extend(2u32.to_le_bytes());
        let push_layer = |b: &mut Vec<u8>, i: usize, o: usize, w: f32, bias: f32| {
            b.extend((i as u32).to_le_bytes());
            b.extend((o as u32).to_le_bytes());
            for _ in 0..i * o {
                b.extend(w.to_le_bytes());
            }
            for _ in 0..o {
                b.extend(bias.to_le_bytes());
            }
        };
        push_layer(&mut b, features, 3, 0.01, -0.5);
        push_layer(&mut b, 3, outputs, 1.0, 0.25);
        b
    }

    #[test]
    fn loads_and_runs() {
        let net = Mlp::from_bytes(&tiny(FEATURES, ACTIONS)).unwrap();
        let x = vec![1.0; FEATURES];
        // hidden = relu(0.01 * 503 - 0.5) = 4.53 each; out = 3 * 4.53 + 0.25
        let y = net.forward(&x);
        assert_eq!(y.len(), ACTIONS);
        assert!((y[0] - (3.0 * (0.01 * FEATURES as f32 - 0.5) + 0.25)).abs() < 1e-3);
        // ReLU on the hidden layer: an all-zero input gives relu(-0.5) = 0, so out = bias.
        assert!((net.forward(&vec![0.0; FEATURES])[7] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn make_head_is_optional() {
        assert!(!Mlp::from_bytes(&tiny(FEATURES, ACTIONS)).unwrap().make_head);
        let two = Mlp::from_bytes(&tiny_out(FEATURES, ACTIONS, 2 * ACTIONS)).unwrap();
        assert!(two.make_head);
        assert_eq!(two.forward(&vec![0.0; FEATURES]).len(), 2 * ACTIONS);
    }

    #[test]
    fn rejects_bad_files() {
        assert!(Mlp::from_bytes(&tiny_out(FEATURES, ACTIONS, ACTIONS + 1)).is_err());
        assert!(Mlp::from_bytes(b"nope").is_err());
        assert!(Mlp::from_bytes(&tiny(10, ACTIONS)).is_err());
        let mut b = tiny(FEATURES, ACTIONS);
        b.pop();
        assert!(Mlp::from_bytes(&b).is_err());
        let mut b = tiny(FEATURES, ACTIONS);
        b.push(0);
        assert!(Mlp::from_bytes(&b).is_err());
    }
}
