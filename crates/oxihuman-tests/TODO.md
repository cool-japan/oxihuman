# oxihuman-tests -- TODO

> Version: 0.2.1 | Updated: 2026-07-13

## Status: Stable

Cross-crate integration test suite (publish = false). 0 stubs. 33,569 passing tests. 964 SLoC.

## Completed

- [x] Cross-crate integration test suite (integration_tests.rs)
- [x] Mesh generation round-trip tests
- [x] Morph target application tests
- [x] Export format validation tests
- [x] Error handling edge case tests

## Future Work

- [ ] Pack file I/O tests (only `export_lod_pack`/`export_lod_pack_with_stats` error-path checks exist today, in the invariant test; no dedicated pack-file I/O round trip found)
- [ ] WASM engine binding tests (no `wasm` reference anywhere in this crate's tests or its `Cargo.toml` dependency surface)
- [ ] CLI subcommand smoke tests (no CLI/subcommand/clap reference anywhere in this crate's tests or its `Cargo.toml` dependency surface)
