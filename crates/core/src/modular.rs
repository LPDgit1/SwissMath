use core::fmt;

use crate::Modulus;

/// Failure to represent an exact arithmetic result in the supported width.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArithmeticError {
    /// The exact result exceeds the supported integer range.
    Overflow,
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => f.write_str("exact arithmetic result exceeds u64"),
        }
    }
}

impl std::error::Error for ArithmeticError {}

/// Computes the greatest common divisor using Euclid's algorithm.
#[inline]
#[must_use]
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

/// Reduces a signed integer to its canonical residue.
#[inline]
#[must_use]
pub fn reduce_i128(value: i128, modulus: Modulus) -> u64 {
    value.rem_euclid(i128::from(modulus.get())) as u64
}

/// Computes the multiplicative inverse of `a` modulo `modulus`, if it exists.
#[must_use]
pub fn inv_mod(a: u64, modulus: Modulus) -> Option<u64> {
    let m = modulus.get();
    if m == 1 {
        return Some(0);
    }

    let mut old_r = i128::from(m);
    let mut r = i128::from(a % m);
    let mut old_t = 0_i128;
    let mut t = 1_i128;

    while r != 0 {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);
        (old_t, t) = (t, old_t - quotient * t);
    }

    (old_r == 1).then(|| reduce_i128(old_t, modulus))
}

/// A small context for repeated operations modulo one fixed modulus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModCtx {
    modulus: Modulus,
}

impl ModCtx {
    /// Creates a context for `modulus`.
    #[inline]
    #[must_use]
    pub const fn new(modulus: Modulus) -> Self {
        Self { modulus }
    }

    /// Returns this context's modulus.
    #[inline]
    #[must_use]
    pub const fn modulus(&self) -> Modulus {
        self.modulus
    }

    /// Adds two canonical residues.
    #[inline]
    #[must_use]
    pub fn add(&self, a: u64, b: u64) -> u64 {
        let m = self.modulus.get();
        debug_assert!(a < m && b < m);
        if a >= m - b { a - (m - b) } else { a + b }
    }

    /// Subtracts two canonical residues.
    #[inline]
    #[must_use]
    pub fn sub(&self, a: u64, b: u64) -> u64 {
        let m = self.modulus.get();
        debug_assert!(a < m && b < m);
        if a >= b { a - b } else { m - (b - a) }
    }

    /// Multiplies two canonical residues exactly.
    #[inline]
    #[must_use]
    pub fn mul(&self, a: u64, b: u64) -> u64 {
        let m = self.modulus.get();
        debug_assert!(a < m && b < m);
        if m <= u64::from(u32::MAX) {
            (a * b) % m
        } else {
            (u128::from(a) * u128::from(b) % u128::from(m)) as u64
        }
    }

    /// Raises a canonical residue to a nonnegative power.
    #[must_use]
    pub fn pow(&self, base: u64, mut exp: u64) -> u64 {
        let m = self.modulus.get();
        debug_assert!(base < m);
        let mut factor = base;
        let mut result = u64::from(m != 1);
        while exp != 0 {
            if exp & 1 != 0 {
                result = self.mul(result, factor);
            }
            exp >>= 1;
            if exp != 0 {
                factor = self.mul(factor, factor);
            }
        }
        result
    }

    /// Computes a multiplicative inverse in this context.
    #[inline]
    #[must_use]
    pub fn inv(&self, a: u64) -> Option<u64> {
        inv_mod(a, self.modulus)
    }
}
