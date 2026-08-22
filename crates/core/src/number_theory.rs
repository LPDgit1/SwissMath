use core::fmt;

use num_bigint::BigUint;
use num_prime::nt_funcs::is_prime as big_is_prime;
use num_prime::{Primality, PrimalityTestConfig};

use crate::{ModCtx, Modulus, gcd};

// Shared by the primality fast path and recursive factor stripping.
pub(crate) const SMALL_PRIMES: &[u64] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
const MILLER_RABIN_WITNESSES: [u64; 7] = [2, 325, 9_375, 28178, 450_775, 9_780_504, 1_795_265_022];
const POLLARD_BATCH_SIZE: u64 = 96;
const POLLARD_ATTEMPTS: u64 = 32;
const POLLARD_WORK_LIMIT: u64 = 2_000_000;
const POLLARD_RECOVERY_LIMIT: u64 = 4_096;

/// Errors from exact u64 number-theory operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumberTheoryError {
    /// Prime factorization is undefined for zero.
    ZeroUndefined,
    /// The bounded Pollard search did not find a verified split.
    SearchFailed,
    /// A checked intermediate exceeded the supported u64 range.
    Overflow,
}

impl fmt::Display for NumberTheoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroUndefined => f.write_str("prime factorization is undefined for zero"),
            Self::SearchFailed => f.write_str("bounded Pollard search failed to find a factor"),
            Self::Overflow => f.write_str("exact number-theory result exceeds u64"),
        }
    }
}

impl std::error::Error for NumberTheoryError {}

/// Errors returned while routing a decimal primality request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimalityInputError {
    /// The input contains no digits.
    Empty,
    /// The input is not an unsigned decimal integer.
    InvalidDecimal,
    /// Unsigned primality assessment does not accept negative values.
    Negative,
}

impl fmt::Display for PrimalityInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("decimal input is empty"),
            Self::InvalidDecimal => f.write_str("decimal input is malformed"),
            Self::Negative => f.write_str("negative values are not supported"),
        }
    }
}

impl std::error::Error for PrimalityInputError {}

/// Semantic result of a primality assessment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimalityAssessment {
    /// The input is zero or one, hence neither prime nor composite.
    Neither,
    /// The input is definitively composite.
    Composite,
    /// The input passed an exact deterministic proof in the supported domain.
    PrimeExact,
    /// The bounded exact proof for a u128 value could not complete.
    ExactProofIncomplete,
    /// The input is larger than u128 and passed Baillie–PSW.
    ProbablePrime,
}

/// Classifies an analyzed integer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerClassification {
    /// The multiplicative identity.
    Unit,
    /// A prime greater than one.
    Prime,
    /// An integer greater than one with more than one prime factor occurrence.
    Composite,
}

/// One prime base and its positive exponent in a factorization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimePower {
    pub prime: u64,
    pub exponent: u32,
}

/// Canonical sorted prime-power factorization of a nonzero u64.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Factorization {
    n: u64,
    factors: Vec<PrimePower>,
}

impl Factorization {
    /// Returns the integer represented by this factorization.
    #[inline]
    #[must_use]
    pub const fn n(&self) -> u64 {
        self.n
    }

    /// Returns prime bases in strictly increasing order.
    #[inline]
    #[must_use]
    pub fn factors(&self) -> &[PrimePower] {
        &self.factors
    }

    /// Returns whether this is the empty factorization of one.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.factors.is_empty()
    }

    /// Computes Euler's totient using this already-computed factorization.
    #[inline]
    #[must_use]
    pub fn euler_phi(&self) -> u64 {
        self.factors.iter().fold(self.n, |result, factor| {
            (result / factor.prime) * (factor.prime - 1)
        })
    }

    /// Computes Carmichael's function using this already-computed factorization.
    pub fn carmichael_lambda(&self) -> Result<u64, NumberTheoryError> {
        carmichael_lambda_from_factorization(self)
    }
}

/// The one-call result used by the integer-analysis GUI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerAnalysis {
    pub n: u64,
    pub classification: IntegerClassification,
    pub primality: PrimalityAssessment,
    pub factorization: Factorization,
    pub phi: u64,
    pub lambda: u64,
}

/// Result of routing a decimal integer through exact or large-number analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecimalIntegerAnalysis {
    /// Zero has a well-defined primality assessment but no prime factorization.
    Neither { n: String },
    /// Exact u64 analysis, including factors and arithmetic functions.
    Exact(IntegerAnalysis),
    /// Bounded exact-first u128 assessment; arithmetic functions remain u64-only.
    U128 {
        /// Normalized decimal representation.
        n: String,
        /// Composite, exact proof, or deliberately incomplete exact proof.
        primality: PrimalityAssessment,
    },
    /// Large-number BPSW assessment; exact u64 arithmetic is unavailable.
    Large {
        /// Normalized decimal representation.
        n: String,
        /// Composite or probable-prime outcome.
        primality: PrimalityAssessment,
    },
}

/// Errors from the one-call decimal integer-analysis route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecimalIntegerAnalysisError {
    /// The decimal input is invalid.
    Input(PrimalityInputError),
    /// Exact u64 analysis failed.
    NumberTheory(NumberTheoryError),
}

impl fmt::Display for DecimalIntegerAnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(error) => error.fmt(f),
            Self::NumberTheory(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for DecimalIntegerAnalysisError {}

/// Explicit outcome of a multiplicative-order request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MultiplicativeOrderResult {
    /// The least positive exponent with `a^order ≡ 1 (mod n)`.
    Exists(u64),
    /// `a` and `n` are not coprime, so no multiplicative order exists.
    DoesNotExist,
}

/// Deterministic primality test exact over the complete u64 domain.
#[must_use]
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }

    for &prime in SMALL_PRIMES {
        if n == prime {
            return true;
        }
        if n % prime == 0 {
            return false;
        }
    }
    let mut odd_part = n - 1;
    let powers_of_two = odd_part.trailing_zeros();
    odd_part >>= powers_of_two;
    let modulus = Modulus::new(n).expect("n >= 2 is a valid modulus");
    let context = ModCtx::new(modulus);

    for &witness in &MILLER_RABIN_WITNESSES {
        let base = witness % n;
        if base == 0 {
            continue;
        }

        let mut value = context.pow(base, odd_part);
        if value == 1 || value == n - 1 {
            continue;
        }

        let mut witness_passed = false;
        for _ in 1..powers_of_two {
            value = context.mul(value, value);
            if value == n - 1 {
                witness_passed = true;
                break;
            }
        }
        if !witness_passed {
            return false;
        }
    }

    true
}

/// Factors a nonzero u64 into sorted unique prime powers.
pub fn factor(n: u64) -> Result<Factorization, NumberTheoryError> {
    if n == 0 {
        return Err(NumberTheoryError::ZeroUndefined);
    }
    if n == 1 {
        return Ok(Factorization {
            n,
            factors: Vec::new(),
        });
    }

    let mut raw = Vec::new();
    let mut remainder = n;
    for &prime in SMALL_PRIMES {
        while remainder % prime == 0 {
            raw.push(prime);
            remainder /= prime;
        }
    }
    if remainder > 1 {
        collect_factors(remainder, &mut raw)?;
    }

    raw.sort_unstable();
    let mut factors: Vec<PrimePower> = Vec::with_capacity(raw.len());
    for prime in raw {
        if let Some(last) = factors.last_mut()
            && last.prime == prime
        {
            last.exponent += 1;
            continue;
        }
        factors.push(PrimePower { prime, exponent: 1 });
    }
    Ok(Factorization { n, factors })
}

fn collect_factors(n: u64, output: &mut Vec<u64>) -> Result<(), NumberTheoryError> {
    if n == 1 {
        return Ok(());
    }
    if is_prime(n) {
        output.push(n);
        return Ok(());
    }
    let divisor = pollard_factor(n)?;
    collect_factors(divisor, output)?;
    collect_factors(n / divisor, output)
}

fn pollard_factor(n: u64) -> Result<u64, NumberTheoryError> {
    for attempt in 0..POLLARD_ATTEMPTS {
        if let Some(divisor) = pollard_brent(n, attempt) {
            if divisor > 1 && divisor < n && n % divisor == 0 {
                return Ok(divisor);
            }
        }
    }
    Err(NumberTheoryError::SearchFailed)
}

fn pollard_brent(n: u64, attempt: u64) -> Option<u64> {
    let modulus = Modulus::new(n)?;
    let context = ModCtx::new(modulus);
    let attempt_seed = splitmix64(n ^ attempt.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let range = n - 1;
    let mut y = splitmix64(attempt_seed) % range + 1;
    let c = splitmix64(attempt_seed ^ 0xD1B5_4A32_D192_ED03) % range + 1;
    let mut g = 1_u64;
    let mut r = 1_u64;
    let mut x = 0_u64;
    let mut ys = y;
    let mut work = 0_u64;

    while g == 1 && work < POLLARD_WORK_LIMIT {
        x = y;
        let mut index = 0_u64;
        while index < r && work < POLLARD_WORK_LIMIT {
            y = rho_step(&context, y, c);
            index += 1;
            work += 1;
        }

        let mut k = 0_u64;
        let mut q = 1_u64;
        while k < r && g == 1 && work < POLLARD_WORK_LIMIT {
            ys = y;
            let batch = (r - k).min(POLLARD_BATCH_SIZE);
            let mut batch_index = 0_u64;
            while batch_index < batch && work < POLLARD_WORK_LIMIT {
                y = rho_step(&context, y, c);
                q = context.mul(q, x.abs_diff(y));
                batch_index += 1;
                work += 1;
            }
            g = gcd(q, n);
            k += batch;
        }
        r = r.saturating_mul(2);
    }

    if g > 1 && g < n {
        return Some(g);
    }
    if g != n {
        return None;
    }

    let mut recovery_steps = 0_u64;
    while recovery_steps < POLLARD_RECOVERY_LIMIT {
        ys = rho_step(&context, ys, c);
        let recovered = gcd(x.abs_diff(ys), n);
        if recovered > 1 {
            return (recovered < n).then_some(recovered);
        }
        recovery_steps += 1;
    }
    None
}

#[inline]
fn rho_step(context: &ModCtx, value: u64, c: u64) -> u64 {
    context.add(context.mul(value, value), c)
}

#[inline]
fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut value = state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn carmichael_lambda_from_factorization(
    factorization: &Factorization,
) -> Result<u64, NumberTheoryError> {
    let mut result = 1_u64;
    for factor in &factorization.factors {
        let component = if factor.prime == 2 {
            match factor.exponent {
                0 => 1,
                1 => 1,
                2 => 2,
                exponent => 1_u64
                    .checked_shl(exponent - 2)
                    .ok_or(NumberTheoryError::Overflow)?,
            }
        } else {
            let mut power = 1_u64;
            for _ in 0..factor.exponent {
                power = power
                    .checked_mul(factor.prime)
                    .ok_or(NumberTheoryError::Overflow)?;
            }
            (power / factor.prime)
                .checked_mul(factor.prime - 1)
                .ok_or(NumberTheoryError::Overflow)?
        };
        let divisor = gcd(result, component);
        result = (result / divisor)
            .checked_mul(component)
            .ok_or(NumberTheoryError::Overflow)?;
    }
    Ok(result)
}

/// Performs the complete one-factorization integer analysis workflow.
pub fn analyze_integer(n: u64) -> Result<IntegerAnalysis, NumberTheoryError> {
    let factorization = factor(n)?;
    let classification = if n == 1 {
        IntegerClassification::Unit
    } else if factorization.factors.len() == 1 && factorization.factors[0].exponent == 1 {
        IntegerClassification::Prime
    } else {
        IntegerClassification::Composite
    };
    let primality = match classification {
        IntegerClassification::Unit => PrimalityAssessment::Neither,
        IntegerClassification::Prime => PrimalityAssessment::PrimeExact,
        IntegerClassification::Composite => PrimalityAssessment::Composite,
    };
    let phi = factorization.euler_phi();
    let lambda = factorization.carmichael_lambda()?;
    Ok(IntegerAnalysis {
        n,
        classification,
        primality,
        factorization,
        phi,
        lambda,
    })
}

/// Computes the multiplicative order of a modulo n when it exists.
pub fn multiplicative_order(
    a: u64,
    n: u64,
) -> Result<MultiplicativeOrderResult, NumberTheoryError> {
    if n == 0 {
        return Err(NumberTheoryError::ZeroUndefined);
    }
    if n == 1 {
        return Ok(MultiplicativeOrderResult::Exists(1));
    }

    let reduced_a = a % n;
    if gcd(reduced_a, n) != 1 {
        return Ok(MultiplicativeOrderResult::DoesNotExist);
    }

    let n_factorization = factor(n)?;
    let mut order = n_factorization.carmichael_lambda()?;
    let order_factorization = factor(order)?;
    let context = ModCtx::new(Modulus::new(n).expect("n > 1 is a valid modulus"));
    for factor in order_factorization.factors() {
        while order % factor.prime == 0 && context.pow(reduced_a, order / factor.prime) == 1 {
            order /= factor.prime;
        }
    }
    Ok(MultiplicativeOrderResult::Exists(order))
}

fn validate_decimal_input(input: &str) -> Result<&str, PrimalityInputError> {
    let value = input.trim();
    if value.is_empty() {
        return Err(PrimalityInputError::Empty);
    }
    if value.starts_with('-') {
        return Err(PrimalityInputError::Negative);
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(PrimalityInputError::InvalidDecimal);
    }
    Ok(value)
}

fn parse_decimal_biguint(value: &str) -> Result<BigUint, PrimalityInputError> {
    BigUint::parse_bytes(value.as_bytes(), 10).ok_or(PrimalityInputError::InvalidDecimal)
}

fn assess_biguint(value: &BigUint) -> PrimalityAssessment {
    match big_is_prime(value, Some(PrimalityTestConfig::bpsw())) {
        Primality::No => PrimalityAssessment::Composite,
        Primality::Yes | Primality::Probable(_) => PrimalityAssessment::ProbablePrime,
    }
}

fn assess_u64(value: u64) -> PrimalityAssessment {
    match value {
        0 | 1 => PrimalityAssessment::Neither,
        _ if is_prime(value) => PrimalityAssessment::PrimeExact,
        _ => PrimalityAssessment::Composite,
    }
}

/// Routes a decimal integer through the exact u64, bounded u128, or BPSW path.
pub fn assess_primality_decimal(input: &str) -> Result<PrimalityAssessment, PrimalityInputError> {
    let input = validate_decimal_input(input)?;
    if let Ok(value_u64) = input.parse::<u64>() {
        return Ok(assess_u64(value_u64));
    }
    if let Ok(value_u128) = input.parse::<u128>() {
        return Ok(crate::wide_primality::assess_primality_u128(value_u128));
    }
    let value = parse_decimal_biguint(input)?;
    Ok(assess_biguint(&value))
}

/// Routes one decimal integer to full exact analysis or large-number assessment.
pub fn analyze_integer_decimal(
    input: &str,
) -> Result<DecimalIntegerAnalysis, DecimalIntegerAnalysisError> {
    let input = validate_decimal_input(input).map_err(DecimalIntegerAnalysisError::Input)?;
    if let Ok(value_u64) = input.parse::<u64>() {
        if value_u64 == 0 {
            return Ok(DecimalIntegerAnalysis::Neither { n: "0".to_owned() });
        }
        return analyze_integer(value_u64)
            .map(DecimalIntegerAnalysis::Exact)
            .map_err(DecimalIntegerAnalysisError::NumberTheory);
    }
    if let Ok(value_u128) = input.parse::<u128>() {
        return Ok(DecimalIntegerAnalysis::U128 {
            n: value_u128.to_string(),
            primality: crate::wide_primality::assess_primality_u128(value_u128),
        });
    }
    let value = parse_decimal_biguint(input).map_err(DecimalIntegerAnalysisError::Input)?;
    let primality = assess_biguint(&value);
    Ok(DecimalIntegerAnalysis::Large {
        n: value.to_str_radix(10),
        primality,
    })
}
