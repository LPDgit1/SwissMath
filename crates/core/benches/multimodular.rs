use std::{hint::black_box, time::Instant};

use num_bigint::BigUint;
use swissmath_core::{MultimodularAccumulator, PrimeField};

const PRIMES: [u64; 32] = [
    1009, 1013, 1019, 1021, 1031, 1033, 1039, 1049, 1051, 1061, 1063, 1069, 1087, 1091, 1093, 1097,
    1103, 1109, 1117, 1123, 1129, 1151, 1153, 1163, 1171, 1181, 1187, 1193, 1201, 1213, 1217, 1223,
];

fn integer_block(field: PrimeField, coordinates: usize) -> Vec<u64> {
    (0..coordinates)
        .map(|index| field.normalize(index as i128 % 997 - 498))
        .collect()
}

fn rational_block(field: PrimeField, coordinates: usize) -> Vec<u64> {
    (0..coordinates)
        .map(|index| {
            let numerator = index as i128 % 17 - 8;
            let denominator = index as u64 % 7 + 1;
            field.mul(
                field.normalize(numerator),
                field.inverse(denominator).unwrap(),
            )
        })
        .collect()
}

fn accumulate(coordinates: usize, prime_count: usize, rational: bool) -> MultimodularAccumulator {
    let mut accumulator = MultimodularAccumulator::new();
    for &prime in &PRIMES[..prime_count] {
        let field = PrimeField::new(prime).unwrap();
        let block = if rational {
            rational_block(field, coordinates)
        } else {
            integer_block(field, coordinates)
        };
        accumulator.push_prime_residues(field, &block).unwrap();
    }
    accumulator
}

fn report(label: &str, coordinates: usize, primes: usize, bits: u64, started: Instant) {
    let elapsed = started.elapsed();
    let rate = coordinates as f64 / elapsed.as_secs_f64();
    println!(
        "{label:18} coords={coordinates:6} primes={primes:2} bits={bits:4} elapsed_ms={:10.3} coords_per_s={rate:12.0}",
        elapsed.as_secs_f64() * 1_000.0
    );
}

fn main() {
    println!("SwissMath Core v0.10 multimodular benchmarks");
    for coordinates in [1_usize, 100, 1_000, 10_000, 100_000] {
        for primes in [2_usize, 4, 8, 16, 32] {
            let started = Instant::now();
            let accumulator = black_box(accumulate(coordinates, primes, false));
            report(
                "incremental CRT",
                coordinates,
                primes,
                accumulator.combined_modulus_bits(),
                started,
            );
        }
    }

    for coordinates in [1_usize, 1_000, 10_000, 100_000] {
        let accumulator = accumulate(coordinates, 8, false);
        let started = Instant::now();
        black_box(
            accumulator
                .reconstruct_integers_bounded(&BigUint::from(500_u64))
                .unwrap(),
        );
        report(
            "integer bounded",
            coordinates,
            8,
            accumulator.combined_modulus_bits(),
            started,
        );
    }

    for coordinates in [1_usize, 100, 1_000, 10_000] {
        for primes in [4_usize, 8, 16] {
            let accumulator = accumulate(coordinates, primes, true);
            let started = Instant::now();
            black_box(
                accumulator
                    .reconstruct_rationals_bounded(&BigUint::from(8_u64), &BigUint::from(7_u64))
                    .unwrap(),
            );
            report(
                "rational bounded",
                coordinates,
                primes,
                accumulator.combined_modulus_bits(),
                started,
            );
        }
    }

    let started = Instant::now();
    let matrix = black_box(accumulate(100 * 100, 8, false));
    report(
        "matrix 100x100",
        matrix.coordinate_count(),
        matrix.prime_count(),
        matrix.combined_modulus_bits(),
        started,
    );
}
