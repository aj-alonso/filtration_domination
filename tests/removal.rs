use filtration_domination::datasets;
use filtration_domination::datasets::{Dataset, Threshold};
use filtration_domination::mpfree::compute_minimal_presentation;
use filtration_domination::removal::{
    remove_filtration_dominated, remove_strongly_filtration_dominated, EdgeOrder,
};
use paste::paste;

const HOMOLOGY: usize = 1;

/// Create test cases that remove edges from the bifiltered graph, and check that
/// the multiparameter persistent homology is preserved
/// (by checking minimal presentations with mpfree).
macro_rules! test_case {
    ($name:expr, $dataset:expr) => {
        paste!{
#[test]
fn [<$name _remove>]() {
    let mut edges = datasets::get_dataset_density_edge_list(
        Dataset::$dataset,
        Threshold::KeepAll,
        None,
        true).unwrap();

    let remaining_edges =
        remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
    println!("Original edges: {}", edges.len());
    println!("Remaining edges: {}", remaining_edges.len());

    let mpfree_all_edges =
        compute_minimal_presentation(&format!("test_mpfree_{}", stringify!($name)), HOMOLOGY, &edges).unwrap();

    let mpfree_remaining = compute_minimal_presentation(
        &format!("test_mpfree_{}_remaining", stringify!($name)),
        HOMOLOGY,
        &remaining_edges,
    )
    .unwrap();

    assert_eq!(mpfree_remaining.output, mpfree_all_edges.output);
}

#[test]
fn [<$name _remove_strong>]() {
    let mut edges = datasets::get_dataset_density_edge_list(
        Dataset::$dataset,
        Threshold::KeepAll,
        None,
        true).unwrap();

    let remaining_edges =
        remove_strongly_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
    println!("Original edges: {}", edges.len());
    println!("Remaining edges: {}", remaining_edges.len());

    let mpfree_all_edges =
        compute_minimal_presentation(&format!("test_mpfree_{}_strong", stringify!($name)), HOMOLOGY, &edges).unwrap();

    let mpfree_remaining = compute_minimal_presentation(
        &format!("test_mpfree_{}_strong_remaining", stringify!($name)),
        HOMOLOGY,
        &remaining_edges,
    )
    .unwrap();

    assert_eq!(mpfree_remaining.output, mpfree_all_edges.output);
}
        }
    }
}

test_case!(senate, Senate);
test_case!(netwsc, Netwsc);
test_case!(eleg, Eleg);

test_case!(uniform, Uniform { n_points: 400 });
test_case!(sphere, Sphere { n_points: 100 });
test_case!(circle, Circle { n_points: 100 });
test_case!(torus, Torus { n_points: 200 });
test_case!(swiss_roll, SwissRoll { n_points: 200 });
