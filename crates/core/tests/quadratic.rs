use swissmath_core::{
    PrimeRoots, QuadraticError, gcd, is_prime, jacobi_symbol, legendre_symbol,
    modular_square_roots, prime_square_roots,
};

fn direct_legendre(a: i128, p: u64) -> i8 {
    let reduced = a.rem_euclid(i128::from(p)) as u64;
    if reduced == 0 {
        return 0;
    }
    if (0..p).any(|x| ((u128::from(x) * u128::from(x)) % u128::from(p)) as u64 == reduced) {
        1
    } else {
        -1
    }
}

fn reference_jacobi(a: i128, n: u64) -> i8 {
    if n == 1 {
        return 1;
    }
    let mut remainder = n;
    let mut result = 1_i8;
    let mut prime = 3_u64;
    while prime <= remainder / prime {
        if remainder % prime == 0 {
            let symbol = direct_legendre(a, prime);
            if symbol == 0 {
                return 0;
            }
            let mut exponent = 0;
            while remainder % prime == 0 {
                remainder /= prime;
                exponent += 1;
            }
            if exponent % 2 == 1 {
                result *= symbol;
            }
        }
        prime += 2;
    }
    if remainder > 1 {
        result *= direct_legendre(a, remainder);
    }
    result
}

fn brute_roots(a: i128, n: u64) -> Vec<u64> {
    let reduced = a.rem_euclid(i128::from(n)) as u64;
    (0..n)
        .filter(|x| ((u128::from(*x) * u128::from(*x)) % u128::from(n)) as u64 == reduced)
        .collect()
}

#[test]
fn jacobi_matches_independent_small_factor_reference() {
    for n in (1..=255_u64).filter(|n| n % 2 == 1) {
        for a in -300_i128..=300 {
            assert_eq!(
                jacobi_symbol(a, n).unwrap(),
                reference_jacobi(a, n),
                "a={a}, n={n}"
            );
        }
    }
    assert_eq!(jacobi_symbol(-1, 3).unwrap(), -1);
    assert_eq!(jacobi_symbol(0, 9).unwrap(), 0);
    assert_eq!(jacobi_symbol(5, 1).unwrap(), 1);
    assert_eq!(
        jacobi_symbol(1, 0),
        Err(QuadraticError::JacobiRequiresOddModulus)
    );
    assert_eq!(
        jacobi_symbol(1, 2),
        Err(QuadraticError::JacobiRequiresOddModulus)
    );
}

#[test]
fn legendre_and_prime_roots_match_exhaustive_prime_domains() {
    for p in (2..=257_u64).filter(|p| is_prime(*p)) {
        if p > 2 {
            for a in -((p as i128) * 2)..=((p as i128) * 2) {
                assert_eq!(
                    legendre_symbol(a, p).unwrap(),
                    direct_legendre(a, p),
                    "a={a}, p={p}"
                );
            }
        }
        for a in 0..p {
            let expected = brute_roots(a as i128, p);
            let actual = prime_square_roots(a as i128, p).unwrap();
            let actual_vec = match actual {
                PrimeRoots::None => Vec::new(),
                PrimeRoots::One(root) => vec![root],
                PrimeRoots::Two(left, right) => vec![left, right],
            };
            assert_eq!(actual_vec, expected, "a={a}, p={p}");
        }
    }
    assert_eq!(legendre_symbol(5, 11).unwrap(), 1);
    assert_eq!(prime_square_roots(10, 13).unwrap(), PrimeRoots::Two(6, 7));
}

#[test]
fn composite_unit_roots_match_brute_force() {
    for n in 2..=128_u64 {
        for a in 0..n {
            let unit = gcd(a, n) == 1;
            let actual = modular_square_roots(a as i128, n);
            if !unit && !is_prime(n) {
                assert_eq!(
                    actual,
                    Err(QuadraticError::NonCoprimeUnsupported),
                    "a={a}, n={n}"
                );
                continue;
            }
            let expected = brute_roots(a as i128, n);
            assert_eq!(actual.unwrap(), expected, "a={a}, n={n}");
        }
    }
}

#[test]
fn targeted_prime_powers_two_powers_and_large_square() {
    assert_eq!(
        modular_square_roots(36, 13_u64.pow(2)).unwrap(),
        vec![6, 163]
    );
    assert_eq!(modular_square_roots(1, 32).unwrap(), vec![1, 15, 17, 31]);

    let modulus = 45;
    let roots = modular_square_roots(4, modulus).unwrap();
    assert_eq!(roots, brute_roots(4, modulus));

    let prime = 18_446_744_073_709_551_557_u64;
    let witness = 123_456_789_u64;
    let square = (u128::from(witness) * u128::from(witness) % u128::from(prime)) as i128;
    let roots = prime_square_roots(square, prime).unwrap();
    let roots = match roots {
        PrimeRoots::None => Vec::new(),
        PrimeRoots::One(root) => vec![root],
        PrimeRoots::Two(left, right) => vec![left, right],
    };
    assert!(roots.iter().all(|root| {
        (u128::from(*root) * u128::from(*root) % u128::from(prime)) as i128 == square
    }));
    assert!(roots.contains(&witness) || roots.contains(&(prime - witness)));
}
