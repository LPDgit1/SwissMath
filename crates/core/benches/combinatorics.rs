use std::{hint::black_box, time::Instant};

use swissmath_core::{
    PrimeField, binomial_mod_prime, binomial_valuation, factorial_mod_prime, factorial_valuation,
};

fn measure<T>(name: &str, repetitions: u64, mut operation: impl FnMut() -> T) {
    black_box(operation());
    let started = Instant::now();
    for _ in 0..repetitions {
        black_box(operation());
    }
    let milliseconds = started.elapsed().as_secs_f64() * 1_000.0 / repetitions as f64;
    println!("{name:44} {milliseconds:12.6} ms/op");
}

fn main() {
    println!("SwissMath Core v0.9 modular-combinatorics benchmarks (lower is better)");
    let binary = PrimeField::new(2).unwrap();
    measure("factorial valuation n=10^18, p=2", 100_000, || {
        factorial_valuation(black_box(1_000_000_000_000_000_000), binary)
    });
    measure("binomial valuation n=10^18, p=2", 100_000, || {
        binomial_valuation(
            black_box(1_000_000_000_000_000_000),
            black_box(1_000_000_000),
            binary,
        )
        .unwrap()
    });

    let large = PrimeField::new(1_000_000_007).unwrap();
    let huge_n = 1_000_000_007_u64.pow(2) + 5_000;
    measure("Lucas huge n, 1000 digit-product steps", 100, || {
        binomial_mod_prime(black_box(huge_n), black_box(1_000), large).unwrap()
    });
    measure("factorial direct, 49999 product steps", 20, || {
        factorial_mod_prime(black_box(50_000), large).unwrap()
    });
    measure("factorial Wilson, 50000 product steps", 20, || {
        factorial_mod_prime(black_box(large.modulus() - 1 - 50_000), large).unwrap()
    });
    measure("binomial computation-limit early exit", 100_000, || {
        binomial_mod_prime(black_box(600_000_003), black_box(300_000_001), large)
    });

    println!("production product-loop calibration (one measured call per row)");
    for steps in [
        1_000_000_u64,
        5_000_000,
        10_000_000,
        25_000_000,
        50_000_000,
        100_000_000,
    ] {
        measure(&format!("factorial direct, {steps} steps"), 1, || {
            factorial_mod_prime(black_box(steps + 1), large).unwrap()
        });
        measure(&format!("binomial digit, {steps} steps"), 1, || {
            binomial_mod_prime(black_box(steps * 2), black_box(steps), large).unwrap()
        });
    }
}
