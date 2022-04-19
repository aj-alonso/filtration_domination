use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::iter::Peekable;
use litemap::LiteMap;
use nohash_hasher::NoHashHasher;

use sorted_iter::assume::{AssumeSortedByItemExt, AssumeSortedByKeyExt};
use sorted_iter::{SortedIterator, SortedPairIterator};

use crate::edges::{BareEdge, FilteredEdge};
use crate::CriticalGrade;

pub(crate) type VertexAdjacency<G> = HashMap<usize, G, BuildHasherDefault<NoHashHasher<usize>>>;

pub(crate) struct AdjacencyMatrix<G> {
    pub(crate) matrix: Vec<VertexAdjacency<G>>,
}

impl<G: CriticalGrade> AdjacencyMatrix<G> {
    pub fn new(n_vertices: usize) -> Self {
        Self {
            matrix: vec![HashMap::with_hasher(BuildHasherDefault::default()); n_vertices],
        }
    }

    pub fn add_edge(&mut self, edge: FilteredEdge<G>) {
        let BareEdge(u, v) = edge.edge;
        self.matrix[u].insert(v, edge.grade.clone());
        self.matrix[v].insert(u, edge.grade);
    }

    pub fn delete_edge(
        &mut self,
        FilteredEdge {
            edge: BareEdge(u, v),
            ..
        }: &FilteredEdge<G>,
    ) {
        self.matrix[*u].remove(v);
        self.matrix[*v].remove(u);
    }

    /// Returns an iterator over the open neighbours of the vertex u and the grade of the edge that
    /// connects u and its neighbor.
    /// The open neighbours of the vertex u are those that are connected by an edge.
    pub fn open_neighbours(&self, u: usize) -> impl Iterator<Item = (usize, G)> + '_ {
        self.matrix[u]
            .iter()
            .map(move |(&vertex, edge_grade)| (vertex, edge_grade.clone()))
    }

    /// Returns an iterator over the closed neighbours of the vertex u and the grade of the edge that
    /// connects u and its neighbor -- when the neighbor is u itself the grade is the grade specified
    /// in the u_value argument.
    /// The closed neighbours of the vertex u are those that are connected by an edge that is either
    /// critical in the current graph, or whose index is equal to or less than max_index_value, in
    /// addition to u itself.
    pub fn closed_neighbours(&self, u: usize, u_value: G) -> impl Iterator<Item = (usize, G)> + '_ {
        std::iter::once((u, u_value)).chain(self.open_neighbours(u))
    }

    fn common_neighbours_raw<'a>(
        &'a self,
        edge: &'a FilteredEdge<G>,
    ) -> impl Iterator<Item = (usize, (G, G))> + 'a {
        let BareEdge(u, v) = edge.edge;
        CommonNeighboursIter {
            u_adj_iter: self.matrix[u].iter(),
            v_adj: &self.matrix[v],
        }
    }

    pub fn common_neighbours<'a>(
        &'a self,
        edge: &'a FilteredEdge<G>,
    ) -> impl Iterator<Item = (usize, G)> + 'a + std::marker::Send {
        self.common_neighbours_raw(edge)
            .map(move |(neigh, (value_u, value_v))| (neigh, value_u.join(&value_v)))
    }

    pub fn closed_neighbours_edge<'a>(
        &'a self,
        edge: &'a FilteredEdge<G>,
    ) -> impl Iterator<Item = (usize, G)> + 'a {
        let BareEdge(edge_u, edge_v) = edge.edge;
        std::iter::once((edge_u, edge.grade.clone()))
            .chain(std::iter::once((edge_v, edge.grade.clone())))
            .chain(self.common_neighbours(edge)
                    .map(move |(neigh, neigh_value)| (neigh, neigh_value.join(&edge.grade)))
            )
    }
}

struct CommonNeighboursIter<'a, G> {
    u_adj_iter: Iter<'a, usize, G>,
    v_adj: &'a VertexAdjacency<G>,
}

impl<'a, G: CriticalGrade> Iterator for CommonNeighboursIter<'a, G> {
    type Item = (usize, (G, G));

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((u_neigh, u_crit)) = self.u_adj_iter.next() {
            if let Some(v_crit) = self.v_adj.get(u_neigh) {
                return Some((*u_neigh, (u_crit.clone(), v_crit.clone())))
            }
        }
        None
    }
}
