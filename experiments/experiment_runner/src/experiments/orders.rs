use clap::Args;
use std::fmt::Formatter;
use std::time::Duration;

use crate::CliDataset;
use crate::{display, display_duration, save_table, Algorithm, Row, Table, ALL_DATASETS};

use filtration_domination::datasets::Threshold;
use filtration_domination::edges::{EdgeList, FilteredEdge};
use filtration_domination::removal::{remove_filtration_dominated_timed, EdgeOrder};
use filtration_domination::{datasets, OneCriticalGrade, Value};

#[derive(Debug, Args)]
pub struct OrderCli {
    #[clap(arg_enum)]
    datasets: Vec<CliDataset>,

    #[clap(short, arg_enum)]
    orders: Option<Vec<Order>>,

    /// Timeout, in seconds, when removing edges.
    #[clap(short, default_value_t = 60 * 60 * 2)]
    timeout: u64,
}

#[derive(Copy, Clone, Debug, clap::ArgEnum)]
pub enum Order {
    ReverseLexicographic,
    ReverseColexicographic,
    ForwardLexicographic,
    ForwardColexicographic,
    Random,
}

impl Order {
    fn apply<VF: Value, const N: usize>(
        &self,
        edges: &mut EdgeList<FilteredEdge<OneCriticalGrade<VF, N>>>,
    ) {
        match self {
            Order::ReverseLexicographic => edges.sort_reverse_lexicographically(),
            Order::ReverseColexicographic => edges.sort_reverse_colexicographically(),
            Order::ForwardLexicographic => edges.sort_lexicographically(),
            Order::ForwardColexicographic => edges.sort_colexicographically(),
            Order::Random => edges.shuffle(),
        }
    }

    fn to_static_str(self) -> &'static str {
        match self {
            Order::ReverseLexicographic => "RevLex",
            Order::ReverseColexicographic => "RevColex",
            Order::ForwardLexicographic => "Lex",
            Order::ForwardColexicographic => "Colex",
            Order::Random => "Rand",
        }
    }
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_static_str())
    }
}

const ALL_ORDERS: [Order; 5] = [
    Order::ReverseLexicographic,
    Order::ReverseColexicographic,
    Order::ForwardLexicographic,
    Order::ForwardColexicographic,
    Order::Random,
];

#[derive(Debug)]
struct OrderRow {
    dataset: CliDataset,
    n_points: usize,
    order: Order,
    modality: Algorithm,
    edges_before_collapse: usize,
    edges_after_collapse: usize,
    collapse_duration: Duration,
}

impl Row for OrderRow {
    fn headers() -> Vec<&'static str> {
        vec![
            "Dataset", "Points", "Modality", "Order", "Before", "After", "Time",
        ]
    }

    fn fields(&self) -> Vec<Option<String>> {
        vec![
            Some(display(self.dataset)),
            Some(display(self.n_points)),
            Some(display(self.modality)),
            Some(display(self.order)),
            Some(display(self.edges_before_collapse)),
            Some(display(self.edges_after_collapse)),
            Some(display_duration(&self.collapse_duration)),
        ]
    }
}

pub fn compare_orders(opts: OrderCli) -> anyhow::Result<()> {
    let datasets = if opts.datasets.is_empty() {
        Vec::from(ALL_DATASETS)
    } else {
        opts.datasets
    };

    let orders = if let Some(orders) = opts.orders {
        orders
    } else {
        Vec::from(ALL_ORDERS)
    };

    let timeout = Duration::from_secs(opts.timeout);
    println!("Using {timeout:?} as timeout.");

    let mut rows: Vec<OrderRow> = Vec::new();
    for dataset in datasets {
        println!("Processing dataset {}", dataset);
        let mut edges = datasets::get_dataset_density_edge_list(
            dataset.to_internal_dataset(None),
            Threshold::KeepAll,
            None,
            true,
        )?;
        let edges_before_collapse = edges.len();
        for &order in &orders {
            order.apply(&mut edges);

            let start = std::time::Instant::now();
            let collapsed_edges =
                remove_filtration_dominated_timed(&mut edges, EdgeOrder::Maintain, Some(timeout));
            let duration = start.elapsed();

            rows.push(OrderRow {
                dataset,
                n_points: edges.n_vertices,
                modality: Algorithm::FiltrationDomination,
                order,
                edges_before_collapse,
                edges_after_collapse: collapsed_edges.len(),
                collapse_duration: duration,
            });
        }
    }

    save_table(Table::new(rows), "compare_orders")?;

    Ok(())
}
