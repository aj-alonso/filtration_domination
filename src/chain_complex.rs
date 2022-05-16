use std::io;

use crate::{CriticalGrade, OneCriticalGrade, Value};

/// A column with coefficients in Z2.
#[derive(Debug, Clone)]
pub struct Column {
    /// Position of the non-zero entries of the column.
    non_zeros: Vec<usize>,
}

impl Column {
    pub fn new_empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn new(non_zeros: Vec<usize>) -> Self {
        Self { non_zeros }
    }
}

impl<const N: usize> From<[usize; N]> for Column {
    fn from(c: [usize; N]) -> Self {
        Self::new(Vec::from(c))
    }
}

/// A column matrix with coefficients in Z2.
#[derive(Debug)]
pub struct ColumnMatrix {
    columns: Vec<Column>,
}

impl ColumnMatrix {
    pub fn new_empty(n_columns: usize) -> Self {
        Self::new(vec![Column::new_empty(); n_columns])
    }

    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns }
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column)
    }

    fn n_cols(&self) -> usize {
        self.columns.len()
    }
}

impl<const N: usize, const M: usize> From<[[usize; N]; M]> for ColumnMatrix {
    fn from(columns: [[usize; N]; M]) -> Self {
        Self {
            columns: Vec::from(columns.map(|c| c.into())),
        }
    }
}

/// A column matrix with a graded associated to each column.
/// The matrix has Z2 coefficients.
#[derive(Debug)]
pub struct GradedMatrix<VF: Value, const N: usize> {
    grades: Vec<OneCriticalGrade<VF, N>>,
    matrix: ColumnMatrix,
}

impl<VF: Value, const N: usize> GradedMatrix<VF, N> {
    pub fn new_empty(n_columns: usize) -> Self {
        Self::new(
            ColumnMatrix::new_empty(n_columns),
            vec![OneCriticalGrade::min_value(); n_columns],
        )
    }

    pub fn new(matrix: ColumnMatrix, grades: Vec<OneCriticalGrade<VF, N>>) -> Self {
        assert_eq!(
            matrix.n_cols(),
            grades.len(),
            "The number of grades must be equal to the number of columns."
        );
        Self { grades, matrix }
    }

    pub fn add_column(&mut self, grade: OneCriticalGrade<VF, N>, column: Column) {
        self.grades.push(grade);
        self.matrix.add_column(column);
    }

    fn n_cols(&self) -> usize {
        self.matrix.n_cols()
    }

    fn iter(&self) -> impl Iterator<Item = (&OneCriticalGrade<VF, N>, &Column)> {
        let column_iter = self.matrix.columns.iter();
        let grades_iter = self.grades.iter();
        Iterator::zip(grades_iter, column_iter)
    }
}

/// A chain complex, a sequence of graded matrices representing free persistence modules.
#[derive(Debug)]
pub struct ChainComplex<VF: Value, const N: usize> {
    matrices: Vec<GradedMatrix<VF, N>>,
}

impl<VF: Value, const N: usize> ChainComplex<VF, N> {
    pub fn new(matrices: Vec<GradedMatrix<VF, N>>) -> Self {
        Self { matrices }
    }
}

impl<VF: Value, const N: usize> ChainComplex<VF, N> {
    pub fn write_scc2020<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        writeln!(w, "scc2020")?;
        writeln!(w, "{}", N)?;

        for (idx, m) in self.matrices.iter().enumerate() {
            write!(w, "{}", m.n_cols())?;
            if idx != self.matrices.len() - 1 {
                write!(w, " ")?;
            }
        }
        writeln!(w)?;

        for (idx_matrix, graded_matrix) in self.matrices.iter().enumerate() {
            // We do not need a description of the generators of the last matrix to calculate homology.
            if idx_matrix == self.matrices.len() - 1 {
                continue;
            }
            for (grade, column) in graded_matrix.iter() {
                for v in grade.iter() {
                    write!(w, "{} ", v)?;
                }

                write!(w, ";")?;

                for c in column.non_zeros.iter() {
                    write!(w, " {}", c)?;
                }
                writeln!(w)?;
            }
            if idx_matrix != self.matrices.len() - 2 {
                writeln!(w)?;
            }
        }

        Ok(())
    }
}

pub trait ToFreeImplicitRepresentation<VF: Value, const N: usize> {
    fn to_free_implicit_representation(&self, homology: usize) -> ChainComplex<VF, N>;

    fn write_scc2020<W: std::io::Write>(&self, homology: usize, w: &mut W) -> io::Result<()> {
        let chain_complex = self.to_free_implicit_representation(homology);
        chain_complex.write_scc2020(w)
    }
}
