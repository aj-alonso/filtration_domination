//! Algorithms to remove edges from a bifiltered graph while maintaining the topological
//! properties of its clique complex, as described in the paper
//! "Filtration-Domination in Bifiltered Graphs".
//!
//! The two main functions are:
//! - [remove_filtration_dominated], which removes filtration-dominated edges, and
//! - [remove_strongly_filtration_dominated], which removes strongly filtration-dominated edges.
//! See the documentation of the functions, and the paper, for more details.
pub use full::{remove_filtration_dominated, remove_filtration_dominated_timed};
pub use strong::{
    remove_strongly_filtration_dominated, remove_strongly_filtration_dominated_timed,
};

pub mod utils;

mod adjacency;
mod full;
mod strong;

/// The order in which we process the edges, and possibly remove them.
#[derive(Debug, Clone, Copy)]
pub enum EdgeOrder {
    /// Go through the order in reverse lexicographic order.
    /// This is usually the fastest.
    ReverseLexicographic,
    /// Go through the edges in the order they currently have in the edge list.
    Maintain,
}
