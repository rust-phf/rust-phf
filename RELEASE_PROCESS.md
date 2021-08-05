# How to make a new release

- Update the version number and all the references to it
- Make a new release on GitHub (i.e. add a new git tag)
- Publish crates in order:
  1. `phf_shared`
  2. `phf_codegen`
  3. `phf_macros`
  4. `phf`
