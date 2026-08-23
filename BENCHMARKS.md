# Phase 0 benchmark notes

Measurements are deliberately kept reproducible in source rather than encoded
as runtime configuration. Run them with the stable Cargo commands in `README.md`.

## Inline threshold decision

The end-to-end surrogate in `crates/core/benches/inline_thresholds.rs` performs
construction plus intersection scans across word and storage boundaries. Each
candidate exposes only its active logical words, matching the production
invariant. On the initial Windows x86_64 release build (ThinLTO,
`codegen-units=1`), best of seven runs on 2026-08-15 was:

| Private threshold | Best elapsed |
| ---: | ---: |
| 1 word | 1,460,900 ns |
| 2 words | 1,230,600 ns |
| 4 words | 814,700 ns |

All runs produced the same nonzero checksum (`3156`). The private production
choice is therefore `INLINE_WORDS = 4`. This is an initial local result, not a
claim that 4 is universally optimal; the benchmark remains available for both
official architectures.

## Build profile decision

Three separately compiled binaries were warmed once and then run sequentially.
Representative macro-workload results were:

| Workload | Normal release | ThinLTO | ThinLTO + 1 CGU |
| --- | ---: | ---: | ---: |
| sum of two squares | 34.880 ms | 35.615 ms | 33.881 ms |
| Pythagorean congruences | 5.963 ms | 6.887 ms | 5.840 ms |
| covering congruences | 1.124 ms | 1.001 ms | 0.988 ms |
| Lonely-Runner-like filter | 0.712 ms | 0.693 ms | 0.511 ms |
| CRT compatibility | 0.086 ms | 0.083 ms | 0.084 ms |

ThinLTO plus one code-generation unit won most representative workloads without
a material CRT regression, so both release and benchmark profiles use it. No
target-specific CPU flags, manual SIMD, allocator changes, or unsafe code are
enabled.

The coarse workload profile also shows that the cartesian-product algorithms in
the sum-of-two-squares and Pythagorean harnesses dominate runtime. This is an
algorithm-level property of the intentionally simple prototypes, not evidence
for complicating modular multiplication or the bitset kernels, so Phase 0 keeps
the safe baseline unchanged.

## v0.3 Modular Sieve: prepared search vs build + search

The deterministic `crates/core/benches/sieve.rs` harness compares the normalized
anchor-based sieve with direct reference filtering on `0 ..= 200000`. `prepared`
constructs the normalized `ModularSieve` before timing; `build + search` includes
filter construction, normalization, and the search itself. One local Windows
x86_64 release run on 2026-08-15 reported:

| Workload | Prepared search | Build + search | Direct reference | Prepared speedup | End-to-end speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| Dense | 254,030 ns/op | 232,965 ns/op | 321,615 ns/op | 1.27× | 1.38× |
| Sparse anchor | 4,615 ns/op | 4,665 ns/op | 342,925 ns/op | 74.31× | 73.51× |
| Multiple moduli | 249,775 ns/op | 249,795 ns/op | 478,555 ns/op | 1.92× | 1.92× |
| Shared modulus after intersection | 58,615 ns/op | 54,265 ns/op | 514,820 ns/op | 8.78× | 9.49× |

The benchmark also asserts that every optimized count equals the direct
reference count. These are directional local measurements, not hard thresholds;
the release keeps the straightforward normalized anchor planner and does not
add a wheel, global LCM fusion, SIMD, or parallel execution.

## v0.3 Prime & Factor

The deterministic `crates/core/benches/number_theory.rs` harness consumes one
canonical factorization per iteration and reports categories separately. One
local Windows x86_64 release run on 2026-08-15 reported:

| Category | Fixed input | Factorization |
| --- | ---: | ---: |
| Easy composite | 2,000,000,014 | 1,795 ns/op |
| Large prime | 18,446,744,073,709,551,557 | 5,850 ns/op |
| Prime power | 9,223,372,036,854,775,808 | 930 ns/op |
| Balanced semiprime | 18,446,743,979,220,271,189 | 1,518,860 ns/op |
| Mixed composite | 54,177,693,696,000 | 610 ns/op |

These are representative directional timings, not hard thresholds. Pollard–Brent
uses deterministic retries and a finite work budget; no randomized fallback or
parallel factorization is enabled.

## v0.4 Large primality: BPSW

The `large_primality` harness routes decimal input through the exact u64 path
when possible and otherwise parses `BigUint` and runs the explicit BPSW
configuration from `num-prime`. Probable primes and composites are reported
separately. One local Windows x86_64 release run on 2026-08-15 reported:

| Category | Bits | Assessment | Time |
| --- | ---: | --- | ---: |
| Probable prime 127 | 127 | BPSW probable prime | 1,624,650 ns/op |
| Composite 127 | 129 | Composite | 27,262.5 ns/op |
| Probable prime 521 | 521 | BPSW probable prime | 15,466,050 ns/op |
| Composite 521 | 523 | Composite | 141,725 ns/op |
| Probable prime 1279 | 1,279 | BPSW probable prime | 126,348,200 ns/op |
| Composite 1279 | 1,281 | Composite | 1,044,550 ns/op |
| Probable prime 2203 | 2,203 | BPSW probable prime | 659,351,600 ns/op |
| Composite 2203 | 2,205 | Composite | 4,873,900 ns/op |

These measurements include decimal parsing and are directional; no timing
threshold is enforced.

## v0.4 Quadratic arithmetic

The `quadratic` harness measures the direct prime path, Tonelli–Shanks,
odd-prime-power Hensel lifting, powers of two, and composite-unit CRT roots
separately. One local Windows x86_64 release run on 2026-08-15 reported:

| Category | Fixed modulus | Roots | Time |
| --- | ---: | ---: | ---: |
| Prime, p % 4 = 3 | 18,446,744,073,709,551,427 | 2 | 7,775 ns/op |
| Tonelli–Shanks, p % 4 = 1 | 18,446,744,073,709,551,557 | 2 | 9,550 ns/op |
| Odd prime power | 13⁸ = 815,730,721 | 2 | 1,175 ns/op |
| Power of two | 2⁴⁰ = 1,099,511,627,776 | 4 | 3,210 ns/op |
| Composite unit modulus | 3⁸·5⁶ = 102,515,625 | 4 | 4,365 ns/op |

The existing baseline, sieve, and u64 number-theory benchmarks remain separate;
the new arbitrary-precision and quadratic paths are not inserted into their
hot loops.

## v0.4 regression rerun

After integration, the existing sieve and u64 factorization harnesses were run
again. All optimized sieve counts still equal the direct reference counts:

| Sieve workload | Prepared | Build + search | Direct reference | End-to-end speedup |
| --- | ---: | ---: | ---: | ---: |
| Dense | 228,050 ns/op | 232,480 ns/op | 323,575 ns/op | 1.39× |
| Sparse anchor | 4,610 ns/op | 4,965 ns/op | 339,400 ns/op | 68.36× |
| Multiple moduli | 245,630 ns/op | 244,985 ns/op | 464,365 ns/op | 1.90× |
| Shared modulus | 52,320 ns/op | 52,280 ns/op | 482,490 ns/op | 9.23× |

The v0.4 u64 factorization rerun reported 2,440 ns/op for the easy composite,
7,890 ns/op for the large prime, 1,745 ns/op for the prime power, 1,419,780
ns/op for the balanced semiprime, and 475 ns/op for the mixed composite. These
remain directional local measurements; the new paths do not enter those hot
loops.

## v0.5 Exact-first u128 primality

The `large_primality` harness now reports the bounded u128 route separately from
the existing >u128 BPSW reference cases. One local Windows x86_64 release run
on 2026-08-22 reported:

| Category | Bits | Assessment | Time |
| --- | ---: | --- | ---: |
| u128 cheap composite | 65 | Composite | 112.50 ns/op |
| u128 proof-friendly prime | 96 | PrimeExact | 3,096,200 ns/op |
| u128 incomplete proof (M127) | 127 | ExactProofIncomplete | 1,629,200 ns/op |
| >u128 probable prime 521 | 521 | BPSW probable prime | 15,351,600 ns/op |
| >u128 composite 521 | 523 | Composite | 134,925 ns/op |
| >u128 probable prime 1279 | 1,279 | BPSW probable prime | 123,167,250 ns/op |
| >u128 composite 1279 | 1,281 | Composite | 892,400 ns/op |
| >u128 probable prime 2203 | 2,203 | BPSW probable prime | 591,916,200 ns/op |
| >u128 composite 2203 | 2,205 | Composite | 4,106,300 ns/op |

These measurements include the requested decimal/u128 routing where applicable
and remain directional; no timing threshold is enforced.

## v0.5 regression rerun

The existing hot-loop harnesses were rerun after adding the u128 path. The
sieve still reports prepared search separately from build + normalization +
search, and all checksums match their direct references:

| Sieve workload | Prepared | Build + search | Direct reference | End-to-end speedup |
| --- | ---: | ---: | ---: | ---: |
| Dense | 298,105 ns/op | 330,075 ns/op | 440,805 ns/op | 1.34× |
| Sparse anchor | 6,110 ns/op | 6,170 ns/op | 467,720 ns/op | 75.81× |
| Multiple moduli | 338,915 ns/op | 368,770 ns/op | 656,685 ns/op | 1.78× |
| Shared modulus | 69,810 ns/op | 70,325 ns/op | 624,335 ns/op | 8.88× |

The u64 factorization harness reported 2,285 ns/op for the easy composite,
12,800 ns/op for the large prime, 1,320 ns/op for the prime power, 1,084,460
ns/op for the balanced semiprime, and 535 ns/op for the mixed composite. The
baseline and quadratic harnesses also completed successfully; the new u128
proof path does not enter their hot loops.

## v0.6 Research primitives

The focused `research_primitives` harness was run locally on Windows x86_64 on
2026-08-23. Results are directional and include deterministic fixed inputs:

| Operation | Time |
| --- | ---: |
| Extended GCD near u64 max | 131.50 ns/op |
| Next prime after 1,000,000,000 | 979.05 ns/op |
| Previous prime before 1,000,000,000 | 2,561.17 ns/op |
| Rational reconstruction modulo 10,009 | 53.62 ns/op |
| Divisors of 897,612,484,786,617,600 | 1,378,565.90 ns/op |

Divisor enumeration materializes the full sorted list and is intentionally
opt-in. Scalar factorization-derived functions are not benchmarked separately.

## Research Workflow v0.1

One local optimized Windows x86_64 end-to-end CLI run on 2026-08-23 reported:

| Workload | Records | Elapsed | Operations/s |
| --- | ---: | ---: | ---: |
| Process startup (`--help`, 20-run average) | 1 | 41.74 ms | — |
| Scalar prime JSON (20-run average) | 1 | 28.03 ms | — |
| Prime JSONL | 10 | 47.17 ms | 212 |
| Prime JSONL | 100 | 26.21 ms | 3,816 |
| Prime JSONL | 1,000 | 31.14 ms | 32,113 |
| Prime JSONL | 10,000 | 145.13 ms | 68,902 |
| Factor 360 JSONL | 10 | 17.68 ms | 566 |
| Factor 360 JSONL | 100 | 25.92 ms | 3,858 |
| Factor 360 JSONL | 1,000 | 51.79 ms | 19,309 |
| Factor 360 JSONL | 10,000 | 122.78 ms | 81,446 |

Small-batch figures are dominated by Windows process and PowerShell pipeline
startup. JSONL records are processed sequentially and are not retained.

The local production WASM bundle reported approximately 211.2 ms initialization,
3.5 ms for one browser prime request, 9.6 ms for 100 rows, and 42.9 ms for 1,000
rows including construction of the simple result table. These browser timings
are directional and hardware/session dependent.
