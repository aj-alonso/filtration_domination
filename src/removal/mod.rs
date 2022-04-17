pub mod utils;

pub use full::{remove_filtration_dominated, remove_filtration_dominated_timed};
pub use strong::{
    remove_strongly_filtration_dominated, remove_strongly_filtration_dominated_timed,
};

#[derive(Debug, Clone, Copy)]
pub enum EdgeOrder {
    ReverseLexicographic,
    Maintain,
}

mod adjacency;
mod full;
mod strong;
