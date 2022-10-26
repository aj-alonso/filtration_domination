
#[derive(Error, Debug)]
enum MinimalPresentationError {
    #[error("Building flag filtration error: {0}")]
    Flag(#[from] BuildFlagError),

    #[error("Running mpfree: {0}")]
    Mpfree(#[from] MpfreeError),
}

/// Compute a minimal presentation of the homology at the given dimension of the clique bifiltration
/// of the given bifiltered edge list.
///
/// The `name` parameter is used to name and identify temporary files.
fn compute_minimal_presentation<VF: Value, G: CriticalGrade>(
    name: &str,
    homology: usize,
    edge_list: &EdgeList<FilteredEdge<G>>,
    maximum_memory_limit_bytes: Option<u64>
) -> Result<MinimalPresentationResult, MinimalPresentationError>
    where
        Filtration<G, MapSimplicialComplex>: ToFreeImplicitRepresentation<VF, 2>,
{
    let mut timers = MinimalPresentationComputationTime::default();

    // Build filtration.
    let start_filtration = std::time::Instant::now();
    let filtration: Filtration<_, MapSimplicialComplex> = build_flag_filtration_maximum_memory(
        edge_list.n_vertices,
        homology + 1,
        edge_list.edge_iter().cloned(),
        maximum_memory_limit_bytes,
    )?;
    timers.build_filtration = start_filtration.elapsed();

    // Save filtration to disk.
    let start_io = std::time::Instant::now();
    let directory = Path::new(TMP_DIRECTORY);
    let filepath_mpfree_input = directory.join(format!("{}_scc2020", name));
    let filepath_out = filepath_mpfree_input.with_extension("out");
    write_bifiltration(&filepath_mpfree_input, homology, &filtration).unwrap();
    timers.write_bifiltration = start_io.elapsed();

    // Compute minimal presentation.
    let start_mpfree = std::time::Instant::now();
    let output = interfaces::mpfree::run_mpfree(filepath_mpfree_input, filepath_out)?;
    timers.mpfree = start_mpfree.elapsed();

    Ok(MinimalPresentationResult { timers, output })
}
