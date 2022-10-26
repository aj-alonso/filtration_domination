use clap::Args;
use clap::ArgEnum;
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
use crate::{display, display_duration, save_table, CliDataset, Row, Table};
use filtration_domination::mpfree::CheckedMpfreeError;
use crate::memory_usage::{get_maximum_memory_usage_all_resources, Kilobytes};

// Degree of homology to do minimal presentations with.
const HOMOLOGY: usize = 1;

const GIGABYTE: u64 = 1_000_000_000;
const MAXIMUM_MEMORY_BYTES: u64 = 60 * GIGABYTE;

#[derive(Debug, Args)]
pub struct MpfreeCli {
    #[clap(arg_enum)]
    dataset: CliDataset,

    #[clap(arg_enum)]
    modality: MpfreeComputationModality,
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
            MpfreeComputationModality::OnlyMpfree => write!(f, "Only mpfree"),
            MpfreeComputationModality::FiltrationDomination => write!(f, "Collapse"),
            MpfreeComputationModality::StrongFiltrationDomination => write!(f, "Strong collapse"),
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
    maximum_memory_kb: Option<Kilobytes>
}

impl<E: StdError> Row for MpfreeRow<E> {
    fn headers() -> Vec<&'static str> {
        vec![
            "Dataset", "Points", "Before", "After", "Modality", "Collapse", "Build", "Write",
            "Mpfree", "Error", "Memory"
        ]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset.to_static_str())),
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
            Some(display_option(self.maximum_memory_kb.as_ref()))
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

fn memory_consumption_check(iteration: usize) -> Result<(), MemoryError> {
    if iteration % 1000 == 0 {
        let proc_info = procfs::process::Process::myself()?;
        if proc_info.stat()?.vsize > MAXIMUM_MEMORY_BYTES {
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
        MpfreeComputationModality::FiltrationDomination => remove_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic),
        MpfreeComputationModality::StrongFiltrationDomination => remove_strongly_filtration_dominated(&mut edges, EdgeOrder::ReverseLexicographic)
    };
    let duration_edge_removal = start.elapsed();

    eprintln!("Computing the minimal presentation with mpfree...");
    let mpfree = compute_minimal_presentation_with_check(
        &format!("comp_mpfree_{}_{}", opts.dataset, opts.modality),
        HOMOLOGY,
        &edges,
        Some(memory_consumption_check),
    );


    rows.push(MpfreeRow {
        dataset: opts.dataset,
        n_points: edges.n_vertices,
        before: n_initial_edges,
        after: edges.len(),
        removal_time: duration_edge_removal,
        mpfree_timers: mpfree.map(|info| info.timers),
        modality: opts.modality,
        maximum_memory_kb: get_maximum_memory_usage_all_resources()
    });

    save_table(Table::new(rows), "compare_mpfree")?;

    Ok(())
}
