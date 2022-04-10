use crate::edges::{EdgeList, FilteredEdge};
use crate::removal::adjacency::AdjacencyMatrix;
use crate::removal::strong::is_subset;
use crate::removal::EdgeOrder;
use crate::CriticalGrade;
use rayon::prelude::{ParallelBridge, ParallelIterator};

pub fn remove_strongly_filtration_dominated_multithread<G: CriticalGrade>(
    edge_list: &mut EdgeList<FilteredEdge<G>>,
    order: EdgeOrder,
) -> EdgeList<FilteredEdge<G>> {
    match order {
        EdgeOrder::ReverseLexicographic => {
            edge_list.edges_mut().sort_by(|a, b| b.cmp(a));
        }
        EdgeOrder::Maintain => {}
    }

    let mut critical_edges: Vec<FilteredEdge<G>> = Vec::with_capacity(edge_list.len());
    let mut adjacency_matrix = AdjacencyMatrix::new(edge_list.n_vertices);

    for edge in edge_list.edge_iter() {
        adjacency_matrix.add_edge(edge.clone());
    }

    for edge in edge_list.edge_iter() {
        if is_stringly_filtration_dominated_par(&adjacency_matrix, edge) {
            adjacency_matrix.delete_edge(edge);
        } else {
            critical_edges.push(edge.clone());
        }
    }

    critical_edges.shrink_to_fit();
    critical_edges.into()
}

fn is_stringly_filtration_dominated_par<G: CriticalGrade>(
    adjacency_matrix: &AdjacencyMatrix<G>,
    edge: &FilteredEdge<G>,
) -> bool {
    adjacency_matrix
        .common_neighbours(edge)
        .par_bridge()
        .any(|(v, value_v)| {
            let edge_neighs = adjacency_matrix.closed_neighbours_edge(edge);
            let v_neighs = adjacency_matrix.closed_neighbours(v, value_v.join(&edge.grade));
            is_subset(edge_neighs, v_neighs)
        })
}
