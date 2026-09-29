//! A small, fast, seedable generator (SplitMix64). Every deal is reproducible from its seed,
//! which the duplicate-deal evaluation and the tests rely on.

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n` (unbiased, by rejection).
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0);
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return x % n;
            }
        }
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            v.swap(i, j);
        }
    }

    /// A fresh generator whose stream doesn't overlap this one's in practice.
    pub fn fork(&mut self) -> Rng {
        Rng(self.next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_is_roughly_uniform() {
        let mut r = Rng::new(7);
        let mut counts = [0u32; 6];
        for _ in 0..60_000 {
            counts[r.below(6) as usize] += 1;
        }
        for c in counts {
            assert!((9_400..10_600).contains(&c), "{counts:?}");
        }
    }

    #[test]
    fn shuffle_is_a_permutation_and_seeded() {
        let mut a: Vec<u8> = (0..60).collect();
        let mut b = a.clone();
        Rng::new(42).shuffle(&mut a);
        Rng::new(42).shuffle(&mut b);
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort();
        assert_eq!(sorted, (0..60).collect::<Vec<u8>>());
        assert_ne!(a, (0..60).collect::<Vec<u8>>());
    }
}
