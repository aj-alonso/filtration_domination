use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::EdgeOrder;
use crate::CriticalGrade;
use sorted_iter::assume::AssumeSortedByItemExt;
use sorted_iter::SortedIterator;
use std::collections::BTreeSet;
use std::time::Duration;

pub fn edge_collapse_naive_timed<G: CriticalGrade>(
    edge_list: &mut EdgeList<FilteredEdge<G>>,
    order: EdgeOrder,
    max_time: Option<Duration>,
) -> EdgeList<FilteredEdge<G>> {
    match order {
        EdgeOrder::ReverseLexicographic => {
            edge_list.edges_mut().sort_by(|a, b| b.cmp(a));
        }
        EdgeOrder::Maintain => {}
    }

    let mut critical_edges: Vec<FilteredEdge<G>> = Vec::with_capacity(edge_list.len());
    let mut adjacency_matrix: AdjacencyMatrix<G> = AdjacencyMatrix::new(edge_list.n_vertices);

    for edge in edge_list.edge_iter() {
        adjacency_matrix.add_edge(edge.clone());
    }

    let start = std::time::Instant::now();
    for edge in edge_list.edge_iter() {
        if let Some(max_time) = max_time {
            if start.elapsed() > max_time {
                break;
            }
        }
        if is_dominated_naive(&adjacency_matrix, edge) {
            adjacency_matrix.delete_edge(edge);
        } else {
            critical_edges.push(edge.clone());
        }
    }

    critical_edges.shrink_to_fit();
    critical_edges.into()
}

pub fn edge_collapse_naive<G: CriticalGrade>(
    edge_list: &mut EdgeList<FilteredEdge<G>>,
    order: EdgeOrder,
) -> EdgeList<FilteredEdge<G>> {
    edge_collapse_naive_timed(edge_list, order, None)
}

fn is_dominated_naive<G: CriticalGrade>(
    adjacency_matrix: &AdjacencyMatrix<G>,
    edge: &FilteredEdge<G>,
) -> bool {
    let mut first_domination_times: BTreeSet<G> = BTreeSet::from_iter([edge.grade.clone()]);

    for (_neigh_vertex, neigh_value) in adjacency_matrix.common_neighbours(edge) {
        first_domination_times.insert(edge.grade.join(&neigh_value));
    }
    let mut domination_times: BTreeSet<G> = BTreeSet::new();
    for time in first_domination_times.iter() {
        for other_time in first_domination_times.iter() {
            domination_times.insert(time.join(other_time));
        }
    }

    for critical_value in domination_times {
        if !is_dominated_at_time(adjacency_matrix, edge, &critical_value) {
            return false;
        }
    }
    true
}

pub(crate) fn is_dominated_at_time<G: CriticalGrade>(
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
    let other_neighs = adjacency_matrix.closed_neighbours_at_value(neigh_vertex, critical_value);
    let applicable_neighs = adjacency_matrix
        .common_neighbours(edge)
        .filter_map(|(v, value)| (value.lte(critical_value)).then(|| v))
        .assume_sorted_by_item();

    applicable_neighs.is_subset(other_neighs)
}
