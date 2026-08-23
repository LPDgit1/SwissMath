use num_bigint::BigInt;

use crate::Rational;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatrixError {
    Empty,
    Ragged,
    NotSquare,
    DimensionMismatch,
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
}
