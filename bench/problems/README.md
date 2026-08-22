# Mathematical workload harnesses

`macrobench` times representative modular-set and CRT patterns: quadratic
residues, sums of two squares, Pythagorean congruences, covering/periodic
filters, a small Diophantine sieve, and a Lonely-Runner-like filter.

`micro_modsieve` is an intentionally small internal stress harness. It builds
sets for `x^k + c = 0 (mod m)`, intersects filters, counts survivors, and ranks
candidate moduli. It is not a DSL or production solver.
