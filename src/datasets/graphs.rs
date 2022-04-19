use crate::datasets::distance_matrices::get_dataset_distance_matrix;
use crate::datasets::Dataset;

use num::Float;
use ordered_float::OrderedFloat;
use std::cmp::max;
use std::io;

use crate::distance_matrix::density_estimation::DensityEstimator;
use crate::distance_matrix::DistanceMatrix;
use crate::edges::{BareEdge, EdgeList, FilteredEdge};
use crate::{OneCriticalGrade, Value};

fn filter_by_threshold<
    'a,
    VF: Value + Float + 'a,
    I: Iterator<Item = FilteredEdge<OneCriticalGrade<VF, N>>> + 'a,
    const N: usize,
>(
    edge_iter: I,
    threshold: VF,
) -> impl Iterator<Item = FilteredEdge<OneCriticalGrade<VF, N>>> + 'a {
    edge_iter.filter(move |&FilteredEdge { grade, edge: _ }| grade.0[N - 1] < threshold)
}

/// Build an edge list out of a distance matrix. Each edge is graded by the distance between its
/// vertices.
/// If `threshold` is given, edges of grade less than `threshold` are not included.
/// If `threshold` is not given then it is set to the enclosing radius.
pub fn get_distance_matrix_edge_list(
    distance_matrix: &DistanceMatrix<OrderedFloat<f64>>,
    threshold: Threshold,
) -> EdgeList<FilteredEdge<OneCriticalGrade<OrderedFloat<f64>, 1>>> {
    let edges = distance_matrix.edges();

    let actual_threshold: Option<OrderedFloat<f64>> = match threshold {
        Threshold::KeepAll => None,
        Threshold::Percentile(p) => Some(*distance_matrix.percentile(p)),
        Threshold::Fixed(t) => Some(OrderedFloat::from(t)),
    };

    if let Some(threshold_value) = actual_threshold {
        EdgeList::from_iterator(filter_by_threshold(edges, threshold_value))
    } else {
        EdgeList::from_iterator(edges)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Threshold {
    KeepAll,
    Percentile(f64),
    Fixed(f64),
}

pub fn get_dataset_density_edge_list(
    dataset: Dataset,
    threshold: Threshold,
    estimator: Option<DensityEstimator<OrderedFloat<f64>>>,
    use_cache: bool,
) -> io::Result<EdgeList<FilteredEdge<OneCriticalGrade<OrderedFloat<f64>, 2>>>> {
    let distance_matrix = get_dataset_distance_matrix(dataset, use_cache)?;

    let estimator = estimator.unwrap_or_else(|| default_estimator(&distance_matrix));
    let mut estimations = estimator.estimate(&distance_matrix);
    // Instead of working with densities, we work with codensities. That is, smaller values correspond
    // to more dense vertices.
    for e in estimations.iter_mut() {
        *e = OrderedFloat::from(1.0) - *e;
    }

    let edges = get_distance_matrix_edge_list(&distance_matrix, threshold);

    let density_edges_it = edges.edges().iter().map(|edge| {
        let FilteredEdge {
            grade: OneCriticalGrade([dist]),
            edge: BareEdge(u, v),
        } = edge;

        // The edge density is the max of the codensity of its vertices.
        let edge_density = max(estimations[*u as usize], estimations[*v as usize]);

        FilteredEdge {
            grade: OneCriticalGrade([edge_density, *dist]),
            edge: BareEdge(*u, *v),
        }
    });

    Ok(EdgeList::from_iterator(density_edges_it))
}

pub fn default_estimator<F: Value + std::fmt::Display>(
    matrix: &DistanceMatrix<F>,
) -> DensityEstimator<F> {
    let bandwidth = matrix.percentile(0.2);
    DensityEstimator::Gaussian(*bandwidth)
}
