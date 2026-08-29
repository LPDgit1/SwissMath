use num_bigint::BigInt;

use crate::{Polynomial, Rational};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatrixError {
    Empty,
    Ragged,
    NotSquare,
    DimensionMismatch,
    Singular,
    NoRelation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RationalMatrix {
    rows: usize,
    columns: usize,
    data: Vec<Vec<Rational>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RrefResult {
    pub matrix: RationalMatrix,
    pub pivot_columns: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Eigenvalue {
    pub value: Rational,
    pub algebraic_multiplicity: usize,
    pub geometric_multiplicity: usize,
    pub eigenspace_basis: Vec<Vec<Rational>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EigenvalueAnalysis {
    pub characteristic_polynomial: Polynomial,
    pub eigenvalues: Vec<Eigenvalue>,
    pub remaining_factor: Option<Polynomial>,
    pub search_limited: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagonalizationResult {
    Diagonalizable {
        analysis: EigenvalueAnalysis,
        p: RationalMatrix,
        d: RationalMatrix,
        inverse: RationalMatrix,
    },
    NotDiagonalizable {
        analysis: EigenvalueAnalysis,
    },
    NotSplit {
        analysis: EigenvalueAnalysis,
    },
    SearchLimit {
        analysis: EigenvalueAnalysis,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LuDecomposition {
    pub permutation: RationalMatrix,
    pub lower: RationalMatrix,
    pub upper: RationalMatrix,
    pub singular: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinearSystemSolution {
    None,
    Unique(Vec<Rational>),
    Infinite {
        particular: Vec<Rational>,
        nullspace_basis: Vec<Vec<Rational>>,
    },
}

impl RationalMatrix {
    pub fn new(data: Vec<Vec<Rational>>) -> Result<Self, MatrixError> {
        if data.is_empty() || data[0].is_empty() {
            return Err(MatrixError::Empty);
        }
        let columns = data[0].len();
        if data.iter().any(|row| row.len() != columns) {
            return Err(MatrixError::Ragged);
        }
        Ok(Self {
            rows: data.len(),
            columns,
            data,
        })
    }

    pub fn from_integers(data: &[Vec<i64>]) -> Result<Self, MatrixError> {
        Self::new(
            data.iter()
                .map(|row| row.iter().map(|&value| Rational::from_i64(value)).collect())
                .collect(),
        )
    }

    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }

    #[must_use]
    pub fn data(&self) -> &[Vec<Rational>] {
        &self.data
    }

    pub fn identity(size: usize) -> Result<Self, MatrixError> {
        if size == 0 {
            return Err(MatrixError::Empty);
        }
        let mut data = vec![vec![Rational::from_i64(0); size]; size];
        for (index, row) in data.iter_mut().enumerate() {
            row[index] = Rational::from_i64(1);
        }
        Self::new(data)
    }

    #[must_use]
    pub fn transpose(&self) -> Self {
        let data = (0..self.columns)
            .map(|column| {
                (0..self.rows)
                    .map(|row| self.data[row][column].clone())
                    .collect()
            })
            .collect();
        Self::new(data).expect("transpose preserves a non-empty rectangular shape")
    }

    pub fn add(&self, other: &Self) -> Result<Self, MatrixError> {
        if self.rows != other.rows || self.columns != other.columns {
            return Err(MatrixError::DimensionMismatch);
        }
        Self::new(
            self.data
                .iter()
                .zip(&other.data)
                .map(|(left, right)| left.iter().zip(right).map(|(a, b)| a.add(b)).collect())
                .collect(),
        )
    }

    pub fn sub(&self, other: &Self) -> Result<Self, MatrixError> {
        if self.rows != other.rows || self.columns != other.columns {
            return Err(MatrixError::DimensionMismatch);
        }
        Self::new(
            self.data
                .iter()
                .zip(&other.data)
                .map(|(left, right)| left.iter().zip(right).map(|(a, b)| a.sub(b)).collect())
                .collect(),
        )
    }

    pub fn mul(&self, other: &Self) -> Result<Self, MatrixError> {
        if self.columns != other.rows {
            return Err(MatrixError::DimensionMismatch);
        }
        let mut data = vec![vec![Rational::from_i64(0); other.columns]; self.rows];
        for (row_index, row) in data.iter_mut().enumerate() {
            for (column_index, value) in row.iter_mut().enumerate() {
                for inner in 0..self.columns {
                    *value = value
                        .add(&self.data[row_index][inner].mul(&other.data[inner][column_index]));
                }
            }
        }
        Self::new(data)
    }

    pub fn trace(&self) -> Result<Rational, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        Ok((0..self.rows).fold(Rational::from_i64(0), |sum, index| {
            sum.add(&self.data[index][index])
        }))
    }

    pub fn determinant(&self) -> Result<Rational, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        let mut data = self.data.clone();
        let mut determinant = Rational::from_i64(1);
        let mut sign = Rational::from_i64(1);
        for column in 0..self.columns {
            let Some(pivot) = (column..self.rows).find(|&row| !data[row][column].is_zero()) else {
                return Ok(Rational::from_i64(0));
            };
            if pivot != column {
                data.swap(pivot, column);
                sign = sign.negated();
            }
            let pivot_value = data[column][column].clone();
            determinant = determinant.mul(&pivot_value);
            for row in column + 1..self.rows {
                let factor = data[row][column]
                    .div(&pivot_value)
                    .expect("a nonzero pivot has an inverse");
                data[row][column] = Rational::from_i64(0);
                let pivot_values = data[column][column + 1..].to_vec();
                for (entry, pivot) in data[row][column + 1..].iter_mut().zip(pivot_values) {
                    *entry = entry.sub(&factor.mul(&pivot));
                }
            }
        }
        Ok(sign.mul(&determinant))
    }

    pub fn power(&self, exponent: u64) -> Result<Self, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        let mut result = Self::identity(self.rows)?;
        let mut base = self.clone();
        let mut exponent = exponent;
        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result.mul(&base)?;
            }
            exponent >>= 1;
            if exponent > 0 {
                base = base.mul(&base)?;
            }
        }
        Ok(result)
    }

    pub fn inverse(&self) -> Result<Self, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        let width = self.columns * 2;
        let mut augmented = vec![vec![Rational::from_i64(0); width]; self.rows];
        for (row_index, row) in augmented.iter_mut().enumerate() {
            row[..self.columns].clone_from_slice(&self.data[row_index]);
            row[self.columns + row_index] = Rational::from_i64(1);
        }
        let reduced = rref(&Self::new(augmented)?);
        if reduced
            .pivot_columns
            .iter()
            .filter(|&&column| column < self.columns)
            .count()
            != self.columns
        {
            return Err(MatrixError::Singular);
        }
        Self::new(
            reduced
                .matrix
                .data
                .iter()
                .map(|row| row[self.columns..].to_vec())
                .collect(),
        )
    }

    pub fn characteristic_polynomial(&self) -> Result<Polynomial, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        let size = self.rows;
        let mut b = Self::identity(size)?;
        let mut coefficients = vec![Rational::from_i64(0); size + 1];
        coefficients[size] = Rational::from_i64(1);
        for step in 1..=size {
            let product = self.mul(&b)?;
            let trace = product.trace()?;
            let divisor = Rational::new(BigInt::from(step), BigInt::from(1))
                .expect("a positive step is a valid rational denominator");
            let coefficient = trace
                .div(&divisor)
                .expect("a positive step is a valid rational denominator")
                .negated();
            coefficients[size - step] = coefficient.clone();
            b = product;
            for index in 0..size {
                b.data[index][index] = b.data[index][index].add(&coefficient);
            }
        }
        Ok(Polynomial::new(coefficients))
    }

    pub fn eigenspace(&self, eigenvalue: &Rational) -> Result<Vec<Vec<Rational>>, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        let mut shifted = self.data.clone();
        for (index, row) in shifted.iter_mut().enumerate() {
            row[index] = row[index].sub(eigenvalue);
        }
        Ok(nullspace(&Self::new(shifted)?))
    }

    pub fn eigenvalue_analysis(&self) -> Result<EigenvalueAnalysis, MatrixError> {
        let characteristic_polynomial = self.characteristic_polynomial()?;
        let (roots, remaining_factor, search_limited) =
            rational_roots_with_multiplicity(&characteristic_polynomial);
        let eigenvalues = roots
            .into_iter()
            .map(|(value, algebraic_multiplicity)| {
                let eigenspace_basis = self.eigenspace(&value)?;
                Ok(Eigenvalue {
                    value,
                    algebraic_multiplicity,
                    geometric_multiplicity: eigenspace_basis.len(),
                    eigenspace_basis,
                })
            })
            .collect::<Result<Vec<_>, MatrixError>>()?;
        Ok(EigenvalueAnalysis {
            characteristic_polynomial,
            eigenvalues,
            remaining_factor,
            search_limited,
        })
    }

    pub fn diagonalize(&self) -> Result<DiagonalizationResult, MatrixError> {
        let analysis = self.eigenvalue_analysis()?;
        if analysis.search_limited {
            return Ok(DiagonalizationResult::SearchLimit { analysis });
        }
        if analysis.remaining_factor.is_some() {
            return Ok(DiagonalizationResult::NotSplit { analysis });
        }
        let dimension = self.rows;
        let has_full_eigenspaces = analysis.eigenvalues.iter().all(|eigenvalue| {
            eigenvalue.algebraic_multiplicity == eigenvalue.geometric_multiplicity
        });
        let total_dimension = analysis
            .eigenvalues
            .iter()
            .map(|eigenvalue| eigenvalue.geometric_multiplicity)
            .sum::<usize>();
        if !has_full_eigenspaces || total_dimension != dimension {
            return Ok(DiagonalizationResult::NotDiagonalizable { analysis });
        }

        let mut p_data = vec![vec![Rational::from_i64(0); dimension]; dimension];
        let mut d_data = vec![vec![Rational::from_i64(0); dimension]; dimension];
        let mut column = 0;
        for eigenvalue in &analysis.eigenvalues {
            for vector in &eigenvalue.eigenspace_basis {
                for row in 0..dimension {
                    p_data[row][column] = vector[row].clone();
                }
                d_data[column][column] = eigenvalue.value.clone();
                column += 1;
            }
        }
        let p = Self::new(p_data)?;
        let d = Self::new(d_data)?;
        let inverse = p.inverse()?;
        Ok(DiagonalizationResult::Diagonalizable {
            analysis,
            p,
            d,
            inverse,
        })
    }

    pub fn minimal_polynomial(&self) -> Result<Polynomial, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        let size = self.rows;
        let mut powers = Vec::with_capacity(size + 1);
        powers.push(Self::identity(size)?);
        for degree in 1..=size {
            let next = powers[degree - 1].mul(self)?;
            powers.push(next);
        }
        for degree in 1..=size {
            let equations = Self::new(flatten_power_columns(&powers[..degree]))?;
            let rhs = flatten_matrix(&powers[degree])
                .into_iter()
                .map(|value| value.negated())
                .collect::<Vec<_>>();
            match solve(&equations, &rhs)? {
                LinearSystemSolution::Unique(coefficients)
                | LinearSystemSolution::Infinite {
                    particular: coefficients,
                    ..
                } => {
                    let mut coefficients = coefficients;
                    coefficients.push(Rational::from_i64(1));
                    return Ok(Polynomial::new(coefficients));
                }
                LinearSystemSolution::None => {}
            }
        }
        Err(MatrixError::NoRelation)
    }

    pub fn lu_decomposition(&self) -> Result<LuDecomposition, MatrixError> {
        if self.rows != self.columns {
            return Err(MatrixError::NotSquare);
        }
        let size = self.rows;
        let mut permutation = Self::identity(size)?.data;
        let mut lower = vec![vec![Rational::from_i64(0); size]; size];
        let mut upper = self.data.clone();
        for (index, row) in lower.iter_mut().enumerate() {
            row[index] = Rational::from_i64(1);
        }
        let mut singular = false;
        for pivot_column in 0..size {
            let Some(pivot_row) =
                (pivot_column..size).find(|&row| !upper[row][pivot_column].is_zero())
            else {
                singular = true;
                continue;
            };
            if pivot_row != pivot_column {
                upper.swap(pivot_row, pivot_column);
                permutation.swap(pivot_row, pivot_column);
                let pivot_row_values = lower[pivot_row][..pivot_column].to_vec();
                let pivot_column_values = lower[pivot_column][..pivot_column].to_vec();
                lower[pivot_row][..pivot_column].clone_from_slice(&pivot_column_values);
                lower[pivot_column][..pivot_column].clone_from_slice(&pivot_row_values);
            }
            let pivot = upper[pivot_column][pivot_column].clone();
            for row in pivot_column + 1..size {
                let factor = upper[row][pivot_column]
                    .div(&pivot)
                    .expect("a nonzero pivot has an inverse");
                lower[row][pivot_column] = factor.clone();
                upper[row][pivot_column] = Rational::from_i64(0);
                let pivot_values = upper[pivot_column][pivot_column + 1..].to_vec();
                for (entry, pivot_value) in
                    upper[row][pivot_column + 1..].iter_mut().zip(pivot_values)
                {
                    *entry = entry.sub(&factor.mul(&pivot_value));
                }
            }
        }
        Ok(LuDecomposition {
            permutation: Self::new(permutation)?,
            lower: Self::new(lower)?,
            upper: Self::new(upper)?,
            singular,
        })
    }
}

pub fn characteristic_polynomial(matrix: &RationalMatrix) -> Result<Polynomial, MatrixError> {
    matrix.characteristic_polynomial()
}

pub fn eigenvalue_analysis(matrix: &RationalMatrix) -> Result<EigenvalueAnalysis, MatrixError> {
    matrix.eigenvalue_analysis()
}

pub fn diagonalize(matrix: &RationalMatrix) -> Result<DiagonalizationResult, MatrixError> {
    matrix.diagonalize()
}

pub fn minimal_polynomial(matrix: &RationalMatrix) -> Result<Polynomial, MatrixError> {
    matrix.minimal_polynomial()
}

pub fn lu_decomposition(matrix: &RationalMatrix) -> Result<LuDecomposition, MatrixError> {
    matrix.lu_decomposition()
}

#[must_use]
pub fn rref(matrix: &RationalMatrix) -> RrefResult {
    let mut data = matrix.data.clone();
    let mut pivot_columns = Vec::new();
    let mut pivot_row = 0;
    for column in 0..matrix.columns {
        let Some(selected) = (pivot_row..matrix.rows).find(|&row| !data[row][column].is_zero())
        else {
            continue;
        };
        data.swap(pivot_row, selected);
        let inverse = Rational::from_i64(1)
            .div(&data[pivot_row][column])
            .expect("pivot is nonzero");
        for value in &mut data[pivot_row] {
            *value = value.mul(&inverse);
        }
        for row in 0..matrix.rows {
            if row == pivot_row || data[row][column].is_zero() {
                continue;
            }
            let factor = data[row][column].clone();
            let pivot_values = data[pivot_row].clone();
            for (entry, pivot) in data[row].iter_mut().zip(pivot_values) {
                *entry = entry.sub(&factor.mul(&pivot));
            }
        }
        pivot_columns.push(column);
        pivot_row += 1;
        if pivot_row == matrix.rows {
            break;
        }
    }
    RrefResult {
        matrix: RationalMatrix::new(data).expect("row operations preserve shape"),
        pivot_columns,
    }
}

#[must_use]
pub fn rank(matrix: &RationalMatrix) -> usize {
    rref(matrix).pivot_columns.len()
}

#[must_use]
pub fn nullspace(matrix: &RationalMatrix) -> Vec<Vec<Rational>> {
    let reduced = rref(matrix);
    nullspace_from_rref(&reduced, matrix.columns)
}

pub fn solve(
    matrix: &RationalMatrix,
    rhs: &[Rational],
) -> Result<LinearSystemSolution, MatrixError> {
    if rhs.len() != matrix.rows {
        return Err(MatrixError::DimensionMismatch);
    }
    let augmented = RationalMatrix::new(
        matrix
            .data
            .iter()
            .zip(rhs)
            .map(|(row, value)| {
                let mut output = row.clone();
                output.push(value.clone());
                output
            })
            .collect(),
    )?;
    let reduced = rref(&augmented);
    for row in reduced.matrix.data() {
        if row[..matrix.columns].iter().all(Rational::is_zero) && !row[matrix.columns].is_zero() {
            return Ok(LinearSystemSolution::None);
        }
    }
    let coefficient_pivots = reduced
        .pivot_columns
        .iter()
        .copied()
        .filter(|&column| column < matrix.columns)
        .collect::<Vec<_>>();
    let mut particular = vec![Rational::from_i64(0); matrix.columns];
    for (row, &pivot) in coefficient_pivots.iter().enumerate() {
        particular[pivot] = reduced.matrix.data[row][matrix.columns].clone();
    }
    if coefficient_pivots.len() == matrix.columns {
        return Ok(LinearSystemSolution::Unique(particular));
    }
    let coefficient_matrix = RationalMatrix::new(
        reduced
            .matrix
            .data
            .iter()
            .map(|row| row[..matrix.columns].to_vec())
            .collect(),
    )?;
    let coefficient_rref = RrefResult {
        matrix: coefficient_matrix,
        pivot_columns: coefficient_pivots,
    };
    Ok(LinearSystemSolution::Infinite {
        particular,
        nullspace_basis: nullspace_from_rref(&coefficient_rref, matrix.columns),
    })
}

pub fn determinant_bareiss(matrix: &[Vec<BigInt>]) -> Result<BigInt, MatrixError> {
    let size = matrix.len();
    if size == 0 {
        return Err(MatrixError::Empty);
    }
    if matrix.iter().any(|row| row.len() != size) {
        return Err(MatrixError::NotSquare);
    }
    if size == 1 {
        return Ok(matrix[0][0].clone());
    }
    let mut data = matrix.to_vec();
    let mut previous = BigInt::from(1);
    let mut sign = BigInt::from(1);
    for pivot_index in 0..size - 1 {
        let Some(selected) =
            (pivot_index..size).find(|&row| data[row][pivot_index] != BigInt::from(0))
        else {
            return Ok(BigInt::from(0));
        };
        if selected != pivot_index {
            data.swap(selected, pivot_index);
            sign = -sign;
        }
        let pivot = data[pivot_index][pivot_index].clone();
        for row in pivot_index + 1..size {
            for column in pivot_index + 1..size {
                data[row][column] = (&data[row][column] * &pivot
                    - &data[row][pivot_index] * &data[pivot_index][column])
                    / &previous;
            }
            data[row][pivot_index] = BigInt::from(0);
        }
        previous = pivot;
    }
    Ok(sign * &data[size - 1][size - 1])
}

pub fn hermite_normal_form(matrix: &[Vec<BigInt>]) -> Result<Vec<Vec<BigInt>>, MatrixError> {
    validate_integer_matrix(matrix)?;
    let mut data = matrix.to_vec();
    let rows = data.len();
    let columns = data[0].len();
    let mut pivot_row = 0;
    for column in 0..columns {
        if pivot_row == rows {
            break;
        }
        let Some(selected) = (pivot_row..rows).find(|&row| data[row][column] != BigInt::from(0))
        else {
            continue;
        };
        data.swap(pivot_row, selected);
        while let Some(row) =
            (pivot_row + 1..rows).find(|&row| data[row][column] != BigInt::from(0))
        {
            let quotient = &data[row][column] / &data[pivot_row][column];
            add_row_multiple(&mut data, row, pivot_row, -quotient);
            if data[row][column] != BigInt::from(0)
                && bigint_abs(&data[row][column]) < bigint_abs(&data[pivot_row][column])
            {
                data.swap(row, pivot_row);
            }
        }
        if data[pivot_row][column] < BigInt::from(0) {
            for entry in &mut data[pivot_row] {
                *entry = -entry.clone();
            }
        }
        let pivot = data[pivot_row][column].clone();
        for row in 0..pivot_row {
            let quotient = div_floor(&data[row][column], &pivot);
            add_row_multiple(&mut data, row, pivot_row, -quotient);
        }
        pivot_row += 1;
    }
    Ok(data)
}

pub fn smith_normal_form_invariants(matrix: &[Vec<BigInt>]) -> Result<Vec<BigInt>, MatrixError> {
    validate_integer_matrix(matrix)?;
    let mut data = matrix.to_vec();
    let rows = data.len();
    let columns = data[0].len();
    let limit = rows.min(columns);
    let mut pivot = 0;
    while pivot < limit {
        let mut selected: Option<(usize, usize)> = None;
        for row in pivot..rows {
            for column in pivot..columns {
                if data[row][column] != BigInt::from(0)
                    && selected.as_ref().is_none_or(|&(best_row, best_column)| {
                        bigint_abs(&data[row][column]) < bigint_abs(&data[best_row][best_column])
                    })
                {
                    selected = Some((row, column));
                }
            }
        }
        let Some((selected_row, selected_column)) = selected else {
            break;
        };
        data.swap(pivot, selected_row);
        swap_columns(&mut data, pivot, selected_column);
        loop {
            let mut changed = false;
            for row in pivot + 1..rows {
                if data[row][pivot] != BigInt::from(0) {
                    let quotient = &data[row][pivot] / &data[pivot][pivot];
                    add_row_multiple(&mut data, row, pivot, -quotient);
                    if data[row][pivot] != BigInt::from(0) {
                        data.swap(row, pivot);
                    }
                    changed = true;
                    break;
                }
            }
            if changed {
                continue;
            }
            for column in pivot + 1..columns {
                if data[pivot][column] != BigInt::from(0) {
                    let quotient = &data[pivot][column] / &data[pivot][pivot];
                    add_column_multiple(&mut data, column, pivot, -quotient);
                    if data[pivot][column] != BigInt::from(0) {
                        swap_columns(&mut data, column, pivot);
                    }
                    changed = true;
                    break;
                }
            }
            if changed {
                continue;
            }
            let offending = (pivot + 1..rows).find_map(|row| {
                (pivot + 1..columns)
                    .find(|&column| &data[row][column] % &data[pivot][pivot] != BigInt::from(0))
                    .map(|column| (row, column))
            });
            if let Some((row, _)) = offending {
                add_row_multiple(&mut data, pivot, row, BigInt::from(1));
                continue;
            }
            break;
        }
        if data[pivot][pivot] < BigInt::from(0) {
            for entry in &mut data[pivot] {
                *entry = -entry.clone();
            }
        }
        pivot += 1;
    }
    Ok((0..limit)
        .map(|index| data[index][index].clone())
        .filter(|value| value != &BigInt::from(0))
        .collect())
}

const RATIONAL_ROOT_VALUE_LIMIT: u64 = 2_000_000;
const RATIONAL_ROOT_CANDIDATE_LIMIT: usize = 100_000;

fn rational_roots_with_multiplicity(
    polynomial: &Polynomial,
) -> (Vec<(Rational, usize)>, Option<Polynomial>, bool) {
    let mut remaining = polynomial.coefficients().to_vec();
    let mut roots = Vec::new();
    let zero = Rational::from_i64(0);
    while remaining.len() > 1 && remaining[0].is_zero() {
        record_root(&mut roots, zero.clone());
        remaining.remove(0);
    }
    if remaining.len() <= 1 {
        return (roots, None, false);
    }

    let integer_coefficients = primitive_integer_coefficients(&remaining);
    let constant = integer_coefficients
        .first()
        .expect("a nonconstant polynomial has a constant coefficient");
    let leading = integer_coefficients
        .last()
        .expect("a nonconstant polynomial has a leading coefficient");
    let Ok(numerator_divisors) = bounded_positive_divisors(&bigint_abs(constant)) else {
        return (roots, Some(Polynomial::new(remaining)), true);
    };
    let Ok(denominator_divisors) = bounded_positive_divisors(&bigint_abs(leading)) else {
        return (roots, Some(Polynomial::new(remaining)), true);
    };
    if numerator_divisors
        .len()
        .saturating_mul(denominator_divisors.len())
        .saturating_mul(2)
        > RATIONAL_ROOT_CANDIDATE_LIMIT
    {
        return (roots, Some(Polynomial::new(remaining)), true);
    }

    let mut candidates = Vec::new();
    for numerator in &numerator_divisors {
        for denominator in &denominator_divisors {
            let positive = Rational::new(numerator.clone(), denominator.clone())
                .expect("positive rational-root theorem denominator");
            let negative = positive.negated();
            if !candidates.contains(&positive) {
                candidates.push(positive);
            }
            if !candidates.contains(&negative) {
                candidates.push(negative);
            }
        }
    }
    candidates.sort_by(Rational::cmp_value);

    for candidate in candidates {
        let mut multiplicity = 0;
        while remaining.len() > 1 {
            let (quotient, remainder) = divide_by_linear(&remaining, &candidate);
            if !remainder.is_zero() {
                break;
            }
            remaining = quotient;
            multiplicity += 1;
        }
        if multiplicity > 0 {
            record_root_n(&mut roots, candidate, multiplicity);
        }
    }
    let remaining_factor = (remaining.len() > 1).then(|| Polynomial::new(remaining));
    (roots, remaining_factor, false)
}

fn record_root(roots: &mut Vec<(Rational, usize)>, value: Rational) {
    record_root_n(roots, value, 1);
}

fn record_root_n(roots: &mut Vec<(Rational, usize)>, value: Rational, multiplicity: usize) {
    if let Some((_, existing)) = roots.iter_mut().find(|(root, _)| *root == value) {
        *existing += multiplicity;
    } else {
        roots.push((value, multiplicity));
    }
}

fn divide_by_linear(coefficients: &[Rational], root: &Rational) -> (Vec<Rational>, Rational) {
    let degree = coefficients.len() - 1;
    let mut quotient = vec![Rational::from_i64(0); degree];
    quotient[degree - 1] = coefficients[degree].clone();
    for index in (1..degree).rev() {
        quotient[index - 1] = coefficients[index].add(&root.mul(&quotient[index]));
    }
    let remainder = coefficients[0].add(&root.mul(&quotient[0]));
    (quotient, remainder)
}

fn primitive_integer_coefficients(coefficients: &[Rational]) -> Vec<BigInt> {
    let common_denominator = coefficients
        .iter()
        .fold(BigInt::from(1), |value, coefficient| {
            bigint_lcm(value, coefficient.denominator().clone())
        });
    let mut integers = coefficients
        .iter()
        .map(|coefficient| {
            coefficient.numerator() * (&common_denominator / coefficient.denominator())
        })
        .collect::<Vec<_>>();
    let content = integers.iter().fold(BigInt::from(0), |value, coefficient| {
        bigint_gcd(value, bigint_abs(coefficient))
    });
    if content != BigInt::from(0) && content != BigInt::from(1) {
        for coefficient in &mut integers {
            *coefficient /= &content;
        }
    }
    if integers
        .last()
        .is_some_and(|coefficient| coefficient < &BigInt::from(0))
    {
        integers
            .iter_mut()
            .for_each(|coefficient| *coefficient = -coefficient.clone());
    }
    integers
}

fn bounded_positive_divisors(value: &BigInt) -> Result<Vec<BigInt>, ()> {
    let value = value.to_string().parse::<u64>().map_err(|_| ())?;
    if value == 0 || value > RATIONAL_ROOT_VALUE_LIMIT {
        return Err(());
    }
    let mut divisors = Vec::new();
    let mut divisor = 1_u64;
    while divisor <= value / divisor {
        if value % divisor == 0 {
            divisors.push(BigInt::from(divisor));
            let paired = value / divisor;
            if paired != divisor {
                divisors.push(BigInt::from(paired));
            }
        }
        divisor += 1;
    }
    Ok(divisors)
}

fn flatten_matrix(matrix: &RationalMatrix) -> Vec<Rational> {
    matrix
        .data
        .iter()
        .flat_map(|row| row.iter().cloned())
        .collect()
}

fn flatten_power_columns(powers: &[RationalMatrix]) -> Vec<Vec<Rational>> {
    let size = powers[0].rows;
    let columns = powers.len();
    (0..size * size)
        .map(|index| {
            let row = index / size;
            let column = index % size;
            powers
                .iter()
                .map(|power| power.data[row][column].clone())
                .take(columns)
                .collect()
        })
        .collect()
}

fn nullspace_from_rref(reduced: &RrefResult, columns: usize) -> Vec<Vec<Rational>> {
    let free_columns = (0..columns)
        .filter(|column| !reduced.pivot_columns.contains(column))
        .collect::<Vec<_>>();
    free_columns
        .into_iter()
        .map(|free| {
            let mut vector = vec![Rational::from_i64(0); columns];
            vector[free] = Rational::from_i64(1);
            for (row, &pivot) in reduced.pivot_columns.iter().enumerate() {
                vector[pivot] = reduced.matrix.data[row][free].negated();
            }
            vector
        })
        .collect()
}

fn validate_integer_matrix(matrix: &[Vec<BigInt>]) -> Result<(), MatrixError> {
    if matrix.is_empty() || matrix[0].is_empty() {
        return Err(MatrixError::Empty);
    }
    if matrix.iter().any(|row| row.len() != matrix[0].len()) {
        return Err(MatrixError::Ragged);
    }
    Ok(())
}

fn add_row_multiple(matrix: &mut [Vec<BigInt>], target: usize, source: usize, factor: BigInt) {
    let source_row = matrix[source].clone();
    for (entry, source_entry) in matrix[target].iter_mut().zip(source_row) {
        *entry += &factor * source_entry;
    }
}

fn swap_columns(matrix: &mut [Vec<BigInt>], left: usize, right: usize) {
    for row in matrix {
        row.swap(left, right);
    }
}

fn add_column_multiple(matrix: &mut [Vec<BigInt>], target: usize, source: usize, factor: BigInt) {
    for row in matrix {
        let value = row[source].clone();
        row[target] += &factor * value;
    }
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0) {
        -value
    } else {
        value.clone()
    }
}

fn bigint_gcd(mut left: BigInt, mut right: BigInt) -> BigInt {
    left = bigint_abs(&left);
    right = bigint_abs(&right);
    while right != BigInt::from(0) {
        let remainder = left % &right;
        left = right;
        right = remainder;
    }
    left
}

fn bigint_lcm(left: BigInt, right: BigInt) -> BigInt {
    if left == BigInt::from(0) || right == BigInt::from(0) {
        return BigInt::from(0);
    }
    bigint_abs(&((left.clone() / bigint_gcd(left, right.clone())) * right))
}

fn div_floor(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder < BigInt::from(0) {
        quotient - 1
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dot(row: &[Rational], vector: &[Rational]) -> Rational {
        row.iter()
            .zip(vector)
            .fold(Rational::from_i64(0), |sum, (left, right)| {
                sum.add(&left.mul(right))
            })
    }

    fn determinant_leibniz(matrix: &[Vec<BigInt>]) -> BigInt {
        fn visit(
            matrix: &[Vec<BigInt>],
            row: usize,
            used: &mut [bool],
            product: BigInt,
            inversions: usize,
            total: &mut BigInt,
        ) {
            if row == matrix.len() {
                if inversions % 2 == 0 {
                    *total += product;
                } else {
                    *total -= product;
                }
                return;
            }
            for column in 0..matrix.len() {
                if used[column] {
                    continue;
                }
                let added_inversions = used[column + 1..].iter().filter(|used| **used).count();
                used[column] = true;
                visit(
                    matrix,
                    row + 1,
                    used,
                    product.clone() * &matrix[row][column],
                    inversions + added_inversions,
                    total,
                );
                used[column] = false;
            }
        }

        let mut total = BigInt::from(0);
        visit(
            matrix,
            0,
            &mut vec![false; matrix.len()],
            BigInt::from(1),
            0,
            &mut total,
        );
        total
    }

    #[test]
    fn exact_elimination_classifies_systems_and_nullspace() {
        let matrix = RationalMatrix::from_integers(&[vec![1, 2, 3], vec![2, 4, 6]]).unwrap();
        assert_eq!(rank(&matrix), 1);
        let basis = nullspace(&matrix);
        assert_eq!(basis.len(), 2);
        for vector in basis {
            for row in matrix.data() {
                let value = row
                    .iter()
                    .zip(&vector)
                    .fold(Rational::from_i64(0), |sum, (left, right)| {
                        sum.add(&left.mul(right))
                    });
                assert!(value.is_zero());
            }
        }
        assert!(matches!(
            solve(&matrix, &[Rational::from_i64(1), Rational::from_i64(3)]).unwrap(),
            LinearSystemSolution::None
        ));
    }

    #[test]
    fn bareiss_and_integer_normal_forms_preserve_invariants() {
        let matrix = vec![vec![2.into(), 4.into()], vec![6.into(), 8.into()]];
        assert_eq!(determinant_bareiss(&matrix).unwrap(), BigInt::from(-8));
        let smith = smith_normal_form_invariants(&matrix).unwrap();
        assert_eq!(smith, vec![BigInt::from(2), BigInt::from(4)]);
        assert_eq!(&smith[1] % &smith[0], BigInt::from(0));
        let hermite = hermite_normal_form(&matrix).unwrap();
        assert_eq!(
            bigint_abs(&determinant_bareiss(&hermite).unwrap()),
            BigInt::from(8)
        );
    }

    #[test]
    fn rref_pivots_and_solution_residuals_are_canonical() {
        let matrix =
            RationalMatrix::from_integers(&[vec![0, 2, 4, 2], vec![1, 1, 1, 3], vec![2, 4, 6, 8]])
                .unwrap();
        let reduced = rref(&matrix);
        for (pivot_row, &pivot_column) in reduced.pivot_columns.iter().enumerate() {
            assert_eq!(
                reduced.matrix.data()[pivot_row][pivot_column],
                Rational::from_i64(1)
            );
            assert!(
                reduced
                    .matrix
                    .data()
                    .iter()
                    .enumerate()
                    .all(|(row, values)| row == pivot_row || values[pivot_column].is_zero())
            );
        }

        let rhs = [
            Rational::from_i64(6),
            Rational::from_i64(6),
            Rational::from_i64(18),
        ];
        let solution = solve(&matrix, &rhs).unwrap();
        let (particular, basis) = match solution {
            LinearSystemSolution::Infinite {
                particular,
                nullspace_basis,
            } => (particular, nullspace_basis),
            other => panic!("expected an affine solution space, got {other:?}"),
        };
        for (row, expected) in matrix.data().iter().zip(&rhs) {
            assert_eq!(&dot(row, &particular), expected);
        }
        for vector in basis {
            assert!(matrix.data().iter().all(|row| dot(row, &vector).is_zero()));
        }
    }

    #[test]
    fn bareiss_matches_an_independent_leibniz_oracle() {
        let cases = [
            vec![vec![7.into()]],
            vec![vec![0.into(), 2.into()], vec![3.into(), 4.into()]],
            vec![
                vec![2.into(), (-1).into(), 3.into()],
                vec![4.into(), 0.into(), 5.into()],
                vec![7.into(), 2.into(), 1.into()],
            ],
            vec![
                vec![1.into(), 2.into(), 3.into(), 4.into()],
                vec![2.into(), 4.into(), 7.into(), 8.into()],
                vec![0.into(), 1.into(), 0.into(), 1.into()],
                vec![3.into(), 5.into(), 9.into(), 2.into()],
            ],
        ];
        for matrix in cases {
            assert_eq!(
                determinant_bareiss(&matrix).unwrap(),
                determinant_leibniz(&matrix)
            );
        }
    }

    #[test]
    fn hermite_and_smith_outputs_satisfy_structural_invariants() {
        let matrix = vec![
            vec![4.into(), 6.into(), 2.into()],
            vec![2.into(), 8.into(), 4.into()],
            vec![6.into(), 10.into(), 8.into()],
        ];
        let hermite = hermite_normal_form(&matrix).unwrap();
        let mut previous_pivot = None;
        for (row_index, row) in hermite.iter().enumerate() {
            let pivot = row.iter().position(|value| value != &BigInt::from(0));
            if let Some(column) = pivot {
                assert!(previous_pivot.is_none_or(|previous| column > previous));
                assert!(row[column] > BigInt::from(0));
                for earlier in &hermite[..row_index] {
                    assert!(earlier[column] >= BigInt::from(0));
                    assert!(earlier[column] < row[column]);
                }
                previous_pivot = Some(column);
            }
        }
        assert_eq!(
            bigint_abs(&determinant_bareiss(&hermite).unwrap()),
            bigint_abs(&determinant_bareiss(&matrix).unwrap())
        );

        let invariants = smith_normal_form_invariants(&matrix).unwrap();
        assert!(invariants.iter().all(|value| value > &BigInt::from(0)));
        assert!(
            invariants
                .windows(2)
                .all(|pair| &pair[1] % &pair[0] == BigInt::from(0))
        );
        let product = invariants
            .iter()
            .fold(BigInt::from(1), |value, item| value * item);
        assert_eq!(product, bigint_abs(&determinant_bareiss(&matrix).unwrap()));
    }

    #[test]
    fn rational_matrix_operations_are_exact() {
        let matrix = RationalMatrix::from_integers(&[vec![1, 2], vec![3, 4]]).unwrap();
        assert_eq!(
            matrix.transpose().data(),
            &[
                vec![Rational::from_i64(1), Rational::from_i64(3)],
                vec![Rational::from_i64(2), Rational::from_i64(4)],
            ]
        );
        assert_eq!(matrix.trace().unwrap(), Rational::from_i64(5));
        assert_eq!(matrix.determinant().unwrap(), Rational::from_i64(-2));
        assert_eq!(
            matrix.power(0).unwrap(),
            RationalMatrix::identity(2).unwrap()
        );
        assert_eq!(
            matrix.power(2).unwrap().data(),
            &[
                vec![Rational::from_i64(7), Rational::from_i64(10)],
                vec![Rational::from_i64(15), Rational::from_i64(22)],
            ]
        );
        let inverse = matrix.inverse().unwrap();
        assert_eq!(
            inverse.data(),
            &[
                vec![
                    Rational::new(BigInt::from(-2), BigInt::from(1)).unwrap(),
                    Rational::new(BigInt::from(1), BigInt::from(1)).unwrap(),
                ],
                vec![
                    Rational::new(BigInt::from(3), BigInt::from(2)).unwrap(),
                    Rational::new(BigInt::from(-1), BigInt::from(2)).unwrap(),
                ],
            ]
        );
        assert_eq!(
            matrix.mul(&inverse).unwrap(),
            RationalMatrix::identity(2).unwrap()
        );
    }

    #[test]
    fn characteristic_eigen_and_diagonalization_results_are_classified() {
        let diagonal = RationalMatrix::from_integers(&[vec![2, 0], vec![0, 3]]).unwrap();
        let characteristic = diagonal.characteristic_polynomial().unwrap();
        assert_eq!(
            characteristic
                .coefficients()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["6", "-5", "1"]
        );
        let analysis = diagonal.eigenvalue_analysis().unwrap();
        assert!(!analysis.search_limited);
        assert!(analysis.remaining_factor.is_none());
        assert_eq!(
            analysis
                .eigenvalues
                .iter()
                .map(|value| (
                    value.value.to_string(),
                    value.algebraic_multiplicity,
                    value.geometric_multiplicity
                ))
                .collect::<Vec<_>>(),
            vec![("2".to_owned(), 1, 1), ("3".to_owned(), 1, 1)]
        );
        let result = diagonal.diagonalize().unwrap();
        let (p, d, inverse) = match result {
            DiagonalizationResult::Diagonalizable { p, d, inverse, .. } => (p, d, inverse),
            other => panic!("expected a diagonalization, got {other:?}"),
        };
        assert_eq!(p.mul(&d).unwrap(), diagonal.mul(&p).unwrap());
        assert_eq!(
            p.mul(&inverse).unwrap(),
            RationalMatrix::identity(2).unwrap()
        );

        let jordan = RationalMatrix::from_integers(&[vec![2, 1], vec![0, 2]]).unwrap();
        assert!(matches!(
            jordan.diagonalize().unwrap(),
            DiagonalizationResult::NotDiagonalizable { .. }
        ));
        let rotation = RationalMatrix::from_integers(&[vec![0, -1], vec![1, 0]]).unwrap();
        assert!(matches!(
            rotation.diagonalize().unwrap(),
            DiagonalizationResult::NotSplit { .. }
        ));
    }

    #[test]
    fn minimal_polynomial_and_lu_preserve_matrix_identities() {
        let matrix = RationalMatrix::from_integers(&[vec![1, 2], vec![3, 4]]).unwrap();
        let minimal = matrix.minimal_polynomial().unwrap();
        assert_eq!(
            minimal
                .coefficients()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["-2", "-5", "1"]
        );
        let lu = matrix.lu_decomposition().unwrap();
        assert!(!lu.singular);
        assert_eq!(
            lu.permutation.mul(&matrix).unwrap(),
            lu.lower.mul(&lu.upper).unwrap()
        );
    }
}
