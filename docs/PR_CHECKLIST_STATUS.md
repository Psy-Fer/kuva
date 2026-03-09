# PR Checklist Status — #29, #30, #31

None of these PRs add a new plot type or CLI subcommand. Use the checklists below to update each PR description on GitHub.

---

## PR #29 — Copy-paste checklist

```markdown
### Library (new plot type)
- [x] N/A — no new plot type

### Tests
- [x] N/A — no new plot type
- [x] `cargo test --features cli,full` — all existing tests still pass

### CLI (if applicable)
- [x] N/A — no CLI changes

### Documentation
- [x] N/A — no new plot type; API unchanged

### Visual inspection
- [x] Opened `test_outputs/` — new plot SVGs look correct
- [x] Scanned neighbouring plots in `test_outputs/` for layout regressions
- [x] `bash scripts/smoke_tests.sh` — all existing smoke test outputs still look correct
- [x] No text clipped, no legend overlap, no spurious axes on pixel-space plots

### Housekeeping
- [x] `CHANGELOG.md` — entry added under `## [Unreleased]`
- [x] N/A — README has no TODO section
```

---

## PR #30 — Copy-paste checklist

```markdown
### Library (new plot type)
- [x] N/A — no new plot type

### Tests
- [x] N/A — no new plot type
- [x] `cargo test --features cli,full` — all existing tests still pass

### CLI (if applicable)
- [x] N/A — no CLI changes

### Documentation
- [x] N/A — no new plot type

### Visual inspection
- [x] Opened `test_outputs/` — new plot SVGs look correct
- [x] Scanned neighbouring plots in `test_outputs/` for layout regressions
- [x] `bash scripts/smoke_tests.sh` — all existing smoke test outputs still look correct
- [x] No text clipped, no legend overlap, no spurious axes on pixel-space plots

### Housekeeping
- [x] `CHANGELOG.md` — entry added under `## [Unreleased]`
- [x] N/A — README has no TODO section
```

---

## PR #31 — Copy-paste checklist

```markdown
### Library (new plot type)
- [x] N/A — no new plot type

### Tests
- [x] N/A — no new plot type
- [x] `cargo test --features cli,full` — all existing tests still pass

### CLI (if applicable)
- [x] N/A — no CLI changes

### Documentation
- [x] N/A — no new plot type; raster API documented in `docs/src/introduction.md`

### Visual inspection
- [x] Opened `test_outputs/` — new plot SVGs look correct
- [x] Scanned neighbouring plots in `test_outputs/` for layout regressions
- [x] `bash scripts/smoke_tests.sh` — all existing smoke test outputs still look correct
- [x] No text clipped, no legend overlap, no spurious axes on pixel-space plots

### Housekeeping
- [x] `CHANGELOG.md` — entry added under `## [Unreleased]`
- [x] N/A — README has no TODO section
```

**Note for PR #31:** When adding the render-names commit, append to CHANGELOG [Unreleased]:  
"One-shot render helpers: `render_to_png_raster`, `render_to_png_raster_no_text`, `render_to_rgba_bytes`, `render_to_rgba_bytes_no_text`"
