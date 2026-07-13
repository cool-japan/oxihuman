# Contributing to OxiHuman

Thanks for considering a contribution. OxiHuman is a pure-Rust, Apache-2.0
parametric human body generator, and it operates under a small set of
licensing and provenance rules that are stricter than most Rust projects.
Please read the **Forbidden Sources** section below before writing any code
that touches mesh generation, morphing, or body-shape parameters — it is not
boilerplate.

## Getting started

1. Fork the repository and create a branch off `master`.
2. Follow the workspace layout in [README.md](README.md#development) — each
   crate under `crates/` is independently buildable with `cargo check -p
   <crate>`.
3. Make your change. Keep files under ~2,000 lines; if a file grows past
   that, split it into focused modules rather than letting it sprawl.
4. Run the test suite for the crate(s) you touched:

   ```sh
   cargo nextest run -p <crate> --all-features
   ```

5. Run Clippy and require zero warnings:

   ```sh
   cargo clippy -p <crate> --all-features -- -D warnings
   ```

6. Open a pull request against `master` describing what changed and why.
   Reference any related issue.

## Code standards

- **No `.unwrap()` / `.expect()` in non-test production code.** Propagate
  errors with `Result` and `thiserror`/`anyhow` as appropriate. Tests may use
  `unwrap()`/`expect()` where a panic is the correct failure mode for a test
  assertion.
- **Pure Rust by default.** New dependencies must not introduce C/C++/Fortran
  build requirements in the default feature set. If a native dependency is
  unavoidable, gate it behind a non-default Cargo feature.
- **Compression/decompression** must go through `oxiarc-*`. Do not add
  `zip`, `flate2`, `zstd`, `bzip2`, `lz4`, `tar`, `snap`, `brotli`, or
  `miniz_oxide` as dependencies.
- **Naming conventions**: standard Rust conventions apply — `snake_case` for
  variables/functions, `UpperCamelCase` for types, `SCREAMING_SNAKE_CASE`
  for constants.
- **Tests must use `std::env::temp_dir()`** (or an equivalent per-test
  tempdir) for any filesystem fixtures — never hardcode absolute paths.
- **No hardcoded absolute paths** anywhere in source or tests; use
  environment variables (see `MAKEHUMAN_DATA_DIR`, `OXIHUMAN_ASSETS_DIR` in
  the README) or paths relative to `CARGO_MANIFEST_DIR` / a tempdir.

## Forbidden sources

OxiHuman is format-compatible with MakeHuman (it reads the documented
`.target` and `.mhclo` file formats) but is **not** a derivative of the
MakeHuman Python application, and it must never become one. The following
are hard rules, not style preferences:

- **Never copy, translate, or paraphrase MakeHuman's AGPL-licensed Python
  source**, in any language, at any granularity — not a function, not a
  loop body, not a variable-naming pattern lifted line-by-line. All format
  support must be implemented from the documented file-format layout only
  (headers, keyword vocabulary, binary layout), not from reading or
  transliterating `shared/*.py` in the upstream MakeHuman repository. See
  [docs/CLEANROOM_AUDIT.md](docs/CLEANROOM_AUDIT.md) for the methodology
  used to verify this for the current codebase, and follow the same
  methodology (structural comparison, documented in a PR-visible note) for
  any new format-compatibility work.
- **Never use, fit, or reference SMPL, SMPL-X, SMPL-H, or STAR** body models
  or their shape/pose spaces (betas, pose parameter layout, joint
  regressors, blend-shape bases, or any derivative thereof). These are
  research/commercially-licensed body models (Meshcapade holds the
  commercial licence for SMPL/SMPL-X/STAR) and are incompatible with this
  project's Apache-2.0 + CC0 licensing model. Do not add code, tests, or
  documentation that imports, mimics, or targets their parameter spaces —
  this includes naming a struct or export target `Smpl*`/`SmplX*` even as a
  stub.
- **Never bundle community asset packs** (clothing, hair, skin textures,
  additional body targets, etc.) from the MakeHuman community asset
  repositories or elsewhere. Community assets carry individual,
  per-asset licences that are not necessarily CC0 or Apache-2.0 compatible.
  Only asset data with verified CC0-1.0 (or equally permissive) provenance,
  recorded per [PROVENANCE.md](PROVENANCE.md), may be added to
  `assets/packs/`.

If you are unsure whether something you want to contribute crosses one of
these lines, open an issue first and ask — it's much cheaper than reverting
a merged PR for a licensing problem.

## Reporting issues

Please include: OxiHuman version (or commit), target (native/wasm32),
the crate(s) involved, and a minimal reproduction (parameter JSON, `.target`
file, or test case) where possible.
