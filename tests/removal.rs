use filtration_domination::datasets;
use filtration_domination::datasets::{Dataset, Threshold};
use filtration_domination::mpfree::compute_minimal_presentation;
use filtration_domination::removal::{
    remove_filtration_dominated, remove_strongly_filtration_dominated, EdgeOrder,
};
use paste::paste;

const HOMOLOGY: usize = 1;

macro_rules! test_case {
    ($dataset:expr) => {
        paste!{
#[test]
fn [<$dataset:lower _remove>]() {
    let mut edges = datasets::get_dataset_density_edge_list(
        Dataset::$dataset,
        Threshold::KeepAll,
        None,
        true).unwrap();

    let start = std::time::Instant::now();
    let remaining_edges =
        remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
    let duration = start.elapsed();
    println!("Original edges: {}", edges.len());
    println!("Remaining edges: {}", remaining_edges.len());
    println!("Time spent removing edges: {:?}", duration);

    let mpfree_all_edges =
        compute_minimal_presentation(&format!("test_mpfree_{}", stringify!($dataset)), HOMOLOGY, &edges).unwrap();

    let mpfree_remaining = compute_minimal_presentation(
        &format!("test_mpfree_{}_strong", stringify!($dataset)),
        HOMOLOGY,
        &remaining_edges,
    )
    .unwrap();

    assert_eq!(mpfree_remaining.output, mpfree_all_edges.output);
}

#[test]
fn [<$dataset:lower _remove_strong>]() {
    let mut edges = datasets::get_dataset_density_edge_list(
        Dataset::$dataset,
        Threshold::KeepAll,
        None,
        true).unwrap();

    let start = std::time::Instant::now();
    let remaining_edges =
        remove_strongly_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
    let duration = start.elapsed();
    println!("Original edges: {}", edges.len());
    println!("Remaining edges: {}", remaining_edges.len());
    println!("Time spent removing edges: {:?}", duration);

    let mpfree_all_edges =
        compute_minimal_presentation(&format!("test_mpfree_{}", stringify!($dataset:lower)), HOMOLOGY, &edges).unwrap();

    let mpfree_remaining = compute_minimal_presentation(
        &format!("test_mpfree_{}_strong", stringify!($dataset:lower)),
        HOMOLOGY,
        &remaining_edges,
    )
    .unwrap();

    assert_eq!(mpfree_remaining.output, mpfree_all_edges.output);
}
        }
    }
}

test_case!(Senate);
test_case!(Netwsc);
test_case!(Eleg);
