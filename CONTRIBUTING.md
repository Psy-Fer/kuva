# Contributing to kuva

Thank you for considering a contribution. This document describes how the codebase is laid out and what needs to be updated for each type of change.

## Branching

**Open all pull requests against the `dev` branch**, not `main`. The `main` branch tracks released versions only; `dev` is where work-in-progress is integrated before a release.

## Toolchain & formatting

The repo pins its dev/CI toolchain via `rust-toolchain.toml` — `rustup` picks this up automatically (installing it on first use if needed), so `cargo build`/`test`/`clippy`/`fmt` all use the same compiler, rustfmt, and clippy version as CI without any manual setup. `rust-version` in `Cargo.toml` is a separate, lower floor (the oldest compiler the *library* must compile on); CI's `msrv` job builds the no-feature and non-PDF-feature combinations against it. The `pdf` feature has its own, higher floor (`krilla`'s MSRV, tracked in `[package.metadata.msrv] pdf_feature` in `Cargo.toml` — kept separate from `rust-version` deliberately, see README.md's note); CI's `pdf-msrv` job builds `--features cli,full,typst-math` against that value instead (the same combination the release binaries use; `typst-math` has a lower floor of its own, so building it here covers both). Bump `pdf_feature` in lockstep if krilla's own MSRV changes.

Run `cargo ci-fmt` before opening a PR — CI fails on any formatting diff. If you need to `git blame` a file, run this once per clone so reformatting-only commits (tagged in `.git-blame-ignore-revs`) don't obscure real authorship:

```bash
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

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
  bin/kuva/              — CLI binary (clap subcommands)

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
- [ ] Run `cargo test --features cli,full` — all existing tests must still pass.

### CLI (if a `kuva <name>` subcommand is warranted)

- [ ] **`src/bin/kuva/<name>.rs`** — clap Args struct (with `/// doc comment` for the man page) + `run()` function.
- [ ] **`src/bin/kuva/main.rs`** — add `mod <name>;` at the top, add variant to `Commands` enum, add arm to the `match` in `main()`.
- [ ] **`scripts/smoke_tests.sh`** — add at least one invocation using `examples/data/`.
- [ ] **`tests/cli_basic.rs`** — add at minimum a test that runs the subcommand and checks for SVG output, plus a content-verification test.
- [ ] **`examples/data/`** — add a TSV file if the existing 22 files don't cover the new plot; regenerate via `examples/data/generate.py` if needed.
- [ ] **`docs/src/cli/index.md`** — add the subcommand to the flag-reference table and example invocations section.
- [ ] **`man/kuva.1`** — regenerate: `cargo build --bin kuva && ./target/debug/kuva man > man/kuva.1`.

### Documentation

- [ ] **`examples/<name>.rs`** — a self-contained Rust example that generates several representative SVG variants; this is what `gen_docs.sh` calls to produce docs assets.
- [ ] **`scripts/gen_docs.sh`** — add invocations for the new example to generate all `docs/src/assets/<name>/*.svg` files.
- [ ] **Run `bash scripts/gen_docs.sh`** — confirm all assets generate without errors.
- [ ] **`docs/src/plots/<name>.md`** — documentation page: one-line description, import path, builder method table, embedded SVG examples with code snippets.
- [ ] **`docs/src/SUMMARY.md`** — add link to the new plot page under `# Plot Types`.
- [ ] **`docs/src/gallery.md`** — add a gallery card using the most visually rich asset.
- [ ] **`README.md`** — add a row to the plot types table.

### Visual inspection

- [ ] Run `cargo test --features cli,full` and open `test_outputs/` — visually inspect the new plot's SVGs and scan neighbouring plots for unexpected layout regressions (margins, label clipping, legend overlap).
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

## Cutting a release

**This should only be done by authorised repo maintainers**

Nothing below is checked by CI — each step is easy to miss because the crate still builds and tests still pass without it.

- [ ] **`Cargo.toml`** — bump `version`.
- [ ] **`Cargo.lock`** — run `cargo build` once; it picks up the new version automatically.
- [ ] **`CHANGELOG.md`** — rename `## [Unreleased]` to `## [X.Y.Z] — YYYY-MM-DD`; add the new compare link at the bottom (`[X.Y.Z]: .../compare/vPREV...vX.Y.Z`) and repoint the `[Unreleased]` link to `compare/vX.Y.Z...HEAD`.
- [ ] **`man/kuva.1`** — regenerate: `cargo build --bin kuva && ./target/debug/kuva man > man/kuva.1`. The version string is baked in at build time, so it silently goes stale on every release otherwise.
- [ ] **`README.md`** — bump the `kuva = "X.Y"` version in the install/dependency section (currently 4 occurrences: base + `png`/`pdf`/`full` feature examples).
- [ ] **`rust-toolchain.toml`** — confirm its pinned `channel` is still >= `[package.metadata.msrv] pdf_feature` in `Cargo.toml`. The release workflow's pre-built binaries (`features: cli,full,typst-math`) build with whatever toolchain this file resolves to — if it ever drops below the `pdf` feature's real MSRV (e.g. a future krilla bump), the release binaries silently stop including working PDF support even though this file isn't checked by CI.
- [ ] Run `cargo ci-fmt && cargo ci-clippy && cargo test --features cli,full` — confirm clean before tagging.
- [ ] Merge `dev` → `main`, then tag `vX.Y.Z` on `main` and push the tag — this triggers `.github/workflows/release.yml`, which builds and attaches cross-platform CLI binaries to the GitHub Release.
- [ ] `cargo publish` once the tag is pushed.

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
cargo build                                        # library
cargo build --bin kuva --features cli              # CLI binary SVG output
cargo build --bin kuva --features cli,png          # CLI + SVG + PNG output
cargo build --bin kuva --features cli,pdf          # CLI + SVG + PDF output
cargo build --bin kuva --features cli,full         # CLI + SVG + PNG + PDF output
cargo build --bin kuva --features cli,full,typst-math  # + typeset $...$ math (matches release binaries)
cargo test --features cli,full                     # all tests
cargo test --features cli,full <test_name>         # single test
cargo test --test cli_basic --features cli,full    # CLI integration tests
cargo ci-fmt                                       # format check (must be clean before opening a PR)
bash scripts/smoke_tests.sh                        # CLI smoke tests (all 22+ subcommands)
bash scripts/gen_docs.sh                           # regenerate docs SVG assets
bash scripts/gen_terminal_docs.sh                  # regenerate terminal output GIFs for docs
cargo build --bin kuva --features cli && ./target/debug/kuva man > man/kuva.1  # regenerate man page
cargo ci-test                                      # what CI runs (cli,full,emit_code)
cargo ci-test-typst                                # the opt-in typst tiers (typst-math + typst)
cargo bench --features full              # run all benchmarks (release build)
```

> **Note:** re-run benchmarks whenever you change `src/render/render.rs` or `src/render/render_utils.rs` to catch performance regressions before they are merged.

> **Note on `target/` size:** kuva builds ~200 separately linked test and example binaries, and Cargo keeps an independent artifact set for every feature combination without ever collecting the old ones, so `target/` grows fast. `[profile.dev]` in `Cargo.toml` therefore sets `debug = "line-tables-only"` and strips debuginfo from dependencies entirely. Backtraces still carry file and line numbers; what you lose is variable inspection in a debugger. For a one-off full-symbol build, use `cargo test --config 'profile.dev.debug=true'` rather than editing the profile. `cargo sweep --time 7` is worth running periodically to drop artifacts from feature combinations you have not built in a while.

---

## Setting up VHS (terminal recording)

Terminal output GIFs in the docs (`docs/src/assets/terminal/`) are generated with [VHS](https://github.com/charmbracelet/vhs) by Charm. You only need this if you are working on the terminal backend or adding new terminal doc examples.

### 1. Install VHS

Download the latest release binary for your platform from:

**https://github.com/charmbracelet/vhs/releases**

On Linux (x86_64):

```bash
# Download and extract
curl -L https://github.com/charmbracelet/vhs/releases/latest/download/vhs_Linux_x86_64.tar.gz \
    | tar -xz vhs
mv vhs ~/.local/bin/
chmod +x ~/.local/bin/vhs
```

Verify: `vhs --version`

### 2. Install VHS runtime dependencies

VHS requires `ttyd` and `ffmpeg` to be on `$PATH`.

**ffmpeg** — available in most package managers:
```bash
sudo apt install ffmpeg        # Debian/Ubuntu
sudo dnf install ffmpeg        # Fedora
brew install ffmpeg            # macOS
```

**ttyd** — grab the static binary from:

**https://github.com/tsl0922/ttyd/releases**

```bash
# Linux x86_64 example
curl -L https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.x86_64 \
    -o ~/.local/bin/ttyd
chmod +x ~/.local/bin/ttyd
```

Verify both are on `$PATH`: `which ttyd && which ffmpeg`

### 3. Build the release binary

VHS tapes invoke the release build for consistent timing:

```bash
cargo build --release --bin kuva --features cli
```

### 4. Regenerate terminal docs

```bash
bash scripts/gen_terminal_docs.sh
```

This runs all tapes in `docs/tapes/` and writes GIFs to `docs/src/assets/terminal/`. Commit the updated GIFs alongside any tape or terminal backend changes.

### 5. Writing a new tape

Tapes live in `docs/tapes/<subcommand>.tape`. A minimal example:

```
Output docs/src/assets/terminal/scatter.gif
Set Width 110
Set Height 35
Set FontSize 14
Set Theme "Dracula"
Set PlaybackSpeed 0.5

Type "kuva scatter examples/data/scatter.tsv --x x --y y --color-by group --terminal"
Enter
Sleep 3s
```

Key settings:
- `Output` — path to the generated GIF (relative to repo root)
- `Width` / `Height` — terminal dimensions in columns/rows; should match what the tape's command uses (no explicit `--term-width`/`--term-height` needed — VHS sets the tty size)
- `PlaybackSpeed` — values below 1.0 slow the playback, making output easier to read
- `Sleep` after `Enter` — give the command time to finish before the recording ends

See the [VHS documentation](https://github.com/charmbracelet/vhs#vhs-command-reference) for the full tape syntax.
