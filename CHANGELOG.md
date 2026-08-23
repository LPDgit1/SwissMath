# Changelog

## 0.7.0 — Finite-Field Computation

- Hardened exact rational linear algebra, polynomial helpers, Berlekamp–Massey,
  and PSLQ with deterministic independent invariants before extending Core.
- Added a small exact prime-field context over the full `u64` prime domain;
  composite moduli are rejected and values remain canonical residues.
- Added dense row-major matrix arithmetic, determinant, rank, RREF, solve,
  inverse, and kernel over Fp using one shared Gauss–Jordan elimination path.
- Added canonical dense polynomial arithmetic, division, monic GCD, extended
  GCD, derivative, evaluation, and modular power over Fp.
- Added exhaustive small-field tests, required F5 smoke cases, dedicated
  16/32/64/128 matrix and 16/64/256/512 polynomial benchmarks, CLI families,
  and a progressive Web v0.3 “Finite fields” category.
- Kept extension fields, polynomial factorization, discrete logarithms, full
  Smith transforms, and generic algebra infrastructure out of scope.

## Research Workflow 0.1

- Added a thin native `swissmath` CLI over Core v0.6 with curated scalar
  commands, compact human output, stable JSON, streaming JSONL, and CSV column
  preservation.
- Batch record errors are emitted structurally without stopping later records;
  invocation errors remain on stderr.
- Added automatic multiline Web batches, a simple result table, Copy/JSON/CSV
  export, reproducible CLI commands, and bounded hash-based share links that do
  not auto-execute.
- Added CLI process tests, Web workflow smoke coverage, cross-surface parity
  checks, and startup/batch performance measurements.
- Kept the Workflow single-threaded, local, configuration-free, and without a
  server, database, account, telemetry, async runtime, or frontend framework.

## 0.6.0 — Research Primitives

- Recovered the interrupted v0.6 implementation without replacing the
  existing primality or factorization architecture.
- Added full-domain u64 extended GCD, explicit p-adic valuation of zero,
  Möbius, radical, squarefree, divisor count/sum/enumeration, and strict next/
  previous-prime navigation.
- Reused each `Factorization` for all derived scalar functions; full divisor
  enumeration remains an explicit request.
- Added default and separately bounded exact rational reconstruction with
  integer-only uniqueness bounds and mandatory congruence verification.
- Added independent small-domain references, u64 boundary cases, a
  deterministic reconstruction corpus, and focused benchmarks.

## 0.5.0 — Exact-first u128 Primality

- Added a bounded exact-first u128 primality route after the existing exact u64
  path: cheap rejection, exact factors of `n - 1`, and bounded Pocklington;
  unfinished proofs return `ExactProofIncomplete` rather than a probable-prime
  claim.
- Kept values above u128 on the existing BigUint/Baillie–PSW probable-prime
  path and preserved all u64 factorization, φ, λ, order, sieve, and quadratic
  domains unchanged.
- Corrected 0 and 1 to `Neither` for primality assessment while retaining
  `1 → Unità` for integer analysis.
- Removed the desktop adapter's duplicate Jacobi/Legendre calculation and
  added routing, theorem-condition, GUI, and benchmark coverage.
- Preserved the preflight invariants and moved the source bundle to
  `release/SwissMath-v0.5-source.zip`.

## 0.4.0 — Large Primality & Quadratic Arithmetic

- Made `Factorization` self-describing with `n()`, method-based φ/λ, and no
  caller-supplied duplicate integer.
- Removed verified redundant small-prime/Pollard branches and added a fixed
  balanced-semiprime robustness corpus.
- Added decimal routing to exact deterministic u64 primality or explicit
  BigUint Baillie–PSW probable-primality assessment through `num-prime`.
- Added Jacobi and Legendre symbols, prime square roots with the direct
  p % 4 = 3 path and Tonelli–Shanks, Hensel lifting, powers of two, and CRT
  roots for composite unit moduli.
- Added exhaustive quadratic tests, deterministic BPSW/quadratic benchmarks,
  and the `Residui quadratici` GUI screen with bounded root previews.
- Updated the integer screen to keep factorization/φ/λ unavailable rather than
  erroneous for values above u64, while labelling probable primes honestly.
- Preserved the v0.3 preflight invariants and moved the portable source bundle
  to `release/SwissMath-v0.4-source.zip`.

## 0.3.0 — Prime & Factor

- Added exact deterministic Miller–Rabin primality testing over the complete
  `u64` domain.
- Added deterministic Pollard–Brent factorization with bounded retries,
  batched GCD, canonical prime powers, and explicit zero/search errors.
- Added Euler φ, Carmichael λ, and multiplicative order without duplicate
  factorization in the integer-analysis workflow.
- Added the `Numeri interi` GUI screen with one-call analysis and order lookup;
  all large values remain decimal strings over IPC.
- Corrected excluded-filter complement construction to use `complement_assign()`
  and extended sieve benchmarks with build-plus-search measurements.
- Corrected source ZIP entries to use portable `/` separators and added the v0.3
  implementation report and source bundle.

## 0.2.0 — Congruence Explorer & Modular Sieve

- Added exact linear-congruence solving with compact `None`/`All`/`Class`
  results, solution counts, and structured explanation facts.
- Added bounded systems of linear congruences by reusing generalized CRT.
- Added `ModularFilter` and `ModularSieve` with allowed, excluded, and reduced
  linear-congruence filters, same-modulus normalization, selective anchoring,
  inclusive ranges, `u128` counts, and bounded previews.
- Added exhaustive small-domain, boundary, system, and deterministic sieve
  reference tests, plus focused sieve benchmarks.
- Extended the offline Tauri GUI with `Congruenze` and `Filtro modulare` tabs;
  all new IPC values remain decimal strings.
- Added a preflight result bar with UTF-8 `.txt` save, native print/PDF, and
  measured elapsed time in both the result context and completion notice.
- Added the reproducible PowerShell source-packaging script and v0.2
  implementation report.

## 0.1.0

- Initial exact modular arithmetic, generalized CRT, residue-set primitives,
  Micro-ModSieve harness, and offline Tauri desktop GUI.
