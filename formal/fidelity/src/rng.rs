//! The random number generator every target's script generator draws from.

/// `SplitMix64`: small, deterministic, and good enough to pick test inputs.
#[derive(Debug)]
pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `low..=high`.
    pub(crate) fn range(&mut self, low: i64, high: i64) -> i64 {
        let span = u64::try_from(high - low + 1).unwrap_or(1);
        low + i64::try_from(self.next() % span).unwrap_or(0)
    }

    pub(crate) fn direction(&mut self) -> i64 {
        if self.next().is_multiple_of(2) { -1 } else { 1 }
    }

    pub(crate) fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        let index = usize::try_from(self.next()).unwrap_or(0) % items.len().max(1);
        items.get(index).copied()
    }
}
