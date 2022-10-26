# Filtration-domination algorithms

## Summary

The present code implements algorithms to remove edges from a bifiltered graph,
while maintaining the topological properties of its clique complex. An
explanation of these terms, and a description of the algorithms, are in the
paper

"Filtration-Domination in Bifiltered Graphs"

The code also includes utilities to handle bifiltered graphs, compute its clique
complexes, and run mpfree. See the documentation below.

This is a Rust project. It has been tested with Rust 1.62. Below, we use
`cargo`, which is the Rust package manager.

## Experiments

This project includes all code required to reproduce the experiments included in
the associated paper mentioned above.

Instructions on how to reproduce the experiments can be found in the
`experiments` folder: we refer to the `experiments/README` file. For maximum
reproducibility, we have included a Docker image that setups the required
environment, see the [Docker section](#Docker) to see how to build the image.

## Usage of the library

The API has two main functions: `remove_filtration_dominated` and
`remove_strongly_filtration_dominated`, both in the `removal` module, which
directly correspond to the described algorithms in the paper. They take as input
a list of (bifiltered) edges, encoded as an ``EdgeList``, and output a reduced
list of edges.

## Tests

The test suite has been tested in GNU/Linux.
It requires that the [mpfree](https://bitbucket.org/mkerber/mpfree/src/master/) executable is found somewhere along the PATH.
The test suite also expects that the datasets are available in the `datasets` directory.
Use the provided script `download_datasets.sh` (by executing from the root directory) to download them.

You can run the tests with
```shell
cargo test --release
```
Note that the command above will run tests in parallel, which might increase memory consumption.
To use less memory, you can run the tests sequentially:
```shell
cargo test --release -- --test-threads 1
```

### Docker

The following instructions explain how to use Docker to run the tests. Docker is
a lightweight virtualization and software packaging utility that allows to setup
the required environment easily. In this way, you don't need to manually install
Rust or setup the environment (like compiling `mpfree`).

First, build the Docker image associated to the Dockerfile at the root directory of the project:
``` shell
docker build -t filtration-domination/runner .
```
The resulting Docker image will have a working Rust toolchain and the required utilities, like `mpfree`, installed.
Once built, download the data datasets by running the `download_datasets.sh` script.
Now, you can run the tests with the help of a Docker container:

``` shell
docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/opt/filt filtration-domination/runner cargo test --release -- --test-threads 1
```

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

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.

Opening a pull requests is assumed to signal agreement with these licensing
terms.
