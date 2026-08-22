use core::num::NonZeroU64;

/// A nonzero modulus.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Modulus(NonZeroU64);

impl Modulus {
    /// Constructs a modulus, returning `None` for zero.
    #[inline]
    #[must_use]
    pub const fn new(m: u64) -> Option<Self> {
        match NonZeroU64::new(m) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the positive integer modulus.
    #[inline]
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }

    /// Returns whether this modulus divides `other`.
    #[inline]
    #[must_use]
    pub fn divides(self, other: Self) -> bool {
        other.get() % self.get() == 0
    }
}
