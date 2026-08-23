# SwissMath Core v0.8 Implementation Report

## Phase A — finite-field preflight

`FpMatrix` and `FpPolynomial` now retain their `PrimeField`. Unary operations
use that stored field, while binary operations reject different moduli with
`FiniteFieldError::FieldMismatch`; no coercion or invariant-breaking public
constructor was added. Core, CLI, WASM, tests, and benchmarks were migrated to
the self-describing API. Same-field v0.7 results remain covered by the complete
regression suite.

The finite-field baseline now covers matrix RREF, rank, determinant, and
multiplication, plus polynomial multiplication, GCD, and modular power. On the
validation machine, representative 64x64 matrix operations took 0.23–0.41 ms;
degree-256 polynomial GCD and powmod took 0.010 ms and 24.62 ms respectively.
These are local measurements without acceptance thresholds.

## Phase B — sequences and multiplicative structure

Recurrences use `a_m = sum(c_j * a_(m-j-1))` over `PrimeField`. The nth-term
implementation computes `x^n` modulo the recurrence polynomial with Kitamasa-
style binary exponentiation in O(k² log n) time and O(k) result storage. The
existing Berlekamp–Massey implementation now supports p=2 and feeds a convenience
workflow whose result is labelled `inferred_recurrence`; it asserts only that
the model fits the supplied prefix.

Primitive-root checking factors p-1 once and tests distinct prime divisors;
search is deterministic and returns the smallest positive generator. Discrete
logarithms first compute the exact order of g and test subgroup membership, then
use Pohlig–Hellman prime-power digit lifting, bounded BSGS, and existing CRT.
The BSGS table is capped at 100,000 baby steps and the cap is checked before
allocation. Public outcomes are exact `Solved`, exact `NoSolution`, or
`SearchLimitReached`; every returned logarithm is verified.

Tests cover recurrence orders 1, 2, and higher, p=2, zero and error cases,
iterative small-index oracles, and n=10^18 against independent Fibonacci fast
doubling. Primitive roots are checked exhaustively over small primes; discrete
logs are cross-checked against brute force, including subgroup failure and a
forced pre-allocation limit. CLI and Web adapter tests share recurrence (55 mod
101), primitive-root (3 mod 17), and discrete-log (x=17 mod 97) smoke values.

Local v0.8 benchmark results (ms/op) were: recurrence n=10^18 at k=2/8/32/64,
0.0032/0.0192/0.2879/1.1375; primitive-root search at p=65537/1000000007/
20000000687, 0.0005/0.0016/0.0026; DLP smooth/medium/early-limit,
0.0051/0.3227/0.0055. No timing threshold is imposed.

CLI v0.3 and Web v0.4 expose supplied and inferred recurrences, primitive-root
search/checking, and discrete logarithms with JSON, copy-command, and share-link
integration. No generic group or sequence framework, Pollard-rho DLP, GF(p^k),
polynomial factorization, Web Worker, or new frontend framework was introduced.
