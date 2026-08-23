use swissmath_core::{
    DiscreteLogResult, MultiplicativeGroupError, PrimeField, discrete_log, is_prime,
    is_primitive_root, primitive_root,
};

fn brute_log(g: u64, h: u64, field: PrimeField) -> Option<u64> {
    let mut value = 1;
    for exponent in 0..field.modulus() {
        if value == h {
            return Some(exponent);
        }
        value = field.mul(value, g);
    }
    None
}

#[test]
fn primitive_roots_are_smallest_generators_for_small_primes() {
    for prime in (2..200).filter(|&value| is_prime(value)) {
        let field = PrimeField::new(prime).unwrap();
        let generator = primitive_root(field).unwrap();
        assert!(is_primitive_root(i128::from(generator), field).unwrap());
        assert_eq!(field.pow(generator, prime - 1), 1);
        assert!(
            (1..generator)
                .all(|candidate| { !is_primitive_root(i128::from(candidate), field).unwrap() })
        );
    }
}

#[test]
fn discrete_log_matches_brute_force_over_small_prime_fields() {
    for prime in (2..80).filter(|&value| is_prime(value)) {
        let field = PrimeField::new(prime).unwrap();
        for g in 1..prime {
            for h in 1..prime {
                let expected = brute_log(g, h, field);
                match (
                    discrete_log(i128::from(g), i128::from(h), field).unwrap(),
                    expected,
                ) {
                    (DiscreteLogResult::Solved { x, order }, Some(reference)) => {
                        assert_eq!(field.pow(g, x), h);
                        assert_eq!(x, reference % order);
                    }
                    (DiscreteLogResult::NoSolution { .. }, None) => {}
                    (actual, expected) => {
                        panic!("p={prime}, g={g}, h={h}: {actual:?} vs {expected:?}")
                    }
                }
            }
        }
    }
}

#[test]
fn discrete_log_covers_subgroups_prime_powers_crt_and_boundaries() {
    let field = PrimeField::new(97).unwrap();
    let generator = primitive_root(field).unwrap();
    for x in [0, 1, 17, 95] {
        let h = field.pow(generator, x);
        assert_eq!(
            discrete_log(i128::from(generator), i128::from(h), field).unwrap(),
            DiscreteLogResult::Solved { x, order: 96 }
        );
    }

    let subgroup_base = field.pow(generator, 4);
    assert_eq!(
        discrete_log(i128::from(subgroup_base), i128::from(generator), field).unwrap(),
        DiscreteLogResult::NoSolution { order: 24 }
    );

    let f2 = PrimeField::new(2).unwrap();
    assert_eq!(
        discrete_log(1, 1, f2).unwrap(),
        DiscreteLogResult::Solved { x: 0, order: 1 }
    );
    assert_eq!(
        discrete_log(0, 1, field),
        Err(MultiplicativeGroupError::ZeroElement)
    );
    assert_eq!(
        discrete_log(1, 0, field),
        Err(MultiplicativeGroupError::ZeroElement)
    );
}
