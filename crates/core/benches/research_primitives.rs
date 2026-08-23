use std::{hint::black_box, time::Instant};

use swissmath_core::{extended_gcd, factor, next_prime, previous_prime, rational_reconstruct};

fn measure<T>(name: &str, repetitions: u64, mut operation: impl FnMut() -> T)
where
    T: std::fmt::Debug,
{
    let warmup = operation();
    let started = Instant::now();
    for _ in 0..repetitions {
        black_box(operation());
    }
    let ns_per_operation = started.elapsed().as_nanos() as f64 / repetitions as f64;
    println!("{name:24} {ns_per_operation:12.2} ns/op  sample={warmup:?}");
}

fn main() {
    println!("SwissMath Core v0.6 research primitives (lower is better)");
    measure("extended_gcd", 100_000, || {
        extended_gcd(black_box(u64::MAX), black_box(u64::MAX - 58))
    });
    measure("next_prime", 10_000, || {
        next_prime(black_box(1_000_000_000)).unwrap()
    });
    measure("previous_prime", 10_000, || {
        previous_prime(black_box(1_000_000_000)).unwrap()
    });
    measure("rational_reconstruct", 100_000, || {
        rational_reconstruct(black_box(1_113), black_box(10_009))
            .unwrap()
            .unwrap()
    });
    let highly_composite = factor(897_612_484_786_617_600).unwrap();
    measure("divisor_enumeration", 1_000, || {
        black_box(highly_composite.divisors()).len()
    });
}
