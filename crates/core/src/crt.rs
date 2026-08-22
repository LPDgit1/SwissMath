use crate::{ArithmeticError, Modulus, gcd, inv_mod, reduce_i128};

/// One canonical congruence class.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Congruence {
    residue: u64,
    modulus: Modulus,
}

impl Congruence {
    /// Constructs a congruence, normalizing its residue.
    #[inline]
    #[must_use]
    pub fn new(residue: u64, modulus: Modulus) -> Self {
        Self {
            residue: residue % modulus.get(),
            modulus,
        }
    }

    /// Returns the canonical residue.
    #[inline]
    #[must_use]
    pub const fn residue(self) -> u64 {
        self.residue
    }

    /// Returns the modulus.
    #[inline]
    #[must_use]
    pub const fn modulus(self) -> Modulus {
        self.modulus
    }
}

/// Tests whether two congruences have a common solution.
#[must_use]
pub fn crt_compatible(a: Congruence, b: Congruence) -> bool {
    let m = a.modulus.get();
    let n = b.modulus.get();
    if m == 1 || n == 1 {
        return true;
    }
    if m == n {
        return a.residue == b.residue;
    }
    let common = gcd(m, n);
    a.residue % common == b.residue % common
}

/// Combines two congruences using the generalized Chinese remainder theorem.
///
/// `Ok(None)` denotes mathematical incompatibility. Overflow is reported only
/// when the exact combined modulus cannot be represented in `u64`.
pub fn crt_pair(a: Congruence, b: Congruence) -> Result<Option<Congruence>, ArithmeticError> {
    let m = a.modulus.get();
    let n = b.modulus.get();

    if m == 1 {
        return Ok(Some(b));
    }
    if n == 1 {
        return Ok(Some(a));
    }
    if m == n {
        return Ok((a.residue == b.residue).then_some(a));
    }

    let common = gcd(m, n);
    if a.residue % common != b.residue % common {
        return Ok(None);
    }
    if common == m {
        return Ok(Some(b));
    }
    if common == n {
        return Ok(Some(a));
    }

    let reduced_m = m / common;
    let reduced_n = n / common;
    let combined_modulus = reduced_m.checked_mul(n).ok_or(ArithmeticError::Overflow)?;

    let inverse = inv_mod(
        reduced_m % reduced_n,
        Modulus::new(reduced_n).expect("reduced modulus is nonzero"),
    )
    .expect("coprime reduced moduli always have an inverse");
    let quotient_difference = (i128::from(b.residue) - i128::from(a.residue)) / i128::from(common);
    let delta = reduce_i128(
        quotient_difference,
        Modulus::new(reduced_n).expect("reduced modulus is nonzero"),
    );
    let step = (u128::from(delta) * u128::from(inverse) % u128::from(reduced_n)) as u64;
    let solution =
        (u128::from(a.residue) + u128::from(m) * u128::from(step)) % u128::from(combined_modulus);

    Ok(Some(Congruence::new(
        solution as u64,
        Modulus::new(combined_modulus).expect("checked LCM is nonzero"),
    )))
}

/// Folds congruences into one exact constraint.
///
/// The empty fold is the identity `0 mod 1`, representing all integers.
pub fn crt_fold<I>(congruences: I) -> Result<Option<Congruence>, ArithmeticError>
where
    I: IntoIterator<Item = Congruence>,
{
    let identity = Congruence::new(0, Modulus::new(1).expect("one is nonzero"));
    let mut accumulated = identity;
    for congruence in congruences {
        match crt_pair(accumulated, congruence)? {
            Some(combined) => accumulated = combined,
            None => return Ok(None),
        }
    }
    Ok(Some(accumulated))
}
