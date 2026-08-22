#![forbid(unsafe_code)]

mod bitops;
mod congruence;
mod crt;
mod modular;
mod modulus;
mod number_theory;
mod quadratic;
mod residue;
mod sieve;
mod wide_primality;

pub use congruence::{
    LinearCongruence, LinearSolution, LinearSolveResult, solve_linear_congruence,
    solve_linear_system,
};
pub use crt::{Congruence, crt_compatible, crt_fold, crt_pair};
pub use modular::{ArithmeticError, ModCtx, gcd, inv_mod, reduce_i128};
pub use modulus::Modulus;
pub use number_theory::{
    DecimalIntegerAnalysis, DecimalIntegerAnalysisError, Factorization, IntegerAnalysis,
    IntegerClassification, MultiplicativeOrderResult, NumberTheoryError, PrimalityAssessment,
    PrimalityInputError, PrimePower, analyze_integer, analyze_integer_decimal,
    assess_primality_decimal, factor, is_prime, multiplicative_order,
};
pub use quadratic::{
    PrimeRoots, QuadraticError, jacobi_symbol, legendre_symbol, modular_square_roots,
    prime_square_roots,
};
pub use residue::{ResidueError, ResidueIter, ResidueSet, required_heap_bytes};
pub use sieve::{ModularFilter, ModularFilterBuild, ModularSieve, SieveError, SieveResult};
pub use wide_primality::assess_primality_u128;
