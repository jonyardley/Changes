//! Tiny deterministic PRNG (SplitMix64). First-party so exercise
//! generation is a pure function of its seed — no `rand` version drift in
//! replay tests, no wall-clock anywhere in core. Seeds arrive via events.

#[derive(Debug, Default)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..bound` (bound > 0).
    pub fn next_below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_a_seed() {
        let a: Vec<u64> = {
            let mut r = SplitMix64::new(42);
            (0..5).map(|_| r.next_u64()).collect()
        };
        let b: Vec<u64> = {
            let mut r = SplitMix64::new(42);
            (0..5).map(|_| r.next_u64()).collect()
        };
        assert_eq!(a, b);
        let c = SplitMix64::new(43).next_u64();
        assert_ne!(a[0], c);
    }

    #[test]
    fn next_below_stays_in_bounds() {
        let mut r = SplitMix64::new(7);
        for _ in 0..1000 {
            assert!(r.next_below(12) < 12);
        }
    }
}
