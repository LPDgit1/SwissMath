# SwissMath Core v0.10 Implementation Report

## Phase A — v0.9 preflight

The old `MAX_COMBINATORIAL_PRODUCT_STEPS` value was 1,000,000. Measurements
used the production `factorial_mod_prime` and `binomial_mod_prime` loops, not a
synthetic multiplication benchmark.

Release-native timings (factorial / binomial) were:

| Product steps | Native time |
|---:|---:|
| 1,000,000 | 6.71 / 7.02 ms |
| 5,000,000 | 35.06 / 35.86 ms |
| 10,000,000 | 72.16 / 74.64 ms |
| 25,000,000 | 171.36 / 181.23 ms |
| 50,000,000 | 350.09 / 368.86 ms |
| 100,000,000 | 752.87 / 732.13 ms |
| 200,000,000 | 1.463 / 1.494 s |

Production browser/WASM timings were:

| Product steps | Browser time |
|---:|---:|
| 1,000,000 | 21.5 / 12.7 ms |
| 5,000,000 | 39.4 / 42.7 ms |
| 10,000,000 | 71.9 / 75.7 ms |
| 25,000,000 | 176.5 / 181.9 ms |
| 50,000,000 | 380.7 / 410.5 ms |
| 100,000,000 | 2.81 s / 833.5 ms |
| 200,000,000 | 1.42 s / 7.41 s |

The 100-million factorial result includes one observed browser outlier; normal
near-limit calls were subsecond, with a measured peak of 2.81 seconds. The
200-million row showed excessive variance, including 7.41 seconds and a later
timed-out repeat. The fixed limit is therefore 100,000,000 product steps. Work
is still estimated before the loop, and mathematical zero/shortcut results
remain ahead of the refusal check. The Web yields one animation frame so that
`Computing…` paints before synchronous WASM execution.

`binomial_valuation(n,k,p)` now returns the existing `Valuation::Infinite` when
`k > n`, consistently with `C(n,k)=0` and `v_p(0)=infinity`; no new valuation
abstraction was introduced. Phase A passed formatter, workspace tests, Clippy
with warnings denied, release Core/CLI/Desktop builds, Web/WASM production
build, JavaScript checks, browser smoke, and CLI tests before Phase B began.

## Phase B — Core v0.10

### Core architecture and mathematics

`multimodular.rs` contains one flat `MultimodularAccumulator`. It stores the
combined modulus as `BigUint`, one canonical `BigUint` per coordinate, and the
prime-block count. Scalar, vector, and matrix adapters differ only in shape;
the Core algorithm is shared.

For a new prime `p`, the accumulator computes `M mod p` and its inverse once
outside the coordinate loop. Each coordinate then applies
`t=(r-(x mod p))*(M mod p)^-1 mod p` and `x=x+M*t`; only after all coordinates
does it set `M=M*p`. Distinct `PrimeField` moduli and coordinate counts are
validated. The source residue block is not retained.

`BigUint` is local to non-negative combined moduli/residues. `BigInt` is used
only for signed centered integers and rational numerators. No `num-rational`
dependency or general arbitrary-precision rational arithmetic was added.

The Core exposes:

- canonical CRT residues modulo the combined modulus;
- centered representatives in `[-floor(M/2), floor(M/2)]`, without claiming
  recovery of an unknown source integer;
- bounded integers with the strict uniqueness condition `2B < M`;
- automatic and explicitly bounded rational reconstruction, with `2AB < M`;
- a reduced positive-denominator `BigRationalReconstruction` result.

Every integer candidate is checked modulo `M`. Every rational candidate is
reduced, bound-checked, denominator-checked, and verified by
`numerator = residue * denominator (mod M)`. Failed coordinates report their
flat index. Tests cover products beyond `u128::MAX` and verify the CRT result
against every original prime congruence.

### Input, CLI, and Web workflow

CLI v0.5 accepts:

```text
swissmath reconstruct multi <crt|integer|rational> <scalar|vector|matrix> [bounds]
```

Input may be stdin or `--input`. Human blocks use `mod <prime>` headers and may
contain signed residues. JSONL uses one block per line with decimal-string
`modulus`, `shape`, and flat `values`. JSONL is parsed block-by-block and the
source block is discarded after it enters the accumulator. Output supports
human text, JSON, JSONL, and CSV; all arbitrary-size values remain decimal
strings.

Web v0.6 adds one tool under Fractions and reconstruction, with shape/mode
selectors, human/JSONL paste input, optional rigorous bounds, and local
`.txt`/`.jsonl` loading. Files are read in the browser and never uploaded. Full
results stay available for JSON/JSONL/CSV export, while the DOM renders only a
bounded vector/matrix preview. Oversized share links remain disabled; Copy
Command produces a file-redirection workflow instead of embedding data.

Web and CLI parity cases cover scalar CRT, bounded integer vectors, and bounded
rational matrices. Matrix JSON, JSONL, CSV, and displayed values use the same
shape and decimal strings. A deterministic 10,000-coordinate CLI JSONL test
checks block streaming, reconstruction, and CSV output.

### Performance

The release benchmark records coordinates, prime count, combined-modulus bits,
elapsed time, and coordinates/second. Selected results on the validation
machine:

| Operation | Coordinates | Primes | Bits | Time | Coordinates/s |
|---|---:|---:|---:|---:|---:|
| incremental CRT | 10,000 | 8 | 81 | 3.568 ms | 2,802,848 |
| incremental CRT | 100,000 | 8 | 81 | 36.918 ms | 2,708,735 |
| incremental CRT | 100,000 | 32 | 324 | 271.923 ms | 367,751 |
| bounded integer | 100,000 | 8 | 81 | 50.357 ms | 1,985,817 |
| bounded rational | 10,000 | 8 | 81 | 12.504 ms | 799,718 |
| 100x100 matrix CRT | 10,000 | 8 | 81 | 4.803 ms | 2,081,945 |

The complete benchmark includes scalar and 100/1,000/10,000/100,000-coordinate
CRT cases, 2/4/8/16/32 prime blocks, integer and rational reconstruction, and a
100x100 matrix workload.

### Validation and deliberate exclusions

The final gate covers formatter checks, workspace and standalone-Web tests,
workspace and standalone-Web Clippy with warnings denied, JavaScript syntax,
release Core/CLI/Desktop builds, production Web/WASM build, CLI integration,
and browser smoke for scalar/vector/matrix workflows and bounded preview.

v0.10 deliberately adds none of the following: generic reconstruction
frameworks; automatic prime selection; automatic rerunning of modular
computations; stabilization heuristics; bad-prime or fault-tolerant CRT;
generic tensors; threads, Rayon, or Web Workers; server/database services; a
new frontend framework; a new serialization protocol; or a general
`BigRational` dependency.
