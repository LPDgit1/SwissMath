use num_bigint::BigUint;
use swissmath_core::{
    DecimalIntegerAnalysis, IntegerClassification, ModCtx, Modulus, MultiplicativeOrderResult,
    NumberTheoryError, PrimalityAssessment, analyze_integer_decimal, assess_primality_decimal,
    assess_primality_u128, factor, gcd, is_prime, multiplicative_order,
};

fn reference_is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n % 2 == 0 {
        return n == 2;
    }
    let mut divisor = 3_u64;
    while divisor <= n / divisor {
        if n % divisor == 0 {
            return false;
        }
        divisor += 2;
    }
    true
}

fn reference_factors(mut n: u64) -> Vec<(u64, u32)> {
    let mut factors = Vec::new();
    let mut divisor = 2_u64;
    while divisor <= n / divisor {
        if n % divisor == 0 {
            let mut exponent = 0;
            while n % divisor == 0 {
                n /= divisor;
                exponent += 1;
            }
            factors.push((divisor, exponent));
        }
        divisor += u64::from(divisor == 2).max(1);
    }
    if n > 1 {
        factors.push((n, 1));
    }
    factors
}

fn reconstruct(factorization: &swissmath_core::Factorization) -> u128 {
    factorization
        .factors()
        .iter()
        .map(|factor| u128::from(factor.prime).pow(factor.exponent))
        .product()
}

#[test]
fn primality_matches_independent_trial_division() {
    for n in 0..=100_000 {
        assert_eq!(is_prime(n), reference_is_prime(n), "n={n}");
    }

    for (n, expected) in [
        (0, false),
        (1, false),
        (2, true),
        (3, true),
        (4, false),
        (49, false),
        (561, false),
        (1_105, false),
        (1_729, false),
        (341_550_071_728_321, false),
        (18_446_744_073_709_551_557, true),
        (u64::MAX - 1, false),
        (u64::MAX, false),
    ] {
        assert_eq!(is_prime(n), expected, "target n={n}");
    }
}

#[test]
fn factorization_matches_trial_division_and_reconstructs() {
    assert_eq!(factor(0), Err(NumberTheoryError::ZeroUndefined));
    assert!(factor(1).unwrap().is_empty());

    for n in 1..=5_000 {
        let actual = factor(n).unwrap();
        let expected = reference_factors(n);
        let actual_pairs = actual
            .factors()
            .iter()
            .map(|factor| (factor.prime, factor.exponent))
            .collect::<Vec<_>>();
        assert_eq!(actual_pairs, expected, "n={n}");
        assert!(
            actual
                .factors()
                .windows(2)
                .all(|pair| pair[0].prime < pair[1].prime)
        );
        assert!(
            actual
                .factors()
                .iter()
                .all(|factor| factor.exponent > 0 && is_prime(factor.prime))
        );
        assert_eq!(reconstruct(&actual), u128::from(n));
    }

    let cases = [
        360_u64,
        1_u64 << 63,
        1_000_000_007_u64 * 1_000_000_009,
        4_294_967_291_u64 * 4_294_967_279,
        u64::MAX,
    ];
    for n in cases {
        let actual = factor(n).unwrap();
        assert_eq!(reconstruct(&actual), u128::from(n), "n={n}");
        assert!(actual.factors().iter().all(|factor| is_prime(factor.prime)));
    }
    assert_eq!(
        factor(360)
            .unwrap()
            .factors()
            .iter()
            .map(|factor| (factor.prime, factor.exponent))
            .collect::<Vec<_>>(),
        vec![(2, 3), (3, 2), (5, 1)]
    );
}

#[test]
fn phi_and_lambda_match_small_definitions() {
    for n in 1..=128_u64 {
        let factorization = factor(n).unwrap();
        assert_eq!(factorization.n(), n);
        let phi = factorization.euler_phi();
        let brute_phi = (1..=n).filter(|value| gcd(*value, n) == 1).count() as u64;
        assert_eq!(phi, brute_phi, "phi({n})");

        let lambda = factorization.carmichael_lambda().unwrap();
        if n == 1 {
            assert_eq!(lambda, 1);
            continue;
        }
        let context = ModCtx::new(Modulus::new(n).unwrap());
        for a in 0..n {
            if gcd(a, n) == 1 {
                assert_eq!(context.pow(a, lambda), 1, "a={a}, n={n}, lambda={lambda}");
            }
        }
    }

    assert_eq!(factor(1).unwrap().euler_phi(), 1);
    assert_eq!(factor(2).unwrap().carmichael_lambda().unwrap(), 1);
    assert_eq!(factor(4).unwrap().carmichael_lambda().unwrap(), 2);
    assert_eq!(factor(8).unwrap().carmichael_lambda().unwrap(), 2);
    assert_eq!(factor(16).unwrap().carmichael_lambda().unwrap(), 4);
}

fn brute_order(a: u64, n: u64) -> Option<u64> {
    if gcd(a % n, n) != 1 {
        return None;
    }
    let mut value = 1_u64;
    for order in 1..=n {
        value = (u128::from(value) * u128::from(a % n) % u128::from(n)) as u64;
        if value == 1 {
            return Some(order);
        }
    }
    panic!("order not found for a={a}, n={n}");
}

#[test]
fn multiplicative_order_matches_brute_force() {
    for n in 2..=64_u64 {
        let lambda = factor(n).unwrap().carmichael_lambda().unwrap();
        for a in 0..n {
            let expected = brute_order(a, n);
            let actual = multiplicative_order(a, n).unwrap();
            match (expected, actual) {
                (None, MultiplicativeOrderResult::DoesNotExist) => {}
                (Some(expected), MultiplicativeOrderResult::Exists(actual)) => {
                    assert_eq!(actual, expected, "a={a}, n={n}");
                    assert_eq!(lambda % actual, 0);
                    let context = ModCtx::new(Modulus::new(n).unwrap());
                    assert_eq!(context.pow(a % n, actual), 1);
                }
                outcome => panic!("unexpected order outcome for a={a}, n={n}: {outcome:?}"),
            }
        }
    }

    assert_eq!(
        multiplicative_order(3, 7).unwrap(),
        MultiplicativeOrderResult::Exists(6)
    );
    assert_eq!(
        multiplicative_order(6, 9).unwrap(),
        MultiplicativeOrderResult::DoesNotExist
    );
    assert_eq!(
        multiplicative_order(1, 0),
        Err(NumberTheoryError::ZeroUndefined)
    );
}

#[test]
fn integer_analysis_factors_once_conceptually_and_classifies_values() {
    let unit = swissmath_core::analyze_integer(1).unwrap();
    assert_eq!(unit.classification, IntegerClassification::Unit);
    assert_eq!(unit.primality, PrimalityAssessment::Neither);
    assert_eq!(unit.phi, 1);
    assert_eq!(unit.lambda, 1);

    let prime = swissmath_core::analyze_integer(13).unwrap();
    assert_eq!(prime.classification, IntegerClassification::Prime);
    assert_eq!(prime.phi, 12);
    assert_eq!(prime.lambda, 12);

    let composite = swissmath_core::analyze_integer(360).unwrap();
    assert_eq!(composite.classification, IntegerClassification::Composite);
    assert_eq!(composite.phi, 96);
    assert_eq!(composite.lambda, 12);
    assert_eq!(reconstruct(&composite.factorization), 360);
}

#[test]
fn pollard_robustness_corpus_reconstructs_prime_bases() {
    let corpus = [
        1_000_000_007_u64 * 1_000_000_009,
        1_000_000_021_u64 * 1_000_000_033,
        1_000_000_087_u64 * 1_000_000_093,
        1_000_000_099_u64 * 1_000_000_103,
        1_000_000_123_u64 * 1_000_000_181,
        1_000_000_193_u64 * 1_000_000_207,
        1_000_000_217_u64 * 1_000_000_223,
        1_000_000_231_u64 * 1_000_000_237,
        4_294_967_291_u64 * 4_294_967_279,
        4_294_967_231_u64 * 4_294_967_197,
        4_294_967_189_u64 * 4_294_967_177,
        4_294_967_161_u64 * 4_294_967_149,
        3_037_000_493_u64 * 3_037_000_501,
        3_037_000_513_u64 * 3_037_000_519,
        2_147_483_647_u64 * 2_147_483_629,
        2_147_483_587_u64 * 2_147_483_579,
    ];
    for n in corpus {
        let actual = factor(n).expect("bounded Pollard corpus must factor");
        assert_eq!(reconstruct(&actual), u128::from(n), "n={n}");
        assert!(actual.factors().iter().all(|factor| is_prime(factor.prime)));
    }
}

#[test]
fn decimal_primality_routes_exact_and_bpsw_domains() {
    for n in [2_u64, 3, 4, u64::MAX - 1, u64::MAX] {
        let expected = if is_prime(n) {
            PrimalityAssessment::PrimeExact
        } else {
            PrimalityAssessment::Composite
        };
        assert_eq!(assess_primality_decimal(&n.to_string()).unwrap(), expected);
    }
    assert_eq!(
        assess_primality_decimal("0").unwrap(),
        PrimalityAssessment::Neither
    );
    assert_eq!(
        assess_primality_decimal("1").unwrap(),
        PrimalityAssessment::Neither
    );

    let boundary = (BigUint::from(u64::MAX) + 1_u32).to_str_radix(10);
    assert_eq!(
        assess_primality_decimal(&boundary).unwrap(),
        PrimalityAssessment::Composite
    );
    assert_eq!(
        assess_primality_decimal("-17"),
        Err(swissmath_core::PrimalityInputError::Negative)
    );
    assert_eq!(
        assess_primality_decimal("12x"),
        Err(swissmath_core::PrimalityInputError::InvalidDecimal)
    );

    let m127 = ((BigUint::from(1_u32) << 127_u32) - 1_u32).to_str_radix(10);
    let m521 = ((BigUint::from(1_u32) << 521_u32) - 1_u32).to_str_radix(10);
    let m1279 = ((BigUint::from(1_u32) << 1279_u32) - 1_u32).to_str_radix(10);
    assert_eq!(
        assess_primality_decimal(&m127).unwrap(),
        PrimalityAssessment::ExactProofIncomplete
    );
    assert_eq!(
        assess_primality_decimal(&m521).unwrap(),
        PrimalityAssessment::ProbablePrime
    );
    assert_eq!(
        assess_primality_decimal(&m1279).unwrap(),
        PrimalityAssessment::ProbablePrime
    );

    let m127_value = BigUint::parse_bytes(m127.as_bytes(), 10).unwrap();
    let large_square = (&m127_value * &m127_value).to_str_radix(10);
    let large_even = (&m521_value(&m521) << 1_u32).to_str_radix(10);
    assert_eq!(
        assess_primality_decimal(&large_square).unwrap(),
        PrimalityAssessment::Composite
    );
    assert_eq!(
        assess_primality_decimal(&large_even).unwrap(),
        PrimalityAssessment::Composite
    );

    match analyze_integer_decimal(&m521).unwrap() {
        DecimalIntegerAnalysis::Large { primality, .. } => {
            assert_eq!(primality, PrimalityAssessment::ProbablePrime)
        }
        DecimalIntegerAnalysis::Exact(_) | DecimalIntegerAnalysis::Neither { .. } => {
            panic!("large value must not enter a small analysis")
        }
        DecimalIntegerAnalysis::U128 { .. } => {
            panic!("521-bit value must route beyond u128")
        }
    }
    assert_eq!(
        analyze_integer_decimal("0").unwrap(),
        DecimalIntegerAnalysis::Neither { n: "0".to_owned() }
    );
}

fn m521_value(value: &str) -> BigUint {
    BigUint::parse_bytes(value.as_bytes(), 10).unwrap()
}

#[test]
fn exact_first_u128_routing_covers_boundaries() {
    for n in 0_u64..=100_000 {
        let expected = match n {
            0 | 1 => PrimalityAssessment::Neither,
            _ if is_prime(n) => PrimalityAssessment::PrimeExact,
            _ => PrimalityAssessment::Composite,
        };
        assert_eq!(
            assess_primality_decimal(&n.to_string()).unwrap(),
            expected,
            "n={n}"
        );
    }
    assert_eq!(
        assess_primality_u128(u64::MAX as u128),
        PrimalityAssessment::Composite
    );
    assert_eq!(
        assess_primality_u128(u64::MAX as u128 + 1),
        PrimalityAssessment::Composite
    );
    assert_eq!(
        assess_primality_u128(u128::MAX),
        PrimalityAssessment::Composite
    );

    let beyond_u128 = (BigUint::from(u128::MAX) + 1_u32).to_str_radix(10);
    assert_eq!(
        assess_primality_decimal(&beyond_u128).unwrap(),
        PrimalityAssessment::Composite
    );
}

#[test]
fn bounded_pocklington_proves_fixed_u128_primes() {
    let cases = [
        39_614_081_257_132_185_645_928_677_377_u128,
        39_614_081_257_132_186_599_411_417_089_u128,
        39_614_081_257_132_188_016_750_624_769_u128,
        39_614_081_257_132_215_281_203_019_777_u128,
        39_614_081_257_132_219_919_767_699_457_u128,
    ];
    for n in cases {
        assert_eq!(
            assess_primality_u128(n),
            PrimalityAssessment::PrimeExact,
            "n={n}"
        );
    }
}

#[test]
fn bounded_u128_path_rejects_composites_and_reports_incomplete_proofs() {
    let composites = [
        (u64::MAX as u128) + 2,
        4_294_967_291_u128 * 4_294_967_291_u128,
        9_223_372_036_854_779_731_u128 * 9_223_372_036_854_779_953_u128,
        u128::MAX,
    ];
    for n in composites {
        assert_eq!(
            assess_primality_u128(n),
            PrimalityAssessment::Composite,
            "n={n}"
        );
    }

    assert_eq!(
        assess_primality_u128(170_141_183_460_469_231_731_687_303_715_884_105_727_u128),
        PrimalityAssessment::ExactProofIncomplete
    );

    match analyze_integer_decimal("39614081257132185645928677377").unwrap() {
        DecimalIntegerAnalysis::U128 { primality, .. } => {
            assert_eq!(primality, PrimalityAssessment::PrimeExact)
        }
        other => panic!("unexpected u128 route: {other:?}"),
    }
    match analyze_integer_decimal("170141183460469231731687303715884105727").unwrap() {
        DecimalIntegerAnalysis::U128 { primality, .. } => {
            assert_eq!(primality, PrimalityAssessment::ExactProofIncomplete)
        }
        other => panic!("unexpected u128 route: {other:?}"),
    }
}

#[test]
fn pocklington_success_satisfies_direct_theorem_conditions() {
    let n = 39_614_081_257_132_185_645_928_677_377_u128;
    let modulus = BigUint::from(n);
    let exponent = BigUint::from(n - 1);
    assert!((n - 1) > n / (n - 1));
    for q_value in [2_u128, 9_223_372_036_854_779_731_u128] {
        let q_exponent = BigUint::from((n - 1) / q_value);
        let mut witness = None;
        for a in 2_u128..=64 {
            let base = BigUint::from(a);
            if base.modpow(&exponent, &modulus) == BigUint::from(1_u8) {
                let value = base.modpow(&q_exponent, &modulus);
                let value = value
                    .to_bytes_le()
                    .iter()
                    .enumerate()
                    .fold(0_u128, |acc, (index, byte)| {
                        acc | (u128::from(*byte) << (index * 8))
                    });
                let difference = if value == 0 { n - 1 } else { value - 1 };
                if gcd_wide(difference, n) == 1 {
                    witness = Some(a);
                    break;
                }
            }
        }
        assert!(witness.is_some(), "no witness found for q={q_value}");
    }
    assert_eq!(assess_primality_u128(n), PrimalityAssessment::PrimeExact);
}

fn gcd_wide(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}
