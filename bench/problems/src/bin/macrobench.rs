use std::{hint::black_box, time::Instant};

use swissmath_core::{Congruence, ModCtx, Modulus, ResidueSet, crt_pair};

fn modulus(value: u64) -> Modulus {
    Modulus::new(value).unwrap()
}

fn timed(name: &str, operation: impl FnOnce() -> u64) {
    let start = Instant::now();
    let checksum = black_box(operation());
    println!(
        "{name:34} {:>10.3} ms  checksum={checksum}",
        start.elapsed().as_secs_f64() * 1e3
    );
}

fn quadratic_residues(m: u64) -> ResidueSet {
    let ctx = ModCtx::new(modulus(m));
    ResidueSet::from_iter(modulus(m), (0..m).map(|x| ctx.mul(x, x))).unwrap()
}

fn main() {
    timed("quadratic residues", || quadratic_residues(100_003).len());
    timed("sum-of-two-squares residues", || {
        let squares = quadratic_residues(10_007);
        let ctx = ModCtx::new(modulus(10_007));
        ResidueSet::from_iter(
            modulus(10_007),
            squares
                .iter()
                .flat_map(|a| squares.iter().map(move |b| ctx.add(a, b))),
        )
        .unwrap()
        .len()
    });
    timed("Pythagorean congruences", || {
        let squares = quadratic_residues(4_099);
        let ctx = ModCtx::new(modulus(4_099));
        squares
            .iter()
            .flat_map(|a| squares.iter().map(move |b| ctx.add(a, b)))
            .filter(|sum| squares.contains(*sum))
            .count() as u64
    });
    timed("covering congruences", || {
        let mut survivors = ResidueSet::try_full(modulus(120_120)).unwrap();
        for divisor in [3, 5, 7, 8, 11, 13] {
            let allowed =
                ResidueSet::from_predicate(modulus(120_120), |x| x % divisor != 0).unwrap();
            survivors.intersect_assign(&allowed).unwrap();
        }
        survivors.len()
    });
    timed("periodic modular filters", || {
        let first = ResidueSet::from_predicate(modulus(90_090), |x| x % 9 < 3).unwrap();
        let second = ResidueSet::from_predicate(modulus(90_090), |x| x % 13 == 1).unwrap();
        first.intersection_count(&second).unwrap()
    });
    timed("small Diophantine sieve", || {
        let m = 65_537;
        let ctx = ModCtx::new(modulus(m));
        ResidueSet::from_predicate(modulus(m), |x| {
            let cube = ctx.mul(ctx.mul(x, x), x);
            ctx.add(cube, 17) == 0
        })
        .unwrap()
        .len()
    });
    timed("Lonely-Runner-like filter", || {
        let m = 100_000;
        let mut allowed = ResidueSet::try_full(modulus(m)).unwrap();
        for speed in [2, 3, 5, 7, 11] {
            let band = ResidueSet::from_predicate(modulus(m), |t| {
                let phase = t * speed % m;
                phase >= m / 6 && phase <= 5 * m / 6
            })
            .unwrap();
            allowed.intersect_assign(&band).unwrap();
        }
        allowed.len()
    });
    timed("CRT compatibility workload", || {
        let mut checksum = 0;
        for a in 0..1_000 {
            let left = Congruence::new(a, modulus(1_009));
            let right = Congruence::new(a * 17 + 3, modulus(1_013));
            checksum ^= crt_pair(left, right).unwrap().unwrap().residue();
        }
        checksum
    });
}
