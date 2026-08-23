use std::{hint::black_box, time::Instant};

use swissmath_core::{PrimeField, discrete_log, linear_recurrence_nth_mod_prime, primitive_root};

fn measure<T>(name: &str, repetitions: u64, mut operation: impl FnMut() -> T) {
    black_box(operation());
    let started = Instant::now();
    for _ in 0..repetitions {
        black_box(operation());
    }
    let milliseconds = started.elapsed().as_secs_f64() * 1_000.0 / repetitions as f64;
    println!("{name:40} {milliseconds:12.4} ms/op");
}

fn recurrence_inputs(order: usize) -> (Vec<i128>, Vec<i128>) {
    let initial = (0..order).map(|index| index as i128 + 1).collect();
    let coefficients = (0..order)
        .map(|index| if index % 3 == 0 { 1 } else { 0 })
        .collect();
    (initial, coefficients)
}

fn main() {
    println!("SwissMath Core v0.8 sequence/group benchmarks (lower is better)");
    let recurrence_field = PrimeField::new(1_000_000_007).unwrap();
    for order in [2, 8, 32, 64] {
        let (initial, coefficients) = recurrence_inputs(order);
        let repetitions = if order <= 8 { 100 } else { 20 };
        measure(
            &format!("recurrence nth k={order}, n=10^18"),
            repetitions,
            || {
                linear_recurrence_nth_mod_prime(
                    &initial,
                    &coefficients,
                    1_000_000_000_000_000_000,
                    recurrence_field,
                )
                .unwrap()
            },
        );
    }

    for prime in [65_537, 1_000_000_007, 20_000_000_687] {
        let field = PrimeField::new(prime).unwrap();
        measure(&format!("primitive root p={prime}"), 3, || {
            primitive_root(field).unwrap()
        });
    }

    let smooth = PrimeField::new(65_537).unwrap();
    let smooth_generator = primitive_root(smooth).unwrap();
    let smooth_target = smooth.pow(smooth_generator, 42_001);
    measure("dlog smooth p=65537", 5, || {
        discrete_log(smooth_generator.into(), smooth_target.into(), smooth).unwrap()
    });

    let medium = PrimeField::new(1_000_000_007).unwrap();
    let medium_generator = primitive_root(medium).unwrap();
    let medium_target = medium.pow(medium_generator, 123_456_789);
    measure("dlog medium factor p=1000000007", 1, || {
        discrete_log(medium_generator.into(), medium_target.into(), medium).unwrap()
    });

    // This safe prime has p-1 = 2 * 10000000343. Its prime factor requires
    // more than 100000 baby steps, so the public path refuses before allocation.
    let limited = PrimeField::new(20_000_000_687).unwrap();
    let limited_generator = primitive_root(limited).unwrap();
    measure("dlog bounded early limit", 3, || {
        discrete_log(limited_generator.into(), 1, limited).unwrap()
    });
}
