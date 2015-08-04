#!/bin/bash

set -e

mkdir doc

for crate in $(echo phf_shared phf_macros phf phf_codegen phf_generator); do
    rm -r $crate/target
    mkdir -p $crate/target
    (cd $crate/target && ln -s ../../doc)
    (cd $crate && cargo doc --no-deps)
done
