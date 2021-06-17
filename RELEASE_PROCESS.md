# How to make a new release

- Update the version number and all the references to it
- Make a new release on GitHub (i.e. add a new git tag)
- Publish crates in order:
  1. `puf_shared`
  2. `phf_generator`
  3. `phf_codegen`
  4. `phf_macros`
  5. `phf`
