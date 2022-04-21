#!/usr/bin/env bash

rm -r target/doc
rm -r doc
cargo doc --no-deps
cp -r target/doc .

rm code.zip
# shellcheck disable=SC2046
zip -r code.zip clippy.toml Cargo.toml src doc $(git ls-files datasets) download_datasets.sh tests examples README.md
