use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::EdgeOrder;
use crate::{CriticalGrade, Vertex};
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
            edge_list.edges_mut().sort_unstable_by(|a, b| b.cmp(a));
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
        let v_neighs = adjacency_matrix.closed_neighbours(v, value_v.join(&edge.grade));
        if is_subset(edge_neighs, v_neighs) {
            return true;
        }
    }
    false
}

fn is_subset<G: CriticalGrade, I, J>(left: I, mut right: J) -> bool
where
    I: Iterator<Item = (Vertex, G)>,
    J: Iterator<Item = (Vertex, G)>,
{
    'next_a: for (a, value_a) in left {
        for (b, value_b) in right.by_ref() {
            match a.cmp(&b) {
                Ordering::Less => break,
                Ordering::Equal => {
                    if value_b.lte(&value_a) {
                        continue 'next_a;
                    } else {
                        break;
                    }
                }
                Ordering::Greater => continue,
            }
        }
        return false;
    }
    true
}
