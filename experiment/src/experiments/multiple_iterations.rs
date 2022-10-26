use std::time::Duration;
use clap::Args;

use filtration_domination::datasets;
use filtration_domination::datasets::Threshold;
use filtration_domination::removal::{EdgeOrder, remove_strongly_filtration_dominated};
use crate::{ALL_DATASETS, CliDataset, display, display_duration, Row, save_table, Table};

#[derive(Debug, Args)]
pub struct MultipleIterationsCli {
    #[clap(arg_enum)]
    datasets: Vec<CliDataset>,

    /// Number of iterations to run per dataset.
    #[clap(short, default_value_t = 6)]
    iterations: usize,
}

#[derive(Debug)]
struct MultipleIterationsRow {
    dataset: CliDataset,
    iteration: usize,
    edges: usize,
    collapse_duration: Duration,
}

impl Row for MultipleIterationsRow {
    fn headers() -> Vec<&'static str> {
        vec!["Dataset", "Iteration", "Edges", "Time"]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset.to_static_str())),
            Some(display(self.iteration)),
            Some(display(self.edges)),
            Some(display_duration(&self.collapse_duration)),
        ]
    }
}

pub fn compare_multiple_iterations(opts: MultipleIterationsCli) -> anyhow::Result<()> {
    let datasets = if opts.datasets.is_empty() {
        Vec::from(ALL_DATASETS)
    } else {
        opts.datasets
    };

    let mut rows: Vec<MultipleIterationsRow> = Vec::new();
    for dataset in datasets {
        println!("Processing dataset {}", dataset);
        let mut edges = datasets::get_dataset_density_edge_list(
            dataset.to_internal_dataset(None),
            Threshold::KeepAll,
            None,
            true,
        )?;
        rows.push(MultipleIterationsRow {
            dataset,
            iteration: 0,
            edges: edges.len(),
            collapse_duration: Default::default(),
        });
        let start = std::time::Instant::now();
        for i in 1..=opts.iterations {
            let collapsed_edges =
                remove_strongly_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);

            let duration = start.elapsed();
            let row = MultipleIterationsRow {
                dataset,
                iteration: i,
                edges: collapsed_edges.len(),
                collapse_duration: duration,
            };

            rows.push(row);

            edges = collapsed_edges;
        }
    }

    save_table(Table::new(rows), "compare_multiple_iterations")?;

    Ok(())
}
