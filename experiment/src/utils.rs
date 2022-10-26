use filtration_domination::edges::{EdgeList, FilteredEdge};
use filtration_domination::{CriticalGrade, OneCriticalGrade, Value};
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

pub fn critical_values<VF: Value, const N: usize>(
    edges: &mut [FilteredEdge<OneCriticalGrade<VF, N>>],
) -> Vec<OneCriticalGrade<usize, N>> {
    let mut critical_values: Vec<OneCriticalGrade<usize, N>> =
        vec![OneCriticalGrade::min_value(); edges.len()];
    let mut n_critical_values = [0; N];
    let mut indexes: Vec<usize> = (0..edges.len()).collect();
    for (n, critical_value_counter) in n_critical_values.iter_mut().enumerate() {
        // Sort on the n-th grade component.
        indexes.sort_by(|&a_idx, &b_idx| edges[a_idx].grade[n].cmp(&edges[b_idx].grade[n]));
        for (i, &idx) in indexes.iter().enumerate() {
            if i != 0 {
                let prev_idx = indexes[i - 1];
                if edges[prev_idx].grade[n] != edges[idx].grade[n] {
                    *critical_value_counter += 1;
                }
            }
            critical_values[idx][n] = *critical_value_counter;
        }
    }
    critical_values
}

pub fn normalize<VF: Value, const N: usize>(
    edge_list: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, N>>>,
) -> EdgeList<FilteredEdge<OneCriticalGrade<usize, N>>> {
    let edges = edge_list.edges_mut();
    let critical = critical_values(edges);
    let mut new_edges = Vec::with_capacity(edges.len());
    for (i, e) in edges.iter().enumerate() {
        new_edges.push(FilteredEdge {
            grade: critical[i],
            edge: e.edge,
        });
    }
    new_edges.into()
}
