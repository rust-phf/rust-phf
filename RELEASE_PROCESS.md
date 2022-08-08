# How to make a new release

Since v0.11.1, this repository uses [`cargo-smart-release`](https://crates.io/crates/cargo-smart-release) to release crates.

## Installation

```sh
cargo install cargo-smart-release
```

## Before running `cargo-smart-release`

If the next version has a major change or bumps MSRV, i.e. it increases a minor version on `v0.Y.Z` or a major version on `vX.Y.Z`,
update the versions mentioned on the README and doc comments (and CI config if it touches MSRV).

## Run `cargo-smart-release`

First, just run:

```sh
cargo smart-release phf phf_codegen phf_generator phf_macros phf_shared
```

This would show what `cargo-smart-release` would do, e.g. how it updates the version number, changelog, etc.
If you satisfy the output, run:

```sh
cargo smart-release -u -e phf phf_codegen phf_generator phf_macros phf_shared
```

This would make actual releases. `-e` means that it executes actual releases and `-u` means that it always updates crates-index.
`cargo-smart-release` also takes care of the Git tags.

After executing it, ensure that all the releases are actually happened and the tags are pushed.
If all the things are fine, that's it!
