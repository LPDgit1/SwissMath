# SwissMath Core v0.7 Implementation Report

## Scope and sequencing

Core v0.7 was implemented only after the existing research primitives passed a
dedicated hardening gate. The baseline algorithms were preserved; the hardening
work added deterministic tests for RREF structure and residuals, Bareiss versus
an independent Leibniz determinant, Hermite and Smith invariants, rational
polynomial division/interpolation/GCD identities, generated modular
recurrences, and independently recomputed PSLQ residuals.

The Phase A gate passed:

```text
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --manifest-path apps/web/Cargo.toml
cargo clippy --manifest-path apps/web/Cargo.toml --all-targets -- -D warnings
node --check apps/web/web/app.js
```

## Core architecture

`PrimeField` validates `p` with the existing exact deterministic `is_prime(u64)`
path and delegates scalar modular operations to the existing `ModCtx`. It is a
small concrete context, not a generic trait hierarchy.

`FpMatrix` stores canonical `u64` residues in one dense row-major vector. It
supports addition, subtraction, multiplication, matrix-vector multiplication,
determinant, rank, RREF, solve, inverse, and kernel. RREF, rank, solve, inverse,
and kernel share one internal Gauss–Jordan elimination implementation.

`FpPolynomial` stores ascending canonical coefficients and removes trailing
zeros. It supports addition, subtraction, multiplication, division with
remainder, monic GCD, extended GCD, formal derivative, Horner evaluation, and
binary modular exponentiation.

## Verification

The deterministic suite includes exhaustive 2x2 matrices over F2, F3, and F5,
closed-form determinant comparison, inverse/rank equivalence, rectangular
kernel residuals, all system classifications, polynomial division and Bézout
identities, repeated-multiplication powmod references, and canonical-coefficient
checks over F2, F3, F5, and F7.

Required F5 smoke evidence is encoded in Core, CLI, and Web tests:

- `[[1,2],[3,4]]` has determinant `3`;
- its inverse is `[[3,1],[4,2]]` and the product is the identity;
- singular and rectangular matrices are covered;
- solve reports none, unique, or affine results;
- the formal derivative of `x^5` is zero in characteristic five.

The dedicated `finite_fields` benchmark covers matrix RREF at sizes 16, 32,
64, and 128 and polynomial multiplication at degrees 16, 64, 256, and 512.
On the validation machine, one optimized run measured matrix RREF at 0.0081,
0.0462, 0.4189, and 2.8088 ms/op, and polynomial multiplication at 0.0004,
0.0057, 0.0885, and 0.3528 ms/op respectively. These figures are local
engineering measurements, not cross-machine performance guarantees.

## Interfaces

CLI v0.2 adds `matrix` and `polynomial` command families while retaining the
existing human and JSON record schema, elapsed time, exactness label, stdin,
and file workflows. Web v0.3 adds one “Finite fields” category with a shared
prime input, paste-friendly matrix/coefficient fields, conditional tool forms,
matrix tables, and the existing copy/export/share/print architecture.

The production WASM bundle and native Core, CLI, and Desktop release targets
built successfully. A browser smoke test on the generated `dist/web` bundle
verified sidebar navigation, the tool dropdown, conditional forms, the F5
determinant, inverse table rendering, elapsed-time status, the characteristic-
five derivative, and an empty browser console error log.

`release/SwissMath-v0.7-source.zip` was generated with one root directory. Its
85 entries include README, MIT license, and this report, with no build, release,
temporary, executable, library, debug-symbol, nested ZIP, or Git artifacts.

## Explicit exclusions

This release does not add a generic algebra framework, extension fields
`GF(p^k)`, polynomial factorization, discrete logarithms, a full Smith
`U*A*V` transform, or new sequence-guessing methods.
