use rayon::prelude::*;
use std::collections::BTreeSet;

use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::full::regions;
use crate::removal::full::regions::NonDominationRegion;
use crate::removal::EdgeOrder;
use crate::{CriticalGrade, OneCriticalGrade, Value};

pub fn remove_filtration_dominated_multithread<VF: Value>(
    edge_list: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
    order: EdgeOrder,
) -> EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>> {
    match order {
        EdgeOrder::ReverseLexicographic => {
            edge_list.edges_mut().sort_by(|a, b| b.cmp(a));
        }
        EdgeOrder::Maintain => {}
    }

    let mut critical_edges: Vec<FilteredEdge<OneCriticalGrade<VF, 2>>> =
        Vec::with_capacity(edge_list.len());
    let mut adjacency_matrix = AdjacencyMatrix::new(edge_list.n_vertices);

    for edge in edge_list.edge_iter() {
        adjacency_matrix.add_edge(*edge);
    }

    for edge in edge_list.edge_iter() {
        if is_filtration_dominated_multithread(&adjacency_matrix, edge) {
            adjacency_matrix.delete_edge(edge);
        } else {
            critical_edges.push(*edge);
        }
    }

    critical_edges.shrink_to_fit();
    critical_edges.into()
}

fn is_filtration_dominated_multithread<VF: Value>(
    adjacency_matrix: &AdjacencyMatrix<OneCriticalGrade<VF, 2>>,
    edge: &FilteredEdge<OneCriticalGrade<VF, 2>>,
) -> bool {
    let n_neighbours = adjacency_matrix.common_neighbours(edge).count();
    let non_domination_regions: Vec<NonDominationRegion<VF>> = adjacency_matrix
        .common_neighbours(edge)
        .par_bridge()
        .map(|(v, value_v)| -> Option<NonDominationRegion<VF>> {
            let non_domination_region =
                regions::calculate_non_domination_region(adjacency_matrix, edge, v, value_v);
            if non_domination_region.is_empty() {
                // The vertex v strongly dominates the edge.
                None
            } else {
                Some(non_domination_region)
            }
        })
        .while_some()
        .collect();

    // At least one vertex strongly dominates.
    if non_domination_regions.len() < n_neighbours {
        return true;
    }

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

    domination_times.into_par_iter().all(|critical_value| {
        let mut dominated = false;
        for region in non_domination_regions.iter() {
            if !region.contains_point(critical_value) {
                dominated = true;
                break;
            }
        }
        if !dominated {
            return false;
        }
        true
    })
}
