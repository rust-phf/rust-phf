//! A fixed-seed PRNG based on the wyrand algorithm.
//!
//! The focus is to provide a fast implementation that is usable in const
//! context, but not to be cryptographically secure by any means.

/// A tiny and fast pseudo-random number generator based on wyrand.
///
/// This must be initialized to a fixed seed which will be the
/// base for random number generation.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Rng {
    seed: u64,
}

impl Rng {
    /// Creates a new RNG given an initial seed.
    pub const fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Generates a pseudo-random [`u64`] value and alters the
    /// internal state.
    /// 
    /// This method may be called repeatedly on the same [`Rng`]
    /// instance to produce several random numbers.
    #[inline]
    pub const fn generate(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(0xa0761d6478bd642f);

        let t: u128 = (self.seed as u128).wrapping_mul((self.seed ^ 0xe7037ed1a0b428db) as u128);
        (t.wrapping_shr(64) ^ t) as u64
    }
}

// TODO: Implement the `Iterator` trait for `Rng` once all its provided methods
//       are decorated with `#[method_body_is_const]`. Before that, we'd have to
//       implement *all* Iterator methods by hand which would become very verbose
//       for mostly unneeded features. Thereby we will wait until we get away with
//       just providing a `next` implementation on our part.
