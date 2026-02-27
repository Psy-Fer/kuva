use std::collections::HashMap;

/// A single intersection of sets.
///
/// Represents the subset of elements that belong to exactly the sets indicated
/// by `mask` — not more, not fewer. Bit `i` of `mask` being set means
/// `set_names[i]` participates in this intersection.
///
/// For example, with three sets A, B, C:
/// - `mask = 0b001` (1) → elements in A only
/// - `mask = 0b011` (3) → elements in A ∩ B but not C
/// - `mask = 0b111` (7) → elements in A ∩ B ∩ C
#[derive(Clone)]
pub struct UpSetIntersection {
    /// Bitmask: bit `i` is set if `set_names[i]` participates in this intersection.
    pub mask: u64,
    /// Number of elements belonging to exactly this combination of sets.
    pub count: usize,
}

/// Controls how intersections are ordered left-to-right in the UpSet plot.
#[derive(Clone, Default)]
pub enum UpSetSort {
    /// Sort by intersection count descending — the largest bar is leftmost (default).
    #[default]
    ByFrequency,
    /// Sort by degree (number of sets involved) descending, then by count within each degree.
    /// Puts the most complex multi-set intersections first.
    ByDegree,
    /// Preserve the order in which `intersections` were supplied.
    /// Useful with [`UpSetPlot::with_data`] when the caller controls the order.
    Natural,
}


/// Bioinformatics-style UpSet plot: vertical intersection-size bars, dot matrix,
/// and optional horizontal set-size bars.
///
/// An UpSet plot is the scalable successor to Venn diagrams for showing
/// set intersections when there are more than three or four sets. It has three
/// visual components:
///
/// - **Intersection size bars** (top): vertical bars showing how many elements
///   belong to each intersection.
/// - **Dot matrix** (middle): a grid of circles. Filled dots indicate which sets
///   participate in each intersection; a vertical line connects the filled dots.
/// - **Set size bars** (left, optional): horizontal bars showing the total size
///   of each set.
///
/// # Input modes
///
/// Two input modes are available:
///
/// - **Raw sets** ([`with_sets`](Self::with_sets)): provide `(name, elements)`
///   pairs. Intersection counts are computed automatically from element
///   membership. Elements can be any hashable type.
/// - **Precomputed** ([`with_data`](Self::with_data)): provide set names, total
///   set sizes, and `(mask, count)` pairs directly. Use this when intersection
///   counts come from an external source (e.g. a database query or enrichment
///   tool output).
///
/// # Pixel-space rendering
///
/// The UpSet plot renders entirely in pixel space. It does not use the standard
/// x/y axis system — `Layout::auto_from_plots` skips axis computation for UpSet
/// plots. A title set on the `Layout` is still rendered.
///
/// # Example
///
/// ```rust,no_run
/// use visus::plot::UpSetPlot;
/// use visus::backend::svg::SvgBackend;
/// use visus::render::render::render_multiple;
/// use visus::render::layout::Layout;
/// use visus::render::plots::Plot;
///
/// let up = UpSetPlot::new()
///     .with_sets(vec![
///         ("Set A", vec!["apple", "banana", "cherry", "date"]),
///         ("Set B", vec!["banana", "cherry", "elderberry", "fig"]),
///         ("Set C", vec!["cherry", "fig", "grape"]),
///     ]);
///
/// let plots = vec![Plot::UpSet(up)];
/// let layout = Layout::auto_from_plots(&plots)
///     .with_title("Set Intersections");
///
/// let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
/// std::fs::write("upset.svg", svg).unwrap();
/// ```
pub struct UpSetPlot {
    /// Names of the sets, one per row in the dot matrix.
    pub set_names: Vec<String>,
    /// Total number of elements in each set (used for the set-size bars).
    pub set_sizes: Vec<usize>,
    /// All non-empty intersections with their element counts.
    pub intersections: Vec<UpSetIntersection>,
    /// Ordering applied to intersections before rendering (default `ByFrequency`).
    pub sort: UpSetSort,
    /// When `Some(n)`, only the first `n` intersections after sorting are shown.
    pub max_visible: Option<usize>,
    /// Whether to show count labels above the intersection bars (default `true`).
    pub show_counts: bool,
    /// Whether to show the horizontal set-size bars on the left panel (default `true`).
    pub show_set_sizes: bool,
    /// Color for intersection bars and set-size bars (default `"#333333"`).
    pub bar_color: String,
    /// Fill color for dots in a participating set (default `"#333333"`).
    pub dot_color: String,
    /// Fill color for dots in a non-participating set (default `"#dddddd"`).
    pub dot_empty_color: String,
}

impl Default for UpSetPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl UpSetPlot {
    /// Create an UpSet plot with default settings.
    ///
    /// Defaults: `ByFrequency` sort, count labels shown, set-size bars shown,
    /// dark gray bars and dots, light gray empty dots.
    pub fn new() -> Self {
        Self {
            set_names: Vec::new(),
            set_sizes: Vec::new(),
            intersections: Vec::new(),
            sort: UpSetSort::ByFrequency,
            max_visible: None,
            show_counts: true,
            show_set_sizes: true,
            bar_color: "#333333".to_string(),
            dot_color: "#333333".to_string(),
            dot_empty_color: "#dddddd".to_string(),
        }
    }

    /// Build from raw sets: an iterable of `(name, elements)` pairs.
    ///
    /// Intersection counts are computed automatically. Each element is assigned
    /// to the unique combination of sets it belongs to (bitmask semantics), so
    /// an element present in sets A and C but not B is counted under the A∩C
    /// intersection — not under A, not under C, and not under A∩B∩C.
    ///
    /// Elements can be any type that implements `Eq + Hash`. Strings, integers,
    /// gene IDs, or any other discrete identifier all work.
    ///
    /// ```rust,no_run
    /// # use visus::plot::UpSetPlot;
    /// let up = UpSetPlot::new().with_sets(vec![
    ///     ("DESeq2",   vec!["BRCA1", "TP53", "EGFR", "MYC",  "CDK4"]),
    ///     ("edgeR",    vec!["TP53",  "EGFR", "RB1",  "PTEN", "CDK4"]),
    ///     ("limma",    vec!["EGFR",  "MYC",  "RB1",  "CDK2"]),
    /// ]);
    /// ```
    pub fn with_sets<S, T, I, J>(mut self, sets: I) -> Self
    where
        S: Into<String>,
        T: Eq + std::hash::Hash,
        I: IntoIterator<Item = (S, J)>,
        J: IntoIterator<Item = T>,
    {
        let named: Vec<(String, std::collections::HashSet<T>)> = sets
            .into_iter()
            .map(|(name, items)| (name.into(), items.into_iter().collect()))
            .collect();

        let n = named.len();
        self.set_names = named.iter().map(|(name, _)| name.clone()).collect();
        self.set_sizes = named.iter().map(|(_, s)| s.len()).collect();

        let mut mask_counts: HashMap<u64, usize> = HashMap::new();

        // Visit each element exactly once: count it when we first encounter it
        // (i.e., in the lowest-indexed set it belongs to).
        for (i, (_, set_i)) in named.iter().enumerate() {
            for elem in set_i.iter() {
                let already = named[..i].iter().any(|(_, sj)| sj.contains(elem));
                if already {
                    continue;
                }
                let mut mask: u64 = 0;
                for (j, (_, sj)) in named.iter().enumerate() {
                    if sj.contains(elem) {
                        mask |= 1u64 << j;
                    }
                }
                *mask_counts.entry(mask).or_insert(0) += 1;
            }
        }

        // Sort by mask for deterministic Natural order.
        let mut intersections: Vec<UpSetIntersection> = mask_counts
            .into_iter()
            .map(|(mask, count)| UpSetIntersection { mask, count })
            .collect();
        intersections.sort_by_key(|i| i.mask);
        self.intersections = intersections;
        let _ = n;
        self
    }

    /// Build from precomputed intersection data.
    ///
    /// Use this when intersection counts come from an external source —
    /// for example a database query, an enrichment analysis tool, or a
    /// summary statistics file — and you do not have or want to enumerate
    /// the individual elements.
    ///
    /// `intersections` is an iterator of `(mask, count)` pairs where `mask`
    /// is a bitmask indicating set membership (bit `i` set means `set_names[i]`
    /// participates) and `count` is the number of elements in that exact
    /// combination.
    ///
    /// `set_sizes` are the total element counts for each set, used to draw the
    /// optional set-size bars on the left panel.
    ///
    /// ```rust,no_run
    /// # use visus::plot::UpSetPlot;
    /// // Three variant callers — bit 0 = GATK, bit 1 = FreeBayes, bit 2 = Strelka
    /// let up = UpSetPlot::new().with_data(
    ///     ["GATK", "FreeBayes", "Strelka"],
    ///     [280usize, 263, 249],
    ///     vec![
    ///         (0b001, 45),   // GATK only
    ///         (0b010, 35),   // FreeBayes only
    ///         (0b100, 28),   // Strelka only
    ///         (0b011, 62),   // GATK ∩ FreeBayes
    ///         (0b101, 55),   // GATK ∩ Strelka
    ///         (0b110, 48),   // FreeBayes ∩ Strelka
    ///         (0b111, 118),  // all three
    ///     ],
    /// );
    /// ```
    pub fn with_data<S: Into<String>>(
        mut self,
        set_names: impl IntoIterator<Item = S>,
        set_sizes: impl IntoIterator<Item = usize>,
        intersections: impl IntoIterator<Item = (u64, usize)>,
    ) -> Self {
        self.set_names = set_names.into_iter().map(Into::into).collect();
        self.set_sizes = set_sizes.into_iter().collect();
        self.intersections = intersections
            .into_iter()
            .map(|(mask, count)| UpSetIntersection { mask, count })
            .collect();
        self
    }

    /// Set the sort order for intersections (default `ByFrequency`).
    ///
    /// - [`UpSetSort::ByFrequency`] — largest count leftmost (default). Best for
    ///   quickly seeing which intersections are most common.
    /// - [`UpSetSort::ByDegree`] — most complex (most sets) leftmost, ties broken
    ///   by count. Useful for emphasising high-order intersections.
    /// - [`UpSetSort::Natural`] — preserve the order supplied to `with_data` or
    ///   the deterministic mask-sorted order from `with_sets`.
    pub fn with_sort(mut self, sort: UpSetSort) -> Self {
        self.sort = sort;
        self
    }

    /// Show only the top `max` intersections after sorting.
    ///
    /// Useful when many intersections exist but only the most prominent ones
    /// need to be displayed. The limit is applied after sorting, so "top"
    /// means the first `max` in the chosen sort order.
    ///
    /// ```rust,no_run
    /// # use visus::plot::UpSetPlot;
    /// let up = UpSetPlot::new()
    ///     .with_sets(vec![("A", vec![1,2,3]), ("B", vec![2,3,4]), ("C", vec![3,4,5])])
    ///     .with_max_visible(3);  // show only the 3 largest intersections
    /// ```
    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = Some(max);
        self
    }

    /// Hide the horizontal set-size bars on the left panel.
    ///
    /// By default a bar for each set shows its total element count. Call this
    /// to produce a more compact layout that focuses solely on the intersections.
    pub fn without_set_sizes(mut self) -> Self {
        self.show_set_sizes = false;
        self
    }

    /// Set the color for intersection bars and set-size bars (default `"#333333"`).
    ///
    /// Accepts any CSS color string.
    pub fn with_bar_color<S: Into<String>>(mut self, color: S) -> Self {
        self.bar_color = color.into();
        self
    }

    /// Set the fill color for dots in a participating set (default `"#333333"`).
    ///
    /// Non-participating dots retain their color set by [`dot_empty_color`](Self::dot_empty_color)
    /// (default light gray). Accepts any CSS color string.
    pub fn with_dot_color<S: Into<String>>(mut self, color: S) -> Self {
        self.dot_color = color.into();
        self
    }

    /// Returns references to intersections sorted and trimmed for rendering.
    pub fn sorted_intersections(&self) -> Vec<&UpSetIntersection> {
        let mut sorted: Vec<&UpSetIntersection> = self.intersections.iter().collect();
        match self.sort {
            UpSetSort::ByFrequency => {
                sorted.sort_by(|a, b| b.count.cmp(&a.count));
            }
            UpSetSort::ByDegree => {
                sorted.sort_by(|a, b| {
                    let da = a.mask.count_ones();
                    let db = b.mask.count_ones();
                    db.cmp(&da).then(b.count.cmp(&a.count))
                });
            }
            UpSetSort::Natural => {}
        }
        if let Some(max) = self.max_visible {
            sorted.truncate(max);
        }
        sorted
    }
}
