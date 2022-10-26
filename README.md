# Filtration-domination algorithms
Copyright 2022 TU Graz

## Summary

The present code implements algorithms to remove edges from a bifiltered graph,
while maintaining the topological properties of its clique complex. An
explanation of these terms, and a description of the algorithms, are in the
paper

"Filtration-Domination in Bifiltered Graphs" by Ángel Javier Alonso, Michael
Kerber, and Siddharth Pritam.

The code also includes utilities to handle bifiltered graphs, compute its clique
complexes, and run mpfree. See the documentation below.

This is a Rust project. It has been tested with Rust 1.62. Below, we use
`cargo`, which is the Rust package manager.

## Experiments

This project includes all code required to reproduce the experiments included in
the associated paper mentioned above.

Instructions on how to reproduce the experiments can be found in the
`experiments` folder: we refer to the
[experiments/README.md](./experiments/README.md) file. For maximum
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

### Install Rust

To compile `filtration-domination` you need to have Rust installed. The easiest
way to install Rust is through [rustup](https://rustup.rs/). This also
guarantees that you have a recent enough version.

To install Rust via rustup execute in any directory:

``` shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
and follow the instructions that appear.

### Install mpfree

The test suite requires to have [mpfree](https://bitbucket.org/mkerber/mpfree/src/master/) installed. To do so you can do the following:

``` shell
git clone https://bitbucket.org/mkerber/mpfree.git
cd mpfree
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Release .. && make
```

Then place the resulting `mpfree` executable (in the `build` folder) somewhere
along your PATH. You can do so, for example, with `sudo cp mpfree
/usr/local/bin`.

### Run the test suite

Before running the test suite you need to download the required datasets. To do
so, execute from the root directory of this project the `download_datasets.sh`
script.

``` shell
./download_datasets.sh
```

Finally, you can run the tests with
```shell
cargo test --release -- --test-threads 1
```
This command will execute the tests sequentially to reduce memory usage. If you have enough memory you can do
`cargo test --release` to do them in parallel.

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

## Contact

Ángel Javier Alonso (alonsohernandez@tugraz.at)
