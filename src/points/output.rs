//! Utilities to write point clouds to disk.
use num::Float;

use crate::points::PointCloud;

/// Write the point cloud to the given writer.
pub fn write_point_cloud<T: Float + std::fmt::Display, W: std::io::Write, const N: usize>(
    cloud: &PointCloud<T, N>,
    w: &mut W,
) -> std::io::Result<()> {
    for p in cloud.0.iter() {
        for i in 0..N {
            write!(w, "{}", p.0[i])?;
            if i == N - 1 {
                writeln!(w)?;
            } else {
                write!(w, ", ")?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::points::output::write_point_cloud;
    use crate::points::{Point, PointCloud};

    #[test]
    fn write_point_cloud_happy_case() {
        let f: PointCloud<f64, 2> = PointCloud(vec![Point([2., 1.]), Point([0., -2.14])]);
        let mut buf = Vec::new();
        write_point_cloud(&f, &mut buf).unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(out, "2, 1\n0, -2.14\n")
    }
}
