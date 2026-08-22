use std::{hint::black_box, time::Instant};

use swissmath_core::{ModularFilter, ModularSieve, Modulus};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

fn allowed(modulus_value: u64, values: impl IntoIterator<Item = u64>) -> ModularFilter {
    ModularFilter::from_allowed(modulus(modulus_value), values).unwrap()
}

fn reference_count(start: u64, end: u64, filters: &[ModularFilter]) -> u128 {
    (start..=end)
        .filter(|candidate| {
            filters.iter().all(|filter| {
                filter
                    .allowed()
                    .contains(*candidate % filter.modulus().get())
            })
        })
        .count() as u128
}

fn measure_sieve(sieve: &ModularSieve, start: u64, end: u64, repetitions: u64) -> (f64, u128) {
    let started = Instant::now();
    let mut checksum = 0_u128;
    for _ in 0..repetitions {
        checksum =
            checksum.wrapping_add(black_box(sieve.search(start, end, 0).unwrap()).survivor_count);
    }
    (
        started.elapsed().as_nanos() as f64 / repetitions as f64,
        checksum,
    )
}

fn measure_reference(
    start: u64,
    end: u64,
    filters: &[ModularFilter],
    repetitions: u64,
) -> (f64, u128) {
    let started = Instant::now();
    let mut checksum = 0_u128;
    for _ in 0..repetitions {
        checksum = checksum.wrapping_add(black_box(reference_count(start, end, filters)));
    }
    (
        started.elapsed().as_nanos() as f64 / repetitions as f64,
        checksum,
    )
}

fn measure_build_and_search(
    filters: &[ModularFilter],
    start: u64,
    end: u64,
    repetitions: u64,
) -> (f64, u128) {
    let started = Instant::now();
    let mut checksum = 0_u128;
    for _ in 0..repetitions {
        let sieve = ModularSieve::new(filters.iter().cloned()).unwrap();
        checksum =
            checksum.wrapping_add(black_box(sieve.search(start, end, 0).unwrap()).survivor_count);
    }
    (
        started.elapsed().as_nanos() as f64 / repetitions as f64,
        checksum,
    )
}

fn report(name: &str, filters: Vec<ModularFilter>, start: u64, end: u64) {
    let expected = reference_count(start, end, &filters);
    let sieve = ModularSieve::new(filters.clone()).unwrap();
    let actual = sieve.search(start, end, 0).unwrap().survivor_count;
    assert_eq!(actual, expected, "benchmark reference mismatch for {name}");

    let repetitions = 20;
    let (optimized_ns, optimized_checksum) = measure_sieve(&sieve, start, end, repetitions);
    let (build_search_ns, build_search_checksum) =
        measure_build_and_search(&filters, start, end, repetitions);
    let (reference_ns, reference_checksum) = measure_reference(start, end, &filters, repetitions);
    assert_eq!(optimized_checksum, reference_checksum);
    assert_eq!(build_search_checksum, reference_checksum);
    println!(
        "{name:18} prepared={optimized_ns:12.2} ns/op  build+search={build_search_ns:12.2} ns/op  reference={reference_ns:12.2} ns/op  prepared_speedup={:6.2}x  end_to_end_speedup={:6.2}x  survivors={expected}",
        reference_ns / optimized_ns,
        reference_ns / build_search_ns,
    );
}

fn main() {
    println!("SwissMath Modular Sieve v1 (lower is better; deterministic workloads)");
    let range = (0, 200_000);
    report("dense", vec![allowed(5, [0, 1, 2, 3])], range.0, range.1);
    report("sparse", vec![allowed(97, [13])], range.0, range.1);
    report(
        "multiple moduli",
        vec![
            allowed(5, [1, 4]),
            allowed(7, [0, 1, 6]),
            allowed(8, [1, 3, 5, 7]),
        ],
        range.0,
        range.1,
    );
    report(
        "shared modulus",
        vec![
            allowed(12, [1, 2, 3, 7]),
            allowed(12, [2, 3, 4, 8]),
            allowed(5, [0, 1, 2, 3, 4]),
        ],
        range.0,
        range.1,
    );
}
