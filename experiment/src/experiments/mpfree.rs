use clap::Args;
use std::error::Error as StdError;
use std::time::Duration;
use thiserror::Error;

use filtration_domination::datasets;
use filtration_domination::datasets::Threshold;
use filtration_domination::mpfree::{
    compute_minimal_presentation_with_check, MinimalPresentationComputationTime,
};
use filtration_domination::removal::{
    remove_filtration_dominated, remove_strongly_filtration_dominated, EdgeOrder,
};

use crate::table::{display_option, display_option_as};
use crate::{display, display_duration, save_table, CliDataset, Row, Table, ALL_DATASETS};
use filtration_domination::mpfree::CheckedMpfreeError;

// Degree of homology to do minimal presentations with.
const HOMOLOGY: usize = 1;

const GIGABYTE: u64 = 1_000_000_000;
const MAXIMUM_MEMORY_BYTES: u64 = 60 * GIGABYTE;

#[derive(Debug, Args)]
pub struct MpfreeCli {
    #[clap(arg_enum)]
    datasets: Vec<CliDataset>,

    /// Number of points to use on dynamic datasets.
    #[clap(short)]
    n_points: Option<usize>,

    /// Only run mpfree after collapsing.
    #[clap(short, long)]
    only_mpfree_with_collapses: bool,
}

#[derive(Debug, Copy, Clone)]
enum MpfreeComputationModality {
    OnlyMpfree,
    WithCollapse,
    WithStrongCollapse,
}

impl std::fmt::Display for MpfreeComputationModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpfreeComputationModality::OnlyMpfree => write!(f, "Only mpfree"),
            MpfreeComputationModality::WithCollapse => write!(f, "Collapse"),
            MpfreeComputationModality::WithStrongCollapse => write!(f, "Strong collapse"),
        }
    }
}

#[derive(Debug)]
struct MpfreeRow<E> {
    dataset: CliDataset,
    n_points: usize,
    before: usize,
    after: Option<usize>,
    modality: MpfreeComputationModality,
    mpfree_timers: Result<MinimalPresentationComputationTime, E>,
    collapse: Option<Duration>,
}

impl<E: StdError> Row for MpfreeRow<E> {
    fn headers() -> Vec<&'static str> {
        vec![
            "Dataset", "Points", "Before", "After", "Modality", "Collapse", "Build", "Write",
            "Mpfree", "Error",
        ]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset.to_static_str())),
            Some(display(self.n_points)),
            Some(display(self.before)),
            Some(display_option(self.after)),
            Some(display(self.modality)),
            Some(display_option_as(self.collapse.as_ref(), display_duration)),
            Some(display_option_as(
                self.mpfree_timers
                    .as_ref()
                    .ok()
                    .map(|t| t.build_filtration)
                    .as_ref(),
                display_duration,
            )),
            Some(display_option_as(
                self.mpfree_timers
                    .as_ref()
                    .ok()
                    .map(|t| t.write_bifiltration)
                    .as_ref(),
                display_duration,
            )),
            Some(display_option_as(
                self.mpfree_timers.as_ref().ok().map(|t| t.mpfree).as_ref(),
                display_duration,
            )),
            Some(display_option(
                self.mpfree_timers.as_ref().err().map(|e| e.to_string()),
            )),
        ]
    }
}

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Procfs error: {0}")]
    Procfs(#[from] procfs::ProcError),

    #[error("Memory limit exceeded")]
    MemoryLimit,
}

fn maximum_memory_check(iteration: usize) -> Result<(), MemoryError> {
    if iteration % 1000 == 0 {
        let proc_info = procfs::process::Process::myself()?;
        if proc_info.stat()?.vsize > MAXIMUM_MEMORY_BYTES {
            return Err(MemoryError::MemoryLimit);
        }
    }
    Ok(())
}

pub fn compare_mpfree(opts: MpfreeCli) -> anyhow::Result<()> {
    let datasets = if opts.datasets.is_empty() {
        Vec::from(ALL_DATASETS)
    } else {
        opts.datasets.clone()
    };

    let mut rows: Vec<MpfreeRow<CheckedMpfreeError<MemoryError>>> = Vec::new();
    for dataset in datasets {
        eprintln!("Processing dataset {dataset}");

        let mut edges = datasets::get_dataset_density_edge_list(
            dataset.to_internal_dataset(None),
            Threshold::KeepAll,
            None,
            true,
        )?;
        let n_initial_edges = edges.len();

        let start = std::time::Instant::now();
        let strong_collapsed_edges =
            remove_strongly_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
        let duration_strong_collapse = start.elapsed();

        eprintln!("Calculating the minimal presentation for strong collapses...");
        let mpfree_strong_collapse = compute_minimal_presentation_with_check(
            &format!("comp_mpfree_{}_strong_collapse", dataset),
            HOMOLOGY,
            &strong_collapsed_edges,
            Some(maximum_memory_check),
        );

        let start = std::time::Instant::now();
        let collapsed_edges =
            remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic);
        let duration_collapse = start.elapsed();

        eprintln!("Calculating the minimal presentation for collapses...");
        let mpfree_collapse = compute_minimal_presentation_with_check(
            &format!("comp_mpfree_{}__collapse", dataset),
            HOMOLOGY,
            &collapsed_edges,
            Some(maximum_memory_check),
        );

        if !opts.only_mpfree_with_collapses {
            eprintln!("Calculating the minimal presentation for no collapses...");
            let mpfree_no_collapse = compute_minimal_presentation_with_check(
                &format!("comp_mpfree_{}", dataset),
                HOMOLOGY,
                &edges,
                Some(maximum_memory_check),
            );

            if let Ok(mpfree_collapse) = mpfree_collapse.as_ref() {
                if let Ok(mpfree_no_collapse) = mpfree_no_collapse.as_ref() {
                    assert_eq!(mpfree_collapse.output, mpfree_no_collapse.output);
                }
                if let Ok(mpfree_strong_collapse) = mpfree_strong_collapse.as_ref() {
                    assert_eq!(mpfree_strong_collapse.output, mpfree_collapse.output);
                }
            }

            rows.push(MpfreeRow {
                dataset,
                n_points: edges.n_vertices,
                before: n_initial_edges,
                after: None,
                collapse: None,
                mpfree_timers: mpfree_no_collapse.map(|info| info.timers),
                modality: MpfreeComputationModality::OnlyMpfree,
            });
        }

        rows.push(MpfreeRow {
            dataset,
            n_points: edges.n_vertices,
            before: n_initial_edges,
            after: Some(strong_collapsed_edges.len()),
            collapse: Some(duration_strong_collapse),
            mpfree_timers: mpfree_strong_collapse.map(|info| info.timers),
            modality: MpfreeComputationModality::WithStrongCollapse,
        });
        rows.push(MpfreeRow {
            dataset,
            n_points: edges.n_vertices,
            before: n_initial_edges,
            after: Some(collapsed_edges.len()),
            collapse: Some(duration_collapse),
            mpfree_timers: mpfree_collapse.map(|info| info.timers),
            modality: MpfreeComputationModality::WithCollapse,
        });
    }

    save_table(Table::new(rows), "compare_mpfree")?;

    Ok(())
}
