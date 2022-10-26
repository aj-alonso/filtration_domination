#!/usr/bin/env bash
### Little utility to compare the barcodes obtained by the different single-parameter algorithms.

DATASETS="senate eleg netwsc hiv dragon sphere uniform circle torus swiss-roll"
ALGS="filtration-domination-single strong-filtration-domination-single single-parameter"
BARCODES_DIR="barcodes"

mkdir -p "$BARCODES_DIR"

for dataset in $DATASETS; do
  for alg in $ALGS; do
    BARCODE_IN="tmp/single_parameter_edges_${dataset}_${alg}.txt"
    if [[ -f "$BARCODE_IN" ]]; then
      BARCODE_OUT="${BARCODES_DIR}/barcode_${dataset}_${alg}.txt"
      ./compute_barcodes.py "$BARCODE_IN" >"$BARCODE_OUT"
      sort "$BARCODE_OUT" -o "$BARCODE_OUT"
      md5sum "$BARCODE_OUT"
    else
      echo "File $BARCODE_IN not found."
    fi
  done
done
