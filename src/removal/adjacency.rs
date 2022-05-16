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
    use crate::edges::{BareEdge, FilteredEdge};
    use crate::removal::adjacency::AdjacencyMatrix;
    use crate::OneCriticalGrade;

    #[test]
    fn closed_edge_neighbours_happy_case() {
        let mut adj: AdjacencyMatrix<OneCriticalGrade<usize, 2>> = AdjacencyMatrix::new(3);
        let query_edge = FilteredEdge {
            edge: BareEdge(0, 1),
            grade: OneCriticalGrade([2, 2]),
        };
        adj.add_edge(query_edge);
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 2),
            grade: OneCriticalGrade([1, 2]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 2),
            grade: OneCriticalGrade([2, 3]),
        });
        let neighs: Vec<_> = adj.closed_neighbours_edge(&query_edge).collect();
        assert_eq!(
            neighs,
            vec![
                (0, OneCriticalGrade([2, 2])),
                (1, OneCriticalGrade([2, 2])),
                (2, OneCriticalGrade([2, 3]))
            ]
        );
    }

    #[test]
    fn closed_edge_neighbours_many() {
        let mut adj: AdjacencyMatrix<OneCriticalGrade<usize, 2>> = AdjacencyMatrix::new(6);
        let query_edge = FilteredEdge {
            edge: BareEdge(0, 1),
            grade: OneCriticalGrade([2, 2]),
        };
        adj.add_edge(query_edge);

        // Add vertex 2 as an edge neighbour.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 2),
            grade: OneCriticalGrade([1, 2]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 2),
            grade: OneCriticalGrade([2, 3]),
        });

        // Add vertex 3 as an edge neighbour.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 3),
            grade: OneCriticalGrade([4, 5]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 3),
            grade: OneCriticalGrade([5, 4]),
        });

        // Add vertex 4 as an edge neighbour.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 4),
            grade: OneCriticalGrade([1, 1]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 4),
            grade: OneCriticalGrade([0, 0]),
        });

        // Vertex 5 is NOT an edge neighbour.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 5),
            grade: OneCriticalGrade([0, 0]),
        });

        let neighs: Vec<_> = adj.closed_neighbours_edge(&query_edge).collect();
        assert_eq!(
            neighs,
            vec![
                (0, OneCriticalGrade([2, 2])),
                (1, OneCriticalGrade([2, 2])),
                (2, OneCriticalGrade([2, 3])),
                (3, OneCriticalGrade([5, 5])),
                (4, OneCriticalGrade([2, 2])),
            ]
        );
    }

    #[test]
    fn closed_neighbours_many() {
        let mut adj: AdjacencyMatrix<OneCriticalGrade<usize, 2>> = AdjacencyMatrix::new(6);
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 1),
            grade: OneCriticalGrade([2, 2]),
        });

        // Connect vertex 2 to 0 and 1.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 2),
            grade: OneCriticalGrade([1, 2]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 2),
            grade: OneCriticalGrade([2, 3]),
        });

        // Connect vertex 3 to 0 and 1.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 3),
            grade: OneCriticalGrade([4, 5]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 3),
            grade: OneCriticalGrade([5, 4]),
        });

        // Connect vertex 4 to 0 and 1.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 4),
            grade: OneCriticalGrade([1, 1]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 4),
            grade: OneCriticalGrade([0, 0]),
        });

        // Connect vertex 5 only to 0.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 5),
            grade: OneCriticalGrade([0, 0]),
        });

        let neighs: Vec<_> = adj
            .closed_neighbours(1, OneCriticalGrade([10, 10]))
            .collect();
        assert_eq!(
            neighs,
            vec![
                (0, OneCriticalGrade([2, 2])),
                (1, OneCriticalGrade([10, 10])),
                (2, OneCriticalGrade([2, 3])),
                (3, OneCriticalGrade([5, 4])),
                (4, OneCriticalGrade([0, 0])),
            ]
        );
    }
}
