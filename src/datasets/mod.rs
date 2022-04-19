use std::fmt::Formatter;

mod distance_matrices;
mod graphs;
mod points;

pub use graphs::get_dataset_density_edge_list;

const DATASET_DIRECTORY: &str = "datasets";

#[derive(Debug, Copy, Clone)]
pub enum Dataset {
    Senate,
    Eleg,
    Netwsc,
    Hiv,
    Dragon,
    Circle { n_points: usize },
    Sphere { n_points: usize },
    Torus { n_points: usize },
    SwissRoll { n_points: usize },
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
