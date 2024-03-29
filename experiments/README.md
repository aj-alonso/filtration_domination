# Experiments

This directory has the required code to reproduce the experiments we show in the
paper. It also includes the scripts that generate all tables of the paper.

It consists of two main parts:
- `experiment_runner`, which is a Rust application that uses the
  `filtration-domination` Rust library of the parent directory and runs the
  experiments. The data from the experiments is saved in `.csv` files in the
  `charts` directory.

- `process_charts.r` is an R script that reads all the `.csv` files and generates
  automatically the LaTeX tables included in the paper.

All this is glued together with the `run_experiments.sh` shell script.
  
In addition, the `single_parameter` directory contains a little utility to
collapse a single-parameter filtration using the algorithm of the paper "Swap,
Shift and Trim to Edge Collapse a Filtration" by Marc Glisse and Siddharth
Pritam, as implemented in the [GUDHI library](https://gudhi.inria.fr/). This
utility is used by `experiment_runner` to compare the multi-parameter case and
the single-parameter case.

## Requirements

We describe the preparations to set up the environment to run the experiments.
You can also skip most of this if you are using Docker, see [below](#Docker) for
further details.

To be able to run the experiments you need the following dependencies:
- The `mpfree` executable somewhere along your PATH.
- The [GUDHI](https://gudhi.inria.fr) library, at least version 3.6.0,
  installed, in order to compile the `single_parameter` executable.
- The `single_parameter` executable (found after compiling the code in the
  `single_parameter` folder) somewhere along your PATH.

To build the `single_parameter` executable:

``` shell
cd single_parameter
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Release .. && make
```
Then place the resulting `single_parameter` executable (in the `build` folder) somewhere
along your PATH. You can do so, for example, with `sudo cp single_parameter
/usr/local/bin`.

After that, you can compile `experiment_runner`:

``` shell
cd experiment_runner
cargo build --release
```

Place the resulting executable, found in
`experiment_runner/target/release/experiment_runner` somewhere along your PATH,
e.g. `sudo cp experiment_runner/target/release/experiment_runner /usr/local/bin`.

Also, download the datasets here too by executing from the `experiments` directory the following:

``` shell
../download_datasets.sh
```

## Running the experiments

With all the requirements in place, you can run the experiments with

``` shell
./run_experiments.sh
```

There is a configurable timeout and memory consumption limit, read
`run_experiments.sh` for details.

After that, you can process the `.csv` files to generate the tables and graphics:

``` shell
Rscript process_charts.r
```

The tables and graphics will also be found in the `charts` directory.  Note that
you will need to install the R dependencies that `process_charts.r` requires:
[tidyverse](https://www.tidyverse.org/) and
[kableExtra](https://cran.r-project.org/web/packages/kableExtra/index.html). You
may install these dependencies by running the following command in an R console
(the `R` command):

``` r
install.packages(c("tidyverse", "kableExtra"))
```

### Docker

You can also use the Docker image we have included in the root directory, as
described in the `README.md` in the root directory. The image has the
executables `mpfree` and `single_parameter` already installed. You only need to
download the datasets (`../download_datasets.sh`) and run:

``` shell
docker run --rm --user "$(id -u)":"$(id -g)" -v "$PWD":/opt/filt filtration-domination/runner ./run_experiments.sh
```
