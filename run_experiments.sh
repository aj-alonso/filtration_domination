#!/usr/bin/env bash

# Download datasets

# Check that the required executables are in the PATH

# Compile experiment binary in release mode
cd experiment || exit 1
cargo build --profile release
cd ..

EXPERIMENT_BIN=./experiment/target/release/experiment

DATASETS="senate eleg netwsc hiv dragon sphere uniform circle torus swiss-roll"

# Create output directory if it does not exists
mkdir -p charts

# Experiment with different orders
ORDERS="reverse-lexicographic reverse-colexicographic"
$EXPERIMENT_BIN order $DATASETS $(printf ' -o %s' $ORDERS)

# Experiment with different methods
$EXPERIMENT_BIN removal $DATASETS

# mpfree comparison experiments
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

$EXPERIMENT_BIN multiple-iterations $DATASETS

$EXPERIMENT_BIN random-densities $DATASETS

ASYMPTOTICS_DATASETS="torus uniform"
$EXPERIMENT_BIN asymptotics $ASYMPTOTICS_DATASETS -n 200 -i 3 -s 400


# Process charts and produce tables and graphics.
PROCESS_CHARTS_SCRIPT="Rscript process_charts.r"
$PROCESS_CHARTS_SCRIPT
