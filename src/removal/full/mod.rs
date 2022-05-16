use std::collections::BTreeSet;
use std::time::Duration;

use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::EdgeOrder;
use crate::Value;
use crate::{CriticalGrade, OneCriticalGrade};

mod regions;
mod stripes;

/// Go through the given edge list, and check each edge for filtration-domination.
/// If it is filtration-dominated we remove them.
/// The order in which we go through the edges is the given in `order`.
/// Returns a reduced edge list.
pub fn remove_filtration_dominated<VF: Value>(
    edge_list: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
    order: EdgeOrder,
) -> EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>> {
    remove_filtration_dominated_timed(edge_list, order, None)
}

/// As [remove_filtration_dominated], but if we take more than the time given in `max_time` then
/// execution stops and a clone of the original list is returned.
/// If `max_time` is None then no timeout is applied.
pub fn remove_filtration_dominated_timed<VF: Value>(
    edge_list: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
    order: EdgeOrder,
    max_time: Option<Duration>,
) -> EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>> {
    match order {
        EdgeOrder::ReverseLexicographic => {
            edge_list.edges_mut().sort_unstable_by(|a, b| b.cmp(a));
        }
        EdgeOrder::Maintain => {}
    }

    let mut remaining_edges: Vec<FilteredEdge<OneCriticalGrade<VF, 2>>> =
        Vec::with_capacity(edge_list.len());
    let mut adjacency_matrix = AdjacencyMatrix::new(edge_list.n_vertices);

    for edge in edge_list.edge_iter() {
        adjacency_matrix.add_edge(*edge);
    }

    let start = std::time::Instant::now();
    for edge in edge_list.edge_iter() {
        if let Some(max_time) = max_time {
            if start.elapsed() > max_time {
                return edge_list.clone();
            }
        }
        if is_filtration_dominated(&adjacency_matrix, edge) {
            adjacency_matrix.delete_edge(edge);
        } else {
            remaining_edges.push(*edge);
        }
    }

    remaining_edges.shrink_to_fit();
    remaining_edges.into()
}

fn is_filtration_dominated<VF: Value>(
    adjacency_matrix: &AdjacencyMatrix<OneCriticalGrade<VF, 2>>,
    edge: &FilteredEdge<OneCriticalGrade<VF, 2>>,
) -> bool {
    // Compute regions of non-domination for every vertex in the edge neighbourhood.
    let mut non_domination_regions = Vec::new();
    for (v, value_v) in adjacency_matrix.common_neighbours(edge) {
        let non_domination_region =
            regions::calculate_non_domination_region(adjacency_matrix, edge, v, value_v);
        if non_domination_region.is_empty() {
            // The vertex v strongly dominates the edge.
            return true;
        }
        non_domination_regions.push(non_domination_region);
    }

    // Compute all critical grades, where we need to check for domination.
    let mut first_domination_times: BTreeSet<OneCriticalGrade<VF, 2>> =
        BTreeSet::from_iter([edge.grade]);

    for (_neigh_vertex, neigh_value) in adjacency_matrix.common_neighbours(edge) {
        first_domination_times.insert(edge.grade.join(&neigh_value));
    }
    let mut domination_times: BTreeSet<OneCriticalGrade<VF, 2>> = BTreeSet::new();
    for time in first_domination_times.iter() {
        for other_time in first_domination_times.iter() {
            domination_times.insert(time.join(other_time));
        }
    }

    for grade in domination_times {
        let mut dominated = false;
        for region in non_domination_regions.iter() {
            if !region.contains_point(grade) {
                dominated = true;
                break;
            }
        }
        if !dominated {
            return false;
        }
    }
    true
}
