use filtration_domination::edges::{EdgeList, FilteredEdge};
use filtration_domination::{OneCriticalGrade, Value};
use rand::distributions::uniform::SampleUniform;
use rand::distributions::Uniform;
use rand::Rng;

pub fn delete_densities<VF: Value>(
    edge_list: &EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
) -> EdgeList<FilteredEdge<OneCriticalGrade<VF, 1>>> {
    let mut new_edge_list = EdgeList::new(edge_list.n_vertices);

    for edge in edge_list.edge_iter() {
        new_edge_list.add_edge(FilteredEdge {
            grade: OneCriticalGrade([edge.grade.0[1]]),
            edge: edge.edge,
        })
    }

    new_edge_list
}

pub fn forget_densities<VF: Value>(
    edge_list: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
) {
    for edge in edge_list.edges_mut() {
        edge.grade.0[0] = VF::zero();
    }
}

pub fn random_densities<VF: Value + SampleUniform>(
    edge_list: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
) {
    let distribution = Uniform::new(VF::zero(), VF::max_value());
    let mut rng = rand::thread_rng();
    for edge in edge_list.edges_mut() {
        edge.grade.0[0] = rng.sample(&distribution);
    }
}

pub fn zero_grades<VF: Value>(edge_list: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>) {
    for edge in edge_list.edges_mut() {
        edge.grade.0[0] = VF::zero();
        edge.grade.0[1] = VF::zero();
    }
}