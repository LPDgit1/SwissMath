use swissmath_core::{ModCtx, Modulus, ResidueSet, required_heap_bytes};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

#[test]
fn storage_and_word_boundaries_preserve_tail_len_and_iteration() {
    let boundaries = [
        1, 2, 63, 64, 65, 127, 128, 129, 191, 192, 193, 255, 256, 257,
    ];
    for m in boundaries {
        let modulus = modulus(m);
        let mut set = ResidueSet::from_predicate(modulus, |r| r % 3 == 1).unwrap();
        let expected: Vec<_> = (0..m).filter(|r| r % 3 == 1).collect();
        assert_eq!(set.iter().collect::<Vec<_>>(), expected);
        assert_eq!(set.len(), expected.len() as u64);

        set.complement_assign();
        let complement: Vec<_> = (0..m).filter(|r| r % 3 != 1).collect();
        assert_eq!(set.iter().collect::<Vec<_>>(), complement);
        assert!(set.iter().all(|r| r < m));
        assert_eq!(set.len(), complement.len() as u64);

        let full = ResidueSet::try_full(modulus).unwrap();
        assert_eq!(full.len(), m);
        assert_eq!(full.iter().last(), Some(m - 1));
        assert!(full.is_full());
    }

    // The selected threshold is private; its externally visible memory estimate
    // confirms only the expected inline-to-heap behavior.
    assert_eq!(required_heap_bytes(modulus(128)), 0);
    assert_eq!(required_heap_bytes(modulus(129)), 0);
    assert_eq!(required_heap_bytes(modulus(256)), 0);
    assert_eq!(required_heap_bytes(modulus(257)), 40);
}

#[test]
fn deterministic_property_sweep_checks_boolean_algebra() {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for &m in &[1, 2, 63, 64, 65, 127, 128, 129, 257, 1_000] {
        for _ in 0..64 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let seed_a = state;
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let seed_b = state;

            let ctx = ModCtx::new(modulus(m));
            let a = ResidueSet::from_predicate(modulus(m), |r| {
                ctx.mul(r, r).wrapping_add(seed_a % m) % m < m / 2
            })
            .unwrap();
            let b = ResidueSet::from_predicate(modulus(m), |r| {
                (r.wrapping_mul(0x9e37_79b9) ^ seed_b) % m < m / 3
            })
            .unwrap();

            assert_eq!(a.intersection(&b).unwrap(), b.intersection(&a).unwrap());
            assert_eq!(a.union(&b).unwrap(), b.union(&a).unwrap());
            assert_eq!(a.intersection(&a).unwrap(), a);
            assert_eq!(a.union(&a).unwrap(), a);
            assert_eq!(a.intersection(&a.union(&b).unwrap()).unwrap(), a);
            assert_eq!(a.complement().complement(), a);
            assert_eq!(
                a.difference(&b).unwrap(),
                a.intersection(&b.complement()).unwrap()
            );
            assert_eq!(
                a.union(&b).unwrap().complement(),
                a.complement().intersection(&b.complement()).unwrap()
            );
            assert_eq!(
                a.intersection(&b).unwrap().complement(),
                a.complement().union(&b.complement()).unwrap()
            );
        }
    }
}
