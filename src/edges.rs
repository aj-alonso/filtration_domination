use crate::{OneCriticalGrade, Value};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::cmp::{max, Ordering};
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

pub trait Edge {
    fn u(&self) -> usize;

    fn u_mut(&mut self) -> &mut usize;

    fn v(&self) -> usize;

    fn v_mut(&mut self) -> &mut usize;

    fn max(&self) -> usize {
        std::cmp::max(self.u(), self.v())
    }

    fn min(&self) -> usize {
        std::cmp::min(self.u(), self.v())
    }

    fn minmax(&self) -> (usize, usize) {
        (self.min(), self.max())
    }
}

/// Undirected edge.
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

impl BareEdge {
    pub fn incident_to(&self, other: &BareEdge) -> bool {
        self.0 == other.0 || self.0 == other.1 || self.1 == other.1 || self.1 == other.0
    }

    pub fn vertices_if_incident(&self, other: &BareEdge) -> Option<[usize; 3]> {
        if self.0 == other.0 || self.1 == other.0 {
            Some([self.0, self.1, other.1])
        } else if self.0 == other.1 || self.1 == other.1 {
            Some([self.0, self.1, other.0])
        } else {
            None
        }
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

/// A graph represented as a list of edges, whose vertices are in the range 0..n_vertices.
/// No self-loops are allowed.
#[derive(Debug, Clone)]
pub struct EdgeList<E> {
    pub n_vertices: usize,
    edges: Vec<E>,
}

impl<E: Edge> EdgeList<E> {
    pub fn new(n_vertices: usize) -> Self {
        Self {
            n_vertices,
            edges: Vec::new(),
        }
    }

    pub fn edges(&self) -> &[E] {
        &self.edges
    }

    pub fn edges_mut(&mut self) -> &mut [E] {
        &mut self.edges
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn from_iterator<I: Iterator<Item = E>>(it: I) -> Self {
        let edges: Vec<E> = it.collect();
        edges.into()
    }

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
    pub fn sort_lexicographically(&mut self) {
        self.edges.sort()
    }

    pub fn sort_reverse_lexicographically(&mut self) {
        self.edges.sort_by(|a, b| b.cmp(a))
    }

    pub fn sort_colexicographically(&mut self) {
        self.edges
            .sort_by(|a, b| a.cmp_by(b, OneCriticalGrade::cmp_colexicographically))
    }

    pub fn sort_reverse_colexicographically(&mut self) {
        self.edges
            .sort_by(|a, b| b.cmp_by(a, OneCriticalGrade::cmp_colexicographically))
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FilteredEdge<G> {
    pub grade: G,
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
