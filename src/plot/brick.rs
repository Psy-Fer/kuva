use std::collections::HashMap;

/// Allows `with_x_offsets` to accept plain `f64` values (auto-wrapped as `Some`)
/// as well as explicit `Option<f64>` values (for `None` fallback entries).
pub trait IntoRowOffset {
    fn into_row_offset(self) -> Option<f64>;
}

impl IntoRowOffset for f64 {
    fn into_row_offset(self) -> Option<f64> { Some(self) }
}

impl IntoRowOffset for Option<f64> {
    fn into_row_offset(self) -> Option<f64> { self }
}


fn canonical_rotation(s: &str) -> String {
    let n = s.len();
    if n == 0 { return String::new(); }
    let doubled = format!("{}{}", s, s);
    (0..n)
        .map(|i| &doubled[i..i + n])
        .min()
        .unwrap()
        .to_string()
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
/// use visus::plot::brick::BrickTemplate;
/// use visus::plot::BrickPlot;
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
/// use visus::plot::BrickPlot;
/// use visus::plot::brick::BrickTemplate;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
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
    /// Per-character nucleotide length for variable-width bricks (strigar mode).
    pub motif_lengths: Option<HashMap<char, usize>>,
    /// When `true`, draw the character label inside each brick.
    pub show_values: bool,
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
            show_values: false,
        }
    }

    /// Load sequences — one string per row, ordered top to bottom.
    ///
    /// Each character in a string is rendered as one brick (or as a
    /// variable-width brick in strigar mode). All characters must have an
    /// entry in the template; unknown characters will cause a panic.
    ///
    /// ```rust,no_run
    /// # use visus::plot::BrickPlot;
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
    /// # use visus::plot::BrickPlot;
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
    /// # use visus::plot::BrickPlot;
    /// let strigars = vec![
    ///     ("CAT:A,T:B".to_string(), "14A1B1A".to_string()),
    ///     ("CAT:A,C:B".to_string(), "12A1B3A".to_string()),
    /// ];
    /// let plot = BrickPlot::new()
    ///     .with_names(vec!["read_1", "read_2"])
    ///     .with_strigars(strigars);
    /// ```
    pub fn with_strigars<T, U, I>(mut self, strigars: I) -> Self
    where
        I: IntoIterator<Item = (T, U)>,
        T: Into<String>,
        U: Into<String>,
    {
        self.strigars = Some(strigars.into_iter()
                                .map(|(motif, strigar)| (motif.into(), strigar.into()))
                                .collect());

        // Phase A: Parse each read's motif string into local_letter → kmer map
        let per_read_maps: Vec<HashMap<char, String>> = self.strigars.as_ref().unwrap().iter()
            .map(|(motif_str, _)| {
                motif_str.split(',')
                    .map(|pair| {
                        let parts: Vec<&str> = pair.split(':').collect();
                        (parts[1].chars().next().unwrap(), parts[0].to_string())
                    })
                    .collect()
            })
            .collect();

        // Phase B: Collect all kmers, canonicalize, count frequencies
        let mut canonical_freq: HashMap<String, usize> = HashMap::new();
        let mut rotation_freq: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for read_map in &per_read_maps {
            for (_letter, kmer) in read_map {
                let canon = canonical_rotation(kmer);
                *canonical_freq.entry(canon.clone()).or_insert(0) += 1;
                *rotation_freq.entry(canon).or_default().entry(kmer.clone()).or_insert(0) += 1;
            }
        }

        // Phase C: Sort canonicals by frequency desc, assign global letters
        let mut sorted_canonicals: Vec<(String, usize)> = canonical_freq.into_iter().collect();
        sorted_canonicals.sort_by(|a, b| b.1.cmp(&a.1));

        let mut canonical_to_global: HashMap<String, char> = HashMap::new();
        let mut global_to_display: HashMap<char, String> = HashMap::new();
        let mut global_to_length: HashMap<char, usize> = HashMap::new();

        for (idx, (canon, _freq)) in sorted_canonicals.iter().enumerate() {
            let global_letter = (b'A' + idx as u8) as char;
            canonical_to_global.insert(canon.clone(), global_letter);

            // Pick the most-frequent original rotation as display label
            let rotations = rotation_freq.get(canon).unwrap();
            let display = rotations.iter().max_by_key(|(_, count)| *count).unwrap().0.clone();
            global_to_display.insert(global_letter, display.clone());
            global_to_length.insert(global_letter, display.len());
        }

        // Phase D: Remap each read's strigar to global letters and expand
        let mut expanded_strigars: Vec<String> = vec![];

        for (i, (_motif_str, strigar_str)) in self.strigars.as_ref().unwrap().iter().enumerate() {
            let read_map = &per_read_maps[i];

            // Build local_letter → global_letter mapping for this read
            let mut local_to_global: HashMap<char, char> = HashMap::new();
            for (local_letter, kmer) in read_map {
                let canon = canonical_rotation(kmer);
                let global = canonical_to_global[&canon];
                local_to_global.insert(*local_letter, global);
            }

            // Remap and expand: "10A1B4A" with A→X, B→Y → "XXXXXXXXXXYYYYY..."
            let expanded: String = strigar_str.split(char::is_alphabetic)
                .zip(strigar_str.matches(char::is_alphabetic))
                .map(|(num, ch)| {
                    let local = ch.chars().next().unwrap();
                    let global = local_to_global[&local];
                    global.to_string().repeat(num.parse::<usize>().unwrap())
                })
                .collect();

            expanded_strigars.push(expanded);
        }

        // Phase E: Auto-generate template colours
        let motif_colors: &[&str] = &[
            "rgb(31,119,180)",   // blue
            "rgb(255,127,14)",   // orange
            "rgb(44,160,44)",    // green
            "rgb(214,39,40)",    // red
            "rgb(148,103,189)",  // purple
            "rgb(140,86,75)",    // brown
            "rgb(227,119,194)",  // pink
            "rgb(127,127,127)",  // gray
            "rgb(188,189,34)",   // olive
            "rgb(23,190,207)",   // cyan
        ];
        let mut auto_template: HashMap<char, String> = HashMap::new();
        for (idx, (canon, _)) in sorted_canonicals.iter().enumerate() {
            let global_letter = canonical_to_global[canon];
            auto_template.insert(global_letter, motif_colors[idx % motif_colors.len()].to_string());
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
    /// use visus::plot::BrickPlot;
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
    /// # use visus::plot::BrickPlot;
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
    /// # use visus::plot::BrickPlot;
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

    /// Overlay the character label inside each brick.
    ///
    /// Useful for short sequences or large bricks where the letter is readable.
    /// For long sequences the text may become too small to see.
    pub fn with_values(mut self) -> Self {
        self.show_values = true;
        self
    }
}
