use clap::Parser;
use filtration_domination::edges::{read_edge_list, write_edge_list, EdgeList, FilteredEdge};
use filtration_domination::removal::{
    remove_filtration_dominated, remove_strongly_filtration_dominated, EdgeOrder,
};
use filtration_domination::OneCriticalGrade;
use ordered_float::OrderedFloat;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Parser)]
struct Cli {
    input: String,

    output: String,

    #[clap(short, long)]
    full: bool,
}

fn main() -> anyhow::Result<()> {
    let opts: Cli = Cli::parse();
    let edge_list_file = File::open(&opts.input)?;
    let reader = BufReader::new(edge_list_file);
    let mut edge_list: EdgeList<FilteredEdge<OneCriticalGrade<OrderedFloat<f64>, 2>>> =
        read_edge_list(reader)?;

    let remaining_edges = if opts.full {
        remove_filtration_dominated(&mut edge_list, EdgeOrder::ReverseLexicographic)
    } else {
        remove_strongly_filtration_dominated(&mut edge_list, EdgeOrder::ReverseLexicographic)
    };

    let out_file = File::create(&opts.output)?;
    let mut writer = BufWriter::new(out_file);
    write_edge_list(&remaining_edges, &mut writer, false)?;

    Ok(())
}
