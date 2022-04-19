use clap::Parser;
use filtration_domination::datasets;
use filtration_domination::datasets::Threshold;
use filtration_domination::distance_matrix::density_estimation::DensityEstimator;
use filtration_domination::mpfree::compute_minimal_presentation;
use filtration_domination::removal::{
    remove_filtration_dominated, remove_strongly_filtration_dominated, EdgeOrder,
};
use ordered_float::OrderedFloat;
use std::fmt::Formatter;

const HOMOLOGY: usize = 1;

/// Run experiments for edge collapse
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct RunCli {
    /// Dataset in which to remove edges.
    #[clap(arg_enum)]
    dataset: Dataset,

    /// Whether to use strong filtration-domination.
    #[clap(short, long)]
    strong: bool,

    /// Bandwidth value to do density estimation.
    #[clap(short, long)]
    bandwidth: Option<f64>,

    /// Maximum value on the distance.
    #[clap(short, long)]
    threshold: Option<f64>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, clap::ArgEnum)]
enum Dataset {
    Senate,
    Eleg,
    Netwsc,
    Hiv,
    Dragon,
    Sphere,
    Uniform,
    Circle,
    Torus,
    SwissRoll,
}

impl Dataset {
    fn to_internal_dataset(self, n_points: Option<usize>) -> datasets::Dataset {
        match self {
            Dataset::Senate => datasets::Dataset::Senate,
            Dataset::Eleg => datasets::Dataset::Eleg,
            Dataset::Netwsc => datasets::Dataset::Netwsc,
            Dataset::Hiv => datasets::Dataset::Hiv,
            Dataset::Dragon => datasets::Dataset::Dragon,
            Dataset::Uniform => datasets::Dataset::Uniform {
                n_points: n_points.unwrap_or(400),
            },
            Dataset::Sphere => datasets::Dataset::Sphere {
                n_points: n_points.unwrap_or(100),
            },
            Dataset::Circle => datasets::Dataset::Circle {
                n_points: n_points.unwrap_or(100),
            },
            Dataset::Torus => datasets::Dataset::Torus {
                n_points: n_points.unwrap_or(200),
            },
            Dataset::SwissRoll => datasets::Dataset::SwissRoll {
                n_points: n_points.unwrap_or(200),
            },
        }
    }
}

impl std::fmt::Display for Dataset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Dataset::Senate => "senate",
                Dataset::Eleg => "eleg",
                Dataset::Netwsc => "netwsc",
                Dataset::Hiv => "hiv",
                Dataset::Dragon => "dragon",
                Dataset::Sphere => "uniform",
                Dataset::Uniform => "sphere",
                Dataset::Circle => "circle",
                Dataset::Torus => "torus",
                Dataset::SwissRoll => "swiss roll",
            }
        )
    }
}

fn main() -> anyhow::Result<()> {
    let opts: RunCli = RunCli::parse();

    let dataset = opts.dataset.to_internal_dataset(None);

    let mut edges = datasets::get_dataset_density_edge_list(
        dataset,
        opts.threshold.map_or(Threshold::KeepAll, Threshold::Fixed),
        opts.bandwidth
            .map(|b| DensityEstimator::Gaussian(OrderedFloat(b))),
        true,
    )?;

    println!("Starting removal...");
    let start = std::time::Instant::now();
    let remaining_edges = if opts.strong {
        remove_strongly_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic)
    } else {
        remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic)
    };
    let duration = start.elapsed();

    println!("Original edges: {}", edges.len());
    println!("Remaining edges: {}", remaining_edges.len());
    println!(
        "Ratio: {:.2}",
        (remaining_edges.len() as f64) / (edges.len() as f64)
    );
    println!("Removal took {duration:?}");

    println!("Running mpfree on full edges...");
    let mpfree_no_collapse =
        compute_minimal_presentation(&format!("test_mpfree_{}", dataset), HOMOLOGY, &edges)?;

    println!("Running mpfree on remaining edges...");
    let mpfree_collapse = compute_minimal_presentation(
        &format!("test_mpfree_{}_strong_collapse", dataset),
        HOMOLOGY,
        &remaining_edges,
    )?;

    assert_eq!(mpfree_collapse.output, mpfree_no_collapse.output);
    println!("{:?}", mpfree_collapse.output);

    Ok(())
}
