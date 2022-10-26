use clap::ArgEnum;
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

use crate::memory_usage::{get_maximum_memory_usage, Kilobytes, Resource};
use crate::table::{display_option, display_option_as};
use crate::{display, display_duration, save_table, CliDataset, Row, Table};
use filtration_domination::mpfree::CheckedMpfreeError;

// Degree of homology to do minimal presentations with.
const HOMOLOGY: usize = 1;

const BYTES_IN_GIGABYTE: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Args)]
pub struct MpfreeCli {
    #[clap(arg_enum)]
    dataset: CliDataset,

    #[clap(arg_enum)]
    modality: MpfreeComputationModality,

    /// The maximum memory, in gigabytes, to allow when building the filtration.
    #[clap(short, long)]
    maximum_memory_gigabytes: Option<u64>,
}

#[derive(Debug, Copy, Clone, ArgEnum)]
enum MpfreeComputationModality {
    OnlyMpfree,
    FiltrationDomination,
    StrongFiltrationDomination,
}

impl std::fmt::Display for MpfreeComputationModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpfreeComputationModality::OnlyMpfree => write!(f, "only-mpfree"),
            MpfreeComputationModality::FiltrationDomination => write!(f, "filtration-domination"),
            MpfreeComputationModality::StrongFiltrationDomination => {
                write!(f, "strong-filtration-domination")
            }
        }
    }
}

#[derive(Debug)]
struct MpfreeRow<E> {
    dataset: CliDataset,
    n_points: usize,
    before: usize,
    after: usize,
    modality: MpfreeComputationModality,
    mpfree_timers: Result<MinimalPresentationComputationTime, E>,
    removal_time: Duration,
    maximum_memory_kb: Option<Kilobytes>,
}

impl<E: StdError> Row for MpfreeRow<E> {
    fn headers() -> Vec<&'static str> {
        vec![
            "Dataset", "Points", "Before", "After", "Modality", "Collapse", "Build", "Write",
            "Mpfree", "Error", "Memory",
        ]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset)),
            Some(display(self.n_points)),
            Some(display(self.before)),
            Some(display(self.after)),
            Some(display(self.modality)),
            Some(display_duration(&self.removal_time)),
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
            Some(display_option(self.maximum_memory_kb.as_ref())),
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

fn memory_consumption_check(iteration: usize, max_memory_bytes: u64) -> Result<(), MemoryError> {
    if iteration % 1000 == 0 {
        let proc_info = procfs::process::Process::myself()?;
        if proc_info.stat()?.vsize > max_memory_bytes {
            return Err(MemoryError::MemoryLimit);
        }
    }
    Ok(())
}

pub fn compare_mpfree(opts: MpfreeCli) -> anyhow::Result<()> {
    let mut rows: Vec<MpfreeRow<CheckedMpfreeError<MemoryError>>> = Vec::new();

    let mut edges = datasets::get_dataset_density_edge_list(
        opts.dataset.to_internal_dataset(None),
        Threshold::KeepAll,
        None,
        true,
    )?;
    let n_initial_edges = edges.len();

    let start = std::time::Instant::now();
    let edges = match opts.modality {
        MpfreeComputationModality::OnlyMpfree => edges,
        MpfreeComputationModality::FiltrationDomination => {
            remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic)
        }
        MpfreeComputationModality::StrongFiltrationDomination => {
            remove_strongly_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic)
        }
    };
    let duration_edge_removal = start.elapsed();

    eprintln!("Computing the minimal presentation...");
    let maximum_memory_check = opts.maximum_memory_gigabytes.map(|gigabytes| {
        move |iteration| memory_consumption_check(iteration, gigabytes * BYTES_IN_GIGABYTE)
    });
    let mpfree = compute_minimal_presentation_with_check(
        &format!("comp_mpfree_{}_{}", opts.dataset, opts.modality),
        HOMOLOGY,
        &edges,
        maximum_memory_check,
    );

    // Get the memory consumed by this process: this includes both the run of the filtration-domination
    // algorithm and the construction of the filtration.
    let myself_memory = get_maximum_memory_usage(Resource::Myself);
    // Get the memory consumed by the children, that is, the call to mpfree as a subprocess.
    let children_memory = get_maximum_memory_usage(Resource::Children);
    // The maximum memory consumption would then be the maximum of "myself" and "children",
    // as if a process would have done the removal and the filtration construction, and another ran mpfree.
    let memory = myself_memory
        .zip(children_memory)
        .map(|(a_kb, b_kb)| std::cmp::max(a_kb, b_kb));

    rows.push(MpfreeRow {
        dataset: opts.dataset,
        n_points: edges.n_vertices,
        before: n_initial_edges,
        after: edges.len(),
        removal_time: duration_edge_removal,
        mpfree_timers: mpfree.map(|info| info.timers),
        modality: opts.modality,
        maximum_memory_kb: memory,
    });

    save_table(
        Table::new(rows),
        &format!("compare_mpfree_{}_{}", opts.dataset, opts.modality),
    )?;

    Ok(())
}
