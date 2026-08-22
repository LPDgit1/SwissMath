use std::{hint::black_box, time::Instant};

use swissmath_core::{Factorization, factor};

fn checksum(factorization: &Factorization) -> u64 {
    factorization.factors().iter().fold(0_u64, |sum, factor| {
        sum.wrapping_add(factor.prime.wrapping_mul(u64::from(factor.exponent)))
    })
}

fn measure(name: &str, n: u64, repetitions: u64) {
    let warmup = factor(n).expect("benchmark input must factor");
    let expected = checksum(&warmup);
    let started = Instant::now();
    let mut observed = 0_u64;
    for _ in 0..repetitions {
        let result = factor(black_box(n)).expect("benchmark input must factor");
        observed = observed.wrapping_add(black_box(checksum(&result)));
    }
    let elapsed_ns = started.elapsed().as_nanos() as f64 / repetitions as f64;
    assert_eq!(observed, expected.wrapping_mul(repetitions));
    println!("{name:18} n={n:20} factor={elapsed_ns:12.2} ns/op  checksum={expected}");
}

fn main() {
    println!("SwissMath Prime & Factor v0.5 (lower is better; deterministic inputs)");
    measure("easy composite", 2 * 1_000_000_007, 20);
    measure("large prime", 18_446_744_073_709_551_557, 20);
    measure("prime power", 1_u64 << 63, 20);
    measure("balanced semiprime", 4_294_967_291_u64 * 4_294_967_279, 5);
    measure(
        "mixed composite",
        (1_u64 << 20) * 3_u64.pow(10) * 5_u64.pow(3) * 7,
        20,
    );
}
