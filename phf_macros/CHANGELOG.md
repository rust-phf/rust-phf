# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.11.1 (2022-08-08)

<csr-id-d40d663ca96f668bcd6f86cc691085629111c0b5/>
<csr-id-8225c4b90d6ee71483304e71342c269fca86a044/>

### Chore

 - <csr-id-d40d663ca96f668bcd6f86cc691085629111c0b5/> upgrade syn/proc-macro

### Bug Fixes

 - <csr-id-caf1ce71aed110fb44206ce2291154572ebfe9b7/> remove now-unnecessary `proc-macro-hack` crate usage
   Resolves <https://github.com/rust-phf/rust-phf/issues/255>.
   
   This resolves an issue with Windows Defender identifying `proc-macro-hack` as threats. It also sheds
   a depedency that is no longer necessary, now that the MSRV of this crate is 1.46 and
   `proc-macro-hack` is only useful for providing support for Rust versions 1.31 through 1.45. Per
   [upstream](https://github.com/dtolnay/proc-macro-hack):
   
   > **Note:** _As of Rust 1.45 this crate is superseded by native support for #\[proc\_macro\] in
   > expression position. Only consider using this crate if you care about supporting compilers between
   > 1.31 and 1.45._

### Other

 - <csr-id-8225c4b90d6ee71483304e71342c269fca86a044/> Update code for changes in Rust
   LitBinary is now LitByteStr

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 150 commits contributed to the release over the course of 2751 calendar days.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Replace handmade changelog with generated one by `cargo-smart-release` ([`cb84cf6`](https://github.com/rust-phf/rust-phf/commit/cb84cf6636ab52823c53e70d6abeac8f648a3482))
    - Add category to crates ([`32a72c3`](https://github.com/rust-phf/rust-phf/commit/32a72c3859997fd6b590e9ec092ae789d2acdf55))
    - Update repository links on Cargo.toml ([`1af3b0f`](https://github.com/rust-phf/rust-phf/commit/1af3b0fe1f8fdcae7ccc1bc8d51de309fb16a6bf))
    - Release 0.11.0 ([`d2efdc0`](https://github.com/rust-phf/rust-phf/commit/d2efdc08a7eb1d0d6c414b7b2ac41ce1fe1f9a43))
    - Merge pull request #257 from JohnTitor/edition-2021 ([`36ec885`](https://github.com/rust-phf/rust-phf/commit/36ec8854a9da4f295618e98d94aaf7150df2173e))
    - Make crates edition 2021 ([`b9d25da`](https://github.com/rust-phf/rust-phf/commit/b9d25da58b912d9927fbc41901631cd77836462b))
    - remove now-unnecessary `proc-macro-hack` crate usage ([`caf1ce7`](https://github.com/rust-phf/rust-phf/commit/caf1ce71aed110fb44206ce2291154572ebfe9b7))
    - Make "unicase + macros" features work ([`11bb242`](https://github.com/rust-phf/rust-phf/commit/11bb2426f0237b1ecea8c8038630b1231ede4871))
    - Extract `phf_macros` tests as a separated crate ([`8cf694d`](https://github.com/rust-phf/rust-phf/commit/8cf694d76e0991b4e24ecdc5d2a88bb74713d9cd))
    - Remove some stuff which is now unnecessary ([`6941e82`](https://github.com/rust-phf/rust-phf/commit/6941e825d09a98c1ea29a08ecd5fd605611584a4))
    - Refine doc comments ([`d8cfc43`](https://github.com/rust-phf/rust-phf/commit/d8cfc436059a1c2c3ede1afb0f9ec2333c046fc6))
    - Fix CI failure ([`d9b5ff2`](https://github.com/rust-phf/rust-phf/commit/d9b5ff23367d2bbcc385ff8243c7d972f45d459c))
    - Fix `phf` dev dep version ([`3cc6f05`](https://github.com/rust-phf/rust-phf/commit/3cc6f05cb07933af4cf886645d1170bdcb306b6b))
    - Prepare for release 0.10.0 ([`588ac25`](https://github.com/rust-phf/rust-phf/commit/588ac25dd5c0afccea084e6f94867328a6a30454))
    - Fix publish failure ([`fbb18f9`](https://github.com/rust-phf/rust-phf/commit/fbb18f925018fa621ce8a8d334f6746ae0f1d072))
    - Prepare for v0.9.1 ([`9b71978`](https://github.com/rust-phf/rust-phf/commit/9b719789149ef195ef5eba093b7e73255fbef8dc))
    - remove Slice type and fix some docs ([`99d3533`](https://github.com/rust-phf/rust-phf/commit/99d353390f8124a283da9202fd4d163e68bc1949))
    - Minor cleanups ([`8868d08`](https://github.com/rust-phf/rust-phf/commit/8868d088e2fed36fcd7741e9a1c5bf68bef4f46e))
    - Bless tests ([`dab668c`](https://github.com/rust-phf/rust-phf/commit/dab668ccc8b638548cd78678de8427ed5e765b21))
    - Fix the release failure ([`647f331`](https://github.com/rust-phf/rust-phf/commit/647f331d43dcf2b61625cccffbd31f95ad076d05))
    - Downgrade `phf` dev-dep version for now ([`7dd8a1b`](https://github.com/rust-phf/rust-phf/commit/7dd8a1b410fea96820bfe489f53f1c6fd9d64ba5))
    - Prepare 0.9.0 release ([`2ca46c4`](https://github.com/rust-phf/rust-phf/commit/2ca46c4f9c9083c128fcc6add33dc5986638940f))
    - Cleanup cargo metadata ([`a9e4b0a`](https://github.com/rust-phf/rust-phf/commit/a9e4b0a1e84825004fa66e938b870f83d3147d0d))
    - Fix test ([`ffa7e41`](https://github.com/rust-phf/rust-phf/commit/ffa7e41a767dd6021a7f42f012dab0befe6d0932))
    - Run rustfmt check on CI ([`1adfb30`](https://github.com/rust-phf/rust-phf/commit/1adfb305704cbced7c63e58b99bd53847298dbe6))
    - Run rustfmt ([`dd86c6c`](https://github.com/rust-phf/rust-phf/commit/dd86c6c103f25021b52144085b8fab0a94582bef))
    - Rename `unicase_support` to `unicase` ([`b47174b`](https://github.com/rust-phf/rust-phf/commit/b47174bb9ebbd68e41316e1aa39c6541a45356a6))
    - Run UI tests only on stable ([`7522b16`](https://github.com/rust-phf/rust-phf/commit/7522b160e76e981e430f6586dbfa8747c85f2f76))
    - Add back ordered_map, ordered_set ([`0ab0108`](https://github.com/rust-phf/rust-phf/commit/0ab01081e4bd8f40bc18ab554c95f217220228d5))
    - Improve implementation for unicase support ([`6957e47`](https://github.com/rust-phf/rust-phf/commit/6957e470b6fcd3b389440bf3d2ddcb12e1d38911))
    - Restore unicase_support for phf_macros ([`77e6cce`](https://github.com/rust-phf/rust-phf/commit/77e6cce1931fe8b43e434061a369f3620b3e97e0))
    - Use `[patch.crates-io]` section instead of path key ([`f47515b`](https://github.com/rust-phf/rust-phf/commit/f47515bce5c433214dbecee262a7a6f14e6a74d4))
    - Fix phf_macros on no_std ([`d7af3dc`](https://github.com/rust-phf/rust-phf/commit/d7af3dc96a67070e2f9000158d074825f0a9d592))
    - Merge pull request #194 from pickfire/patch-1 ([`caec346`](https://github.com/rust-phf/rust-phf/commit/caec346b07cf04cc7850e4aeeca077856b79256a))
    - Update stderrs ([`0f1407e`](https://github.com/rust-phf/rust-phf/commit/0f1407ec8aa6df74e7ed95dd073685295958d5d5))
    - Update expected test case output for latest nightly ([`e387f69`](https://github.com/rust-phf/rust-phf/commit/e387f69540138026ab679537322c94500876fe8d))
    - Release v0.8.0 ([`4060288`](https://github.com/rust-phf/rust-phf/commit/4060288dc2c1ebe3b0630e4016ed51935bb0c863))
    - Avoid missing main error in tests ([`1992222`](https://github.com/rust-phf/rust-phf/commit/19922229dfe8c25076ab13344a0b876fe2c3bda3))
    - Merge pull request #172 from kornelski/patch-1 ([`eee56c0`](https://github.com/rust-phf/rust-phf/commit/eee56c077c84cb84565eb3897c306865a3b29cc9))
    - upgrade syn/proc-macro ([`d40d663`](https://github.com/rust-phf/rust-phf/commit/d40d663ca96f668bcd6f86cc691085629111c0b5))
    - remove ordered_map, ordered_set, phf_builder ([`8ae2bb8`](https://github.com/rust-phf/rust-phf/commit/8ae2bb886841a69a4fc482f439e2374f2373ab15))
    - port compile-fail tests to trybuild ([`4a4256c`](https://github.com/rust-phf/rust-phf/commit/4a4256cf1963a349c8d63f4f93c7c562e8963d59))
    - create `Display` adapters for `phf_codegen` builders ([`93aa7ae`](https://github.com/rust-phf/rust-phf/commit/93aa7ae1de87345ea19f38e747283bc712384650))
    - Merge pull request #164 from abonander/perf-improvements ([`70129c6`](https://github.com/rust-phf/rust-phf/commit/70129c6fbcdf428ce9f1014eea935301ac70e410))
    - ignore compiletest ([`f1362b2`](https://github.com/rust-phf/rust-phf/commit/f1362b25674538ed02d41fcc9f7cc1c8ba6ec57c))
    - proc_macro_hygiene is not needed with proc-macro-hack ([`ab473a4`](https://github.com/rust-phf/rust-phf/commit/ab473a4c7fcc1a8e8a99594c261fe00b4ad96865))
    - Merge pull request #157 from abonander/array-formatting ([`8fc18be`](https://github.com/rust-phf/rust-phf/commit/8fc18be75dd3cb284b0b34b6c9e99c3c92544268))
    - Made macros work in stable ([`4fc0d1a`](https://github.com/rust-phf/rust-phf/commit/4fc0d1a8c3bcc3950082b614d8bfa4a0f63d6962))
    - implement support for 128-bit ints and fix high magnitude vals ([`5be5919`](https://github.com/rust-phf/rust-phf/commit/5be59199389c0703fff62f640eb1a0d19243fc48))
    - Fixed typo in benchmark ([`f46b2e1`](https://github.com/rust-phf/rust-phf/commit/f46b2e19622de2f845ea5eb8e8d4f54ece364242))
    - Fix tests ([`ae4ef3e`](https://github.com/rust-phf/rust-phf/commit/ae4ef3ea68d6baca0916b5ef2a15245ad78674ae))
    - Release v0.7.24 ([`1287414`](https://github.com/rust-phf/rust-phf/commit/1287414b1302d2d717c5f4be81accf4c12ccad48))
    - Reexport macros through phf crate ([`588fd1a`](https://github.com/rust-phf/rust-phf/commit/588fd1a785492afa5ad76db0556097e32e24387d))
    - Convert phf_macros to new-style proc-macros ([`5ae4131`](https://github.com/rust-phf/rust-phf/commit/5ae413129c391223782bc2944ec0ffbded103791))
    - Release v0.7.23 ([`a050b6f`](https://github.com/rust-phf/rust-phf/commit/a050b6f2a6b825bf0824339266ab9545340420d4))
    - Update to nightly-2018-08-23 ([`e03f536`](https://github.com/rust-phf/rust-phf/commit/e03f536f32a8a2a31d07e43b19e05c7d4fd1cb82))
    - Release 0.7.22 ([`ab88405`](https://github.com/rust-phf/rust-phf/commit/ab884054fa17eef915db2bdb5259c7aa71fbfea6))
    - Fix build ([`2071d25`](https://github.com/rust-phf/rust-phf/commit/2071d2515ff37590c45ee2e88cead583cdb81089))
    - Update to latest nightly ([`fcf758f`](https://github.com/rust-phf/rust-phf/commit/fcf758faa21c6c2c93dbab9fe6ac82a36bab0dd9))
    - Upgrade rand ([`e7b5a35`](https://github.com/rust-phf/rust-phf/commit/e7b5a35d14f6927a748f3c55a1c87b5b751ececd))
    - Release v0.7.21 ([`6c7e2d9`](https://github.com/rust-phf/rust-phf/commit/6c7e2d9ce17ff1b87507925bdbe87e6e682ed3e4))
    - Upgrade to rustc 1.16.0-nightly (c07a6ae77 2017-01-17) ([`dc756bf`](https://github.com/rust-phf/rust-phf/commit/dc756bfb1400715eeedd0dfaa394296274f59be4))
    - Don't ICE on bad syntax ([`e87e95f`](https://github.com/rust-phf/rust-phf/commit/e87e95fb96cfad1cc6699b828fb8994d2429f424))
    - Link to docs.rs ([`61142c5`](https://github.com/rust-phf/rust-phf/commit/61142c5aa168cff1bf53a6961ddc12012b49e1bb))
    - Cleanup ([`9278c47`](https://github.com/rust-phf/rust-phf/commit/9278c470b33571de286314cae555c4de9dd7d177))
    - Fix tests ([`5947cd1`](https://github.com/rust-phf/rust-phf/commit/5947cd14b9aac452f4f8feb25b57fd11240970ee))
    - Remove time dependency ([`98f56e5`](https://github.com/rust-phf/rust-phf/commit/98f56e53c212795e048c7baa0f488e1b294e9c37))
    - Dependency cleanup ([`f106aa6`](https://github.com/rust-phf/rust-phf/commit/f106aa66d85abfba3d627d12fd46a9b080c83e95))
    - Release v0.7.20 ([`f631f50`](https://github.com/rust-phf/rust-phf/commit/f631f50abfaf6ea3d6fc8caaada47975b6df3a62))
    - Update to Rust 1.15.0-nightly (7b3eeea22 2016-11-21) ([`39cc485`](https://github.com/rust-phf/rust-phf/commit/39cc485f777daaf2076f1da7337cc5ad7e9f00ad))
    - Release v0.7.19 ([`0a98dd1`](https://github.com/rust-phf/rust-phf/commit/0a98dd1865d12a3fa4cc27bdb38fa1e7374940d9))
    - Update phf_macros to Rust 1.14.0-nightly (7c69b0d5a 2016-11-01) ([`b7d2d4d`](https://github.com/rust-phf/rust-phf/commit/b7d2d4d36cb43a8fa159135250bd2265cb30f523))
    - Release v0.7.18 ([`3f71765`](https://github.com/rust-phf/rust-phf/commit/3f717650f4331f5dbb9d7a3f878228fcf1138729))
    - Fix for latest nightly ([`35e991b`](https://github.com/rust-phf/rust-phf/commit/35e991b11efca3bd065a28f661ab76f423a83601))
    - Release v0.7.17 ([`21ecf72`](https://github.com/rust-phf/rust-phf/commit/21ecf72101715e4754db95a64ecd7de5a37b7f14))
    - Fix for latest nightly ([`cb1ec95`](https://github.com/rust-phf/rust-phf/commit/cb1ec955442750fc712d155346beeb9562905602))
    - Remove dead code ([`df0d8e8`](https://github.com/rust-phf/rust-phf/commit/df0d8e8ae9b23482fb19ca70f1f3bd6cdfe59358))
    - Add compile-fail test for equivalent UniCase keys ([`711515a`](https://github.com/rust-phf/rust-phf/commit/711515ad0ab53c14303b6c659a1fb3c2b3c86df5))
    - Add UniCase support to phf_macros and bump unicase version ([`2af3abb`](https://github.com/rust-phf/rust-phf/commit/2af3abb00cafc85d43755e43767a2a8b274f6670))
    - Release v0.7.16 ([`8bf29c1`](https://github.com/rust-phf/rust-phf/commit/8bf29c10a878c83d73cc40385f0e96cb9cc95afa))
    - Update the TokenTree import ([`f404629`](https://github.com/rust-phf/rust-phf/commit/f40462989e75ce85de8c88d6faaee934d05fe006))
    - Release v0.7.15 ([`20f896e`](https://github.com/rust-phf/rust-phf/commit/20f896e6975cabb9cf9883b08eaa5b3da8597f11))
    - Release v0.7.14 ([`fee66fc`](https://github.com/rust-phf/rust-phf/commit/fee66fc20e33f2b119f830a8926f3b6e52abcf09))
    - Introduce a Slice abstraction for buffers ([`0cc3844`](https://github.com/rust-phf/rust-phf/commit/0cc38449c21f29bd9348e28c5719d650e16159cf))
    - Release v0.7.13 ([`4769a6d`](https://github.com/rust-phf/rust-phf/commit/4769a6d2ce1d392da06e4b3cb833a1cdccb1f1aa))
    - Update to Rust 2016-02-22 ([`c995514`](https://github.com/rust-phf/rust-phf/commit/c9955143ffdb07bf85a525494811bd96517bf688))
    - Release v0.7.12 ([`9b75ee5`](https://github.com/rust-phf/rust-phf/commit/9b75ee5ed14060c45a5785fba0387be09e698624))
    - Support byte string keys in phf_macros (fixes #76) ([`652beae`](https://github.com/rust-phf/rust-phf/commit/652beae0cac6711ab0931d8dc844cd291559dad7))
    - Release v0.7.11 ([`a004227`](https://github.com/rust-phf/rust-phf/commit/a0042277b181ec95fcbf29751b9a453f4f962ebb))
    - Update for changed return value of parser.eat ([`82da9f0`](https://github.com/rust-phf/rust-phf/commit/82da9f00f404634c09097f9116cda9e8e742d556))
    - Switch timing info back to a hint ([`771e781`](https://github.com/rust-phf/rust-phf/commit/771e781e704e581c1a103f56ed0f6f2a68917883))
    - Release v0.7.10 ([`c43154b`](https://github.com/rust-phf/rust-phf/commit/c43154b2661dc09620a7879c16f37b47d6ec03ae))
    - Update for syntax changes ([`3be2db8`](https://github.com/rust-phf/rust-phf/commit/3be2db8d9254214bf1571fafd466ed7d6b96af55))
    - Release v0.7.9 ([`b7d29df`](https://github.com/rust-phf/rust-phf/commit/b7d29dfe0df288b2da74de195f764eace1c8e443))
    - Registry now seems to live in rustc_plugin instead of rustc::plugin ([`ba8d701`](https://github.com/rust-phf/rust-phf/commit/ba8d7019599cb779b9f7ab983f6cc2aa4f422991))
    - Release v0.7.8 ([`aad0b9b`](https://github.com/rust-phf/rust-phf/commit/aad0b9b658fb970e3df60b066961aafca1a17c44))
    - Rustup ([`a6c43fa`](https://github.com/rust-phf/rust-phf/commit/a6c43fa25e06684121df6a93b2b90405d8e0fc2e))
    - Release v0.7.7 ([`c9e7a93`](https://github.com/rust-phf/rust-phf/commit/c9e7a93f4d6f85a72651aba6187e4c956d8c1167))
    - rustup for phf_macros ([`4c51ffc`](https://github.com/rust-phf/rust-phf/commit/4c51ffc6d63f768dea75cab65ad6cb809bce9bb4))
    - Run through rustfmt ([`58e2223`](https://github.com/rust-phf/rust-phf/commit/58e222380b7fc9609a055cb5a6110ba04e47d677))
    - Release v0.7.6 ([`5bcd5c9`](https://github.com/rust-phf/rust-phf/commit/5bcd5c95215f5aa29e133cb2912662085a8158f0))
    - Release v0.7.5 ([`fda44f5`](https://github.com/rust-phf/rust-phf/commit/fda44f550401c1bd4aad29bb2c07030b86761028))
    - Update code for changes in Rust ([`8225c4b`](https://github.com/rust-phf/rust-phf/commit/8225c4b90d6ee71483304e71342c269fca86a044))
    - Macro assemble benchmark map and match to ensure sync ([`a2486ed`](https://github.com/rust-phf/rust-phf/commit/a2486eda19c647d16c9976bb33ba8634388a0569))
    - Add benchmarks ([`9585cc3`](https://github.com/rust-phf/rust-phf/commit/9585cc3c0391725d02f6199eaed500ba5fafcaf3))
    - Release v0.7.4 ([`c7c0d3c`](https://github.com/rust-phf/rust-phf/commit/c7c0d3c294126157f0275a05b7c3a65c419234a1))
    - Update PhfHash to mirror std::hash::Hash ([`96ef156`](https://github.com/rust-phf/rust-phf/commit/96ef156baae669b233673d6be2b96617ad48551e))
    - Release v0.7.3 ([`77ea239`](https://github.com/rust-phf/rust-phf/commit/77ea23917e908b10c4c5c463671a8409292f8661))
    - Release v0.7.2 ([`642b69d`](https://github.com/rust-phf/rust-phf/commit/642b69d0100a4ee7ec6e430ef1351bd1f28f9a4a))
    - Add an index test ([`f51f449`](https://github.com/rust-phf/rust-phf/commit/f51f449261ddd8ad30bfb5507b166e7980df1aa7))
    - Release v0.7.1 ([`9cb9de9`](https://github.com/rust-phf/rust-phf/commit/9cb9de911ad4e16964f0def29780dde1630c3619))
    - Fix phf-macros ([`6c98e9f`](https://github.com/rust-phf/rust-phf/commit/6c98e9f16a6d9ebf11e0a9c8e9ff91b4b320d2af))
    - Release v0.7.0 ([`555a690`](https://github.com/rust-phf/rust-phf/commit/555a690561673597aee068650ac884bbcc2e31cf))
    - Stabilize phf ([`e215273`](https://github.com/rust-phf/rust-phf/commit/e2152739cbdd471116d88bb4a9cea4cdfede1e42))
    - Release v0.6.19 ([`5810d30`](https://github.com/rust-phf/rust-phf/commit/5810d30ef2162f33cfb4da99c65b7344c7f2913b))
    - Release v0.6.18 ([`36efc72`](https://github.com/rust-phf/rust-phf/commit/36efc721478d097fba1e5458cbdd9f288637abae))
    - Fix for upstream changes ([`eabadcf`](https://github.com/rust-phf/rust-phf/commit/eabadcf7e8af351ba8f07d86746e35adc8c5812e))
    - Release v0.6.17 ([`271ccc2`](https://github.com/rust-phf/rust-phf/commit/271ccc27d885363d4d8c549f75624d08c48e56c5))
    - Release v0.6.15 ([`ede14df`](https://github.com/rust-phf/rust-phf/commit/ede14df1e574674852b09bcafff4ad549ebfd4ae))
    - Remove broken test ([`f54adb7`](https://github.com/rust-phf/rust-phf/commit/f54adb783a71678c9397b4d7c1e02ee82b9646b8))
    - Release v0.6.14 ([`cf64ebb`](https://github.com/rust-phf/rust-phf/commit/cf64ebb8f769c9f12c9a03d05713dde6b8caf371))
    - Release v0.6.13 ([`4fdb533`](https://github.com/rust-phf/rust-phf/commit/4fdb5331fd9978ca3e180a06fb2e34627f50fb77))
    - Fix warnings and use debug builders ([`4d28684`](https://github.com/rust-phf/rust-phf/commit/4d28684b72333e911e23b898b5780947d49822a5))
    - Release v0.6.12 ([`59ca586`](https://github.com/rust-phf/rust-phf/commit/59ca58637206c9806c13cc24cb35cb7d0ce9d23f))
    - Fix phf_macros ([`6567152`](https://github.com/rust-phf/rust-phf/commit/6567152be9e018a99fedf6e54017d827812b8f13))
    - Release v0.6.11 ([`e1e6d3b`](https://github.com/rust-phf/rust-phf/commit/e1e6d3b40a6babddd0989406f2b4e952443ff52e))
    - Release v0.6.10 ([`fc45373`](https://github.com/rust-phf/rust-phf/commit/fc45373b34a461664f532c5108f3d2625172c128))
    - Add doc URLs ([`4605db3`](https://github.com/rust-phf/rust-phf/commit/4605db3e7e0c4bef09ccf6c09c7dbcc36b707a9f))
    - Add documentation for phf_macros ([`8eca797`](https://github.com/rust-phf/rust-phf/commit/8eca79711f33d04ad773a023581b6bd0a6f1efdc))
    - Move generation logic to its own crate ([`cfeee87`](https://github.com/rust-phf/rust-phf/commit/cfeee8714caa4ecb3199df2a2ac149fe6a28ecc0))
    - Move tests to phf_macros ([`40dbc32`](https://github.com/rust-phf/rust-phf/commit/40dbc328456003484716021cc317156967f1b2c1))
    - Release v0.6.9 ([`822f4e3`](https://github.com/rust-phf/rust-phf/commit/822f4e3fb127dc02d36d802803d71aa5b98bed3c))
    - More fixes ([`0c04b9c`](https://github.com/rust-phf/rust-phf/commit/0c04b9cb2679a63394778a7362ef14441b6c2032))
    - Release v0.6.8 ([`cd637ca`](https://github.com/rust-phf/rust-phf/commit/cd637cafb6d37b1901b6c119a7d26f253e9a288e))
    - Release v0.6.7 ([`bfc36c9`](https://github.com/rust-phf/rust-phf/commit/bfc36c979225f652cdb72f3b1f2a25e77b50ab8c))
    - Fix for upstream changes ([`5ff7040`](https://github.com/rust-phf/rust-phf/commit/5ff70403a1b12c30206b128ac619b31c69e42eb4))
    - rustup to current master ([`f6922e2`](https://github.com/rust-phf/rust-phf/commit/f6922e245752b4932f9a3a420c1f8d10e66e0b78))
    - Release v0.6.6 ([`b09a174`](https://github.com/rust-phf/rust-phf/commit/b09a174a166c7744c5989bedc6ba68340f6f7fd1))
    - Release v0.6.5 ([`271e784`](https://github.com/rust-phf/rust-phf/commit/271e7848f35b31d6ce9fc9268de173738464bfc8))
    - Move docs to this repo and auto build them ([`f8ef160`](https://github.com/rust-phf/rust-phf/commit/f8ef160480e2d4ce72fa7afb6ebce70e45acbc76))
    - Release v0.6.4 ([`6866c1b`](https://github.com/rust-phf/rust-phf/commit/6866c1bf5ad5091bc969f1356884aa86c27458cb))
    - Remove unused feature ([`2ee5f78`](https://github.com/rust-phf/rust-phf/commit/2ee5f788d493d929b669550c144ff23aad52721b))
    - InternedString.get() removal; brings us to rustc 1.0.0-dev (80627cd3c 2015-02-07 12:01:31 +0000) ([`3150bf0`](https://github.com/rust-phf/rust-phf/commit/3150bf0d608b051f2c8db3826ee21ce593f4f61c))
    - Release v0.6.3 ([`b0c5e3c`](https://github.com/rust-phf/rust-phf/commit/b0c5e3cb69742f81160ea80a3ba1782a0b4e01a2))
    - Use out of tree rand ([`9e1623b`](https://github.com/rust-phf/rust-phf/commit/9e1623bc7d1b8a432cdae47187eab40fa168401f))
    - Release v0.6.2 ([`d9ddf45`](https://github.com/rust-phf/rust-phf/commit/d9ddf45b15ba812b0d3acedffb08e901742e56c4))
    - Release v0.6.1 ([`ca0e9f6`](https://github.com/rust-phf/rust-phf/commit/ca0e9f6b9c737f3d11bcad2f4624bb5603a8170e))
    - Fix for stability changes ([`f7fb510`](https://github.com/rust-phf/rust-phf/commit/f7fb510dfe67f11522a2d214bd14d21f910bfd7b))
    - Release v0.6.0 ([`09d6870`](https://github.com/rust-phf/rust-phf/commit/09d687053caf4d321f72907528573b3334fae3c2))
    - Rename phf_mac to phf_macros ([`c50d107`](https://github.com/rust-phf/rust-phf/commit/c50d1077b1d53fccd703021911a7100b8937bbc7))
</details>

