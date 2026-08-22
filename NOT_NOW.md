# Not now

SwissMath Core v0.5 intentionally stops at exact u64 arithmetic plus a bounded
exact-first u128 primality proof, a large-number probable-primality assessment,
and focused quadratic unit-root operations.
The following ideas remain out of scope until a concrete consumer and
measurements justify them:

- symbolic algebra, polynomial/finite-field APIs, expression parsing, DSLs, or
  generic proof/solver frameworks;
- prime generation, prime counting, factor tables, divisor functions, or exact
  arbitrary-precision factorization/φ/λ/order;
- formal primality certificate export/verification (ECPP, APR-CL, AKS, Pratt),
  general arbitrary-precision modular arithmetic, sparse/compressed residue-set
  backends, or global-LCM
  residue materialization;
- sieve wheels, incremental remainder engines, CRT fusion, competing planners,
  SIMD, GPU, or parallel execution;
- Montgomery or Barrett arithmetic, ECM, SQUFOF, quadratic sieve, perfect-power
  decomposition, semiprime special paths, or a Pollard algorithm portfolio;
- general composite non-coprime modular square roots, Cipolla, symbolic root
  families, or generic Hensel frameworks;
- plugins, bindings, scripting, networking, cloud services, telemetry,
  databases, async/distributed execution, or internal threading;
- custom allocators, production `unsafe`, target-specific tuning, or speculative
  arithmetic-reduction machinery;
- a query language or a frontend framework for the desktop UI.

The v0.2 sieve deliberately uses one simple normalized anchor planner. Future
optimization should be evidence-driven by the deterministic benchmark rather
than added pre-emptively.
