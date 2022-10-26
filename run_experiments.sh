#!/usr/bin/env bash

# Download datasets

# Check that the required executables are in the PATH

# Compile experiment binary in release mode
cd experiment || exit 1
cargo build --profile release
cd ..

EXPERIMENT_BIN=./experiment/target/release/experiment

#$EXPERIMENT_BIN orders

#$EXPERIMENT_BIN removal

DATASETS="senate netwsc"
MODALITIES="strong-filtration-domination only-mpfree"
MPFREE_OUT_FILE="charts/compare_mpfree.csv"
ITER=0
for dataset in $DATASETS; do
  for modality in $MODALITIES; do
    $EXPERIMENT_BIN mpfree "$dataset" "$modality"

    # Merge all produced CSVs.
    if [[ $ITER -eq 0 ]]; then
      # Copy CSV header if we are on the first iteration.
      head -1 "charts/compare_mpfree_${dataset}_${modality}.csv" >"$MPFREE_OUT_FILE"
    fi
    # Copy CSV body to merge the results.
    tail -n +2 "charts/compare_mpfree_${dataset}_${modality}.csv" >>"$MPFREE_OUT_FILE"
    ITER=$((ITER + 1))
  done
done

#$EXPERIMENT_BIN mpfree

#$EXPERIMENT_BIN multiple-iterations

#$EXPERIMENT_BIN asymptotics

#$EXPERIMENT_BIN random-densities
