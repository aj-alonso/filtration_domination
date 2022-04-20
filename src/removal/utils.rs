//! Utilities to study bifiltered graphs.
use sorted_iter::assume::AssumeSortedByItemExt;
use sorted_iter::SortedIterator;

use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::{CriticalGrade, OneCriticalGrade, Value};

/// Given an edge list, returns a tuple that contains the number of edges that are
/// isolated (that is, they have empty edge neighborhood) at their critical grade,
/// and the number of edge dominated at their critical grade.
#[must_use]
pub fn count_isolated_edges<VF: Value>(
    edge_list: &EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
) -> (usize, usize) {
    let mut isolated_edges = 0;
    let mut dominated_when_appear = 0;

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
        if is_dominated_at_time(&adjacency_matrix, edge, &edge.grade) {
            dominated_when_appear += 1;
        }
    }

    (isolated_edges, dominated_when_appear)
}

fn is_dominated_at_time<G: CriticalGrade>(
    adjacency_matrix: &AdjacencyMatrix<G>,
    edge: &FilteredEdge<G>,
    critical_value: &G,
) -> bool {
    for neigh_vertex in adjacency_matrix
        .common_neighbours(edge)
        .filter_map(|(v, value)| (value.lte(critical_value)).then(|| v))
    {
        if is_dominated_at_time_by(adjacency_matrix, edge, critical_value, neigh_vertex) {
            return true;
        }
    }
    false
}

fn is_dominated_at_time_by<G: CriticalGrade>(
    adjacency_matrix: &AdjacencyMatrix<G>,
    edge: &FilteredEdge<G>,
    critical_value: &G,
    neigh_vertex: usize,
) -> bool {
    let other_neighs = adjacency_matrix
        .closed_neighbours(neigh_vertex, critical_value.clone())
        .filter_map(move |(v, v_value)| v_value.lte(critical_value).then(|| v))
        .assume_sorted_by_item();
    let applicable_neighs = adjacency_matrix
        .common_neighbours(edge)
        .filter_map(|(v, value)| (value.lte(critical_value)).then(|| v))
        .assume_sorted_by_item();

    applicable_neighs.is_subset(other_neighs)
}
