use std::ops::RangeInclusive;

/// Seeded random numbers (PCG32).
///
/// Implemented here rather than taken from a crate, so a seed renders the same video on every
/// platform and with every version of our dependencies.
#[derive(Clone, Debug)]
pub(crate) struct Rng {
    state: u64,
}

const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const INCREMENT: u64 = 1_442_695_040_888_963_407;

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut rng = Self { state: 0 };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rotation = (old >> 59) as u32;
        xorshifted.rotate_right(rotation)
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        f64::from(self.next_u32()) / (f64::from(u32::MAX) + 1.0)
    }

    pub fn uniform(&mut self, range: RangeInclusive<f64>) -> f64 {
        range.start() + self.unit() * (range.end() - range.start())
    }

    /// Uniform integer in `range`, both ends included.
    pub fn int(&mut self, range: RangeInclusive<i32>) -> i32 {
        let (start, end) = range.into_inner();
        debug_assert!(start <= end);
        let span = u64::from(end.abs_diff(start)) + 1;
        start + (u64::from(self.next_u32()) % span) as i32
    }

    /// The lowest of `draws` picks from `range`: the more draws, the stronger the pull toward
    /// the start of the range.
    pub fn int_biased_low(&mut self, range: RangeInclusive<i32>, draws: u32) -> i32 {
        (0..draws.max(1)).map(|_| self.int(range.clone())).min().unwrap_or(*range.start())
    }

    /// Uniform index below `len`, which must not be 0.
    pub fn index(&mut self, len: usize) -> usize {
        debug_assert!(len > 0);
        (u64::from(self.next_u32()) % len as u64) as usize
    }

    pub fn chance(&mut self, probability: f64) -> bool {
        self.unit() < probability
    }

    pub fn one_in(&mut self, n: u32) -> bool {
        self.next_u32() % n == 0
    }

    /// `1.0` or `-1.0`.
    pub fn sign(&mut self) -> f64 {
        if self.one_in(2) { 1.0 } else { -1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_same_sequence() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        assert!((0..100).all(|_| a.next_u32() == b.next_u32()));
    }

    #[test]
    fn int_stays_inside_inclusive_range_and_reaches_both_ends() {
        let mut rng = Rng::new(1);
        let picks: Vec<i32> = (0..1000).map(|_| rng.int(-3..=3)).collect();
        assert!(picks.iter().all(|v| (-3..=3).contains(v)));
        assert!(picks.contains(&-3) && picks.contains(&3));
    }

    #[test]
    fn biased_low_prefers_the_start() {
        let mut rng = Rng::new(2);
        let mean = |rng: &mut Rng, draws| {
            (0..2000).map(|_| f64::from(rng.int_biased_low(0..=10, draws))).sum::<f64>() / 2000.0
        };
        assert!(mean(&mut rng, 10) < mean(&mut rng, 1));
    }
}
