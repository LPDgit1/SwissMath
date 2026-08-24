#![forbid(unsafe_code)]

mod arithmetic;
mod bitops;
mod combinatorics;
mod congruence;
mod crt;
mod discovery;
mod finite_field;
mod finite_field_matrix;
mod finite_field_polynomial;
mod fractions;
mod linear_algebra;
mod modular;
mod modulus;
mod multimodular;
mod multiplicative_groups;
mod number_theory;
mod polynomials;
mod quadratic;
mod recurrence;
mod residue;
mod sieve;
mod wide_primality;

pub use arithmetic::{
    BaseConversionError, ExtendedGcd, IntegerRoot, PerfectPower, extended_gcd, format_in_base,
    integer_nth_root, lcm, parse_in_base, perfect_power,
};
pub use combinatorics::{
    CombinatoricsError, binomial_mod_prime, binomial_valuation, factorial_mod_prime,
    factorial_valuation,
};
pub use congruence::{
    LinearCongruence, LinearSolution, LinearSolveResult, solve_linear_congruence,
    solve_linear_system,
};
pub use crt::{Congruence, crt_compatible, crt_fold, crt_pair};
pub use discovery::{
    DiscoveryError, GuessCandidate, IntegerRelation, RecurrenceCandidate, berlekamp_massey,
    berlekamp_massey_mod_prime, find_recurrence, guess_sequence, pslq,
};
pub use finite_field::{FiniteFieldError, PrimeField};
pub use finite_field_matrix::{FpLinearSystemSolution, FpMatrix, FpRrefResult};
pub use finite_field_polynomial::{FpExtendedGcd, FpPolynomial};
pub use fractions::{
    FractionError, Rational, RationalReconstruction, Rationalization, continued_fraction,
    convergents, parse_decimal, rational_reconstruct, rational_reconstruct_bounded,
    rationalize_decimal,
};
pub use linear_algebra::{
    LinearSystemSolution, MatrixError, RationalMatrix, RrefResult, determinant_bareiss,
    hermite_normal_form, nullspace, rank, rref, smith_normal_form_invariants, solve,
};
pub use modular::{ArithmeticError, ModCtx, gcd, inv_mod, reduce_i128};
pub use modulus::Modulus;
pub use multimodular::{
    BigRationalReconstruction, MultimodularAccumulator, MultimodularError, centered_representative,
    rational_reconstruct_big, rational_reconstruct_big_bounded, reconstruct_integer_bounded,
};
pub use multiplicative_groups::{
    DiscreteLogResult, MultiplicativeGroupError, discrete_log, is_primitive_root, primitive_root,
};
pub use number_theory::{
    DecimalIntegerAnalysis, DecimalIntegerAnalysisError, DivisorSummary, Factorization,
    IntegerAnalysis, IntegerClassification, MultiplicativeOrderResult, NumberTheoryError,
    PrimalityAssessment, PrimalityInputError, PrimePower, Valuation, analyze_integer,
    analyze_integer_decimal, assess_primality_decimal, factor, is_prime, multiplicative_order,
    next_prime, previous_prime, valuation,
};
pub use polynomials::{
    FiniteDifferences, Polynomial, PolynomialError, finite_differences, interpolate, polynomial_gcd,
};
pub use quadratic::{
    PrimeRoots, QuadraticError, jacobi_symbol, legendre_symbol, modular_square_roots,
    prime_square_roots,
};
pub use recurrence::{
    InferredRecurrenceResult, RecurrenceError, infer_recurrence_nth_mod_prime,
    linear_recurrence_nth_mod_prime,
};
pub use residue::{ResidueError, ResidueIter, ResidueSet, required_heap_bytes};
pub use sieve::{ModularFilter, ModularFilterBuild, ModularSieve, SieveError, SieveResult};
pub use wide_primality::assess_primality_u128;
