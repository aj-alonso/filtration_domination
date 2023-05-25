use ::filtration_domination::edges::{BareEdge, EdgeList, FilteredEdge};
use ::filtration_domination::OneCriticalGrade;
use ::filtration_domination::removal::EdgeOrder;
use ::filtration_domination::points::{Point, PointCloud};
use ::filtration_domination::distance_matrix::density_estimation::DensityEstimator;
use ordered_float::OrderedFloat;
use pyo3::prelude::*;

type Edge = (usize, usize);
type BifilteredEdge = (Edge, (f64, f64));

fn vector_to_edge_list(edges: Vec<BifilteredEdge>) -> EdgeList<FilteredEdge<OneCriticalGrade<OrderedFloat<f64>, 2>>> {
    let mut edge_list = EdgeList::new(0);
    for ((u, v), (g1, g2)) in edges {
        edge_list.add_edge(FilteredEdge {
            grade: OneCriticalGrade([OrderedFloat(g1), OrderedFloat(g2)]),
            edge: BareEdge(u, v),
        });
    }
    edge_list
}

fn edge_list_to_vector(edge_list: &EdgeList<FilteredEdge<OneCriticalGrade<OrderedFloat<f64>, 2>>>) -> Vec<BifilteredEdge> {
    let mut edges = Vec::with_capacity(edge_list.edges().len());
    for e in edge_list.edge_iter() {
        let bare_edge = (e.edge.0, e.edge.1);
        let grade = (e.grade.0[0].0, e.grade.0[1].0);
        edges.push((bare_edge, grade))
    }
    edges
}

fn remove_strongly_filtration_dominated_original(edges: Vec<BifilteredEdge>) -> PyResult<Vec<BifilteredEdge>> {
    let mut edge_list = vector_to_edge_list(edges);
    let reduced = ::filtration_domination::removal::remove_strongly_filtration_dominated(&mut edge_list, EdgeOrder::ReverseLexicographic);
    Ok(edge_list_to_vector(&reduced))
}

#[pyfunction]
fn remove_strongly_filtration_dominated(py: Python<'_>, edges: Vec<BifilteredEdge>) -> PyResult<Vec<BifilteredEdge>> {
    py.allow_threads(|| remove_strongly_filtration_dominated_original(edges))
}

fn remove_filtration_dominated_original(edges: Vec<BifilteredEdge>) -> PyResult<Vec<BifilteredEdge>> {
    let mut edge_list = vector_to_edge_list(edges);
    let reduced = ::filtration_domination::removal::remove_filtration_dominated(&mut edge_list, EdgeOrder::ReverseLexicographic);
    Ok(edge_list_to_vector(&reduced))
}
#[pyfunction]
fn remove_filtration_dominated(py: Python<'_>, edges: Vec<BifilteredEdge>) -> PyResult<Vec<BifilteredEdge>> {
    py.allow_threads(|| remove_filtration_dominated_original(edges))
}

#[pyfunction]
fn gaussian_density_estimation(points: Vec<(f64, f64)>, bandwidth: f64) -> PyResult<Vec<f64>> {
    let points = points.into_iter().map(|(x, y)| Point([x, y])).collect();
    let cloud = PointCloud(points);
    let dist_matrix = cloud.distance_matrix();
    let estimator = DensityEstimator::Gaussian(bandwidth);
    Ok(estimator.estimate(&dist_matrix))
}

#[pymodule]
fn filtration_domination(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    let utils = PyModule::new(_py, "utils")?;
    utils.add_function(wrap_pyfunction!(gaussian_density_estimation, m)?)?;
    m.add_submodule(utils)?;

    m.add_function(wrap_pyfunction!(remove_strongly_filtration_dominated, m)?)?;
    m.add_function(wrap_pyfunction!(remove_filtration_dominated, m)?)?;
    Ok(())
}