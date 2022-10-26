use clap::Args;
use std::time::Duration;

use filtration_domination::datasets;
use filtration_domination::datasets::Threshold;
use filtration_domination::removal::{
    remove_filtration_dominated, remove_strongly_filtration_dominated, EdgeOrder,
};

use crate::{
    display, display_duration, save_table, Algorithm, CliDataset, Row, Table, ALL_DATASETS,
};

#[derive(Debug, Args)]
pub struct AsymptoticCli {
    #[clap(arg_enum)]
    datasets: Vec<CliDataset>,

    #[clap(short, default_value_t = 25)]
    iterations: usize,

    #[clap(short, default_value_t = 5)]
    repeats: usize,

    /// Number of points to use on dynamic datasets.
    #[clap(short)]
    n_points: Option<usize>,

    /// New points to add in each iteration.
    #[clap(short, default_value_t = 20)]
    step: usize,

    #[clap(short)]
    full: bool,
}

#[derive(Debug)]
struct AsymptoticsRow {
    dataset: CliDataset,
    n_points: usize,
    max_degree: usize,
    edges_before_collapse: usize,
    edges_after_collapse: usize,
    collapse_duration: Duration,
    algorithm: Algorithm,
}

impl Row for AsymptoticsRow {
    fn headers() -> Vec<&'static str> {
        vec![
            "Dataset",
            "Points",
            "Algorithm",
            "MaxDegree",
            "Before",
            "After",
            "Time",
        ]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset)),
            Some(display(self.n_points)),
            Some(display(self.algorithm)),
            Some(display(self.max_degree)),
            Some(display(self.edges_before_collapse)),
            Some(display(self.edges_after_collapse)),
            Some(display_duration(&self.collapse_duration)),
        ]
    }
}

pub fn compare_asymptotics(opts: AsymptoticCli) -> anyhow::Result<()> {
    let datasets = if opts.datasets.is_empty() {
        Vec::from(ALL_DATASETS)
    } else {
        opts.datasets
    };

    let mut rows: Vec<AsymptoticsRow> = Vec::new();
    for dataset in datasets {
        if dataset.default_n_points().is_none() {
            println!(
                "Skipping dataset {} because it has a fixed number of points",
                dataset
            );
            continue;
        }
        println!("Processing dataset {}", dataset);
        let mut n_points = opts
            .n_points
            .or_else(|| dataset.default_n_points())
            .unwrap();

        for _i in 0..opts.iterations {
            for _r in 0..opts.repeats {
                println!("Sampling {n_points} points");
                let mut edges = datasets::get_dataset_density_edge_list(
                    dataset.to_internal_dataset(Some(n_points)),
                    Threshold::KeepAll,
                    None,
                    false, // Do not use cache, we want to generate a new dataset each time.
                )?;
                let edges_before_collapse = edges.len();
                let max_degree = edges.maximum_degree();
                println!("Got {edges_before_collapse} edges");

                let start = std::time::Instant::now();
                let (collapsed_edges, algorithm) = if opts.full {
                    let collapsed_edges =
                        remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
                    (collapsed_edges, Algorithm::FiltrationDomination)
                } else {
                    let collapsed_edges = remove_strongly_filtration_dominated(
                        &mut edges,
                        EdgeOrder::ReverseLexicographic,
                    );
                    (collapsed_edges, Algorithm::StrongFiltrationDomination)
                };
                let duration = start.elapsed();

                let row = AsymptoticsRow {
                    dataset,
                    n_points: edges.n_vertices,
                    max_degree,
                    edges_before_collapse,
                    edges_after_collapse: collapsed_edges.len(),
                    collapse_duration: duration,
                    algorithm,
                };

                rows.push(row);
            }
            n_points += opts.step;
        }
    }

    save_table(Table::new(rows), "compare_asymptotics")?;

    Ok(())
}
