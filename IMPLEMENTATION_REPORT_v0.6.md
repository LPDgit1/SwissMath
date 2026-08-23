# SwissMath Core v0.6 Implementation Report

Date: 2026-08-23

## Recovery

Development resumed from an uncommitted interrupted worktree at `f44c1d1`.
The recovered code already contained partial extended GCD, Möbius/divisor
summary, next-prime, and rational-reconstruction implementations. It also
contained broader experimental arithmetic, fractions, polynomial, linear
algebra, and discovery work. No reset, overwrite, or remote push was used.
The initial evidence matrix is in `V0.6_RECOVERY_AUDIT.md`.

## Completion and repairs

- Extended GCD now covers both inputs over the complete u64 domain with local
  i128 coefficients and iterative Euclid.
- Added `valuation`, with a required exact-prime base and an explicit
  `Valuation::Infinite` result for zero.
- Completed factorization-derived Möbius, radical, squarefree, divisor count,
  divisor sum, and sorted divisor enumeration. `analyze_integer` factors once
  and reuses that object for every scalar result. Enumeration is opt-in.
- Preserved the recovered strict `next_prime` implementation and added strict
  `previous_prime`; overflow/no-result cases are explicit.
- Replaced the incomplete reconstruction surface with a minimal
  `RationalReconstruction { numerator, denominator }` result plus default and
  separately bounded APIs. The unrelated recovered general `Rational` code was
  preserved, not expanded for v0.6.
- Core metadata now reports `0.6.0`.

## Rational reconstruction convention

The default API uses
`A = B = floor(sqrt((m - 1) / 2))`, computed with integer arithmetic. Therefore
`2AB < m`, providing the conventional uniqueness condition. The bounded API
accepts `max_numerator_abs` and `max_denominator` separately. Every success is
normalized to a positive denominator, reduced, checked against both bounds,
and verified against `a ≡ r*b (mod m)`. Invalid parameters are errors; absence
of a reconstruction is `Ok(None)`. Residue zero is canonical `0/1`.

## Validation

- `cargo fmt --all --check`: PASS.
- `cargo test --workspace --offline`: PASS, including all legacy modular,
  sieve, primality/factorization, u128, quadratic, Desktop, and documentation
  tests.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: PASS with
  the installed `rustc` path made explicit for the Tauri build script.
- Standalone Web tests and Clippy: PASS (13 adapter tests).
- WASM release build with wasm-bindgen 0.2.127: PASS.
- JavaScript syntax check: PASS.
- Browser smoke: `next_prime(100) = 101`, completion timing visible, no console
  warning/error.

The new independent integration suite checks small-domain arithmetic functions
against brute force, divisor invariants, prime navigation through 10,000 and
near u64 max, valuation boundaries, complete-u64 Bezout identities, and a
deterministic signed reconstruction corpus.

## Focused benchmark highlights

One local optimized Windows x86_64 run reported: extended GCD 131.50 ns/op,
next prime 979.05 ns/op, previous prime 2,561.17 ns/op, rational reconstruction
53.62 ns/op, and explicit enumeration of 103,680 divisors 1,378,565.90 ns/op.
The preserved factorization benchmark also completed. These are directional
local measurements, not performance contracts.

## Deliberate limitations

- u64 is the supported domain for these primitives.
- Valuation does not accept composite bases.
- Divisor enumeration materializes only when explicitly requested.
- Reconstruction returns no heuristic approximation and uses no floating
  point.
- No generic arithmetic-function framework, prime table, server, async layer,
  or parallel execution was introduced.
- Experimental modules already present in the interrupted worktree were
  preserved but were not treated as v0.6 completion evidence.
