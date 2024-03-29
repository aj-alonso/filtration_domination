use crate::single_collapse::run_single_parameter_edge_collapse;
use crate::utils::{delete_densities, forget_densities};
use crate::{display, display_duration, save_table, CliDataset, Row, Table, ALL_DATASETS};
use clap::Args;
use filtration_domination::datasets::Threshold;
use filtration_domination::edges::{write_edge_list, EdgeList, FilteredEdge};
use filtration_domination::removal::{
    remove_filtration_dominated, remove_strongly_filtration_dominated, EdgeOrder,
};
use filtration_domination::{datasets, OneCriticalGrade, Value};
use std::fmt::{Display, Formatter};
use std::time::Duration;

const TMP_DIRECTORY: &str = "tmp";

#[derive(Debug, Args)]
pub struct RemovalCli {
    #[clap(arg_enum)]
    datasets: Vec<CliDataset>,

    #[clap(short, arg_enum)]
    policies: Vec<RemovalPolicy>,

    #[clap(long)]
    /// On the single-parameter algorithms, whether to save the reduced list of edges to disk.
    save_single_parameter_edges: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, clap::ArgEnum)]
enum RemovalPolicy {
    StrongFiltrationDomination,
    FiltrationDomination,
    SingleParameter,

    StrongFiltrationDominationSingle,
    FiltrationDominationSingle,
}

impl Display for RemovalPolicy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RemovalPolicy::StrongFiltrationDomination => write!(f, "strong-filtration-domination"),
            RemovalPolicy::FiltrationDomination => write!(f, "filtration-domination"),
            RemovalPolicy::SingleParameter => write!(f, "single-parameter"),
            RemovalPolicy::StrongFiltrationDominationSingle => {
                write!(f, "strong-filtration-domination-single")
            }
            RemovalPolicy::FiltrationDominationSingle => write!(f, "filtration-domination-single"),
        }
    }
}

const ALL_REMOVAL_POLICIES: [RemovalPolicy; 3] = [
    RemovalPolicy::StrongFiltrationDomination,
    RemovalPolicy::FiltrationDomination,
    RemovalPolicy::SingleParameter,
    // By default we do not do the single parameter variants of (strong) filtration domination.
];

#[derive(Debug)]
struct RemovalRow {
    dataset: CliDataset,
    n_points: usize,
    policy: RemovalPolicy,
    edges_before_collapse: usize,
    edges_after_collapse: usize,
    collapse_duration: Duration,
}

impl Row for RemovalRow {
    fn headers() -> Vec<&'static str> {
        vec!["Dataset", "Points", "Policy", "Before", "After", "Time"]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset)),
            Some(display(self.n_points)),
            Some(display(self.policy)),
            Some(display(self.edges_before_collapse)),
            Some(display(self.edges_after_collapse)),
            Some(display_duration(&self.collapse_duration)),
        ]
    }
}

fn save_single_parameter_edges<VF: Value>(
    edges: &EdgeList<FilteredEdge<OneCriticalGrade<VF, 2>>>,
    dataset: CliDataset,
    policy: RemovalPolicy,
) -> anyhow::Result<()> {
    let edges_no_density = delete_densities(edges);

    let directory = std::path::Path::new(TMP_DIRECTORY);
    std::fs::create_dir_all(&directory)?;
    let out_edges_path =
        directory.join(format!("single_parameter_edges_{}_{}.txt", dataset, policy));

    let mut out_edges_file = std::fs::File::create(out_edges_path)?;
    write_edge_list(&edges_no_density, &mut out_edges_file, true)?;

    Ok(())
}

pub fn compare_removals(opts: RemovalCli) -> anyhow::Result<()> {
    let datasets = if opts.datasets.is_empty() {
        Vec::from(ALL_DATASETS)
    } else {
        opts.datasets
    };

    let policies = if opts.policies.is_empty() {
        Vec::from(ALL_REMOVAL_POLICIES)
    } else {
        opts.policies
    };

    let mut rows: Vec<RemovalRow> = Vec::new();
    for dataset in datasets {
        println!("Processing dataset {}", dataset);
        let mut edges = datasets::get_dataset_density_edge_list(
            dataset.to_internal_dataset(None),
            Threshold::KeepAll,
            None,
            true,
        )?;
        let single_parameter_edges = delete_densities(&edges);

        let mut zero_density_edges = edges.clone();
        forget_densities(&mut zero_density_edges);

        let edges_before_collapse = edges.len();
        let n_points = edges.n_vertices;

        for &policy in policies.iter() {
            let (edges_after_collapse, duration) = match policy {
                RemovalPolicy::StrongFiltrationDomination => {
                    let start = std::time::Instant::now();
                    let resulting_edges = remove_strongly_filtration_dominated(
                        &mut edges,
                        EdgeOrder::ReverseLexicographic,
                    );
                    (resulting_edges.len(), start.elapsed())
                }
                RemovalPolicy::FiltrationDomination => {
                    let start = std::time::Instant::now();
                    let resulting_edges =
                        remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
                    (resulting_edges.len(), start.elapsed())
                }
                RemovalPolicy::SingleParameter => {
                    let result = run_single_parameter_edge_collapse(&single_parameter_edges)?;

                    if opts.save_single_parameter_edges {
                        // HACK: the single parameter utility outputs the resulting edges to edges_out.txt.
                        let directory = std::path::Path::new(TMP_DIRECTORY);
                        std::fs::create_dir_all(&directory)?;
                        let result_edges_out_file = directory
                            .join(format!("single_parameter_edges_{}_{}.txt", dataset, policy));
                        std::fs::copy("edges_out.txt", result_edges_out_file)?;
                    }

                    result
                }
                RemovalPolicy::StrongFiltrationDominationSingle => {
                    let start = std::time::Instant::now();
                    let resulting_edges = remove_strongly_filtration_dominated(
                        &mut zero_density_edges,
                        EdgeOrder::ReverseLexicographic,
                    );

                    if opts.save_single_parameter_edges {
                        save_single_parameter_edges(&resulting_edges, dataset, policy)?;
                    }

                    (resulting_edges.len(), start.elapsed())
                }
                RemovalPolicy::FiltrationDominationSingle => {
                    let start = std::time::Instant::now();
                    let resulting_edges = remove_filtration_dominated(
                        &mut zero_density_edges,
                        EdgeOrder::ReverseLexicographic,
                    );

                    if opts.save_single_parameter_edges {
                        save_single_parameter_edges(&resulting_edges, dataset, policy)?;
                    }

                    (resulting_edges.len(), start.elapsed())
                }
            };

            let row = RemovalRow {
                dataset,
                n_points,
                policy,
                edges_before_collapse,
                edges_after_collapse,
                collapse_duration: duration,
            };

            println!("Ran policy {policy} in {duration:?}.");

            rows.push(row);
        }
    }

    save_table(Table::new(rows), "compare_removal")?;

    Ok(())
}
