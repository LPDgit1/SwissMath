use swissmath_core::{
    FractionError, Modulus, NumberTheoryError, Valuation, extended_gcd, factor, inv_mod, is_prime,
    next_prime, previous_prime, rational_reconstruct, rational_reconstruct_bounded, reduce_i128,
    valuation,
};

fn brute_divisors(n: u64) -> Vec<u64> {
    (1..=n).filter(|candidate| n % candidate == 0).collect()
}

fn brute_mobius(n: u64) -> i8 {
    let mut remaining = n;
    let mut prime_count = 0_u32;
    let mut p = 2_u64;
    while p <= remaining / p {
        if remaining % p == 0 {
            remaining /= p;
            if remaining % p == 0 {
                return 0;
            }
            prime_count += 1;
        }
        p += 1;
    }
    if remaining > 1 {
        prime_count += 1;
    }
    if prime_count % 2 == 0 { 1 } else { -1 }
}

#[test]
fn extended_gcd_covers_zero_equal_coprime_and_large_u64_inputs() {
    for (a, b) in [
        (0, 0),
        (0, 37),
        (37, 0),
        (91, 91),
        (35, 64),
        (u64::MAX, u64::MAX - 1),
        (u64::MAX, 0x8000_0000_0000_0000),
    ] {
        let result = extended_gcd(a, b);
        assert_eq!(
            i128::from(a) * result.x + i128::from(b) * result.y,
            i128::from(result.gcd),
            "a={a}, b={b}"
        );
        assert_eq!(a % result.gcd.max(1), 0);
        assert_eq!(b % result.gcd.max(1), 0);
    }
}

#[test]
fn valuation_is_exact_and_zero_is_explicitly_infinite() {
    assert_eq!(valuation(0, 2), Ok(Valuation::Infinite));
    assert_eq!(valuation(1, 2), Ok(Valuation::Finite(0)));
    assert_eq!(valuation(1_u64 << 63, 2), Ok(Valuation::Finite(63)));
    assert_eq!(valuation(3_u64.pow(20) * 10, 3), Ok(Valuation::Finite(20)));
    assert_eq!(valuation(81, 1), Err(NumberTheoryError::NonPrimeBase));
    assert_eq!(valuation(81, 9), Err(NumberTheoryError::NonPrimeBase));
}

#[test]
fn factorization_derived_functions_match_small_independent_references() {
    for n in 1..=500_u64 {
        let factorization = factor(n).unwrap();
        let expected = brute_divisors(n);
        let expected_sum = expected
            .iter()
            .map(|&value| u128::from(value))
            .sum::<u128>();
        let expected_radical = factorization
            .factors()
            .iter()
            .map(|factor| factor.prime)
            .product::<u64>();
        assert_eq!(factorization.mobius(), brute_mobius(n), "mu({n})");
        assert_eq!(factorization.radical(), expected_radical, "rad({n})");
        assert_eq!(factorization.is_squarefree(), brute_mobius(n) != 0, "n={n}");
        assert_eq!(
            factorization.divisor_count(),
            expected.len() as u64,
            "tau({n})"
        );
        assert_eq!(factorization.divisor_sum(), expected_sum, "sigma({n})");
        assert_eq!(factorization.divisors(), expected, "divisors({n})");
        assert_eq!(
            factorization.divisors().len() as u64,
            factorization.divisor_count()
        );
        assert!(
            factorization
                .divisors()
                .iter()
                .all(|divisor| n % divisor == 0)
        );
    }
}

#[test]
fn prime_navigation_matches_an_independent_small_oracle_and_boundaries() {
    for n in 0..=10_000_u64 {
        let expected_next = (n + 1..).find(|&candidate| is_prime(candidate)).unwrap();
        let expected_previous = (2..n).rev().find(|&candidate| is_prime(candidate));
        assert_eq!(next_prime(n), Ok(expected_next), "next after {n}");
        assert_eq!(previous_prime(n), expected_previous, "previous before {n}");
    }

    const LARGEST_U64_PRIME: u64 = 18_446_744_073_709_551_557;
    assert_eq!(next_prime(LARGEST_U64_PRIME - 1), Ok(LARGEST_U64_PRIME));
    assert_eq!(
        next_prime(LARGEST_U64_PRIME),
        Err(NumberTheoryError::Overflow)
    );
    assert_eq!(next_prime(u64::MAX), Err(NumberTheoryError::Overflow));
    assert_eq!(previous_prime(u64::MAX), Some(LARGEST_U64_PRIME));
}

#[test]
fn rational_reconstruction_handles_exact_signed_corpus_and_failures() {
    let modulus = 10_009_u64;
    for (numerator, denominator) in [(0_i128, 1_u64), (42, 1), (3, 7), (-5, 9)] {
        let inverse = inv_mod(denominator, Modulus::new(modulus).unwrap()).unwrap();
        let residue = ((u128::from(reduce_i128(numerator, Modulus::new(modulus).unwrap()))
            * u128::from(inverse))
            % u128::from(modulus)) as u64;
        let reconstructed = rational_reconstruct(residue, modulus).unwrap().unwrap();
        assert_eq!(reconstructed.numerator, numerator);
        assert_eq!(reconstructed.denominator, denominator);
    }

    assert_eq!(
        rational_reconstruct_bounded(0, 101, 0, 1)
            .unwrap()
            .unwrap()
            .to_string(),
        "0"
    );
    assert_eq!(rational_reconstruct_bounded(50, 101, 1, 1), Ok(None));
    assert_eq!(
        rational_reconstruct_bounded(1, 101, 10, 0),
        Err(FractionError::InvalidBound)
    );
    assert_eq!(
        rational_reconstruct(1, 1),
        Err(FractionError::InvalidModulus)
    );
}
