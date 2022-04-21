use num::Float;
use ordered_float::OrderedFloat;
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::datasets::sampling::{
    sample_noisy_sphere, sample_random_points, sample_swiss_roll, sample_torus,
};
use crate::datasets::{Dataset, DatasetError, Threshold, DATASET_DIRECTORY};
use crate::distance_matrix::input::read_lower_triangular_distance_matrix;
use crate::distance_matrix::output::write_lower_triangular_distance_matrix;
use crate::distance_matrix::DistanceMatrix;
use crate::edges::{EdgeList, FilteredEdge};
use crate::points::PointCloud;
use crate::{OneCriticalGrade, Value};

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

/// Returns the distance matrix of the given dataset.
pub fn get_dataset_distance_matrix(
    dataset: Dataset,
    use_cache: bool,
) -> Result<DistanceMatrix<OrderedFloat<f64>>, DatasetError> {
    let dataset_directory: &Path = Path::new(DATASET_DIRECTORY);
    match dataset {
        Dataset::Senate => read_distance_matrix_from_file(
            dataset_directory.join("senate104_edge_list.txt_0.68902_distmat.txt"),
        ),
        Dataset::Eleg => read_distance_matrix_from_file(dataset_directory.join(
            "celegans_weighted_undirected_reindexed_for_matlab_maxdist_2.6429_SP_distmat.txt",
        )),
        Dataset::Netwsc => read_distance_matrix_from_file(
            dataset_directory.join("network379_edge_list.txt_38.3873_distmat.txt"),
        ),
        Dataset::Hiv => read_distance_matrix_from_file(
            dataset_directory.join("HIV1_2011.all.nt.concat.fa_hdm.txt"),
        ),
        Dataset::Dragon => read_distance_matrix_from_file(
            dataset_directory.join("dragon_vrip.ply.txt_2000_.txt_distmat.txt"),
        ),
        Dataset::Sphere { n_points } => {
            let dst_filename = dataset_directory.join(format!("sphere_{n_points}_distmat.txt"));
            read_or_save_distance_matrix(
                dst_filename,
                || {
                    sample_distance_matrix(n_points, |n| {
                        sample_noisy_sphere::<f64, 3>(n, 0.9, 0.75, 0.3)
                    })
                },
                use_cache,
            )
        }
        Dataset::Torus { n_points } => {
            let dst_filename = dataset_directory.join(format!("torus_{n_points}_distmat.txt"));
            read_or_save_distance_matrix(
                dst_filename,
                || sample_distance_matrix(n_points, sample_torus),
                use_cache,
            )
        }
        Dataset::SwissRoll { n_points } => {
            let dst_filename = dataset_directory.join(format!("swiss_roll_{n_points}_distmat.txt"));
            read_or_save_distance_matrix(
                dst_filename,
                || sample_distance_matrix(n_points, sample_swiss_roll),
                use_cache,
            )
        }
        Dataset::Circle { n_points } => {
            let dst_filename = dataset_directory.join(format!("circle_{n_points}_distmat.txt"));
            read_or_save_distance_matrix(
                dst_filename,
                || {
                    sample_distance_matrix(n_points, |n| {
                        sample_noisy_sphere::<f64, 2>(n, 1., 0., 0.)
                    })
                },
                use_cache,
            )
        }
        Dataset::Uniform { n_points } => {
            let dst_filename = dataset_directory.join(format!("uniform_{n_points}_distmat.txt"));
            read_or_save_distance_matrix(
                dst_filename,
                || sample_distance_matrix(n_points, sample_random_points::<f64, 2>),
                use_cache,
            )
        }
    }
}

fn read_distance_matrix_from_file<P: AsRef<Path>>(
    filepath: P,
) -> Result<DistanceMatrix<OrderedFloat<f64>>, DatasetError> {
    if !filepath.as_ref().is_file() {
        return Err(DatasetError::FileNotFound(format!(
            "{}",
            filepath.as_ref().display()
        )));
    }
    let file = fs::File::open(filepath)?;
    let reader = BufReader::new(&file);
    let distance_matrix = read_lower_triangular_distance_matrix(reader)?;

    Ok(distance_matrix)
}

fn sample_distance_matrix<F: Fn(usize) -> PointCloud<f64, N>, const N: usize>(
    n_points: usize,
    f: F,
) -> DistanceMatrix<OrderedFloat<f64>> {
    let points: PointCloud<OrderedFloat<f64>, N> = f(n_points).into();
    points.distance_matrix()
}

fn read_or_save_distance_matrix<
    P: AsRef<Path>,
    F: FnOnce() -> DistanceMatrix<OrderedFloat<f64>>,
>(
    dst_filename: P,
    distance_matrix_builder: F,
    use_cache: bool,
) -> Result<DistanceMatrix<OrderedFloat<f64>>, DatasetError> {
    if dst_filename.as_ref().is_file() && use_cache {
        read_distance_matrix_from_file(dst_filename)
    } else {
        let distance_matrix = distance_matrix_builder();

        if use_cache {
            let dst_file = fs::File::create(dst_filename)?;
            let mut dst_writer = BufWriter::new(dst_file);
            write_lower_triangular_distance_matrix(&distance_matrix, &mut dst_writer)?;
        }

        Ok(distance_matrix)
    }
}

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
