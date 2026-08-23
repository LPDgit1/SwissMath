use num_bigint::BigInt;

use crate::{ModCtx, Modulus, Rational, finite_differences, interpolate, is_prime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    InsufficientData,
    InvalidInput,
    InvalidModulus,
    NoRelation,
    CoefficientLimit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegerRelation {
    pub coefficients: Vec<i64>,
    pub residual: f64,
    pub max_coefficient: u64,
    pub iterations: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurrenceCandidate {
    pub coefficients: Vec<i64>,
    pub order: usize,
    pub terms_checked: usize,
    pub exact: bool,
    pub modulus: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuessCandidate {
    pub kind: String,
    pub formula: String,
    pub reason: String,
    pub terms_checked: usize,
}

pub fn pslq(
    values: &[f64],
    tolerance: f64,
    coefficient_limit: u64,
    max_iterations: usize,
) -> Result<IntegerRelation, DiscoveryError> {
    let n = values.len();
    if n < 2
        || !tolerance.is_finite()
        || tolerance <= 0.0
        || coefficient_limit == 0
        || values.iter().any(|value| !value.is_finite())
    {
        return Err(DiscoveryError::InvalidInput);
    }
    let norm = values.iter().map(|value| value * value).sum::<f64>().sqrt();
    if norm == 0.0 {
        return Err(DiscoveryError::InvalidInput);
    }
    let mut y = values.iter().map(|value| value / norm).collect::<Vec<_>>();
    let mut suffix = vec![0.0; n];
    let mut total = 0.0;
    for index in (0..n).rev() {
        total = (total * total + y[index] * y[index]).sqrt();
        suffix[index] = total;
    }
    let mut h = vec![vec![0.0; n - 1]; n];
    for row in 0..n {
        for column in 0..n - 1 {
            if row == column {
                h[row][column] = suffix[column + 1] / suffix[column];
            } else if row > column {
                h[row][column] = -y[row] * y[column] / (suffix[column] * suffix[column + 1]);
            }
        }
    }
    let mut b = identity_i128(n);
    size_reduce(&mut y, &mut h, &mut b)?;
    for iteration in 1..=max_iterations {
        if let Some(relation) =
            relation_from_state(values, &y, &b, tolerance, coefficient_limit, iteration)
        {
            return Ok(relation);
        }
        let gamma = 2.0 / 3.0_f64.sqrt();
        let m = (0..n - 1)
            .max_by(|&left, &right| {
                (gamma.powi((left + 1) as i32) * h[left][left].abs())
                    .total_cmp(&(gamma.powi((right + 1) as i32) * h[right][right].abs()))
            })
            .ok_or(DiscoveryError::NoRelation)?;
        y.swap(m, m + 1);
        h.swap(m, m + 1);
        for row in &mut b {
            row.swap(m, m + 1);
        }
        if m + 1 < n - 1 {
            let first = h[m][m];
            let second = h[m][m + 1];
            let length = first.hypot(second);
            if length == 0.0 {
                return Err(DiscoveryError::NoRelation);
            }
            let cosine = first / length;
            let sine = second / length;
            for row in h.iter_mut().take(n).skip(m) {
                let left = row[m];
                let right = row[m + 1];
                row[m] = cosine * left + sine * right;
                row[m + 1] = -sine * left + cosine * right;
            }
        }
        size_reduce(&mut y, &mut h, &mut b)?;
    }
    Err(DiscoveryError::NoRelation)
}

pub fn berlekamp_massey(sequence: &[i64], modulus: u64) -> Result<Vec<i64>, DiscoveryError> {
    if sequence.len() < 2 {
        return Err(DiscoveryError::InsufficientData);
    }
    if modulus < 3 || !is_prime(modulus) {
        return Err(DiscoveryError::InvalidModulus);
    }
    let context = ModCtx::new(Modulus::new(modulus).expect("prime modulus is nonzero"));
    let values = sequence
        .iter()
        .map(|&value| i128::from(value).rem_euclid(i128::from(modulus)) as u64)
        .collect::<Vec<_>>();
    let mut c = vec![1_u64];
    let mut b = vec![1_u64];
    let mut order = 0_usize;
    let mut shift = 1_usize;
    let mut last_discrepancy = 1_u64;
    for index in 0..values.len() {
        let mut discrepancy = values[index];
        for coefficient in 1..=order {
            discrepancy = context.add(
                discrepancy,
                context.mul(c[coefficient], values[index - coefficient]),
            );
        }
        if discrepancy == 0 {
            shift += 1;
            continue;
        }
        let scale = context.mul(
            discrepancy,
            context
                .inv(last_discrepancy)
                .expect("nonzero field element has inverse"),
        );
        let previous = c.clone();
        if c.len() < b.len() + shift {
            c.resize(b.len() + shift, 0);
        }
        for (position, &coefficient) in b.iter().enumerate() {
            let target = position + shift;
            c[target] = context.sub(c[target], context.mul(scale, coefficient));
        }
        if 2 * order <= index {
            order = index + 1 - order;
            b = previous;
            last_discrepancy = discrepancy;
            shift = 1;
        } else {
            shift += 1;
        }
    }
    let coefficients = (1..=order)
        .map(|index| {
            let value = if c[index] == 0 { 0 } else { modulus - c[index] };
            if value > modulus / 2 {
                i64::try_from(i128::from(value) - i128::from(modulus))
                    .expect("signed residue fits i64")
            } else {
                i64::try_from(value).expect("selected modulus fits i64")
            }
        })
        .collect();
    Ok(coefficients)
}

pub fn find_recurrence(sequence: &[i64]) -> Result<RecurrenceCandidate, DiscoveryError> {
    const MODULUS: u64 = 1_000_000_007;
    let coefficients = berlekamp_massey(sequence, MODULUS)?;
    if coefficients.is_empty() || coefficients.len() * 2 > sequence.len() {
        return Err(DiscoveryError::InsufficientData);
    }
    let exact = validate_recurrence(sequence, &coefficients);
    if !exact {
        return Err(DiscoveryError::NoRelation);
    }
    Ok(RecurrenceCandidate {
        order: coefficients.len(),
        coefficients,
        terms_checked: sequence.len(),
        exact,
        modulus: MODULUS,
    })
}

pub fn guess_sequence(sequence: &[i64]) -> Result<Vec<GuessCandidate>, DiscoveryError> {
    if sequence.len() < 2 {
        return Err(DiscoveryError::InsufficientData);
    }
    let mut candidates = Vec::new();
    if sequence.iter().all(|value| value == &sequence[0]) {
        candidates.push(GuessCandidate {
            kind: "Constant sequence".into(),
            formula: format!("a(n) = {}", sequence[0]),
            reason: format!("Exact match for all {} supplied terms", sequence.len()),
            terms_checked: sequence.len(),
        });
        return Ok(candidates);
    }
    let difference = i128::from(sequence[1]) - i128::from(sequence[0]);
    if sequence
        .windows(2)
        .all(|pair| i128::from(pair[1]) - i128::from(pair[0]) == difference)
    {
        candidates.push(GuessCandidate {
            kind: "Arithmetic progression".into(),
            formula: format!("a(n) = {} + (n-1)·({difference})", sequence[0]),
            reason: format!(
                "Constant first difference; exact match for {0}/{0} terms",
                sequence.len()
            ),
            terms_checked: sequence.len(),
        });
    }
    if sequence[0] != 0 {
        let numerator = sequence[1];
        let denominator = sequence[0];
        if sequence.windows(2).all(|pair| {
            i128::from(pair[1]) * i128::from(denominator)
                == i128::from(pair[0]) * i128::from(numerator)
        }) {
            candidates.push(GuessCandidate {
                kind: "Geometric progression".into(),
                formula: format!(
                    "a(n) = {}·({}/{})^(n-1)",
                    sequence[0], numerator, denominator
                ),
                reason: format!(
                    "Constant exact ratio; exact match for {0}/{0} terms",
                    sequence.len()
                ),
                terms_checked: sequence.len(),
            });
        }
    }
    let big_values = sequence
        .iter()
        .copied()
        .map(BigInt::from)
        .collect::<Vec<_>>();
    let differences = finite_differences(&big_values);
    if let Some(degree) = differences.polynomial_degree {
        let points = sequence
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                (
                    Rational::from_i64((index + 1) as i64),
                    Rational::from_i64(value),
                )
            })
            .collect::<Vec<_>>();
        if let Ok(polynomial) = interpolate(&points) {
            candidates.push(GuessCandidate {
                kind: "Polynomial sequence".into(),
                formula: format!("a(n) = {}", polynomial.format_human("n")),
                reason: format!(
                    "Constant difference of order {degree}; exact match for {0}/{0} terms",
                    sequence.len()
                ),
                terms_checked: sequence.len(),
            });
        }
    }
    if let Ok(recurrence) = find_recurrence(sequence) {
        let terms = recurrence
            .coefficients
            .iter()
            .enumerate()
            .map(|(index, coefficient)| format!("{coefficient}·a(n-{})", index + 1))
            .collect::<Vec<_>>()
            .join(" + ");
        candidates.push(GuessCandidate {
            kind: "Linear recurrence".into(),
            formula: format!("a(n) = {terms}"),
            reason: format!(
                "Exact match for {terms}/{terms} terms; order {order}",
                terms = sequence.len(),
                order = recurrence.order
            ),
            terms_checked: sequence.len(),
        });
    }
    candidates.truncate(3);
    Ok(candidates)
}

fn validate_recurrence(sequence: &[i64], coefficients: &[i64]) -> bool {
    (coefficients.len()..sequence.len()).all(|index| {
        coefficients
            .iter()
            .enumerate()
            .try_fold(0_i128, |sum, (offset, coefficient)| {
                sum.checked_add(i128::from(*coefficient) * i128::from(sequence[index - offset - 1]))
            })
            == Some(i128::from(sequence[index]))
    })
}

fn identity_i128(size: usize) -> Vec<Vec<i128>> {
    (0..size)
        .map(|row| (0..size).map(|column| i128::from(row == column)).collect())
        .collect()
}

fn size_reduce(
    y: &mut [f64],
    h: &mut [Vec<f64>],
    b: &mut [Vec<i128>],
) -> Result<(), DiscoveryError> {
    let n = y.len();
    for row in 1..n {
        for column in (0..row.min(n - 1)).rev() {
            if h[column][column].abs() <= f64::EPSILON {
                return Err(DiscoveryError::NoRelation);
            }
            let multiplier = (h[row][column] / h[column][column]).round();
            if !multiplier.is_finite() || multiplier.abs() > i128::MAX as f64 {
                return Err(DiscoveryError::CoefficientLimit);
            }
            let integer = multiplier as i128;
            if integer == 0 {
                continue;
            }
            y[column] += multiplier * y[row];
            let (previous_rows, current_rows) = h.split_at_mut(row);
            let pivot = &previous_rows[column];
            let current = &mut current_rows[0];
            for (entry, pivot_entry) in current[..=column].iter_mut().zip(&pivot[..=column]) {
                *entry -= multiplier * pivot_entry;
            }
            for b_row in b.iter_mut() {
                b_row[column] = b_row[column]
                    .checked_add(
                        integer
                            .checked_mul(b_row[row])
                            .ok_or(DiscoveryError::CoefficientLimit)?,
                    )
                    .ok_or(DiscoveryError::CoefficientLimit)?;
            }
        }
    }
    Ok(())
}

fn relation_from_state(
    values: &[f64],
    y: &[f64],
    b: &[Vec<i128>],
    tolerance: f64,
    limit: u64,
    iterations: usize,
) -> Option<IntegerRelation> {
    y.iter()
        .enumerate()
        .filter(|(_, value)| value.abs() <= tolerance)
        .find_map(|(column, _)| {
            let raw = b.iter().map(|row| row[column]).collect::<Vec<_>>();
            let common = raw
                .iter()
                .fold(0_i128, |accumulator, value| gcd_i128(accumulator, *value));
            if common == 0 {
                return None;
            }
            let mut normalized = raw
                .into_iter()
                .map(|value| value / common)
                .collect::<Vec<_>>();
            if normalized
                .iter()
                .find(|value| **value != 0)
                .is_some_and(|value| *value < 0)
            {
                for value in &mut normalized {
                    *value = -*value;
                }
            }
            let coefficients = normalized
                .into_iter()
                .map(i64::try_from)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            let max_coefficient = coefficients
                .iter()
                .map(|value| value.unsigned_abs())
                .max()
                .unwrap_or(0);
            if max_coefficient > limit {
                return None;
            }
            let residual = coefficients
                .iter()
                .zip(values)
                .map(|(coefficient, value)| *coefficient as f64 * value)
                .sum::<f64>()
                .abs();
            (residual <= tolerance * values.iter().map(|value| value.abs()).sum::<f64>().max(1.0))
                .then_some(IntegerRelation {
                    coefficients,
                    residual,
                    max_coefficient,
                    iterations,
                })
        })
}

fn gcd_i128(mut left: i128, mut right: i128) -> i128 {
    left = left.abs();
    right = right.abs();
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibonacci_recurrence_and_sequence_guesses_are_exact() {
        let sequence = [0, 1, 1, 2, 3, 5, 8, 13];
        let recurrence = find_recurrence(&sequence).unwrap();
        assert_eq!(recurrence.coefficients, vec![1, 1]);
        let guesses = guess_sequence(&[1, 4, 9, 16, 25, 36]).unwrap();
        assert!(
            guesses
                .iter()
                .any(|candidate| candidate.kind == "Polynomial sequence")
        );
    }

    #[test]
    fn pslq_finds_a_small_relation_and_reports_residual() {
        let relation = pslq(
            &[1.0, 2.0_f64.sqrt(), 2.0 * 2.0_f64.sqrt()],
            1e-12,
            100,
            1_000,
        )
        .unwrap();
        assert!(relation.residual < 1e-12);
        assert!(relation.coefficients.iter().any(|value| *value != 0));
    }
}
