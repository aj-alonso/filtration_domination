use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::naive;
use crate::{CriticalGrade, OneCriticalGrade, Value};

pub fn count_isolated_edges<VF: Value>(
    edge_list: &EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
) -> (usize, usize) {
    let mut isolated_edges = 0;
    let mut non_dominated_when_appear = 0;

    let mut adjacency_matrix = AdjacencyMatrix::new(edge_list.n_vertices);

    for edge in edge_list.edge_iter() {
        adjacency_matrix.add_edge(*edge);
    }

    for edge in edge_list.edge_iter() {
        let mut neighbors_it = adjacency_matrix
            .common_neighbours(edge)
            .filter_map(|(v, value)| (value.lte(&edge.grade)).then(|| v));
        if neighbors_it.next().is_none() {
            // Edge has empty neighborhood.
            isolated_edges += 1;
        }
        if naive::is_dominated_at_time(&adjacency_matrix, edge, &edge.grade) {
            non_dominated_when_appear += 1;
        }
    }

    (isolated_edges, non_dominated_when_appear)
}
