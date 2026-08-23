use swissmath_core::{
    FiniteFieldError, FpLinearSystemSolution, FpMatrix, FpPolynomial, PrimeField,
};

fn matrix(field: PrimeField, rows: &[&[i128]]) -> FpMatrix {
    FpMatrix::new(
        field,
        &rows.iter().map(|row| row.to_vec()).collect::<Vec<_>>(),
    )
    .unwrap()
}

#[test]
fn prime_context_rejects_nonfields_and_normalizes_signed_values() {
    for composite in [0, 1, 4, 6, 9, 15, 21] {
        assert_eq!(
            PrimeField::new(composite),
            Err(FiniteFieldError::InvalidPrime)
        );
    }
    let field = PrimeField::new(5).unwrap();
    assert_eq!(field.normalize(-7), 3);
    for value in 1..field.modulus() {
        assert_eq!(field.mul(value, field.inverse(value).unwrap()), 1);
    }
}

#[test]
fn required_f5_matrix_smoke_and_singular_cases_are_exact() {
    let field = PrimeField::new(5).unwrap();
    let input = matrix(field, &[&[1, 2], &[3, 4]]);
    assert_eq!(input.determinant(field).unwrap(), 3);
    let inverse = input.inverse(field).unwrap();
    assert_eq!(inverse.to_rows(), vec![vec![3, 1], vec![4, 2]]);
    assert_eq!(
        input.mul(field, &inverse).unwrap().to_rows(),
        vec![vec![1, 0], vec![0, 1]]
    );

    let singular = matrix(field, &[&[1, 2], &[2, 4]]);
    assert_eq!(singular.determinant(field).unwrap(), 0);
    assert_eq!(singular.inverse(field), Err(FiniteFieldError::Singular));
    assert_eq!(singular.rank(field), 1);
}

#[test]
fn rectangular_kernel_and_system_classification_satisfy_residuals() {
    let field = PrimeField::new(5).unwrap();
    let rectangular = matrix(field, &[&[1, 2, 3], &[2, 4, 1]]);
    let basis = rectangular.kernel(field);
    assert_eq!(basis.len(), rectangular.columns() - rectangular.rank(field));
    for vector in &basis {
        let signed = vector
            .iter()
            .map(|&value| i128::from(value))
            .collect::<Vec<_>>();
        assert_eq!(rectangular.mul_vector(field, &signed).unwrap(), vec![0, 0]);
    }

    match rectangular.solve(field, &[1, 2]).unwrap() {
        FpLinearSystemSolution::Infinite {
            particular,
            kernel_basis,
        } => {
            let signed = particular
                .iter()
                .map(|&value| i128::from(value))
                .collect::<Vec<_>>();
            assert_eq!(rectangular.mul_vector(field, &signed).unwrap(), vec![1, 2]);
            assert_eq!(kernel_basis, basis);
        }
        other => panic!("expected infinitely many solutions, got {other:?}"),
    }
    assert_eq!(
        matrix(field, &[&[1, 1], &[1, 1]])
            .solve(field, &[0, 1])
            .unwrap(),
        FpLinearSystemSolution::None
    );
}

#[test]
fn exhaustive_two_by_two_matrices_match_closed_form_determinants() {
    for prime in [2, 3, 5] {
        let field = PrimeField::new(prime).unwrap();
        for a in 0..prime {
            for b in 0..prime {
                for c in 0..prime {
                    for d in 0..prime {
                        let input = matrix(
                            field,
                            &[
                                &[i128::from(a), i128::from(b)],
                                &[i128::from(c), i128::from(d)],
                            ],
                        );
                        let expected = field.sub(field.mul(a, d), field.mul(b, c));
                        assert_eq!(input.determinant(field).unwrap(), expected);
                        assert_eq!(input.inverse(field).is_ok(), expected != 0);
                        assert_eq!(input.rank(field) == 2, expected != 0);
                    }
                }
            }
        }
    }
}

#[test]
fn polynomial_operations_obey_division_bezout_and_calculus_identities() {
    let field = PrimeField::new(5).unwrap();
    let left = FpPolynomial::new(field, &[1, 2, 0, 1]);
    let right = FpPolynomial::new(field, &[4, 1]);
    let product = left.mul(field, &right);
    let (quotient, remainder) = product.div_rem(field, &left).unwrap();
    assert_eq!(quotient, right);
    assert!(remainder.is_zero());

    let common = FpPolynomial::new(field, &[4, 0, 1]);
    let first = common.mul(field, &FpPolynomial::new(field, &[1, 1]));
    let second = common.mul(field, &FpPolynomial::new(field, &[2, 1]));
    assert_eq!(first.gcd(field, &second).unwrap(), common);
    let extended = first.extended_gcd(field, &second).unwrap();
    assert_eq!(
        extended
            .left_coefficient
            .mul(field, &first)
            .add(field, &extended.right_coefficient.mul(field, &second)),
        extended.gcd
    );

    let x_to_five = FpPolynomial::new(field, &[0, 0, 0, 0, 0, 1]);
    assert!(x_to_five.derivative(field).is_zero());
    assert_eq!(left.evaluate(field, 2), 3);
}

#[test]
fn polynomial_powmod_and_canonical_remainders_match_repeated_multiplication() {
    for prime in [2, 3, 5, 7] {
        let field = PrimeField::new(prime).unwrap();
        let base = FpPolynomial::new(field, &[1, 1]);
        let modulus = FpPolynomial::new(field, &[1, 0, 1]);
        let mut repeated = FpPolynomial::new(field, &[1]);
        for exponent in 0..12 {
            assert_eq!(base.pow_mod(field, exponent, &modulus).unwrap(), repeated);
            repeated = repeated
                .mul(field, &base)
                .div_rem(field, &modulus)
                .unwrap()
                .1;
            assert!(repeated.degree().is_none_or(|degree| degree < 2));
            assert!(repeated.coefficients().iter().all(|&value| value < prime));
        }
    }
}
