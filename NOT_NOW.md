# Not now

SwissMath Core v0.10 intentionally remains a focused exact-computation toolkit.
The following ideas remain out of scope until a concrete consumer and
measurements justify them:

- symbolic algebra, expression parsing, DSLs, or generic proof/solver
  frameworks;
- prime counting, factor tables, or exact arbitrary-precision
  factorization/φ/λ/order;
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
- a query language or a frontend framework for the desktop UI;
- recurrence-period search and asymptotically faster recurrence algorithms;
- Pollard-rho discrete log, kth roots via discrete logs, and index calculus;
- GF(p^k), polynomial factorization, and generic group/sequence frameworks;
- Web Workers for expensive finite-field or multiplicative-group calculations;
- binomials modulo prime powers or composite moduli, CRT combinatorics,
  multinomials, Stirling/Bell/Catalan functions, prepared factorial tables,
  and faster large-factorial algorithms.
- generic reconstruction engines or strategies, automatic prime selection or
  rerunning of modular computations, stabilization heuristics, bad-prime or
  fault-tolerant CRT, prime powers, and non-coprime multimodular datasets;
- tensor abstractions, Rayon/threads/Web Workers, modulus schedulers, a general
  `BigRational` dependency, or a new reconstruction protocol/framework.

The v0.2 sieve deliberately uses one simple normalized anchor planner. Future
optimization should be evidence-driven by the deterministic benchmark rather
than added pre-emptively.
