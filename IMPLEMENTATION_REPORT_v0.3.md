# SwissMath v0.3 implementation report

## Scope delivered

SwissMath v0.3 extends the validated v0.2 codebase without changing the
dependency direction: plain local GUI → small Tauri IPC layer → independently
usable `swissmath-core`. No production dependency, network service, telemetry,
thread, plugin, database, JavaScript mathematical implementation, or `unsafe`
code was added.

## Preflight corrections

- `ModularFilter::from_excluded()` now constructs the excluded set and calls
  `complement_assign()`, reusing its allocation and avoiding the allocating
  `complement()` panic path in production.
- The sieve benchmark retains prepared-search timings and now also measures
  construction + normalization + search. `BENCHMARKS.md` labels the two paths
  separately.
- `scripts/package-source.ps1` now writes ZIP entries with portable `/`
  separators using the standard .NET compression API and validates the v0.3
  single-root archive.

## Files changed

- `crates/core/src/number_theory.rs`: primality, factorization, φ, λ, integer
  analysis, and multiplicative order.
- `crates/core/src/lib.rs`: public v0.3 exports.
- `crates/core/tests/number_theory.rs`: independent primality, factorization,
  arithmetic-function, and order checks.
- `crates/core/benches/number_theory.rs`: deterministic category benchmark.
- `crates/core/src/sieve.rs`, `crates/core/benches/sieve.rs`: preflight fixes.
- `apps/desktop/src-tauri/src/lib.rs`: analysis and order commands with
  decimal-string IPC responses and command tests.
- `apps/desktop/ui/index.html`, `app.js`, `styles.css`: `Numeri interi` screen
  and one-call analysis UI.
- `README.md`, `BENCHMARKS.md`, `CHANGELOG.md`, `NOT_NOW.md`, and this report.
- `scripts/package-source.ps1`: v0.3 portable source bundle.

## Algorithms and APIs

`is_prime(u64)` handles trivial values and the shared small-prime table before
using Miller–Rabin with witnesses `2, 325, 9375, 28178, 450775, 9780504,
1795265022`, which is deterministic for the complete u64 domain. It reuses
`ModCtx::pow` and the existing exact u128-backed modular multiplication.

`factor(u64)` rejects zero, returns the empty factorization for one, strips the
shared small-prime table, detects prime remainders early, and recursively splits
composites with Pollard–Brent. The polynomial is `x² + c mod n`; differences
are batched with a private batch size of 96. Deterministic SplitMix64-style
seeds derive starting states and polynomial constants from `n` and the attempt
index. There are 32 attempts with a 2,000,000-step per-attempt work budget and
a bounded 4,096-step `g == n` recovery. Every returned divisor is verified
before recursion; failure returns `SearchFailed` rather than an unverified
factor.

`Factorization` stores sorted unique `PrimePower` values. Euler φ uses
`(result / p) * (p - 1)`. Carmichael λ uses the prime-power rules for odd
primes and powers of two, then checked LCM. `analyze_integer(n)` factors once
and returns classification, factors, φ, and λ together. `multiplicative_order`
first checks coprimality, uses λ(n) as the initial bound, factors λ once, and
reduces the candidate by prime divisors with modular exponentiation.

## GUI integration

`Numeri interi` sends one `analyze_integer` IPC request for n and displays type,
canonical factorization, φ(n), and λ(n). The same screen keeps the selected n
for a separate `calculate_multiplicative_order` request. Large values remain
decimal strings across IPC; JavaScript only validates input with `BigInt`.

## Validation

The v0.3 validation suite includes the existing v0.2 arithmetic, CRT, residue,
congruence, and sieve tests; an independent trial-division comparison for
`is_prime()` over `0 ..= 100000`; Carmichael numbers and strong pseudoprimes;
large prime/composite and u64-boundary cases; factorization reconstruction and
prime-base invariants; φ brute-force checks; λ exponent checks for small
moduli; brute-force multiplicative-order comparisons; and desktop IPC tests for
the one-call analysis and order outcomes.

## Benchmarks and limitations

The local benchmark run recorded in `BENCHMARKS.md` measured the balanced
32-bit-prime semiprime `18,446,743,979,220,271,189` at 1,518,860 ns/op, the
large prime `18,446,744,073,709,551,557` at 5,850 ns/op, and the other fixed
categories separately. The sieve benchmark now reports both prepared and
end-to-end paths; all optimized counts still match direct reference filtering.

The factorizer is intentionally bounded and dependency-free. A pathological
input can return `SearchFailed`; no such case occurred in the release test or
benchmark corpus. Arbitrary precision, prime generation/counting, divisor
functions, modular square roots, advanced Pollard/ECM/SQUFOF/quadratic-sieve
methods, Montgomery/Barrett arithmetic, parallelism, and caching remain out of
scope; see `NOT_NOW.md`.

## Distribution

- Windows x64 standalone executable and optional NSIS installer:
  `release/SwissMath-v0.3-win-x64/`.
- External-review source bundle:
  `release/SwissMath-v0.3-source.zip`.

The Windows artifacts are unsigned and require the WebView2 runtime supplied by
Windows. Release hashes are recorded in `release/SwissMath-v0.3-win-x64/SHA256SUMS.txt`.

```text
3D40EE0CE873113319CA653F5877A65BE0705397748650CF55C8B5B39BEF523A  SwissMath-0.3.0-x64.exe
6523CE90701FAE5A44D1C85E84EFBAE0C09A856EE424F9CA978228DB7CA59F28  SwissMath_0.3.0_x64-setup.exe
```
