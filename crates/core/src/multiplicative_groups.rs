use std::collections::HashMap;

use crate::{
    Congruence, Modulus, MultiplicativeOrderResult, PrimeField, crt_fold, factor,
    multiplicative_order,
};

const MAX_BSGS_BABY_STEPS: u64 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MultiplicativeGroupError {
    ZeroElement,
    FactorizationFailed,
    ArithmeticFailure,
    InternalVerificationFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscreteLogResult {
    Solved { x: u64, order: u64 },
    NoSolution { order: u64 },
    SearchLimitReached { order: u64 },
}

pub fn is_primitive_root(g: i128, field: PrimeField) -> Result<bool, MultiplicativeGroupError> {
    let candidate = field.normalize(g);
    if candidate == 0 {
        return Ok(false);
    }
    if field.modulus() == 2 {
        return Ok(candidate == 1);
    }
    let factors =
        factor(field.modulus() - 1).map_err(|_| MultiplicativeGroupError::FactorizationFailed)?;
    Ok(is_generator_with_factors(candidate, field, &factors))
}

pub fn primitive_root(field: PrimeField) -> Result<u64, MultiplicativeGroupError> {
    if field.modulus() == 2 {
        return Ok(1);
    }
    let factors =
        factor(field.modulus() - 1).map_err(|_| MultiplicativeGroupError::FactorizationFailed)?;
    (2..field.modulus())
        .find(|&candidate| is_generator_with_factors(candidate, field, &factors))
        .ok_or(MultiplicativeGroupError::InternalVerificationFailed)
}

pub fn discrete_log(
    g: i128,
    h: i128,
    field: PrimeField,
) -> Result<DiscreteLogResult, MultiplicativeGroupError> {
    discrete_log_bounded(g, h, field, MAX_BSGS_BABY_STEPS)
}

fn discrete_log_bounded(
    g: i128,
    h: i128,
    field: PrimeField,
    baby_step_limit: u64,
) -> Result<DiscreteLogResult, MultiplicativeGroupError> {
    let base = field.normalize(g);
    let target = field.normalize(h);
    if base == 0 || target == 0 {
        return Err(MultiplicativeGroupError::ZeroElement);
    }
    let order = match multiplicative_order(base, field.modulus())
        .map_err(|_| MultiplicativeGroupError::FactorizationFailed)?
    {
        MultiplicativeOrderResult::Exists(order) => order,
        MultiplicativeOrderResult::DoesNotExist => {
            return Err(MultiplicativeGroupError::ZeroElement);
        }
    };
    if field.pow(target, order) != 1 {
        return Ok(DiscreteLogResult::NoSolution { order });
    }
    let order_factors = factor(order).map_err(|_| MultiplicativeGroupError::FactorizationFailed)?;
    if order_factors
        .factors()
        .iter()
        .any(|factor| ceil_sqrt(factor.prime) > baby_step_limit)
    {
        return Ok(DiscreteLogResult::SearchLimitReached { order });
    }

    let mut congruences = Vec::with_capacity(order_factors.factors().len());
    for prime_power in order_factors.factors() {
        let prime = prime_power.prime;
        let digit_base = field.pow(base, order / prime);
        let mut residue = 0_u64;
        let mut power = 1_u64;
        for _ in 0..prime_power.exponent {
            let next_power = power
                .checked_mul(prime)
                .ok_or(MultiplicativeGroupError::ArithmeticFailure)?;
            let base_to_residue = field.pow(base, residue);
            let inverse = field
                .inverse(base_to_residue)
                .ok_or(MultiplicativeGroupError::InternalVerificationFailed)?;
            let adjusted = field.mul(target, inverse);
            let digit_target = field.pow(adjusted, order / next_power);
            let digit =
                match baby_step_giant_step(digit_base, digit_target, prime, field, baby_step_limit)
                {
                    BsgsResult::Solved(value) => value,
                    BsgsResult::NoSolution => {
                        return Err(MultiplicativeGroupError::InternalVerificationFailed);
                    }
                    BsgsResult::LimitExceeded => {
                        return Ok(DiscreteLogResult::SearchLimitReached { order });
                    }
                };
            residue = residue
                .checked_add(
                    digit
                        .checked_mul(power)
                        .ok_or(MultiplicativeGroupError::ArithmeticFailure)?,
                )
                .ok_or(MultiplicativeGroupError::ArithmeticFailure)?;
            power = next_power;
        }
        congruences.push(Congruence::new(
            residue,
            Modulus::new(power).expect("a prime power is nonzero"),
        ));
    }
    let combined = crt_fold(congruences)
        .map_err(|_| MultiplicativeGroupError::ArithmeticFailure)?
        .ok_or(MultiplicativeGroupError::InternalVerificationFailed)?;
    let x = combined.residue() % order;
    if field.pow(base, x) != target {
        return Err(MultiplicativeGroupError::InternalVerificationFailed);
    }
    Ok(DiscreteLogResult::Solved { x, order })
}

fn is_generator_with_factors(
    candidate: u64,
    field: PrimeField,
    factors: &crate::Factorization,
) -> bool {
    let order = field.modulus() - 1;
    factors
        .factors()
        .iter()
        .all(|factor| field.pow(candidate, order / factor.prime) != 1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BsgsResult {
    Solved(u64),
    NoSolution,
    LimitExceeded,
}

fn baby_step_giant_step(
    base: u64,
    target: u64,
    known_order: u64,
    field: PrimeField,
    baby_step_limit: u64,
) -> BsgsResult {
    let width = ceil_sqrt(known_order);
    if width > baby_step_limit {
        return BsgsResult::LimitExceeded;
    }
    let mut baby_steps = HashMap::with_capacity(width as usize);
    let mut value = 1_u64;
    for exponent in 0..width {
        baby_steps.entry(value).or_insert(exponent);
        value = field.mul(value, base);
    }
    let base_width = field.pow(base, width);
    let Some(giant_factor) = field.inverse(base_width) else {
        return BsgsResult::NoSolution;
    };
    let mut giant = target;
    for giant_index in 0..=width {
        if let Some(&baby_exponent) = baby_steps.get(&giant) {
            let candidate = u128::from(giant_index) * u128::from(width) + u128::from(baby_exponent);
            if candidate < u128::from(known_order) {
                return BsgsResult::Solved(candidate as u64);
            }
        }
        giant = field.mul(giant, giant_factor);
    }
    BsgsResult::NoSolution
}

fn ceil_sqrt(value: u64) -> u64 {
    if value <= 1 {
        return value;
    }
    let mut root = (value as f64).sqrt() as u64;
    while u128::from(root + 1) * u128::from(root + 1) <= u128::from(value) {
        root += 1;
    }
    while u128::from(root) * u128::from(root) > u128::from(value) {
        root -= 1;
    }
    if u128::from(root) * u128::from(root) == u128::from(value) {
        root
    } else {
        root + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_path_reports_limit_before_allocating() {
        let field = PrimeField::new(23).unwrap();
        assert_eq!(
            discrete_log_bounded(5, 10, field, 2).unwrap(),
            DiscreteLogResult::SearchLimitReached { order: 22 }
        );
        assert_eq!(
            baby_step_giant_step(2, 8, 11, field, 2),
            BsgsResult::LimitExceeded
        );
        assert_eq!(ceil_sqrt(u64::MAX), 4_294_967_296);
    }
}
