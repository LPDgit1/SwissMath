use core::fmt;

use crate::{
    ArithmeticError, Congruence, ModCtx, Modulus, NumberTheoryError, crt_pair, factor, gcd,
    inv_mod, is_prime, reduce_i128,
};

/// Errors from the focused quadratic-arithmetic API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuadraticError {
    /// A modulus of zero is not a modular-arithmetic domain.
    ZeroModulus,
    /// Jacobi symbols require a positive odd denominator.
    JacobiRequiresOddModulus,
    /// The requested Legendre/prime-root operation needs an odd prime.
    PrimeModulusRequired,
    /// General composite non-unit roots are deliberately out of scope.
    NonCoprimeUnsupported,
    /// A checked u64 intermediate could not be represented.
    Arithmetic,
    /// Exact factorization of a composite u64 modulus failed.
    Factorization(NumberTheoryError),
}

impl fmt::Display for QuadraticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroModulus => f.write_str("modulus must be positive"),
            Self::JacobiRequiresOddModulus => {
                f.write_str("Jacobi symbols require a positive odd modulus")
            }
            Self::PrimeModulusRequired => f.write_str("an odd prime modulus is required"),
            Self::NonCoprimeUnsupported => {
                f.write_str("non-coprime composite square roots are unsupported")
            }
            Self::Arithmetic => f.write_str("quadratic arithmetic exceeded u64"),
            Self::Factorization(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for QuadraticError {}

impl From<NumberTheoryError> for QuadraticError {
    fn from(error: NumberTheoryError) -> Self {
        Self::Factorization(error)
    }
}

impl From<ArithmeticError> for QuadraticError {
    fn from(_: ArithmeticError) -> Self {
        Self::Arithmetic
    }
}

/// Compact root representation for a prime modulus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimeRoots {
    /// No square root exists.
    None,
    /// Exactly one root exists (zero modulo a prime, or modulo two).
    One(u64),
    /// Exactly two canonical roots, in ascending order.
    Two(u64, u64),
}

impl PrimeRoots {
    fn into_vec(self) -> Vec<u64> {
        match self {
            Self::None => Vec::new(),
            Self::One(root) => vec![root],
            Self::Two(left, right) => vec![left, right],
        }
    }
}

/// Computes the Jacobi symbol `(a / n)` for positive odd `n`.
pub fn jacobi_symbol(a: i128, n: u64) -> Result<i8, QuadraticError> {
    if n == 0 || n % 2 == 0 {
        return Err(QuadraticError::JacobiRequiresOddModulus);
    }

    let mut a = a.rem_euclid(i128::from(n)) as u64;
    let mut n = n;
    let mut result = 1_i8;

    while a != 0 {
        while a % 2 == 0 {
            a /= 2;
            if matches!(n % 8, 3 | 5) {
                result = -result;
            }
        }
        (a, n) = (n, a);
        if a % 4 == 3 && n % 4 == 3 {
            result = -result;
        }
        a %= n;
    }

    Ok(if n == 1 { result } else { 0 })
}

/// Computes the Legendre symbol `(a / p)` for an odd prime `p`.
pub fn legendre_symbol(a: i128, p: u64) -> Result<i8, QuadraticError> {
    if p < 3 || !is_prime(p) {
        return Err(QuadraticError::PrimeModulusRequired);
    }
    Ok(legendre_for_known_prime(a, p))
}

fn legendre_for_known_prime(a: i128, p: u64) -> i8 {
    // p is odd and prime here, so Jacobi's value is exactly Legendre's value.
    jacobi_symbol(a, p).expect("known odd prime has a valid Jacobi denominator")
}

/// Finds all roots of `x² = a (mod p)` for a prime `p`.
pub fn prime_square_roots(a: i128, p: u64) -> Result<PrimeRoots, QuadraticError> {
    if p < 2 || !is_prime(p) {
        return Err(QuadraticError::PrimeModulusRequired);
    }
    prime_square_roots_known(a, p)
}

fn prime_square_roots_known(a: i128, p: u64) -> Result<PrimeRoots, QuadraticError> {
    let a_mod = a.rem_euclid(i128::from(p)) as u64;
    if p == 2 {
        return Ok(PrimeRoots::One(a_mod));
    }
    if a_mod == 0 {
        return Ok(PrimeRoots::One(0));
    }
    if legendre_for_known_prime(a_mod as i128, p) != 1 {
        return Ok(PrimeRoots::None);
    }

    let root = if p % 4 == 3 {
        let exponent = ((u128::from(p) + 1) / 4) as u64;
        ModCtx::new(Modulus::new(p).expect("prime modulus is nonzero")).pow(a_mod, exponent)
    } else {
        tonelli_shanks(a_mod, p)?
    };
    Ok(canonical_prime_roots(root, p))
}

fn canonical_prime_roots(root: u64, modulus: u64) -> PrimeRoots {
    let other = if root == 0 { 0 } else { modulus - root };
    if root == other {
        PrimeRoots::One(root)
    } else if root < other {
        PrimeRoots::Two(root, other)
    } else {
        PrimeRoots::Two(other, root)
    }
}

fn tonelli_shanks(a: u64, p: u64) -> Result<u64, QuadraticError> {
    let context = ModCtx::new(Modulus::new(p).expect("prime modulus is nonzero"));
    let mut q = p - 1;
    let mut s = q.trailing_zeros();
    q >>= s;

    let mut non_residue = 2_u64;
    while legendre_for_known_prime(non_residue as i128, p) != -1 {
        non_residue = non_residue
            .checked_add(1)
            .ok_or(QuadraticError::Arithmetic)?;
    }

    let mut c = context.pow(non_residue % p, q);
    let mut x = context.pow(a, q.div_ceil(2));
    let mut t = context.pow(a, q);

    while t != 1 {
        let mut i = 1_u32;
        let mut current = context.mul(t, t);
        while current != 1 {
            current = context.mul(current, current);
            i += 1;
            if i >= s {
                return Err(QuadraticError::Arithmetic);
            }
        }
        let exponent = 1_u64
            .checked_shl(s - i - 1)
            .ok_or(QuadraticError::Arithmetic)?;
        let b = context.pow(c, exponent);
        x = context.mul(x, b);
        let b_squared = context.mul(b, b);
        t = context.mul(t, b_squared);
        c = b_squared;
        s = i;
    }
    Ok(x)
}

fn square_mod(value: u64, modulus: u64) -> u64 {
    (u128::from(value) * u128::from(value) % u128::from(modulus)) as u64
}

fn checked_prime_power(prime: u64, exponent: u32) -> Result<u64, QuadraticError> {
    (0..exponent).try_fold(1_u64, |value, _| {
        value.checked_mul(prime).ok_or(QuadraticError::Arithmetic)
    })
}

fn odd_prime_power_roots(a: i128, prime: u64, exponent: u32) -> Result<Vec<u64>, QuadraticError> {
    let prime_roots = prime_square_roots_known(a, prime)?;
    let mut root = match prime_roots {
        PrimeRoots::None => return Ok(Vec::new()),
        PrimeRoots::One(root) | PrimeRoots::Two(root, _) => root,
    };

    let mut modulus = prime;
    if exponent > 1 {
        let root_mod_prime = root;
        let derivative = (2 * (root_mod_prime % prime)) % prime;
        let inverse = inv_mod(
            derivative,
            Modulus::new(prime).expect("prime modulus is nonzero"),
        )
        .ok_or(QuadraticError::Arithmetic)?;

        for _ in 1..exponent {
            let next = modulus
                .checked_mul(prime)
                .ok_or(QuadraticError::Arithmetic)?;
            let target = reduce_i128(a, Modulus::new(next).expect("next modulus is nonzero"));
            let square = square_mod(root, next);
            let difference = ((u128::from(target) + u128::from(next) - u128::from(square))
                % u128::from(next)) as u64;
            if difference % modulus != 0 {
                return Err(QuadraticError::Arithmetic);
            }
            let correction = (u128::from(difference / modulus) % u128::from(prime)) as u64;
            let step = (u128::from(correction) * u128::from(inverse) % u128::from(prime)) as u64;
            root = root
                .checked_add(
                    step.checked_mul(modulus)
                        .ok_or(QuadraticError::Arithmetic)?,
                )
                .ok_or(QuadraticError::Arithmetic)?;
            if square_mod(root, next) != target {
                return Err(QuadraticError::Arithmetic);
            }
            modulus = next;
        }
    }

    let other = if root == 0 { 0 } else { modulus - root };
    let mut roots = if root == other {
        vec![root]
    } else if root < other {
        vec![root, other]
    } else {
        vec![other, root]
    };
    roots.sort_unstable();
    Ok(roots)
}

fn power_of_two_roots(a: i128, exponent: u32) -> Result<Vec<u64>, QuadraticError> {
    let modulus = checked_prime_power(2, exponent)?;
    let a_mod = reduce_i128(a, Modulus::new(modulus).expect("power of two is nonzero"));
    match exponent {
        0 => Ok(vec![0]),
        1 => Ok(if a_mod % 2 == 1 { vec![1] } else { Vec::new() }),
        2 => Ok(if a_mod % 4 == 1 {
            vec![1, 3]
        } else {
            Vec::new()
        }),
        _ if a_mod % 8 != 1 => Ok(Vec::new()),
        _ => {
            let mut roots = vec![1_u64, 3, 5, 7];
            let mut current_modulus = 8_u64;
            for _ in 3..exponent {
                let next_modulus = current_modulus
                    .checked_mul(2)
                    .ok_or(QuadraticError::Arithmetic)?;
                let mut lifted = Vec::with_capacity(roots.len() * 2);
                for root in roots {
                    if square_mod(root, next_modulus) == a_mod % next_modulus {
                        lifted.push(root);
                    }
                    let shifted = root
                        .checked_add(current_modulus)
                        .ok_or(QuadraticError::Arithmetic)?;
                    if square_mod(shifted, next_modulus) == a_mod % next_modulus {
                        lifted.push(shifted);
                    }
                }
                roots = lifted;
                current_modulus = next_modulus;
            }
            roots.sort_unstable();
            Ok(roots)
        }
    }
}

/// Finds all unit roots of `x² = a (mod n)` in the supported u64 domain.
///
/// Prime moduli also support `a = 0`; for composite moduli a non-coprime
/// right-hand side is rejected explicitly rather than returning partial roots.
pub fn modular_square_roots(a: i128, n: u64) -> Result<Vec<u64>, QuadraticError> {
    if n == 0 {
        return Err(QuadraticError::ZeroModulus);
    }
    if n == 1 {
        return Ok(vec![0]);
    }
    if is_prime(n) {
        return Ok(prime_square_roots_known(a, n)?.into_vec());
    }

    let modulus = Modulus::new(n).expect("n > 1 is nonzero");
    let reduced_a = reduce_i128(a, modulus);
    if gcd(reduced_a, n) != 1 {
        return Err(QuadraticError::NonCoprimeUnsupported);
    }

    let factorization = factor(n)?;
    let mut current_roots = vec![0_u64];
    let mut current_modulus = 1_u64;
    for factor in factorization.factors() {
        let component_modulus = checked_prime_power(factor.prime, factor.exponent)?;
        let component_roots = if factor.prime == 2 {
            power_of_two_roots(a, factor.exponent)?
        } else {
            odd_prime_power_roots(a, factor.prime, factor.exponent)?
        };
        if component_roots.is_empty() {
            return Ok(Vec::new());
        }
        let capacity = current_roots
            .len()
            .checked_mul(component_roots.len())
            .ok_or(QuadraticError::Arithmetic)?;
        let mut combined = Vec::with_capacity(capacity);
        for &left in &current_roots {
            for &right in &component_roots {
                let left_congruence = Congruence::new(
                    left,
                    Modulus::new(current_modulus).expect("current modulus is nonzero"),
                );
                let right_congruence = Congruence::new(
                    right,
                    Modulus::new(component_modulus).expect("component modulus is nonzero"),
                );
                if let Some(result) = crt_pair(left_congruence, right_congruence)? {
                    combined.push(result.residue());
                }
            }
        }
        combined.sort_unstable();
        combined.dedup();
        current_roots = combined;
        current_modulus = current_modulus
            .checked_mul(component_modulus)
            .ok_or(QuadraticError::Arithmetic)?;
    }
    Ok(current_roots)
}

#[cfg(test)]
mod tests {
    use super::{odd_prime_power_roots, square_mod};

    #[test]
    fn hensel_lift_preserves_each_intermediate_level() {
        let a = 10_i128;
        let prime = 13_u64;
        let roots = odd_prime_power_roots(a, prime, 3).unwrap();
        let mut modulus = prime;
        for _ in 1..=3 {
            for &root in &roots {
                assert_eq!(
                    square_mod(root, modulus),
                    (a.rem_euclid(i128::from(modulus))) as u64
                );
            }
            modulus *= prime;
        }
    }
}
