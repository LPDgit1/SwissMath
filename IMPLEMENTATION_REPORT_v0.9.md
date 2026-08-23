# SwissMath Core v0.9 Implementation Report

## Phase A — v0.8 corrective preflight

The prime-field discrete-log path now factors `p-1` once, derives both
`ord(g)` and its prime-power decomposition by stripping factors with modular
powers, and passes that decomposition directly to Pohlig–Hellman. The former
generic `multiplicative_order` plus `factor(order)` path was removed from DLP.
Primitive-root Web/CLI output no longer refactors `p-1` solely for decorative
metadata.

`SearchLimitReached` now has status `search_limit_reached` and exactness
`bounded_incomplete`; solved and subgroup-no-solution results remain exact.
Supplied recurrences validate every term beyond their order, return an
`InconsistentInitialTerms { index }` error on disagreement, and return the
observed normalized value when `n` lies inside the validated prefix. The full
v0.8 test, Clippy, release-build, WASM, JavaScript, and browser gate passed
before Phase B began.

## Phase B — efficient modular combinatorics

One `combinatorics.rs` module exports four focused APIs over `PrimeField`:
`factorial_valuation`, `binomial_valuation`, `binomial_mod_prime`, and
`factorial_mod_prime`. Legendre evaluates `v_p(n!)` by repeated division;
Kummer counts base-p carries for `v_p(C(n,k))`; Lucas decomposes binomials into
base-p digits and computes each small digit binomial with one denominator
inverse; factorial residues choose the shorter direct or Wilson-complement
product. Valuations are `O(log_p n)`; Lucas adds linear work in the symmetric
digit sizes.

Both modular product paths estimate total sequential work first and return
`ComputationLimitReached { estimated_steps, limit }` before computation when
the fixed 1,000,000-step bound is exceeded. Mathematical zero shortcuts run
before the bound. This status is exposed as `bounded_incomplete`, never as an
approximate or exact completed answer.

Tests cross-check Kummer against independent Legendre differences, Lucas
against Pascal rows for all entries through n=120 over p=2,3,5,7,11,13, and
factorials against direct products and Wilson identities. Huge-u64 identities,
edge cases, and tiny private-limit refusals are covered. Local release timings
were approximately 0.000027 ms for `v_2((10^18)!)`, 0.000068 ms for a huge
Kummer valuation, 0.00519 ms for a 1,000-step huge-n Lucas case, 0.236/0.246 ms
for 49,999/50,000-step direct/Wilson factorial paths, and 0.000006 ms for a
limit early exit.

CLI v0.4 adds one positional `comb` family. Web v0.5 adds one coherent Modular
combinatorics tool with an operation selector and conditional `k` input; JSON,
Copy Command, share links, elapsed time, exactness, and limit explanations use
the existing workflow infrastructure.

No factorial cache, generic combinatorics framework, prime-power/composite
binomial path, multinomial family, arbitrary-precision exact binomial
construction, thread, Web Worker, or frontend framework was introduced.
