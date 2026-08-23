use num_bigint::BigInt;

use crate::{FractionError, Rational};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolynomialError {
    Empty,
    DuplicateAbscissa,
    DivisionByZero,
    Fraction(FractionError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Polynomial {
    coefficients: Vec<Rational>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FiniteDifferences {
    pub rows: Vec<Vec<BigInt>>,
    pub polynomial_degree: Option<usize>,
}

impl Polynomial {
    #[must_use]
    pub fn new(mut coefficients: Vec<Rational>) -> Self {
        trim(&mut coefficients);
        Self { coefficients }
    }

    #[must_use]
    pub fn from_integers(coefficients: &[i64]) -> Self {
        Self::new(
            coefficients
                .iter()
                .map(|&value| Rational::from_i64(value))
                .collect(),
        )
    }

    #[must_use]
    pub fn coefficients(&self) -> &[Rational] {
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
    pub fn evaluate(&self, value: &Rational) -> Rational {
        self.coefficients
            .iter()
            .rev()
            .fold(Rational::from_i64(0), |accumulator, coefficient| {
                accumulator.mul(value).add(coefficient)
            })
    }

    #[must_use]
    pub fn format_human(&self, variable: &str) -> String {
        if self.is_zero() {
            return "0".to_owned();
        }
        self.coefficients
            .iter()
            .enumerate()
            .filter(|(_, coefficient)| !coefficient.is_zero())
            .map(|(degree, coefficient)| match degree {
                0 => coefficient.to_string(),
                1 => format!("({coefficient})·{variable}"),
                _ => format!("({coefficient})·{variable}^{degree}"),
            })
            .collect::<Vec<_>>()
            .join(" + ")
            .replace("+ (-", "- (")
    }

    fn leading(&self) -> Option<&Rational> {
        self.coefficients.last()
    }

    fn sub(&self, other: &Self) -> Self {
        let length = self.coefficients.len().max(other.coefficients.len());
        let mut output = Vec::with_capacity(length);
        for index in 0..length {
            let left = self
                .coefficients
                .get(index)
                .cloned()
                .unwrap_or_else(|| Rational::from_i64(0));
            let right = other
                .coefficients
                .get(index)
                .cloned()
                .unwrap_or_else(|| Rational::from_i64(0));
            output.push(left.sub(&right));
        }
        Self::new(output)
    }

    fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::new(Vec::new());
        }
        let mut output =
            vec![Rational::from_i64(0); self.coefficients.len() + other.coefficients.len() - 1];
        for (left_index, left) in self.coefficients.iter().enumerate() {
            for (right_index, right) in other.coefficients.iter().enumerate() {
                let product = left.mul(right);
                output[left_index + right_index] = output[left_index + right_index].add(&product);
            }
        }
        Self::new(output)
    }

    fn scale(&self, factor: &Rational) -> Self {
        Self::new(
            self.coefficients
                .iter()
                .map(|coefficient| coefficient.mul(factor))
                .collect(),
        )
    }

    fn remainder(&self, divisor: &Self) -> Result<Self, PolynomialError> {
        if divisor.is_zero() {
            return Err(PolynomialError::DivisionByZero);
        }
        let mut remainder = self.clone();
        let divisor_degree = divisor.degree().expect("nonzero divisor has a degree");
        while let Some(remainder_degree) = remainder.degree()
            && remainder_degree >= divisor_degree
        {
            let factor = remainder
                .leading()
                .expect("nonzero polynomial has a leading coefficient")
                .div(
                    divisor
                        .leading()
                        .expect("nonzero divisor has a leading coefficient"),
                )
                .map_err(PolynomialError::Fraction)?;
            let mut shifted = vec![Rational::from_i64(0); remainder_degree - divisor_degree];
            shifted.extend(divisor.scale(&factor).coefficients);
            remainder = remainder.sub(&Self::new(shifted));
        }
        Ok(remainder)
    }
}

pub fn polynomial_gcd(
    mut left: Polynomial,
    mut right: Polynomial,
) -> Result<Polynomial, PolynomialError> {
    while !right.is_zero() {
        let remainder = left.remainder(&right)?;
        left = right;
        right = remainder;
    }
    if let Some(leading) = left.leading().cloned() {
        left = left.scale(
            &Rational::from_i64(1)
                .div(&leading)
                .map_err(PolynomialError::Fraction)?,
        );
    }
    Ok(left)
}

pub fn interpolate(points: &[(Rational, Rational)]) -> Result<Polynomial, PolynomialError> {
    if points.is_empty() {
        return Err(PolynomialError::Empty);
    }
    let mut result = Polynomial::new(Vec::new());
    for (index, (x_i, y_i)) in points.iter().enumerate() {
        let mut basis = Polynomial::from_integers(&[1]);
        let mut denominator = Rational::from_i64(1);
        for (other_index, (x_j, _)) in points.iter().enumerate() {
            if index == other_index {
                continue;
            }
            let difference = x_i.sub(x_j);
            if difference.is_zero() {
                return Err(PolynomialError::DuplicateAbscissa);
            }
            basis = basis.mul(&Polynomial::new(vec![x_j.negated(), Rational::from_i64(1)]));
            denominator = denominator.mul(&difference);
        }
        let scale = y_i.div(&denominator).map_err(PolynomialError::Fraction)?;
        let term = basis.scale(&scale);
        result = result.sub(&term.scale(&Rational::from_i64(-1)));
    }
    Ok(result)
}

#[must_use]
pub fn finite_differences(sequence: &[BigInt]) -> FiniteDifferences {
    if sequence.is_empty() {
        return FiniteDifferences {
            rows: Vec::new(),
            polynomial_degree: None,
        };
    }
    let mut rows = vec![sequence.to_vec()];
    let mut degree = None;
    while rows.last().is_some_and(|row| row.len() > 1) {
        let previous = rows.last().expect("rows is nonempty");
        let next = previous
            .windows(2)
            .map(|pair| &pair[1] - &pair[0])
            .collect::<Vec<_>>();
        rows.push(next);
        let current = rows.last().expect("just pushed a row");
        if current.iter().all(|value| value == &current[0]) {
            degree = Some(rows.len() - 1);
            break;
        }
    }
    FiniteDifferences {
        rows,
        polynomial_degree: degree,
    }
}

fn trim(coefficients: &mut Vec<Rational>) {
    while coefficients.last().is_some_and(Rational::is_zero) {
        coefficients.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horner_interpolation_and_gcd_are_exact() {
        let polynomial = Polynomial::from_integers(&[1, 2, 3]);
        assert_eq!(
            polynomial.evaluate(&Rational::from_i64(4)).to_string(),
            "57"
        );
        let points =
            [(0, 1), (1, 6), (2, 17)].map(|(x, y)| (Rational::from_i64(x), Rational::from_i64(y)));
        assert_eq!(interpolate(&points).unwrap(), polynomial);
        let common = Polynomial::from_integers(&[-1, 1]);
        let left = common.mul(&Polynomial::from_integers(&[2, 1]));
        let right = common.mul(&Polynomial::from_integers(&[3, 1]));
        assert_eq!(polynomial_gcd(left, right).unwrap(), common);
    }

    #[test]
    fn finite_difference_degree_is_detected() {
        let values = [1_i64, 4, 9, 16, 25].map(BigInt::from);
        let result = finite_differences(&values);
        assert_eq!(result.polynomial_degree, Some(2));
    }

    #[test]
    fn canonical_zero_and_long_division_identity_hold() {
        let zero = Polynomial::from_integers(&[0, 0, 0]);
        assert!(zero.is_zero());
        assert_eq!(zero.degree(), None);

        let divisor = Polynomial::from_integers(&[-1, 0, 1]);
        let quotient = Polynomial::from_integers(&[2, 3, 1]);
        let remainder = Polynomial::from_integers(&[5, -2]);
        let dividend = divisor
            .mul(&quotient)
            .sub(&remainder.scale(&Rational::from_i64(-1)));
        assert_eq!(dividend.remainder(&divisor).unwrap(), remainder);
        assert!(remainder.degree().unwrap() < divisor.degree().unwrap());
    }

    #[test]
    fn interpolation_reproduces_samples_and_gcd_is_monic() {
        let source = Polynomial::from_integers(&[-3, 2, 0, 1]);
        let points = (-3..=3)
            .map(|x| {
                let x = Rational::from_i64(x);
                let y = source.evaluate(&x);
                (x, y)
            })
            .collect::<Vec<_>>();
        let recovered = interpolate(&points).unwrap();
        assert_eq!(recovered, source);
        for (x, y) in points {
            assert_eq!(recovered.evaluate(&x), y);
        }

        let common = Polynomial::from_integers(&[2, -3, 1]);
        let left = common.mul(&Polynomial::from_integers(&[-2, 1]));
        let right = common.mul(&Polynomial::from_integers(&[4, 1]));
        let gcd = polynomial_gcd(left, right).unwrap();
        assert_eq!(gcd, common);
        assert_eq!(gcd.leading(), Some(&Rational::from_i64(1)));
    }
}
