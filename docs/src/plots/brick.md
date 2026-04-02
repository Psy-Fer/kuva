# Brick Plot

A brick plot displays sequences as rows of colored rectangles — one brick per character. It is designed for bioinformatics workflows involving **DNA/RNA sequence visualization** and **tandem-repeat structure analysis**. Each character maps to a color defined by a template; rows are labeled on the y-axis.

**Import paths:** `kuva::plot::BrickPlot`, `kuva::plot::brick::BrickTemplate`

---

## DNA sequences

`BrickTemplate::dna()` provides a standard A / C / G / T color scheme. Pass the `.template` field to `BrickPlot::with_template()`. Use `with_x_offset(n)` to skip a common flanking region so the region of interest starts at x = 0.

```rust,no_run
use std::collections::HashMap;
use kuva::plot::BrickPlot;
use kuva::plot::brick::BrickTemplate;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

let tmpl = BrickTemplate::new().dna();

let plot = BrickPlot::new()
    .with_sequences(vec![
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCAT",
        "CGGCGATCAGGCCGCACTCATCATCATCATCATCATCATCAT",
    ])
    .with_names(vec!["read_1", "read_2"])
    .with_template(tmpl.template)
    .with_x_offset(18.0);  // skip 18-base common prefix

let plots = vec![Plot::Brick(plot)];
let layout = Layout::auto_from_plots(&plots)
    .with_title("DNA Repeat Region");

let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
std::fs::write("brick.svg", svg).unwrap();
```

<img src="../assets/brick/dna.svg" alt="Brick plot — DNA sequences with x-offset alignment" width="700">

The 18-character flanking prefix is hidden by `with_x_offset(18.0)`. All rows start at the same x = 0, aligning the CAT repeat region across reads.

---

## Per-row offsets

When reads begin at different positions, `with_x_offsets` accepts one offset per row. Pass `None` for any row that should fall back to the global `x_offset`.

```rust,no_run
# use kuva::plot::BrickPlot;
# use kuva::plot::brick::BrickTemplate;
# use kuva::render::plots::Plot;
let plot = BrickPlot::new()
    .with_sequences(sequences)
    .with_names(names)
    .with_template(BrickTemplate::new().dna().template)
    .with_x_offset(12.0)                       // global fallback
    .with_x_offsets(vec![
        Some(18.0_f64),  // read 1: skip 18
        Some(10.0),      // read 2: skip 10
        Some(16.0),      // read 3: skip 16
        Some(5.0),       // read 4: skip 5
        None,            // read 5: use global (12)
    ]);
```

<img src="../assets/brick/per_row_offsets.svg" alt="Brick plot — per-row x offsets" width="700">

Each row is shifted independently, aligning the repeat boundary across reads with different flanking lengths.

---

## Custom template with value overlay

`with_template` accepts any `HashMap<char, String>`. Here a protein secondary-structure alphabet (H, E, C, T) gets custom colors. `with_values()` prints the character label inside each brick.

```rust,no_run
use std::collections::HashMap;
# use kuva::plot::BrickPlot;
# use kuva::render::plots::Plot;

let mut tmpl: HashMap<char, String> = HashMap::new();
tmpl.insert('H', "steelblue".into());   // α-helix
tmpl.insert('E', "firebrick".into());   // β-strand
tmpl.insert('C', "#aaaaaa".into());     // coil
tmpl.insert('T', "seagreen".into());    // turn

let plot = BrickPlot::new()
    .with_sequences(vec!["CCCHHHHHHHHHHCCCCEEEEEECCC"])
    .with_names(vec!["prot_1"])
    .with_template(tmpl)
    .with_values();  // show letter labels inside bricks
```

<img src="../assets/brick/custom.svg" alt="Brick plot — custom secondary-structure alphabet with value overlay" width="700">

Any single-character alphabet can be used — amino acids, repeat unit categories, chromatin states, etc.

---

## Strigar mode (tandem-repeat motifs)

`with_strigars` switches to **strigar mode** for structured tandem-repeat data produced by [BLADERUNNER](https://github.com/Psy-Fer/bladerunner). Each read is described by two strings:

- **motif string** — maps local letters to k-mers: `"CAT:A,C:B,T:C"`
- **strigar string** — run-length encoding of those letters: `"10A1B4A1C1A"`

`with_strigars` normalises k-mers across all reads by canonical rotation, assigns global letters (A, B, C, …) by frequency, auto-generates colors, and renders **variable-width** bricks proportional to each motif's nucleotide length.

```rust,no_run
# use kuva::plot::BrickPlot;
# use kuva::render::plots::Plot;
let strigars: Vec<(String, String)> = vec![
    ("CAT:A,C:B,T:C".to_string(),   "10A1B4A1C1A".to_string()),
    ("CAT:A,T:B".to_string(),        "14A1B1A".to_string()),
    ("CAT:A,C:B,GGT:C".to_string(), "10A1B8A1C5A".to_string()),
    // ...
];

let plot = BrickPlot::new()
    .with_names(names)
    .with_strigars(strigars);  // sequences not needed — derived from strigars
```

<img src="../assets/brick/strigar.svg" alt="Brick plot — strigar mode showing CAT tandem repeats with interruptions" width="700">

Bricks are proportional to motif length (CAT = 3 bp wide; single-nucleotide interruptions are narrower). The dominant repeat unit (CAT) is assigned letter A and the first color; rarer motifs receive subsequent letters and colors.

---

## Bladerunner stitched format

Bladerunner's stitched output joins multiple STR candidates with `|` separators. Each `|`-delimited section is its own candidate with its own local letter assignments; `with_strigars` handles this automatically.

### Inter-candidate gaps

Gaps between candidates appear as `N@` in the strigar (where N is the gap width in nucleotides) and render as light grey bricks:

| Gap type | Motif string entry | Strigar token | Rendered width |
|----------|-------------------|---------------|----------------|
| **Large gap** (above threshold) | *(none)* | `N@` | N nt |
| **Small gap** (below threshold) | `@:SEQUENCE` | `1@` | `len(seq)` nt |

```rust,no_run
# use kuva::plot::BrickPlot;
# use kuva::render::plots::Plot;
// Three stitched candidates; two large gaps between them.
// ACCCTA, TAACCC, CCCTAA are all rotations of the same unit —
// with_strigars assigns them the same global letter and colour.
let strigars: Vec<(String, String)> = vec![
    (
        "ACCCTA:A | ACCCTA:A | TAACCC:A,T:B | CCCTAA:A,ACCTAACCCTTAA:B".to_string(),
        "2A | 36@ | 2A | 213@ | 2A1B3A | 31@ | 2A1B2A".to_string(),
    ),
];

let plot = BrickPlot::new()
    .with_names(vec!["read_1"])
    .with_strigars(strigars);
```

### Aligning reads by genomic position

Use `with_start_positions` to pass the reference coordinate where each read begins. Reads are shifted on the shared x-axis so repeat regions line up visually. Combine with `with_x_origin` to anchor a biologically meaningful position (e.g. the repeat start) to x = 0.

```rust,no_run
# use kuva::plot::BrickPlot;
# use kuva::render::plots::Plot;
// read_1: A-repeat (16 nt) + small gap GAA (3 nt) + AGA-repeat.
//         AGA region starts at reference position 19.
// read_2: AGA-repeat only, starting at reference position 19.
//
// with_start_positions aligns both reads on the shared reference axis.
// with_x_origin(19) places x=0 at the repeat start; the pre-repeat
// flanking region of read_1 appears at negative x values.
let strigars: Vec<(String, String)> = vec![
    ("A:A | @:GAA | AGA:B".to_string(), "16A | 1@ | 9B".to_string()),
    ("AGA:A".to_string(),               "12A".to_string()),
];

let plot = BrickPlot::new()
    .with_names(vec!["read_1", "read_2"])
    .with_strigars(strigars)
    .with_start_positions(vec![0.0_f64, 19.0])  // genomic start coord per read
    .with_x_origin(19.0);                        // x=0 at the repeat start
```

`with_start_positions` is equivalent to `with_x_offsets` with negated values but expresses intent clearly: pass the actual reference start coordinate for each read and kuva handles the shift. `with_x_origin` is a separate, independent axis shift applied on top — it does not interact with per-row offsets and can be used with or without `with_start_positions`.

---

## Built-in templates

| Method | Alphabet | Colors |
|--------|----------|--------|
| `BrickTemplate::new().dna()` | A C G T | green / blue / orange / red |
| `BrickTemplate::new().rna()` | A C G U | green / blue / orange / red |

Access the populated map via `.template` and pass it to `with_template()`.

---

## API reference

| Method | Description |
|--------|-------------|
| `BrickPlot::new()` | Create with defaults |
| `.with_sequences(iter)` | Load character sequences (one string per row) |
| `.with_names(iter)` | Load row labels (one per sequence) |
| `.with_template(map)` | Set `HashMap<char, CSS color>` |
| `.with_x_offset(f)` | Global x-offset applied to all rows (shift left by f nt) |
| `.with_x_offsets(iter)` | Per-row offsets (`f64` or `Option<f64>`; `None` → global fallback) |
| `.with_start_positions(iter)` | Per-row genomic start coordinates; shifts each read so it begins at that x position |
| `.with_x_origin(f)` | Reference coordinate mapped to x = 0; applied on top of all per-row offsets |
| `.with_values()` | Draw character labels inside bricks |
| `.with_strigars(iter)` | Load strigar data and switch to strigar mode; accepts bladerunner stitched format |
| `BrickTemplate::new().dna()` | Pre-built DNA (A/C/G/T) color template |
| `BrickTemplate::new().rna()` | Pre-built RNA (A/C/G/U) color template |
