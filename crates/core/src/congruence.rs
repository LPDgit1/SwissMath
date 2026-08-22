use crate::{ArithmeticError, Congruence, Modulus, crt_fold, gcd, inv_mod};

/// A canonical linear congruence `a * x = b (mod modulus)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinearCongruence {
    a: u64,
    b: u64,
    modulus: Modulus,
}

impl LinearCongruence {
    /// Constructs a congruence, reducing `a` and `b` to canonical residues.
    #[must_use]
    pub fn new(a: u64, b: u64, modulus: Modulus) -> Self {
        let m = modulus.get();
        Self {
            a: a % m,
            b: b % m,
            modulus,
        }
    }

    /// Returns the normalized coefficient of `x`.
    #[inline]
    #[must_use]
    pub const fn a(self) -> u64 {
        self.a
    }

    /// Returns the normalized right-hand side.
    #[inline]
    #[must_use]
    pub const fn b(self) -> u64 {
        self.b
    }

    /// Returns the nonzero modulus.
    #[inline]
    #[must_use]
    pub const fn modulus(self) -> Modulus {
        self.modulus
    }
}

/// The compact mathematical result of a linear congruence or system.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinearSolution {
    /// No integer satisfies the constraint.
    None,
    /// Every integer satisfies the constraint.
    All,
    /// One congruence class describes all solutions.
    Class(Congruence),
}

/// Structured facts produced while solving one linear congruence.
///
/// The fields are deliberately concrete: they are exactly the values needed by
/// a small explanation UI and are not a general proof trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinearSolveResult {
    /// The normalized coefficient `a`.
    pub normalized_a: u64,
    /// The normalized right-hand side `b`.
    pub normalized_b: u64,
    /// `gcd(a, modulus)`.
    pub gcd: u64,
    /// Reduced coefficient after division by the gcd, when reduction applies.
    pub reduced_a: Option<u64>,
    /// Reduced right-hand side after division by the gcd, when reduction applies.
    pub reduced_b: Option<u64>,
    /// Reduced modulus after division by the gcd.
    pub reduced_modulus: u64,
    /// Inverse of the reduced coefficient, when one is needed and exists.
    pub inverse: Option<u64>,
    /// The compact solution classification.
    pub solution: LinearSolution,
}

impl LinearSolveResult {
    /// Number of represented solutions modulo the original modulus.
    #[must_use]
    pub fn solution_count(&self, original_modulus: Modulus) -> u64 {
        match self.solution {
            LinearSolution::None => 0,
            LinearSolution::All => original_modulus.get(),
            LinearSolution::Class(_) => self.gcd,
        }
    }
}

/// Solves `a * x = b (mod m)` exactly.
#[must_use]
pub fn solve_linear_congruence(equation: LinearCongruence) -> LinearSolveResult {
    let modulus = equation.modulus;
    let m = modulus.get();
    let a = equation.a;
    let b = equation.b;

    if m == 1 {
        return LinearSolveResult {
            normalized_a: 0,
            normalized_b: 0,
            gcd: 1,
            reduced_a: None,
            reduced_b: None,
            reduced_modulus: 1,
            inverse: None,
            solution: LinearSolution::All,
        };
    }

    let divisor = gcd(a, m);
    let reduced_modulus = m / divisor;
    if b % divisor != 0 {
        return LinearSolveResult {
            normalized_a: a,
            normalized_b: b,
            gcd: divisor,
            reduced_a: None,
            reduced_b: None,
            reduced_modulus,
            inverse: None,
            solution: LinearSolution::None,
        };
    }

    if reduced_modulus == 1 {
        return LinearSolveResult {
            normalized_a: a,
            normalized_b: b,
            gcd: divisor,
            reduced_a: Some(0),
            reduced_b: Some(0),
            reduced_modulus,
            inverse: None,
            solution: LinearSolution::All,
        };
    }

    let reduced_a = a / divisor;
    let reduced_b = b / divisor;
    let reduced_modulus_value = Modulus::new(reduced_modulus).expect("reduced modulus is nonzero");
    let inverse = inv_mod(reduced_a, reduced_modulus_value)
        .expect("the reduced coefficient is coprime to the reduced modulus");
    let residue =
        (u128::from(reduced_b) * u128::from(inverse) % u128::from(reduced_modulus)) as u64;

    LinearSolveResult {
        normalized_a: a,
        normalized_b: b,
        gcd: divisor,
        reduced_a: Some(reduced_a),
        reduced_b: Some(reduced_b),
        reduced_modulus,
        inverse: Some(inverse),
        solution: LinearSolution::Class(Congruence::new(residue, reduced_modulus_value)),
    }
}

/// Solves a finite system of linear congruences using the existing generalized CRT.
pub fn solve_linear_system<I>(equations: I) -> Result<LinearSolution, ArithmeticError>
where
    I: IntoIterator<Item = LinearCongruence>,
{
    let mut classes = Vec::new();
    for equation in equations {
        match solve_linear_congruence(equation).solution {
            LinearSolution::None => return Ok(LinearSolution::None),
            LinearSolution::All => {}
            LinearSolution::Class(congruence) => classes.push(congruence),
        }
    }

    if classes.is_empty() {
        return Ok(LinearSolution::All);
    }

    Ok(match crt_fold(classes)? {
        Some(congruence) => LinearSolution::Class(congruence),
        None => LinearSolution::None,
    })
}
