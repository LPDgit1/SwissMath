//! Exact modular combinatorics over a [`PrimeField`].
//!
//! Factorial valuation uses Legendre in `O(log_p n)`, binomial valuation uses
//! Kummer in `O(log_p n)`, and binomial residues use Lucas plus multiplicative
//! digit binomials. Factorial residues choose the shorter direct or Wilson-
//! complement product. Linear product work is bounded before it starts.

use crate::PrimeField;

const MAX_COMBINATORIAL_PRODUCT_STEPS: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombinatoricsError {
    KExceedsN,
    ComputationLimitReached { estimated_steps: u64, limit: u64 },
    InternalInverseFailure,
}

/// Returns `v_p(n!)` by Legendre's formula in `O(log_p n)` time.
#[must_use]
pub fn factorial_valuation(mut n: u64, field: PrimeField) -> u64 {
    let mut valuation = 0_u64;
    while n != 0 {
        n /= field.modulus();
        valuation += n;
    }
    valuation
}

/// Returns `v_p(C(n,k))` by counting base-p carries (Kummer's theorem).
pub fn binomial_valuation(n: u64, k: u64, field: PrimeField) -> Result<u64, CombinatoricsError> {
    if k > n {
        return Err(CombinatoricsError::KExceedsN);
    }
    let prime = field.modulus();
    let mut left = k;
    let mut right = n - k;
    let mut carry = 0_u64;
    let mut carries = 0_u64;
    while left != 0 || right != 0 || carry != 0 {
        let sum = u128::from(left % prime) + u128::from(right % prime) + u128::from(carry);
        carry = u64::from(sum >= u128::from(prime));
        carries += carry;
        left /= prime;
        right /= prime;
    }
    Ok(carries)
}

/// Returns `C(n,k) mod p` using Lucas' theorem and bounded digit products.
pub fn binomial_mod_prime(n: u64, k: u64, field: PrimeField) -> Result<u64, CombinatoricsError> {
    binomial_mod_prime_bounded(n, k, field, MAX_COMBINATORIAL_PRODUCT_STEPS)
}

/// Returns `n! mod p`, choosing the shorter direct or Wilson-complement path.
pub fn factorial_mod_prime(n: u64, field: PrimeField) -> Result<u64, CombinatoricsError> {
    factorial_mod_prime_bounded(n, field, MAX_COMBINATORIAL_PRODUCT_STEPS)
}

fn binomial_mod_prime_bounded(
    n: u64,
    k: u64,
    field: PrimeField,
    work_limit: u64,
) -> Result<u64, CombinatoricsError> {
    if k > n {
        return Ok(0);
    }
    let prime = field.modulus();
    let mut n_digits = n;
    let mut k_digits = k;
    let mut estimated_steps = 0_u64;
    while n_digits != 0 || k_digits != 0 {
        let n_digit = n_digits % prime;
        let k_digit = k_digits % prime;
        if k_digit > n_digit {
            return Ok(0);
        }
        estimated_steps += k_digit.min(n_digit - k_digit);
        n_digits /= prime;
        k_digits /= prime;
    }
    ensure_work(estimated_steps, work_limit)?;

    let mut result = 1_u64;
    let mut n_digits = n;
    let mut k_digits = k;
    while n_digits != 0 || k_digits != 0 {
        result = field.mul(
            result,
            small_digit_binomial(n_digits % prime, k_digits % prime, field)?,
        );
        n_digits /= prime;
        k_digits /= prime;
    }
    Ok(result)
}

fn small_digit_binomial(n: u64, k: u64, field: PrimeField) -> Result<u64, CombinatoricsError> {
    let reduced_k = k.min(n - k);
    let mut numerator = 1_u64;
    let mut denominator = 1_u64;
    for offset in 1..=reduced_k {
        numerator = field.mul(numerator, n - reduced_k + offset);
        denominator = field.mul(denominator, offset);
    }
    let inverse = field
        .inverse(denominator)
        .ok_or(CombinatoricsError::InternalInverseFailure)?;
    Ok(field.mul(numerator, inverse))
}

fn factorial_mod_prime_bounded(
    n: u64,
    field: PrimeField,
    work_limit: u64,
) -> Result<u64, CombinatoricsError> {
    let prime = field.modulus();
    if n >= prime {
        return Ok(0);
    }
    if n <= 1 {
        return Ok(1);
    }
    let direct_steps = n - 1;
    let complement_steps = prime - 1 - n;
    let estimated_steps = direct_steps.min(complement_steps);
    ensure_work(estimated_steps, work_limit)?;

    if direct_steps <= complement_steps {
        let mut result = 1_u64;
        for value in 2..=n {
            result = field.mul(result, value);
        }
        Ok(result)
    } else {
        let mut complement = 1_u64;
        for value in (n + 1)..prime {
            complement = field.mul(complement, value);
        }
        let inverse = field
            .inverse(complement)
            .ok_or(CombinatoricsError::InternalInverseFailure)?;
        Ok(field.sub(0, inverse))
    }
}

fn ensure_work(estimated_steps: u64, limit: u64) -> Result<(), CombinatoricsError> {
    if estimated_steps > limit {
        Err(CombinatoricsError::ComputationLimitReached {
            estimated_steps,
            limit,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_limits_force_refusal_before_product_work() {
        let field = PrimeField::new(101).unwrap();
        assert_eq!(
            binomial_mod_prime_bounded(50, 25, field, 3),
            Err(CombinatoricsError::ComputationLimitReached {
                estimated_steps: 25,
                limit: 3
            })
        );
        assert_eq!(
            factorial_mod_prime_bounded(50, field, 3),
            Err(CombinatoricsError::ComputationLimitReached {
                estimated_steps: 49,
                limit: 3
            })
        );
    }

    #[test]
    fn zero_shortcuts_precede_the_work_limit() {
        let field = PrimeField::new(101).unwrap();
        assert_eq!(binomial_mod_prime_bounded(101, 1, field, 0), Ok(0));
        assert_eq!(factorial_mod_prime_bounded(101, field, 0), Ok(0));
    }
}
