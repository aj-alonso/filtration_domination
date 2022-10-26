#!/usr/bin/env bash

# Download datasets

# Check that the required executables are in the PATH

# Compile experiment binary in release mode
cd experiment || exit 1
cargo build --profile release
cd ..

EXPERIMENT_BIN=./experiment/target/release/experiment

$EXPERIMENT_BIN orders

$EXPERIMENT_BIN removal

$EXPERIMENT_BIN mpfree

$EXPERIMENT_BIN multiple-iterations

$EXPERIMENT_BIN asymptotics

$EXPERIMENT_BIN random-densities
