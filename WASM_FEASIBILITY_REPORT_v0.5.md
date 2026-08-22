# SwissMath v0.5 — WebAssembly feasibility audit

Audit date: 2026-08-22. This is an audit artifact, not a SwissMath release and not a web application.

## Conclusion

**YELLOW-GREEN — WASM FEASIBLE WITH MINOR TARGET CONFIGURATION**

The mathematical core can be reused in a browser through a thin `wasm-bindgen` adapter. The untouched core has no filesystem, networking, threading, Windows API, FFI, architecture-specific intrinsic, or `unsafe` requirement. The only blocker in the direct baseline is the browser configuration of a transitive `getrandom` dependency.

## Environment

| Item | Result |
| --- | --- |
| Rust | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| Cargo | `cargo 1.97.0 (c980f4866 2026-06-30)` |
| Target | `wasm32-unknown-unknown` already installed |
| `wasm-bindgen` CLI | `0.2.127` |
| adapter crate | `wasm-bindgen 0.2.126` (the locally cached `js-sys 0.3.103` requires it) |
| `wasm-pack` | not installed; not required |
| Node.js | not installed; actual browser smoke was used instead |

This audit predates the public Git repository and did not depend on Git metadata. Relevant v0.5 manifests and source were inspected before the experiment.

## Static portability audit

The production files under `crates/core/src` contain no `std::fs`, `std::net`, `std::process`, `std::thread`, filesystem path, environment, Windows API, FFI, architecture-specific intrinsic, or OS entropy use. The crate explicitly has `#![forbid(unsafe_code)]`. `std::error::Error`, `Vec`, and ordinary allocation are used and compiled successfully for WASM.

Tauri/Windows code exists only under `apps/desktop/src-tauri` and was not part of the adapter dependency path. Matches in tests/benches were treated separately.

## Dependency audit

The resolved production chain is:

```text
swissmath-core 0.5.0
├─ num-bigint 0.4.8 (std)
│  └─ rand 0.8.7 → rand_core 0.6.4 → getrandom 0.2.17
└─ num-prime 0.5.0 (big-int)
   └─ num-bigint/rand → rand 0.8.7 → rand_core → getrandom 0.2.17
```

`rand` and `getrandom` are not used directly by the SwissMath production source; they are enabled transitively by the current `num-prime`/`num-bigint` feature set. On `wasm32-unknown-unknown`, `getrandom 0.2.17` emits its documented compile error unless the `js` feature is enabled. The temporary adapter enabled only this target-specific feature:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2.17", features = ["js"] }
```

That activates `getrandom`'s `wasm-bindgen`/`js-sys` browser backend. No RNG or mathematical implementation was added.

## Untouched-core baseline

Command:

```text
cargo check -p swissmath-core --target wasm32-unknown-unknown
```

Result: **FAIL**, before any adapter or configuration change. First meaningful error:

```text
error: the wasm*-unknown-unknown targets are not supported by default,
you may need to enable the "js" feature
--> getrandom-0.2.17/src/lib.rs
```

This is a target configuration issue, not an incompatible SwissMath algorithm or OS API.

## Temporary adapter

The temporary adapter source was retained only during the audit and is not part
of the public release tree:

- `Cargo.toml`: standalone workspace, path dependency on `swissmath-core`, target-only `getrandom/js` feature;
- `src/lib.rs`: three `wasm-bindgen` string/result wrappers only.

The adapter compiled successfully with:

```text
cargo check --offline --target wasm32-unknown-unknown
cargo build --offline --release --target wasm32-unknown-unknown
```

It does not duplicate primality, quadratic-root, or sieve mathematics.

## Browser package and runtime smoke

`wasm-bindgen --target web` produced a browser-loadable package in the temporary `pkg-web/` directory. A minimal static HTML module loaded it from `127.0.0.1` and executed in the available browser; no framework, bundler, Node.js, or browser automation package was installed.

| Smoke call | Expected / observed result | Rough elapsed time |
| --- | --- | ---: |
| primality(`97`) | `PrimeExact` | 6.7 ms |
| primality(`360`) | `Composite` | 0.1 ms |
| primality(`39614081257132185645928677377`) | `PrimeExact` (u128) | 5.4 ms |
| primality(M521) | `ProbablePrime` (>u128 BPSW) | 37.1 ms |
| quadratic `x² = 10 (mod 13)` | `[6, 7]` | 0.3 ms |
| fixed shared-modulus sieve | `count=18`, preview `2,3,14,15,…,98,99` | 0.4 ms |

All six browser assertions passed. Timings are single-call, rough browser measurements intended only to detect catastrophic slowdown; they are not benchmarks. The existing native v0.5 benchmark context is recorded separately in `BENCHMARKS.md` (for example, 3.096 ms for the native u128 proof-friendly prime and 15.352 ms for native M521 BPSW), but environments are not directly comparable.

## Artifact size

Release browser artifacts (before any size optimization):

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `swissmath_wasm_audit_smoke_bg.wasm` | 269,603 | `5704BA77F3CAC933B050B8B80685D3EB983FF813A414942ECCCCA99C5E834FD8` |
| `swissmath_wasm_audit_smoke.js` | 12,326 | `AD3325A0F8275C998D9516CDE9070DF00061E49D719DC1F14271E50A14CADE67` |

## Regression and scope guard

The production workspace remained v0.5.0 and its algorithms, Tauri adapter, and release packaging were not modified for this audit. Final checks passed:

```text
cargo fmt --all --check                         PASS
cargo test --offline --workspace                PASS
cargo clippy --offline --workspace --all-targets -- -D warnings   PASS
```

The generated WASM build directory and package are disposable audit outputs and are not part of a release ZIP.

## Minimum next step

Before a real SwissMath Web implementation, keep the browser adapter thin and add the same target-specific `getrandom/js` configuration in the production WASM adapter (with compatible `wasm-bindgen`/`js-sys` versions). Then add a dedicated web package only in the next phase. No mathematical algorithm, desktop code, or v0.5 release artifact needs to change for feasibility.
