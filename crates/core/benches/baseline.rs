use std::{hint::black_box, time::Instant};

use swissmath_core::{
    Congruence, ModCtx, Modulus, ResidueSet, crt_compatible, crt_pair, gcd, inv_mod,
};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

fn measure(mut operation: impl FnMut() -> u64, iterations: u64) -> (f64, u64) {
    let start = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..iterations {
        let value = black_box(&mut operation)();
        checksum = checksum.wrapping_add(black_box(value));
    }
    (
        start.elapsed().as_nanos() as f64 / iterations as f64,
        checksum,
    )
}

fn report(name: &str, iterations: u64, operation: impl FnMut() -> u64) {
    let (nanoseconds, checksum) = measure(operation, iterations);
    println!("{name:42} {nanoseconds:12.2} ns/op  checksum={checksum}");
}

fn arithmetic_benchmarks() {
    let small = ModCtx::new(modulus(1_000_000_007));
    let large = ModCtx::new(modulus(u64::MAX - 58));
    report("ModCtx::add", 2_000_000, || {
        small.add(900_000_000, 800_000_000)
    });
    report("ModCtx::sub", 2_000_000, || small.sub(3, 900_000_000));
    report("ModCtx::mul u64", 2_000_000, || {
        small.mul(900_000_000, 800_000_000)
    });
    report("ModCtx::mul u128", 1_000_000, || {
        large.mul(u64::MAX - 60, u64::MAX - 61)
    });
    report("gcd", 1_000_000, || {
        gcd(18_446_744_073_709_551_557, 12_345_678_901_234_567)
    });
    report("inv_mod", 200_000, || {
        inv_mod(1_000_000_006, modulus(1_000_000_007)).unwrap()
    });

    let left = Congruence::new(12_345, modulus(65_537));
    let right = Congruence::new(23_456, modulus(65_539));
    report("crt_compatible", 500_000, || {
        u64::from(crt_compatible(left, right))
    });
    report("crt_pair", 200_000, || {
        crt_pair(left, right).unwrap().unwrap().residue()
    });
}

fn residue_benchmarks() {
    let moduli = [
        32, 64, 65, 127, 128, 129, 256, 1_000, 10_000, 100_000, 1_000_000,
    ];
    let densities = [(1_u64, 1_000_u64), (1, 100), (1, 10), (1, 2), (9, 10)];

    for m in moduli {
        for (numerator, denominator) in densities {
            let label = format!("construction m={m} d={numerator}/{denominator}");
            let repeats = if m <= 1_000 { 100 } else { 1 };
            report(&label, repeats, || {
                ResidueSet::from_predicate(modulus(m), |r| {
                    (r.wrapping_mul(6_364_136_223_846_793_005) % denominator) < numerator
                })
                .unwrap()
                .len()
            });
        }
    }

    for m in moduli {
        let a = ResidueSet::from_predicate(modulus(m), |r| r % 10 < 5).unwrap();
        let b = ResidueSet::from_predicate(modulus(m), |r| r.wrapping_mul(17) % 10 < 5).unwrap();
        let mut insert_work = a.clone();
        let mut remove_work = a.clone();
        let mut intersection_work = a.clone();
        let mut union_work = a.clone();
        let mut difference_work = a.clone();
        let scalar_repeats = 100_000_u64.min(10_000_000 / m.max(1)).max(10);
        let scan_repeats = (1_000_000 / m.max(1)).clamp(1, 10_000);

        report(&format!("contains m={m}"), scalar_repeats, || {
            u64::from(a.contains(m / 2))
        });
        report(&format!("insert m={m}"), scalar_repeats, || {
            let _ = insert_work.remove(m - 1).unwrap();
            u64::from(insert_work.insert(m - 1).unwrap())
        });
        report(&format!("remove m={m}"), scalar_repeats, || {
            let _ = remove_work.insert(m / 2).unwrap();
            u64::from(remove_work.remove(m / 2).unwrap())
        });
        report(&format!("intersection_count m={m}"), scan_repeats, || {
            a.intersection_count(&b).unwrap()
        });
        report(&format!("intersects m={m}"), scan_repeats, || {
            u64::from(a.intersects(&b).unwrap())
        });
        report(&format!("is_subset_of m={m}"), scan_repeats, || {
            u64::from(a.is_subset_of(&b).unwrap())
        });
        report(&format!("intersection m={m}"), scan_repeats, || {
            a.intersection(&b).unwrap().len()
        });
        report(&format!("intersect_assign m={m}"), scan_repeats, || {
            intersection_work.intersect_assign(&b).unwrap();
            intersection_work.len()
        });
        report(&format!("union m={m}"), scan_repeats, || {
            a.union(&b).unwrap().len()
        });
        report(&format!("union_assign m={m}"), scan_repeats, || {
            union_work.union_assign(&b).unwrap();
            union_work.len()
        });
        report(&format!("difference m={m}"), scan_repeats, || {
            a.difference(&b).unwrap().len()
        });
        report(&format!("difference_assign m={m}"), scan_repeats, || {
            difference_work.difference_assign(&b).unwrap();
            difference_work.len()
        });
        report(&format!("complement m={m}"), scan_repeats, || {
            a.complement().len()
        });
        report(&format!("iteration m={m}"), scan_repeats, || {
            a.iter().fold(0, u64::wrapping_add)
        });
    }
}

fn main() {
    println!("SwissMath stable-Rust baseline (lower is better)");
    arithmetic_benchmarks();
    residue_benchmarks();
}
