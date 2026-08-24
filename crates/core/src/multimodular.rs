//! Streaming multimodular CRT and exact reconstruction.
//!
//! One [`MultimodularAccumulator`] stores a flat coordinate vector modulo an
//! arbitrarily large `BigUint` product. Each distinct prime block is combined
//! incrementally; the shared `M mod p` and its inverse are computed once per
//! block. Centered representatives are canonical residue representatives,
//! while bounded integer and rational reconstruction additionally enforce
//! exact uniqueness conditions and verify every candidate.

use num_bigint::{BigInt, BigUint, Sign};

use crate::PrimeField;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MultimodularError {
    EmptyResidueBlock,
    CoordinateCountMismatch { expected: usize, actual: usize },
    DuplicatePrimeModulus { prime: u64 },
    InvalidModulus,
    InvalidBound,
    InsufficientModulus,
    NoReconstruction,
    CoordinateReconstructionFailed { index: usize },
    InternalInverseFailure,
    InternalVerificationFailure,
    CoordinateVerificationFailed { index: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BigRationalReconstruction {
    pub numerator: BigInt,
    pub denominator: BigUint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultimodularAccumulator {
    modulus: BigUint,
    values: Vec<BigUint>,
    prime_count: usize,
}

impl Default for MultimodularAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl MultimodularAccumulator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            modulus: BigUint::from(1_u8),
            values: Vec::new(),
            prime_count: 0,
        }
    }

    /// Combines one prime residue block, retaining no copy of the source block.
    pub fn push_prime_residues(
        &mut self,
        field: PrimeField,
        residues: &[u64],
    ) -> Result<(), MultimodularError> {
        if residues.is_empty() {
            return Err(MultimodularError::EmptyResidueBlock);
        }
        let prime = field.modulus();
        if self.prime_count == 0 {
            self.modulus = BigUint::from(prime);
            self.values = residues
                .iter()
                .map(|residue| BigUint::from(residue % prime))
                .collect();
            self.prime_count = 1;
            return Ok(());
        }
        if residues.len() != self.values.len() {
            return Err(MultimodularError::CoordinateCountMismatch {
                expected: self.values.len(),
                actual: residues.len(),
            });
        }

        let modulus_mod_prime = biguint_mod_u64(&self.modulus, prime);
        if modulus_mod_prime == 0 {
            return Err(MultimodularError::DuplicatePrimeModulus { prime });
        }
        let inverse = field
            .inverse(modulus_mod_prime)
            .ok_or(MultimodularError::InternalInverseFailure)?;
        let old_modulus = self.modulus.clone();
        for (value, residue) in self.values.iter_mut().zip(residues) {
            let current_mod_prime = biguint_mod_u64(value, prime);
            let delta = field.sub(residue % prime, current_mod_prime);
            let step = field.mul(delta, inverse);
            *value += &old_modulus * BigUint::from(step);
        }
        self.modulus *= prime;
        self.prime_count += 1;
        Ok(())
    }

    #[must_use]
    pub fn combined_modulus(&self) -> &BigUint {
        &self.modulus
    }

    #[must_use]
    pub fn values(&self) -> &[BigUint] {
        &self.values
    }

    #[must_use]
    pub fn coordinate_count(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn prime_count(&self) -> usize {
        self.prime_count
    }

    #[must_use]
    pub fn combined_modulus_bits(&self) -> u64 {
        self.modulus.bits()
    }

    pub fn centered_representatives(&self) -> Result<Vec<BigInt>, MultimodularError> {
        self.ensure_nonempty()?;
        self.values
            .iter()
            .map(|value| centered_representative(value, &self.modulus))
            .collect()
    }

    pub fn reconstruct_integers_bounded(
        &self,
        max_abs: &BigUint,
    ) -> Result<Vec<BigInt>, MultimodularError> {
        self.ensure_nonempty()?;
        ensure_integer_uniqueness(&self.modulus, max_abs)?;
        self.values
            .iter()
            .enumerate()
            .map(|(index, residue)| {
                reconstruct_integer_bounded_checked(residue, &self.modulus, max_abs).map_err(
                    |error| match error {
                        MultimodularError::NoReconstruction => {
                            MultimodularError::CoordinateReconstructionFailed { index }
                        }
                        MultimodularError::InternalVerificationFailure => {
                            MultimodularError::CoordinateVerificationFailed { index }
                        }
                        other => other,
                    },
                )
            })
            .collect()
    }

    pub fn reconstruct_rationals(
        &self,
    ) -> Result<Vec<BigRationalReconstruction>, MultimodularError> {
        self.ensure_nonempty()?;
        let bound = automatic_rational_bound(&self.modulus)?;
        let denominator_bound = bound.clone().max(BigUint::from(1_u8));
        self.reconstruct_rationals_bounded_checked(&bound, &denominator_bound)
    }

    pub fn reconstruct_rationals_bounded(
        &self,
        max_numerator_abs: &BigUint,
        max_denominator: &BigUint,
    ) -> Result<Vec<BigRationalReconstruction>, MultimodularError> {
        self.ensure_nonempty()?;
        ensure_rational_uniqueness(&self.modulus, max_numerator_abs, max_denominator)?;
        self.reconstruct_rationals_bounded_checked(max_numerator_abs, max_denominator)
    }

    fn reconstruct_rationals_bounded_checked(
        &self,
        max_numerator_abs: &BigUint,
        max_denominator: &BigUint,
    ) -> Result<Vec<BigRationalReconstruction>, MultimodularError> {
        self.values
            .iter()
            .enumerate()
            .map(|(index, residue)| {
                rational_reconstruct_big_bounded_checked(
                    residue,
                    &self.modulus,
                    max_numerator_abs,
                    max_denominator,
                )
                .and_then(|candidate| candidate.ok_or(MultimodularError::NoReconstruction))
                .map_err(|error| match error {
                    MultimodularError::NoReconstruction => {
                        MultimodularError::CoordinateReconstructionFailed { index }
                    }
                    MultimodularError::InternalVerificationFailure => {
                        MultimodularError::CoordinateVerificationFailed { index }
                    }
                    other => other,
                })
            })
            .collect()
    }

    fn ensure_nonempty(&self) -> Result<(), MultimodularError> {
        if self.prime_count == 0 {
            Err(MultimodularError::InvalidModulus)
        } else {
            Ok(())
        }
    }
}

/// Returns the canonical representative in `[-floor(M/2), floor(M/2)]`.
pub fn centered_representative(
    residue: &BigUint,
    modulus: &BigUint,
) -> Result<BigInt, MultimodularError> {
    ensure_valid_modulus(modulus)?;
    let residue = residue % modulus;
    if residue <= modulus >> 1_usize {
        Ok(BigInt::from(residue))
    } else {
        Ok(BigInt::from(residue) - BigInt::from(modulus.clone()))
    }
}

/// Reconstructs the unique integer with `|x| <= max_abs` when `2*max_abs < M`.
pub fn reconstruct_integer_bounded(
    residue: &BigUint,
    modulus: &BigUint,
    max_abs: &BigUint,
) -> Result<BigInt, MultimodularError> {
    ensure_integer_uniqueness(modulus, max_abs)?;
    reconstruct_integer_bounded_checked(residue, modulus, max_abs)
}

fn reconstruct_integer_bounded_checked(
    residue: &BigUint,
    modulus: &BigUint,
    max_abs: &BigUint,
) -> Result<BigInt, MultimodularError> {
    let normalized = residue % modulus;
    let candidate = centered_representative(&normalized, modulus)?;
    if candidate.magnitude() > max_abs {
        return Err(MultimodularError::NoReconstruction);
    }
    if canonical_bigint_mod(&candidate, modulus) != normalized {
        return Err(MultimodularError::InternalVerificationFailure);
    }
    Ok(candidate)
}

/// Reconstructs with the conventional automatic bound
/// `floor(sqrt((M-1)/2))` for numerator and denominator.
pub fn rational_reconstruct_big(
    residue: &BigUint,
    modulus: &BigUint,
) -> Result<Option<BigRationalReconstruction>, MultimodularError> {
    let bound = automatic_rational_bound(modulus)?;
    let denominator_bound = bound.clone().max(BigUint::from(1_u8));
    rational_reconstruct_big_bounded(residue, modulus, &bound, &denominator_bound)
}

/// Reconstructs a reduced `a/b` under explicit bounds and `2*A*B < M`.
pub fn rational_reconstruct_big_bounded(
    residue: &BigUint,
    modulus: &BigUint,
    max_numerator_abs: &BigUint,
    max_denominator: &BigUint,
) -> Result<Option<BigRationalReconstruction>, MultimodularError> {
    ensure_rational_uniqueness(modulus, max_numerator_abs, max_denominator)?;
    rational_reconstruct_big_bounded_checked(residue, modulus, max_numerator_abs, max_denominator)
}

fn rational_reconstruct_big_bounded_checked(
    residue: &BigUint,
    modulus: &BigUint,
    max_numerator_abs: &BigUint,
    max_denominator: &BigUint,
) -> Result<Option<BigRationalReconstruction>, MultimodularError> {
    let residue = residue % modulus;
    if residue == BigUint::from(0_u8) {
        return Ok(Some(BigRationalReconstruction {
            numerator: BigInt::from(0_u8),
            denominator: BigUint::from(1_u8),
        }));
    }

    let mut old_r = BigInt::from(modulus.clone());
    let mut r = BigInt::from(residue.clone());
    let mut old_t = BigInt::from(0_u8);
    let mut t = BigInt::from(1_u8);
    while r != BigInt::from(0_u8) && r.magnitude() > max_numerator_abs {
        let quotient = &old_r / &r;
        let next_r = &old_r - &quotient * &r;
        let next_t = &old_t - quotient * &t;
        old_r = r;
        r = next_r;
        old_t = t;
        t = next_t;
    }
    if r == BigInt::from(0_u8) || t == BigInt::from(0_u8) {
        return Ok(None);
    }

    let (numerator, denominator) = if t.sign() == Sign::Minus {
        (-r, t.magnitude().clone())
    } else {
        (r, t.magnitude().clone())
    };
    if numerator.magnitude() > max_numerator_abs
        || denominator == BigUint::from(0_u8)
        || &denominator > max_denominator
        || biguint_gcd(numerator.magnitude().clone(), denominator.clone()) != BigUint::from(1_u8)
    {
        return Ok(None);
    }
    if canonical_bigint_mod(&numerator, modulus) != (&residue * &denominator) % modulus {
        return Err(MultimodularError::InternalVerificationFailure);
    }
    Ok(Some(BigRationalReconstruction {
        numerator,
        denominator,
    }))
}

fn ensure_valid_modulus(modulus: &BigUint) -> Result<(), MultimodularError> {
    if modulus < &BigUint::from(2_u8) {
        Err(MultimodularError::InvalidModulus)
    } else {
        Ok(())
    }
}

fn ensure_integer_uniqueness(
    modulus: &BigUint,
    max_abs: &BigUint,
) -> Result<(), MultimodularError> {
    ensure_valid_modulus(modulus)?;
    if (max_abs << 1_usize) >= *modulus {
        Err(MultimodularError::InsufficientModulus)
    } else {
        Ok(())
    }
}

fn ensure_rational_uniqueness(
    modulus: &BigUint,
    max_numerator_abs: &BigUint,
    max_denominator: &BigUint,
) -> Result<(), MultimodularError> {
    ensure_valid_modulus(modulus)?;
    if max_denominator == &BigUint::from(0_u8) {
        return Err(MultimodularError::InvalidBound);
    }
    if (max_numerator_abs * max_denominator) << 1_usize >= *modulus {
        Err(MultimodularError::InsufficientModulus)
    } else {
        Ok(())
    }
}

fn automatic_rational_bound(modulus: &BigUint) -> Result<BigUint, MultimodularError> {
    ensure_valid_modulus(modulus)?;
    Ok(biguint_sqrt(&((modulus - BigUint::from(1_u8)) >> 1_usize)))
}

fn biguint_sqrt(value: &BigUint) -> BigUint {
    if value < &BigUint::from(2_u8) {
        return value.clone();
    }
    let shift = usize::try_from(value.bits().div_ceil(2)).unwrap_or(usize::MAX);
    let mut current = BigUint::from(1_u8) << shift;
    loop {
        let next = (&current + value / &current) >> 1_usize;
        if next >= current {
            return current;
        }
        current = next;
    }
}

fn biguint_gcd(mut left: BigUint, mut right: BigUint) -> BigUint {
    while right != BigUint::from(0_u8) {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    left
}

fn biguint_mod_u64(value: &BigUint, modulus: u64) -> u64 {
    (value % modulus).iter_u64_digits().next().unwrap_or(0)
}

fn canonical_bigint_mod(value: &BigInt, modulus: &BigUint) -> BigUint {
    let modulus_int = BigInt::from(modulus.clone());
    let mut remainder = value % &modulus_int;
    if remainder.sign() == Sign::Minus {
        remainder += modulus_int;
    }
    let (sign, bytes) = remainder.to_bytes_be();
    debug_assert_ne!(sign, Sign::Minus);
    BigUint::from_bytes_be(&bytes)
}
