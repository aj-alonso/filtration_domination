# Filtration-domination algorithms

The present code implements algorithms to remove edges from a bifiltered graph,
while maintaining the topological properties of its clique complex. An
explanation of these terms, and a description of the algorithms, are in the
paper

"Filtration-Domination in Bifiltered Graphs".

The code also includes utilities to handle bifiltered graphs, compute its clique
complexes, and run mpfree. See the documentation below.

This is a Rust project. It has been tested with Rust 1.60.

## Usage

The API has two main functions: `remove_filtration_dominated` and
`remove_strongly_filtration_dominated`, both in the `removal` module, which
directly correspond to the described algorithms in the paper. They take as input
a list of (bifiltered) edges, encoded as an ``EdgeList``, and output a reduced
list of edges.

## Tests

Run the tests with
```shell
cargo test --release -- --show-output
```
It expects that the datasets are available in the `datasets` directory.
Use the provided script `download_datasets.sh` to download them.
It is required that the `mpfree` executable is available in the PATH.

## Example

There is an example in `examples/run.rs`. It can be run with
```shell
cargo run --release --example run -- senate
```
where `senate` can be any of the available datasets.

Passing the `-m` option to the example
computes a minimal presentation with mpfree; run it as follows:
```shell
cargo run --release --example run -- senate -m
```

You can also pass the `--strong` option to use the strong filtration-dominated
removal algorithm, instead of the non-strong one, which is the default.

## Documentation

We include documentation of the API in the doc folder. To see it, point your
browser to
[doc/filtration_domination/index.html](doc/filtration_domination/index.html).
