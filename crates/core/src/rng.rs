//! Seeded pseudo-random number generator (sfc32, seeded through splitmix32).
//!
//! The whole generator state is four `u32` words held in the game state, so it
//! round-trips through a save exactly. Same seed + same calls = same numbers.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Rng {
    s: [u32; 4],
}

/// FNV-1a over bytes. Used for text seeds, daily seeds and per-room sub-seeds.
pub fn hash_str(text: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in text.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Mix two words into one well-distributed word (for derived sub-seeds).
pub fn mix(a: u32, b: u32) -> u32 {
    let mut z = a ^ b.wrapping_mul(0x9e37_79b9).rotate_left(13);
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

impl Rng {
    /// A new generator from a 32-bit seed.
    pub fn from_seed(seed: u32) -> Self {
        let mut x = seed;
        let mut next = || {
            x = x.wrapping_add(0x9e37_79b9);
            let mut z = x;
            z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
            z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
            z ^ (z >> 16)
        };
        let mut s = [next(), next(), next(), next()];
        if s == [0; 4] {
            s[3] = 1;
        }
        let mut rng = Rng { s };
        // Discard the first outputs so nearby seeds diverge immediately.
        for _ in 0..12 {
            rng.next_u32();
        }
        rng
    }

    /// A generator for a fixed sub-stream, e.g. one room's loot.
    pub fn derived(seed: u32, salt: &str, index: u32) -> Self {
        Rng::from_seed(mix(mix(seed, hash_str(salt)), index))
    }

    /// False for the one state sfc32 must never hold.
    pub fn is_valid(&self) -> bool {
        self.s != [0; 4]
    }

    pub fn next_u32(&mut self) -> u32 {
        let [a, b, c, d] = self.s;
        let t = a.wrapping_add(b).wrapping_add(d);
        let d = d.wrapping_add(1);
        let a = b ^ (b >> 9);
        let b = c.wrapping_add(c << 3);
        let c = c.rotate_left(21).wrapping_add(t);
        self.s = [a, b, c, d];
        t
    }

    /// Uniform integer in `0..n`. Returns 0 when `n == 0`.
    pub fn below(&mut self, n: u32) -> u32 {
        if n <= 1 {
            // Still consume a draw so call sequences stay aligned.
            self.next_u32();
            return 0;
        }
        let limit = u32::MAX - (u32::MAX % n);
        loop {
            let x = self.next_u32();
            if x < limit {
                return x % n;
            }
        }
    }

    /// Uniform integer in `lo..=hi`; returns `lo` if the range is empty.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            self.next_u32();
            return lo;
        }
        let span = (i64::from(hi) - i64::from(lo) + 1).min(i64::from(u32::MAX)) as u32;
        lo + self.below(span) as i32
    }

    /// True with probability `pct`/100 (clamped). Always consumes one draw.
    pub fn chance(&mut self, pct: i32) -> bool {
        let p = pct.clamp(0, 100) as u32;
        self.below(100) < p
    }

    /// Index chosen in proportion to its weight; `None` if all weights are 0.
    pub fn weighted(&mut self, weights: &[u32]) -> Option<usize> {
        let total: u64 = weights.iter().map(|&w| u64::from(w)).sum();
        if total == 0 {
            return None;
        }
        let total = total.min(u64::from(u32::MAX)) as u32;
        let mut roll = self.below(total);
        for (i, &w) in weights.iter().enumerate() {
            if roll < w {
                return Some(i);
            }
            roll -= w;
        }
        weights.iter().rposition(|&w| w > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_divergent() {
        let mut a = Rng::from_seed(1234);
        let mut b = Rng::from_seed(1234);
        let mut c = Rng::from_seed(1235);
        let xs: Vec<u32> = (0..50).map(|_| a.next_u32()).collect();
        let ys: Vec<u32> = (0..50).map(|_| b.next_u32()).collect();
        let zs: Vec<u32> = (0..50).map(|_| c.next_u32()).collect();
        assert_eq!(xs, ys);
        assert_ne!(xs, zs);
    }

    #[test]
    fn pinned_sequence_matches_m1_algorithm() {
        // Same values the M1 TypeScript generator produced for seed 42.
        let mut r = Rng::from_seed(42);
        let seq: Vec<u32> = (0..4).map(|_| r.next_u32()).collect();
        assert_eq!(seq, vec![1028872839, 2516511472, 400437680, 853279530]);
    }

    #[test]
    fn ranges_and_chances() {
        let mut r = Rng::from_seed(7);
        let mut seen = [0u32; 6];
        for _ in 0..60_000 {
            let v = r.range(-2, 3);
            assert!((-2..=3).contains(&v));
            seen[(v + 2) as usize] += 1;
        }
        for c in seen {
            assert!((9_000..11_000).contains(&c));
        }
        for _ in 0..1000 {
            assert!(!r.chance(0));
            assert!(r.chance(100));
        }
        assert_eq!(r.range(4, 4), 4);
        assert_eq!(r.below(0), 0);
    }

    #[test]
    fn weighted_respects_zero() {
        let mut r = Rng::from_seed(11);
        let mut counts = [0u32; 3];
        for _ in 0..40_000 {
            counts[r.weighted(&[1, 3, 0]).unwrap()] += 1;
        }
        assert_eq!(counts[2], 0);
        assert!(counts[1] > counts[0] * 2);
        assert_eq!(r.weighted(&[0, 0]), None);
        assert_eq!(r.weighted(&[]), None);
    }

    #[test]
    fn serde_round_trip() {
        let mut a = Rng::from_seed(99);
        for _ in 0..17 {
            a.next_u32();
        }
        let json = serde_json::to_string(&a).unwrap();
        let mut b: Rng = serde_json::from_str(&json).unwrap();
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
        assert!(!serde_json::from_str::<Rng>("[0,0,0,0]").unwrap().is_valid());
    }
}
