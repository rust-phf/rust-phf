#!/bin/bash
set -e

if [ $# -ne 2 ]; then
    echo Usage: $0 "<old version>" "<new version>"
    exit 1
fi

old=$1
new=$2

tomls=$(find . -name Cargo.toml)
libs=$(find . -name lib.rs)

perl -pi -e "s/version = \"$old\"/version = \"$new\"/g" $tomls
perl -pi -e "s/version = \"=$old\"/version = \"=$new\"/g" $tomls
perl -pi -e "s|/$old/|/$new/|g" $tomls $libs README.md

git add .
git commit -ve -m "Release v$new"

git tag -a v$new
