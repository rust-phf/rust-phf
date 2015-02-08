#!/bin/bash

set -e

mkdir doc

for crate in $(echo phf_shared phf_macros phf); do
    mkdir -p $crate/target
    ln -s -t $crate/target ../../doc
    (cd $crate && cargo doc --no-deps)
done
