#!/usr/bin/env bash
cd "$(dirname "$0")" || exit

EXPERIMENT_BIN=experiment_runner
MEMORY_LIMIT_GB=64

DATASETS="senate eleg netwsc hiv dragon sphere uniform circle torus swiss-roll"
ASYMPTOTICS_DATASETS="torus uniform"
ORDERS="reverse-lexicographic reverse-colexicographic forward-colexicographic forward-lexicographic random"
ORDER_TIMEOUT=$((2 * 60 * 60))

# Experiment with different orders
$EXPERIMENT_BIN order -t $ORDER_TIMEOUT $DATASETS $(printf ' -o %s' $ORDERS)

# Experiment with different methods
$EXPERIMENT_BIN removal $DATASETS

# mpfree comparison experiments
MODALITIES="strong-filtration-domination only-mpfree"
MPFREE_OUT_FILE="charts/compare_mpfree.csv"
ITER=0
for dataset in $DATASETS; do
  for modality in $MODALITIES; do
    if [[ "$dataset" = "dragon" ]] && [[ "$modality" = "only-mpfree" ]]; then
      # The dragon dataset consumes more than 64 GB of memory when building the flag filtration,
      # and will go into swap. We limit the amount of memory in that step to stop early.
      $EXPERIMENT_BIN mpfree -m $MEMORY_LIMIT_GB "$dataset" "$modality"
    elif [[ "$dataset" = "hiv" ]] && [[ "$modality" = "only-mpfree" ]]; then
      # The hiv dataset consumes more than 64 GB of memory on the mpfree step,
      # and will go into swap. We limit the amount of virtual memory with ulimit so mpfree stops early.
      (ulimit -v $((MEMORY_LIMIT_GB * 1024 * 1024)); $EXPERIMENT_BIN mpfree "$dataset" "$modality")
    else
      $EXPERIMENT_BIN mpfree "$dataset" "$modality"
    fi

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

$EXPERIMENT_BIN asymptotics $ASYMPTOTICS_DATASETS -n 200 -i 9 -r 1 -s 400
