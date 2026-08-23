use core::cmp::Ordering;

use crate::{ArithmeticError, gcd};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtendedGcd {
    pub gcd: u64,
    pub x: i128,
    pub y: i128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegerRoot {
    pub floor: u128,
    pub exact: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PerfectPower {
    pub base: u128,
    pub exponent: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseConversionError {
    InvalidBase,
    InvalidDigit,
    Overflow,
}

pub fn lcm(a: u64, b: u64) -> Result<u64, ArithmeticError> {
    if a == 0 || b == 0 {
        return Ok(0);
    }
    (a / gcd(a, b))
        .checked_mul(b)
        .ok_or(ArithmeticError::Overflow)
}

#[must_use]
pub fn extended_gcd(a: u64, b: u64) -> ExtendedGcd {
    // For u64 inputs, Euclidean remainders and Bezout coefficients fit in
    // i128. Keeping the signed arithmetic local avoids a BigInt dependency in
    // this hot primitive while covering the complete supported domain.
    let mut old_r = i128::from(a);
    let mut r = i128::from(b);
    let mut old_s = 1_i128;
    let mut s = 0_i128;
    let mut old_t = 0_i128;
    let mut t = 1_i128;
    while r != 0 {
        let quotient = old_r.div_euclid(r);
        (old_r, r) = (r, old_r - quotient * r);
        (old_s, s) = (s, old_s - quotient * s);
        (old_t, t) = (t, old_t - quotient * t);
    }
    ExtendedGcd {
        gcd: old_r as u64,
        x: old_s,
        y: old_t,
    }
}

pub fn integer_nth_root(value: u128, degree: u32) -> Option<IntegerRoot> {
    if degree == 0 {
        return None;
    }
    if degree == 1 || value < 2 {
        return Some(IntegerRoot {
            floor: value,
            exact: true,
        });
    }
    let bits = 128 - value.leading_zeros();
    let upper_bits = bits.div_ceil(degree);
    let mut low = 0_u128;
    let mut high = if upper_bits >= 128 {
        u128::MAX
    } else {
        1_u128 << upper_bits
    };
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        match pow_cmp(middle, degree, value) {
            Ordering::Greater => high = middle,
            Ordering::Equal | Ordering::Less => low = middle,
        }
    }
    Some(IntegerRoot {
        floor: low,
        exact: pow_cmp(low, degree, value) == Ordering::Equal,
    })
}

#[must_use]
pub fn perfect_power(value: u128) -> Option<PerfectPower> {
    if value < 4 {
        return None;
    }
    let max_exponent = 127 - value.leading_zeros();
    for exponent in (2..=max_exponent).rev() {
        let root = integer_nth_root(value, exponent)?;
        if root.exact && root.floor > 1 {
            return Some(PerfectPower {
                base: root.floor,
                exponent,
            });
        }
    }
    None
}

pub fn parse_in_base(input: &str, base: u32) -> Result<u128, BaseConversionError> {
    if !(2..=36).contains(&base) {
        return Err(BaseConversionError::InvalidBase);
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(BaseConversionError::InvalidDigit);
    }
    let mut value = 0_u128;
    for character in trimmed.chars() {
        let digit = character
            .to_digit(base)
            .ok_or(BaseConversionError::InvalidDigit)?;
        value = value
            .checked_mul(u128::from(base))
            .and_then(|current| current.checked_add(u128::from(digit)))
            .ok_or(BaseConversionError::Overflow)?;
    }
    Ok(value)
}

pub fn format_in_base(mut value: u128, base: u32) -> Result<String, BaseConversionError> {
    if !(2..=36).contains(&base) {
        return Err(BaseConversionError::InvalidBase);
    }
    if value == 0 {
        return Ok("0".to_owned());
    }
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut output = Vec::new();
    while value != 0 {
        output.push(DIGITS[(value % u128::from(base)) as usize]);
        value /= u128::from(base);
    }
    output.reverse();
    Ok(String::from_utf8(output).expect("digits are ASCII"))
}

fn pow_cmp(mut base: u128, mut exponent: u32, limit: u128) -> Ordering {
    let mut result = 1_u128;
    while exponent != 0 {
        if exponent & 1 != 0 {
            let Some(product) = result.checked_mul(base) else {
                return Ordering::Greater;
            };
            if product > limit {
                return Ordering::Greater;
            }
            result = product;
        }
        exponent >>= 1;
        if exponent != 0 {
            let Some(square) = base.checked_mul(base) else {
                return Ordering::Greater;
            };
            if square > limit && exponent != 0 {
                base = limit.saturating_add(1);
            } else {
                base = square;
            }
        }
    }
    result.cmp(&limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_powers_and_bases_are_exact() {
        assert_eq!(
            integer_nth_root(80, 3),
            Some(IntegerRoot {
                floor: 4,
                exact: false
            })
        );
        assert_eq!(
            integer_nth_root(81, 4),
            Some(IntegerRoot {
                floor: 3,
                exact: true
            })
        );
        assert_eq!(
            perfect_power(64),
            Some(PerfectPower {
                base: 2,
                exponent: 6
            })
        );
        assert_eq!(parse_in_base("ff", 16), Ok(255));
        assert_eq!(format_in_base(255, 2).as_deref(), Ok("11111111"));
    }

    #[test]
    fn extended_identity_and_lcm_hold() {
        let result = extended_gcd(240, 46);
        assert_eq!(result.gcd, 2);
        assert_eq!(240_i128 * result.x + 46_i128 * result.y, 2);
        assert_eq!(lcm(21, 6), Ok(42));
    }
}
