#[derive(Clone, Copy)]
pub struct WyRand {
    seed: u64,
}

impl WyRand {
    /// Create a new [`WyRand`] instance, using a provided seed.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }
}

impl WyRand {
    pub const fn rand(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(0xa0761d6478bd642f);
        let t: u128 = (self.seed as u128).wrapping_mul((self.seed ^ 0xe7037ed1a0b428db) as u128);
        (t.wrapping_shr(64) ^ t) as u64
    }
}
