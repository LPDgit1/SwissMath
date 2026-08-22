use std::{hint::black_box, time::Instant};

use swissmath_core::modular_square_roots;

fn measure(name: &str, a: i128, modulus: u64, repetitions: u64) {
    let warmup = modular_square_roots(a, modulus).expect("benchmark input must solve");
    let expected = checksum(&warmup);
    let started = Instant::now();
    let mut observed = 0_u64;
    for _ in 0..repetitions {
        let roots = modular_square_roots(black_box(a), black_box(modulus))
            .expect("benchmark input must solve");
        observed = observed.wrapping_add(black_box(checksum(&roots)));
    }
    let elapsed_ns = started.elapsed().as_nanos() as f64 / repetitions as f64;
    assert_eq!(observed, expected.wrapping_mul(repetitions));
    println!(
        "{name:22} n={modulus:20} roots={:3}  {elapsed_ns:12.2} ns/op",
        warmup.len()
    );
}

fn checksum(values: &[u64]) -> u64 {
    values
        .iter()
        .fold(0_u64, |sum, value| sum.wrapping_add(*value))
}

fn main() {
    println!("SwissMath quadratic arithmetic v0.5 (lower is better; deterministic inputs)");
    measure(
        "prime p % 4 = 3",
        (u128::from(123_456_789_u64).pow(2) % u128::from(18_446_744_073_709_551_427_u64)) as i128,
        18_446_744_073_709_551_427,
        4,
    );
    measure(
        "Tonelli-Shanks p % 4 = 1",
        (u128::from(123_456_789_u64).pow(2) % u128::from(18_446_744_073_709_551_557_u64)) as i128,
        18_446_744_073_709_551_557,
        4,
    );
    measure("odd prime power", 36, 13_u64.pow(8), 20);
    measure("power of two", 1, 1_u64 << 40, 20);
    measure("composite unit modulus", 4, 3_u64.pow(8) * 5_u64.pow(6), 20);
}
