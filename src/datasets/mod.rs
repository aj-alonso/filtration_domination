//! Dataset reading and sampling.
//!
//! The main entry point is [get_dataset_density_edge_list], which returns a bifiltered edge list.
use ordered_float::OrderedFloat;
use std::cmp::max;
use std::fmt::Formatter;
use std::io;
use thiserror::Error;

use crate::datasets::distance_matrices::get_dataset_distance_matrix;
use crate::distance_matrix::density_estimation::DensityEstimator;
use crate::distance_matrix::DistanceMatrix;
use crate::edges::{BareEdge, EdgeList, FilteredEdge};
use crate::{OneCriticalGrade, Value};

mod distance_matrices;
mod sampling;

const DATASET_DIRECTORY: &str = "datasets";

/// All datasets that we support.
#[derive(Debug, Copy, Clone)]
pub enum Dataset {
    /// The senate dataset from <https://github.com/n-otter/PH-roadmap>.
    Senate,
    /// The eleg dataset from <https://github.com/n-otter/PH-roadmap>.
    Eleg,
    /// The netwsc dataset from <https://github.com/n-otter/PH-roadmap>.
    Netwsc,
    /// The hiv dataset from <https://github.com/n-otter/PH-roadmap>.
    Hiv,
    /// The dragon dataset from <https://github.com/n-otter/PH-roadmap>.
    Dragon,
    /// A circle in R^2.
    Circle { n_points: usize },
    /// A noisy sphere in R^3.
    Sphere { n_points: usize },
    /// A torus sphere in R^3.
    Torus { n_points: usize },
    /// A swiss roll, that is, a plane rolled up in a spiral in R^3.
    SwissRoll { n_points: usize },
    /// Points sampled uniformly from a square in the plane.
    Uniform { n_points: usize },
}

impl std::fmt::Display for Dataset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Dataset::Senate => {
                write!(f, "senate")
            }
            Dataset::Eleg => {
                write!(f, "eleg")
            }
            Dataset::Netwsc => {
                write!(f, "netwsc")
            }
            Dataset::Hiv => {
                write!(f, "hiv")
            }
            Dataset::Dragon => {
                write!(f, "dragon")
            }
            Dataset::Circle { n_points } => {
                write!(f, "circle({n_points})")
            }
            Dataset::Sphere { n_points } => {
                write!(f, "sphere({n_points})")
            }
            Dataset::Torus { n_points } => {
                write!(f, "torus({n_points})")
            }
            Dataset::SwissRoll { n_points } => {
                write!(f, "swiss-roll({n_points})")
            }
            Dataset::Uniform { n_points } => {
                write!(f, "uniform({n_points})")
            }
        }
    }
}

/// Possible thresholding settings.
#[derive(Debug, Copy, Clone)]
pub enum Threshold {
    /// Keep all edges.
    KeepAll,
    /// Restrict to the edges of length less than the given percentile of all distances.
    Percentile(f64),
    /// Restrict to the edges of length less that the given value.
    Fixed(f64),
}

/// Error when reading or creating a dataset.
#[derive(Error, Debug)]
pub enum DatasetError {
    #[error("Couldn't find file \"{0}\". Did you download the datasets?")]
    FileNotFound(String),

    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Return the edge list of the associated dataset. Each edge is bifiltered by codensity and length.
/// Codensity means that we order the density parameter from densest to least dense.
///
/// Possibly removes some edges according to `threshold`. See [Threshold].
/// If a `estimator` is not provided, the function uses the Gaussian kernel estimator with
/// bandwidth parameter set to the 20th percentile of the distances.
/// If `use_cache` is set, the function caches the distance matrices of the sampled datasets.
pub fn get_dataset_density_edge_list(
    dataset: Dataset,
    threshold: Threshold,
    estimator: Option<DensityEstimator<OrderedFloat<f64>>>,
    use_cache: bool,
) -> Result<EdgeList<FilteredEdge<OneCriticalGrade<OrderedFloat<f64>, 2>>>, DatasetError> {
    let distance_matrix = get_dataset_distance_matrix(dataset, use_cache)?;

    let estimator = estimator.unwrap_or_else(|| default_estimator(&distance_matrix));
    let mut estimations = estimator.estimate(&distance_matrix);
    // Instead of working with densities, we work with codensities. That is, smaller values correspond
    // to higher density estimations.
    for e in estimations.iter_mut() {
        *e = OrderedFloat::from(1.0) - *e;
    }

    let edges = distance_matrices::get_distance_matrix_edge_list(&distance_matrix, threshold);

    let density_edges_it = edges.edges().iter().map(|edge| {
        let FilteredEdge {
            grade: OneCriticalGrade([dist]),
            edge: BareEdge(u, v),
        } = edge;

        // The edge density is the max of the codensity of its vertices.
        let edge_density = max(estimations[*u], estimations[*v]);

        FilteredEdge {
            grade: OneCriticalGrade([edge_density, *dist]),
            edge: BareEdge(*u, *v),
        }
    });

    Ok(EdgeList::from_iterator(density_edges_it))
}

fn default_estimator<F: Value + std::fmt::Display>(
    matrix: &DistanceMatrix<F>,
) -> DensityEstimator<F> {
    let bandwidth = matrix.percentile(0.2);
    DensityEstimator::Gaussian(*bandwidth)
}
