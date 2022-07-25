# Changelog

## Unreleased


## 0.11.0

* Bump up MSRV to 1.60
* Crates' edition is now 2021 [#257]
* Remove `proc-macro-hack` dependency [#256]
* Now the `unicase` feature works fine with the `macros` feature, doesn't need to import `phf-macros` directly anymore. [#251]

[#251]: https://github.com/rust-phf/rust-phf/pull/251
[#256]: https://github.com/rust-phf/rust-phf/pull/256
[#257]: https://github.com/rust-phf/rust-phf/pull/257

## 0.10.1

* Allow serializing `Map` ([#244])
* Improve docs ([#240], [#243])

[#240]: https://github.com/rust-phf/rust-phf/pull/240
[#243]: https://github.com/rust-phf/rust-phf/pull/243
[#244]: https://github.com/rust-phf/rust-phf/pull/244

## 0.10.0

* Constify `len` and `is_empty` ([#224])
* Implement `Clone`, `Debug`, and `FusedIterator` ([#226])

[#224]: https://github.com/rust-phf/rust-phf/pull/224
[#226]: https://github.com/rust-phf/rust-phf/pull/226

## 0.9.1

**Yanked except for `phf-generator`, use 0.10.0 instead.**

* (phf-generator): Pin `criterion` version to keep MSRV
* Constify `len` and `is_empty` ([#224]) (**yanked**)
* Implement `Clone`, `Debug`, and `FusedIterator` ([#226]) (**yanked**)

## 0.9.0

* Our MSRV is now 1.41 or 1.46 (because of dependencies)
* `rand` dependency has been upgraded to 0.8
* Fix some crates' build on `no_std`
* Restore the `unicase` feature for `phf_macros`
* Allow using the owned `String` type for `phf` dynamic code generation
* Add back `OrderedMap` and `OrderedSet`
* (**breaking change**) Use `PhfBorrow` trait instead of `std::borrow::Borrow`

## 0.8.0

* `phf_macros` now works on stable.
* :tada: Fixed asymptotic slowdowns when constructing maps over very large datasets (+1M keys)
* (**breaking change**) The `core` features of `phf` and `phf_shared` have been changed to `std` default-features.
* (**breaking change**) The types in `phf_codegen` can be used with formatting macros via their `Display` impls and the `build()` methods no longer take `&mut Write`.
* Support has been added for using 128-bit integers as keys.
* (**breaking change**) The `OrderedMap` and `OrderedSet` types and the `phf_builder` crate have been removed due to lack of use.
* Byte strings now work correctly as keys.
* `unicase` dependency has been upgraded to 2.4.0
