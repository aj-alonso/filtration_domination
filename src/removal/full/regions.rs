use std::cmp::Ordering;

use crate::edges::FilteredEdge;
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::full::stripes::{Stripe, Stripes};
use crate::{CriticalGrade, OneCriticalGrade, Value};

pub type Pair<VF> = (OneCriticalGrade<VF, 2>, OneCriticalGrade<VF, 2>);

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

    let mut edge_neighs = adjacency_matrix.closed_neighbours_edge(edge);
    let mut v_neighs = adjacency_matrix
        .closed_neighbours(v, value_v.join(&edge.grade))
        .peekable();
    for (a, value_a) in edge_neighs {
        if a == v {
            continue;
        }
        if let Some(value_b) = adjacency_matrix.matrix[v].get(&a) {
            add_pair(
                &mut vertical_stripes,
                &mut horizontal_stripes,
                (value_a.clone(), value_a.join(value_b)),
            );
        } else {
            add_pair(
                &mut vertical_stripes,
                &mut horizontal_stripes,
                (value_a.clone(), OneCriticalGrade::max_value()),
            );
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
