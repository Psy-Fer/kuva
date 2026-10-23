//! Sequence logo — stacked-letter plot of position probability matrices.
//!
//! A sequence logo displays a position-weight matrix as a column of letters
//! per position, where each letter's height is proportional to its
//! contribution at that position.  Bits (Schneider / Stephens information
//! content), raw probability, and frequency are supported via
//! [`LogoUnits`].
//!
//! # Input paths
//!
//! - [`SequenceLogo::from_matrix`] — pre-computed matrix of shape
//!   `[n_positions][alphabet_len]`. Rows sum to 1.0 (probability) or to the
//!   sample size (counts).
//! - [`SequenceLogo::from_sequences`] — a slice of equal-length sequences.
//!   Counts, probabilities, and (for bits mode) Schneider's small-sample
//!   correction are computed internally.
//! - [`SequenceLogo::from_fasta_str`] — parse a FASTA blob and dispatch to
//!   [`SequenceLogo::from_sequences`].
//!
//! # Example
//!
//! ```rust,ignore
//! use kuva::plot::logo::{Alphabet, SequenceLogo};
//!
//! let logo = SequenceLogo::from_sequences(
//!     &["ACGTAAA", "ACGTAAG", "ACGTAAC"],
//!     Alphabet::Dna,
//! ).unwrap();
//! ```

use std::collections::HashMap;

// ── Alphabet ────────────────────────────────────────────────────────────────

/// The character alphabet a logo represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alphabet {
    /// A, C, G, T.
    Dna,
    /// A, C, G, U.
    Rna,
    /// Standard 20 amino acids.
    Protein,
    /// Arbitrary letter set (e.g. methylation states, codons of length 1).
    Custom(Vec<char>),
}

impl Alphabet {
    /// The ordered list of letters this alphabet contains.
    pub fn letters(&self) -> Vec<char> {
        match self {
            Alphabet::Dna => vec!['A', 'C', 'G', 'T'],
            Alphabet::Rna => vec!['A', 'C', 'G', 'U'],
            Alphabet::Protein => vec![
                'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S',
                'T', 'V', 'W', 'Y',
            ],
            Alphabet::Custom(cs) => cs.clone(),
        }
    }

    /// Whether the alphabet is nucleic (supports reverse-complement).
    pub fn is_nucleic(&self) -> bool {
        matches!(self, Alphabet::Dna | Alphabet::Rna)
    }

    /// A rough IC ceiling for the alphabet: log₂ of its size.
    pub fn max_bits(&self) -> f64 {
        (self.letters().len() as f64).log2()
    }

    /// Complement a base in this alphabet (A↔T for DNA, A↔U for RNA).
    ///
    /// Non-base characters (and any character in a non-nucleic alphabet)
    /// are returned unchanged.
    pub fn complement(&self, c: char) -> char {
        match self {
            Alphabet::Dna => match c {
                'A' => 'T',
                'T' => 'A',
                'C' => 'G',
                'G' => 'C',
                _ => c,
            },
            Alphabet::Rna => match c {
                'A' => 'U',
                'U' => 'A',
                'C' => 'G',
                'G' => 'C',
                _ => c,
            },
            Alphabet::Protein | Alphabet::Custom(_) => c,
        }
    }
}

/// The column label used for gaps when [`SequenceLogo::with_include_gaps`]
/// adds them to the stack.
pub const GAP_LETTER: char = '-';

// ── Units ───────────────────────────────────────────────────────────────────

/// Y-axis units for a logo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogoUnits {
    /// Shannon information content in bits (Schneider / Stephens).
    #[default]
    Bits,
    /// Raw probabilities in `[0, 1]`.
    Probability,
    /// Raw frequency counts (letters sized by count, y unbounded).
    Frequency,
}

// ── Palette ─────────────────────────────────────────────────────────────────

/// Canonical color presets for common alphabets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PalettePreset {
    /// NCBI/WebLogo default: A green, C blue, G orange, T red.
    /// For protein, the WebLogo chemistry groups (many residues per color).
    Classic,
    /// WebLogo protein palette (chemistry groups).
    WebLogo,
    /// Shapely (RasMol) palette — per-residue structural-biology colors.
    Shapely,
    /// Clustal-X protein palette (chemistry groups).
    Clustal,
    /// Taylor (1997) palette — a distinct color for every amino acid.
    ///
    /// Fully saturated by design, which reads well for small letters in an
    /// alignment viewer but glares at sequence-logo scale, where each glyph
    /// is a large block of flat color. Prefer [`PalettePreset::Muted`]
    /// unless you specifically want Taylor's colors.
    Taylor,
    /// Soft palette: one hue family per chemistry group, with a lightness
    /// ramp *within* each group.
    ///
    /// This is the protein default. Chemistry reads at a glance — aliphatic
    /// residues are browns, aromatics purples, basic blues, acidic reds,
    /// polar greens, and G/P/C each get their own colour — while every
    /// residue still has its own shade. Mid-saturation throughout: large
    /// filled glyphs are far more tiring than small text, so it sits between
    /// the glare of [`PalettePreset::Taylor`] and a flat grey ramp.
    Muted,
    /// All letters in black.
    Mono,
}

impl PalettePreset {
    /// The preset used when the caller does not pick one.
    ///
    /// Nucleotides get the familiar green/blue/amber/red ACGT colors. Protein
    /// and custom alphabets get [`PalettePreset::Muted`]: a 4- or 20-letter
    /// alphabet is the difference between "four colors everyone knows" and
    /// "twenty large blocks of flat color to sit and read", and the
    /// saturated schemes are tiring at the second scale.
    pub fn default_for(alphabet: &Alphabet) -> Self {
        if alphabet.is_nucleic() {
            PalettePreset::Classic
        } else {
            PalettePreset::Muted
        }
    }
}

/// Letter → RGB color map.
///
/// Constructed from a [`PalettePreset`] (via [`AlphabetPalette::preset`]) or
/// a raw `HashMap` (via [`AlphabetPalette::from_map`]).  Per-letter overrides
/// take precedence over the preset via [`AlphabetPalette::override_colors`].
#[derive(Debug, Clone)]
pub struct AlphabetPalette {
    map: HashMap<char, [u8; 3]>,
}

impl AlphabetPalette {
    /// Build a palette from a preset and an alphabet.
    ///
    /// Any letter absent from the preset's built-in table falls back to a
    /// muted gray at render time.
    pub fn preset(preset: PalettePreset, alphabet: &Alphabet) -> Self {
        let mut map = HashMap::new();
        match preset {
            PalettePreset::Mono => {
                for c in alphabet.letters() {
                    map.insert(c, [0, 0, 0]);
                }
            }
            PalettePreset::Classic | PalettePreset::WebLogo => match alphabet {
                Alphabet::Dna | Alphabet::Rna => {
                    map.insert('A', [0, 150, 0]);
                    map.insert('C', [0, 0, 200]);
                    map.insert('G', [209, 113, 5]);
                    map.insert('T', [200, 0, 0]);
                    map.insert('U', [200, 0, 0]);
                }
                Alphabet::Protein | Alphabet::Custom(_) => {
                    // WebLogo chemistry groups.
                    for c in ['G', 'S', 'T', 'Y', 'C'] {
                        map.insert(c, [0, 150, 0]);
                    }
                    for c in ['Q', 'N'] {
                        map.insert(c, [128, 0, 128]);
                    }
                    for c in ['K', 'R', 'H'] {
                        map.insert(c, [0, 0, 200]);
                    }
                    for c in ['D', 'E'] {
                        map.insert(c, [200, 0, 0]);
                    }
                    for c in ['A', 'V', 'L', 'I', 'P', 'W', 'F', 'M'] {
                        map.insert(c, [0, 0, 0]);
                    }
                }
            },
            PalettePreset::Shapely => match alphabet {
                Alphabet::Dna | Alphabet::Rna => {
                    map.insert('A', [160, 160, 255]);
                    map.insert('C', [255, 140, 75]);
                    map.insert('G', [255, 112, 112]);
                    map.insert('T', [160, 255, 160]);
                    map.insert('U', [160, 255, 160]);
                }
                Alphabet::Protein | Alphabet::Custom(_) => {
                    // RasMol shapely residue colors.  Glycine is white in
                    // RasMol, which is invisible on kuva's white ground, so
                    // it is darkened to a light grey here.
                    for (c, rgb) in [
                        ('A', [140, 255, 140]),
                        ('R', [0, 0, 124]),
                        ('N', [255, 124, 112]),
                        ('D', [160, 0, 66]),
                        ('C', [255, 255, 112]),
                        ('Q', [255, 76, 76]),
                        ('E', [102, 0, 0]),
                        ('G', [200, 200, 200]),
                        ('H', [112, 112, 255]),
                        ('I', [0, 76, 0]),
                        ('L', [69, 94, 69]),
                        ('K', [71, 71, 184]),
                        ('M', [184, 160, 66]),
                        ('F', [83, 76, 82]),
                        ('P', [82, 82, 82]),
                        ('S', [255, 112, 66]),
                        ('T', [184, 76, 0]),
                        ('W', [79, 70, 0]),
                        ('Y', [140, 112, 76]),
                        ('V', [255, 140, 255]),
                    ] {
                        map.insert(c, rgb);
                    }
                }
            },
            PalettePreset::Muted => {
                // One hue family per chemistry group, stepped dark → light so
                // residues stay separable within a group. Mid-saturation —
                // large filled glyphs are far more tiring than small text,
                // but an all-grey ramp reads as mud.
                for (c, rgb) in [
                    // Aliphatic hydrophobic — brown ramp.
                    ('A', [140, 98, 57]),
                    ('V', [166, 122, 72]),
                    ('L', [189, 146, 92]),
                    ('I', [207, 170, 118]),
                    ('M', [222, 192, 148]),
                    // Aromatic — purple ramp.
                    ('F', [106, 76, 147]),
                    ('W', [139, 107, 177]),
                    ('Y', [174, 145, 204]),
                    // Basic — blue ramp.
                    ('K', [43, 95, 158]),
                    ('R', [74, 130, 192]),
                    ('H', [122, 166, 217]),
                    // Acidic — red ramp.
                    ('D', [184, 67, 60]),
                    ('E', [217, 112, 95]),
                    // Polar uncharged — green ramp.
                    ('S', [46, 125, 79]),
                    ('T', [71, 160, 107]),
                    ('N', [111, 191, 142]),
                    ('Q', [158, 216, 178]),
                    // Special — each its own: glycine grey, proline orange,
                    // cysteine the conventional sulfur yellow.
                    ('G', [120, 120, 120]),
                    ('P', [224, 123, 57]),
                    ('C', [212, 176, 0]),
                ] {
                    map.insert(c, rgb);
                }
                // Nucleotides keep the original ACGT colors. Four familiar
                // colors don't have the glare problem that twenty large
                // blocks do, and these are the ones people already read.
                if alphabet.is_nucleic() {
                    map.insert('A', [0, 150, 0]);
                    map.insert('C', [0, 0, 200]);
                    map.insert('G', [209, 113, 5]);
                    map.insert('T', [200, 0, 0]);
                    map.insert('U', [200, 0, 0]);
                }
            }
            PalettePreset::Taylor => {
                // Taylor (1997): one distinct hue per amino acid.
                for (c, rgb) in [
                    ('A', [204, 255, 0]),
                    ('R', [0, 0, 255]),
                    ('N', [204, 0, 255]),
                    ('D', [255, 0, 0]),
                    ('C', [255, 255, 0]),
                    ('Q', [255, 0, 204]),
                    ('E', [255, 0, 102]),
                    ('G', [255, 153, 0]),
                    ('H', [0, 102, 255]),
                    ('I', [102, 255, 0]),
                    ('L', [51, 255, 0]),
                    ('K', [102, 0, 255]),
                    ('M', [0, 255, 0]),
                    ('F', [0, 255, 102]),
                    ('P', [255, 204, 0]),
                    ('S', [255, 51, 0]),
                    ('T', [255, 102, 0]),
                    ('W', [0, 204, 255]),
                    ('Y', [0, 255, 204]),
                    ('V', [153, 255, 0]),
                ] {
                    map.insert(c, rgb);
                }
                // Nucleotides keep the familiar classic colors.
                if alphabet.is_nucleic() {
                    map.insert('A', [0, 150, 0]);
                    map.insert('C', [0, 0, 200]);
                    map.insert('G', [209, 113, 5]);
                    map.insert('T', [200, 0, 0]);
                    map.insert('U', [200, 0, 0]);
                }
            }
            PalettePreset::Clustal => {
                for c in ['G', 'A', 'S', 'T'] {
                    map.insert(c, [255, 160, 0]);
                }
                for c in ['I', 'L', 'V', 'M', 'F', 'C'] {
                    map.insert(c, [0, 150, 0]);
                }
                for c in ['H', 'K', 'R'] {
                    map.insert(c, [0, 160, 200]);
                }
                for c in ['D', 'E'] {
                    map.insert(c, [220, 20, 60]);
                }
                for c in ['N', 'Q'] {
                    map.insert(c, [40, 130, 40]);
                }
                map.insert('P', [200, 190, 0]);
                for c in ['W', 'Y'] {
                    map.insert(c, [180, 0, 130]);
                }
            }
        }
        AlphabetPalette { map }
    }

    /// Wrap a raw letter→RGB map.
    pub fn from_map(map: HashMap<char, [u8; 3]>) -> Self {
        AlphabetPalette { map }
    }

    /// Merge per-letter overrides on top of the current map.
    pub fn override_colors<I>(&mut self, overrides: I)
    where
        I: IntoIterator<Item = (char, [u8; 3])>,
    {
        for (c, rgb) in overrides {
            self.map.insert(c, rgb);
        }
    }

    /// Color for `c`, or a muted gray if the letter has no assigned color.
    pub fn color_of(&self, c: char) -> [u8; 3] {
        self.map.get(&c).copied().unwrap_or([120, 120, 120])
    }

    /// Iterate the (letter, color) entries in arbitrary map order.
    /// Legend collection re-sorts by letter, so ordering here is not load-bearing.
    pub fn entries(&self) -> impl Iterator<Item = (char, [u8; 3])> + '_ {
        self.map.iter().map(|(c, rgb)| (*c, *rgb))
    }
}

// ── Matrix orientation ──────────────────────────────────────────────────────

/// Orientation of a raw PWM/PPM matrix loaded from a CSV/TSV file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatrixOrientation {
    /// Rows are positions, columns are letters (WebLogo / MEME / HOMER).
    #[default]
    RowsArePositions,
    /// Rows are letters, columns are positions (JASPAR).
    RowsAreLetters,
}

/// Autodetect matrix orientation from the header row.
///
/// If the first cell of every column header is a single-character letter of
/// the alphabet, returns [`MatrixOrientation::RowsArePositions`].  If the
/// first cell of every row header is such a letter, returns
/// [`MatrixOrientation::RowsAreLetters`].  Otherwise `None`.
pub fn detect_orientation(
    first_row: &[String],
    first_col: &[String],
    alphabet_letters: &[char],
) -> Option<MatrixOrientation> {
    let is_letter = |s: &str| {
        let mut cs = s.chars();
        let c = cs.next()?;
        if cs.next().is_some() {
            return None;
        }
        let upper = c.to_ascii_uppercase();
        alphabet_letters.contains(&upper).then_some(upper)
    };
    let row_all_letters = !first_row.is_empty() && first_row.iter().all(|s| is_letter(s).is_some());
    let col_all_letters = !first_col.is_empty() && first_col.iter().all(|s| is_letter(s).is_some());
    match (row_all_letters, col_all_letters) {
        (true, _) => Some(MatrixOrientation::RowsArePositions),
        (false, true) => Some(MatrixOrientation::RowsAreLetters),
        _ => None,
    }
}

// ── SequenceLogo ────────────────────────────────────────────────────────────

/// A stacked-letter sequence-logo plot.
///
/// Rows of `matrix` are positions; columns follow `letters` (the ordered
/// alphabet).  Values are raw frequencies from the source data — no
/// normalisation is applied at construction time.  The renderer converts to
/// probabilities / bits according to [`SequenceLogo::units`].
#[derive(Debug, Clone)]
pub struct SequenceLogo {
    /// Per-position frequencies, `matrix[position][letter_index]`.
    pub matrix: Vec<Vec<f64>>,
    /// Alphabet the columns of `matrix` follow.
    pub alphabet: Alphabet,
    /// Cached alphabet letters (parallel to columns of `matrix`).
    pub letters: Vec<char>,
    /// Number of sequences used to build `matrix`, when known.
    ///
    /// Required for the Schneider small-sample correction; `None` when
    /// building from a pre-computed matrix.
    pub sample_size: Option<usize>,

    /// Y-axis units.
    pub units: LogoUnits,
    /// Letter → color mapping.
    pub palette: AlphabetPalette,
    /// Offset applied to the x-axis labels (genomic coordinate of position 1).
    pub start: i64,
    /// Whether to draw the consensus row.
    pub show_consensus: bool,
    /// Fraction above which consensus letters render uppercase.
    pub consensus_threshold: f64,
    /// `None` = auto (protein/custom shows, DNA/RNA hides); `Some(b)` forces.
    pub show_legend: Option<bool>,
    /// Reverse-complement (nucleic alphabets only).
    pub reverse_complement: bool,
    /// Apply Schneider small-sample IC correction (bits mode, sequence input).
    pub apply_correction: bool,
    /// Include gaps as their own column of the alphabet.
    pub include_gaps: bool,
    /// Treat lowercase input letters as soft-masked (gaps) instead of
    /// canonicalising to uppercase.
    pub respect_case: bool,
    /// Optional plot title carried through to the layout when the layout
    /// has none of its own (see `Layout::auto_from_plots`).
    pub title: Option<String>,
    /// Raw sequences kept when built via [`SequenceLogo::from_sequences`],
    /// so `with_respect_case` / `with_include_gaps` can re-count.  `None`
    /// for matrix-built logos, where those toggles have nothing to act on.
    pub source_sequences: Option<Vec<String>>,
}

impl SequenceLogo {
    /// Shared field defaults; every constructor funnels through here.
    fn base(matrix: Vec<Vec<f64>>, alphabet: Alphabet) -> Self {
        let letters = alphabet.letters();
        let palette = AlphabetPalette::preset(PalettePreset::default_for(&alphabet), &alphabet);
        Self {
            matrix,
            alphabet,
            letters,
            sample_size: None,
            units: LogoUnits::Bits,
            palette,
            start: 1,
            show_consensus: false,
            consensus_threshold: 0.5,
            show_legend: None,
            reverse_complement: false,
            apply_correction: true,
            include_gaps: false,
            respect_case: false,
            title: None,
            source_sequences: None,
        }
    }

    /// Build a logo directly from a pre-computed matrix.
    ///
    /// `matrix[position][letter_index]` is the raw frequency; letter indices
    /// follow `alphabet.letters()`.  `with_respect_case` / `with_include_gaps`
    /// have no effect on matrix-built logos — case and gap information is
    /// already gone from a matrix.
    pub fn from_matrix(matrix: Vec<Vec<f64>>, alphabet: Alphabet) -> Self {
        Self::base(matrix, alphabet)
    }

    /// Build a logo from an equal-length set of sequences.
    ///
    /// Counts non-gap letters per position.  Small-sample correction and
    /// ambiguity-code handling are applied when the logo is rendered.
    ///
    /// Errors when: no sequences given; sequences differ in length; the
    /// sequences are empty.
    pub fn from_sequences<I, S>(sequences: I, alphabet: Alphabet) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let seqs: Vec<String> = sequences
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();
        if seqs.is_empty() {
            return Err("SequenceLogo::from_sequences: no sequences provided".to_string());
        }
        let width = seqs[0].chars().count();
        for (i, s) in seqs.iter().enumerate() {
            if s.chars().count() != width {
                return Err(format!(
                    "SequenceLogo::from_sequences: sequence {i} has length {} but sequence 0 has length {width}",
                    s.chars().count()
                ));
            }
        }
        if width == 0 {
            return Err("SequenceLogo::from_sequences: sequences are empty".to_string());
        }

        let (letters, matrix) = count_matrix(&seqs, &alphabet, false, false);
        let mut logo = Self::base(matrix, alphabet);
        logo.letters = letters;
        logo.sample_size = Some(seqs.len());
        logo.source_sequences = Some(seqs);
        Ok(logo)
    }

    /// Re-count the matrix from the stored source sequences, honouring the
    /// current `respect_case` / `include_gaps` settings.  No-op for
    /// matrix-built logos.
    fn recount(&mut self) {
        if let Some(seqs) = self.source_sequences.take() {
            let (letters, matrix) =
                count_matrix(&seqs, &self.alphabet, self.respect_case, self.include_gaps);
            self.letters = letters;
            self.matrix = matrix;
            self.source_sequences = Some(seqs);
        }
    }

    /// Parse a FASTA blob and delegate to [`SequenceLogo::from_sequences`].
    ///
    /// Sequences are joined across wrapped lines, headers dropped.  Empty
    /// records are skipped.  Any parse or shape error is returned verbatim.
    pub fn from_fasta_str(text: &str, alphabet: Alphabet) -> Result<Self, String> {
        let records = parse_fasta(text);
        if records.is_empty() {
            return Err("SequenceLogo::from_fasta_str: no records in input".to_string());
        }
        let seqs: Vec<&str> = records.iter().map(|(_, s)| s.as_str()).collect();
        SequenceLogo::from_sequences(seqs, alphabet)
    }

    // ── Builder methods ────────────────────────────────────────────────────

    pub fn with_units(mut self, units: LogoUnits) -> Self {
        self.units = units;
        self
    }

    pub fn with_palette(mut self, palette: AlphabetPalette) -> Self {
        self.palette = palette;
        self
    }

    pub fn with_preset(mut self, preset: PalettePreset) -> Self {
        self.palette = AlphabetPalette::preset(preset, &self.alphabet);
        self
    }

    pub fn with_colors<I>(mut self, overrides: I) -> Self
    where
        I: IntoIterator<Item = (char, [u8; 3])>,
    {
        self.palette.override_colors(overrides);
        self
    }

    pub fn with_start(mut self, start: i64) -> Self {
        self.start = start;
        self
    }

    pub fn with_consensus(mut self) -> Self {
        self.show_consensus = true;
        self
    }

    pub fn with_consensus_threshold(mut self, t: f64) -> Self {
        self.consensus_threshold = t.clamp(0.0, 1.0);
        self
    }

    pub fn with_no_correction(mut self) -> Self {
        self.apply_correction = false;
        self
    }

    /// Count gap characters (`-`, `.`) as their own [`GAP_LETTER`] column.
    ///
    /// Only meaningful for sequence-built logos; matrix-built logos have no
    /// gap information left to count.
    pub fn with_include_gaps(mut self) -> Self {
        self.include_gaps = true;
        self.recount();
        self
    }

    /// Treat lowercase input letters as soft-masked (counted as gaps)
    /// instead of canonicalising them to uppercase.
    ///
    /// Only meaningful for sequence-built logos.
    pub fn with_respect_case(mut self) -> Self {
        self.respect_case = true;
        self.recount();
        self
    }

    pub fn with_reverse_complement(mut self) -> Result<Self, String> {
        if !self.alphabet.is_nucleic() {
            return Err(format!(
                "with_reverse_complement is only valid for DNA/RNA alphabets (got {:?})",
                self.alphabet
            ));
        }
        self.reverse_complement = true;
        Ok(self)
    }

    pub fn with_legend(mut self) -> Self {
        self.show_legend = Some(true);
        self
    }

    pub fn with_no_legend(mut self) -> Self {
        self.show_legend = Some(false);
        self
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    // ── Computed views ────────────────────────────────────────────────────

    /// Number of x-axis positions.
    pub fn n_positions(&self) -> usize {
        self.matrix.len()
    }

    /// IC ceiling in bits for the letters actually in play.
    ///
    /// Uses `self.letters` (not the raw alphabet) so `with_include_gaps`,
    /// which adds a [`GAP_LETTER`] column, raises the ceiling consistently.
    pub fn max_bits(&self) -> f64 {
        (self.letters.len() as f64).log2()
    }

    /// Ceiling on the y-axis for the current [`LogoUnits`].
    pub fn y_max(&self) -> f64 {
        match self.units {
            LogoUnits::Bits => self.max_bits(),
            LogoUnits::Probability => 1.0,
            LogoUnits::Frequency => self
                .matrix
                .iter()
                .map(|row| row.iter().sum::<f64>())
                .fold(0.0_f64, f64::max),
        }
    }

    /// The data-space x-extent: half a column of padding on each side so
    /// column 1 starts flush against the y-axis.  Shared by `Plot::bounds`
    /// and the axis pinning in `Layout::auto_from_plots`.
    pub fn x_span(&self) -> (f64, f64) {
        (0.5, self.n_positions() as f64 + 0.5)
    }

    /// Whether a letter→color legend should render: the explicit
    /// `show_legend` setting if present, else automatic — shown for
    /// protein / custom alphabets (20 colors nobody memorises), hidden for
    /// DNA / RNA.
    pub fn legend_visible(&self) -> bool {
        self.show_legend.unwrap_or(!self.alphabet.is_nucleic())
    }

    /// Legend rows: one per *distinct color*, labelled with the letters
    /// sharing it.
    ///
    /// Chemistry-group palettes (WebLogo protein, Clustal) deliberately map
    /// many letters onto one color, so a row per letter would repeat the
    /// same swatch ten times and tell the reader nothing. Grouping turns
    /// that into "AFILMPVW ■" — which is the hydrophobic set, and reads as
    /// one fact.
    ///
    /// Returns empty when the legend is hidden, or when every letter shares
    /// a single color (as under the `Mono` preset) — a one-row legend
    /// carries no information.
    pub fn legend_groups(&self) -> Vec<(String, [u8; 3])> {
        if !self.legend_visible() {
            return Vec::new();
        }
        // Preserve alphabet order: first letter to claim a color fixes that
        // group's position in the legend.
        let mut groups: Vec<(String, [u8; 3])> = Vec::new();
        for &c in &self.letters {
            let rgb = self.palette.color_of(c);
            match groups.iter_mut().find(|(_, g)| *g == rgb) {
                Some((label, _)) => label.push(c),
                None => groups.push((c.to_string(), rgb)),
            }
        }
        if groups.len() < 2 {
            return Vec::new();
        }
        groups
    }

    /// Return the per-position stack (letters + heights) for rendering.
    ///
    /// Letters are sorted ascending by height so the tallest is drawn last
    /// (top of stack).  Heights are in y-axis units for the current
    /// [`LogoUnits`], clipped at 0 (Schneider small-sample can go negative
    /// on high-entropy positions; WebLogo clips there, so do we).
    pub fn stack(&self, position: usize) -> Vec<(char, f64)> {
        if position >= self.matrix.len() {
            return Vec::new();
        }
        let row = &self.matrix[position];
        let total: f64 = row.iter().sum();
        if total <= 0.0 {
            return Vec::new();
        }
        let mut probabilities: Vec<(char, f64)> = self
            .letters
            .iter()
            .zip(row.iter())
            .map(|(&c, &v)| (c, v / total))
            .collect();

        let column_height = match self.units {
            LogoUnits::Bits => {
                // Shannon entropy of the observed distribution.
                let h: f64 = probabilities
                    .iter()
                    .filter(|(_, p)| *p > 0.0)
                    .map(|(_, p)| -p * p.log2())
                    .sum();
                let mut r = self.max_bits() - h;
                if self.apply_correction {
                    if let Some(n) = self.sample_size {
                        if n > 0 {
                            let s = self.letters.len() as f64;
                            let e_n = (s - 1.0) / (2.0 * (2.0_f64).ln() * n as f64);
                            r -= e_n;
                        }
                    }
                }
                // Schneider clip: high-entropy small-sample positions can go
                // slightly negative, which would render as an inverted stack —
                // WebLogo clips at 0 and we follow suit.
                r.max(0.0)
            }
            LogoUnits::Probability => 1.0,
            LogoUnits::Frequency => total,
        };

        for (_, p) in probabilities.iter_mut() {
            *p *= column_height;
        }
        probabilities.sort_by(|a, b| a.1.total_cmp(&b.1));
        probabilities
    }

    /// Consensus letter and probability at `position`.
    pub fn consensus(&self, position: usize) -> Option<(char, f64)> {
        let row = self.matrix.get(position)?;
        let total: f64 = row.iter().sum();
        if total <= 0.0 {
            return None;
        }
        row.iter()
            .zip(self.letters.iter())
            .max_by(|a, b| a.0.total_cmp(b.0))
            .map(|(v, &c)| (c, v / total))
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Count letters per position across an equal-length sequence set.
///
/// Returns the column letters (the alphabet, plus [`GAP_LETTER`] when
/// `include_gaps`) and the `[position][letter]` count matrix.
///
/// Character handling per the logo conventions:
/// - whitespace: skipped entirely;
/// - lowercase: uppercased, unless `respect_case` — then counted as a gap
///   (soft-masked regions from upstream maskers);
/// - gap characters (`-`, `.`): counted in the [`GAP_LETTER`] column when
///   `include_gaps`, otherwise dropped;
/// - IUPAC ambiguity codes: treated as gaps, with a one-time stderr warning;
/// - anything else: dropped silently.
fn count_matrix(
    seqs: &[String],
    alphabet: &Alphabet,
    respect_case: bool,
    include_gaps: bool,
) -> (Vec<char>, Vec<Vec<f64>>) {
    let mut letters = alphabet.letters();
    if include_gaps {
        letters.push(GAP_LETTER);
    }
    let letter_index: HashMap<char, usize> =
        letters.iter().enumerate().map(|(i, c)| (*c, i)).collect();
    let gap_idx = include_gaps.then(|| letters.len() - 1);

    let width = seqs.first().map(|s| s.chars().count()).unwrap_or(0);
    let mut matrix = vec![vec![0.0_f64; letters.len()]; width];
    let mut warned_ambiguous = false;
    let count_gap = |row: &mut Vec<f64>| {
        if let Some(gi) = gap_idx {
            row[gi] += 1.0;
        }
    };

    for seq in seqs {
        for (pos, c) in seq.chars().enumerate() {
            if c.is_whitespace() {
                continue;
            }
            let row = &mut matrix[pos];
            if c == '-' || c == '.' {
                count_gap(row);
                continue;
            }
            if respect_case && c.is_lowercase() {
                // Soft-masked base — honour the masker's judgement.
                count_gap(row);
                continue;
            }
            let upper = c.to_ascii_uppercase();
            if let Some(&idx) = letter_index.get(&upper) {
                row[idx] += 1.0;
            } else if is_ambiguity(upper, alphabet) {
                if !warned_ambiguous {
                    eprintln!(
                        "kuva logo: ignoring IUPAC ambiguity codes in sequence input (first seen: '{upper}')"
                    );
                    warned_ambiguous = true;
                }
                count_gap(row);
            }
            // Anything else drops silently.
        }
    }
    (letters, matrix)
}

fn is_ambiguity(c: char, alphabet: &Alphabet) -> bool {
    match alphabet {
        Alphabet::Dna | Alphabet::Rna => {
            matches!(
                c,
                'N' | 'R' | 'Y' | 'S' | 'W' | 'K' | 'M' | 'B' | 'D' | 'H' | 'V'
            )
        }
        Alphabet::Protein => matches!(c, 'X' | 'B' | 'Z' | 'J' | 'U' | 'O'),
        Alphabet::Custom(_) => false,
    }
}

/// Parse a FASTA blob into `(header, sequence)` records.
///
/// Whitespace inside sequences (line breaks, tabs, spaces) is stripped.
/// Records with empty sequences are dropped.
pub fn parse_fasta(text: &str) -> Vec<(String, String)> {
    let mut records = Vec::new();
    let mut header: Option<String> = None;
    let mut seq = String::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some(rest) = line.strip_prefix('>') {
            if let Some(h) = header.take() {
                if !seq.is_empty() {
                    records.push((h, std::mem::take(&mut seq)));
                } else {
                    seq.clear();
                }
            }
            header = Some(rest.trim().to_string());
        } else if header.is_some() {
            for c in line.chars() {
                if !c.is_whitespace() {
                    seq.push(c);
                }
            }
        }
    }
    if let Some(h) = header {
        if !seq.is_empty() {
            records.push((h, seq));
        }
    }
    records
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphabet_letters_dna() {
        assert_eq!(Alphabet::Dna.letters(), vec!['A', 'C', 'G', 'T']);
        assert_eq!(Alphabet::Dna.max_bits(), 2.0);
    }

    #[test]
    fn fasta_parses_wrapped_records() {
        let text = ">seq1\nACGT\nACGT\n>seq2\nTTTT\nAAAA\n";
        let records = parse_fasta(text);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].0, "seq1");
        assert_eq!(records[0].1, "ACGTACGT");
        assert_eq!(records[1].1, "TTTTAAAA");
    }

    #[test]
    fn from_sequences_counts_correctly() {
        let logo = SequenceLogo::from_sequences(["ACGT", "ACGT", "ACGA"], Alphabet::Dna).unwrap();
        assert_eq!(logo.n_positions(), 4);
        assert_eq!(logo.sample_size, Some(3));
        // Position 3: two T, one A.
        assert_eq!(logo.matrix[3][0], 1.0); // A
        assert_eq!(logo.matrix[3][3], 2.0); // T
    }

    #[test]
    fn from_sequences_rejects_ragged() {
        let err = SequenceLogo::from_sequences(["ACGT", "AC"], Alphabet::Dna).unwrap_err();
        assert!(err.contains("length"));
    }

    #[test]
    fn ic_at_fully_conserved_position() {
        // Fully conserved A at position 0, three sequences → IC ≈ 2 bits minus
        // Schneider correction e_n = (4-1) / (2 ln2 * 3) ≈ 0.7213.
        let logo = SequenceLogo::from_sequences(["A", "A", "A"], Alphabet::Dna).unwrap();
        let stack = logo.stack(0);
        // Only A is non-zero; height = 2 - 0 - e_n ≈ 1.2787.
        let a_height = stack.iter().find(|(c, _)| *c == 'A').unwrap().1;
        let expected = 2.0 - (3.0 / (2.0 * (2.0_f64).ln() * 3.0));
        assert!((a_height - expected).abs() < 1e-9, "got {}", a_height);
    }

    #[test]
    fn ic_clips_at_zero_for_uniform() {
        // Uniform 4 letters at position 0, only 2 samples → e_n = 3/(2 ln2 * 2) ≈ 1.08.
        // Raw IC = 2 - 2 = 0 → corrected = -1.08 → clipped to 0.
        let logo = SequenceLogo::from_sequences(["AC", "GT"], Alphabet::Dna).unwrap();
        let stack = logo.stack(0);
        let sum: f64 = stack.iter().map(|(_, h)| h).sum();
        assert!(sum <= 1e-9, "expected clip to 0, got sum {}", sum);
    }

    #[test]
    fn palette_preset_dna_classic() {
        let p = AlphabetPalette::preset(PalettePreset::Classic, &Alphabet::Dna);
        assert_eq!(p.color_of('A'), [0, 150, 0]);
        assert_eq!(p.color_of('T'), [200, 0, 0]);
    }

    #[test]
    fn palette_override() {
        let mut p = AlphabetPalette::preset(PalettePreset::Classic, &Alphabet::Dna);
        p.override_colors([('A', [10, 20, 30])]);
        assert_eq!(p.color_of('A'), [10, 20, 30]);
    }

    #[test]
    fn detect_orientation_from_letter_headers() {
        let alph = Alphabet::Dna.letters();
        let row: Vec<String> = vec!["A".into(), "C".into(), "G".into(), "T".into()];
        let col: Vec<String> = vec!["1".into(), "2".into(), "3".into()];
        assert_eq!(
            detect_orientation(&row, &col, &alph),
            Some(MatrixOrientation::RowsArePositions)
        );
    }

    #[test]
    fn reverse_complement_needs_nucleic() {
        let logo = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 5], Alphabet::Protein)
            .with_units(LogoUnits::Bits);
        assert!(logo.with_reverse_complement().is_err());
    }

    #[test]
    fn complement_is_alphabet_aware() {
        assert_eq!(Alphabet::Dna.complement('A'), 'T');
        assert_eq!(Alphabet::Rna.complement('A'), 'U');
        assert_eq!(Alphabet::Rna.complement('U'), 'A');
        assert_eq!(Alphabet::Protein.complement('A'), 'A');
    }

    #[test]
    fn respect_case_counts_lowercase_as_gap() {
        // Sequence 2 is fully soft-masked; with respect_case its letters drop.
        let logo = SequenceLogo::from_sequences(["ACGT", "acgt"], Alphabet::Dna)
            .unwrap()
            .with_respect_case();
        // Each position now counts only the uppercase sequence.
        for row in &logo.matrix {
            let total: f64 = row.iter().sum();
            assert_eq!(total, 1.0);
        }
    }

    #[test]
    fn include_gaps_adds_gap_column() {
        let logo = SequenceLogo::from_sequences(["A-", "AA"], Alphabet::Dna)
            .unwrap()
            .with_include_gaps();
        assert_eq!(*logo.letters.last().unwrap(), GAP_LETTER);
        // Position 1: one A, one gap.
        let gap_idx = logo.letters.len() - 1;
        assert_eq!(logo.matrix[1][gap_idx], 1.0);
        assert_eq!(logo.matrix[1][0], 1.0);
        // Gap column raises the IC ceiling to log2(5).
        assert!((logo.max_bits() - 5.0_f64.log2()).abs() < 1e-12);
    }

    #[test]
    fn gaps_excluded_by_default() {
        let logo = SequenceLogo::from_sequences(["A-", "AA"], Alphabet::Dna).unwrap();
        assert_eq!(logo.letters.len(), 4);
        // Position 1 counts only the single A; the gap is dropped.
        let total: f64 = logo.matrix[1].iter().sum();
        assert_eq!(total, 1.0);
    }

    #[test]
    fn legend_auto_by_alphabet() {
        let dna = SequenceLogo::from_sequences(["A"], Alphabet::Dna).unwrap();
        assert!(!dna.legend_visible());
        let protein = SequenceLogo::from_matrix(vec![vec![1.0; 20]], Alphabet::Protein);
        assert!(protein.legend_visible());
        let forced = SequenceLogo::from_sequences(["A"], Alphabet::Dna)
            .unwrap()
            .with_legend();
        assert!(forced.legend_visible());
    }
}
