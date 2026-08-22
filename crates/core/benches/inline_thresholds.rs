use std::{hint::black_box, time::Instant};

enum CandidateStorage<const N: usize> {
    Inline([u64; N]),
    Heap(Vec<u64>),
}

struct CandidateSet<const N: usize> {
    words: CandidateStorage<N>,
    active_words: usize,
}

impl<const N: usize> CandidateSet<N> {
    fn build(modulus: usize, salt: u64) -> Self {
        let count = modulus.div_ceil(64);
        let mut result = if count <= N {
            Self {
                words: CandidateStorage::Inline([0; N]),
                active_words: count,
            }
        } else {
            Self {
                words: CandidateStorage::Heap(vec![0; count]),
                active_words: count,
            }
        };
        for (index, word) in result.words_mut().iter_mut().enumerate() {
            *word = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ salt;
        }
        let tail = modulus % 64;
        if tail != 0 {
            *result.words_mut().last_mut().unwrap() &= (1_u64 << tail) - 1;
        }
        result
    }

    fn words(&self) -> &[u64] {
        match &self.words {
            CandidateStorage::Inline(words) => &words[..self.active_words],
            CandidateStorage::Heap(words) => words,
        }
    }

    fn words_mut(&mut self) -> &mut [u64] {
        match &mut self.words {
            CandidateStorage::Inline(words) => &mut words[..self.active_words],
            CandidateStorage::Heap(words) => words,
        }
    }
}

fn workload<const N: usize>() -> (u128, u64) {
    let moduli = [
        32, 64, 65, 127, 128, 129, 191, 192, 193, 255, 256, 257, 1_000, 10_000,
    ];
    let start = Instant::now();
    let mut checksum = 0_u64;
    for round in 0..20_000_u64 {
        let modulus = moduli[round as usize % moduli.len()];
        let left = CandidateSet::<N>::build(modulus, round);
        let right = CandidateSet::<N>::build(
            modulus,
            round.rotate_left(17).wrapping_add(0xd1b5_4a32_d192_ed03),
        );
        checksum ^= left
            .words()
            .iter()
            .zip(right.words())
            .map(|(a, b)| u64::from((a & b).count_ones()))
            .sum::<u64>();
        black_box((&left, &right));
    }
    (start.elapsed().as_nanos(), checksum)
}

fn report<const N: usize>() {
    let mut best = u128::MAX;
    let mut checksum = 0;
    for _ in 0..7 {
        let (elapsed, value) = workload::<N>();
        best = best.min(elapsed);
        checksum ^= value;
    }
    println!("INLINE_WORDS={N}: best {best} ns, checksum={checksum}");
}

fn main() {
    println!("End-to-end inline threshold experiment (construction + scan)");
    report::<1>();
    report::<2>();
    report::<4>();
}
