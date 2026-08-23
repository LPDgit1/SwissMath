use swissmath_core::{
    PrimeField, RecurrenceError, infer_recurrence_nth_mod_prime, linear_recurrence_nth_mod_prime,
};

fn iterative(initial: &[i128], coefficients: &[i128], n: usize, field: PrimeField) -> u64 {
    if n < initial.len() {
        return field.normalize(initial[n]);
    }
    let order = coefficients.len();
    let mut values = initial
        .iter()
        .map(|&value| field.normalize(value))
        .collect::<Vec<_>>();
    while values.len() <= n {
        let index = values.len();
        let next = coefficients
            .iter()
            .enumerate()
            .fold(0, |sum, (offset, &coefficient)| {
                field.add(
                    sum,
                    field.mul(field.normalize(coefficient), values[index - offset - 1]),
                )
            });
        values.push(next);
        assert!(values.len() >= order);
    }
    values[n]
}

fn fibonacci_fast_doubling(n: u64, modulus: u64) -> u64 {
    fn pair(n: u64, modulus: u64) -> (u64, u64) {
        if n == 0 {
            return (0, 1);
        }
        let (a, b) = pair(n / 2, modulus);
        let m = u128::from(modulus);
        let two_b_minus_a = (2 * u128::from(b) + m - u128::from(a)) % m;
        let c = (u128::from(a) * two_b_minus_a % m) as u64;
        let d = ((u128::from(a) * u128::from(a) + u128::from(b) * u128::from(b)) % m) as u64;
        if n % 2 == 0 {
            (c, d)
        } else {
            (d, ((u128::from(c) + u128::from(d)) % m) as u64)
        }
    }
    pair(n, modulus).0
}

#[test]
fn supplied_recurrences_cover_edges_and_match_iteration() {
    for prime in [2, 5, 101] {
        let field = PrimeField::new(prime).unwrap();
        let cases = [
            (vec![3], vec![2]),
            (vec![0, 1], vec![1, 1]),
            (vec![1, 2, 3, 4], vec![2, -1, 3, 0]),
            (vec![0, 0, 0], vec![0, 0, 0]),
        ];
        for (initial, coefficients) in cases {
            for n in 0..80 {
                assert_eq!(
                    linear_recurrence_nth_mod_prime(&initial, &coefficients, n as u64, field)
                        .unwrap(),
                    iterative(&initial, &coefficients, n, field)
                );
            }
        }
    }
    let field = PrimeField::new(5).unwrap();
    assert_eq!(
        linear_recurrence_nth_mod_prime(&[1], &[], 10, field),
        Err(RecurrenceError::EmptyRecurrence)
    );
    assert_eq!(
        linear_recurrence_nth_mod_prime(&[1], &[1, 1], 10, field),
        Err(RecurrenceError::InsufficientInitialTerms)
    );
}

#[test]
fn huge_fibonacci_matches_independent_fast_doubling() {
    let field = PrimeField::new(1_000_000_007).unwrap();
    let n = 1_000_000_000_000_000_000;
    assert_eq!(
        linear_recurrence_nth_mod_prime(&[0, 1], &[1, 1], n, field).unwrap(),
        fibonacci_fast_doubling(n, field.modulus())
    );
}

#[test]
fn inferred_recurrence_is_explicitly_conditional_on_the_prefix() {
    let field = PrimeField::new(101).unwrap();
    let sequence = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
    let result = infer_recurrence_nth_mod_prime(&sequence, 50, field).unwrap();
    assert_eq!(result.coefficients, vec![1, 1]);
    assert_eq!(result.order, 2);
    assert_eq!(result.predicted_term, fibonacci_fast_doubling(50, 101));
    assert_eq!(result.terms_checked, sequence.len());
    assert!(result.model_verified_on_supplied_prefix);

    let zero = infer_recurrence_nth_mod_prime(&[0, 0, 0, 0], u64::MAX, field).unwrap();
    assert_eq!(zero.order, 0);
    assert_eq!(zero.predicted_term, 0);
}

#[test]
fn supplied_extra_terms_are_validated_and_observed_terms_are_returned() {
    let field = PrimeField::new(101).unwrap();
    let supplied = [0, 1, 1, 2, 3, 5, 8, 13];
    assert_eq!(
        linear_recurrence_nth_mod_prime(&supplied, &[1, 1], 6, field).unwrap(),
        8
    );
    assert_eq!(
        linear_recurrence_nth_mod_prime(&supplied, &[1, 1], 50, field).unwrap(),
        fibonacci_fast_doubling(50, 101)
    );

    let inconsistent = [0, 1, 1, 2, 4, 5];
    assert_eq!(
        linear_recurrence_nth_mod_prime(&inconsistent, &[1, 1], 3, field),
        Err(RecurrenceError::InconsistentInitialTerms { index: 4 })
    );
}
