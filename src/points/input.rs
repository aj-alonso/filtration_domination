//! Utilities to read point clouds from disk.
use num::Float;
use std::fmt::Display;
use std::io;
use std::io::BufRead;
use std::str::FromStr;

use crate::io_utils::parse_next;
use crate::points::{Point, PointCloud};

/// Read a point cloud from the given reader.
pub fn read_point_cloud<T: Float + FromStr + Display, R: BufRead, const N: usize>(
    r: R,
) -> Result<PointCloud<T, N>, io::Error>
where
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    let mut points = Vec::new();
    let lines: Vec<String> = r.lines().collect::<io::Result<Vec<String>>>()?;
    for mut line in lines {
        remove_whitespace(&mut line);
        let mut coords = line.splitn(N, ',');
        let mut values = [T::zero(); N];

        for i in 0..N {
            values[i] = parse_next(&mut coords)?;
        }
        points.push(Point(values));
    }

    Ok(PointCloud(points))
}

fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::points::input::read_point_cloud;
    use crate::points::{Point, PointCloud};

    #[test]
    fn read_point_cloud_happy_case() {
        let s = "1.57, 2.40\n\
                      1.21, -2.70";
        let points: PointCloud<f64, 2> = read_point_cloud(BufReader::new(s.as_bytes())).unwrap();
        assert_eq!(points.0, [Point([1.57, 2.40]), Point([1.21, -2.7])]);
    }
}
