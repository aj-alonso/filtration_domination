# Experiments

This directory has the required code to reproduce the experiments we show in the
paper. It also includes the scripts that generate all tables of the paper.

It consists of two main parts:
- `experiment_runner`, which is a Rust application that use the
  `filtration-domination` Rust library of the parent directory and runs the
  experiments. The data from the experiments in saved in `.csv` files in the
  `charts` directory.

- `process_charts.r` is an R script that reads all `.csv` files and generates
  automatically the LaTeX tables included in the paper.

All this is glued together with the `run_experiments.sh` shell script.
  
In addition, the `single_parameter` directory contains the code of the paper
"Swap, Shift and Trim to Edge Collapse a Filtration" by Marc Glisse and
Siddharth Pritam. This code is part of the GUDHI library, but has been included
here (with a little adaptation to be able to use it from `experiment_runner`) for
convenience. This is only use to include a comparison between the multi-parameter case and the single-parameter case.

## Requirements

We describe the preparations to setup the environment to run the experiments.
You can also skip most of this if you are using Docker, see [below](#Docker) for
further details.

To be able to run the experiments you need the following dependencies:
- The `mpfree` executable somewhere along your PATH.
- The `single_parameter` executable (found after compiling the code in the `single_parameter` folder) somewhere along your PATH.

After that, you can compile `experiment_runner`:

``` shell
cd experiment_runner
cargo build --release
```

Place the resulting executable, found in
`experiment_runner/target/release/experiment_runner` somewhere along your PATH.

Also, download the datasets here too by executing from the `experiments` directory the following:

``` shell
../download_datasets.sh
```

## Running the experiments

With all the requirements in place, you can run the experiments with

``` shell
./run_experiments.sh
```

After that, process the `.csv` files with R:

``` shell
Rscript process_charts.r
```

The tables and graphics will also be found in the `charts` directory.  Note that
you will need to install the R dependencies that `process_charts.r` requires,
read `process_charts.r` to see which are those.

### Docker

You can also use the Docker image we have included in the root directory, as
described in the `README.md` in the root directory. The image has the
executables `mpfree` and `single_parameter` already installed. You only need to
download the datasets (`../download_datasets.sh`) and run:

``` shell
docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/opt/filt filtration-domination/runner ./run_experiments.sh
```