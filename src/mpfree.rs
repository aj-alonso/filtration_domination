use crate::chain_complex::ToFreeImplicitRepresentation;
use crate::edges::{EdgeList, FilteredEdge};
use crate::filtration::{build_flag_filtration, Filtration};
use crate::simplicial_complex::MapSimplicialComplex;
use crate::{CriticalGrade, Value};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;
use std::{fs, io};
use thiserror::Error;

const TMP_DIRECTORY: &str = "tmp";

#[derive(Debug, Clone)]
pub struct MinimalPresentationResult {
    pub timers: MinimalPresentationComputationTime,
    pub output: ParsedMpfreeOutput,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MinimalPresentationComputationTime {
    build_filtration: Duration,
    write_bifiltration: Duration,
    mpfree: Duration,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParsedMpfreeOutput {
    pub parameters: usize,
    pub sizes: [usize; 3],
}

pub fn compute_minimal_presentation<VF: Value, G: CriticalGrade>(
    name: &str,
    homology: usize,
    edge_list: &EdgeList<FilteredEdge<G>>,
) -> Result<MinimalPresentationResult, MpfreeError>
where
    Filtration<G, MapSimplicialComplex>: ToFreeImplicitRepresentation<VF, 2>,
{
    let mut timers = MinimalPresentationComputationTime::default();

    // Build filtration.
    let start_filtration = std::time::Instant::now();
    let filtration: Filtration<_, MapSimplicialComplex> = build_flag_filtration(
        edge_list.n_vertices,
        homology + 1,
        edge_list.edge_iter().cloned(),
    );
    timers.build_filtration = start_filtration.elapsed();

    // Save filtration to disk.
    let start_io = std::time::Instant::now();
    let directory = Path::new(TMP_DIRECTORY);
    fs::create_dir_all(&directory)?;
    let filepath_mpfree_input = directory.join(format!("{}_scc2020", name));
    let filepath_out = filepath_mpfree_input.with_extension("out");
    write_bifiltration(&filepath_mpfree_input, homology, &filtration).unwrap();
    timers.write_bifiltration = start_io.elapsed();

    // Compute minimal presentation.
    let start_mpfree = std::time::Instant::now();
    let output = run_mpfree(filepath_mpfree_input, filepath_out)?;
    timers.mpfree = start_mpfree.elapsed();

    Ok(MinimalPresentationResult { timers, output })
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

fn run_mpfree<P: AsRef<Path>>(
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
    let mut betti_raw = buffer.trim().split_whitespace();
    let mut betti: [usize; 3] = [0, 0, 0];
    for b in betti.iter_mut() {
        *b = betti_raw
            .next()
            .ok_or(MpfreeError::ParsingBettiNumbers)?
            .parse()?;
    }

    Ok(ParsedMpfreeOutput {
        parameters,
        sizes: betti,
    })
}
