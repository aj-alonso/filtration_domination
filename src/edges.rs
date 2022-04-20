//! Edges, edge lists, and associated functions.
use crate::{OneCriticalGrade, Value};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::cmp::{max, Ordering};
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

/// Common functionality of an undirected edge. See [BareEdge] and [FilteredEdge].
pub trait Edge {
    /// First endpoint. This is an undirected edge, but the first endpoint must be consistent
    /// for a fixed instance.
    fn u(&self) -> usize;

    /// Returns a mutable reference to the first endpoint.
    fn u_mut(&mut self) -> &mut usize;

    /// Second endpoint. This is an undirected edge, but the second endpoint must be consistent
    /// for a fixed instance.
    fn v(&self) -> usize;

    /// Returns a mutable reference to the second endpoint.
    fn v_mut(&mut self) -> &mut usize;

    /// The greatest endpoint.
    fn max(&self) -> usize {
        std::cmp::max(self.u(), self.v())
    }

    /// The least endpoint.
    fn min(&self) -> usize {
        std::cmp::min(self.u(), self.v())
    }

    /// Return a pair whose first element is the greatest endpoint,
    /// and the second is the least endpoint.
    fn minmax(&self) -> (usize, usize) {
        (self.min(), self.max())
    }
}

/// Edge that is not filtered.
#[derive(Debug, Clone, Copy)]
pub struct BareEdge(pub usize, pub usize);

impl Edge for BareEdge {
    fn u(&self) -> usize {
        self.0
    }

    fn u_mut(&mut self) -> &mut usize {
        &mut self.0
    }

    fn v(&self) -> usize {
        self.1
    }

    fn v_mut(&mut self) -> &mut usize {
        &mut self.1
    }
}

impl PartialEq for BareEdge {
    fn eq(&self, other: &Self) -> bool {
        self.minmax() == other.minmax()
    }
}

impl Eq for BareEdge {}

/// Lexicographic order on the minimum and maximum vertex.
impl PartialOrd for BareEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Lexicographic order on the minimum and maximum vertex.
impl Ord for BareEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.minmax().cmp(&other.minmax())
    }
}

impl Hash for BareEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.minmax().hash(state);
    }
}

impl std::fmt::Display for BareEdge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.0, self.1)
    }
}

/// An edge with its associated critical grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FilteredEdge<G> {
    /// The critical grade of this edge.
    pub grade: G,
    /// The endpoints of this edge.
    pub edge: BareEdge,
}

impl<G> Edge for FilteredEdge<G> {
    fn u(&self) -> usize {
        self.edge.u()
    }

    fn u_mut(&mut self) -> &mut usize {
        self.edge.u_mut()
    }

    fn v(&self) -> usize {
        self.edge.v()
    }

    fn v_mut(&mut self) -> &mut usize {
        self.edge.v_mut()
    }
}

/// Implements a total ordering, same as .cmp().
impl<G: Ord> PartialOrd<Self> for FilteredEdge<G> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Implements a lexicographic ordering.
/// First lexicographically compare the grades, and resolve ties by comparing edges.
impl<G: Ord> Ord for FilteredEdge<G> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.grade.cmp(&other.grade) {
            Ordering::Equal => self.edge.cmp(&other.edge),
            not_eq => not_eq,
        }
    }
}

impl<G: Ord> FilteredEdge<G> {
    /// First compare grades, by the given function `grade_cmp`,
    /// and, if they are equal, compare edge values.
    fn cmp_by(&self, other: &Self, grade_cmp: impl Fn(&G, &G) -> Ordering) -> Ordering {
        match grade_cmp(&self.grade, &other.grade) {
            Ordering::Equal => self.edge.cmp(&other.edge),
            not_eq => not_eq,
        }
    }
}

impl<G: std::fmt::Display> std::fmt::Display for FilteredEdge<G> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.edge, self.grade)?;
        Ok(())
    }
}

impl<G> From<FilteredEdge<G>> for BareEdge {
    fn from(e: FilteredEdge<G>) -> Self {
        e.edge
    }
}

/// A graph represented as a list of edges, whose vertices are in the range 0..`n_vertices`.
/// No self-loops are allowed.
#[derive(Debug, Clone)]
pub struct EdgeList<E> {
    /// Total number of vertices.
    pub n_vertices: usize,
    edges: Vec<E>,
}

impl<E: Edge> EdgeList<E> {
    /// New empty edge list.
    pub fn new(n_vertices: usize) -> Self {
        Self {
            n_vertices,
            edges: Vec::new(),
        }
    }

    /// Returns the underlying slice of edges.
    pub fn edges(&self) -> &[E] {
        &self.edges
    }

    /// Returns a mutable slice of the underlying slice of edges.
    pub fn edges_mut(&mut self) -> &mut [E] {
        &mut self.edges
    }

    /// Returns the number of edges.
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Returns whether there are edges.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Collects all the edges from the given iterator.
    pub fn from_iterator<I: Iterator<Item = E>>(it: I) -> Self {
        let edges: Vec<E> = it.collect();
        edges.into()
    }

    /// Returns the number of vertices.
    pub fn number_of_vertices(&self) -> usize {
        self.n_vertices
    }

    /// Adds an edge to the graph.
    /// Panics: if the edge to add is a self-loop.
    pub fn add_edge(&mut self, e: E) {
        let u = e.u();
        let v = e.v();
        assert_ne!(u, v, "Trying to add a self loop to a graph");

        let greatest_vertex = max(u, v);
        self.n_vertices = max(self.n_vertices, greatest_vertex + 1);
        self.edges.push(e);
    }

    /// Returns an iterator over the edges.
    pub fn edge_iter(&self) -> impl Iterator<Item = &E> + '_ {
        self.edges.iter()
    }

    /// Returns a count of the degree of each vertex.
    pub fn degrees(&self) -> Vec<usize> {
        let mut degree_count = vec![0; self.n_vertices];
        for e in self.edge_iter() {
            degree_count[e.u()] += 1;
            degree_count[e.v()] += 1;
        }
        degree_count
    }

    /// Returns the maximum degree of a vertex in the edge list.
    pub fn maximum_degree(&self) -> usize {
        // Return 0 as maximum degree if there are no vertices.
        self.degrees().into_iter().max().unwrap_or(0usize)
    }

    fn count_vertices(edges: &[E]) -> usize {
        let mut n_vertices = 0;

        for e in edges.iter() {
            n_vertices = max(n_vertices, e.max() + 1);
        }

        n_vertices
    }
}

impl<VF: Value, const N: usize> EdgeList<FilteredEdge<OneCriticalGrade<VF, N>>> {
    /// Sort the filtered edges lexicographically in increasing order.
    pub fn sort_lexicographically(&mut self) {
        self.edges.sort()
    }

    /// Reverse sort the filtered edges lexicographically.
    pub fn sort_reverse_lexicographically(&mut self) {
        self.edges.sort_by(|a, b| b.cmp(a))
    }

    /// Sort the filtered edges colexicographically in increasing order.
    pub fn sort_colexicographically(&mut self) {
        self.edges
            .sort_by(|a, b| a.cmp_by(b, OneCriticalGrade::cmp_colexicographically))
    }

    /// Reverse sort the filtered edges colexicographically.
    pub fn sort_reverse_colexicographically(&mut self) {
        self.edges
            .sort_by(|a, b| b.cmp_by(a, OneCriticalGrade::cmp_colexicographically))
    }

    /// Put a random order on the edges..
    pub fn shuffle(&mut self) {
        self.edges.shuffle(&mut thread_rng())
    }
}

impl<E: Edge> From<Vec<E>> for EdgeList<E> {
    fn from(edges: Vec<E>) -> Self {
        let n_vertices = Self::count_vertices(&edges);
        Self { n_vertices, edges }
    }
}

#[cfg(test)]
mod tests {
    use crate::edges::{BareEdge, EdgeList, FilteredEdge};
    use crate::OneCriticalGrade;

    #[test]
    fn edge_list_lexicographic_order() {
        let mut edges: EdgeList<_> = sorting_test_dataset();
        edges.sort_lexicographically();
        let grades: Vec<OneCriticalGrade<usize, 2>> = edges.edge_iter().map(|e| e.grade).collect();
        let expected_grades: Vec<OneCriticalGrade<usize, 2>> =
            vec![[1, 1].into(), [1, 2].into(), [2, 1].into(), [2, 2].into()];
        assert_eq!(grades, expected_grades);
    }

    #[test]
    fn edge_list_reverse_lexicographic_order() {
        let mut edges: EdgeList<_> = sorting_test_dataset();
        edges.sort_reverse_lexicographically();
        let grades: Vec<OneCriticalGrade<usize, 2>> = edges.edge_iter().map(|e| e.grade).collect();
        let expected_grades: Vec<OneCriticalGrade<usize, 2>> =
            vec![[2, 2].into(), [2, 1].into(), [1, 2].into(), [1, 1].into()];
        assert_eq!(grades, expected_grades);
    }

    #[test]
    fn edge_list_colexicographic_order() {
        let mut edges: EdgeList<_> = sorting_test_dataset();
        edges.sort_colexicographically();
        let grades: Vec<OneCriticalGrade<usize, 2>> = edges.edge_iter().map(|e| e.grade).collect();
        let expected_grades: Vec<OneCriticalGrade<usize, 2>> =
            vec![[1, 1].into(), [2, 1].into(), [1, 2].into(), [2, 2].into()];
        assert_eq!(grades, expected_grades);
    }

    #[test]
    fn edge_list_reverse_colexicographic_order() {
        let mut edges: EdgeList<_> = sorting_test_dataset();
        edges.sort_reverse_colexicographically();
        let grades: Vec<OneCriticalGrade<usize, 2>> = edges.edge_iter().map(|e| e.grade).collect();
        let expected_grades: Vec<OneCriticalGrade<usize, 2>> =
            vec![[2, 2].into(), [1, 2].into(), [2, 1].into(), [1, 1].into()];
        assert_eq!(grades, expected_grades);
    }

    fn sorting_test_dataset() -> EdgeList<FilteredEdge<OneCriticalGrade<usize, 2>>> {
        vec![
            FilteredEdge {
                grade: [1, 1].into(),
                edge: BareEdge(0, 1),
            },
            FilteredEdge {
                grade: [2, 2].into(),
                edge: BareEdge(5, 3),
            },
            FilteredEdge {
                grade: [2, 1].into(),
                edge: BareEdge(0, 3),
            },
            FilteredEdge {
                grade: [1, 2].into(),
                edge: BareEdge(2, 1),
            },
        ]
        .into()
    }
}
