use swissmath_core::{
    Congruence, LinearCongruence, LinearSolution, ModCtx, ModularFilter, ModularFilterBuild,
    ModularSieve, Modulus, ResidueSet, SieveError, solve_linear_congruence, solve_linear_system,
};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

fn brute_linear(a: u64, b: u64, m: u64) -> Vec<u64> {
    (0..m)
        .filter(|x| (u128::from(a) * u128::from(*x)) % u128::from(m) == u128::from(b))
        .collect()
}

fn described_solution(solution: LinearSolution, original_modulus: u64) -> Vec<u64> {
    match solution {
        LinearSolution::None => Vec::new(),
        LinearSolution::All => (0..original_modulus).collect(),
        LinearSolution::Class(congruence) => (0..original_modulus)
            .filter(|x| *x % congruence.modulus().get() == congruence.residue())
            .collect(),
    }
}

#[test]
fn exhaustive_linear_congruences_through_modulus_sixty_four() {
    for m in 1..=64_u64 {
        for a in 0..m {
            for b in 0..m {
                let equation = LinearCongruence::new(a, b, modulus(m));
                let result = solve_linear_congruence(equation);
                let expected = brute_linear(a, b, m);
                assert_eq!(
                    described_solution(result.solution, m),
                    expected,
                    "{a}x = {b} mod {m}"
                );
                assert_eq!(result.solution_count(modulus(m)), expected.len() as u64);
            }
        }
    }
}

#[test]
fn linear_explanation_and_boundary_cases_are_structured() {
    let result = solve_linear_congruence(LinearCongruence::new(14, 8, modulus(30)));
    assert_eq!(result.normalized_a, 14);
    assert_eq!(result.normalized_b, 8);
    assert_eq!(result.gcd, 2);
    assert_eq!(result.reduced_a, Some(7));
    assert_eq!(result.reduced_b, Some(4));
    assert_eq!(result.reduced_modulus, 15);
    assert_eq!(result.inverse, Some(13));
    assert_eq!(
        result.solution,
        LinearSolution::Class(Congruence::new(7, modulus(15)))
    );
    assert_eq!(result.solution_count(modulus(30)), 2);

    assert_eq!(
        solve_linear_congruence(LinearCongruence::new(6, 5, modulus(15))).solution,
        LinearSolution::None
    );
    assert_eq!(
        solve_linear_congruence(LinearCongruence::new(0, 0, modulus(19))).solution,
        LinearSolution::All
    );
    assert_eq!(
        solve_linear_congruence(LinearCongruence::new(0, 1, modulus(19))).solution,
        LinearSolution::None
    );
    assert_eq!(
        solve_linear_congruence(LinearCongruence::new(
            u64::MAX - 1,
            u64::MAX - 2,
            modulus(u64::MAX)
        ))
        .solution_count(modulus(u64::MAX)),
        1
    );
    assert_eq!(
        solve_linear_congruence(LinearCongruence::new(u64::MAX, u64::MAX, modulus(u64::MAX)))
            .solution,
        LinearSolution::All
    );
}

#[test]
fn systems_reuse_linear_solver_and_generalized_crt() {
    let compatible = solve_linear_system([
        LinearCongruence::new(14, 8, modulus(30)),
        LinearCongruence::new(3, 6, modulus(15)),
    ])
    .unwrap();
    assert_eq!(
        compatible,
        LinearSolution::Class(Congruence::new(7, modulus(15)))
    );

    let incompatible = solve_linear_system([
        LinearCongruence::new(1, 1, modulus(2)),
        LinearCongruence::new(1, 0, modulus(4)),
    ])
    .unwrap();
    assert_eq!(incompatible, LinearSolution::None);

    assert_eq!(
        solve_linear_system([
            LinearCongruence::new(0, 0, modulus(12)),
            LinearCongruence::new(0, 0, modulus(5)),
        ])
        .unwrap(),
        LinearSolution::All
    );
    assert_eq!(
        solve_linear_system(Vec::<LinearCongruence>::new()).unwrap(),
        LinearSolution::All
    );
    assert_eq!(
        solve_linear_system([
            LinearCongruence::new(0, 1, modulus(12)),
            LinearCongruence::new(1, 0, modulus(5)),
        ])
        .unwrap(),
        LinearSolution::None
    );
}

#[test]
fn linear_filters_are_reduced_without_materializing_original_residues() {
    let build = ModularFilter::from_linear_congruence(LinearCongruence::new(14, 8, modulus(30)));
    let ModularFilterBuild::Filter(filter) = build else {
        panic!("expected a reduced filter");
    };
    assert_eq!(filter.modulus(), modulus(15));
    assert_eq!(filter.allowed().iter().collect::<Vec<_>>(), vec![7]);

    assert_eq!(
        ModularFilter::from_linear_congruence(LinearCongruence::new(0, 0, modulus(30))),
        ModularFilterBuild::All
    );
    assert_eq!(
        ModularFilter::from_linear_congruence(LinearCongruence::new(6, 5, modulus(15))),
        ModularFilterBuild::None
    );
}

fn brute_sieve(start: u64, end: u64, filters: &[ModularFilter]) -> Vec<u64> {
    (start..=end)
        .filter(|candidate| {
            filters.iter().all(|filter| {
                filter
                    .allowed()
                    .contains(*candidate % filter.modulus().get())
            })
        })
        .collect()
}

#[test]
fn sieve_matches_direct_reference_and_normalizes_filters() {
    let filters = vec![
        ModularFilter::from_allowed(modulus(5), [1, 4]).unwrap(),
        ModularFilter::from_allowed(modulus(7), [0, 1, 6]).unwrap(),
        ModularFilter::from_allowed(modulus(8), [1, 3, 5, 7]).unwrap(),
    ];
    let sieve = ModularSieve::new(filters.clone()).unwrap();
    let expected = brute_sieve(0, 10_000, &filters);
    let result = sieve.search(0, 10_000, 50).unwrap();
    assert_eq!(result.survivor_count, expected.len() as u128);
    assert_eq!(result.preview, expected[..50].to_vec());
    assert_eq!(result.total_values, 10_001);
    assert_eq!(result.normalized_filter_count, 3);
    assert_eq!(result.anchor_modulus, Some(modulus(5)));
    assert_eq!(result.anchor_allowed_count, 2);

    let shared = ModularSieve::new([
        ModularFilter::from_allowed(modulus(12), [1, 2, 3]).unwrap(),
        ModularFilter::from_allowed(modulus(12), [2, 3, 4]).unwrap(),
        ModularFilter::from_allowed(modulus(5), 0..5).unwrap(),
    ])
    .unwrap();
    let shared_result = shared.search(0, 100, 20).unwrap();
    assert_eq!(shared_result.normalized_filter_count, 1);
    assert_eq!(shared_result.survivor_count, 18);
    assert_eq!(
        shared_result.preview,
        vec![
            2, 3, 14, 15, 26, 27, 38, 39, 50, 51, 62, 63, 74, 75, 86, 87, 98, 99
        ]
    );
}

#[test]
fn sieve_handles_empty_full_linear_and_u64_boundary_ranges() {
    let impossible =
        ModularSieve::new([ModularFilter::from_allowed(modulus(7), []).unwrap()]).unwrap();
    let result = impossible.search(20, 30, 50).unwrap();
    assert_eq!(result.survivor_count, 0);
    assert!(result.preview.is_empty());

    let all = ModularSieve::new([ModularFilter::from_allowed(modulus(1), [0]).unwrap()]).unwrap();
    let result = all.search(u64::MAX - 2, u64::MAX, 5).unwrap();
    assert_eq!(result.total_values, 3);
    assert_eq!(result.survivor_count, 3);
    assert_eq!(result.preview, vec![u64::MAX - 2, u64::MAX - 1, u64::MAX]);

    let no_filters = ModularSieve::new(Vec::<ModularFilter>::new()).unwrap();
    let result = no_filters.search(u64::MAX - 1, u64::MAX, 10).unwrap();
    assert_eq!(result.total_values, 2);
    assert_eq!(result.survivor_count, 2);
    assert_eq!(result.preview, vec![u64::MAX - 1, u64::MAX]);
    let whole_u64_domain = no_filters.search(0, u64::MAX, 0).unwrap();
    assert_eq!(whole_u64_domain.total_values, 1_u128 << 64);
    assert_eq!(whole_u64_domain.survivor_count, 1_u128 << 64);
    assert_eq!(no_filters.search(10, 9, 1), Err(SieveError::InvalidRange));

    let full_congruence = ModularSieve::new(
        match ModularFilter::from_linear_congruence(LinearCongruence::new(0, 0, modulus(10))) {
            ModularFilterBuild::All => Vec::new(),
            _ => panic!("expected tautology"),
        },
    )
    .unwrap();
    assert_eq!(full_congruence.search(4, 4, 1).unwrap().survivor_count, 1);
}

#[test]
fn deterministic_sieve_sweep_matches_reference() {
    let mut state = 0x1234_5678_9abc_def0_u64;
    for modulus_value in 2..=17_u64 {
        for round in 0..20 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let mut residues = Vec::new();
            for residue in 0..modulus_value {
                if (state.rotate_left(residue as u32) & 3) != 0 {
                    residues.push(residue);
                }
            }
            let filter = ModularFilter::from_allowed(modulus(modulus_value), residues).unwrap();
            let start = round * 3;
            let end = start + 71;
            let expected = brute_sieve(start, end, std::slice::from_ref(&filter));
            let result = ModularSieve::new([filter])
                .unwrap()
                .search(start, end, 13)
                .unwrap();
            assert_eq!(result.survivor_count, expected.len() as u128);
            assert_eq!(
                result.preview,
                expected.into_iter().take(13).collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn residue_set_operations_used_by_filtering_remain_exact() {
    let a = ResidueSet::from_iter(modulus(9), [1, 3, 5, 7]).unwrap();
    let b = ResidueSet::from_iter(modulus(9), [3, 4, 7]).unwrap();
    assert_eq!(
        a.intersection(&b).unwrap().iter().collect::<Vec<_>>(),
        vec![3, 7]
    );
    assert!(a.complement().contains(0));
    let ctx = ModCtx::new(modulus(9));
    assert_eq!(ctx.mul(4, 7), 1);
}
