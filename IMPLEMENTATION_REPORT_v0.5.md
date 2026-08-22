# SwissMath v0.5 — Implementation report

## Scope

SwissMath v0.5 is a small incremental release over the validated v0.4 tree.
It adds only a bounded exact-first primality path for `u128`, corrects the
primality semantics of 0 and 1, and removes duplicate Jacobi/Legendre work in
the desktop adapter. The existing u64 arithmetic, factorization, sieve,
quadratic operations, GUI preflight, and packaging behavior remain in place.

## Preflight and routing

- `ResidueSet` excluded filters still use the owned in-place complement path.
- Sieve benchmarks still distinguish prepared search from build + normalization
  + search.
- `Factorization` remains self-describing (`n()`, factors, φ, λ methods).
- The deterministic difficult-semiprime Pollard corpus remains unchanged and
  the redundant second small-prime stripping pass remains absent.
- The source archive is single-root, uses portable `/` entries, and excludes
  build artifacts.
- Decimal routing is Rust-owned: `u64` → exact deterministic path; `u128` →
  bounded exact-first path; values above `u128` → existing BigUint/BPSW path.

## Primality semantics

`PrimalityAssessment` now includes:

- `Neither` for 0 and 1;
- `Composite`;
- `PrimeExact` for a completed exact proof;
- `ExactProofIncomplete` for a bounded u128 proof that cannot complete;
- `ProbablePrime` only for values above `u128` that pass BPSW.

Integer analysis retains `1 → Unità`; factorization, φ, λ, and order remain
available only in the existing u64 analysis domain.

## Exact-first u128 proof

The private wide-primality path first performs small-prime rejection and uses
BigUint/BPSW only as a one-way composite filter. It then factors only `n - 1`:

1. strip the existing small-prime table and accumulate complete prime powers in
   `F`;
2. if the residual fits u64, reuse the existing exact factorizer;
3. otherwise recurse only on the whole residual cofactor, with depth bounded by
   8;
4. stop factor acquisition as soon as `F > n / F`;
5. verify Pocklington for every distinct known prime factor of `F`, using
   BigUint `modpow` and deterministic witnesses `2..=64`.

No u128 factorizer, U256 arithmetic, certificate API, randomness, scheduler,
or proof subsystem was added. An incomplete proof is an intentional result.

The desktop adapter now computes Jacobi once and reuses it as the Legendre
value after the existing exact prime check, without changing the public
`legendre_symbol()` API.

## Verification

- Core and desktop tests cover 0/1 semantics, u64 routing, u128 boundaries,
  fixed proven u128 primes, composites, an inconclusive M127 proof, direct
  Pocklington invariants, the existing Pollard corpus, and quadratic regressions.
- `cargo fmt --all --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` pass.
- JavaScript syntax validation and a desktop smoke test cover u64 analysis,
  u128 exact/composite/incomplete outcomes, >u128 probable/composite outcomes,
  PDF/save preflight, and existing quadratic examples.

## Benchmarks

`crates/core/benches/large_primality.rs` reports separate deterministic cases
for a cheap u128 composite, a proof-friendly u128 prime, an incomplete u128
proof, and representative >u128 BPSW inputs. Existing `baseline`, `sieve`,
`number_theory`, and `quadratic` harnesses remain separate. Measurements are
recorded in `BENCHMARKS.md` and are directional; no timing threshold is used.

## Deliberate limitations

- Exact u128 proof can be inconclusive.
- SwissMath does not factor general u128 or BigUint values.
- Values above u128 remain probable-prime assessments rather than formal
  certificates.
- The rest of the arithmetic core remains u64-only.

## Release artifacts

- Windows standalone: `release/SwissMath-v0.5-win-x64/SwissMath-0.5.0-x64.exe`
  SHA-256 `0B349818628A38DF660723AC284FEA6075192399B0B6F11DF3167100DA51B441`.
- Windows installer: `release/SwissMath-v0.5-win-x64/SwissMath_0.5.0_x64-setup.exe`
  SHA-256 `21B0DF14AB29B27D56972C294B9805A5B6ACDC786AB2C0507D3BD7171751219F`.
- Source bundle: `release/SwissMath-v0.5-source.zip`, single root,
  portable separators, and no forbidden artifacts. Its final SHA-256 is
  recorded in the handoff response rather than inside the self-containing
  archive.
