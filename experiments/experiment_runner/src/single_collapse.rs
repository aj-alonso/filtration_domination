use anyhow::Result;
use filtration_domination::edges::{write_edge_list, EdgeList, FilteredEdge};
use filtration_domination::{OneCriticalGrade, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::time::Duration;

/// Runs a single-parameter edge collapser on the given edge list, and returns the number of
/// resulting edges.
/// Not thread-safe, because it writes to a fixed file.
pub fn run_single_parameter_edge_collapse<T: Value + std::fmt::Display>(
    edges: &EdgeList<FilteredEdge<OneCriticalGrade<T, 1>>>,
) -> Result<(usize, Duration)> {
    let edges_out_file = "edges.txt";
    {
        let mut out_edges_file = fs::File::create(edges_out_file)?;
        write_edge_list(edges, &mut out_edges_file)?;
        out_edges_file.sync_data()?;
    }

    let command_name = "single_parameter";

    let mut collapser_command = Command::new(command_name);
    collapser_command.args(vec![edges_out_file]);
    let collapser_output = collapser_command.output()?;

    let mut stdout = BufReader::new(&collapser_output.stdout[..]);

    let mut buffer = String::new();
    stdout.read_line(&mut buffer)?;
    let resulting_edges: usize = buffer.trim().parse()?;
    buffer.clear();
    stdout.read_line(&mut buffer)?;
    let seconds: f64 = buffer.trim().parse()?;

    Ok((resulting_edges, Duration::from_secs_f64(seconds)))
}
