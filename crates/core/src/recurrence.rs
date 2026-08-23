use crate::{DiscoveryError, PrimeField, berlekamp_massey_mod_prime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecurrenceError {
    EmptyRecurrence,
    InsufficientInitialTerms,
    InconsistentInitialTerms { index: usize },
    InsufficientSequence,
    InferenceFailed(DiscoveryError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferredRecurrenceResult {
    pub coefficients: Vec<u64>,
    pub order: usize,
    pub predicted_term: u64,
    pub terms_checked: usize,
    pub model_verified_on_supplied_prefix: bool,
    pub modulus: u64,
}

/// Computes `a_n` for `a_m = sum(c_j * a_(m-j-1))` over the supplied prime field.
pub fn linear_recurrence_nth_mod_prime(
    initial_terms: &[i128],
    coefficients: &[i128],
    n: u64,
    field: PrimeField,
) -> Result<u64, RecurrenceError> {
    let order = coefficients.len();
    if order == 0 {
        return Err(RecurrenceError::EmptyRecurrence);
    }
    if initial_terms.len() < order {
        return Err(RecurrenceError::InsufficientInitialTerms);
    }
    let recurrence = coefficients
        .iter()
        .map(|&value| field.normalize(value))
        .collect::<Vec<_>>();
    let supplied = initial_terms
        .iter()
        .map(|&value| field.normalize(value))
        .collect::<Vec<_>>();
    for index in order..supplied.len() {
        let expected = recurrence
            .iter()
            .enumerate()
            .fold(0, |sum, (offset, &coefficient)| {
                field.add(sum, field.mul(coefficient, supplied[index - offset - 1]))
            });
        if supplied[index] != expected {
            return Err(RecurrenceError::InconsistentInitialTerms { index });
        }
    }
    if n < supplied.len() as u64 {
        return Ok(supplied[n as usize]);
    }
    let initial = &supplied[..order];
    let weights = recurrence_power(n, &recurrence, field);
    Ok(weights
        .iter()
        .zip(initial.iter().copied())
        .fold(0, |sum, (&weight, value)| {
            field.add(sum, field.mul(weight, value))
        }))
}

pub fn infer_recurrence_nth_mod_prime(
    sequence: &[i128],
    n: u64,
    field: PrimeField,
) -> Result<InferredRecurrenceResult, RecurrenceError> {
    if sequence.len() < 2 {
        return Err(RecurrenceError::InsufficientSequence);
    }
    let coefficients =
        berlekamp_massey_mod_prime(sequence, field).map_err(RecurrenceError::InferenceFailed)?;
    if coefficients.is_empty() {
        if sequence.iter().all(|&value| field.normalize(value) == 0) {
            return Ok(InferredRecurrenceResult {
                coefficients,
                order: 0,
                predicted_term: 0,
                terms_checked: sequence.len(),
                model_verified_on_supplied_prefix: true,
                modulus: field.modulus(),
            });
        }
        return Err(RecurrenceError::InferenceFailed(DiscoveryError::NoRelation));
    }
    let signed_coefficients = coefficients
        .iter()
        .map(|&value| i128::from(value))
        .collect::<Vec<_>>();
    let predicted_term = linear_recurrence_nth_mod_prime(sequence, &signed_coefficients, n, field)?;
    Ok(InferredRecurrenceResult {
        order: coefficients.len(),
        coefficients,
        predicted_term,
        terms_checked: sequence.len(),
        model_verified_on_supplied_prefix: true,
        modulus: field.modulus(),
    })
}

fn recurrence_power(n: u64, recurrence: &[u64], field: PrimeField) -> Vec<u64> {
    let order = recurrence.len();
    let mut result = vec![0; order];
    result[0] = 1;
    let mut base = vec![0; order];
    if order == 1 {
        base[0] = recurrence[0];
    } else {
        base[1] = 1;
    }
    let mut exponent = n;
    while exponent > 0 {
        if exponent & 1 == 1 {
            result = multiply_reduce(&result, &base, recurrence, field);
        }
        exponent >>= 1;
        if exponent > 0 {
            base = multiply_reduce(&base, &base, recurrence, field);
        }
    }
    result
}

fn multiply_reduce(left: &[u64], right: &[u64], recurrence: &[u64], field: PrimeField) -> Vec<u64> {
    let order = recurrence.len();
    let mut product = vec![0; 2 * order - 1];
    for (left_degree, &left_value) in left.iter().enumerate() {
        if left_value == 0 {
            continue;
        }
        for (right_degree, &right_value) in right.iter().enumerate() {
            let target = left_degree + right_degree;
            product[target] = field.add(product[target], field.mul(left_value, right_value));
        }
    }
    for degree in (order..product.len()).rev() {
        let value = product[degree];
        if value == 0 {
            continue;
        }
        for (offset, &coefficient) in recurrence.iter().enumerate() {
            let target = degree - 1 - offset;
            product[target] = field.add(product[target], field.mul(value, coefficient));
        }
    }
    product.truncate(order);
    product
}
