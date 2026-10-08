use crate::plot::annotations::ReferenceLine;
use std::collections::HashMap;

/// Controls horizontal alignment of brick rows.
///
/// Used with [`BrickPlot::with_anchor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrickAnchor {
    /// Rows are left-aligned (default): each row starts at its offset.
    #[default]
    Left,
    /// Rows are right-aligned: trailing edges of all rows line up on the right.
    Right,
}

/// A CIGAR operation, as drawn in the per-row CIGAR bar.
///
/// Only the operations that mean something in read space are modelled.
/// `H` (hard clip) consumes neither read nor reference and is ignored;
/// `P` (padding) likewise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CigarOp {
    /// `M` or `=` — aligned to the reference.
    Match,
    /// `X` — aligned but mismatching.
    Mismatch,
    /// `I` — present in the read, absent from the reference.
    Insertion,
    /// `D` or `N` — present in the reference, absent from the read. Consumes
    /// no read bases, so it is drawn as a zero-width caret rather than a span.
    Deletion,
    /// `S` — soft-clipped: present in the read, not aligned.
    SoftClip,
}

impl CigarOp {
    /// Parse a CIGAR operation character. `None` for `H`/`P` (no read or
    /// reference consumption) and anything unrecognised.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'M' | '=' => Some(CigarOp::Match),
            'X' => Some(CigarOp::Mismatch),
            'I' => Some(CigarOp::Insertion),
            'D' | 'N' => Some(CigarOp::Deletion),
            'S' => Some(CigarOp::SoftClip),
            _ => None,
        }
    }

    /// Whether this operation consumes read bases (and so has width in the bar).
    pub fn consumes_read(self) -> bool {
        !matches!(self, CigarOp::Deletion)
    }

    /// Default fill colour. Deliberately outside the motif palette's hue
    /// range so an op can never be mistaken for a motif brick beneath it.
    pub fn default_color(self) -> &'static str {
        match self {
            CigarOp::Match => "#cfcfcf",
            CigarOp::Mismatch => "#9a9a9a",
            CigarOp::Insertion => "#f2c53d",
            CigarOp::Deletion => "#c2352d",
            CigarOp::SoftClip => "#46566b",
        }
    }

    /// Legend label.
    pub fn label(self) -> &'static str {
        match self {
            CigarOp::Match => "match",
            CigarOp::Mismatch => "mismatch",
            CigarOp::Insertion => "insertion",
            CigarOp::Deletion => "deletion",
            CigarOp::SoftClip => "softclip",
        }
    }
}

/// Row ordering for a [`BrickPlot`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BrickSort {
    /// Keep rows in the order supplied. Default.
    #[default]
    None,
    /// Group rows sharing a CIGAR, largest group first. This is the order
    /// that lets CIGAR-bar dedup draw the fewest possible bars (one per
    /// distinct CIGAR); any other order can split a CIGAR across several
    /// runs and repeat its bar.
    Cigar,
    /// A caller-supplied permutation of row indices. Use this to hand kuva an
    /// externally computed ranking (e.g. bladerunner's distance-from-consensus)
    /// without kuva needing to know how it was derived. Indices that are out
    /// of range or repeated are ignored, and any rows the permutation omits
    /// keep their original relative order at the end.
    Custom(Vec<usize>),
}

/// One parsed CIGAR span, positioned in read space.
///
/// `start`/`len` are in read bases. A [`CigarOp::Deletion`] has `len == 0`
/// here (the deleted length is kept in `ref_len`) because it consumes no read
/// bases: it is a point event between two read positions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CigarSpan {
    pub op: CigarOp,
    /// Offset in read bases from the start of the row's sequence.
    pub start: f64,
    /// Width in read bases. Zero for deletions.
    pub len: f64,
    /// Reference bases consumed. Non-zero for deletions; used for tooltips.
    pub ref_len: f64,
}

/// Why a row's CIGAR could not be drawn.
///
/// A bar whose length disagrees with its bricks misaligns every operation by
/// the difference while still looking plausible, so these are surfaced rather
/// than rendered.
#[derive(Debug, Clone, PartialEq)]
pub enum CigarIssue {
    /// The CIGAR string could not be parsed.
    Unparseable { row: usize, cigar: String },
    /// The CIGAR's read-consuming length disagrees with the row it annotates.
    ///
    /// Almost always means the CIGAR was clipped to a different span than the
    /// one the row draws. It must cover **every base the row draws, flanks
    /// included**: left flank + STRIGAR + right flank, which is what
    /// [`BrickPlot::row_base_len`] returns. A CIGAR covering only the STRIGAR
    /// section of a flanked row lands here.
    ///
    /// Do not re-derive the CIGAR from locus coordinates: whatever produced
    /// the row is the only thing that knows where its own span starts and
    /// ends.
    LengthMismatch {
        row: usize,
        cigar_read_bases: f64,
        row_bases: f64,
    },
}

/// Parse a CIGAR string into read-space spans.
///
/// Returns `None` if the string is malformed. An empty string parses to an
/// empty span list, which is treated as "no bar for this row".
pub fn parse_cigar(cigar: &str) -> Option<Vec<CigarSpan>> {
    let mut spans = Vec::new();
    let mut read_pos = 0.0_f64;
    let mut digits = String::new();
    for ch in cigar.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        if digits.is_empty() {
            return None; // operation with no preceding length
        }
        let len: f64 = digits.parse().ok()?;
        digits.clear();
        match CigarOp::from_char(ch) {
            Some(op) if op.consumes_read() => {
                spans.push(CigarSpan {
                    op,
                    start: read_pos,
                    len,
                    ref_len: if op == CigarOp::Insertion || op == CigarOp::SoftClip {
                        0.0
                    } else {
                        len
                    },
                });
                read_pos += len;
            }
            Some(op) => {
                // Deletion: a point event at the current read position.
                spans.push(CigarSpan {
                    op,
                    start: read_pos,
                    len: 0.0,
                    ref_len: len,
                });
            }
            None => {} // H / P / unknown: consumes nothing in read space
        }
    }
    if !digits.is_empty() {
        return None; // trailing length with no operation
    }
    Some(spans)
}

/// Total read bases consumed by a parsed CIGAR.
pub fn cigar_read_bases(spans: &[CigarSpan]) -> f64 {
    spans.iter().map(|s| s.len).sum()
}

/// Height of a CIGAR bar (plus its gap) as a fraction of one brick row.
///
/// Charged only to rows that draw a bar, so a run of identical reads keeps
/// the normal pitch and space appears only where the CIGAR changes.
pub(crate) const BAR_ROW_UNITS: f64 = 0.45;

/// Resolved CIGAR bar placement for a [`BrickPlot`], in display order.
///
/// Produced by `BrickPlot::cigar_layout` and consumed by both `Plot::bounds`
/// and the renderer, so the reserved y extent always matches what is drawn.
#[derive(Debug, Clone)]
pub(crate) struct CigarLayout {
    /// Source row index for each display position.
    pub order: Vec<usize>,
    /// Parsed spans per display position; `None` when there is no usable CIGAR.
    pub spans: Vec<Option<Vec<CigarSpan>>>,
    /// Whether this display position draws a bar (false for deduped rows).
    pub draws_bar: Vec<bool>,
    /// Index into `runs` for each display position.
    pub run_of_row: Vec<usize>,
    /// `(first_pos, last_pos)` of each run of rows sharing a CIGAR.
    pub runs: Vec<(usize, usize)>,
    /// Top of each row in y-units, measured from the top of the plot.
    pub row_top_units: Vec<f64>,
    /// Total height in y-units, including every bar allowance.
    pub total_units: f64,
    /// Rows whose CIGAR was rejected.
    pub issues: Vec<CigarIssue>,
}

/// Allows `with_cigars` to accept plain `&str`/`String` values (auto-wrapped
/// as `Some`) as well as explicit `Option` values for rows that have no CIGAR.
///
/// Without this, `with_cigars(["108M", "42M3D63M"])` would not compile, since
/// `&str` does not implement `Into<Option<String>>`.
pub trait IntoOptionalCigar {
    fn into_optional_cigar(self) -> Option<String>;
}

impl IntoOptionalCigar for &str {
    fn into_optional_cigar(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoOptionalCigar for String {
    fn into_optional_cigar(self) -> Option<String> {
        Some(self)
    }
}

impl IntoOptionalCigar for Option<&str> {
    fn into_optional_cigar(self) -> Option<String> {
        self.map(|s| s.to_string())
    }
}

impl IntoOptionalCigar for Option<String> {
    fn into_optional_cigar(self) -> Option<String> {
        self
    }
}

/// Allows `with_x_offsets` to accept plain `f64` values (auto-wrapped as `Some`)
/// as well as explicit `Option<f64>` values (for `None` fallback entries).
pub trait IntoRowOffset {
    fn into_row_offset(self) -> Option<f64>;
}

impl IntoRowOffset for f64 {
    fn into_row_offset(self) -> Option<f64> {
        Some(self)
    }
}

impl IntoRowOffset for Option<f64> {
    fn into_row_offset(self) -> Option<f64> {
        self
    }
}

fn canonical_rotation(s: &str) -> String {
    let n = s.len();
    if n == 0 {
        return String::new();
    }
    let doubled = format!("{}{}", s, s);
    (0..n)
        .map(|i| &doubled[i..i + n])
        .min()
        .expect("range 0..n is non-empty when n > 0")
        .to_string()
}

/// Parse a STRIGAR motif map (`"kmer:letter,kmer:letter,..."`) into a
/// local-letter -> kmer map for one row.
///
/// Letters are bijective base-26 (`A..Z, AA, AB, ...`), so the whole field after
/// the `:` is the letter. Letters are local to the row. Malformed or empty pairs
/// are skipped rather than erroring. See the bladerunner STRIGAR contract.
fn parse_motif_map(s: &str) -> HashMap<String, String> {
    s.split(',')
        .filter_map(|pair| {
            let mut parts = pair.trim().splitn(2, ':');
            let kmer = parts.next()?.trim();
            let letter = parts.next()?.trim();
            if kmer.is_empty() || letter.is_empty() {
                return None;
            }
            Some((letter.to_string(), kmer.to_string()))
        })
        .collect()
}

/// Tokenise a STRIGAR string (`"<count><letter>..."`) into `(count, letter)` runs.
///
/// `count` is one or more digits (copies of the letter's kmer); `letter` is the
/// maximal run of one or more uppercase ASCII characters (bijective base-26).
/// Reading a single char would truncate multi-character letters (`3AA` -> `3A`
/// plus a stray `A`), so the maximal uppercase run is consumed. Malformed
/// fragments are dropped and the scan always makes progress, so a bad row
/// degrades instead of panicking.
fn parse_strigar_runs(s: &str) -> Vec<(usize, String)> {
    let mut runs = Vec::new();
    let mut chars = s.chars().peekable();
    while chars.peek().is_some() {
        let mut num = String::new();
        while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
            num.push(chars.next().expect("peeked digit"));
        }
        let mut letter = String::new();
        while chars.peek().is_some_and(|c| c.is_ascii_uppercase()) {
            letter.push(chars.next().expect("peeked uppercase"));
        }
        if num.is_empty() && letter.is_empty() {
            // Neither a count nor a letter at the cursor (e.g. a leftover `@` or
            // lowercase char): consume one char to guarantee progress.
            chars.next();
        } else if let Ok(count) = num.parse::<usize>() {
            if !letter.is_empty() {
                runs.push((count, letter));
            }
            // else: count with no letter -> drop this partial run.
        }
    }
    runs
}

/// Pre-built character-to-color mappings for common biological alphabets.
///
/// Call a constructor method to populate the [`template`](BrickTemplate::template)
/// `HashMap`, then pass it to
/// [`BrickPlot::with_template`](BrickPlot::with_template).
///
/// # Available templates
///
/// | Method | Alphabet | Colors |
/// |--------|----------|--------|
/// | `.dna()` | A C G T | green / blue / orange / red |
/// | `.rna()` | A C G U | green / blue / orange / red |
///
/// # Example
///
/// ```rust,no_run
/// use kuva::plot::brick::BrickTemplate;
/// use kuva::plot::BrickPlot;
///
/// let tmpl = BrickTemplate::new().dna();
/// let plot = BrickPlot::new()
///     .with_sequences(vec!["ACGTACGT"])
///     .with_names(vec!["seq_1"])
///     .with_template(tmpl.template);
/// ```
#[derive(Debug, Clone)]
pub struct BrickTemplate {
    /// Map from character to CSS color string.
    pub template: HashMap<char, String>,
}

impl Default for BrickTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl BrickTemplate {
    /// Create an empty template. Call `.dna()` or `.rna()` to populate it.
    pub fn new() -> Self {
        Self {
            template: HashMap::new(),
        }
    }

    /// Populate with standard DNA colors: A → green, C → blue, G → orange, T → red.
    pub fn dna(mut self) -> Self {
        self.template.insert('A', "rgb(0,150,0)".into());
        self.template.insert('C', "rgb(0,0,255)".into());
        self.template.insert('G', "rgb(209,113,5)".into());
        self.template.insert('T', "rgb(255,0,0)".into());

        self
    }

    /// Populate with standard RNA colors: A → green, C → blue, G → orange, U → red.
    pub fn rna(mut self) -> Self {
        self.template.insert('A', "green".into());
        self.template.insert('C', "blue".into());
        self.template.insert('G', "orange".into());
        self.template.insert('U', "red".into());

        self
    }
}

/// Builder for a brick plot — a row-per-sequence visualization where each
/// character maps to a colored rectangle.
///
/// Brick plots are used in bioinformatics to display **DNA/RNA sequences**,
/// **tandem repeat structures**, and any other character-encoded per-row data.
/// Each character in a sequence is drawn as a colored brick; the color is
/// determined by a [`HashMap<char, String>`] template.
///
/// # Input modes
///
/// | Mode | How to load | Use when |
/// |------|-------------|----------|
/// | **Sequence mode** | [`with_sequences`](Self::with_sequences) + [`with_template`](Self::with_template) | Raw DNA/RNA or custom character strings |
/// | **Strigar mode** | [`with_strigars`](Self::with_strigars) | Structured tandem-repeat motif data (BLADERUNNER format) |
///
/// # Alignment
///
/// By default all rows start at x = 0. Use [`with_x_offset`](Self::with_x_offset)
/// to apply a single global offset (e.g. skip a common flanking region), or
/// [`with_x_offsets`](Self::with_x_offsets) for independent per-row alignment.
///
/// # Example
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use kuva::plot::BrickPlot;
/// use kuva::plot::brick::BrickTemplate;
/// use kuva::backend::svg::SvgBackend;
/// use kuva::render::render::render_multiple;
/// use kuva::render::layout::Layout;
/// use kuva::render::plots::Plot;
///
/// let tmpl = BrickTemplate::new().dna();
///
/// let plot = BrickPlot::new()
///     .with_sequences(vec![
///         "CGGCGATCAGGCCGCACTCATCATCATCATCAT",
///         "CGGCGATCAGGCCGCACTCATCATCATCATCATCAT",
///     ])
///     .with_names(vec!["read_1", "read_2"])
///     .with_template(tmpl.template)
///     .with_x_offset(18.0);
///
/// let plots = vec![Plot::Brick(plot)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("DNA Repeat Region");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("brick.svg", svg).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct BrickPlot {
    /// Ordered character sequences — one string per row.
    pub sequences: Vec<String>,
    /// Row labels — must match `sequences` in length.
    pub names: Vec<String>,
    /// Strigar data: `(motif_string, strigar_string)` pairs used in strigar mode.
    pub strigars: Option<Vec<(String, String)>>,
    /// Global letter → k-mer display string (set automatically in strigar mode).
    pub motifs: Option<HashMap<char, String>>,
    /// Expanded sequences derived from strigar strings (set automatically).
    pub strigar_exp: Option<Vec<String>>,
    /// Character → CSS color string. Built from [`BrickTemplate`] or supplied directly.
    pub template: Option<HashMap<char, String>>,
    /// Global x-offset applied to all rows. Default: `0.0`.
    pub x_offset: f64,
    /// Per-row offsets. `None` entries fall back to `x_offset`.
    pub x_offsets: Option<Vec<Option<f64>>>,
    /// Reference coordinate that maps to x = 0 on the axis. Default: `0.0`.
    pub x_origin: f64,
    /// Per-character nucleotide length for variable-width bricks (strigar mode).
    pub motif_lengths: Option<HashMap<char, usize>>,
    /// When `true`, draw the character label inside each brick.
    pub show_values: bool,
    /// User-supplied color palette for strigar mode. When set, overrides the
    /// built-in 20-color default. Colors are assigned in global-letter order
    /// (most-frequent motif first) and cycle if there are more motifs than colors.
    pub strigar_palette: Option<Vec<String>>,
    /// Explicit per-motif colours keyed by **canonical** k-mer (see
    /// [`with_motif_colors`](Self::with_motif_colors)). Keys are stored
    /// canonicalized. A motif listed here takes this colour in strigar mode,
    /// overriding both the DNA default and the auto-palette; motifs not listed
    /// fall back to the normal assignment. This is the stable, representation-
    /// independent way to keep a motif one colour across many plots.
    pub motif_colors: Option<HashMap<String, String>>,
    /// Horizontal alignment of rows. Default: `BrickAnchor::Left`.
    pub anchor: BrickAnchor,
    /// Per-row left flanking DNA sequences (set by [`with_flanked_strigars`](Self::with_flanked_strigars)).
    pub left_flanks: Option<Vec<String>>,
    /// Per-row right flanking DNA sequences (set by [`with_flanked_strigars`](Self::with_flanked_strigars)).
    pub right_flanks: Option<Vec<String>>,
    /// Append `*` to the legend label for the primary (most-frequent) motif (global letter A).
    pub mark_primary: bool,
    /// Row index whose motif rotations seed the global display labels.
    /// Set this **before** calling [`with_strigars`](Self::with_strigars) or
    /// [`with_flanked_strigars`](Self::with_flanked_strigars).
    pub consensus_row: Option<usize>,
    /// Pre-computed human-readable notation strings, one per row.
    /// `None` entries render nothing above that row.
    /// E.g. `Some("(CAG)12(GAA)1".to_string())`.
    pub notations: Option<Vec<Option<String>>>,
    /// Desired pixel height per brick row. When set, `auto_from_plots` computes
    /// the canvas height as `row_height_px * num_rows + margin_overhead`, and
    /// `Figure` computes per-grid-row heights so that panels with different read
    /// counts still have identically-sized bricks.
    pub row_height_px: Option<f64>,
    /// Vertical (and, if desired, horizontal) marker lines drawn over the bricks
    /// at reference-coordinate positions. See [`with_vline`](BrickPlot::with_vline).
    pub vlines: Vec<ReferenceLine>,
    /// Per-row CIGAR for the read's repeat span. `None` entries draw no bar.
    /// See [`with_cigars`](BrickPlot::with_cigars).
    pub cigars: Option<Vec<Option<String>>>,
    /// Collapse a run of consecutive rows sharing a CIGAR into a single bar.
    /// Default `true`.
    pub cigar_dedup: bool,
    /// Bracket each deduped run in the left gutter with an `xN` count.
    /// Default `true`; has no effect when `cigar_dedup` is `false`.
    pub cigar_run_bracket: bool,
    /// Per-operation colour overrides. Operations absent from the map keep
    /// [`CigarOp::default_color`].
    pub cigar_colors: Option<HashMap<CigarOp, String>>,
    /// Row ordering applied before rendering. Default [`BrickSort::None`].
    pub sort: BrickSort,
    /// Collapse runs of consecutive same-colour bricks into a single rect when the
    /// per-unit pixel width is small enough that the inter-brick gaps would be
    /// invisible anyway. Off by default (per-brick rendering). Turning it on is
    /// essential for very long sequences (e.g. RFC1/BEAN1 expansions of thousands
    /// of units across many rows), where per-unit bricks are both sub-pixel and
    /// prohibitively numerous. Above the threshold the per-brick look is preserved,
    /// so this only ever changes appearance where the gaps could not be seen.
    pub merge_runs: bool,
}

impl Default for BrickPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl BrickPlot {
    /// Create a brick plot with default settings (no data, no template, offset `0.0`).
    pub fn new() -> Self {
        Self {
            sequences: vec![],
            names: vec![],
            strigars: None,
            motifs: None,
            strigar_exp: None,
            template: Some(HashMap::new()),
            motif_lengths: None,
            x_offset: 0.0,
            x_offsets: None,
            x_origin: 0.0,
            show_values: false,
            strigar_palette: None,
            motif_colors: None,
            anchor: BrickAnchor::Left,
            left_flanks: None,
            right_flanks: None,
            mark_primary: false,
            consensus_row: None,
            notations: None,
            row_height_px: None,
            vlines: Vec::new(),
            merge_runs: false,
            cigars: None,
            cigar_dedup: true,
            cigar_run_bracket: true,
            cigar_colors: None,
            sort: BrickSort::None,
        }
    }

    /// Per-row CIGAR for the read's repeat span, in the same order as the
    /// rows. `None` entries render no bar.
    ///
    /// The CIGAR must cover **every base the row draws, flanks included**: for
    /// a row built with [`with_flanked_strigars`](Self::with_flanked_strigars)
    /// that is left flank + STRIGAR + right flank, and in sequence mode it is
    /// the sequence itself. Its read-consuming length (`M`/`I`/`S`/`=`/`X`)
    /// must equal [`row_base_len`](Self::row_base_len), which measures exactly
    /// that. The bar is drawn from the row's first drawn base, so the flanks
    /// are annotated too.
    ///
    /// Supply it from whatever produced the row rather than re-deriving it
    /// from locus coordinates: only that producer knows where its own span
    /// begins and ends, and an insertion anchored on the span boundary cannot
    /// be attributed correctly from coordinates alone. Rows that fail the
    /// check draw no bar and are reported by
    /// [`cigar_issues`](BrickPlot::cigar_issues).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new().with_cigars(["108M", "42M3D63M"]);
    /// ```
    pub fn with_cigars<I, T>(mut self, cigars: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoOptionalCigar,
    {
        self.cigars = Some(
            cigars
                .into_iter()
                .map(|c| c.into_optional_cigar())
                .collect(),
        );
        self
    }

    /// Collapse a run of consecutive rows sharing a CIGAR into a single bar
    /// (default `true`).
    ///
    /// With rows sorted so identical CIGARs are adjacent, this leaves one bar
    /// per distinct CIGAR: a block of identical reads states its CIGAR once,
    /// and a bar appears only where something changes.
    pub fn with_cigar_dedup(mut self, on: bool) -> Self {
        self.cigar_dedup = on;
        self
    }

    /// Bracket each deduped run in the left gutter with an `xN` count
    /// (default `true`), so a bar visibly belongs to the rows beneath it
    /// rather than to its own row alone.
    pub fn with_cigar_run_bracket(mut self, on: bool) -> Self {
        self.cigar_run_bracket = on;
        self
    }

    /// Override CIGAR operation colours. Operations left unset keep
    /// [`CigarOp::default_color`].
    pub fn with_cigar_colors<I, S>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = (CigarOp, S)>,
        S: Into<String>,
    {
        self.cigar_colors = Some(colors.into_iter().map(|(o, c)| (o, c.into())).collect());
        self
    }

    /// Set the row ordering. See [`BrickSort`].
    pub fn with_sort(mut self, sort: BrickSort) -> Self {
        self.sort = sort;
        self
    }

    /// Resolved fill colour for a CIGAR operation.
    pub(crate) fn cigar_color(&self, op: CigarOp) -> &str {
        self.cigar_colors
            .as_ref()
            .and_then(|m| m.get(&op))
            .map(|s| s.as_str())
            .unwrap_or_else(|| op.default_color())
    }

    /// Number of rows, from the expanded strigar rows when present.
    pub(crate) fn row_count(&self) -> usize {
        self.strigar_exp
            .as_ref()
            .map(|e| e.len())
            .unwrap_or(self.sequences.len())
    }

    /// Length of row `i` in bases, honouring per-motif widths.
    ///
    /// Motif lengths matter: a STRIGAR row can mix a 1 bp motif with an 11 bp
    /// one, so assuming a uniform width silently mis-scales the whole row.
    /// Total length of row `i` in bases: left flank + STRIGAR + right flank.
    ///
    /// This is the length a row's CIGAR must consume in read bases. Use it to
    /// check a CIGAR before handing it to [`with_cigars`](Self::with_cigars)
    /// rather than discovering the mismatch in
    /// [`cigar_issues`](Self::cigar_issues).
    ///
    /// The STRIGAR section honours per-motif widths from `motif_lengths`, so a
    /// row mixing a 1 bp motif with an 11 bp one measures correctly; in
    /// sequence mode it is simply the sequence length.
    pub fn row_base_len(&self, i: usize) -> f64 {
        self.left_flank_len(i) + self.strigar_base_len(i) + self.right_flank_len(i)
    }

    /// Length of row `i`'s STRIGAR section in bases, excluding flanks.
    pub(crate) fn strigar_base_len(&self, i: usize) -> f64 {
        let row = match self.strigar_exp.as_ref() {
            Some(exp) => exp.get(i),
            None => self.sequences.get(i),
        };
        let Some(row) = row else { return 0.0 };
        match self.motif_lengths.as_ref() {
            Some(ml) => row.chars().map(|c| *ml.get(&c).unwrap_or(&1) as f64).sum(),
            None => row.chars().count() as f64,
        }
    }

    /// Length of row `i`'s left flank in bases (0 when unset).
    pub(crate) fn left_flank_len(&self, i: usize) -> f64 {
        self.left_flanks
            .as_ref()
            .and_then(|f| f.get(i))
            .map(|s| s.chars().count() as f64)
            .unwrap_or(0.0)
    }

    /// Length of row `i`'s right flank in bases (0 when unset).
    pub(crate) fn right_flank_len(&self, i: usize) -> f64 {
        self.right_flanks
            .as_ref()
            .and_then(|f| f.get(i))
            .map(|s| s.chars().count() as f64)
            .unwrap_or(0.0)
    }

    /// The row order to render in, as a permutation of row indices.
    pub(crate) fn row_order(&self) -> Vec<usize> {
        let n = self.row_count();
        match &self.sort {
            BrickSort::None => (0..n).collect(),
            BrickSort::Cigar => {
                let key = |i: usize| -> String {
                    self.cigars
                        .as_ref()
                        .and_then(|c| c.get(i))
                        .and_then(|c| c.clone())
                        .unwrap_or_default()
                };
                let mut counts: HashMap<String, usize> = HashMap::new();
                for i in 0..n {
                    *counts.entry(key(i)).or_insert(0) += 1;
                }
                let mut order: Vec<usize> = (0..n).collect();
                // Largest group first, then by CIGAR string, then original
                // index. The string tie-break keeps this deterministic rather
                // than dependent on map iteration order.
                order.sort_by(|&a, &b| {
                    let (ka, kb) = (key(a), key(b));
                    counts[&kb]
                        .cmp(&counts[&ka])
                        .then_with(|| ka.cmp(&kb))
                        .then_with(|| a.cmp(&b))
                });
                order
            }
            BrickSort::Custom(idx) => {
                let mut seen = vec![false; n];
                let mut order = Vec::with_capacity(n);
                for &i in idx {
                    if i < n && !seen[i] {
                        seen[i] = true;
                        order.push(i);
                    }
                }
                for (i, s) in seen.iter().enumerate() {
                    if !*s {
                        order.push(i);
                    }
                }
                order
            }
        }
    }

    /// How many CIGAR bars will actually be drawn.
    ///
    /// With [`BrickSort::Cigar`] this equals the number of distinct CIGARs,
    /// which is the minimum achievable: any other row order can split one
    /// CIGAR across several runs and draw its bar more than once. Useful for
    /// reporting ("32 reads, 4 distinct CIGARs") and for deciding whether the
    /// bars are worth the vertical space at a given locus.
    pub fn cigar_bar_count(&self) -> usize {
        self.cigar_layout().draws_bar.iter().filter(|b| **b).count()
    }

    /// Rows whose CIGAR could not be used, with the reason.
    pub fn cigar_issues(&self) -> Vec<CigarIssue> {
        self.cigar_layout().issues
    }

    /// Per-row CIGAR bar placement and the resulting vertical layout.
    ///
    /// Shared by `Plot::bounds` and the renderer so the y extent and the drawn
    /// geometry cannot disagree: a bar costs vertical space, and both sides
    /// must allocate exactly the same amount.
    pub(crate) fn cigar_layout(&self) -> CigarLayout {
        let order = self.row_order();
        let n = order.len();
        let mut out = CigarLayout {
            order,
            spans: vec![None; n],
            draws_bar: vec![false; n],
            run_of_row: vec![0; n],
            runs: Vec::new(),
            row_top_units: Vec::with_capacity(n),
            total_units: n as f64,
            issues: Vec::new(),
        };
        let Some(cigars) = self.cigars.as_ref() else {
            out.row_top_units = (0..n).map(|i| i as f64).collect();
            return out;
        };

        // Parse + validate in display order.
        for (pos, &src) in out.order.iter().enumerate() {
            let Some(Some(raw)) = cigars.get(src) else {
                continue;
            };
            if raw.is_empty() {
                continue;
            }
            let Some(spans) = parse_cigar(raw) else {
                out.issues.push(CigarIssue::Unparseable {
                    row: src,
                    cigar: raw.clone(),
                });
                continue;
            };
            let read_bases = cigar_read_bases(&spans);
            let row_bases = self.row_base_len(src);
            if (read_bases - row_bases).abs() > f64::EPSILON * row_bases.max(1.0) * 8.0 {
                out.issues.push(CigarIssue::LengthMismatch {
                    row: src,
                    cigar_read_bases: read_bases,
                    row_bases,
                });
                continue;
            }
            out.spans[pos] = Some(spans);
        }

        // Runs of consecutive rows sharing a CIGAR (compared on the raw
        // string; rows whose CIGAR was rejected never start or join a run).
        let raw_at = |pos: usize| -> Option<&String> {
            let src = out.order[pos];
            out.spans[pos].as_ref()?;
            cigars.get(src).and_then(|c| c.as_ref())
        };
        let mut prev: Option<&String> = None;
        for pos in 0..n {
            let cur = raw_at(pos);
            let new_run = match (cur, prev) {
                (None, _) => true,
                (Some(c), Some(p)) => c != p,
                (Some(_), None) => true,
            };
            if new_run {
                out.runs.push((pos, pos));
            } else if let Some(last) = out.runs.last_mut() {
                last.1 = pos;
            }
            out.run_of_row[pos] = out.runs.len().saturating_sub(1);
            out.draws_bar[pos] = cur.is_some() && (!self.cigar_dedup || new_run);
            prev = cur;
        }

        // Vertical allocation: a bar costs BAR_ROW_UNITS of a row's height,
        // charged only to rows that actually draw one, so a run of identical
        // reads packs at the normal pitch.
        let mut y = 0.0_f64;
        for pos in 0..n {
            if out.draws_bar[pos] {
                y += BAR_ROW_UNITS;
            }
            out.row_top_units.push(y);
            y += 1.0;
        }
        out.total_units = y;
        out
    }

    /// Enable (or disable) run-length merging of consecutive same-colour bricks.
    /// See [`merge_runs`](BrickPlot::merge_runs). Recommended for very long sequences.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new().with_merge_runs(true);
    /// ```
    pub fn with_merge_runs(mut self, on: bool) -> Self {
        self.merge_runs = on;
        self
    }

    /// Draw a vertical marker line across all rows at reference coordinate `x`
    /// (dashed, in the default reference-line colour). Handy for marking a locus,
    /// primer boundary, or variant position.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new().with_vline(150.0);
    /// ```
    pub fn with_vline(mut self, x: f64) -> Self {
        self.vlines.push(ReferenceLine::vertical(x));
        self
    }

    /// Vertical marker line at `x` with a text label drawn at the top.
    pub fn with_vline_labeled(mut self, x: f64, label: impl Into<String>) -> Self {
        self.vlines
            .push(ReferenceLine::vertical(x).with_label(label));
        self
    }

    /// Add a fully-styled marker line (use [`ReferenceLine::vertical`] /
    /// [`ReferenceLine::horizontal`] with `.with_color()`, `.with_dasharray()`, etc.).
    pub fn with_marker_line(mut self, line: ReferenceLine) -> Self {
        self.vlines.push(line);
        self
    }

    /// Load sequences — one string per row, ordered top to bottom.
    ///
    /// Each character in a string is rendered as one brick (or as a
    /// variable-width brick in strigar mode). All characters must have an
    /// entry in the template; unknown characters will cause a panic.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new()
    ///     .with_sequences(vec!["ACGTACGT", "ACGTACGT"]);
    /// ```
    pub fn with_sequences<T, I>(mut self, sequences: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.sequences = sequences.into_iter().map(|x| x.into()).collect();

        self
    }

    /// Load row labels — one name per sequence, rendered on the y-axis.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new()
    ///     .with_sequences(vec!["ACGT"])
    ///     .with_names(vec!["read_1"]);
    /// ```
    pub fn with_names<T, I>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.names = names.into_iter().map(|x| x.into()).collect();

        self
    }

    /// Load strigar data and switch to **strigar mode**.
    ///
    /// Accepts `(motif_string, strigar_string)` pairs in
    /// [BLADERUNNER](https://github.com/Psy-Fer/bladerunner) format:
    ///
    /// - **motif string** — comma-separated `kmer:letter` assignments, e.g.
    ///   `"CAT:A,C:B,T:C"` binds the CAT trinucleotide to local letter `A`.
    /// - **strigar string** — run-length encoded local letters, e.g.
    ///   `"10A1B4A1C1A"` expands to ten `A`s, one `B`, four `A`s, etc.
    ///
    /// `with_strigars` normalises k-mers across all reads by canonical
    /// rotation, assigns global letters (A, B, C, …) ordered by frequency,
    /// auto-generates colors from a 10-color palette, and computes variable
    /// brick widths proportional to each motif's nucleotide length.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let strigars = vec![
    ///     ("CAT:A,T:B".to_string(), "14A1B1A".to_string()),
    ///     ("CAT:A,C:B".to_string(), "12A1B3A".to_string()),
    /// ];
    /// let plot = BrickPlot::new()
    ///     .with_names(vec!["read_1", "read_2"])
    ///     .with_strigars(strigars);
    /// ```
    /// Override the auto-generated motif colors used in strigar mode.
    ///
    /// Colors are assigned to global letters in order of motif frequency
    /// (most frequent motif gets the first color). If fewer colors are
    /// supplied than there are motifs, the list cycles.
    ///
    /// Call this **before** [`with_strigars`](Self::with_strigars) so the
    /// palette is available during color assignment.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new()
    ///     .with_strigar_colors(["#e41a1c", "#377eb8", "#4daf4a", "#984ea3"])
    ///     .with_strigars(vec![
    ///         ("CAT:A,C:B".to_string(), "12A1B3A".to_string()),
    ///     ]);
    /// ```
    pub fn with_strigar_colors<I, S>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.strigar_palette = Some(colors.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Assign explicit colours to specific motifs, keyed by k-mer sequence.
    ///
    /// This is the stable, representation-independent way to keep a motif the
    /// **same colour across many plots** (e.g. every locus in a cohort). Keys are
    /// matched by canonical rotation, so you may pass any rotation of a motif
    /// (`"AATGG"`, `"GGAAT"`, ...) and it resolves to the same entry; you never
    /// need to know or reproduce kuva's internal colour ordering or token space.
    ///
    /// A motif listed here takes the given colour in strigar mode, overriding both
    /// the built-in DNA colours and the auto-palette. Motifs not listed fall back
    /// to the normal assignment ([`with_strigar_colors`](Self::with_strigar_colors)
    /// or the default palette). Call this **before**
    /// [`with_strigars`](Self::with_strigars) so the colours are applied during
    /// motif-colour assignment.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// # use std::collections::HashMap;
    /// let mut colors = HashMap::new();
    /// colors.insert("AATGG".to_string(), "#e41a1c".to_string());
    /// colors.insert("CAG".to_string(), "#377eb8".to_string());
    /// let plot = BrickPlot::new()
    ///     .with_motif_colors(colors)
    ///     .with_strigars(vec![("AATGG:A,CAG:B".to_string(), "10A2B".to_string())]);
    /// ```
    pub fn with_motif_colors<K, V, I>(mut self, colors: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.motif_colors = Some(
            colors
                .into_iter()
                .map(|(k, v)| (canonical_rotation(&k.into()), v.into()))
                .collect(),
        );
        self
    }

    pub fn with_strigars<T, U, I>(mut self, strigars: I) -> Self
    where
        I: IntoIterator<Item = (T, U)>,
        T: Into<String>,
        U: Into<String>,
    {
        self.strigars = Some(
            strigars
                .into_iter()
                .map(|(motif, strigar)| (motif.into(), strigar.into()))
                .collect(),
        );

        let strigars_ref = self.strigars.as_ref().expect("strigars just set");

        // Phase B: walk every row, collecting kmers by canonical rotation into
        // brick-count frequency tables. One motif namespace per row (the format has
        // no segments): a run's `count` is copies of that letter's kmer, and we score
        // canonicals by total copies across all rows so the most-used motif ranks first.
        let mut canonical_freq: HashMap<String, usize> = HashMap::new();
        let mut rotation_freq: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for (motif_str, strigar_str) in strigars_ref {
            let local_map = parse_motif_map(motif_str);
            for (count, letter) in parse_strigar_runs(strigar_str) {
                if let Some(kmer) = local_map.get(&letter) {
                    let canon = canonical_rotation(kmer);
                    *canonical_freq.entry(canon.clone()).or_insert(0) += count;
                    *rotation_freq
                        .entry(canon)
                        .or_default()
                        .entry(kmer.clone())
                        .or_insert(0) += count;
                }
            }
        }

        // Phase B.5: if consensus_row is set, pick each canonical's display rotation from
        // that row so Phase C can lock the label to what the consensus sequence uses.
        //
        // With the flat (single-namespace) format, a consensus row can carry two rotations
        // of the same canonical under different letters (e.g. an interruption that happens
        // to be a rotation of a tract's motif). We must NOT let the winner depend on
        // `HashMap` iteration order (randomised per process), or the label flips run to run.
        // Pick the rotation with the highest copy count in the consensus row, breaking ties
        // lexicographically (larger wins) to match Phase C's fallback selection below.
        let mut consensus_rotations: HashMap<String, String> = HashMap::new();
        if let Some(cons_row) = self.consensus_row {
            if let Some((motif_str, strigar_str)) = strigars_ref.get(cons_row) {
                let local_map = parse_motif_map(motif_str);
                // Tally copies per rotation within the consensus row.
                let mut cons_rotation_freq: HashMap<String, HashMap<String, usize>> =
                    HashMap::new();
                for (count, letter) in parse_strigar_runs(strigar_str) {
                    if let Some(kmer) = local_map.get(&letter) {
                        let canon = canonical_rotation(kmer);
                        *cons_rotation_freq
                            .entry(canon)
                            .or_default()
                            .entry(kmer.clone())
                            .or_insert(0) += count;
                    }
                }
                for (canon, rotations) in &cons_rotation_freq {
                    let display = rotations
                        .iter()
                        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                        .expect("cons_rotation_freq entry is non-empty")
                        .0
                        .clone();
                    consensus_rotations.insert(canon.clone(), display);
                }
            }
        }

        // Phase C: sort canonicals by frequency desc, canonical string asc as tiebreak.
        let mut sorted_canonicals: Vec<(String, usize)> = canonical_freq.into_iter().collect();
        sorted_canonicals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Global identifiers are INTERNAL single-char tokens. They are never shown to the
        // user (the legend and notations use the display kmer via `motifs`), so their exact
        // value is irrelevant. We draw them from the Unicode Private Use Area in frequency
        // order, which (a) has no 26-symbol ceiling like `b'A' + idx` did, and (b) keeps the
        // char-packed `strigar_exp` and char-keyed template/renderer pipeline unchanged.
        // Bladerunner's LOCAL letters may be multi-character base-26; they are parsed as
        // strings above and never enter this internal token space.
        let global_token =
            |idx: usize| -> char { char::from_u32(0xE000u32 + idx as u32).unwrap_or('\u{E000}') };

        let mut canonical_to_global: HashMap<String, char> = HashMap::new();
        let mut global_to_display: HashMap<char, String> = HashMap::new();
        let mut global_to_length: HashMap<char, usize> = HashMap::new();

        for (idx, (canon, _freq)) in sorted_canonicals.iter().enumerate() {
            let global = global_token(idx);
            canonical_to_global.insert(canon.clone(), global);

            // Pick the display rotation: consensus row's rotation takes priority;
            // fall back to most-frequent rotation (tiebreak: prefer lexicographically larger).
            let rotations = rotation_freq
                .get(canon)
                .expect("canon derived from rotation_freq keys");
            let display = if let Some(cons_rot) = consensus_rotations.get(canon) {
                cons_rot.clone()
            } else {
                rotations
                    .iter()
                    .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                    .expect("rotation_freq entry is non-empty")
                    .0
                    .clone()
            };
            global_to_length.insert(global, display.chars().count());
            global_to_display.insert(global, display);
        }

        // Phase D: expand each row's strigar into a per-brick token string. Each run
        // contributes `count` copies of its canonical's global token (one brick per copy);
        // brick width comes from `motif_lengths` (the kmer length). A run whose letter is
        // absent from the row's motif map, or whose canonical was never scored, is dropped.
        //
        // Because the global token is canonical-rotation-aware (Phase B/C), the same STR
        // unit appearing under different local letters across rows (e.g. ACCCTA:A in one and
        // TAACCC:A in another) is automatically assigned the same token and colour.
        let mut expanded_strigars: Vec<String> = Vec::new();

        for (motif_str, strigar_str) in strigars_ref {
            let local_map = parse_motif_map(motif_str);
            let mut local_to_global: HashMap<String, char> = HashMap::new();
            for (letter, kmer) in &local_map {
                let canon = canonical_rotation(kmer);
                if let Some(&global) = canonical_to_global.get(&canon) {
                    local_to_global.insert(letter.clone(), global);
                }
            }

            let mut expanded = String::new();
            for (count, letter) in parse_strigar_runs(strigar_str) {
                if let Some(&global) = local_to_global.get(&letter) {
                    for _ in 0..count {
                        expanded.push(global);
                    }
                }
            }
            expanded_strigars.push(expanded);
        }

        // Phase E: Auto-generate template colours
        // Default 20-color palette (tab10 + tab20 lighter variants).
        // Override with `with_strigar_colors` before calling `with_strigars`.
        const DEFAULT_COLORS: &[&str] = &[
            "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
            "#bcbd22", "#17becf", "#aec7e8", "#ffbb78", "#98df8a", "#ff9896", "#c5b0d5", "#c49c94",
            "#f7b6d2", "#c7c7c7", "#dbdb8d", "#9edae5",
        ];
        let palette: Vec<&str> = match &self.strigar_palette {
            Some(p) => p.iter().map(|s| s.as_str()).collect(),
            None => DEFAULT_COLORS.to_vec(),
        };
        // Single-base motifs use the same DNA colors as the flanking-base renderer
        // so that A/T/C/G bricks are visually consistent with flanking sequence bricks.
        // Multi-base motifs consume palette slots in order, skipping single-base motifs.
        let dna_brick_color = |canon: &str| -> Option<&'static str> {
            if canon.len() != 1 {
                return None;
            }
            match canon {
                "A" | "a" => Some("rgb(0,150,0)"),
                "C" | "c" => Some("rgb(0,0,255)"),
                "G" | "g" => Some("rgb(209,113,5)"),
                "T" | "t" => Some("rgb(255,0,0)"),
                _ => None,
            }
        };
        let mut auto_template: HashMap<char, String> = HashMap::new();
        let mut palette_idx = 0usize;
        for (canon, _) in sorted_canonicals.iter() {
            let global_letter = canonical_to_global[canon];
            // Precedence: explicit per-motif colour (keyed by canonical k-mer) wins over
            // the DNA default and the auto-palette. Overridden motifs do NOT consume a
            // palette slot, so the remaining auto-assigned motifs keep a stable order.
            let color = if let Some(c) = self.motif_colors.as_ref().and_then(|m| m.get(canon)) {
                c.clone()
            } else if let Some(dna) = dna_brick_color(canon) {
                dna.to_string()
            } else {
                let c = palette[palette_idx % palette.len()].to_string();
                palette_idx += 1;
                c
            };
            auto_template.insert(global_letter, color);
        }

        self.template = Some(auto_template);
        self.motifs = Some(global_to_display);
        self.strigar_exp = Some(expanded_strigars);
        self.motif_lengths = Some(global_to_length);

        self
    }

    /// Set the character-to-color template.
    ///
    /// Keys are single characters matching those in the sequences. Values
    /// are CSS color strings. Build from [`BrickTemplate`] or construct
    /// manually for custom alphabets.
    ///
    /// ```rust,no_run
    /// use std::collections::HashMap;
    /// use kuva::plot::BrickPlot;
    ///
    /// let mut tmpl = HashMap::new();
    /// tmpl.insert('H', "steelblue".to_string());   // helix
    /// tmpl.insert('E', "firebrick".to_string());   // strand
    /// tmpl.insert('C', "#aaaaaa".to_string());     // coil
    ///
    /// let plot = BrickPlot::new()
    ///     .with_sequences(vec!["HHHCCCEEEE"])
    ///     .with_names(vec!["prot_1"])
    ///     .with_template(tmpl);
    /// ```
    pub fn with_template(mut self, template: HashMap<char, String>) -> Self {
        self.template = Some(template);
        self
    }

    /// Apply a single offset to every row.
    ///
    /// Shifts all sequences left by `x_offset` characters. Use this to align
    /// the region of interest at x = 0 when all reads share the same
    /// flanking prefix.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// // Skip an 18-character common prefix so the repeat starts at x = 0
    /// let plot = BrickPlot::new()
    ///     .with_x_offset(18.0);
    /// ```
    pub fn with_x_offset(mut self, x_offset: f64) -> Self {
        self.x_offset = x_offset;
        self
    }

    /// Apply independent offsets to individual rows.
    ///
    /// Accepts an iterable of `f64` or `Option<f64>` values (one per row,
    /// same order as [`with_sequences`](Self::with_sequences)). Plain `f64`
    /// values are treated as `Some(v)`; `None` entries fall back to the
    /// global [`x_offset`](Self::x_offset). Rows beyond the iterator length
    /// also fall back.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// // Three reads with different prefix lengths; fourth falls back to global offset 12.
    /// let plot = BrickPlot::new()
    ///     .with_x_offset(12.0)
    ///     .with_x_offsets(vec![Some(18.0_f64), Some(10.0), None]);
    /// ```
    pub fn with_x_offsets<T, I>(mut self, offsets: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: IntoRowOffset,
    {
        self.x_offsets = Some(offsets.into_iter().map(|x| x.into_row_offset()).collect());
        self
    }

    /// Align reads by their genomic start position (in nucleotides).
    ///
    /// Accepts one start coordinate per read (same order as
    /// [`with_sequences`](Self::with_sequences) /
    /// [`with_strigars`](Self::with_strigars)). Each value is the position in
    /// the reference at which that read begins; kuva shifts the row so that
    /// position aligns with the shared x-axis.
    ///
    /// This is a convenience wrapper around [`with_x_offsets`](Self::with_x_offsets):
    /// internally each start position `s` is stored as `x_offset = -s` so that
    /// `map_x(x_start - (-s)) = map_x(x_start + s)` places the first brick at
    /// coordinate `s`.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// // Two reads; read_2 starts 19 nt into the reference so its first brick
    /// // lines up with position 19 on the shared axis.
    /// let plot = BrickPlot::new()
    ///     .with_names(vec!["read_1", "read_2"])
    ///     .with_start_positions(vec![0.0_f64, 19.0]);
    /// ```
    pub fn with_start_positions<T, I>(self, positions: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<f64>,
    {
        let offsets: Vec<Option<f64>> = positions.into_iter().map(|p| Some(-p.into())).collect();
        self.with_x_offsets(offsets)
    }

    /// Set the reference coordinate that appears at x = 0 on the axis.
    ///
    /// Applied on top of any per-row offsets from
    /// [`with_x_offsets`](Self::with_x_offsets) or
    /// [`with_start_positions`](Self::with_start_positions). Use this to
    /// anchor a biologically meaningful position (e.g. the repeat start) to
    /// the axis origin so the x-axis reads in coordinates relative to that
    /// point.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// // Reads start at positions 0 and 19; set x=0 at the repeat start (pos 19).
    /// // x-axis will show -19 … +N rather than 0 … M.
    /// let plot = BrickPlot::new()
    ///     .with_start_positions(vec![0.0_f64, 19.0])
    ///     .with_x_origin(19.0);
    /// ```
    pub fn with_x_origin(mut self, origin: f64) -> Self {
        self.x_origin = origin;
        self
    }

    /// Load flanked strigar data: left flank DNA, STR motif/strigar, right flank DNA.
    ///
    /// Each item is a `(left_seq, motif_string, strigar_string, right_seq)` tuple.
    /// The left and right sequences are raw DNA strings (one character = 1 nucleotide
    /// = 1 unit of axis space). The STR region is decoded the same way as
    /// [`with_strigars`](Self::with_strigars).
    ///
    /// The rendered layout per row is:
    /// `[left_flank] [STR bricks] [right_flank]`
    ///
    /// Flanks are drawn using the standard DNA colour template
    /// (A = green, C = blue, G = orange/gold, T = red).
    ///
    /// Set [`with_consensus_row`](Self::with_consensus_row) **before** calling this
    /// method if you want consensus-anchored rotation labels.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new()
    ///     .with_names(vec!["consensus", "read_1"])
    ///     .with_consensus_row(0)
    ///     .with_flanked_strigars(vec![
    ///         ("ACGTACGT", "CAG:A",  "12A", "TGCATGCA"),
    ///         ("ACGTACGT", "CAG:A",  "10A", "TGCATGCA"),
    ///     ]);
    /// ```
    pub fn with_flanked_strigars<L, M, S, R, I>(mut self, flanked: I) -> Self
    where
        I: IntoIterator<Item = (L, M, S, R)>,
        L: Into<String>,
        M: Into<String>,
        S: Into<String>,
        R: Into<String>,
    {
        let mut lefts = Vec::new();
        let mut strigars = Vec::new();
        let mut rights = Vec::new();
        for (left, motif, strigar, right) in flanked {
            lefts.push(left.into());
            strigars.push((motif.into(), strigar.into()));
            rights.push(right.into());
        }
        self.left_flanks = Some(lefts);
        self.right_flanks = Some(rights);
        self.with_strigars(strigars)
    }

    /// Set horizontal alignment for all rows.
    ///
    /// `BrickAnchor::Left` (default) — rows start at their offset.
    /// `BrickAnchor::Right` — trailing edges of all rows align on the right.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// # use kuva::plot::brick::BrickAnchor;
    /// let plot = BrickPlot::new()
    ///     .with_anchor(BrickAnchor::Right);
    /// ```
    pub fn with_anchor(mut self, anchor: BrickAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Append `*` to the legend label of the primary (most-frequent) motif.
    ///
    /// In strigar mode the most-frequent motif is always assigned global letter A.
    /// Calling this method marks it in the legend as, e.g. `"CAG*"`.
    pub fn with_mark_primary(mut self) -> Self {
        self.mark_primary = true;
        self
    }

    /// Lock display rotations to the rotations used by a specific row (the consensus).
    ///
    /// When set, `with_strigars` / `with_flanked_strigars` seed the global display
    /// labels from this row's motif strings, so every read shows the same rotation as
    /// the consensus rather than the most-frequent rotation across all reads.
    ///
    /// **Must be called before** [`with_strigars`](Self::with_strigars) or
    /// [`with_flanked_strigars`](Self::with_flanked_strigars).
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new()
    ///     .with_names(vec!["consensus", "read_1", "read_2"])
    ///     .with_consensus_row(0)          // row 0 is the consensus
    ///     .with_strigars(vec![
    ///         ("CAG:A".to_string(), "12A".to_string()),
    ///         ("AGC:A".to_string(), "10A".to_string()), // same kmer, different rotation
    ///         ("GCA:A".to_string(),  "9A".to_string()),
    ///     ]);
    /// ```
    pub fn with_consensus_row(mut self, row: usize) -> Self {
        self.consensus_row = Some(row);
        self
    }

    /// Set pre-computed human-readable notation strings, one per row.
    ///
    /// Each element is `Some(text)` to render a centred label above that row,
    /// or `None` to draw nothing. Typically the consensus row has a notation
    /// and reads may or may not.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// let plot = BrickPlot::new()
    ///     .with_names(vec!["consensus", "read_1"])
    ///     .with_notations(vec![
    ///         Some("(CAG)12".to_string()),
    ///         None,
    ///     ]);
    /// ```
    pub fn with_notations<I, T>(mut self, notations: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Option<String>>,
    {
        self.notations = Some(notations.into_iter().map(|n| n.into()).collect());
        self
    }

    /// Overlay the character label inside each brick.
    ///
    /// Useful for short sequences or large bricks where the letter is readable.
    /// For long sequences the text may become too small to see.
    pub fn with_values(mut self) -> Self {
        self.show_values = true;
        self
    }

    /// Set the desired pixel height per brick row.
    ///
    /// When set, `auto_from_plots` computes the canvas height so that each row
    /// is exactly `px` pixels tall. In a [`Figure`](crate::render::figure::Figure),
    /// panels in the same grid row auto-size so that all brick rows across
    /// different read-count plots remain identically sized — making hap1 (3 reads)
    /// and hap2 (50 reads) render with the same brick dimensions on a shared x-axis.
    ///
    /// ```rust,no_run
    /// # use kuva::plot::BrickPlot;
    /// # use kuva::plot::brick::BrickTemplate;
    /// let tmpl = BrickTemplate::new().dna();
    /// let brick = BrickPlot::new()
    ///     .with_sequences(vec!["ACGT", "ACGTACGT", "ACGT"])
    ///     .with_names(vec!["r1", "r2", "r3"])
    ///     .with_template(tmpl.template)
    ///     .with_row_height(20.0);   // 20 px per row → canvas auto-sized to 3*20 + margins
    /// ```
    pub fn with_row_height(mut self, px: f64) -> Self {
        self.row_height_px = Some(px);
        self
    }

    /// Return the number of rows (reads) in this brick plot.
    pub fn num_rows(&self) -> usize {
        if let Some(ref exp) = self.strigar_exp {
            exp.len()
        } else {
            self.sequences.len()
        }
    }

    /// Reorder rows so that new row `k` is old row `order[k]`. `order` must be a
    /// permutation of `0..num_rows`; otherwise this is a no-op. All row-parallel vectors
    /// (sequences, names, expanded strigars, flanks, per-row offsets, notations) are
    /// permuted together; per-motif colour/length maps are untouched, and `consensus_row`
    /// is remapped to its new position. Used by composites that sort alleles.
    pub fn permute_rows(mut self, order: &[usize]) -> Self {
        let n = self.num_rows();
        if order.len() != n || !order.iter().all(|&i| i < n) {
            return self;
        }
        fn permute<T: Clone>(v: &mut Vec<T>, order: &[usize]) {
            if v.len() == order.len() {
                *v = order.iter().map(|&i| v[i].clone()).collect();
            }
        }
        permute(&mut self.sequences, order);
        permute(&mut self.names, order);
        if let Some(v) = self.strigar_exp.as_mut() {
            permute(v, order);
        }
        if let Some(v) = self.strigars.as_mut() {
            permute(v, order);
        }
        if let Some(v) = self.left_flanks.as_mut() {
            permute(v, order);
        }
        if let Some(v) = self.right_flanks.as_mut() {
            permute(v, order);
        }
        if let Some(v) = self.x_offsets.as_mut() {
            permute(v, order);
        }
        if let Some(v) = self.notations.as_mut() {
            permute(v, order);
        }
        // Must follow the rows: a CIGAR left behind would draw its bar over a
        // different read, which still looks like a valid annotation.
        if let Some(v) = self.cigars.as_mut() {
            permute(v, order);
        }
        if let Some(c) = self.consensus_row {
            self.consensus_row = order.iter().position(|&i| i == c);
        }
        self
    }
}

#[cfg(test)]
mod cigar_tests {
    use super::*;

    fn plot_with(strigars: &[(&str, &str)], cigars: Vec<Option<String>>) -> BrickPlot {
        BrickPlot::new()
            .with_strigars(strigars.iter().map(|(m, s)| (*m, *s)))
            .with_cigars(cigars)
    }

    #[test]
    fn parses_ops_in_read_space() {
        let spans = parse_cigar("10M3I5M2D4S").expect("parses");
        let got: Vec<_> = spans.iter().map(|s| (s.op, s.start, s.len)).collect();
        assert_eq!(
            got,
            vec![
                (CigarOp::Match, 0.0, 10.0),
                (CigarOp::Insertion, 10.0, 3.0),
                (CigarOp::Match, 13.0, 5.0),
                // Deletion consumes no read bases: zero width, at the junction.
                (CigarOp::Deletion, 18.0, 0.0),
                (CigarOp::SoftClip, 18.0, 4.0),
            ]
        );
        assert_eq!(cigar_read_bases(&spans), 22.0);
    }

    #[test]
    fn hard_clip_and_padding_consume_nothing() {
        let spans = parse_cigar("5H10M5H").expect("parses");
        assert_eq!(cigar_read_bases(&spans), 10.0);
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn malformed_cigars_are_rejected_not_guessed() {
        assert!(parse_cigar("M10").is_none(), "op before length");
        assert!(parse_cigar("10").is_none(), "length with no op");
        assert!(parse_cigar("10M5").is_none(), "trailing length");
    }

    // A CIGAR whose read length disagrees with the row misaligns every
    // operation by the difference while still looking plausible, so it must be
    // reported rather than drawn.
    #[test]
    fn length_mismatch_is_reported_and_the_row_draws_no_bar() {
        let p = plot_with(
            &[("CAG:A,CAA:B", "3A3B9A1B1A1B18A")],
            vec![Some("99M".to_string())],
        );
        let issues = p.cigar_issues();
        assert_eq!(issues.len(), 1, "expected one issue, got {issues:?}");
        match &issues[0] {
            CigarIssue::LengthMismatch {
                row,
                cigar_read_bases,
                row_bases,
            } => {
                assert_eq!(*row, 0);
                assert_eq!(*cigar_read_bases, 99.0);
                assert_eq!(*row_bases, 108.0);
            }
            other => panic!("wrong issue: {other:?}"),
        }
        assert!(!p.cigar_layout().draws_bar[0]);
    }

    // Motif lengths are not uniform in real data: SCA17_TBP carries an 11 bp
    // motif and a 1 bp motif on the same locus. Validating against a constant
    // width rejects perfectly good CIGARs.
    #[test]
    fn row_length_honours_variable_motif_widths() {
        let p = plot_with(
            &[("CAG:A,CAACACAACAA:B,CAA:C", "3A1B9A1C1A1C17A")],
            vec![Some("13M1D94M".to_string())],
        );
        // 3*3 + 1*11 + 9*3 + 1*3 + 1*3 + 1*3 + 17*3 = 107
        assert_eq!(p.row_base_len(0), 107.0);
        assert!(p.cigar_issues().is_empty(), "{:?}", p.cigar_issues());
    }

    #[test]
    fn dedup_draws_one_bar_per_run_and_brackets_it() {
        let rows: Vec<(&str, &str)> = std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B18A"), 3)
            .chain(std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B17A"), 2))
            .collect();
        let cigars = vec![
            Some("108M".into()),
            Some("108M".into()),
            Some("108M".into()),
            Some("42M3D63M".into()),
            Some("42M3D63M".into()),
        ];
        let p = plot_with(&rows, cigars);
        let cl = p.cigar_layout();
        assert_eq!(cl.draws_bar, vec![true, false, false, true, false]);
        assert_eq!(cl.runs, vec![(0, 2), (3, 4)]);
    }

    #[test]
    fn dedup_off_draws_every_bar() {
        let rows: Vec<(&str, &str)> =
            std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B18A"), 3).collect();
        let p = plot_with(&rows, vec![Some("108M".into()); 3]).with_cigar_dedup(false);
        assert_eq!(p.cigar_layout().draws_bar, vec![true, true, true]);
    }

    // The point of "no gap unless drawn": a run of identical reads keeps the
    // normal pitch, and only a row that draws a bar costs extra height.
    #[test]
    fn only_rows_that_draw_a_bar_consume_extra_height() {
        let rows: Vec<(&str, &str)> =
            std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B18A"), 4).collect();
        let p = plot_with(&rows, vec![Some("108M".into()); 4]);
        let cl = p.cigar_layout();
        // One bar across four rows: 4 rows + 1 bar allowance.
        assert!((cl.total_units - (4.0 + BAR_ROW_UNITS)).abs() < 1e-9);
        // Rows after the first are exactly one unit apart: no reserved gap.
        // (Compared with a tolerance: the offsets accumulate by summation.)
        assert!((cl.row_top_units[2] - cl.row_top_units[1] - 1.0).abs() < 1e-9);
        assert!((cl.row_top_units[3] - cl.row_top_units[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn no_cigars_means_no_extra_height() {
        let rows: Vec<(&str, &str)> =
            std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B18A"), 5).collect();
        let p = BrickPlot::new().with_strigars(rows.iter().map(|(m, s)| (*m, *s)));
        let cl = p.cigar_layout();
        assert_eq!(cl.total_units, 5.0);
        assert_eq!(cl.order, vec![0, 1, 2, 3, 4]);
    }

    // Sorting by CIGAR groups identical CIGARs so dedup draws the fewest
    // possible bars. Any other order can split one CIGAR across several runs.
    #[test]
    fn cigar_sort_groups_identical_cigars_largest_first() {
        let rows: Vec<(&str, &str)> =
            std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B18A"), 4).collect();
        // Interleaved on purpose: A B A A -> sorted must be A A A then B.
        let p = plot_with(
            &rows,
            vec![
                Some("108M".into()),
                Some("54M3I51M".into()),
                Some("108M".into()),
                Some("108M".into()),
            ],
        )
        .with_sort(BrickSort::Cigar);
        let cl = p.cigar_layout();
        assert_eq!(cl.order, vec![0, 2, 3, 1]);
        assert_eq!(cl.draws_bar, vec![true, false, false, true]);
        assert_eq!(cl.runs.len(), 2, "one run per distinct CIGAR");
    }

    #[test]
    fn custom_sort_takes_a_caller_ranking_and_keeps_omitted_rows() {
        let rows: Vec<(&str, &str)> =
            std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B18A"), 4).collect();
        let p = plot_with(&rows, vec![Some("108M".into()); 4])
            // Out-of-range and duplicate indices are ignored; omitted rows keep
            // their original relative order at the end.
            .with_sort(BrickSort::Custom(vec![3, 1, 3, 99]));
        assert_eq!(p.cigar_layout().order, vec![3, 1, 0, 2]);
    }

    #[test]
    fn rows_without_a_cigar_draw_nothing_and_break_the_run() {
        let rows: Vec<(&str, &str)> =
            std::iter::repeat_n(("CAG:A,CAA:B", "3A3B9A1B1A1B18A"), 3).collect();
        let p = plot_with(&rows, vec![Some("108M".into()), None, Some("108M".into())]);
        let cl = p.cigar_layout();
        assert_eq!(cl.draws_bar, vec![true, false, true]);
        // The gap means the third row cannot inherit the first row's bar.
        assert_eq!(cl.runs.len(), 3);
    }

    #[test]
    fn op_colors_can_be_overridden() {
        let p = BrickPlot::new().with_cigar_colors([(CigarOp::Insertion, "#123456")]);
        assert_eq!(p.cigar_color(CigarOp::Insertion), "#123456");
        // Unset ops keep their default.
        assert_eq!(
            p.cigar_color(CigarOp::Match),
            CigarOp::Match.default_color()
        );
    }
}
