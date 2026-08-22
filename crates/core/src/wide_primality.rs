//! Bounded exact-first primality for the native `u128` domain.
//!
//! This module intentionally proves only primality.  It does not widen the
//! modular arithmetic, factorization, or arithmetic-function APIs.  A
//! candidate is rejected cheaply first, then receives a bounded Pocklington
//! proof based on the exactly known factors of `n - 1`.

use num_bigint::BigUint;
use num_prime::nt_funcs::is_prime as big_is_prime;
use num_prime::{Primality, PrimalityTestConfig};

use crate::PrimalityAssessment;
use crate::number_theory::{SMALL_PRIMES, factor, is_prime};

const MAX_PROOF_DEPTH: u8 = 8;
const FIRST_WITNESS: u128 = 2;
const LAST_WITNESS: u128 = 64;

/// Assesses a value using the exact u64 path or the bounded exact-first u128
/// path.  Values larger than u64 never fall back to a probable-prime result:
/// they are either rejected as composite, proved exact, or reported as
/// inconclusive.
#[must_use]
pub fn assess_primality_u128(n: u128) -> PrimalityAssessment {
    if n <= u128::from(u64::MAX) {
        return assess_u64(n as u64);
    }
    assess_u128_inner(n, 0)
}

fn assess_u64(n: u64) -> PrimalityAssessment {
    match n {
        0 | 1 => PrimalityAssessment::Neither,
        _ if is_prime(n) => PrimalityAssessment::PrimeExact,
        _ => PrimalityAssessment::Composite,
    }
}

fn assess_u128_inner(n: u128, depth: u8) -> PrimalityAssessment {
    if n < 2 {
        return PrimalityAssessment::Neither;
    }
    for &small_prime in SMALL_PRIMES {
        let prime = u128::from(small_prime);
        if n == prime {
            return PrimalityAssessment::PrimeExact;
        }
        if n % prime == 0 {
            return PrimalityAssessment::Composite;
        }
    }

    // BPSW is used only as a one-way composite filter in the u128 domain.
    // Passing it never produces the public probable-prime result here.
    if bpsw_rejects(n) {
        return PrimalityAssessment::Composite;
    }
    if depth >= MAX_PROOF_DEPTH {
        return PrimalityAssessment::ExactProofIncomplete;
    }
    pocklington_from_n_minus_one(n, depth)
}

fn bpsw_rejects(n: u128) -> bool {
    let value = BigUint::from(n);
    matches!(
        big_is_prime(&value, Some(PrimalityTestConfig::bpsw())),
        Primality::No
    )
}

fn pocklington_from_n_minus_one(n: u128, depth: u8) -> PrimalityAssessment {
    let mut remainder = n - 1;
    let mut known_factors = Vec::new();
    let mut known_product = 1_u128;

    for &small_prime in SMALL_PRIMES {
        let prime = u128::from(small_prime);
        if remainder % prime != 0 {
            continue;
        }
        let mut prime_power = 1_u128;
        while remainder % prime == 0 {
            remainder /= prime;
            prime_power = match prime_power.checked_mul(prime) {
                Some(value) => value,
                None => return PrimalityAssessment::ExactProofIncomplete,
            };
        }
        known_product = match known_product.checked_mul(prime_power) {
            Some(value) => value,
            None => return PrimalityAssessment::ExactProofIncomplete,
        };
        known_factors.push(prime);
        if passes_sqrt_threshold(known_product, n) {
            return pocklington(n, known_factors);
        }
    }

    if remainder > 1 && remainder <= u128::from(u64::MAX) {
        let remainder_factors = match factor(remainder as u64) {
            Ok(value) => value,
            Err(_) => return PrimalityAssessment::ExactProofIncomplete,
        };
        for factor in remainder_factors.factors() {
            let prime = u128::from(factor.prime);
            let prime_power = match checked_pow_u128(prime, factor.exponent) {
                Some(value) => value,
                None => return PrimalityAssessment::ExactProofIncomplete,
            };
            known_product = match known_product.checked_mul(prime_power) {
                Some(value) => value,
                None => return PrimalityAssessment::ExactProofIncomplete,
            };
            known_factors.push(prime);
            if passes_sqrt_threshold(known_product, n) {
                return pocklington(n, known_factors);
            }
        }
    } else if remainder > u128::from(u64::MAX) {
        // The whole residual is the only permitted recursive target.  A
        // composite residual does not make the parent composite; it merely
        // leaves the Pocklington proof without enough known factor mass.
        if depth + 1 >= MAX_PROOF_DEPTH {
            return PrimalityAssessment::ExactProofIncomplete;
        }
        match assess_u128_inner(remainder, depth + 1) {
            PrimalityAssessment::PrimeExact => {
                known_product = match known_product.checked_mul(remainder) {
                    Some(value) => value,
                    None => return PrimalityAssessment::ExactProofIncomplete,
                };
                known_factors.push(remainder);
            }
            PrimalityAssessment::Composite | PrimalityAssessment::ExactProofIncomplete => {
                return PrimalityAssessment::ExactProofIncomplete;
            }
            PrimalityAssessment::Neither | PrimalityAssessment::ProbablePrime => {
                return PrimalityAssessment::ExactProofIncomplete;
            }
        }
    }

    if passes_sqrt_threshold(known_product, n) {
        pocklington(n, known_factors)
    } else {
        PrimalityAssessment::ExactProofIncomplete
    }
}

fn checked_pow_u128(base: u128, exponent: u32) -> Option<u128> {
    let mut result = 1_u128;
    for _ in 0..exponent {
        result = result.checked_mul(base)?;
    }
    Some(result)
}

#[inline]
fn passes_sqrt_threshold(known_product: u128, n: u128) -> bool {
    known_product > n / known_product
}

fn pocklington(n: u128, known_factors: Vec<u128>) -> PrimalityAssessment {
    let modulus = BigUint::from(n);
    let n_minus_one = BigUint::from(n - 1);
    let mut distinct = known_factors;
    distinct.sort_unstable();
    distinct.dedup();

    for prime_factor in distinct {
        let exponent = (n - 1) / prime_factor;
        let exponent = BigUint::from(exponent);
        let mut witness_found = false;
        for witness in FIRST_WITNESS..=LAST_WITNESS {
            if gcd_u128(witness, n) != 1 {
                continue;
            }
            let base = BigUint::from(witness);
            if base.modpow(&n_minus_one, &modulus) != BigUint::from(1_u8) {
                continue;
            }
            let reduced = base.modpow(&exponent, &modulus);
            let bytes = reduced.to_bytes_le();
            if bytes.len() > 16 {
                return PrimalityAssessment::ExactProofIncomplete;
            }
            let reduced_u128 = bytes
                .iter()
                .enumerate()
                .fold(0_u128, |value, (index, byte)| {
                    value | (u128::from(*byte) << (index * 8))
                });
            let difference = if reduced_u128 == 0 {
                n - 1
            } else {
                reduced_u128 - 1
            };
            if gcd_u128(difference, n) == 1 {
                witness_found = true;
                break;
            }
        }
        if !witness_found {
            return PrimalityAssessment::ExactProofIncomplete;
        }
    }
    PrimalityAssessment::PrimeExact
}

#[inline]
fn gcd_u128(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

#[cfg(test)]
mod tests {
    use super::{gcd_u128, pocklington_from_n_minus_one};
    use crate::{PrimalityAssessment, is_prime};

    #[test]
    fn gcd_handles_wide_values() {
        assert_eq!(gcd_u128(0, 7), 7);
        assert_eq!(gcd_u128(u128::MAX, u128::MAX - 1), 1);
        assert_eq!(gcd_u128(12_u128 << 64, 18_u128 << 64), 6_u128 << 64);
    }

    #[test]
    fn pocklington_helper_never_certifies_a_composite_u64_value() {
        let primes = [
            53_u64,
            97,
            101,
            1_009,
            65_537,
            1_000_000_007,
            1_000_000_009,
            2_147_483_647,
            2_147_483_629,
            3_037_000_493,
            4_294_967_291,
            18_446_744_073_709_551_557,
        ];
        for n in primes {
            let result = pocklington_from_n_minus_one(u128::from(n), 0);
            assert!(
                matches!(
                    result,
                    PrimalityAssessment::PrimeExact | PrimalityAssessment::ExactProofIncomplete
                ),
                "unexpected result for prime {n}: {result:?}"
            );
            if result == PrimalityAssessment::PrimeExact {
                assert!(is_prime(n));
            }
        }

        for n in [
            4_u64,
            9,
            15,
            561,
            341_550_071_728_321,
            1_000_000_007_u64 * 1_000_000_009,
            u64::MAX,
        ] {
            assert_ne!(
                pocklington_from_n_minus_one(u128::from(n), 0),
                PrimalityAssessment::PrimeExact,
                "composite {n} was certified"
            );
        }
    }
}
