# SwissMath

SwissMath is a lightweight computational mathematics toolkit written in Rust,
focused on fast exact modular arithmetic and computational number theory, with
desktop, command-line, and WebAssembly interfaces.

By Luca Pezzullo, 2026

Current source status: SwissMath Core v0.10, Web v0.6, CLI v0.5, and Desktop
v0.5. The public deployment may lag the validated local source release.

## Try it online

[Open SwissMath Web](https://swissmath.lucapezzullo.chatgpt.site)

Calculations run locally in the browser through WebAssembly; no application
backend or telemetry is required. Check the version shown in the application,
because the public deployment may lag the source repository.

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
- exact dense matrix arithmetic over prime fields Fp: addition, subtraction,
  multiplication, matrix-vector products, determinant, rank, RREF, solve,
  inverse, and kernel;
- exact dense polynomials over Fp: arithmetic, division with remainder, monic
  GCD, extended GCD, derivative, evaluation, and modular exponentiation;
- O(k² log n) exact nth-term evaluation for supplied linear recurrences over
  Fp, plus explicitly conditional Berlekamp–Massey extrapolation;
- primitive-root search/checking and bounded exact discrete logarithms over
  Fp with solved, no-solution, and search-limit outcomes;
- factorial/binomial p-adic valuations and bounded exact factorial/binomial
  residues modulo a prime;
- streaming multimodular CRT and verified exact reconstruction of integer or
  rational scalars, vectors, and matrices from distinct prime-residue blocks;
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

### Core v0.9 finite-field and multiplicative domain

Finite-field operations accept only a prime `p` in the exact `u64` domain.
Inputs are reduced to canonical residues `0..p-1`; composite moduli are rejected.
Matrices and polynomials retain their own field and reject cross-field binary
operations. Multiplicative-group tools operate only in Fp*. Discrete logarithms
use Pohlig–Hellman with a bounded baby-step/giant-step subroutine; a search-limit
result means the exact algorithm refused an oversized table before allocation,
not that the mathematical answer is uncertain. This release deliberately does
not implement extension fields, polynomial factorization, or generic algebra or
group frameworks.

An inferred recurrence is the minimal recurrence fitting the supplied finite
prefix over Fp. Extrapolated terms are exact under that inferred model, but do
not prove that an unknown generating process follows it indefinitely.

### Modular combinatorics

Core v0.9 computes `v_p(n!)`, `v_p(C(n,k))`, `C(n,k) mod p`, and `n! mod p`
for `u64` inputs and an exact prime field. Legendre, Kummer, Lucas, symmetry,
and Wilson reduction avoid constructing gigantic factorials or binomial
coefficients. For example, `C(10^18,10^9) mod p` is reduced to base-p digit
work. Difficult mid-digit products are declined before execution when they
exceed the fixed interactive work bound; this is an incomplete bounded
computation, not an approximate answer.

### Multimodular reconstruction

Core v0.10 incrementally combines matching residue blocks over distinct prime
fields. One flat `BigUint` accumulator serves scalars, vectors, and matrices;
source blocks need not remain in memory. A centered result is only the canonical
representative modulo the combined modulus. Supplied integer or rational bounds
add exact uniqueness conditions (`2B < M` or `2AB < M`), and every returned
candidate is verified.

Human input uses a `mod <prime>` header followed by matching rows:

```text
mod 101
100 2
3 97

mod 103
102 2
3 99
```

The Web tool can paste this format or load a local `.txt`/`.jsonl` file without
uploading it. For a streaming research pipeline, each JSONL line is one block:

```json
{"modulus":"101","shape":[2,2],"values":["100","2","3","97"]}
{"modulus":"103","shape":[2,2],"values":["102","2","3","99"]}
```

```powershell
target\release\swissmath.exe reconstruct multi integer matrix 100 --input residues.jsonl --csv --output exact.csv
```

### Rational reconstruction semantics

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
- `scripts`, `tools` — Web build, portable launcher, and source-bundle tooling;
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

For the simplest local use on Windows, double-click
`release/SwissMath-Web-Portable.exe`. The executable contains the complete Web
interface and WASM engine, starts a server bound only to `127.0.0.1`, and opens
the default browser. It does not require installation, Python, Node.js, a build,
or a network connection. Keep its small terminal window open while using the
application and close it when finished.

`Avvia-SwissMath-Web.cmd` selects the portable executable automatically when it
is present and remains a development fallback for an existing `dist/web`
bundle.

Alternatively:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/serve-web.ps1
```

Maintainers can regenerate the portable executable after a Web change with:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build-web-portable.ps1
```

Do not open `dist/web/index.html` directly: browsers block ES modules and WASM
loading from `file://`, leaving the tool menu and sidebar inactive.

## Build and use the CLI

```powershell
cargo build -p swissmath-cli --release
target\release\swissmath.exe prime 1000000007
target\release\swissmath.exe factor 360 --json
target\release\swissmath.exe matrix det 5 "1,2;3,4" --json
target\release\swissmath.exe polynomial derivative 5 "0,0,0,0,0,1" --json
target\release\swissmath.exe recurrence nth 1000000007 "0,1" "1,1" 1000000000000000000 --json
target\release\swissmath.exe group primitive-root 17 --json
target\release\swissmath.exe group dlog 97 5 83 --json
target\release\swissmath.exe comb factorial-valuation 2 1000000000000000000 --json
target\release\swissmath.exe comb binomial-valuation 2 1000000000000000000 1000000000 --json
target\release\swissmath.exe comb binomial-mod 1000003 1000000000000000000 1000000000 --json
target\release\swissmath.exe comb factorial-mod 1000000007 1000000 --json
target\release\swissmath.exe reconstruct multi integer matrix 100 --input residues.jsonl --csv --output exact.csv
```

Streaming PowerShell example:

```powershell
Get-Content numbers.txt | target\release\swissmath.exe prime --jsonl
```

Equivalent Unix example:

```sh
cat numbers.txt | ./target/release/swissmath prime --jsonl
```

Whole matrices can also be supplied through stdin or `--input`:

```powershell
Get-Content matrix.txt | target\release\swissmath.exe matrix rank 5 --json
target\release\swissmath.exe matrix rank 5 --input matrix.txt --json
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
JSON/JSONL/CSV, copies reproducible CLI commands, creates small hash-based share
links, and retains local result saving and print/PDF output.

## Source bundles

`scripts/package-source.ps1` creates a reproducible single-root ZIP. The
optional `-BundleName` parameter can produce, for example:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package-source.ps1
powershell -ExecutionPolicy Bypass -File scripts/package-source.ps1 -BundleName SwissMath-v0.10-source
```

Archives exclude build output, local work folders, binaries, `.git`, and other
generated release artifacts.

## License

SwissMath is released under the [MIT License](LICENSE).
