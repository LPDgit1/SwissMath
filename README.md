# SwissMath

SwissMath is a lightweight computational mathematics toolkit written in Rust,
focused on fast exact modular arithmetic and computational number theory, with
desktop, command-line, and WebAssembly interfaces.

By Luca Pezzullo, 2026

Current source status: SwissMath Core v0.6, Web v0.2, and Research Workflow
v0.1. The public deployment may lag the validated local source release.

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
- Core v0.6 research primitives: extended GCD, p-adic valuation, Möbius,
  radical, squarefree test, divisor count/sum/enumeration, prime navigation,
  and exact modular rational reconstruction;
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

## Why SwissMath?

- **Web** provides zero-install interactive calculations in the browser.
- **CLI** provides fast automation, streaming batch work, and JSON/JSONL/CSV
  interoperability.
- **Desktop** provides a local graphical Windows application.
- Every surface calls the same Rust Core and preserves explicit exact,
  probable, and proof-incomplete semantics.
- Calculations require no account, API key, server, telemetry, or network.

## Applications

- **Desktop** — offline Tauri 2 GUI for Windows.
- **Web** — Rust/WASM client-side application for modern browsers.
- **CLI** — small native binary for scripts, stdin/stdout, JSONL, and CSV.

All surfaces reuse the same `swissmath-core` implementation.

### Core v0.6 reconstruction semantics

`rational_reconstruct(r, m)` uses the exact integer bound
`floor(sqrt((m - 1) / 2))` for numerator magnitude and denominator. This
ensures the conventional `2AB < m` uniqueness condition. The bounded API
accepts distinct numerator and denominator limits. A successful result is
reduced, has a positive denominator, satisfies every bound, and is verified
against `a ≡ r·b (mod m)` before it is returned. No-result and invalid-parameter
outcomes remain distinct.

```text
                  swissmath-core
                  /       |        \
                 ↓        ↓         ↓
             Desktop     CLI       WASM
                                     ↓
                                   Browser
```

## Project layout

- `crates/core` — the reusable mathematical library, tests, and benchmarks;
- `apps/desktop` — Tauri GUI and desktop adapter;
- `apps/web` — thin `wasm-bindgen` adapter and browser UI;
- `apps/cli` — thin native research-workflow CLI;
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

## Build and use the CLI

```powershell
cargo build -p swissmath-cli --release
target\release\swissmath.exe prime 1000000007
target\release\swissmath.exe factor 360 --json
```

Streaming PowerShell example:

```powershell
Get-Content numbers.txt | target\release\swissmath.exe prime --jsonl
```

Equivalent Unix example:

```sh
cat numbers.txt | ./target/release/swissmath prime --jsonl
```

CSV preserves existing columns and appends stable SwissMath result columns:

```powershell
target\release\swissmath.exe prime --input numbers.csv --column n --output results.csv
```

A Python package is not required; subprocess and JSONL provide direct
interoperability:

```python
import json, subprocess

process = subprocess.Popen(
    ["swissmath", "prime", "--jsonl"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
)
stdout, _ = process.communicate("97\n99\n101\n")
records = [json.loads(line) for line in stdout.splitlines()]
```

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
cargo test -p swissmath-cli
cargo fmt --manifest-path apps/web/Cargo.toml --all --check
cargo test --manifest-path apps/web/Cargo.toml
cargo clippy --manifest-path apps/web/Cargo.toml --all-targets -- -D warnings
```

The CLI and Web adapter keep mathematical logic in `swissmath-core`. The Web
GUI reports elapsed time, accepts newline-separated scalar batches, exports
JSON/CSV, copies reproducible CLI commands, creates small hash-based share
links, and retains local result saving and print/PDF output.

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
