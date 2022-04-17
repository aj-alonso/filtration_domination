use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::{AdjacencyMatrix, VertexAdjacency};
use crate::removal::EdgeOrder;
use crate::CriticalGrade;
use std::cmp::Ordering;
use std::time::Duration;

/// As [crate::removal::remove_filtration_dominated], but instead of filtration-dominated edges
/// this function checks for strongly filtration-dominated edges.
pub fn remove_strongly_filtration_dominated<G: CriticalGrade>(
    edge_list: &mut EdgeList<FilteredEdge<G>>,
    order: EdgeOrder,
) -> EdgeList<FilteredEdge<G>> {
    remove_strongly_filtration_dominated_timed(edge_list, order, None)
}

/// As [remove_strongly_filtration_dominated], but if we take more than the time given in `max_time` then
/// execution stops and a clone of the original list is returned.
/// If `max_time` is None then no timeout is applied.
pub fn remove_strongly_filtration_dominated_timed<G: CriticalGrade>(
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
    let mut adjacency_matrix = AdjacencyMatrix::new(edge_list.n_vertices);

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

        if is_strongly_filtration_dominated(&adjacency_matrix, edge) {
            adjacency_matrix.delete_edge(edge);
        } else {
            critical_edges.push(edge.clone());
        }
    }

    critical_edges.shrink_to_fit();
    critical_edges.into()
}

fn is_strongly_filtration_dominated<G: CriticalGrade>(
    adjacency_matrix: &AdjacencyMatrix<G>,
    edge: &FilteredEdge<G>,
) -> bool {
    for (v, value_v) in adjacency_matrix.common_neighbours(edge) {
        let edge_neighs = adjacency_matrix.closed_neighbours_edge(edge);
        if is_subset(edge_neighs, &adjacency_matrix.matrix[v], v, value_v) {
            return true;
        }
    }
    false
}

fn is_subset<G: CriticalGrade, I>(left: I, v_neighs: &VertexAdjacency<G>, v: usize, value_v: G) -> bool
where
    I: Iterator<Item = (usize, G)>,
{
    for (a, value_a) in left {
        if a == v {
            continue;
        }
        if let Some(value_edge) = v_neighs.get(&a) {
            if !value_edge.lte(&value_a) {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}
