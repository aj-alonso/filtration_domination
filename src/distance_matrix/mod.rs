//! Distance matrices: reading them, outputting them, and handling them,
//! including density estimation.
use num::Zero;
use std::cmp::max;

use crate::edges::{BareEdge, FilteredEdge};
use crate::{OneCriticalGrade, Value};

pub mod density_estimation;
pub mod input;
pub mod output;

/// Stores a distance matrix of a number of vertices.
pub struct DistanceMatrix<T> {
    // We store a lower triangular distance matrix.
    distances: Vec<Vec<T>>,
}

impl<T: Zero + Clone> DistanceMatrix<T> {
    /// Create a new distance matrix on the given number of points.
    pub fn new(n: usize) -> DistanceMatrix<T> {
        let mut distances = Vec::with_capacity(n);
        for v in 0..n {
            distances.push(vec![T::zero(); v + 1]);
        }
        DistanceMatrix { distances }
    }

    /// Set the distance between two points.
    /// Panics when u == v.
    pub fn set(&mut self, u: usize, v: usize, d: T) {
        if u == v {
            if !d.is_zero() {
                panic!("The distance between the same vertex cannot be different from zero.");
            }
        } else {
            let (new_u, new_v) = max_min(u, v);
            self.distances[new_u][new_v] = d;
        }
    }
}

impl<T> DistanceMatrix<T> {
    /// Returns the number of points.
    pub fn len(&self) -> usize {
        self.distances.len()
    }

    /// Returns whether the distance matrix is empty.
    pub fn is_empty(&self) -> bool {
        self.distances.is_empty()
    }

    /// Returns the distance between two points.
    pub fn get(&self, u: usize, v: usize) -> &T {
        let (new_u, new_v) = max_min(u, v);
        &self.distances[new_u][new_v]
    }
}

impl<T: Zero + Clone + Ord> DistanceMatrix<T> {
    /// Calculates the given percentile (from 0.0 to 1.0) of the distances.
    pub fn percentile(&self, percentile: f64) -> &T {
        let mut all_distances = Vec::with_capacity(self.len() * self.len());
        for u in 0..self.len() {
            for v in 0..u {
                all_distances.push(self.get(u, v));
            }
        }
        let pos = (all_distances.len() as f64) * percentile;
        all_distances.sort_unstable();
        all_distances[pos as usize]
    }

    /// Calculates the eccentricity (maximum distance of a vertex to any other vertex) of each vertex,
    /// in a straightforward O(n^2) way.
    pub fn eccentricity_vector(&self) -> Vec<T> {
        let mut eccentricities = Vec::with_capacity(self.len());
        for u in 0..self.len() {
            let mut u_max = T::zero();
            for v in 0..self.len() {
                u_max = max(u_max, self.get(u, v).clone());
            }
            eccentricities.push(u_max);
        }
        eccentricities
    }
}

impl<T: Value> DistanceMatrix<T> {
    /// Returns an iterator that goes through all edges on the complete graph associated to
    /// this distance matrix.
    pub fn edges(&self) -> EdgeIterator<'_, T> {
        EdgeIterator::new(self)
    }
}

/// Iterator that outputs the edges on the complete graph associated to a distance matrix.
/// See [DistanceMatrix::edges].
pub struct EdgeIterator<'a, T> {
    matrix: &'a DistanceMatrix<T>,
    current_edge: BareEdge,
}

impl<'a, T: Value> EdgeIterator<'a, T> {
    fn new(matrix: &DistanceMatrix<T>) -> EdgeIterator<T> {
        EdgeIterator {
            matrix,
            current_edge: BareEdge(0, 0),
        }
    }

    fn increment_edge(e: BareEdge) -> BareEdge {
        let BareEdge(mut u, mut v) = e;
        v += 1;
        if v > u {
            u += 1;
            v = 0;
        }
        BareEdge(u, v)
    }
}

impl<'a, T: Value> Iterator for EdgeIterator<'a, T> {
    type Item = FilteredEdge<OneCriticalGrade<T, 1>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_edge == BareEdge(self.matrix.len() - 1, self.matrix.len() - 2) {
            return None;
        }
        self.current_edge = Self::increment_edge(self.current_edge);
        if self.current_edge.0 == self.current_edge.1 {
            // If it is a self-loop, get next edge.
            self.current_edge = Self::increment_edge(self.current_edge);
        }
        Some(FilteredEdge {
            grade: OneCriticalGrade([*self.matrix.get(self.current_edge.0, self.current_edge.1)]),
            edge: self.current_edge,
        })
    }
}

fn max_min(u: usize, v: usize) -> (usize, usize) {
    (std::cmp::max(u, v), std::cmp::min(u, v))
}

#[cfg(test)]
mod tests {
    use crate::distance_matrix::DistanceMatrix;
    use crate::edges::BareEdge;
    use crate::edges::FilteredEdge;
    use crate::OneCriticalGrade;
    use ordered_float::OrderedFloat;

    #[test]
    fn edge_iterator_happy_case() {
        let mut m = DistanceMatrix::new(4);
        m.set(0, 1, (4.).into());
        m.set(0, 2, (5.).into());
        let edges: Vec<FilteredEdge<OneCriticalGrade<OrderedFloat<f64>, 1>>> = m.edges().collect();
        assert_eq!(
            edges,
            vec![
                FilteredEdge {
                    grade: OrderedFloat(4.).into(),
                    edge: BareEdge(0, 1)
                },
                FilteredEdge {
                    grade: OrderedFloat(5.).into(),
                    edge: BareEdge(0, 2)
                },
                FilteredEdge {
                    grade: OrderedFloat(0.).into(),
                    edge: BareEdge(1, 2)
                },
                FilteredEdge {
                    grade: OrderedFloat(0.).into(),
                    edge: BareEdge(0, 3)
                },
                FilteredEdge {
                    grade: OrderedFloat(0.).into(),
                    edge: BareEdge(1, 3)
                },
                FilteredEdge {
                    grade: OrderedFloat(0.).into(),
                    edge: BareEdge(2, 3)
                },
            ]
        );
    }

    #[test]
    fn test_percentile() {
        let mut m: DistanceMatrix<OrderedFloat<f64>> = DistanceMatrix::new(5);
        m.set(1, 0, 0.1.into());
        m.set(2, 0, 0.2.into());
        m.set(2, 1, 0.3.into());
        m.set(3, 0, 0.4.into());
        m.set(3, 1, 0.5.into());
        m.set(3, 2, 0.6.into());
        m.set(4, 0, 0.7.into());
        m.set(4, 1, 0.8.into());
        m.set(4, 2, 0.9.into());
        m.set(4, 3, 0.10.into());
        assert_eq!(*m.percentile(0.00), OrderedFloat(0.1));
        assert_eq!(*m.percentile(0.20), OrderedFloat(0.2));
        assert_eq!(*m.percentile(0.50), OrderedFloat(0.5));
        assert_eq!(*m.percentile(0.55), OrderedFloat(0.5));
    }
}
