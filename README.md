# SwissMath

SwissMath is a lightweight computational mathematics toolkit written in Rust,
focused on fast exact modular arithmetic and computational number theory, with
desktop and WebAssembly frontends.

Author: Luca Pezzullo, 2026

## Try it online

[Open SwissMath Web](https://swissmath.lucapezzullo.chatgpt.site)

The public application is SwissMath Web v0.1, powered by SwissMath Core v0.5.
Calculations run locally in the browser through WebAssembly; no application
backend or telemetry is required.

## What it does

- modular arithmetic and generalized CRT;
- materialized residue sets;
- linear congruences and systems;
- modular sieve over finite ranges;
- exact u64 primality;
- bounded exact-first u128 primality;
- probable primality above u128;
- u64 factorization, Euler φ, Carmichael λ, and multiplicative order;
- Jacobi and Legendre symbols;
- modular roots in the currently supported domains.

Primality labels are deliberately precise:

```text
≤ u64:              exact deterministic result
> u64 and ≤ u128:   bounded exact-first proof
                    exact prime / composite / proof incomplete
> u128:             BPSW assessment
                    composite / probable prime
```

## Applications

- **Desktop** — offline Tauri 2 GUI for Windows.
- **Web** — Rust/WASM client-side application for modern browsers.

Both frontends reuse the same `swissmath-core` implementation.

```text
                  swissmath-core
                  /            \
                 ↓              ↓
           Tauri adapter    WASM adapter
                 ↓              ↓
             Desktop           Browser
```

## Project layout

- `crates/core` — the reusable mathematical library, tests, and benchmarks;
- `apps/desktop` — Tauri GUI and desktop adapter;
- `apps/web` — thin `wasm-bindgen` adapter and browser UI;
- `bench/problems` — repeatable benchmark harnesses;
- `scripts` — Web build and source-bundle tooling;
- `CHANGELOG.md`, `NOT_NOW.md` — release history and explicit scope boundary;
- `WASM_FEASIBILITY_REPORT_v0.5.md` — technical feasibility evidence;
- `WEB_IMPLEMENTATION_REPORT_v0.1.md` — Web implementation and deployment report.

Generated `target/`, `dist/`, `release/`, and `work/` content is intentionally
excluded from the public source repository.

## Build the Web application

Prerequisites:

- stable Rust;
- the `wasm32-unknown-unknown` target;
- a compatible `wasm-bindgen-cli` (the validated environment used CLI 0.2.127
  with the `wasm-bindgen` 0.2.126 crate).

From the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-web.ps1
```

Use `-Offline` when all Rust dependencies and tools are already cached. The
script accepts optional `-CargoPath` and `-WasmBindgenPath` overrides and fails
clearly when either command is not available; it never guesses a user home
directory or installs tools.

The validated browser bundle is written to `dist/web/`.

## Build the desktop application

With Rust and `cargo-tauri` available:

```powershell
cargo check -p swissmath-desktop
cargo tauri build --no-bundle
cargo tauri build --bundles nsis
```

Windows WebView2 is required by Tauri. SwissMath itself has no networking,
database, telemetry, remote font, or remote-service dependency.

## Validation

```text
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --manifest-path apps/web/Cargo.toml --all --check
cargo test --manifest-path apps/web/Cargo.toml
cargo clippy --manifest-path apps/web/Cargo.toml --all-targets -- -D warnings
```

The Web adapter keeps mathematical logic in `swissmath-core`; its JSON/WASM
entrypoints are only a browser transport layer. The GUI also reports elapsed
calculation time and supports local result saving and print/PDF output.

## Source bundles

`scripts/package-source.ps1` creates a reproducible single-root ZIP. The
optional `-BundleName` parameter can produce, for example:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package-source.ps1
powershell -ExecutionPolicy Bypass -File scripts/package-source.ps1 -BundleName SwissMath-Web-v0.1-source
```

Archives exclude build output, local work folders, binaries, `.git`, and other
generated release artifacts.

## License

SwissMath is released under the [MIT License](LICENSE).
