use crate::{FiniteFieldError, PrimeField};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FpPolynomial {
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
        Self::canonical(
            coefficients
                .iter()
                .map(|&value| field.normalize(value))
                .collect(),
        )
    }

    fn canonical(mut coefficients: Vec<u64>) -> Self {
        while coefficients.last() == Some(&0) {
            coefficients.pop();
        }
        Self { coefficients }
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

    #[must_use]
    pub fn add(&self, field: PrimeField, other: &Self) -> Self {
        let length = self.coefficients.len().max(other.coefficients.len());
        Self::canonical(
            (0..length)
                .map(|index| {
                    field.add(
                        self.coefficients.get(index).copied().unwrap_or(0),
                        other.coefficients.get(index).copied().unwrap_or(0),
                    )
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn sub(&self, field: PrimeField, other: &Self) -> Self {
        let length = self.coefficients.len().max(other.coefficients.len());
        Self::canonical(
            (0..length)
                .map(|index| {
                    field.sub(
                        self.coefficients.get(index).copied().unwrap_or(0),
                        other.coefficients.get(index).copied().unwrap_or(0),
                    )
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn mul(&self, field: PrimeField, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::canonical(Vec::new());
        }
        let mut output = vec![0; self.coefficients.len() + other.coefficients.len() - 1];
        for (left_index, &left) in self.coefficients.iter().enumerate() {
            for (right_index, &right) in other.coefficients.iter().enumerate() {
                let target = left_index + right_index;
                output[target] = field.add(output[target], field.mul(left, right));
            }
        }
        Self::canonical(output)
    }

    pub fn div_rem(
        &self,
        field: PrimeField,
        divisor: &Self,
    ) -> Result<(Self, Self), FiniteFieldError> {
        if divisor.is_zero() {
            return Err(FiniteFieldError::DivisionByZero);
        }
        let divisor_degree = divisor.degree().expect("nonzero divisor has a degree");
        let divisor_leading = divisor.coefficients[divisor_degree];
        let divisor_inverse = field
            .inverse(divisor_leading)
            .expect("a nonzero field value is invertible");
        let mut remainder = self.clone();
        let mut quotient = vec![0; self.degree().unwrap_or(0).saturating_sub(divisor_degree) + 1];
        while let Some(remainder_degree) = remainder.degree()
            && remainder_degree >= divisor_degree
        {
            let shift = remainder_degree - divisor_degree;
            let factor = field.mul(remainder.coefficients[remainder_degree], divisor_inverse);
            quotient[shift] = factor;
            for index in 0..=divisor_degree {
                let target = index + shift;
                remainder.coefficients[target] = field.sub(
                    remainder.coefficients[target],
                    field.mul(factor, divisor.coefficients[index]),
                );
            }
            remainder = Self::canonical(remainder.coefficients);
        }
        Ok((Self::canonical(quotient), remainder))
    }

    pub fn gcd(&self, field: PrimeField, other: &Self) -> Result<Self, FiniteFieldError> {
        let mut left = self.clone();
        let mut right = other.clone();
        while !right.is_zero() {
            let (_, remainder) = left.div_rem(field, &right)?;
            left = right;
            right = remainder;
        }
        Ok(left.monic(field))
    }

    pub fn extended_gcd(
        &self,
        field: PrimeField,
        other: &Self,
    ) -> Result<FpExtendedGcd, FiniteFieldError> {
        let zero = Self::canonical(Vec::new());
        let one = Self::canonical(vec![1]);
        let (mut old_r, mut r) = (self.clone(), other.clone());
        let (mut old_s, mut s) = (one.clone(), zero.clone());
        let (mut old_t, mut t) = (zero, one);
        while !r.is_zero() {
            let (quotient, remainder) = old_r.div_rem(field, &r)?;
            (old_r, r) = (r, remainder);
            let next_s = old_s.sub(field, &quotient.mul(field, &s));
            (old_s, s) = (s, next_s);
            let next_t = old_t.sub(field, &quotient.mul(field, &t));
            (old_t, t) = (t, next_t);
        }
        if old_r.is_zero() {
            return Ok(FpExtendedGcd {
                gcd: old_r,
                left_coefficient: old_s,
                right_coefficient: old_t,
            });
        }
        let inverse = field
            .inverse(
                *old_r
                    .coefficients
                    .last()
                    .expect("nonzero polynomial has a leading value"),
            )
            .expect("a nonzero field value is invertible");
        Ok(FpExtendedGcd {
            gcd: old_r.scale(field, inverse),
            left_coefficient: old_s.scale(field, inverse),
            right_coefficient: old_t.scale(field, inverse),
        })
    }

    #[must_use]
    pub fn derivative(&self, field: PrimeField) -> Self {
        Self::canonical(
            self.coefficients
                .iter()
                .enumerate()
                .skip(1)
                .map(|(degree, &coefficient)| {
                    field.mul(coefficient, field.normalize(degree as i128))
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn evaluate(&self, field: PrimeField, value: i128) -> u64 {
        let value = field.normalize(value);
        self.coefficients
            .iter()
            .rev()
            .fold(0, |result, &coefficient| {
                field.add(field.mul(result, value), coefficient)
            })
    }

    pub fn pow_mod(
        &self,
        field: PrimeField,
        mut exponent: u64,
        modulus: &Self,
    ) -> Result<Self, FiniteFieldError> {
        if modulus.is_zero() {
            return Err(FiniteFieldError::DivisionByZero);
        }
        let mut result = Self::canonical(vec![1]).div_rem(field, modulus)?.1;
        let mut base = self.div_rem(field, modulus)?.1;
        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result.mul(field, &base).div_rem(field, modulus)?.1;
            }
            exponent >>= 1;
            if exponent > 0 {
                base = base.mul(field, &base).div_rem(field, modulus)?.1;
            }
        }
        Ok(result)
    }

    #[must_use]
    fn scale(&self, field: PrimeField, factor: u64) -> Self {
        Self::canonical(
            self.coefficients
                .iter()
                .map(|&coefficient| field.mul(coefficient, factor))
                .collect(),
        )
    }

    #[must_use]
    fn monic(&self, field: PrimeField) -> Self {
        self.coefficients.last().map_or_else(
            || self.clone(),
            |&leading| {
                self.scale(
                    field,
                    field
                        .inverse(leading)
                        .expect("nonzero leading value is invertible"),
                )
            },
        )
    }
}
