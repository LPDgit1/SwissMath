# SwissMath Research Workflow v0.1 Implementation Report

Date: 2026-08-23. Core prerequisite: validated SwissMath Core v0.6.

## Architecture

`swissmath-core` remains the single mathematical implementation. Desktop, the
new native CLI, and the thin WASM adapter call it directly. The browser remains
plain HTML/CSS/JavaScript. No service, workflow engine, command bus, RPC layer,
server, database, account, or configuration subsystem was added.

## CLI

`apps/cli` builds the `swissmath` binary. Curated commands include prime,
factor, analyze, gcd/xgcd, inverse, linear congruence, next/previous prime,
rational reconstruction, modular square roots, valuation, Möbius, radical,
squarefree, divisor count/sum, and explicit divisor enumeration.

Human output is compact. `--json` emits a stable object with operation, input,
status, result, exactness, Core version, and elapsed milliseconds. `--jsonl`
reads stdin incrementally and writes one object per non-empty input line. A bad
record produces a structured error and later records continue; stderr is kept
for invocation-level failures.

CSV accepts a scalar command plus `--input`, `--column`, and optional
`--output`. Existing columns are retained and four result columns are appended.
The local environment could not acquire the optional `csv` crate because its
Windows TLS provider had no credentials, so the final dependency-minimal CLI
uses a confined streaming CSV reader/writer supporting UTF-8, quoted commas,
doubled quotes, CRLF, and quoted multiline fields. It adds no runtime
dependency beyond the already-used serde/serde_json stack.

## Web workflow

Naturally scalar tools accept one value or newline-separated values. Batch
results use a four-column table: input, result, status, and exactness. Current
results can be copied as text or JSON and downloaded as JSON or CSV. Tools with
direct CLI equivalents expose Copy Command. Small scalar inputs can be encoded
in a URL fragment; batch/oversized state disables Share, and restored links do
not auto-execute.

Large integers stay decimal strings across the WASM boundary. JavaScript only
collects UI state and calls Rust; it does not duplicate algorithms.

## Tests and parity

- CLI process tests cover prime 97, factor 360, analyze, a v0.6 primitive,
  reconstruction, JSON, JSONL streaming, malformed-record continuation, and
  CSV preservation.
- Web adapter tests include previous-prime, radical, and valuation v0.6 calls.
- Production browser smoke covered multiline 97/99/101, result table,
  Copy JSON, CSV generation, Copy Command, share creation/restoration, and no
  automatic execution after restore. Browser console errors/warnings: none.
- Copy Command produced `swissmath next-prime 100`; Web and CLI both returned
  101.
- Web batch 97/99/101 classified prime/composite/prime; CLI JSONL returned the
  same classifications and exactness.
- Desktop tests and release build remained part of the final regression gate.

## Performance

The optimized CLI started in about 41.74 ms in the measured Windows shell and
one scalar prime JSON invocation averaged 28.03 ms. At 10,000 rows, measured
JSONL throughput was about 68,902 prime operations/s and 81,446 factorizations
of 360/s. Small batches are process/pipeline dominated.

The local production Web bundle initialized WASM in about 211.2 ms. Reported
UI compute/render times were 3.5 ms scalar, 9.6 ms for 100 prime rows, and
42.9 ms for 1,000 rows. Execution remains deliberately single-threaded.

## Deliberate non-features

No SDK, REST API, daemon, history, persistence, IndexedDB, telemetry, async
runtime, worker pool, parallel scheduler, plugin system, frontend framework,
configuration file, expression parser, or mathematical DSL was introduced.
No public Site or GitHub push was performed during implementation.
