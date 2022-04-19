#!/usr/bin/env bash

echo "Downloading the datasets from https://github.com/n-otter/PH-roadmap ..."

DOWNLOAD_PREFIX="https://raw.githubusercontent.com/n-otter/PH-roadmap/master/data_sets/roadmap_datasets_distmat/"
Senate="senate104_edge_list.txt_0.68902_distmat.txt"
Eleg="celegans_weighted_undirected_reindexed_for_matlab_maxdist_2.6429_SP_distmat.txt"
Netwsc="network379_edge_list.txt_38.3873_distmat.txt"
HIV="HIV1_2011.all.nt.concat.fa_hdm.txt"
Dragon="dragon_vrip.ply.txt_2000_.txt_distmat.txt"

mkdir -p "datasets"
cd "datasets" || exit 1

for dataset in $Senate $Eleg $Netwsc $HIV $Dragon; do
    curl -O "${DOWNLOAD_PREFIX}$dataset"
done

echo "Done!"
