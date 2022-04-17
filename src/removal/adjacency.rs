use litemap::LiteMap;

use sorted_iter::assume::{AssumeSortedByItemExt, AssumeSortedByKeyExt};
use sorted_iter::{SortedIterator, SortedPairIterator};

use crate::edges::{BareEdge, FilteredEdge};
use crate::CriticalGrade;

pub(crate) struct AdjacencyMatrix<G> {
    matrix: Vec<LiteMap<usize, G>>,
}

impl<G: CriticalGrade> AdjacencyMatrix<G> {
    pub fn new(n_vertices: usize) -> Self {
        Self {
            matrix: vec![LiteMap::new(); n_vertices],
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
    ///
    /// The returned iterator is sorted by vertex.
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
    ///
    /// The returned iterator is sorted by vertex.
    pub fn closed_neighbours(&self, u: usize, u_value: G) -> impl Iterator<Item = (usize, G)> + '_ {
        self.open_neighbours(u)
            .assume_sorted_by_item()
            .union(std::iter::once((u, u_value)))
    }

    fn common_neighbours_raw<'a>(
        &'a self,
        edge: &'a FilteredEdge<G>,
    ) -> impl Iterator<Item = (usize, (G, G))> + 'a {
        let BareEdge(u, v) = edge.edge;
        let neigh_u = self.open_neighbours(u).assume_sorted_by_key();
        let neigh_v = self.open_neighbours(v).assume_sorted_by_key();
        neigh_u.join(neigh_v)
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
        self.common_neighbours(edge)
            .map(move |(neigh, neigh_value)| (neigh, neigh_value.join(&edge.grade)))
            .assume_sorted_by_item()
            .union(std::iter::once((edge_u, edge.grade.clone())))
            .union(std::iter::once((edge_v, edge.grade.clone())))
    }
}

#[cfg(test)]
mod tests {
    use crate::edges::BareEdge;
    use crate::graph::GradedEdge;
    use crate::removal::adjacency::AdjacencyMatrix;
    use crate::CriticalGrade;
    use sorted_iter::assume::AssumeSortedByKeyExt;
    use sorted_iter::SortedPairIterator;

    #[test]
    fn eleventh_counterexample_graph() {
        let edges: Vec<GradedEdge<usize, 2>> = vec![
            GradedEdge {
                grade: [4, 5].into(),
                edge: BareEdge(1, 10),
            },
            GradedEdge {
                grade: [3, 12].into(),
                edge: BareEdge(1, 13),
            },
            GradedEdge {
                grade: [4, 11].into(),
                edge: BareEdge(10, 13),
            },
            GradedEdge {
                grade: [5, 3].into(),
                edge: BareEdge(1, 14),
            },
            GradedEdge {
                grade: [2, 7].into(),
                edge: BareEdge(1, 15),
            },
            GradedEdge {
                grade: [3, 4].into(),
                edge: BareEdge(13, 15),
            },
            GradedEdge {
                grade: [5, 6].into(),
                edge: BareEdge(14, 15),
            },
            GradedEdge {
                grade: [1, 16].into(),
                edge: BareEdge(1, 16),
            },
            GradedEdge {
                grade: [4, 9].into(),
                edge: BareEdge(10, 16),
            },
            GradedEdge {
                grade: [3, 1].into(),
                edge: BareEdge(13, 16),
            },
            GradedEdge {
                grade: [2, 14].into(),
                edge: BareEdge(15, 16),
            },
            GradedEdge {
                grade: [0, 0].into(),
                edge: BareEdge(1, 17),
            },
            GradedEdge {
                grade: [4, 2].into(),
                edge: BareEdge(10, 17),
            },
            GradedEdge {
                grade: [3, 13].into(),
                edge: BareEdge(13, 17),
            },
            GradedEdge {
                grade: [2, 10].into(),
                edge: BareEdge(15, 17),
            },
            GradedEdge {
                grade: [1, 15].into(),
                edge: BareEdge(16, 17),
            },
        ];
        let mut matrix = AdjacencyMatrix::new(20);
        for e in edges {
            matrix.add_edge(e);
        }
        let neigh_u = matrix.open_neighbours(13).assume_sorted_by_key();
        let neigh_v = matrix.open_neighbours(17).assume_sorted_by_key();
        let neigh_join = neigh_u.join(neigh_v);
        println!(
            "Common neighbors of (13, 17): {:?}",
            neigh_join
                .map_values(move |(value_u, value_v)| value_u.join(&value_v))
                .assume_sorted_by_key()
                .collect::<Vec<_>>()
        );
    }
}
