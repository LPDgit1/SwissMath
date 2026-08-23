use crate::{FiniteFieldError, PrimeField};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FpMatrix {
    rows: usize,
    columns: usize,
    data: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FpRrefResult {
    pub matrix: FpMatrix,
    pub pivot_columns: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FpLinearSystemSolution {
    None,
    Unique(Vec<u64>),
    Infinite {
        particular: Vec<u64>,
        kernel_basis: Vec<Vec<u64>>,
    },
}

impl FpMatrix {
    pub fn new(field: PrimeField, rows: &[Vec<i128>]) -> Result<Self, FiniteFieldError> {
        if rows.is_empty() || rows[0].is_empty() {
            return Err(FiniteFieldError::Empty);
        }
        let columns = rows[0].len();
        if rows.iter().any(|row| row.len() != columns) {
            return Err(FiniteFieldError::Ragged);
        }
        Ok(Self {
            rows: rows.len(),
            columns,
            data: rows
                .iter()
                .flat_map(|row| row.iter().map(|&value| field.normalize(value)))
                .collect(),
        })
    }

    fn canonical(rows: usize, columns: usize, data: Vec<u64>) -> Self {
        debug_assert_eq!(data.len(), rows * columns);
        Self {
            rows,
            columns,
            data,
        }
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
    pub fn data(&self) -> &[u64] {
        &self.data
    }

    #[must_use]
    pub fn to_rows(&self) -> Vec<Vec<u64>> {
        self.data
            .chunks(self.columns)
            .map(<[u64]>::to_vec)
            .collect()
    }

    pub fn add(&self, field: PrimeField, other: &Self) -> Result<Self, FiniteFieldError> {
        self.same_shape(other)?;
        Ok(Self::canonical(
            self.rows,
            self.columns,
            self.data
                .iter()
                .zip(&other.data)
                .map(|(&left, &right)| field.add(left, right))
                .collect(),
        ))
    }

    pub fn sub(&self, field: PrimeField, other: &Self) -> Result<Self, FiniteFieldError> {
        self.same_shape(other)?;
        Ok(Self::canonical(
            self.rows,
            self.columns,
            self.data
                .iter()
                .zip(&other.data)
                .map(|(&left, &right)| field.sub(left, right))
                .collect(),
        ))
    }

    pub fn mul(&self, field: PrimeField, other: &Self) -> Result<Self, FiniteFieldError> {
        if self.columns != other.rows {
            return Err(FiniteFieldError::DimensionMismatch);
        }
        let mut output = vec![0; self.rows * other.columns];
        for row in 0..self.rows {
            for inner in 0..self.columns {
                let left = self[(row, inner)];
                if left == 0 {
                    continue;
                }
                for column in 0..other.columns {
                    let target = row * other.columns + column;
                    output[target] =
                        field.add(output[target], field.mul(left, other[(inner, column)]));
                }
            }
        }
        Ok(Self::canonical(self.rows, other.columns, output))
    }

    pub fn mul_vector(
        &self,
        field: PrimeField,
        vector: &[i128],
    ) -> Result<Vec<u64>, FiniteFieldError> {
        if self.columns != vector.len() {
            return Err(FiniteFieldError::DimensionMismatch);
        }
        let vector = vector
            .iter()
            .map(|&value| field.normalize(value))
            .collect::<Vec<_>>();
        Ok((0..self.rows)
            .map(|row| {
                (0..self.columns).fold(0, |sum, column| {
                    field.add(sum, field.mul(self[(row, column)], vector[column]))
                })
            })
            .collect())
    }

    #[must_use]
    pub fn rref(&self, field: PrimeField) -> FpRrefResult {
        let (data, pivots) = eliminate(
            field,
            self.data.clone(),
            self.rows,
            self.columns,
            self.columns,
        );
        FpRrefResult {
            matrix: Self::canonical(self.rows, self.columns, data),
            pivot_columns: pivots,
        }
    }

    #[must_use]
    pub fn rank(&self, field: PrimeField) -> usize {
        self.rref(field).pivot_columns.len()
    }

    pub fn determinant(&self, field: PrimeField) -> Result<u64, FiniteFieldError> {
        if self.rows != self.columns {
            return Err(FiniteFieldError::NotSquare);
        }
        let mut data = self.data.clone();
        let mut determinant = 1;
        for column in 0..self.columns {
            let Some(pivot) =
                (column..self.rows).find(|&row| data[row * self.columns + column] != 0)
            else {
                return Ok(0);
            };
            if pivot != column {
                swap_rows(&mut data, self.columns, pivot, column);
                determinant = field.sub(0, determinant);
            }
            let pivot_value = data[column * self.columns + column];
            determinant = field.mul(determinant, pivot_value);
            let inverse = field
                .inverse(pivot_value)
                .expect("a nonzero field value is invertible");
            for row in column + 1..self.rows {
                let factor = field.mul(data[row * self.columns + column], inverse);
                for target_column in column..self.columns {
                    let target = row * self.columns + target_column;
                    let source = column * self.columns + target_column;
                    data[target] = field.sub(data[target], field.mul(factor, data[source]));
                }
            }
        }
        Ok(determinant)
    }

    pub fn solve(
        &self,
        field: PrimeField,
        rhs: &[i128],
    ) -> Result<FpLinearSystemSolution, FiniteFieldError> {
        if rhs.len() != self.rows {
            return Err(FiniteFieldError::DimensionMismatch);
        }
        let augmented_columns = self.columns + 1;
        let mut augmented = Vec::with_capacity(self.rows * augmented_columns);
        for (row, &value) in rhs.iter().enumerate() {
            augmented.extend_from_slice(&self.data[row * self.columns..(row + 1) * self.columns]);
            augmented.push(field.normalize(value));
        }
        let (reduced, pivots) =
            eliminate(field, augmented, self.rows, augmented_columns, self.columns);
        for row in 0..self.rows {
            let offset = row * augmented_columns;
            if reduced[offset..offset + self.columns]
                .iter()
                .all(|&value| value == 0)
                && reduced[offset + self.columns] != 0
            {
                return Ok(FpLinearSystemSolution::None);
            }
        }
        let mut particular = vec![0; self.columns];
        for (row, &pivot) in pivots.iter().enumerate() {
            particular[pivot] = reduced[row * augmented_columns + self.columns];
        }
        if pivots.len() == self.columns {
            return Ok(FpLinearSystemSolution::Unique(particular));
        }
        let coefficient_data = (0..self.rows)
            .flat_map(|row| {
                reduced[row * augmented_columns..row * augmented_columns + self.columns]
                    .iter()
                    .copied()
            })
            .collect();
        let coefficient_rref = FpRrefResult {
            matrix: Self::canonical(self.rows, self.columns, coefficient_data),
            pivot_columns: pivots,
        };
        Ok(FpLinearSystemSolution::Infinite {
            particular,
            kernel_basis: kernel_from_rref(field, &coefficient_rref),
        })
    }

    #[must_use]
    pub fn kernel(&self, field: PrimeField) -> Vec<Vec<u64>> {
        kernel_from_rref(field, &self.rref(field))
    }

    pub fn inverse(&self, field: PrimeField) -> Result<Self, FiniteFieldError> {
        if self.rows != self.columns {
            return Err(FiniteFieldError::NotSquare);
        }
        let width = self.columns * 2;
        let mut augmented = vec![0; self.rows * width];
        for row in 0..self.rows {
            for column in 0..self.columns {
                augmented[row * width + column] = self[(row, column)];
            }
            augmented[row * width + self.columns + row] = 1;
        }
        let (reduced, pivots) = eliminate(field, augmented, self.rows, width, self.columns);
        if pivots.len() != self.columns {
            return Err(FiniteFieldError::Singular);
        }
        let data = (0..self.rows)
            .flat_map(|row| {
                reduced[row * width + self.columns..(row + 1) * width]
                    .iter()
                    .copied()
            })
            .collect();
        Ok(Self::canonical(self.rows, self.columns, data))
    }

    fn same_shape(&self, other: &Self) -> Result<(), FiniteFieldError> {
        if self.rows == other.rows && self.columns == other.columns {
            Ok(())
        } else {
            Err(FiniteFieldError::DimensionMismatch)
        }
    }
}

impl std::ops::Index<(usize, usize)> for FpMatrix {
    type Output = u64;

    fn index(&self, (row, column): (usize, usize)) -> &Self::Output {
        &self.data[row * self.columns + column]
    }
}

fn eliminate(
    field: PrimeField,
    mut data: Vec<u64>,
    rows: usize,
    columns: usize,
    pivot_limit: usize,
) -> (Vec<u64>, Vec<usize>) {
    let mut pivot_row = 0;
    let mut pivots = Vec::new();
    for column in 0..pivot_limit {
        let Some(selected) = (pivot_row..rows).find(|&row| data[row * columns + column] != 0)
        else {
            continue;
        };
        swap_rows(&mut data, columns, pivot_row, selected);
        let inverse = field
            .inverse(data[pivot_row * columns + column])
            .expect("a nonzero field value is invertible");
        for target_column in column..columns {
            let target = pivot_row * columns + target_column;
            data[target] = field.mul(data[target], inverse);
        }
        for row in 0..rows {
            if row == pivot_row {
                continue;
            }
            let factor = data[row * columns + column];
            if factor == 0 {
                continue;
            }
            for target_column in column..columns {
                let target = row * columns + target_column;
                let source = pivot_row * columns + target_column;
                data[target] = field.sub(data[target], field.mul(factor, data[source]));
            }
        }
        pivots.push(column);
        pivot_row += 1;
        if pivot_row == rows {
            break;
        }
    }
    (data, pivots)
}

fn swap_rows(data: &mut [u64], columns: usize, left: usize, right: usize) {
    if left == right {
        return;
    }
    for column in 0..columns {
        data.swap(left * columns + column, right * columns + column);
    }
}

fn kernel_from_rref(field: PrimeField, reduced: &FpRrefResult) -> Vec<Vec<u64>> {
    (0..reduced.matrix.columns)
        .filter(|column| !reduced.pivot_columns.contains(column))
        .map(|free| {
            let mut vector = vec![0; reduced.matrix.columns];
            vector[free] = 1;
            for (row, &pivot) in reduced.pivot_columns.iter().enumerate() {
                vector[pivot] = field.sub(0, reduced.matrix[(row, free)]);
            }
            vector
        })
        .collect()
}
