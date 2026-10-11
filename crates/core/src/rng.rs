//! The seeded generator. Every random choice in a game comes from here, so a
//! seed reproduces a game exactly. The page supplies the seed; the core never
//! reads a clock or a random device.

/// SplitMix64: small, fast, and good enough for shuffling listings.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`. `n` must be at least 1.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// A number in `lo..=hi`.
    pub fn between(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo + 1) as u64) as i64
    }

    /// True with probability `permille` in a thousand.
    pub fn chance(&mut self, permille: u64) -> bool {
        self.below(1000) < permille
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_numbers() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn between_stays_inside_its_bounds_and_reaches_both() {
        let mut r = Rng::new(7);
        let mut seen_lo = false;
        let mut seen_hi = false;
        for _ in 0..2000 {
            let v = r.between(-3, 3);
            assert!((-3..=3).contains(&v));
            seen_lo |= v == -3;
            seen_hi |= v == 3;
        }
        assert!(seen_lo && seen_hi);
    }

    #[test]
    fn shuffle_keeps_every_element() {
        let mut r = Rng::new(9);
        let mut v: Vec<u32> = (0..20).collect();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0..20).collect::<Vec<u32>>());
    }
}
