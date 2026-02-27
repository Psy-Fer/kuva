use crate::render::palette::Palette;
use crate::plot::volcano::LabelStyle;

/// A single SNP or variant displayed in a Manhattan plot.
pub struct ManhattanPoint {
    /// Chromosome name (normalised — "chr" prefix stripped).
    pub chromosome: String,
    /// Cumulative x coordinate: sequential index, base-pair offset, or user-supplied value.
    pub x: f64,
    /// Raw p-value (not −log10). Zero p-values are handled automatically.
    pub pvalue: f64,
    /// Optional gene or SNP label, shown when the point is in the top-N selection
    /// or was named via [`ManhattanPlot::with_point_labels`].
    pub label: Option<String>,
}

/// A labeled chromosome band on the x-axis.
pub struct ChromSpan {
    /// Chromosome name (without "chr" prefix).
    pub name: String,
    /// Left edge of the band in plot x-coordinates.
    pub x_start: f64,
    /// Right edge of the band in plot x-coordinates.
    pub x_end: f64,
}

/// Builder for a Manhattan plot.
///
/// A Manhattan plot displays GWAS p-values across the genome. Each point
/// represents a SNP; the x-axis spans chromosomes and the y-axis shows
/// **−log₁₀(p-value)**. Dashed threshold lines are drawn automatically at
/// the genome-wide and suggestive significance levels. Chromosomes are
/// colored with an alternating two-color scheme (or a full [`Palette`]).
///
/// # Input modes
///
/// Three methods load data, each mapping `(chrom, …, pvalue)` onto the
/// cumulative x-axis:
///
/// | Method | x origin | Use when |
/// |--------|----------|----------|
/// | [`with_data`](Self::with_data) | Sequential integer index | No position info needed |
/// | [`with_data_bp`](Self::with_data_bp) | Base-pair offset via `GenomeBuild` | Standard GWAS output |
/// | [`with_data_x`](Self::with_data_x) | Pre-computed x values | Custom or non-human genomes |
///
/// # Gene labels
///
/// - [`with_label_top(n)`](Self::with_label_top): label the `n` points above the genome-wide
///   threshold with the lowest p-values.
/// - [`with_point_labels`](Self::with_point_labels): attach specific gene or SNP names to
///   individual points by `(chrom, x, label)`.
/// - [`with_label_style`](Self::with_label_style): choose [`LabelStyle::Nudge`] (default),
///   [`LabelStyle::Exact`], or [`LabelStyle::Arrow`].
///
/// # Zero p-values
///
/// p-values of exactly `0.0` cannot be log-transformed. They are
/// automatically capped at the smallest non-zero p-value in the data.
/// Set an explicit cap with [`with_pvalue_floor`](Self::with_pvalue_floor).
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::{ManhattanPlot, GenomeBuild};
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// // (chrom, bp_position, pvalue) triplets from PLINK/GCTA output
/// let data: Vec<(String, f64, f64)> = vec![];  // ...your data here
///
/// let mp = ManhattanPlot::new()
///     .with_data_bp(data, GenomeBuild::Hg38)
///     .with_label_top(10)
///     .with_legend("GWAS thresholds");
///
/// let plots = vec![Plot::Manhattan(mp)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("GWAS — Base-pair Coordinates (GRCh38)")
///     .with_y_label("−log₁₀(p-value)");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("manhattan.svg", svg).unwrap();
/// ```
pub struct ManhattanPlot {
    /// All data points.
    pub points: Vec<ManhattanPoint>,
    /// Chromosome spans used to draw bands and labels on the x-axis.
    pub spans: Vec<ChromSpan>,
    /// Genome-wide significance threshold in −log₁₀ scale. Default: `7.301` (p = 5×10⁻⁸).
    pub genome_wide: f64,
    /// Suggestive significance threshold in −log₁₀ scale. Default: `5.0` (p = 1×10⁻⁵).
    pub suggestive: f64,
    /// Color for even-indexed chromosomes (0, 2, 4, …). Default: `"steelblue"`.
    pub color_a: String,
    /// Color for odd-indexed chromosomes (1, 3, 5, …). Default: `"#5aadcb"`.
    pub color_b: String,
    /// Optional palette overriding the alternating two-color scheme.
    /// Colors cycle with modulo wrapping across chromosomes.
    pub palette: Option<Palette>,
    /// Radius of each data point in pixels. Default: `2.5`.
    pub point_size: f64,
    /// Number of most-significant points to label (must exceed `genome_wide`).
    /// Default: `0` (no labels).
    pub label_top: usize,
    /// Label placement style. Default: [`LabelStyle::Nudge`].
    pub label_style: LabelStyle,
    /// Hard floor for p-values before the −log₁₀ transform. Auto-detected if `None`.
    pub pvalue_floor: Option<f64>,
    /// When `Some`, a legend shows genome-wide and suggestive threshold line entries.
    pub legend_label: Option<String>,
}

/// Reference genome chromosome sizes used for cumulative x-coordinate layout.
///
/// Pass a `GenomeBuild` to [`ManhattanPlot::with_data_bp`] so that each SNP's
/// base-pair position is converted to a cumulative genomic x coordinate. All
/// chromosomes in the build appear on the x-axis as labeled bands regardless of
/// whether they contain data.
///
/// Chromosome names are accepted **with or without** the `"chr"` prefix in both
/// `Custom` entries and data items.
pub enum GenomeBuild {
    /// GRCh37 / hg19 — 24 chromosomes (1–22, X, Y) plus MT.
    Hg19,
    /// GRCh38 / hg38 — 24 chromosomes (1–22, X, Y) plus MT.
    Hg38,
    /// T2T-CHM13 v2.0 / hs1 — the first complete telomere-to-telomere assembly.
    T2T,
    /// User-supplied chromosome list.
    ///
    /// Provide a `Vec<(chrom_name, size_in_bp)>` in the order you want chromosomes
    /// to appear on the x-axis. Names may include or omit the `"chr"` prefix.
    ///
    /// ```rust,no_run
    /// use visus::plot::GenomeBuild;
    /// let build = GenomeBuild::Custom(vec![
    ///     ("chr1".to_string(), 120_000_000),
    ///     ("chr2".to_string(),  95_000_000),
    ///     ("chrX".to_string(),  55_000_000),
    /// ]);
    /// ```
    Custom(Vec<(String, u64)>),
}

// ── Chromosome size tables ──────────────────────────────────────────────────

const HG19_SIZES: &[(&str, u64)] = &[
    ("1",249_250_621),("2",243_199_373),("3",198_022_430),("4",191_154_276),
    ("5",180_915_260),("6",171_115_067),("7",159_138_663),("8",146_364_022),
    ("9",141_213_431),("10",135_534_747),("11",135_006_516),("12",133_851_895),
    ("13",115_169_878),("14",107_349_540),("15",102_531_392),("16",90_354_753),
    ("17",81_195_210),("18",78_077_248),("19",59_128_983),("20",63_025_520),
    ("21",48_129_895),("22",51_304_566),("X",155_270_560),("Y",59_373_566),("MT",16_571),
];

const HG38_SIZES: &[(&str, u64)] = &[
    ("1",248_956_422),("2",242_193_529),("3",198_295_559),("4",190_214_555),
    ("5",181_538_259),("6",170_805_979),("7",159_345_973),("8",145_138_636),
    ("9",138_394_717),("10",133_797_422),("11",135_086_622),("12",133_275_309),
    ("13",114_364_328),("14",107_043_718),("15",101_991_189),("16",90_338_345),
    ("17",83_257_441),("18",80_373_285),("19",58_617_616),("20",64_444_167),
    ("21",46_709_983),("22",50_818_468),("X",156_040_895),("Y",57_227_415),("MT",16_569),
];

const T2T_SIZES: &[(&str, u64)] = &[
    ("1",248_387_328),("2",242_696_752),("3",201_105_948),("4",193_574_945),
    ("5",182_045_439),("6",172_126_628),("7",160_567_428),("8",146_259_331),
    ("9",150_617_247),("10",134_758_134),("11",135_127_769),("12",133_324_548),
    ("13",113_566_686),("14",101_161_492),("15",99_753_195),("16",96_330_374),
    ("17",84_276_897),("18",80_542_538),("19",61_707_364),("20",66_210_255),
    ("21",45_090_682),("22",51_324_926),("X",154_259_566),("Y",62_460_029),("MT",16_569),
];

// ── Private helpers ─────────────────────────────────────────────────────────

/// Standard chromosome sort order: 1-22, X, Y, MT, then lexicographic.
fn chrom_sort_key(name: &str) -> (u8, u32, String) {
    let s = strip_chr(name);
    match s {
        "X" | "x" => (1, 0, String::new()),
        "Y" | "y" => (2, 0, String::new()),
        "MT" | "M" | "mt" | "m" => (3, 0, String::new()),
        other => {
            if let Ok(n) = other.parse::<u32>() {
                (0, n, String::new())
            } else {
                (4, 0, other.to_string())
            }
        }
    }
}

/// Strip optional "chr" prefix for lookup.
fn strip_chr(name: &str) -> &str {
    name.strip_prefix("chr").unwrap_or(name)
}

/// Resolve the size slice from a GenomeBuild, normalising Custom entries with strip_chr.
fn build_sizes(build: &GenomeBuild) -> Vec<(&str, u64)> {
    match build {
        GenomeBuild::Hg19 => HG19_SIZES.iter().map(|&(n, s)| (n, s)).collect(),
        GenomeBuild::Hg38 => HG38_SIZES.iter().map(|&(n, s)| (n, s)).collect(),
        GenomeBuild::T2T  => T2T_SIZES.iter().map(|&(n, s)| (n, s)).collect(),
        GenomeBuild::Custom(v) => v.iter().map(|(n, s)| (strip_chr(n.as_str()), *s)).collect(),
    }
}

// ── ManhattanPlot ────────────────────────────────────────────────────────────

impl Default for ManhattanPlot {
    fn default() -> Self { Self::new() }
}

impl ManhattanPlot {
    /// Create a Manhattan plot with default settings.
    ///
    /// Defaults: genome-wide threshold `7.301` (p = 5×10⁻⁸), suggestive `5.0`
    /// (p = 1×10⁻⁵), steelblue / `#5aadcb` alternating colors, point size `2.5`,
    /// no labels, no legend.
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            spans: Vec::new(),
            genome_wide: -5e-8_f64.log10(), // ≈ 7.301
            suggestive: 5.0,
            color_a: "steelblue".into(),
            color_b: "#5aadcb".into(),
            palette: None,
            point_size: 2.5,
            label_top: 0,
            label_style: LabelStyle::default(),
            pvalue_floor: None,
            legend_label: None,
        }
    }

    /// Compute the p-value floor used for -log10 transformation.
    pub fn floor(&self) -> f64 {
        if let Some(f) = self.pvalue_floor { return f; }
        self.points.iter()
            .map(|p| p.pvalue)
            .filter(|&p| p > 0.0)
            .fold(f64::INFINITY, f64::min)
            .max(1e-300)
    }

    /// **Input mode 1** — sequential integer x-coordinates.
    ///
    /// Accepts `(chrom, pvalue)` pairs. Chromosomes are sorted into standard
    /// genomic order (1–22, X, Y, MT); points within each chromosome receive
    /// consecutive integer x positions starting from the previous chromosome's
    /// end. Use this mode when base-pair positions are unavailable or unimportant.
    ///
    /// ```rust,no_run
    /// use visus::plot::ManhattanPlot;
    ///
    /// let data: Vec<(String, f64)> = vec![
    ///     ("1".into(), 0.42), ("1".into(), 3e-8),
    ///     ("2".into(), 0.17), ("2".into(), 5e-6),
    /// ];
    /// let mp = ManhattanPlot::new().with_data(data);
    /// ```
    pub fn with_data<I, S, G>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, G)>,
        S: Into<String>,
        G: Into<f64>,
    {
        let mut chrom_order: Vec<String> = Vec::new();
        let mut by_chrom: std::collections::HashMap<String, Vec<f64>> =
            std::collections::HashMap::new();

        for (s, g) in iter {
            let chrom: String = s.into();
            let pvalue: f64 = g.into();
            if !by_chrom.contains_key(&chrom) {
                chrom_order.push(chrom.clone());
            }
            by_chrom.entry(chrom).or_default().push(pvalue);
        }

        chrom_order.sort_by_key(|c| chrom_sort_key(c));

        let mut span_offset = 0.0_f64;
        let mut spans = Vec::new();
        let mut points = Vec::new();

        for chrom in &chrom_order {
            let pvalues = by_chrom.get(chrom).expect("chrom_order derived from by_chrom keys");
            let x_start = span_offset;
            for (i, &pvalue) in pvalues.iter().enumerate() {
                points.push(ManhattanPoint {
                    chromosome: chrom.clone(),
                    x: span_offset + i as f64,
                    pvalue,
                    label: None,
                });
            }
            let x_end = span_offset + pvalues.len() as f64 - 1.0;
            spans.push(ChromSpan { name: chrom.clone(), x_start, x_end });
            span_offset += pvalues.len() as f64;
        }

        self.points = points;
        self.spans = spans;
        self
    }

    /// **Input mode 2** — base-pair x-coordinates resolved from a reference genome build.
    ///
    /// Accepts `(chrom, bp_position, pvalue)` triplets. Each SNP's x coordinate is
    /// computed as the chromosome's cumulative offset in the build plus its base-pair
    /// position, giving a true genomic x-axis. All chromosomes defined in the build
    /// appear as labeled bands even if they contain no data.
    ///
    /// Chromosome names may include or omit the `"chr"` prefix. Chromosomes not found
    /// in the build are appended after the last known chromosome.
    ///
    /// ```rust,no_run
    /// use visus::plot::{ManhattanPlot, GenomeBuild};
    ///
    /// let data = vec![("1", 100_000_000_f64, 3e-10_f64), ("6", 50_000_000_f64, 8e-9)];
    /// let mp = ManhattanPlot::new().with_data_bp(data, GenomeBuild::Hg38);
    /// ```
    pub fn with_data_bp<I, S, F, G>(mut self, iter: I, build: GenomeBuild) -> Self
    where
        I: IntoIterator<Item = (S, F, G)>,
        S: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        // Normalise chromosome names (strip "chr" prefix) at ingestion time.
        let raw: Vec<(String, f64, f64)> = iter.into_iter()
            .map(|(s, f, g)| {
                let chrom_raw: String = s.into();
                let chrom = strip_chr(&chrom_raw).to_string();
                (chrom, f.into(), g.into())
            })
            .collect();

        let sizes = build_sizes(&build);

        // Build cumulative offsets in build order.
        let mut cum_offsets: std::collections::HashMap<&str, u64> =
            std::collections::HashMap::new();
        let mut running = 0u64;
        for &(name, size) in &sizes {
            cum_offsets.insert(name, running);
            running += size;
        }
        let total_genome = running;

        // Assign x coordinates to points.
        let mut points = Vec::new();
        for (chrom, bp, pvalue) in &raw {
            let x = if let Some(&offset) = cum_offsets.get(chrom.as_str()) {
                offset as f64 + bp
            } else {
                total_genome as f64 + bp
            };
            points.push(ManhattanPoint {
                chromosome: chrom.clone(),
                x,
                pvalue: *pvalue,
                label: None,
            });
        }

        // Build spans for ALL chromosomes in the build order.
        // Chromosomes without data appear as empty (labelled) regions on the x-axis.
        let mut running = 0u64;
        let mut spans = Vec::new();
        for &(name, size) in &sizes {
            spans.push(ChromSpan {
                name: name.to_string(),
                x_start: running as f64,
                x_end: (running + size) as f64,
            });
            running += size;
        }

        // Handle chromosomes not found in the build (fallback span from data x range).
        let mut unknown_bounds: std::collections::HashMap<String, (f64, f64)> =
            std::collections::HashMap::new();
        for pt in &points {
            if !cum_offsets.contains_key(pt.chromosome.as_str()) {
                let e = unknown_bounds
                    .entry(pt.chromosome.clone())
                    .or_insert((f64::INFINITY, f64::NEG_INFINITY));
                e.0 = e.0.min(pt.x);
                e.1 = e.1.max(pt.x);
            }
        }
        if !unknown_bounds.is_empty() {
            let mut extra: Vec<ChromSpan> = unknown_bounds
                .into_iter()
                .map(|(name, (xs, xe))| ChromSpan { name, x_start: xs, x_end: xe })
                .collect();
            extra.sort_by(|a, b| {
                a.x_start.partial_cmp(&b.x_start).unwrap_or(std::cmp::Ordering::Equal)
            });
            spans.extend(extra);
        }

        self.points = points;
        self.spans = spans;
        self
    }

    /// **Input mode 3** — pre-computed cumulative x-coordinates.
    ///
    /// Accepts `(chrom, x, pvalue)` triplets where `x` is already in the
    /// cumulative coordinate system you want. Spans are derived from the
    /// min/max x per chromosome. Use this mode when working with non-human
    /// genomes or when you want full control over x positioning.
    ///
    /// ```rust,no_run
    /// use visus::plot::ManhattanPlot;
    ///
    /// // x positions are cumulative across three custom chromosomes
    /// let data = vec![
    ///     ("A",  10.0_f64, 0.42_f64), ("A", 20.0, 3e-8),
    ///     ("B", 120.0,     0.17),     ("B", 130.0, 5e-6),
    /// ];
    /// let mp = ManhattanPlot::new().with_data_x(data);
    /// ```
    pub fn with_data_x<I, S, F, G>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, F, G)>,
        S: Into<String>,
        F: Into<f64>,
        G: Into<f64>,
    {
        let raw: Vec<(String, f64, f64)> = iter.into_iter()
            .map(|(s, f, g)| (s.into(), f.into(), g.into()))
            .collect();

        let mut points = Vec::new();
        let mut chrom_bounds: std::collections::HashMap<String, (f64, f64)> =
            std::collections::HashMap::new();
        let mut seen_chroms: Vec<String> = Vec::new();

        for (chrom, x, pvalue) in &raw {
            points.push(ManhattanPoint {
                chromosome: chrom.clone(),
                x: *x,
                pvalue: *pvalue,
                label: None,
            });
            if !chrom_bounds.contains_key(chrom) {
                seen_chroms.push(chrom.clone());
                chrom_bounds.insert(chrom.clone(), (*x, *x));
            } else {
                let e = chrom_bounds.get_mut(chrom).expect("chrom already inserted in seen_chroms");
                e.0 = e.0.min(*x);
                e.1 = e.1.max(*x);
            }
        }

        let mut spans: Vec<ChromSpan> = seen_chroms
            .into_iter()
            .map(|name| {
                let (x_start, x_end) = chrom_bounds[&name];
                ChromSpan { name, x_start, x_end }
            })
            .collect();
        spans.sort_by(|a, b| {
            a.x_start.partial_cmp(&b.x_start).unwrap_or(std::cmp::Ordering::Equal)
        });

        self.points = points;
        self.spans = spans;
        self
    }

    // ── Builder methods ──────────────────────────────────────────────────────

    /// Set the genome-wide significance threshold in −log₁₀ scale.
    ///
    /// Default: `7.301` (corresponding to p = 5×10⁻⁸). A dashed red line is
    /// drawn at this y position. Only points above this threshold are candidates
    /// for [`with_label_top`](Self::with_label_top) labels.
    pub fn with_genome_wide(mut self, threshold: f64) -> Self {
        self.genome_wide = threshold;
        self
    }

    /// Set the suggestive significance threshold in −log₁₀ scale.
    ///
    /// Default: `5.0` (corresponding to p = 1×10⁻⁵). A dashed gray line is
    /// drawn at this y position.
    pub fn with_suggestive(mut self, threshold: f64) -> Self {
        self.suggestive = threshold;
        self
    }

    /// Set the color for even-indexed chromosomes (0, 2, 4, …).
    ///
    /// Default: `"steelblue"`. Accepts any CSS color string. Ignored when a
    /// full [`Palette`] is set via [`with_palette`](Self::with_palette).
    pub fn with_color_a<S: Into<String>>(mut self, color: S) -> Self {
        self.color_a = color.into();
        self
    }

    /// Set the color for odd-indexed chromosomes (1, 3, 5, …).
    ///
    /// Default: `"#5aadcb"`. Accepts any CSS color string. Ignored when a
    /// full [`Palette`] is set via [`with_palette`](Self::with_palette).
    pub fn with_color_b<S: Into<String>>(mut self, color: S) -> Self {
        self.color_b = color.into();
        self
    }

    /// Override the alternating two-color scheme with a full palette.
    ///
    /// Colors are assigned to chromosomes in order, cycling with modulo
    /// wrapping when the palette has fewer entries than chromosomes. Use any
    /// named constructor from [`Palette`] or [`Palette::custom`].
    ///
    /// ```rust,no_run
    /// use visus::plot::{ManhattanPlot, GenomeBuild};
    /// use visus::Palette;
    ///
    /// let mp = ManhattanPlot::new()
    ///     .with_data_bp(vec![("1", 1_f64, 0.01_f64)], GenomeBuild::Hg38)
    ///     .with_palette(Palette::tol_bright());
    /// ```
    pub fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the radius of each data point in pixels. Default: `2.5`.
    pub fn with_point_size(mut self, size: f64) -> Self {
        self.point_size = size;
        self
    }

    /// Label the `n` most significant points above the genome-wide threshold.
    ///
    /// Points are selected by lowest p-value among those exceeding
    /// `genome_wide`. Use [`with_label_style`](Self::with_label_style) to
    /// control placement. Default: `0` (no labels).
    ///
    /// ```rust,no_run
    /// # use visus::plot::ManhattanPlot;
    /// let mp = ManhattanPlot::new()
    ///     // ...load data...
    ///     .with_label_top(10);  // label the 10 most significant hits
    /// ```
    pub fn with_label_top(mut self, n: usize) -> Self {
        self.label_top = n;
        self
    }

    /// Set the label placement style. Default: [`LabelStyle::Nudge`].
    ///
    /// - [`LabelStyle::Nudge`] — labels sorted by x and nudged vertically to
    ///   reduce overlap. Best default for most datasets.
    /// - [`LabelStyle::Exact`] — labels at the exact point position; may overlap.
    /// - [`LabelStyle::Arrow`] — labels offset by `(offset_x, offset_y)` px with
    ///   a gray leader line back to the point.
    ///
    /// ```rust,no_run
    /// use visus::plot::{ManhattanPlot, LabelStyle};
    /// let mp = ManhattanPlot::new()
    ///     .with_label_top(10)
    ///     .with_label_style(LabelStyle::Arrow { offset_x: 10.0, offset_y: 14.0 });
    /// ```
    pub fn with_label_style(mut self, style: LabelStyle) -> Self {
        self.label_style = style;
        self
    }

    /// Set an explicit p-value floor for the −log₁₀ transform.
    ///
    /// Points with `pvalue == 0.0` are clamped to this value before transformation.
    /// Also sets the y-axis ceiling to `−log10(floor)`. When not set, the floor is
    /// inferred as the minimum non-zero p-value in the data. Set it explicitly when
    /// comparing multiple plots that should share the same y-axis scale.
    pub fn with_pvalue_floor(mut self, floor: f64) -> Self {
        self.pvalue_floor = Some(floor);
        self
    }

    /// Enable a legend showing genome-wide and suggestive threshold line entries.
    ///
    /// The string argument is not used as a title but must be set to enable the legend.
    ///
    /// ```rust,no_run
    /// # use visus::plot::ManhattanPlot;
    /// let mp = ManhattanPlot::new()
    ///     .with_legend("GWAS thresholds");
    /// ```
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Attach gene or SNP labels to individual points by `(chromosome, x, label)`.
    ///
    /// The `x` value must match the coordinate assigned at data-load time:
    /// - `with_data`: sequential integer index (0-based within the chromosome span)
    /// - `with_data_bp`: cumulative base-pair position
    /// - `with_data_x`: the raw x value you supplied
    ///
    /// Matching uses a tolerance of ±0.5, so integer positions are always found
    /// exactly. Points that already have a label are overwritten. Points with no
    /// match are silently skipped.
    ///
    /// ```rust,no_run
    /// use visus::plot::ManhattanPlot;
    ///
    /// // Three-chromosome dataset with pre-computed x positions
    /// let data = vec![
    ///     ("1",  40.0_f64, 2e-10_f64),
    ///     ("2", 140.0,     5e-9),
    /// ];
    /// let mp = ManhattanPlot::new()
    ///     .with_data_x(data)
    ///     .with_point_labels(vec![
    ///         ("1",  40.0, "BRCA2"),
    ///         ("2", 140.0, "TP53"),
    ///     ]);
    /// ```
    pub fn with_point_labels<I, S, F, L>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, F, L)>,
        S: Into<String>,
        F: Into<f64>,
        L: Into<String>,
    {
        for (s, f, l) in iter {
            let chrom: String = s.into();
            let x: f64 = f.into();
            let label: String = l.into();
            if let Some(pt) = self.points.iter_mut()
                .find(|p| p.chromosome == chrom && (p.x - x).abs() < 0.5)
            {
                pt.label = Some(label);
            }
        }
        self
    }
}
