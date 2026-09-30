//! Chance of winning the game from the score situation: the small network fitted by
//! `python -m wizard_rl.winprob` and exported with `python -m wizard_rl.winprob export`.
//!
//! ```text
//! b"WZWINP01"  u32 layers
//! per layer:   u32 in  u32 out  f32[out * in] weights (row-major)  f32[out] bias
//! ```
//!
//! little-endian, ReLU between layers, one output logit. Inputs: players one-hot (3..6),
//! rounds left / 20, margin over the best other player / 100 and over the second best / 100
//! (both clamped to ±6).

use std::io::Read;
use std::path::Path;

const MAGIC: &[u8; 8] = b"WZWINP01";

struct Layer {
    input: usize,
    output: usize,
    w: Vec<f32>,
    b: Vec<f32>,
}

pub struct WinProb {
    layers: Vec<Layer>,
}

fn read_u32(r: &mut impl Read) -> Result<u32, String> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)
        .map_err(|e| format!("truncated: {e}"))?;
    Ok(u32::from_le_bytes(b))
}

fn read_f32s(r: &mut impl Read, n: usize) -> Result<Vec<f32>, String> {
    let mut bytes = vec![0u8; n * 4];
    r.read_exact(&mut bytes)
        .map_err(|e| format!("truncated: {e}"))?;
    Ok(bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

impl WinProb {
    pub fn load(path: impl AsRef<Path>) -> Result<WinProb, String> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        WinProb::from_bytes(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<WinProb, String> {
        let mut r = bytes;
        let mut magic = [0u8; 8];
        r.read_exact(&mut magic).map_err(|_| "too short")?;
        if &magic != MAGIC {
            return Err("not a win-probability file (wrong header)".into());
        }
        let n = read_u32(&mut r)? as usize;
        let mut layers = Vec::with_capacity(n);
        let mut prev = 7;
        for _ in 0..n {
            let input = read_u32(&mut r)? as usize;
            let output = read_u32(&mut r)? as usize;
            if input != prev || output == 0 || output > 4096 {
                return Err(format!("bad layer shape {input} -> {output}"));
            }
            let w = read_f32s(&mut r, input * output)?;
            let b = read_f32s(&mut r, output)?;
            layers.push(Layer {
                input,
                output,
                w,
                b,
            });
            prev = output;
        }
        if prev != 1 {
            return Err("the last layer must have one output".into());
        }
        Ok(WinProb { layers })
    }

    /// Chance of winning with `left` rounds still to play, `d1` points ahead of the best other
    /// player and `d2` ahead of the second best (negative = behind). With no rounds left it is
    /// the result itself (a tie for the lead counts half).
    pub fn prob(&self, players: u8, left: u32, d1: i32, d2: i32) -> f64 {
        if left == 0 {
            return match d1.cmp(&0) {
                std::cmp::Ordering::Greater => 1.0,
                std::cmp::Ordering::Equal => 0.5,
                std::cmp::Ordering::Less => 0.0,
            };
        }
        let mut x = vec![0f32; 7];
        if (3..=6).contains(&players) {
            x[(players - 3) as usize] = 1.0;
        }
        x[4] = left as f32 / 20.0;
        x[5] = (d1 as f32 / 100.0).clamp(-6.0, 6.0);
        x[6] = (d2 as f32 / 100.0).clamp(-6.0, 6.0);
        for (i, l) in self.layers.iter().enumerate() {
            let mut y = l.b.clone();
            for (o, yo) in y.iter_mut().enumerate() {
                let row = &l.w[o * l.input..(o + 1) * l.input];
                *yo += row.iter().zip(&x).map(|(a, b)| a * b).sum::<f32>();
                if i + 1 < self.layers.len() {
                    *yo = yo.max(0.0);
                }
            }
            debug_assert_eq!(y.len(), l.output);
            x = y;
        }
        1.0 / (1.0 + (-x[0] as f64).exp())
    }

    /// Everyone's chance of winning from these game totals, scaled to add up to 1.
    pub fn chances(&self, totals: &[i32], left: u32) -> Vec<f64> {
        let raw: Vec<f64> = (0..totals.len())
            .map(|s| {
                let (d1, d2) = margins(totals, s);
                self.prob(totals.len() as u8, left, d1, d2)
            })
            .collect();
        let sum: f64 = raw.iter().sum();
        if sum > 0.0 {
            raw.iter().map(|p| p / sum).collect()
        } else {
            raw
        }
    }
}

/// (margin over the best other player, over the second best other player) for `seat`.
pub fn margins(totals: &[i32], seat: usize) -> (i32, i32) {
    let mut others: Vec<i32> = totals
        .iter()
        .enumerate()
        .filter(|&(s, _)| s != seat)
        .map(|(_, &t)| t)
        .collect();
    others.sort_unstable_by(|a, b| b.cmp(a));
    let me = totals[seat];
    (
        me - others[0],
        me - others.get(1).copied().unwrap_or(others[0]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margins_and_final_result() {
        assert_eq!(margins(&[50, 80, 20, 60], 0), (-30, -10));
        assert_eq!(margins(&[50, 80, 20, 60], 1), (20, 30));
        let wp = WinProb {
            layers: vec![Layer {
                input: 7,
                output: 1,
                w: vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                b: vec![0.0],
            }],
        };
        assert_eq!(wp.prob(4, 0, 10, 20), 1.0);
        assert_eq!(wp.prob(4, 0, 0, 20), 0.5);
        assert!((wp.prob(4, 3, 0, 0) - 0.5).abs() < 1e-9);
        let c = wp.chances(&[100, 0, 0], 2);
        assert!((c.iter().sum::<f64>() - 1.0).abs() < 1e-9 && c[0] > c[1]);
    }
}
