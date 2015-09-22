#!/bin/bash

set -e

for crate in $(echo phf_shared phf_macros phf phf_codegen phf_generator); do
    (cd $crate && rm -f Cargo.lock && CARGO_TARGET_DIR=../target cargo doc --no-deps)
done
