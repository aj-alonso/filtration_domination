use crate::chain_complex::ToFreeImplicitRepresentation;
use crate::Value;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParsedMpfreeOutput {
    pub parameters: usize,
    pub sizes: [usize; 3],
}

impl ParsedMpfreeOutput {
    pub fn betti(&self) -> [usize; 3] {
        [self.sizes[1], self.sizes[0], self.sizes[0] - self.sizes[1]]
    }
}

pub fn write_bifiltration<
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
