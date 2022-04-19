use litemap::LiteMap;

use sorted_iter::assume::{AssumeSortedByItemExt, AssumeSortedByKeyExt};
use sorted_iter::{SortedIterator, SortedPairIterator};

use crate::edges::{BareEdge, FilteredEdge};
use crate::{CriticalGrade, Vertex};

pub(crate) struct AdjacencyMatrix<G> {
    matrix: Vec<LiteMap<Vertex, G>>,
}

impl<G: CriticalGrade> AdjacencyMatrix<G> {
    pub fn new(n_vertices: Vertex) -> Self {
        Self {
            matrix: vec![LiteMap::new(); n_vertices as usize],
        }
    }

    pub fn add_edge(&mut self, edge: FilteredEdge<G>) {
        let BareEdge(u, v) = edge.edge;
        self.matrix[u as usize].insert(v, edge.grade.clone());
        self.matrix[v as usize].insert(u, edge.grade);
    }

    pub fn delete_edge(
        &mut self,
        FilteredEdge {
            edge: BareEdge(u, v),
            ..
        }: &FilteredEdge<G>,
    ) {
        self.matrix[*u as usize].remove(v);
        self.matrix[*v as usize].remove(u);
    }

    /// Returns an iterator over the open neighbours of the vertex u and the grade of the edge that
    /// connects u and its neighbor.
    /// The open neighbours of the vertex u are those that are connected by an edge.
    ///
    /// The returned iterator is sorted by vertex.
    pub fn open_neighbours(&self, u: Vertex) -> impl Iterator<Item = (Vertex, G)> + '_ {
        self.matrix[u as usize]
            .iter()
            .map(move |(&vertex, edge_grade)| (vertex, edge_grade.clone()))
    }

    /// Returns an iterator over the closed neighbours of the vertex u and the grade of the edge that
    /// connects u and its neighbor -- when the neighbor is u itself the grade is the grade specified
    /// in the u_value argument.
    /// The closed neighbours of the vertex u are those that are connected by an edge that is either
    /// critical in the current graph, or whose index is equal to or less than max_index_value, in
    /// addition to u itself.
    ///
    /// The returned iterator is sorted by vertex.
    pub fn closed_neighbours(
        &self,
        u: Vertex,
        u_value: G,
    ) -> impl Iterator<Item = (Vertex, G)> + '_ {
        self.open_neighbours(u)
            .assume_sorted_by_item()
            .union(std::iter::once((u, u_value)))
    }

    fn common_neighbours_raw<'a>(
        &'a self,
        edge: &'a FilteredEdge<G>,
    ) -> impl Iterator<Item = (Vertex, (G, G))> + 'a {
        let BareEdge(u, v) = edge.edge;
        let neigh_u = self.open_neighbours(u).assume_sorted_by_key();
        let neigh_v = self.open_neighbours(v).assume_sorted_by_key();
        neigh_u.join(neigh_v)
    }

    pub fn common_neighbours<'a>(
        &'a self,
        edge: &'a FilteredEdge<G>,
    ) -> impl Iterator<Item = (Vertex, G)> + 'a + std::marker::Send {
        self.common_neighbours_raw(edge)
            .map(move |(neigh, (value_u, value_v))| (neigh, value_u.join(&value_v)))
    }

    pub fn closed_neighbours_edge<'a>(
        &'a self,
        edge: &'a FilteredEdge<G>,
    ) -> impl Iterator<Item = (Vertex, G)> + 'a {
        let BareEdge(edge_u, edge_v) = edge.edge;
        self.common_neighbours(edge)
            .map(move |(neigh, neigh_value)| (neigh, neigh_value.join(&edge.grade)))
            .assume_sorted_by_item()
            .union(std::iter::once((edge_u, edge.grade.clone())))
            .union(std::iter::once((edge_v, edge.grade.clone())))
    }
}
