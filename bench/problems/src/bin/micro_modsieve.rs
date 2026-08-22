use std::{hint::black_box, time::Instant};

use swissmath_core::{ModCtx, Modulus, ResidueSet};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

fn polynomial_zeroes(m: u64, exponent: u64, constant: u64) -> ResidueSet {
    let ctx = ModCtx::new(modulus(m));
    ResidueSet::from_predicate(modulus(m), |x| {
        ctx.add(ctx.pow(x, exponent), constant % m) == 0
    })
    .unwrap()
}

fn rank_candidate(m: u64) -> (u64, u64) {
    let ctx = ModCtx::new(modulus(m));
    let square_band = ResidueSet::from_predicate(modulus(m), |x| ctx.pow(x, 2) < m / 4).unwrap();
    let cubic_band =
        ResidueSet::from_predicate(modulus(m), |x| ctx.add(ctx.pow(x, 3), 17 % m) < m / 3).unwrap();
    let survivors = square_band.intersection_count(&cubic_band).unwrap();
    (survivors, m)
}

fn main() {
    // Retain an exact-equation workload alongside the ranking filters.
    black_box(polynomial_zeroes(4_099, 2, 4_098).len());
    let candidates = [
        127, 128, 129, 251, 256, 257, 509, 1_009, 2_003, 4_099, 8_191,
    ];
    let start = Instant::now();
    let mut ranking: Vec<_> = candidates.into_iter().map(rank_candidate).collect();
    ranking.sort_unstable();
    black_box(&ranking);
    println!("Micro-ModSieve candidate ranking (fewest survivors first):");
    for (survivors, m) in ranking {
        println!("  modulus={m:5} survivors={survivors}");
    }
    println!("elapsed: {:.3} ms", start.elapsed().as_secs_f64() * 1e3);
}
