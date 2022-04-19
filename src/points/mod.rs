//! Point clouds: create and modify them.
use num::Float;
use ordered_float::OrderedFloat;
use rand::distributions::Distribution;
use rand::Rng;
use std::fmt::Formatter;

use crate::distance_matrix::DistanceMatrix;

pub mod input;
pub mod output;

/// A point in `R^N`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Point<T, const N: usize>(pub [T; N]);

impl<T: Float, const N: usize> Point<T, N> {
    /// Computes the Euclidean distance between the given points.
    pub fn euclidean_distance(&self, other: &Point<T, N>) -> T {
        let mut d = T::zero();
        for i in 0..N {
            d = d + (self.0[i] - other.0[i]).powi(2);
        }
        d.sqrt()
    }

    /// Computes the norm of the point.
    pub fn norm(&self) -> T {
        let mut d = T::zero();
        for i in 0..N {
            d = d + (self.0[i]).powi(2);
        }
        d.sqrt()
    }

    /// Compute the unit vector of the same direction.
    pub fn normalize(&mut self) {
        let norm = self.norm();
        for i in 0..N {
            self.0[i] = self.0[i] / norm;
        }
    }

    /// Sample a random point, coordinate by coordinate.
    pub fn random<D: Distribution<T>, R: Rng>(distribution: &D, rng: &mut R) -> Point<T, N> {
        let mut p = Point([T::zero(); N]);
        for x in p.0.iter_mut() {
            *x = rng.sample(distribution);
        }
        p
    }
}

impl<T: Float + std::fmt::Display, const N: usize> std::fmt::Display for Point<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for i in 0..N {
            write!(f, "{}", self.0[i])?;
            if i != N - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl<T: Float, const N: usize> std::ops::Sub for Point<T, N> {
    type Output = Point<T, N>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut point = Point([T::zero(); N]);
        for i in 0..N {
            point.0[i] = self.0[i] - rhs.0[i];
        }
        point
    }
}

impl<T: Float, S: Copy, const N: usize> From<[S; N]> for Point<T, N>
where
    S: Into<T>,
{
    fn from(x: [S; N]) -> Self {
        let mut point = Point([T::zero(); N]);
        for (i, &x_i) in x.iter().enumerate() {
            point.0[i] = x_i.into();
        }
        point
    }
}

/// A collection of points.
pub struct PointCloud<T: Float, const N: usize>(pub Vec<Point<T, N>>);

impl<T: Float, const N: usize> Default for PointCloud<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Float, const N: usize> PointCloud<T, N> {
    /// Create a new empty point cloud.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a new point.
    pub fn push_point(&mut self, p: Point<T, N>) {
        self.0.push(p)
    }

    /// Return the distance matrix of the point cloud, where the order is the order in which the
    /// points where added.
    pub fn distance_matrix(&self) -> DistanceMatrix<T> {
        let n = self.len();
        let mut matrix = DistanceMatrix::new(n);
        for u in 0..n {
            for v in (u + 1)..n {
                matrix.set(u, v, self.0[u].euclidean_distance(&self.0[v]))
            }
        }
        matrix
    }

    /// Returns the number of points in the point cloud.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the point cloud has no points.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<const N: usize> From<PointCloud<f64, N>> for PointCloud<OrderedFloat<f64>, N> {
    fn from(points: PointCloud<f64, N>) -> Self {
        let mut result: PointCloud<OrderedFloat<f64>, N> = PointCloud::new();
        for p in points.0.into_iter() {
            result.push_point(p.0.try_into().unwrap());
        }
        result
    }
}
