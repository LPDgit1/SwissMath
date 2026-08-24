use num_bigint::{BigInt, BigUint};
use swissmath_core::{
    Congruence, Modulus, MultimodularAccumulator, MultimodularError, PrimeField,
    centered_representative, crt_fold, rational_reconstruct, rational_reconstruct_big,
    reconstruct_integer_bounded,
};

const SMALL_PRIMES: [u64; 4] = [101, 103, 107, 109];

fn mod_u64(value: &BigUint, modulus: u64) -> u64 {
    (value % modulus).iter_u64_digits().next().unwrap_or(0)
}

fn signed_residue(value: i128, field: PrimeField) -> u64 {
    field.normalize(value)
}

fn fraction_residue(numerator: i128, denominator: u64, field: PrimeField) -> u64 {
    field.mul(
        field.normalize(numerator),
        field.inverse(denominator).unwrap(),
    )
}

#[test]
fn incremental_crt_covers_scalar_vector_matrix_and_input_errors() {
    let mut scalar = MultimodularAccumulator::new();
    scalar
        .push_prime_residues(PrimeField::new(101).unwrap(), &[42])
        .unwrap();
    assert_eq!(scalar.values(), &[BigUint::from(42_u8)]);
    assert_eq!(scalar.coordinate_count(), 1);
    assert_eq!(scalar.prime_count(), 1);

    let blocks = [
        (101_u64, vec![1, 2, 3, 4]),
        (103, vec![12, 27, 99, 5]),
        (107, vec![42, 11, 18, 90]),
    ];
    let mut matrix = MultimodularAccumulator::new();
    for (prime, residues) in &blocks {
        matrix
            .push_prime_residues(PrimeField::new(*prime).unwrap(), residues)
            .unwrap();
    }
    assert_eq!(matrix.coordinate_count(), 4);
    assert_eq!(matrix.prime_count(), 3);
    for index in 0..4 {
        let expected = crt_fold(blocks.iter().map(|(prime, residues)| {
            Congruence::new(
                residues[index],
                Modulus::new(*prime).expect("prime is nonzero"),
            )
        }))
        .unwrap()
        .unwrap();
        assert_eq!(matrix.values()[index], BigUint::from(expected.residue()));
    }

    assert_eq!(
        matrix.push_prime_residues(PrimeField::new(101).unwrap(), &[0; 4]),
        Err(MultimodularError::DuplicatePrimeModulus { prime: 101 })
    );
    assert_eq!(
        matrix.push_prime_residues(PrimeField::new(113).unwrap(), &[1, 2]),
        Err(MultimodularError::CoordinateCountMismatch {
            expected: 4,
            actual: 2,
        })
    );
    assert_eq!(
        MultimodularAccumulator::new().push_prime_residues(PrimeField::new(101).unwrap(), &[]),
        Err(MultimodularError::EmptyResidueBlock)
    );
}

#[test]
fn combined_modulus_grows_beyond_u128_and_every_congruence_is_preserved() {
    let primes = [
        998_244_353,
        1_004_535_809,
        469_762_049,
        167_772_161,
        754_974_721,
    ];
    let coordinates = [0_u64, 1, 17, 123_456_789];
    let mut accumulator = MultimodularAccumulator::new();
    let mut source_blocks = Vec::new();
    for prime in primes {
        let field = PrimeField::new(prime).unwrap();
        let residues = coordinates
            .iter()
            .map(|value| value % prime)
            .collect::<Vec<_>>();
        accumulator.push_prime_residues(field, &residues).unwrap();
        source_blocks.push((prime, residues));
    }
    assert!(accumulator.combined_modulus() > &BigUint::from(u128::MAX));
    for (prime, residues) in source_blocks {
        for (value, expected) in accumulator.values().iter().zip(residues) {
            assert_eq!(mod_u64(value, prime), expected);
        }
    }
}

#[test]
fn centered_and_bounded_integer_reconstruction_are_verified_and_unique() {
    let values = [-1_999_999_i128, -42, 0, 1_234_567];
    let bound = BigUint::from(2_000_000_u64);
    let mut accumulator = MultimodularAccumulator::new();
    for prime in SMALL_PRIMES {
        let field = PrimeField::new(prime).unwrap();
        let residues = values
            .iter()
            .map(|value| signed_residue(*value, field))
            .collect::<Vec<_>>();
        accumulator.push_prime_residues(field, &residues).unwrap();
    }
    let reconstructed = accumulator.reconstruct_integers_bounded(&bound).unwrap();
    assert_eq!(
        reconstructed,
        values.into_iter().map(BigInt::from).collect::<Vec<_>>()
    );

    let residue = BigUint::from(100_u8);
    assert_eq!(
        centered_representative(&residue, &BigUint::from(101_u8)).unwrap(),
        BigInt::from(-1)
    );
    assert_eq!(
        reconstruct_integer_bounded(
            &BigUint::from(50_u8),
            &BigUint::from(101_u8),
            &BigUint::from(50_u8),
        ),
        Ok(BigInt::from(50))
    );
    assert_eq!(
        reconstruct_integer_bounded(
            &BigUint::from(1_u8),
            &BigUint::from(100_u8),
            &BigUint::from(50_u8),
        ),
        Err(MultimodularError::InsufficientModulus)
    );
}

#[test]
fn known_rational_matrix_is_recovered_exactly_under_global_bounds() {
    let fractions = [(2_i128, 3_u64), (-5, 7), (0, 1), (97, 89)];
    let mut accumulator = MultimodularAccumulator::new();
    for prime in [101_u64, 103, 107] {
        let field = PrimeField::new(prime).unwrap();
        let residues = fractions
            .iter()
            .map(|(numerator, denominator)| fraction_residue(*numerator, *denominator, field))
            .collect::<Vec<_>>();
        accumulator.push_prime_residues(field, &residues).unwrap();
    }
    let reconstructed = accumulator
        .reconstruct_rationals_bounded(&BigUint::from(100_u8), &BigUint::from(100_u8))
        .unwrap();
    for (actual, (numerator, denominator)) in reconstructed.iter().zip(fractions) {
        assert_eq!(actual.numerator, BigInt::from(numerator));
        assert_eq!(actual.denominator, BigUint::from(denominator));
    }

    assert_eq!(
        swissmath_core::rational_reconstruct_big_bounded(
            &BigUint::from(0_u8),
            &BigUint::from(101_u8),
            &BigUint::from(5_u8),
            &BigUint::from(11_u8),
        ),
        Err(MultimodularError::InsufficientModulus)
    );
}

#[test]
fn automatic_big_rational_reconstruction_matches_the_existing_scalar_oracle() {
    for modulus in [101_u64, 257, 1_009, 10_009] {
        for residue in (0..modulus).step_by((modulus / 97).max(1) as usize) {
            let small = rational_reconstruct(residue, modulus).unwrap();
            let big =
                rational_reconstruct_big(&BigUint::from(residue), &BigUint::from(modulus)).unwrap();
            assert_eq!(small.is_some(), big.is_some());
            if let (Some(small), Some(big)) = (small, big) {
                assert_eq!(BigInt::from(small.numerator), big.numerator);
                assert_eq!(BigUint::from(small.denominator), big.denominator);
            }
        }
    }
}

#[test]
fn ten_thousand_coordinates_are_accumulated_without_retaining_source_blocks() {
    let coordinate_count = 10_000_usize;
    let mut accumulator = MultimodularAccumulator::new();
    for prime in SMALL_PRIMES {
        let residues = (0..coordinate_count)
            .map(|index| (index as u64 * 17 + 3) % prime)
            .collect::<Vec<_>>();
        accumulator
            .push_prime_residues(PrimeField::new(prime).unwrap(), &residues)
            .unwrap();
    }
    assert_eq!(accumulator.coordinate_count(), coordinate_count);
    assert_eq!(accumulator.prime_count(), SMALL_PRIMES.len());
    for (index, value) in accumulator.values().iter().enumerate() {
        assert_eq!(value, &BigUint::from(index as u64 * 17 + 3));
    }
}
