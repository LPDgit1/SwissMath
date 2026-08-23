use crate::{FiniteFieldError, PrimeField};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FpPolynomial {
    field: PrimeField,
    coefficients: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FpExtendedGcd {
    pub gcd: FpPolynomial,
    pub left_coefficient: FpPolynomial,
    pub right_coefficient: FpPolynomial,
}

impl FpPolynomial {
    #[must_use]
    pub fn new(field: PrimeField, coefficients: &[i128]) -> Self {
        Self::from_normalized(
            field,
            coefficients
                .iter()
                .map(|&value| field.normalize(value))
                .collect(),
        )
    }

    fn from_normalized(field: PrimeField, mut coefficients: Vec<u64>) -> Self {
        while coefficients.last() == Some(&0) {
            coefficients.pop();
        }
        Self {
            field,
            coefficients,
        }
    }

    #[must_use]
    pub const fn field(&self) -> PrimeField {
        self.field
    }

    #[must_use]
    pub fn coefficients(&self) -> &[u64] {
        &self.coefficients
    }

    #[must_use]
    pub fn degree(&self) -> Option<usize> {
        (!self.coefficients.is_empty()).then(|| self.coefficients.len() - 1)
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.coefficients.is_empty()
    }

    pub fn add(&self, other: &Self) -> Result<Self, FiniteFieldError> {
        self.ensure_same_field(other)?;
        let length = self.coefficients.len().max(other.coefficients.len());
        Ok(Self::from_normalized(
            self.field,
            (0..length)
                .map(|index| {
                    self.field.add(
                        self.coefficients.get(index).copied().unwrap_or(0),
                        other.coefficients.get(index).copied().unwrap_or(0),
                    )
                })
                .collect(),
        ))
    }

    pub fn sub(&self, other: &Self) -> Result<Self, FiniteFieldError> {
        self.ensure_same_field(other)?;
        let length = self.coefficients.len().max(other.coefficients.len());
        Ok(Self::from_normalized(
            self.field,
            (0..length)
                .map(|index| {
                    self.field.sub(
                        self.coefficients.get(index).copied().unwrap_or(0),
                        other.coefficients.get(index).copied().unwrap_or(0),
                    )
                })
                .collect(),
        ))
    }

    pub fn mul(&self, other: &Self) -> Result<Self, FiniteFieldError> {
        self.ensure_same_field(other)?;
        if self.is_zero() || other.is_zero() {
            return Ok(Self::from_normalized(self.field, Vec::new()));
        }
        let mut output = vec![0; self.coefficients.len() + other.coefficients.len() - 1];
        for (left_index, &left) in self.coefficients.iter().enumerate() {
            for (right_index, &right) in other.coefficients.iter().enumerate() {
                let target = left_index + right_index;
                output[target] = self.field.add(output[target], self.field.mul(left, right));
            }
        }
        Ok(Self::from_normalized(self.field, output))
    }

    pub fn div_rem(&self, divisor: &Self) -> Result<(Self, Self), FiniteFieldError> {
        self.ensure_same_field(divisor)?;
        if divisor.is_zero() {
            return Err(FiniteFieldError::DivisionByZero);
        }
        let divisor_degree = divisor.degree().expect("nonzero divisor has a degree");
        let divisor_leading = divisor.coefficients[divisor_degree];
        let divisor_inverse = self
            .field
            .inverse(divisor_leading)
            .expect("a nonzero field value is invertible");
        let mut remainder = self.clone();
        let mut quotient = vec![0; self.degree().unwrap_or(0).saturating_sub(divisor_degree) + 1];
        while let Some(remainder_degree) = remainder.degree()
            && remainder_degree >= divisor_degree
        {
            let shift = remainder_degree - divisor_degree;
            let factor = self
                .field
                .mul(remainder.coefficients[remainder_degree], divisor_inverse);
            quotient[shift] = factor;
            for index in 0..=divisor_degree {
                let target = index + shift;
                remainder.coefficients[target] = self.field.sub(
                    remainder.coefficients[target],
                    self.field.mul(factor, divisor.coefficients[index]),
                );
            }
            remainder = Self::from_normalized(self.field, remainder.coefficients);
        }
        Ok((Self::from_normalized(self.field, quotient), remainder))
    }

    pub fn gcd(&self, other: &Self) -> Result<Self, FiniteFieldError> {
        self.ensure_same_field(other)?;
        let mut left = self.clone();
        let mut right = other.clone();
        while !right.is_zero() {
            let (_, remainder) = left.div_rem(&right)?;
            left = right;
            right = remainder;
        }
        Ok(left.monic())
    }

    pub fn extended_gcd(&self, other: &Self) -> Result<FpExtendedGcd, FiniteFieldError> {
        self.ensure_same_field(other)?;
        let zero = Self::from_normalized(self.field, Vec::new());
        let one = Self::from_normalized(self.field, vec![1]);
        let (mut old_r, mut r) = (self.clone(), other.clone());
        let (mut old_s, mut s) = (one.clone(), zero.clone());
        let (mut old_t, mut t) = (zero, one);
        while !r.is_zero() {
            let (quotient, remainder) = old_r.div_rem(&r)?;
            (old_r, r) = (r, remainder);
            let next_s = old_s.sub(&quotient.mul(&s)?)?;
            (old_s, s) = (s, next_s);
            let next_t = old_t.sub(&quotient.mul(&t)?)?;
            (old_t, t) = (t, next_t);
        }
        if old_r.is_zero() {
            return Ok(FpExtendedGcd {
                gcd: old_r,
                left_coefficient: old_s,
                right_coefficient: old_t,
            });
        }
        let inverse = self
            .field
            .inverse(
                *old_r
                    .coefficients
                    .last()
                    .expect("nonzero polynomial has a leading value"),
            )
            .expect("a nonzero field value is invertible");
        Ok(FpExtendedGcd {
            gcd: old_r.scale(inverse),
            left_coefficient: old_s.scale(inverse),
            right_coefficient: old_t.scale(inverse),
        })
    }

    #[must_use]
    pub fn derivative(&self) -> Self {
        Self::from_normalized(
            self.field,
            self.coefficients
                .iter()
                .enumerate()
                .skip(1)
                .map(|(degree, &coefficient)| {
                    self.field
                        .mul(coefficient, self.field.normalize(degree as i128))
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn evaluate(&self, value: i128) -> u64 {
        let value = self.field.normalize(value);
        self.coefficients
            .iter()
            .rev()
            .fold(0, |result, &coefficient| {
                self.field.add(self.field.mul(result, value), coefficient)
            })
    }

    pub fn pow_mod(&self, mut exponent: u64, modulus: &Self) -> Result<Self, FiniteFieldError> {
        self.ensure_same_field(modulus)?;
        if modulus.is_zero() {
            return Err(FiniteFieldError::DivisionByZero);
        }
        let mut result = Self::from_normalized(self.field, vec![1])
            .div_rem(modulus)?
            .1;
        let mut base = self.div_rem(modulus)?.1;
        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result.mul(&base)?.div_rem(modulus)?.1;
            }
            exponent >>= 1;
            if exponent > 0 {
                base = base.mul(&base)?.div_rem(modulus)?.1;
            }
        }
        Ok(result)
    }

    #[must_use]
    fn scale(&self, factor: u64) -> Self {
        Self::from_normalized(
            self.field,
            self.coefficients
                .iter()
                .map(|&coefficient| self.field.mul(coefficient, factor))
                .collect(),
        )
    }

    #[must_use]
    fn monic(&self) -> Self {
        self.coefficients.last().map_or_else(
            || self.clone(),
            |&leading| {
                self.scale(
                    self.field
                        .inverse(leading)
                        .expect("nonzero leading value is invertible"),
                )
            },
        )
    }

    fn ensure_same_field(&self, other: &Self) -> Result<(), FiniteFieldError> {
        if self.field == other.field {
            Ok(())
        } else {
            Err(FiniteFieldError::FieldMismatch {
                left_modulus: self.field.modulus(),
                right_modulus: other.field.modulus(),
            })
        }
    }
}
