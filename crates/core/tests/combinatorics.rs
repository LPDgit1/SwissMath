use swissmath_core::{
    CombinatoricsError, PrimeField, binomial_mod_prime, binomial_valuation, factorial_mod_prime,
    factorial_valuation,
};

fn pascal_row(n: usize, field: PrimeField) -> Vec<u64> {
    let mut row = vec![1_u64];
    for _ in 0..n {
        let mut next = vec![1_u64; row.len() + 1];
        for index in 1..row.len() {
            next[index] = field.add(row[index - 1], row[index]);
        }
        row = next;
    }
    row
}

fn direct_factorial(n: u64, field: PrimeField) -> u64 {
    (2..=n).fold(1, |product, value| field.mul(product, value))
}

#[test]
fn legendre_and_kummer_match_required_values_and_independent_identity() {
    assert_eq!(factorial_valuation(10, PrimeField::new(2).unwrap()), 8);
    assert_eq!(factorial_valuation(100, PrimeField::new(5).unwrap()), 24);
    assert_eq!(
        factorial_valuation(1_000_000_000_000_000_000, PrimeField::new(2).unwrap()),
        999_999_999_999_999_976
    );

    for prime in [2, 3, 5, 7, 11, 13] {
        let field = PrimeField::new(prime).unwrap();
        for n in 0..=300 {
            for k in 0..=n {
                let legendre = factorial_valuation(n, field)
                    - factorial_valuation(k, field)
                    - factorial_valuation(n - k, field);
                assert_eq!(binomial_valuation(n, k, field), Ok(legendre));
            }
        }
    }
    assert_eq!(
        binomial_valuation(3, 4, PrimeField::new(5).unwrap()),
        Err(CombinatoricsError::KExceedsN)
    );
}

#[test]
fn lucas_matches_pascal_exhaustively_across_multiple_digits() {
    for prime in [2, 3, 5, 7, 11, 13] {
        let field = PrimeField::new(prime).unwrap();
        for n in 0..=120_usize {
            let row = pascal_row(n, field);
            for (k, &expected) in row.iter().enumerate() {
                assert_eq!(
                    binomial_mod_prime(n as u64, k as u64, field),
                    Ok(expected),
                    "p={prime}, n={n}, k={k}"
                );
            }
        }
    }
    assert_eq!(
        binomial_mod_prime(10, 3, PrimeField::new(7).unwrap()),
        Ok(1)
    );
}

#[test]
fn huge_binomial_identities_depend_on_digits_not_integer_size() {
    let field = PrimeField::new(1_000_003).unwrap();
    let n = 1_000_000_000_000_000_000;
    assert_eq!(binomial_mod_prime(n, 0, field), Ok(1));
    assert_eq!(binomial_mod_prime(n, n, field), Ok(1));
    assert_eq!(binomial_mod_prime(n, 1, field), Ok(n % field.modulus()));
    assert_eq!(
        binomial_mod_prime(1_u64 << 60, 123_456_789, PrimeField::new(2).unwrap()),
        Ok(0)
    );
    assert_eq!(
        binomial_mod_prime(5_u64.pow(27), 17, PrimeField::new(5).unwrap()),
        Ok(0)
    );
    assert_eq!(binomial_mod_prime(10, 11, field), Ok(0));
}

#[test]
fn factorial_residues_match_direct_oracle_and_wilson_identities() {
    for prime in [2, 3, 5, 7, 11, 13, 97] {
        let field = PrimeField::new(prime).unwrap();
        for n in 0..prime {
            assert_eq!(
                factorial_mod_prime(n, field),
                Ok(direct_factorial(n, field))
            );
        }
        assert_eq!(factorial_mod_prime(prime, field), Ok(0));
        assert_eq!(factorial_mod_prime(prime + 10, field), Ok(0));
        assert_eq!(factorial_mod_prime(prime - 1, field), Ok(prime - 1));
        if prime > 2 {
            assert_eq!(factorial_mod_prime(prime - 2, field), Ok(1));
        }
    }
}
