use std::cmp::Ordering;
use std::time::Duration;

use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::EdgeOrder;
use crate::CriticalGrade;

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

    let mut remaining_edges: Vec<FilteredEdge<G>> = Vec::with_capacity(edge_list.len());
    let mut adjacency_matrix = AdjacencyMatrix::new(edge_list.n_vertices);

    for edge in edge_list.edge_iter() {
        adjacency_matrix.add_edge(edge.clone());
    }

    let start = std::time::Instant::now();
    for edge in edge_list.edge_iter() {
        if let Some(max_time) = max_time {
            if start.elapsed() > max_time {
                return edge_list.clone();
            }
        }

        if is_strongly_filtration_dominated(&adjacency_matrix, edge) {
            adjacency_matrix.delete_edge(edge);
        } else {
            remaining_edges.push(edge.clone());
        }
    }

    remaining_edges.shrink_to_fit();
    remaining_edges.into()
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
    I: Iterator<Item = (usize, G)>,
    J: Iterator<Item = (usize, G)>,
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

#[cfg(test)]
mod tests {
    use crate::edges::{BareEdge, FilteredEdge};
    use crate::removal::adjacency::AdjacencyMatrix;
    use crate::removal::strong::{is_strongly_filtration_dominated, is_subset};
    use crate::OneCriticalGrade;

    #[test]
    fn strongly_filtration_dominated_happy_case() {
        let mut adj: AdjacencyMatrix<OneCriticalGrade<usize, 2>> = AdjacencyMatrix::new(6);
        let query_edge = FilteredEdge {
            edge: BareEdge(0, 1),
            grade: OneCriticalGrade([2, 2]),
        };
        adj.add_edge(query_edge);

        // Add 2 to the edge neighborhood at grade [2, 2].
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 2),
            grade: OneCriticalGrade([1, 2]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 2),
            grade: OneCriticalGrade([2, 1]),
        });

        // Add 3 to the edge neighborhood at grade [4, 4].
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 3),
            grade: OneCriticalGrade([4, 3]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 3),
            grade: OneCriticalGrade([3, 4]),
        });

        // Connect 2 to 3 when 3 appears.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(3, 2),
            grade: OneCriticalGrade([4, 4]),
        });

        assert!(is_strongly_filtration_dominated(&adj, &query_edge));
    }

    #[test]
    fn not_strongly_filtration_dominated() {
        let mut adj: AdjacencyMatrix<OneCriticalGrade<usize, 2>> = AdjacencyMatrix::new(6);
        let query_edge = FilteredEdge {
            edge: BareEdge(0, 1),
            grade: OneCriticalGrade([2, 2]),
        };
        adj.add_edge(query_edge);

        // Add 2 to the edge neighborhood at grade [2, 2].
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 2),
            grade: OneCriticalGrade([1, 2]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 2),
            grade: OneCriticalGrade([2, 1]),
        });

        // Add 3 to the edge neighborhood at grade [4, 4].
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 3),
            grade: OneCriticalGrade([4, 3]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 3),
            grade: OneCriticalGrade([3, 4]),
        });

        // Connect 2 to 3 after 3 appears.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(3, 2),
            grade: OneCriticalGrade([5, 5]),
        });

        assert!(!is_strongly_filtration_dominated(&adj, &query_edge));
    }

    #[test]
    fn is_subset_happy_case() {
        let a = vec![
            (0, OneCriticalGrade([2, 1])),
            (10, OneCriticalGrade([2, 3])),
            (20, OneCriticalGrade([4, 5])),
            (30, OneCriticalGrade([3, 4])),
        ];

        let b = vec![
            (0, OneCriticalGrade([2, 1])),
            (1, OneCriticalGrade([2, 1])),
            (2, OneCriticalGrade([2, 1])),
            (10, OneCriticalGrade([1, 1])),
            (15, OneCriticalGrade([2, 3])),
            (16, OneCriticalGrade([2, 3])),
            (20, OneCriticalGrade([3, 5])),
            (24, OneCriticalGrade([4, 5])),
            (30, OneCriticalGrade([3, 2])),
        ];

        assert!(is_subset(a.into_iter(), b.into_iter()));
    }

    #[test]
    fn is_not_subset() {
        let a = vec![
            (0, OneCriticalGrade([2, 1])),
            (30, OneCriticalGrade([3, 4])),
        ];

        let b = vec![
            (0, OneCriticalGrade([2, 1])),
            (1, OneCriticalGrade([2, 1])),
            (2, OneCriticalGrade([2, 1])),
            (10, OneCriticalGrade([1, 1])),
            (15, OneCriticalGrade([2, 3])),
            (16, OneCriticalGrade([2, 3])),
            (20, OneCriticalGrade([3, 5])),
            (24, OneCriticalGrade([4, 5])),
            // The value of 30 is to high.
            (30, OneCriticalGrade([3, 5])),
        ];

        assert!(!is_subset(a.into_iter(), b.into_iter()));
    }
}
