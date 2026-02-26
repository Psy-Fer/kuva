# Contributing to visus

Thank you for considering a contribution. This document describes how the codebase is laid out and what needs to be updated for each type of change.

## Quick orientation

```
src/
  lib.rs                  — crate root; public re-exports
  plot/                   — one file per plot type (builder-pattern structs)
  render/
    plots.rs              — Plot enum wrapping every plot type
    render.rs             — render_*() functions; render_multiple() dispatcher
    layout.rs             — Layout (user config) + ComputedLayout (pixel math)
    figure.rs             — Figure grid layout
    palette.rs            — Palette / colour cycles
    theme.rs              — Theme definitions
    annotations.rs        — TextAnnotation, ReferenceLine, ShadedRegion
    axis.rs               — Axis drawing, tick marks, category labels
    render_utils.rs       — Statistical helpers (KDE, regression, ticks, UPGMA)
  backend/
    svg.rs                — SvgBackend: Scene → SVG string
    png.rs                — PngBackend (feature: png)
    pdf.rs                — PdfBackend (feature: pdf)
  bin/visus/              — CLI binary (clap subcommands)

tests/                    — Integration tests; SVG output to test_outputs/
examples/                 — One Rust example per plot type (used by gen_docs.sh)
examples/data/            — TSV files for CLI smoke tests and examples
docs/src/                 — mdBook source
  plots/                  — One .md page per plot type
  assets/                 — Generated SVG images for docs pages
  gallery.md              — One-card-per-plot scrollable overview
scripts/
  gen_docs.sh             — Regenerates docs/src/assets/ SVGs
  smoke_tests.sh          — End-to-end CLI tests against example data
```

---

## Adding a new plot type

Work through every item below before opening a PR. Each area is listed in the order you would naturally touch it.

### Library

- [ ] **`src/plot/<name>.rs`** — new file with the plot struct and builder methods (`new()`, `with_data()`, `with_color()`, etc.). Follow the existing pattern: all fields private, all setters return `Self`.
- [ ] **`src/plot/mod.rs`** — add `pub mod <name>;` and re-export key public types (`pub use <name>::<Name>Plot;`).
- [ ] **`src/render/plots.rs`** — add a variant to the `Plot` enum; implement `bounds()`, `colorbar_info()`, and `set_color()` for it.
- [ ] **`src/render/render.rs`** — write `add_<name>()` and `render_<name>()` functions; add the variant to the `match` inside `render_multiple()`; if pixel-space (no axes), add to the `skip_axes` check.
- [ ] **`src/render/layout.rs`** — if the plot uses categories, extend `auto_from_plots()` to populate `x_categories` / `y_categories` from it.
- [ ] **`src/lib.rs`** — if any types need to be at the crate root, add re-exports.

### Tests

- [ ] **`tests/<name>_basic.rs`** (or `tests/<name>_svg.rs`) — integration tests that write SVGs to `test_outputs/`; at minimum: one basic render test, one test verifying a key SVG element is present, one test verifying the legend if applicable.
- [ ] Run `cargo test` — all 196+ existing tests must still pass.

### CLI (if a `visus <name>` subcommand is warranted)

- [ ] **`src/bin/visus/<name>.rs`** — clap Args struct (with `/// doc comment` for the man page) + `run()` function.
- [ ] **`src/bin/visus/main.rs`** — add `mod <name>;` at the top, add variant to `Commands` enum, add arm to the `match` in `main()`.
- [ ] **`scripts/smoke_tests.sh`** — add at least one invocation using `examples/data/`.
- [ ] **`tests/cli_basic.rs`** — add at minimum a test that runs the subcommand and checks for SVG output, plus a content-verification test.
- [ ] **`examples/data/`** — add a TSV file if the existing 22 files don't cover the new plot; regenerate via `examples/data/generate.py` if needed.
- [ ] **`docs/src/cli/index.md`** — add the subcommand to the flag-reference table and example invocations section.
- [ ] **`man/visus.1`** — regenerate: `cargo build --bin visus && ./target/debug/visus man > man/visus.1`.

### Documentation

- [ ] **`examples/<name>.rs`** — a self-contained Rust example that generates several representative SVG variants; this is what `gen_docs.sh` calls to produce docs assets.
- [ ] **`scripts/gen_docs.sh`** — add invocations for the new example to generate all `docs/src/assets/<name>/*.svg` files.
- [ ] **Run `bash scripts/gen_docs.sh`** — confirm all assets generate without errors.
- [ ] **`docs/src/plots/<name>.md`** — documentation page: one-line description, import path, builder method table, embedded SVG examples with code snippets.
- [ ] **`docs/src/SUMMARY.md`** — add link to the new plot page under `# Plot Types`.
- [ ] **`docs/src/gallery.md`** — add a gallery card using the most visually rich asset.
- [ ] **`README.md`** — add a row to the plot types table.

### Visual inspection

- [ ] Run `cargo test` and open `test_outputs/` — visually inspect the new plot's SVGs and scan neighbouring plots for unexpected layout regressions (margins, label clipping, legend overlap).
- [ ] Run `bash scripts/smoke_tests.sh` and open `smoke_test_outputs/` — verify all 22+ existing CLI outputs still look correct.

### Housekeeping

- [ ] **`CHANGELOG.md`** — add an entry under `## [Unreleased]`.
- [ ] **`README.md`** — mark the new plot as done if it was listed in the TODO section

---

## Adding a new feature (non-plot-type)

- [ ] Implement in the relevant `src/` file(s).
- [ ] Add tests covering the new behaviour — both a positive case and at least one edge case.
- [ ] Update the relevant `docs/src/` page(s) if the feature is user-visible.
- [ ] If the feature affects rendered output, run the visual inspection steps above.
- [ ] **`CHANGELOG.md`** — add an entry under `## [Unreleased]`.

---

## Fixing a bug

- [ ] Fix in the relevant file.
- [ ] Add a regression test that would have caught the bug before the fix.
- [ ] If the fix changes rendered output, run the visual inspection steps above and regenerate any affected doc assets.
- [ ] **`CHANGELOG.md`** — add an entry under `## [Unreleased]`.

---

## Visual inspection checklist

When any rendering change is made, open `test_outputs/` and verify:

- [ ] No text is clipped at the canvas edges (titles, axis labels, tick labels, legend text).
- [ ] Legend does not overlap the plot area.
- [ ] Colour bar (if present) is fully visible and labelled.
- [ ] Log-scale plots have correct 1-2-5 tick placement.
- [ ] Rotated tick labels (Manhattan, bar with many categories) have enough bottom margin.
- [ ] Pixel-space plots (Pie, UpSet, Chord, Sankey, PhyloTree, Synteny) have no spurious axes drawn.
- [ ] Dark and publication/minimal themes both render without contrast issues.

---

## Build commands reference

```bash
cargo build                              # library
cargo build --bin visus                  # CLI binary SVG output
cargo build --bin visus --features png   # CLI + SVG + PNG output
cargo build --bin visus --features pdf   # CLI + SVG + PDF output
cargo build --bin visus --features all   # CLI + SVG + PNG + PDF output
cargo test                               # all tests
cargo test <test_name>                   # single test
cargo test --test cli_basic              # CLI integration tests
bash scripts/smoke_tests.sh              # CLI smoke tests (all 22+ subcommands)
bash scripts/gen_docs.sh                 # regenerate docs SVG assets
cargo build --bin visus && ./target/debug/visus man > man/visus.1  # regenerate man page
```
