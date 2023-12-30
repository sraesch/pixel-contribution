# Pixel Contribution
Small research project for determining the pixel contribution of a geometry from all views.

## Getting Started

### Build
In order to build the debug-version run
```bash
cargo build
```
and
```bash
cargo build --release
```
for the release build.
The resulting artefact is then either in `./target/debug/pixel-contribution` or `./target/release/pixel-contribution`, respectively.

### Run
The program has the mandatory parameter `-i, --input-file`. The parameter expects a path to a 3D/CAD-file that is supported by the library `https://crates.io/crates/cad_import` and is the 3D-data that is being used for the pixel contribution calculation.
For more information execute the `--help` option.