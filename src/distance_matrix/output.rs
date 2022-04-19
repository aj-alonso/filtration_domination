//! Utilities to save distance matrices to disk.

use std::fmt::Display;
use std::io;
use std::io::Write;

use crate::distance_matrix::{Distance, DistanceMatrix};

pub fn write_lower_triangular_distance_matrix<T: Distance + Display, W: Write>(
    distance_matrix: &DistanceMatrix<T>,
    writer: &mut W,
) -> io::Result<()> {
    let n_vertices = distance_matrix.len();

    for u in 0..n_vertices {
        for v in 0..=u {
            if v != 0 {
                write!(writer, " ")?;
            }
            write!(writer, "{}", distance_matrix.get(u, v))?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

/// Write a full distance matrix (that is, a square matrix of size equal to the number of vertices).
pub fn write_distance_matrix<T: Distance + Display, W: Write>(
    distance_matrix: &DistanceMatrix<T>,
    writer: &mut W,
) -> io::Result<()> {
    let n_vertices = distance_matrix.len();

    for u in 0..n_vertices {
        for v in 0..n_vertices {
            if v != 0 {
                write!(writer, " ")?;
            }
            write!(writer, "{}", distance_matrix.get(u, v))?;
        }
        writeln!(writer)?;
    }

    Ok(())
}
