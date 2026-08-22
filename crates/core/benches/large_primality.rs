use std::{hint::black_box, time::Instant};

use num_bigint::BigUint;
use swissmath_core::{PrimalityAssessment, assess_primality_decimal, assess_primality_u128};

fn mersenne(exponent: u32) -> BigUint {
    (BigUint::from(1_u32) << exponent) - 1_u32
}

fn measure(name: &str, value: &BigUint, expected: PrimalityAssessment, repetitions: u64) {
    let decimal = value.to_str_radix(10);
    let warmup = assess_primality_decimal(&decimal).expect("benchmark input is valid");
    assert_eq!(warmup, expected);
    let started = Instant::now();
    let mut observed = PrimalityAssessment::Composite;
    for _ in 0..repetitions {
        observed = black_box(assess_primality_decimal(black_box(&decimal)))
            .expect("benchmark input is valid");
    }
    let elapsed_ns = started.elapsed().as_nanos() as f64 / repetitions as f64;
    assert_eq!(observed, expected);
    println!(
        "{name:18} bits={:4} assessment={expected:?}  {elapsed_ns:12.2} ns/op",
        value.bits()
    );
}

fn measure_u128(name: &str, value: u128, expected: PrimalityAssessment, repetitions: u64) {
    let warmup = assess_primality_u128(value);
    assert_eq!(warmup, expected);
    let started = Instant::now();
    let mut observed = PrimalityAssessment::Composite;
    for _ in 0..repetitions {
        observed = black_box(assess_primality_u128(black_box(value)));
    }
    let elapsed_ns = started.elapsed().as_nanos() as f64 / repetitions as f64;
    assert_eq!(observed, expected);
    println!(
        "{name:24} bits={:4} assessment={expected:?}  {elapsed_ns:12.2} ns/op",
        128 - value.leading_zeros()
    );
}

fn main() {
    println!("SwissMath large primality v0.5 (exact-first u128; lower is better)");
    measure_u128(
        "u128 composite",
        u64::MAX as u128 + 3,
        PrimalityAssessment::Composite,
        8,
    );
    measure_u128(
        "u128 proof-friendly prime",
        39_614_081_257_132_185_645_928_677_377_u128,
        PrimalityAssessment::PrimeExact,
        4,
    );
    measure_u128(
        "u128 incomplete proof",
        170_141_183_460_469_231_731_687_303_715_884_105_727_u128,
        PrimalityAssessment::ExactProofIncomplete,
        2,
    );

    println!("\n>u128 BPSW reference cases");
    for (exponent, repetitions) in [(521_u32, 4), (1_279, 2), (2_203, 1)] {
        let prime = mersenne(exponent);
        let composite = &prime * 3_u32;
        measure(
            &format!("probable prime {exponent}"),
            &prime,
            PrimalityAssessment::ProbablePrime,
            repetitions,
        );
        measure(
            &format!("composite {exponent}"),
            &composite,
            PrimalityAssessment::Composite,
            repetitions,
        );
    }
}
