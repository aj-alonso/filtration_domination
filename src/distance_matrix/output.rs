//! Utilities to save distance matrices to disk.

use std::fmt::Display;
use std::io;
use std::io::Write;

use crate::distance_matrix::{Distance, DistanceMatrix};
use crate::edges::{EdgeList, FilteredEdge};
use crate::OneCriticalGrade;
use crate::Value;

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

pub fn write_edge_list<T: Value + Display, W: Write, const N: usize>(
    edges: &EdgeList<FilteredEdge<OneCriticalGrade<T, N>>>,
    writer: &mut W,
) -> io::Result<()> {
    writeln!(writer, "{}", edges.len())?;

    for e in edges.edge_iter() {
        write!(writer, "{} {}", e.edge.0, e.edge.1)?;
        for i in 0..N {
            write!(writer, " {}", e.grade.0[i])?;
        }
        writeln!(writer)?;
    }

    Ok(())
}
