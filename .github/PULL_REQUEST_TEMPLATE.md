## Summary

<!-- Briefly describe what this PR does and why. -->

## Related Issue(s)

<!-- Link any related issues: Fixes #123, Closes #456 -->

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Refactor / code cleanup
- [ ] Documentation update
- [ ] Dependency update
- [ ] CI / tooling change

## Checklist

### Correctness
- [ ] All existing tests pass (`cargo test --all-features`)
- [ ] New tests added for new behaviour or bug fixes
- [ ] No regressions in benchmark results (`cargo bench --all-features --no-run`)

### Code Quality
- [ ] No new `.unwrap()` calls in production code (use `?` or proper error handling)
- [ ] No new `.expect()` calls in production code without a clear justification comment
- [ ] All source files touched are under 2 000 lines (refactor if needed)
- [ ] Clippy passes without new warnings (`cargo clippy --all-features -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all`)

### COOLJAPAN Ecosystem Policy
- [ ] No `openblas` / `openblas-src` dependency introduced (use `oxiblas` instead)
- [ ] No `bincode` dependency introduced (use `oxicode` instead)
- [ ] No `zip` crate dependency introduced (use `oxiarc-archive` instead)
- [ ] No `rustfft` dependency introduced (use OxiFFT instead)
- [ ] No C/Fortran transitive dependencies added without a feature-gate
- [ ] Cargo versions use workspace inheritance (`*.workspace = true`) where applicable

### Documentation
- [ ] Public API items have rustdoc comments (`///`)
- [ ] If behaviour changed, `CHANGELOG.md` updated with a summary under `[Unreleased]`
- [ ] If a new crate was added, it is listed in the workspace `Cargo.toml`

### WASM (if touching `oxihuman-wasm`)
- [ ] WASM build still compiles (`cargo build -p oxihuman-wasm --target wasm32-unknown-unknown --release`)
- [ ] Final `.wasm` binary stays below 5 MB (lite) / 15 MB (hard limit)

## Testing Done

<!-- Describe manual or automated testing performed. -->

## Screenshots / Benchmarks (if applicable)

<!-- Paste relevant output, flamegraphs, or before/after numbers here. -->
