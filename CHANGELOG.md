# Changelog

## Unreleased

* Our MSRV is now 1.46.0 (because of dependencies)
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
