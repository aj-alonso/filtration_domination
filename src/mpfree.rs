//! Interface with mpfree that allows to compute minimal presentations.
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;
use std::{fs, io};
use thiserror::Error;

use crate::chain_complex::ToFreeImplicitRepresentation;
use crate::edges::{EdgeList, FilteredEdge};
use crate::filtration::{build_flag_filtration_with_check, Filtration};
use crate::simplicial_complex::MapSimplicialComplex;
use crate::{CriticalGrade, Value};

const TMP_DIRECTORY: &str = "tmp";

/// The time taken to run mpfree, and the parsed output.
#[derive(Debug, Clone)]
pub struct MinimalPresentationComputationSummary {
    pub timers: MinimalPresentationComputationTime,
    pub output: ParsedMpfreeOutput,
}

/// Timers related to minimal presentation computation.
#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalPresentationComputationTime {
    pub build_filtration: Duration,
    pub write_bifiltration: Duration,
    pub mpfree: Duration,
}

/// Summaries of the minimal presentation computed by mpfree.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParsedMpfreeOutput {
    pub parameters: usize,
    pub sizes: [usize; 3],
}

/// Compute a minimal presentation of the homology at the given dimension of the clique bifiltration
/// of the given bifiltered edge list.
///
/// The `name` parameter is used to name and identify temporary files.
pub fn compute_minimal_presentation<VF: Value, G: CriticalGrade>(
    name: &str,
    homology: usize,
    edge_list: &EdgeList<FilteredEdge<G>>,
) -> Result<MinimalPresentationComputationSummary, MpfreeError>
where
    Filtration<G, MapSimplicialComplex>: ToFreeImplicitRepresentation<VF, 2>,
{
    let result = compute_minimal_presentation_with_check::<_, _, std::io::Error, fn(usize) -> Result<(), io::Error>>(
        name, homology, edge_list, None,
    );
    match result {
        Ok(summary) => Ok(summary),
        Err(err) => match err {
            CheckedMpfreeError::CheckFailed(_) => {
                panic!("Programming error: we didn't specify a check.")
            }
            CheckedMpfreeError::Mpfree(err) => Err(err),
        },
    }
}

#[derive(Error, Debug)]
pub enum CheckedMpfreeError<E> {
    #[error(transparent)]
    CheckFailed(E),

    #[error(transparent)]
    Mpfree(#[from] MpfreeError),
}

/// Compute a minimal presentation of the homology at the given dimension of the clique bifiltration
/// of the given bifiltered edge list.
///
/// The `name` parameter is used to name and identify temporary files.
pub fn compute_minimal_presentation_with_check<VF: Value, G: CriticalGrade, E: std::error::Error, F: Fn(usize) -> Result<(), E>>(
    name: &str,
    homology: usize,
    edge_list: &EdgeList<FilteredEdge<G>>,
    memory_check_fn: Option<F>,
) -> Result<MinimalPresentationComputationSummary, CheckedMpfreeError<E>>
where
    Filtration<G, MapSimplicialComplex>: ToFreeImplicitRepresentation<VF, 2>,
{
    let mut timers = MinimalPresentationComputationTime::default();

    // Build filtration.
    let start_filtration = std::time::Instant::now();
    let filtration: Filtration<_, MapSimplicialComplex> = build_flag_filtration_with_check(
        edge_list.n_vertices,
        homology + 1,
        edge_list.edge_iter().cloned(),
        memory_check_fn,
    )
    .map_err(CheckedMpfreeError::CheckFailed)?;
    timers.build_filtration = start_filtration.elapsed();

    // Save filtration to disk.
    let start_io = std::time::Instant::now();
    let directory = Path::new(TMP_DIRECTORY);
    fs::create_dir_all(&directory).map_err(MpfreeError::Io)?;
    let filepath_mpfree_input = directory.join(format!("{}_scc2020", name));
    let filepath_out = filepath_mpfree_input.with_extension("out");
    write_bifiltration(&filepath_mpfree_input, homology, &filtration).unwrap();
    timers.write_bifiltration = start_io.elapsed();

    // Compute minimal presentation.
    let start_mpfree = std::time::Instant::now();
    let output = run_mpfree(filepath_mpfree_input, filepath_out)?;
    timers.mpfree = start_mpfree.elapsed();

    Ok(MinimalPresentationComputationSummary { timers, output })
}

fn write_bifiltration<
    VF: Value,
    F: ToFreeImplicitRepresentation<VF, N>,
    P: AsRef<Path>,
    const N: usize,
>(
    filepath: P,
    homology: usize,
    f: &F,
) -> io::Result<()> {
    let file = std::fs::File::create(&filepath)?;
    let mut writer = BufWriter::new(&file);
    f.write_scc2020(homology, &mut writer)?;
    Ok(())
}

/// A error that happened when executing mpfree.
#[derive(Error, Debug)]
pub enum MpfreeError {
    #[error("Mpfree ended with a non-okay exit code: {0}")]
    ExitStatus(ExitStatus),

    #[error("Error parsing output header")]
    BadOutputHeader,

    #[error("Error parsing Betti numbers")]
    ParsingBettiNumbers,

    #[error("IO error running mpfree: {0}")]
    Io(#[from] io::Error),

    #[error("Error parsing number: {0}")]
    WrongNumberFormat(#[from] std::num::ParseIntError),
}

pub fn run_mpfree<P: AsRef<Path>>(
    filepath_in: P,
    filepath_out: P,
) -> Result<ParsedMpfreeOutput, MpfreeError> {
    let mut child = Command::new("mpfree")
        .args([
            filepath_in.as_ref().as_os_str(),
            filepath_out.as_ref().as_os_str(),
        ])
        .stdout(Stdio::null())
        .spawn()?;
    let exit_code = child.wait()?;
    if !exit_code.success() {
        return Err(MpfreeError::ExitStatus(exit_code));
    }

    let output_file = File::open(filepath_out.as_ref())?;
    let mut child_stdout = BufReader::new(output_file);
    let mut buffer = String::new();
    child_stdout.read_line(&mut buffer)?;
    if buffer != "scc2020\n" {
        return Err(MpfreeError::BadOutputHeader);
    }
    buffer.clear();
    child_stdout.read_line(&mut buffer)?;
    let parameters: usize = buffer.trim().parse()?;
    buffer.clear();
    child_stdout.read_line(&mut buffer)?;
    let mut sizes_raw = buffer.split_whitespace();
    let mut sizes: [usize; 3] = [0, 0, 0];
    for s in sizes.iter_mut() {
        *s = sizes_raw
            .next()
            .ok_or(MpfreeError::ParsingBettiNumbers)?
            .parse()?;
    }

    Ok(ParsedMpfreeOutput { parameters, sizes })
}
