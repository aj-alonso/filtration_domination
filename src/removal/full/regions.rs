use std::cmp::Ordering;

use crate::edges::FilteredEdge;
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::full::stripes::{Stripe, Stripes};
use crate::{CriticalGrade, OneCriticalGrade, Value};

pub type Pair<VF> = (OneCriticalGrade<VF, 2>, OneCriticalGrade<VF, 2>);

#[derive(Debug)]
pub struct NonDominationRegion<VF> {
    vertical_stripes: Stripes<VF>,
    horizontal_stripes: Stripes<VF>,
}

impl<VF: Value> NonDominationRegion<VF> {
    pub fn new(vertical_stripes: Vec<Stripe<VF>>, horizontal_stripes: Vec<Stripe<VF>>) -> Self {
        Self {
            vertical_stripes: Stripes::new(vertical_stripes),
            horizontal_stripes: Stripes::new(horizontal_stripes),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertical_stripes.is_empty() && self.horizontal_stripes.is_empty()
    }

    pub fn contains_point(&self, grade: OneCriticalGrade<VF, 2>) -> bool {
        let vertical_point = (grade.0[0], grade.0[1]);
        let horizontal_point = (grade.0[1], grade.0[0]);
        self.vertical_stripes.contains_point(vertical_point)
            || self.horizontal_stripes.contains_point(horizontal_point)
    }
}

pub(crate) fn calculate_non_domination_region<VF: Value>(
    adjacency_matrix: &AdjacencyMatrix<OneCriticalGrade<VF, 2>>,
    edge: &FilteredEdge<OneCriticalGrade<VF, 2>>,
    v: usize,
    value_v: OneCriticalGrade<VF, 2>,
) -> NonDominationRegion<VF> {
    let mut vertical_stripes = Vec::new();
    let mut horizontal_stripes = Vec::new();

    let mut edge_neighs = adjacency_matrix.closed_neighbours_edge(edge).peekable();
    let mut v_neighs = adjacency_matrix
        .closed_neighbours(v, value_v.join(&edge.grade))
        .peekable();
    while let Some((a, value_a)) = edge_neighs.peek() {
        if let Some((b, value_b)) = v_neighs.peek() {
            match a.cmp(b) {
                // The current vertex of edge_neighs is not in v_neighs.
                // This vertex will never get dominated.
                Ordering::Less => {
                    add_pair(
                        &mut vertical_stripes,
                        &mut horizontal_stripes,
                        (*value_a, OneCriticalGrade::max_value()),
                    );
                    // Advance edge_neighs.
                    edge_neighs.next();
                }
                // The current vertex of edge_neighs is in v_neighs.
                // This vertex will get eventually dominated.
                Ordering::Equal => {
                    add_pair(
                        &mut vertical_stripes,
                        &mut horizontal_stripes,
                        (*value_a, value_a.join(value_b)),
                    );
                    // Advance edge_neighs.
                    edge_neighs.next();
                }
                Ordering::Greater => {
                    v_neighs.next();
                }
            }
        } else {
            add_pair(
                &mut vertical_stripes,
                &mut horizontal_stripes,
                (*value_a, OneCriticalGrade::max_value()),
            );
            // Advance edge_neighs.
            edge_neighs.next();
        }
    }

    NonDominationRegion::new(vertical_stripes, horizontal_stripes)
}

fn add_pair<VF: Value>(
    vertical_stripes: &mut Vec<Stripe<VF>>,
    horizontal_stripes: &mut Vec<Stripe<VF>>,
    pair: Pair<VF>,
) {
    let (p, q) = pair;
    let p = p.0;
    let q = q.0;
    if p[0] != q[0] {
        vertical_stripes.push(((p[0], q[0]), p[1]));
    }
    if p[1] != q[1] {
        horizontal_stripes.push(((p[1], q[1]), p[0]));
    }
}

#[cfg(test)]
mod tests {
    use crate::edges::{BareEdge, FilteredEdge};
    use crate::removal::adjacency::AdjacencyMatrix;
    use crate::removal::full::regions::{
        add_pair, calculate_non_domination_region, NonDominationRegion,
    };
    use crate::OneCriticalGrade;

    #[test]
    fn add_pair_happy_case() {
        let mut vertical_stripes = Vec::new();
        let mut horizontal_stripes = Vec::new();
        add_pair(
            &mut vertical_stripes,
            &mut horizontal_stripes,
            (OneCriticalGrade([1, 1]), OneCriticalGrade([3, 4])),
        );
        assert_eq!(vertical_stripes, vec![((1, 3), 1)]);
        assert_eq!(horizontal_stripes, vec![((1, 4), 1)]);

        let regions = NonDominationRegion::new(vertical_stripes, horizontal_stripes);

        assert!(regions.contains_point(OneCriticalGrade([1, 1])));
        assert!(regions.contains_point(OneCriticalGrade([2, 1])));
        assert!(regions.contains_point(OneCriticalGrade([1, 2])));
        assert!(regions.contains_point(OneCriticalGrade([2, 2])));
        assert!(regions.contains_point(OneCriticalGrade([3, 2])));
        assert!(regions.contains_point(OneCriticalGrade([3, 3])));
        assert!(!regions.contains_point(OneCriticalGrade([3, 4])));
        assert!(!regions.contains_point(OneCriticalGrade([3, 5])));
        assert!(!regions.contains_point(OneCriticalGrade([4, 4])));
        assert!(!regions.contains_point(OneCriticalGrade([10, 10])));
    }

    #[test]
    fn add_pair_empty_case() {
        let mut vertical_stripes = Vec::new();
        let mut horizontal_stripes = Vec::new();
        add_pair(
            &mut vertical_stripes,
            &mut horizontal_stripes,
            (OneCriticalGrade([1, 1]), OneCriticalGrade([1, 1])),
        );
        assert!(vertical_stripes.is_empty());
        assert!(horizontal_stripes.is_empty());

        let regions = NonDominationRegion::new(vertical_stripes, horizontal_stripes);
        assert!(!regions.contains_point(OneCriticalGrade([0, 0])));
    }

    #[test]
    fn non_domination_region_happy_case() {
        let mut adj: AdjacencyMatrix<OneCriticalGrade<usize, 2>> = AdjacencyMatrix::new(6);
        let query_edge = FilteredEdge {
            edge: BareEdge(0, 1),
            grade: OneCriticalGrade([2, 2]),
        };
        adj.add_edge(query_edge);

        // Add 2 to the edge neighborhood at grade [2, 3].
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 2),
            grade: OneCriticalGrade([1, 2]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 2),
            grade: OneCriticalGrade([2, 3]),
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

        // Add 4 to the edge neighborhood at grade [5, 5].
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 4),
            grade: OneCriticalGrade([2, 1]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 4),
            grade: OneCriticalGrade([5, 5]),
        });

        // Add 5 to the edge neighborhood at grade [10, 10].
        adj.add_edge(FilteredEdge {
            edge: BareEdge(0, 5),
            grade: OneCriticalGrade([10, 0]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(1, 5),
            grade: OneCriticalGrade([5, 10]),
        });

        // Connect 3 to 2 and 4.
        adj.add_edge(FilteredEdge {
            edge: BareEdge(3, 2),
            grade: OneCriticalGrade([1, 1]),
        });
        adj.add_edge(FilteredEdge {
            edge: BareEdge(3, 4),
            grade: OneCriticalGrade([6, 6]),
        });

        let neighs: Vec<_> = adj.closed_neighbours_edge(&query_edge).collect();
        assert_eq!(
            neighs,
            vec![
                (0, OneCriticalGrade([2, 2])),
                (1, OneCriticalGrade([2, 2])),
                (2, OneCriticalGrade([2, 3])),
                (3, OneCriticalGrade([4, 4])),
                (4, OneCriticalGrade([5, 5])),
                (5, OneCriticalGrade([10, 10])),
            ]
        );
        let region =
            calculate_non_domination_region(&adj, &query_edge, 3, OneCriticalGrade([4, 4]));

        // Vertex 3 is not connected to vertex 2 at grade [2, 2].
        assert!(region.contains_point(OneCriticalGrade([2, 2])));
        // But is connected at grade [4, 4].
        assert!(!region.contains_point(OneCriticalGrade([4, 4])));

        // Vertex 3 is not connected to vertex 4 at grade [5, 5].
        assert!(region.contains_point(OneCriticalGrade([5, 5])));
        // But is connected at grade [6, 6].
        assert!(!region.contains_point(OneCriticalGrade([4, 4])));

        // Vertex 3 is never connected to vertex 5.
        assert!(region.contains_point(OneCriticalGrade([10, 10])));
        assert!(region.contains_point(OneCriticalGrade([1000, 1000])));
        assert!(region.contains_point(OneCriticalGrade([10, 11])));
        assert!(region.contains_point(OneCriticalGrade([11, 10])));
        assert!(!region.contains_point(OneCriticalGrade([9, 10])));
    }
}
