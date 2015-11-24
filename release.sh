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

release_branch=release-v$new
git checkout -b $release_branch

sed -i '' -e "s/version = \"$old\"/version = \"$new\"/g" $tomls
sed -i '' -e "s/version = \"=$old\"/version = \"=$new\"/g" $tomls
sed -i '' -e "s|doc/v$old|doc/v$new|g" $tomls $libs README.md

git add .
git commit -ve -m "Release v$new"

git checkout release
git pull

git merge --no-ff $release_branch
git tag -a v$new

git checkout master
git pull

git merge --no-ff release
