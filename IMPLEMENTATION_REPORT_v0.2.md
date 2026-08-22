# SwissMath v0.2 implementation report

## Scope delivered

SwissMath v0.2 extends the existing v0.1 architecture without changing the
dependency direction: plain local GUI → small Tauri IPC layer → independently
usable `swissmath-core`. No production dependency, network service, telemetry,
thread, plugin, database, or JavaScript mathematical implementation was added.

## Material changes

- `crates/core/src/congruence.rs`: exact linear-congruence solver, structured
  facts, and systems composed through the existing generalized CRT.
- `crates/core/src/sieve.rs`: `ModularFilter`, direct reduced linear filters,
  same-modulus normalization, and inclusive-range `ModularSieve` results.
- `crates/core/tests/linear_sieve.rs`: exhaustive bounded solver checks,
  system cases, residue-set references, deterministic sieve comparisons, and
  `u64::MAX` range boundaries.
- `crates/core/benches/sieve.rs`: deterministic dense/sparse/multiple/shared
  workloads against a direct reference.
- `apps/desktop/src-tauri/src/lib.rs`: decimal-string IPC commands for single
  equations, systems, and sieve searches.
- `apps/desktop/ui/index.html`, `app.js`, `styles.css`: `Congruenze` and
  `Filtro modulare` form-based screens, with bounded presentation lists.
- `apps/desktop/ui/index.html`, `app.js`, `styles.css`: preflight result
  actions for UTF-8 text export and native print/PDF, with elapsed-time
  reporting in the result bar and completion notice.
- `README.md`, `BENCHMARKS.md`, `CHANGELOG.md`, `NOT_NOW.md`, and this report.
- `scripts/package-source.ps1`: safe staged source ZIP creation and inspection.

## Algorithms and correctness decisions

For `a·x ≡ b (mod m)`, inputs are canonicalized, `d = gcd(a,m)` is computed,
the divisibility condition is checked, and the reduced coprime equation is
solved with the existing modular inverse primitive. The primary result remains
`None`, `All`, or one compact `x ≡ r (mod M)` class; original residues are
generated only for a small GUI preview. A normal solvable equation reports
`d` solutions modulo the original modulus.

Systems solve each equation first, stop at `None`, discard `All`, and pass the
remaining classes to the existing generalized CRT fold. No second CRT or proof
trace framework was introduced.

Filters use materialized `ResidueSet` values only where the v0.2 feature needs
them. A linear congruence is reduced directly (for example, `14x ≡ 8 mod 30`
becomes `mod 15 ∈ {7}`), avoiding the original-modulus `{7,22}` materialization.
Normalization intersects filters sharing a modulus, removes full filters, and
detects empty contradictions. The sieve chooses the sparsest anchor by exact
`u128` cross-products, enumerates anchor blocks in ascending order, then tests
the remaining filters. Counts use `u128`, and block/candidate stepping is safe
at `u64::MAX`.

## Benchmarks

On the deterministic local Windows x86_64 run recorded in `BENCHMARKS.md`, the
sieve was 1.41× faster than direct filtering on a dense case, 45.87× on a
sparse anchor, 1.66× on multiple moduli, and 7.84× after shared-modulus
intersection. These are directional measurements, not hard thresholds; the
simple v0.2 planner was retained.

## Validation

The release validation suite includes `cargo fmt --all --check`, workspace
tests, workspace Clippy with `-D warnings`, desktop command tests, exhaustive
linear cases for `m = 1..64`, deterministic sieve/reference comparisons, and
the focused sieve benchmark. The JavaScript bundle passes the Node syntax check.

## GUI and limitations

The new tabs expose one equation, an editable system table, allowed/excluded
residue rows, linear-congruence rows, inclusive ranges, count, percentage,
anchor, and bounded ascending preview. Large values remain decimal strings over
IPC. Each completed calculation measures the elapsed IPC call and displays it
as milliseconds or seconds. The preflight bar can save the visible result as a
UTF-8 `.txt` snapshot or open the native print dialog; the print stylesheet
includes only the current result, so the dialog can be used with a PDF printer.
The GUI caps materialized allowed/excluded sets at modulus 2,000,000 and caps
preview requests at 1,000; the core remains independently exact but
materialized residue sets can require substantial memory for very large moduli.

Deferred features include factorization/primality, symbolic algebra, sparse
residue backends, global LCM/CRT sieve fusion, wheel compilation, SIMD, GPU,
parallelism, and frontend frameworks; see `NOT_NOW.md`.

## Distribution

- Windows x64 standalone executable and optional NSIS installer:
  `release/SwissMath-v0.2-win-x64/`.
- External-review source bundle:
  `release/SwissMath-v0.2-source.zip`.

The reproducible local artifacts currently have these SHA-256 values:

```text
090FCA4574656076E047E160E900980AB031A8650F0A2809A25C50F7D8E5AD25  SwissMath-0.2.0-x64.exe
D587FDA695B7AEC2EF72B5A95A0D3DD1E525BB53165947427054F1101CE225DA  SwissMath_0.2.0_x64-setup.exe
```
