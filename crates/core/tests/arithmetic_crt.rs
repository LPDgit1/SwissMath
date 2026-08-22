use swissmath_core::{
    ArithmeticError, Congruence, ModCtx, Modulus, crt_compatible, crt_fold, crt_pair, gcd, inv_mod,
    reduce_i128,
};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

#[test]
fn modulus_and_signed_reduction_are_exact() {
    assert!(Modulus::new(0).is_none());
    assert_eq!(reduce_i128(-1, modulus(7)), 6);
    assert_eq!(reduce_i128(-15, modulus(7)), 6);
    assert_eq!(
        reduce_i128(i128::MIN, modulus(u64::MAX)),
        9_223_372_036_854_775_807
    );
    assert!(modulus(7).divides(modulus(35)));
    assert!(!modulus(7).divides(modulus(36)));
}

#[test]
fn arithmetic_boundaries_match_wide_reference() {
    let values = [
        1,
        2,
        u64::from(u32::MAX),
        u64::from(u32::MAX) + 1,
        u64::from(u32::MAX) + 2,
        1_u64 << 63,
        u64::MAX,
    ];
    for m in values {
        let ctx = ModCtx::new(modulus(m));
        let residues = [0, 1 % m, 2 % m, m / 2, m - 1];
        for &a in &residues {
            for &b in &residues {
                assert_eq!(
                    ctx.add(a, b),
                    ((u128::from(a) + u128::from(b)) % u128::from(m)) as u64
                );
                let expected_sub = (u128::from(a) + u128::from(m) - u128::from(b)) % u128::from(m);
                assert_eq!(ctx.sub(a, b), expected_sub as u64);
                assert_eq!(
                    ctx.mul(a, b),
                    (u128::from(a) * u128::from(b) % u128::from(m)) as u64
                );
            }
        }
        assert_eq!(ctx.pow(0, 0), u64::from(m != 1));
        assert_eq!(ctx.pow(m - 1, 2), u64::from(m != 1));
    }
}

#[test]
fn inverses_are_valid_across_boundaries() {
    let moduli = [1, 2, 3, 65, (1_u64 << 32) + 1, (1_u64 << 63) + 1, u64::MAX];
    for m in moduli {
        for a in [0, 1, 2, m / 2, m - 1] {
            let inverse = inv_mod(a, modulus(m));
            assert_eq!(inverse.is_some(), m == 1 || gcd(a % m, m) == 1);
            if let Some(value) = inverse {
                assert!(value < m);
                if m != 1 {
                    assert_eq!(u128::from(a % m) * u128::from(value) % u128::from(m), 1);
                }
            }
        }
    }
}

#[test]
fn exhaustive_crt_through_modulus_sixty_four() {
    for m in 1..=64_u64 {
        for n in 1..=64_u64 {
            let common = gcd(m, n);
            let lcm = m / common * n;
            let mut first_solution = vec![None; (m * n) as usize];
            for x in 0..lcm {
                let index = ((x % m) * n + x % n) as usize;
                first_solution[index].get_or_insert(x);
            }
            for a in 0..m {
                for b in 0..n {
                    let expected = first_solution[(a * n + b) as usize];
                    let left = Congruence::new(a, modulus(m));
                    let right = Congruence::new(b, modulus(n));
                    assert_eq!(crt_compatible(left, right), expected.is_some());
                    let actual = crt_pair(left, right).unwrap();
                    match (actual, expected) {
                        (None, None) => {}
                        (Some(combined), Some(residue)) => {
                            assert_eq!(combined.modulus().get(), lcm);
                            assert_eq!(combined.residue(), residue);
                        }
                        pair => panic!("CRT mismatch for {a} mod {m}, {b} mod {n}: {pair:?}"),
                    }
                }
            }
        }
    }
}

#[test]
fn crt_identity_divisibility_incompatibility_and_overflow() {
    let identity = crt_fold([]).unwrap().unwrap();
    assert_eq!(identity, Congruence::new(0, modulus(1)));

    let small = Congruence::new(3, modulus(4));
    let large = Congruence::new(11, modulus(12));
    assert_eq!(crt_pair(small, large).unwrap(), Some(large));
    assert_eq!(
        crt_pair(
            Congruence::new(1, modulus(2)),
            Congruence::new(0, modulus(4))
        )
        .unwrap(),
        None
    );
    assert_eq!(crt_fold([small, large]).unwrap(), Some(large));

    let overflow = crt_pair(
        Congruence::new(0, modulus(u64::MAX)),
        Congruence::new(0, modulus(u64::MAX - 1)),
    );
    assert_eq!(overflow, Err(ArithmeticError::Overflow));
}
