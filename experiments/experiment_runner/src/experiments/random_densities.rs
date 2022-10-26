use clap::Args;

use crate::experiments::orders::Order;
use crate::utils::{forget_densities, random_densities, zero_grades};
use crate::{display, display_duration, save_table, CliDataset, Row, Table, ALL_DATASETS};
use filtration_domination::datasets;
use filtration_domination::datasets::Threshold;
use filtration_domination::removal::utils::count_isolated_edges;
use filtration_domination::removal::{remove_strongly_filtration_dominated_timed, EdgeOrder};
use std::fmt::Formatter;
use std::time::Duration;

const TIMEOUT_DURATION_RANDOM_DENSITIES: Duration = Duration::from_secs(60 * 60 * 2);

#[derive(Debug, Args)]
pub struct RandomDensitiesCli {
    #[clap(arg_enum)]
    datasets: Vec<CliDataset>,

    /// Whether to include a run with the colexicographic order or not.
    #[clap(short, long)]
    colexicograhic: bool,

    /// Only do runs with the random densities modification.
    #[clap(long)]
    only_random: bool,

    /// Timeout when doing runs in seconds.
    #[clap(short, default_value_t = 60 * 30)]
    timeout: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum GradeStructure {
    Zero,
    NoDensities,
    Normal,
    RandomDensities,
}

impl std::fmt::Display for GradeStructure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GradeStructure::Zero => {
                    "zero"
                }
                GradeStructure::NoDensities => {
                    "no-densities"
                }
                GradeStructure::Normal => {
                    "normal"
                }
                GradeStructure::RandomDensities => {
                    "random"
                }
            }
        )
    }
}

#[derive(Debug)]
struct RandomDensitiesRow {
    dataset: CliDataset,
    n_points: usize,
    grade_structure: GradeStructure,
    order: Order,
    edges_before_collapse: usize,
    edges_after_collapse: usize,
    collapse_duration: Duration,
    isolated: usize,
    dominated: usize,
}

impl Row for RandomDensitiesRow {
    fn headers() -> Vec<&'static str> {
        vec![
            "Dataset",
            "Points",
            "Structure",
            "Order",
            "Before",
            "After",
            "Time",
            "Isolated",
            "Dominated",
        ]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset)),
            Some(display(self.n_points)),
            Some(display(self.grade_structure)),
            Some(display(self.order)),
            Some(display(self.edges_before_collapse)),
            Some(display(self.edges_after_collapse)),
            Some(display_duration(&self.collapse_duration)),
            Some(display(self.isolated)),
            Some(display(self.dominated)),
        ]
    }
}
pub fn compare_random_densities(opts: RandomDensitiesCli) -> anyhow::Result<()> {
    let datasets = if opts.datasets.is_empty() {
        Vec::from(ALL_DATASETS)
    } else {
        opts.datasets
    };

    let mut rows: Vec<RandomDensitiesRow> = Vec::new();
    for dataset in datasets {
        println!("Processing dataset {}", dataset);

        let mut edges = datasets::get_dataset_density_edge_list(
            dataset.to_internal_dataset(None),
            Threshold::KeepAll,
            None,
            true,
        )?;

        let mut zero_density_edges = edges.clone();
        forget_densities(&mut zero_density_edges);

        let mut zero_grades_edges = edges.clone();
        zero_grades(&mut zero_grades_edges);

        let edges_before_collapse = edges.len();
        let n_points = edges.n_vertices;

        let mut edges_random_densities = edges.clone();
        random_densities(&mut edges_random_densities);

        if opts.colexicograhic {
            let (isolated, dominated) = count_isolated_edges(&edges_random_densities);
            edges_random_densities.sort_reverse_colexicographically();

            let start = std::time::Instant::now();
            let resulting_edges = remove_strongly_filtration_dominated_timed(
                &mut edges_random_densities,
                EdgeOrder::Maintain,
                Some(TIMEOUT_DURATION_RANDOM_DENSITIES),
            );
            let edges_after_collapse = resulting_edges.len();

            rows.push(RandomDensitiesRow {
                dataset,
                n_points,
                grade_structure: GradeStructure::RandomDensities,
                order: Order::ReverseColexicographic,
                edges_before_collapse,
                edges_after_collapse,
                collapse_duration: start.elapsed(),
                isolated,
                dominated,
            });
        }

        {
            let (isolated, dominated) = count_isolated_edges(&edges_random_densities);
            let start = std::time::Instant::now();
            let resulting_edges = remove_strongly_filtration_dominated_timed(
                &mut edges_random_densities,
                EdgeOrder::ReverseLexicographic,
                Some(TIMEOUT_DURATION_RANDOM_DENSITIES),
            );
            let edges_after_collapse = resulting_edges.len();

            rows.push(RandomDensitiesRow {
                dataset,
                n_points,
                grade_structure: GradeStructure::RandomDensities,
                order: Order::ReverseLexicographic,
                edges_before_collapse,
                edges_after_collapse,
                collapse_duration: start.elapsed(),
                isolated,
                dominated,
            });
        }

        if !opts.only_random {
            let (isolated, dominated) = count_isolated_edges(&zero_density_edges);
            let start = std::time::Instant::now();
            let resulting_edges = remove_strongly_filtration_dominated_timed(
                &mut zero_density_edges,
                EdgeOrder::ReverseLexicographic,
                Some(TIMEOUT_DURATION_RANDOM_DENSITIES),
            );
            let edges_after_collapse = resulting_edges.len();

            rows.push(RandomDensitiesRow {
                dataset,
                n_points,
                grade_structure: GradeStructure::NoDensities,
                order: Order::ReverseLexicographic,
                edges_before_collapse,
                edges_after_collapse,
                collapse_duration: start.elapsed(),
                isolated,
                dominated,
            });
        }

        if !opts.only_random {
            let (isolated, dominated) = count_isolated_edges(&zero_grades_edges);
            let start = std::time::Instant::now();
            let resulting_edges = remove_strongly_filtration_dominated_timed(
                &mut zero_grades_edges,
                EdgeOrder::ReverseLexicographic,
                Some(TIMEOUT_DURATION_RANDOM_DENSITIES),
            );
            let edges_after_collapse = resulting_edges.len();

            rows.push(RandomDensitiesRow {
                dataset,
                n_points,
                grade_structure: GradeStructure::Zero,
                order: Order::ReverseLexicographic,
                edges_before_collapse,
                edges_after_collapse,
                collapse_duration: start.elapsed(),
                isolated,
                dominated,
            });
        }

        if !opts.only_random {
            let (isolated, dominated) = count_isolated_edges(&edges);
            let start = std::time::Instant::now();
            let resulting_edges = remove_strongly_filtration_dominated_timed(
                &mut edges,
                EdgeOrder::ReverseLexicographic,
                Some(TIMEOUT_DURATION_RANDOM_DENSITIES),
            );
            let edges_after_collapse = resulting_edges.len();

            rows.push(RandomDensitiesRow {
                dataset,
                n_points,
                grade_structure: GradeStructure::Normal,
                order: Order::ReverseLexicographic,
                edges_before_collapse,
                edges_after_collapse,
                collapse_duration: start.elapsed(),
                isolated,
                dominated,
            });
        }
    }

    save_table(Table::new(rows), "compare_random_densities")?;

    Ok(())
}
