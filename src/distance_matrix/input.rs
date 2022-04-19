//! Utilities to read graphs and distance matrices from files.
use num::Zero;
use std::fmt::Display;
use std::io;
use std::io::BufRead;
use std::str::FromStr;

use crate::distance_matrix::DistanceMatrix;
use crate::io_utils::parse;

/// Read a space separated lower triangular distance matrix.
/// It can also be used to read a full distance matrix.
pub fn read_lower_triangular_distance_matrix<T: Zero + Clone + FromStr + Display, R: BufRead>(
    r: R,
) -> io::Result<DistanceMatrix<T>>
where
    <T as FromStr>::Err: std::error::Error + 'static + Send + Sync,
{
    let lines: Vec<String> = r.lines().collect::<io::Result<_>>()?;

    let mut matrix = DistanceMatrix::new(lines.len());
    for (u, line) in lines.into_iter().enumerate() {
        for (v, d) in line.split_whitespace().enumerate() {
            if v > u {
                break;
            }
            matrix.set(u, v, parse(d)?);
        }
    }

    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::distance_matrix::input::read_lower_triangular_distance_matrix;
    use crate::distance_matrix::DistanceMatrix;

    #[test]
    fn read_lower_triangular_distance_matrix_happy_case() {
        let s = "0
                      0.1 0
                      123. 456.2112 0";
        let matrix: DistanceMatrix<f64> =
            read_lower_triangular_distance_matrix(BufReader::new(s.as_bytes())).unwrap();
        assert_eq!(*matrix.get(0, 0), 0.);
        assert_eq!(*matrix.get(1, 0), 0.1);
        assert_eq!(*matrix.get(2, 0), 123.);
        assert_eq!(*matrix.get(2, 1), 456.2112);
    }

    #[test]
    fn read_distance_matrix_happy_case() {
        let s = "0 0.1 123.
                      0.1 0 456.2112
                      123. 456.2112 0";
        let matrix: DistanceMatrix<f64> =
            read_lower_triangular_distance_matrix(BufReader::new(s.as_bytes())).unwrap();
        assert_eq!(*matrix.get(0, 0), 0.);
        assert_eq!(*matrix.get(1, 0), 0.1);
        assert_eq!(*matrix.get(2, 0), 123.);
        assert_eq!(*matrix.get(2, 1), 456.2112);
    }
}
