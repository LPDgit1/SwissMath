use crate::{ModCtx, Modulus, is_prime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FiniteFieldError {
    InvalidPrime,
    Empty,
    Ragged,
    DimensionMismatch,
    NotSquare,
    Singular,
    DivisionByZero,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimeField {
    modulus: Modulus,
}

impl PrimeField {
    pub fn new(prime: u64) -> Result<Self, FiniteFieldError> {
        if prime < 2 || !is_prime(prime) {
            return Err(FiniteFieldError::InvalidPrime);
        }
        Ok(Self {
            modulus: Modulus::new(prime).expect("a prime is a nonzero modulus"),
        })
    }

    #[must_use]
    pub const fn modulus(self) -> u64 {
        self.modulus.get()
    }

    #[must_use]
    pub fn normalize(self, value: i128) -> u64 {
        value.rem_euclid(i128::from(self.modulus())) as u64
    }

    #[must_use]
    pub fn add(self, left: u64, right: u64) -> u64 {
        self.context().add(left, right)
    }

    #[must_use]
    pub fn sub(self, left: u64, right: u64) -> u64 {
        self.context().sub(left, right)
    }

    #[must_use]
    pub fn mul(self, left: u64, right: u64) -> u64 {
        self.context().mul(left, right)
    }

    #[must_use]
    pub fn pow(self, base: u64, exponent: u64) -> u64 {
        self.context().pow(base, exponent)
    }

    #[must_use]
    pub fn inverse(self, value: u64) -> Option<u64> {
        self.context().inv(value)
    }

    #[must_use]
    fn context(self) -> ModCtx {
        ModCtx::new(self.modulus)
    }
}
