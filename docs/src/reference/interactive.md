# SVG Interactivity

kuva can embed browser interactivity directly into SVG output — no server, no external dependencies, no JavaScript CDN. Everything is self-contained in the `.svg` file.

## Try it

The volcano plot below is fully interactive. Click inside it and try:

- **Type `BRCA1`** in the search box to find it instantly
- Try `TP53`, `MYC`, `KRAS`, `EGFR`, or any gene name
- **Hover** any point to see the gene name, fold change, and p-value
- **Click** a point to pin it; **Escape** to clear
- Click **Up** or **Down** in the legend to toggle those series

<iframe src="../assets/interactive_volcano.svg" width="720" height="540" style="border:none;display:block;margin:0 auto;"></iframe>

*Generated with: `kuva volcano data.tsv --name-col gene --x-col log2fc --y-col pvalue --legend --interactive --top-n 15 -o volcano.svg`*

---

## Enabling it

**Library:**
```rust
let layout = Layout::auto_from_plots(&plots)
    .with_interactive();
```

**CLI:**
```bash
kuva scatter data.tsv --x x --y y --color-by group --legend --interactive -o plot.svg
```

The flag is accepted by every subcommand. Open the output file in any modern browser (Chrome, Firefox, Safari, Edge).

## Features

| Feature | How to use |
|---------|-----------|
| **Hover tooltip** | Move the cursor over any data point to see its label and value |
| **Click to pin** | Click a point to keep it highlighted; click again or press **Escape** to clear |
| **Search** | Type in the search box (top-left of the plot area) to dim non-matching points; **Escape** clears |
| **Coordinate readout** | While the cursor is inside the plot area, the current x/y in data space is shown near the cursor |
| **Legend toggle** | Click a legend entry to hide that series; click again to show it |
| **Save SVG** | The Save button (top-right) captures the current DOM state. *Download is not yet functional — will be fixed in v0.2.* |

## Plot support

Interactivity is fully wired (hover, search, legend toggle) for:

- `scatter`
- `line`
- `bar`
- `strip`
- `volcano`

All other subcommands accept `--interactive` and render the coordinate readout and search UI, but individual data points do not yet respond to hover or search. Full renderer coverage is planned for v0.2.

## Non-SVG contexts

`--interactive` is silently ignored when:
- Output is PNG (`--features png`) or PDF (`--features pdf`)
- Output is the terminal (`--terminal`)
- The SVG is opened in Inkscape or Illustrator (script tags are stripped)

Non-interactive plots are byte-identical to today — the flag is purely additive.
