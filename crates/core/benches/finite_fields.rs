use std::{hint::black_box, time::Instant};

use swissmath_core::{FpMatrix, FpPolynomial, PrimeField};

fn measure<T>(name: &str, repetitions: u64, mut operation: impl FnMut() -> T) {
    black_box(operation());
    let started = Instant::now();
    for _ in 0..repetitions {
        black_box(operation());
    }
    let milliseconds = started.elapsed().as_secs_f64() * 1_000.0 / repetitions as f64;
    println!("{name:32} {milliseconds:12.4} ms/op");
}

fn matrix(size: usize, field: PrimeField) -> FpMatrix {
    let rows = (0..size)
        .map(|row| {
            (0..size)
                .map(|column| (row * 131 + column * 17 + usize::from(row == column)) as i128)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    FpMatrix::new(field, &rows).unwrap()
}

fn polynomial(degree: usize, field: PrimeField) -> FpPolynomial {
    FpPolynomial::new(
        field,
        &(0..=degree)
            .map(|index| (index * 31 + 1) as i128)
            .collect::<Vec<_>>(),
    )
}

fn main() {
    let field = PrimeField::new(1_000_000_007).unwrap();
    println!("SwissMath Core v0.7 finite-field benchmarks (lower is better)");
    for size in [16, 32, 64, 128] {
        let input = matrix(size, field);
        let repetitions = if size <= 32 { 10 } else { 2 };
        measure(&format!("matrix rref {size}x{size}"), repetitions, || {
            input.rref(field)
        });
    }
    for degree in [16, 64, 256, 512] {
        let left = polynomial(degree, field);
        let right = polynomial(degree, field);
        let repetitions = if degree <= 64 { 100 } else { 10 };
        measure(
            &format!("polynomial mul degree {degree}"),
            repetitions,
            || left.mul(field, &right),
        );
    }
}
