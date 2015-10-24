#!/bin/bash
set -e

export CARGO_TARGET_DIR=target

for toml in $(find . -name "Cargo.toml"); do
    cargo update --manifest-path $toml
    cargo doc --no-deps --manifest-path $toml
done
