# SwissMath v0.4 implementation report

## Scope delivered

SwissMath v0.4 extends the validated v0.3 workspace without changing its
dependency direction: local HTML/CSS/JavaScript GUI → small Tauri IPC adapter →
independently usable `swissmath-core`. The exact modular/arithmetic core stays
`u64`; arbitrary precision is deliberately limited to decimal primality
assessment.

## Preflight and number-theory corrections

- `ModularFilter::from_excluded()` still uses `complement_assign()` in-place.
- The sieve benchmark still distinguishes prepared search from build +
  normalization + search, and checks both against direct reference counts.
- The source-packaging script still emits one top-level ZIP directory with `/`
  separators and rejects excluded/compiled artifacts.
- `Factorization` now stores its represented integer and exposes `n()`,
  `factors()`, `euler_phi()`, and `carmichael_lambda()` without a duplicate `n`
  argument. Integer analysis, order calculation, tests, benchmarks, and the
  desktop adapter use this API.
- The recursive factor path relies on the invariant established by the initial
  `SMALL_PRIMES` strip. Redundant scans in `collect_factors()`, private Pollard
  entry checks for 2/3, and the immediately repeated even branch in
  `is_prime()` were removed without changing public correctness.
- A fixed 16-input balanced-semiprime Pollard corpus near the 32-bit range was
  added. Every case factors successfully, reconstructs exactly, and has prime
  bases.

## Large-number primality

`is_prime(u64)` remains SwissMath's exact deterministic Miller–Rabin path over
the complete u64 domain. `assess_primality_decimal()` first validates decimal
syntax and parses once; values fitting u64 route directly to `is_prime`, while
larger values use `num-bigint` and `num-prime` 0.5.0 with only the `big-int`
feature and `PrimalityTestConfig::bpsw()`.

The semantic result is one of `Composite`, `PrimeExact`, or `ProbablePrime`.
The decimal one-call analysis returns full factors/φ/λ only for exact u64
values. Larger values return a normalized decimal string and a BPSW
assessment; they are never presented as an exact prime and are never factored.
Malformed, empty, and negative decimal inputs are rejected.

The deterministic routing tests cover 0, 1, 2, the u64 boundary, large even and
square composites, known Mersenne probable primes at 127, 521, and 1,279 bits,
and a 2,203-bit Mersenne input. Exact-domain routing is compared directly with
the existing `is_prime(u64)` function.

## Quadratic arithmetic

`crates/core/src/quadratic.rs` contains the focused v0.4 module:

- logarithmic binary Jacobi symbols for signed `i128` numerators and positive
  odd denominators;
- Legendre symbols implemented by validating the odd prime and reusing Jacobi;
- prime-modulus roots with the p % 4 = 3 shortcut and compact Tonelli–Shanks;
- one-root Hensel lifting for odd prime powers, with one derivative inverse and
  the second root derived by negation;
- direct bit-by-bit lifting for unit roots modulo powers of two;
- one exact factorization of composite moduli, component solving, and reuse of
  the existing generalized CRT;
- `n = 1` represented canonically by the single root 0;
- explicit `NonCoprimeUnsupported` for composite non-unit right-hand sides.

Prime roots use `PrimeRoots::{None, One, Two}`. Composite roots are materialized
only inside the bounded u64 domain; Tauri sends an exact count and at most the
first 100 ascending roots.

Independent Jacobi tests use trial factorization plus direct prime-symbol
enumeration. Every prime through 257 is checked against brute-force roots for
every residue, composite unit moduli through 128 are checked against complete
brute force, and targeted Hensel tests verify every intermediate modulus.

## GUI and IPC

`analyze_integer` is one logical command. The Rust adapter chooses exact u64
analysis or large BPSW assessment, leaving JavaScript free of mathematical
logic. For large values, factorization, φ, λ, and multiplicative order are
shown as unavailable rather than errors, with the explicit Baillie–PSW note.

The new `Residui quadratici` screen exposes `CALCOLA SIMBOLO` and `TROVA
RADICI`, accepts signed `a`, displays Jacobi and (for an odd prime) Legendre,
and shows existence, exact root count, and a bounded ascending preview. The
existing save/print/timing preflight bar remains shared by the new results.

## Benchmarks

The new `large_primality` benchmark reports probable primes and composites
separately at 127, 521, 1,279, and 2,203 bits. The new `quadratic` benchmark
reports the p % 4 = 3 path, Tonelli–Shanks, odd prime powers, powers of two,
and composite-unit CRT roots separately. Measurements are recorded in
`BENCHMARKS.md` and are directional only; no timing thresholds are enforced.
Existing baseline, sieve, and u64 number-theory benchmarks remain outside the
new hot paths.

Representative release measurements were: BPSW probable primes at 127/521/
1,279/2,203 bits in 1.624650 ms / 15.466050 ms / 126.348200 ms / 659.351600
ms per operation, with the corresponding small-factor composites at 0.027263
ms / 0.141725 ms / 1.044550 ms / 4.873900 ms. Quadratic timings were 7,775
ns/op for the p % 4 = 3 prime path, 9,550 ns/op for Tonelli–Shanks, 1,175
ns/op for an odd prime power, 3,210 ns/op for 2⁴⁰, and 4,365 ns/op for a
composite unit modulus.

## Validation

The final validation sequence includes:

- `cargo fmt --all --check`;
- `cargo test --workspace --offline`, including v0.2/v0.3 regressions, the
  Pollard corpus, large-primality routing, exhaustive quadratic tests, and
  Tauri command tests;
- `cargo clippy --workspace --all-targets --offline -- -D warnings`;
- JavaScript syntax validation with Node;
- desktop smoke checks for 360 = 2³·3²·5, φ(360) = 96, λ(360) = 12, an exact
  large u64 prime, a >u64 probable prime, Legendre(5/11) = 1, roots 6 and 7
  for x² ≡ 10 (mod 13), Tonelli–Shanks, an odd prime power, a power of two, a
  composite unit, and the unsupported non-coprime composite case.

## Deliberate limitations

- Big integers support primality assessment only; factorization, φ, λ, order,
  and quadratic arithmetic remain u64 operations.
- A large-number `ProbablePrime` result is a BPSW probable-prime result, not an
  exact proof or a probability percentage.
- General composite non-coprime modular roots are not implemented and are
  rejected explicitly.
- Pollard–Brent remains bounded and may return `SearchFailed` on pathological
  u64 inputs; no corpus input produced that result.

## Distribution

The final Windows x64 standalone executable and current-user NSIS installer are
kept outside the source bundle in `release/SwissMath-v0.4-win-x64/`.
The rebuildable source bundle is `release/SwissMath-v0.4-source.zip` and is
verified to have exactly one top-level directory and portable entry paths.

SHA-256:

```text
E0BA48D70228A32E80A9A1AF58CA49D89012D624D47B16E48BC6C7C339447A11  SwissMath-0.4.0-x64.exe
5AB88A96C8AC82ADE2DA504C5C03C31E4A650827EFB760E91B1F7CD27F0F5B0A  SwissMath_0.4.0_x64-setup.exe
```
