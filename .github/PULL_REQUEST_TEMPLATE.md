## Description

<!-- What does this PR do? Why? -->

## Type of change

- [ ] New plot type
- [ ] New feature / API addition
- [ ] Bug fix
- [ ] Documentation / assets only
- [ ] Refactor / housekeeping

---

## Checklist

### Library (new plot type)
- [ ] `src/plot/<name>.rs` — struct + builder methods
- [ ] `src/plot/mod.rs` — `pub mod` + re-export
- [ ] `src/render/plots.rs` — `Plot` enum variant + `bounds()` / `colorbar_info()` / `set_color()`
- [ ] `src/render/render.rs` — `render_<name>()`, added to `render_multiple()` match, `skip_axes` if pixel-space
- [ ] `src/render/layout.rs` — `auto_from_plots()` extended if categories needed

### Tests
- [ ] New test file in `tests/` with ≥ basic render + SVG content + legend tests
- [ ] `cargo test` — all existing tests still pass

### CLI (if applicable)
- [ ] `src/bin/visus/<name>.rs` — Args struct (with `/// doc comment`) + `run()`
- [ ] `src/bin/visus/main.rs` — module, Commands variant, match arm
- [ ] `scripts/smoke_tests.sh` — at least one invocation
- [ ] `tests/cli_basic.rs` — SVG output test + content verification test
- [ ] `docs/src/cli/index.md` — subcommand entry
- [ ] `man/visus.1` — regenerated (`./target/debug/visus man > man/visus.1`)

### Documentation
- [ ] `examples/<name>.rs` — Rust example for doc asset generation
- [ ] `scripts/gen_docs.sh` — invocations added; `bash scripts/gen_docs.sh` runs clean
- [ ] `docs/src/plots/<name>.md` — documentation page with embedded SVGs
- [ ] `docs/src/SUMMARY.md` — link added
- [ ] `docs/src/gallery.md` — gallery card added
- [ ] `README.md` — plot types table updated

### Visual inspection
- [ ] Opened `test_outputs/` — new plot SVGs look correct
- [ ] Scanned neighbouring plots in `test_outputs/` for layout regressions
- [ ] `bash scripts/smoke_tests.sh` — all existing smoke test outputs still look correct
- [ ] No text clipped, no legend overlap, no spurious axes on pixel-space plots

### Housekeeping
- [ ] `CHANGELOG.md` — entry added under `## [Unreleased]`
- [ ] `README.md` — item marked done in TODO section if applicable
