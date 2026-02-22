use crate::render::palette::Palette;
use crate::plot::volcano::LabelStyle;

pub struct ManhattanPoint {
    pub chromosome: String,
    pub x: f64,
    pub pvalue: f64,
    pub label: Option<String>,
}

pub struct ChromSpan {
    pub name: String,
    pub x_start: f64,
    pub x_end: f64,
}

pub struct ManhattanPlot {
    pub points: Vec<ManhattanPoint>,
    pub spans: Vec<ChromSpan>,
    /// Genome-wide significance threshold in -log10 scale. Default: 7.301 (p=5e-8).
    pub genome_wide: f64,
    /// Suggestive significance threshold in -log10 scale. Default: 5.0 (p=1e-5).
    pub suggestive: f64,
    /// Color for even-indexed chromosomes. Default: "steelblue".
    pub color_a: String,
    /// Color for odd-indexed chromosomes. Default: "#5aadcb".
    pub color_b: String,
    /// Optional palette overriding the alternating color scheme.
    pub palette: Option<Palette>,
    /// Radius of each data point. Default: 2.5.
    pub point_size: f64,
    /// Number of top hits to label (above genome_wide threshold). Default: 0.
    pub label_top: usize,
    /// Label placement style. Default: Nudge.
    pub label_style: LabelStyle,
    /// Hard floor for p-values before -log10 transform; auto-detected if None.
    pub pvalue_floor: Option<f64>,
    /// If set, a legend entry is shown with genome-wide and suggestive threshold lines.
    pub legend_label: Option<String>,
}

/// Specifies the reference genome chromosome sizes used for cumulative x-coordinate layout.
pub enum GenomeBuild {
    /// GRCh37 / hg19
    Hg19,
    /// GRCh38 / hg38 (default for new analyses)
    Hg38,
    /// T2T-CHM13 v2.0 / hs1
    T2T,
    /// User-supplied list of (chrom_name, size_in_bp) pairs.
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
fn build_sizes<'a>(build: &'a GenomeBuild) -> Vec<(&'a str, u64)> {
    match build {
        GenomeBuild::Hg19 => HG19_SIZES.iter().map(|&(n, s)| (n, s)).collect(),
        GenomeBuild::Hg38 => HG38_SIZES.iter().map(|&(n, s)| (n, s)).collect(),
        GenomeBuild::T2T  => T2T_SIZES.iter().map(|&(n, s)| (n, s)).collect(),
        GenomeBuild::Custom(v) => v.iter().map(|(n, s)| (strip_chr(n.as_str()), *s)).collect(),
    }
}

// ── ManhattanPlot ────────────────────────────────────────────────────────────

impl ManhattanPlot {
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

    /// Mode 1: sequential index x-coordinates.
    /// Items are `(chrom, pvalue)`. Chromosomes are sorted by standard genomic order;
    /// points within each chromosome receive sequential integer x coordinates.
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
            let pvalues = by_chrom.get(chrom).unwrap();
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

    /// Mode 2: base-pair x-coordinates resolved from a reference genome build.
    /// Items are `(chrom, bp_pos, pvalue)`. Chromosome names are accepted with or without
    /// the "chr" prefix. Chromosomes not found in the build fall back to raw bp values.
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

    /// Mode 3: pre-computed cumulative x-coordinates.
    /// Items are `(chrom, x, pvalue)`. Points are stored as-is; spans are derived from
    /// min/max x per chromosome and sorted by x_start.
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
                let e = chrom_bounds.get_mut(chrom).unwrap();
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

    /// Set the genome-wide significance threshold in -log10 scale. Default: 7.301 (p=5e-8).
    pub fn with_genome_wide(mut self, threshold: f64) -> Self {
        self.genome_wide = threshold;
        self
    }

    /// Set the suggestive significance threshold in -log10 scale. Default: 5.0 (p=1e-5).
    pub fn with_suggestive(mut self, threshold: f64) -> Self {
        self.suggestive = threshold;
        self
    }

    /// Set the color for even-indexed chromosomes (0, 2, 4, …).
    pub fn with_color_a<S: Into<String>>(mut self, color: S) -> Self {
        self.color_a = color.into();
        self
    }

    /// Set the color for odd-indexed chromosomes (1, 3, 5, …).
    pub fn with_color_b<S: Into<String>>(mut self, color: S) -> Self {
        self.color_b = color.into();
        self
    }

    /// Override alternating colors with a full palette (cycles with modulo wrapping).
    pub fn with_palette(mut self, palette: Palette) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set the radius of each data point. Default: 2.5.
    pub fn with_point_size(mut self, size: f64) -> Self {
        self.point_size = size;
        self
    }

    /// Label the top N points above the genome-wide threshold. Default: 0 (no labels).
    pub fn with_label_top(mut self, n: usize) -> Self {
        self.label_top = n;
        self
    }

    /// Set the label placement style. Default: Nudge.
    pub fn with_label_style(mut self, style: LabelStyle) -> Self {
        self.label_style = style;
        self
    }

    /// Clamp p-values to this floor before applying -log10. Auto-detected if not set.
    pub fn with_pvalue_floor(mut self, floor: f64) -> Self {
        self.pvalue_floor = Some(floor);
        self
    }

    /// When set, a legend is shown with genome-wide and suggestive threshold line entries.
    pub fn with_legend<S: Into<String>>(mut self, label: S) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    /// Attach gene or SNP labels to individual points by `(chromosome, x, label)`.
    ///
    /// The `x` value must match the coordinate computed at data-load time:
    /// - `with_data`: sequential index (0-based within the chromosome span)
    /// - `with_data_bp`: cumulative genomic base-pair position
    /// - `with_data_x`: the raw x value you supplied
    ///
    /// A tolerance of ±0.5 is used for matching, so integer positions are always found exactly.
    /// Points that already have a label are overwritten. Points with no match are silently skipped.
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
