use core::fmt;
use std::cmp::Ordering;

use num_bigint::BigInt;

use crate::{ModCtx, Modulus, gcd, reduce_i128};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FractionError {
    ZeroDenominator,
    InvalidDecimal,
    InvalidBound,
    ReconstructionFailed,
    InvalidModulus,
}

/// A normalized exact result from modular rational reconstruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RationalReconstruction {
    pub numerator: i128,
    pub denominator: u64,
}

impl fmt::Display for RationalReconstruction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            self.numerator.fmt(formatter)
        } else {
            write!(formatter, "{}/{}", self.numerator, self.denominator)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rational {
    numerator: BigInt,
    denominator: BigInt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rationalization {
    pub fraction: Rational,
    pub absolute_error: Rational,
    pub max_denominator: u64,
}

impl Rational {
    pub fn new(numerator: BigInt, denominator: BigInt) -> Result<Self, FractionError> {
        if denominator == BigInt::from(0) {
            return Err(FractionError::ZeroDenominator);
        }
        let mut numerator = numerator;
        let mut denominator = denominator;
        if denominator < BigInt::from(0) {
            numerator = -numerator;
            denominator = -denominator;
        }
        let common = bigint_gcd(numerator.clone(), denominator.clone());
        Ok(Self {
            numerator: numerator / &common,
            denominator: denominator / common,
        })
    }

    #[must_use]
    pub fn from_i64(value: i64) -> Self {
        Self {
            numerator: BigInt::from(value),
            denominator: BigInt::from(1),
        }
    }

    #[must_use]
    pub fn numerator(&self) -> &BigInt {
        &self.numerator
    }

    #[must_use]
    pub fn denominator(&self) -> &BigInt {
        &self.denominator
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.numerator == BigInt::from(0)
    }

    #[must_use]
    pub fn negated(&self) -> Self {
        Self {
            numerator: -self.numerator.clone(),
            denominator: self.denominator.clone(),
        }
    }

    #[must_use]
    pub fn abs(&self) -> Self {
        if self.numerator < BigInt::from(0) {
            self.negated()
        } else {
            self.clone()
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self::new(
            &self.numerator * &other.denominator + &other.numerator * &self.denominator,
            &self.denominator * &other.denominator,
        )
        .expect("products of positive denominators stay positive")
    }

    pub fn sub(&self, other: &Self) -> Self {
        self.add(&other.negated())
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self::new(
            &self.numerator * &other.numerator,
            &self.denominator * &other.denominator,
        )
        .expect("products of positive denominators stay positive")
    }

    pub fn div(&self, other: &Self) -> Result<Self, FractionError> {
        Self::new(
            &self.numerator * &other.denominator,
            &self.denominator * &other.numerator,
        )
    }

    #[must_use]
    pub fn cmp_value(&self, other: &Self) -> Ordering {
        (&self.numerator * &other.denominator).cmp(&(&other.numerator * &self.denominator))
    }

    #[must_use]
    pub fn to_f64(&self) -> f64 {
        let numerator = self
            .numerator
            .to_string()
            .parse::<f64>()
            .unwrap_or_else(|_| {
                if self.numerator < BigInt::from(0) {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                }
            });
        let denominator = self
            .denominator
            .to_string()
            .parse::<f64>()
            .unwrap_or(f64::INFINITY);
        numerator / denominator
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == BigInt::from(1) {
            self.numerator.fmt(formatter)
        } else {
            write!(formatter, "{}/{}", self.numerator, self.denominator)
        }
    }
}

pub fn parse_decimal(input: &str) -> Result<Rational, FractionError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(FractionError::InvalidDecimal);
    }
    let (mantissa, exponent) = match trimmed.find(['e', 'E']) {
        Some(index) => (
            &trimmed[..index],
            trimmed[index + 1..]
                .parse::<i32>()
                .map_err(|_| FractionError::InvalidDecimal)?,
        ),
        None => (trimmed, 0),
    };
    let negative = mantissa.starts_with('-');
    let unsigned = mantissa.strip_prefix(['-', '+']).unwrap_or(mantissa);
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fractional = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || (integer.is_empty() && fractional.is_empty())
        || !integer
            .chars()
            .chain(fractional.chars())
            .all(|character| character.is_ascii_digit())
    {
        return Err(FractionError::InvalidDecimal);
    }
    let digits = format!("{integer}{fractional}");
    let mut numerator = digits
        .parse::<BigInt>()
        .map_err(|_| FractionError::InvalidDecimal)?;
    if negative {
        numerator = -numerator;
    }
    let scale =
        i32::try_from(fractional.len()).map_err(|_| FractionError::InvalidDecimal)? - exponent;
    if scale >= 0 {
        Rational::new(numerator, pow10(scale as u32))
    } else {
        Rational::new(numerator * pow10((-scale) as u32), BigInt::from(1))
    }
}

#[must_use]
pub fn continued_fraction(value: &Rational) -> Vec<BigInt> {
    let mut numerator = value.numerator.clone();
    let mut denominator = value.denominator.clone();
    let mut terms = Vec::new();
    while denominator != BigInt::from(0) {
        let quotient = div_floor(&numerator, &denominator);
        let remainder = numerator - &quotient * &denominator;
        terms.push(quotient);
        numerator = denominator;
        denominator = remainder;
    }
    terms
}

pub fn convergents(terms: &[BigInt]) -> Vec<Rational> {
    let mut p0 = BigInt::from(0);
    let mut p1 = BigInt::from(1);
    let mut q0 = BigInt::from(1);
    let mut q1 = BigInt::from(0);
    let mut output = Vec::with_capacity(terms.len());
    for term in terms {
        let p2 = term * &p1 + &p0;
        let q2 = term * &q1 + &q0;
        output.push(
            Rational::new(p2.clone(), q2.clone())
                .expect("continued-fraction denominator is nonzero"),
        );
        (p0, p1) = (p1, p2);
        (q0, q1) = (q1, q2);
    }
    output
}

pub fn rationalize_decimal(
    input: &str,
    max_denominator: u64,
) -> Result<Rationalization, FractionError> {
    if max_denominator == 0 {
        return Err(FractionError::InvalidBound);
    }
    let exact = parse_decimal(input)?;
    let terms = continued_fraction(&exact);
    let limit = BigInt::from(max_denominator);
    let mut p0 = BigInt::from(0);
    let mut p1 = BigInt::from(1);
    let mut q0 = BigInt::from(1);
    let mut q1 = BigInt::from(0);
    for term in terms {
        let p2 = &term * &p1 + &p0;
        let q2 = &term * &q1 + &q0;
        if q2 > limit {
            let k = (&limit - &q0) / &q1;
            let bound = Rational::new(&p0 + &k * &p1, &q0 + &k * &q1)?;
            let previous = Rational::new(p1, q1)?;
            let bound_error = exact.sub(&bound).abs();
            let previous_error = exact.sub(&previous).abs();
            let fraction = if bound_error.cmp_value(&previous_error) == Ordering::Less {
                bound
            } else {
                previous
            };
            let absolute_error = exact.sub(&fraction).abs();
            return Ok(Rationalization {
                fraction,
                absolute_error,
                max_denominator,
            });
        }
        (p0, p1) = (p1, p2);
        (q0, q1) = (q1, q2);
    }
    Ok(Rationalization {
        fraction: exact.clone(),
        absolute_error: Rational::from_i64(0),
        max_denominator,
    })
}

/// Reconstructs using `A = B = floor(sqrt((m - 1) / 2))`.
///
/// Thus `2*A*B < m`, the conventional uniqueness condition. Zero is handled
/// canonically as `0/1`, including for the smallest supported modulus.
pub fn rational_reconstruct(
    residue: u64,
    modulus: u64,
) -> Result<Option<RationalReconstruction>, FractionError> {
    if modulus < 2 {
        return Err(FractionError::InvalidModulus);
    }
    let bound = integer_sqrt((modulus - 1) / 2);
    rational_reconstruct_bounded(residue, modulus, bound, bound.max(1))
}

/// Reconstructs a reduced `a/b` satisfying the requested bounds and
/// `a == residue*b (mod modulus)`.
pub fn rational_reconstruct_bounded(
    residue: u64,
    modulus: u64,
    max_numerator_abs: u64,
    max_denominator: u64,
) -> Result<Option<RationalReconstruction>, FractionError> {
    if modulus < 2 {
        return Err(FractionError::InvalidModulus);
    }
    if max_denominator == 0 {
        return Err(FractionError::InvalidBound);
    }
    let residue = residue % modulus;
    if residue == 0 {
        return Ok(Some(RationalReconstruction {
            numerator: 0,
            denominator: 1,
        }));
    }

    let mut old_r = i128::from(modulus);
    let mut r = i128::from(residue);
    let mut old_t = 0_i128;
    let mut t = 1_i128;
    while r.unsigned_abs() > u128::from(max_numerator_abs) && r != 0 {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);
        (old_t, t) = (t, old_t - quotient * t);
    }
    if r == 0 || t == 0 {
        return Ok(None);
    }

    let (numerator, denominator_abs) = if t < 0 { (-r, -t) } else { (r, t) };
    let Ok(denominator) = u64::try_from(denominator_abs) else {
        return Ok(None);
    };
    if numerator.unsigned_abs() > u128::from(max_numerator_abs)
        || denominator > max_denominator
        || gcd(numerator.unsigned_abs() as u64, denominator) != 1
    {
        return Ok(None);
    }

    let modulus_value = modulus;
    let modulus = Modulus::new(modulus_value).expect("modulus was validated");
    let context = ModCtx::new(modulus);
    if reduce_i128(numerator, modulus) != context.mul(residue, denominator % modulus_value) {
        return Ok(None);
    }
    Ok(Some(RationalReconstruction {
        numerator,
        denominator,
    }))
}

fn integer_sqrt(value: u64) -> u64 {
    if value < 2 {
        return value;
    }
    let mut low = 1_u64;
    let mut high = 1_u64 << (64 - value.leading_zeros()).div_ceil(2);
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        if middle <= value / middle {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

fn pow10(exponent: u32) -> BigInt {
    BigInt::from(10).pow(exponent)
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0) {
        -value
    } else {
        value.clone()
    }
}

fn bigint_gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    left = bigint_abs(&left);
    right = bigint_abs(&right);
    while right != BigInt::from(0) {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    if left == BigInt::from(0) {
        BigInt::from(1)
    } else {
        left
    }
}

fn div_floor(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder < BigInt::from(0) {
        quotient - 1
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continued_fractions_and_rationalization_are_exact() {
        let value = Rational::new(BigInt::from(355), BigInt::from(113)).unwrap();
        assert_eq!(
            continued_fraction(&value),
            vec![3.into(), 7.into(), 16.into()]
        );
        let result = rationalize_decimal("3.141592653589793", 10_000).unwrap();
        assert_eq!(result.fraction.to_string(), "355/113");
    }

    #[test]
    fn reconstruction_validates_the_modular_identity() {
        let result = rational_reconstruct(7, 101).unwrap().unwrap();
        assert_eq!(result.numerator, 7);
        assert_eq!(result.denominator, 1);
        assert_eq!(rational_reconstruct_bounded(50, 101, 1, 1), Ok(None));
    }
}
