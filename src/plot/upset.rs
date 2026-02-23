use std::collections::HashMap;

/// A single intersection of sets.
#[derive(Clone)]
pub struct UpSetIntersection {
    /// Bitmask: bit `i` set if `set_names[i]` is in this intersection.
    pub mask: u64,
    /// Number of elements belonging to exactly this combination of sets.
    pub count: usize,
}

/// Controls how intersections are ordered left-to-right in the UpSet plot.
#[derive(Clone)]
pub enum UpSetSort {
    /// Sort by intersection count descending (default).
    ByFrequency,
    /// Sort by degree (number of sets involved) descending, then by count.
    ByDegree,
    /// Preserve the order of `intersections` as supplied.
    Natural,
}

impl Default for UpSetSort {
    fn default() -> Self {
        UpSetSort::ByFrequency
    }
}

/// Bioinformatics-style UpSet plot: vertical intersection-size bars, dot matrix, and
/// optional horizontal set-size bars.
pub struct UpSetPlot {
    pub set_names: Vec<String>,
    pub set_sizes: Vec<usize>,
    pub intersections: Vec<UpSetIntersection>,
    pub sort: UpSetSort,
    /// Show only the top N intersections after sorting.
    pub max_visible: Option<usize>,
    /// Show count labels above intersection bars.
    pub show_counts: bool,
    /// Show horizontal set-size bars on the left panel.
    pub show_set_sizes: bool,
    /// Color for intersection bars and set-size bars.
    pub bar_color: String,
    /// Fill color for dots in a participating set.
    pub dot_color: String,
    /// Fill color for dots in a non-participating set.
    pub dot_empty_color: String,
}

impl Default for UpSetPlot {
    fn default() -> Self {
        Self::new()
    }
}

impl UpSetPlot {
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

    /// Build from raw sets: an iterable of `(set_name, elements)` pairs.
    ///
    /// Computes exclusive intersections (bitmask semantics): an element with
    /// membership mask M is counted under exactly that mask.
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

    /// Provide precomputed intersections as `(mask, count)` pairs alongside
    /// set names and total set sizes.
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

    pub fn with_sort(mut self, sort: UpSetSort) -> Self {
        self.sort = sort;
        self
    }

    pub fn with_max_visible(mut self, max: usize) -> Self {
        self.max_visible = Some(max);
        self
    }

    pub fn without_set_sizes(mut self) -> Self {
        self.show_set_sizes = false;
        self
    }

    pub fn with_bar_color<S: Into<String>>(mut self, color: S) -> Self {
        self.bar_color = color.into();
        self
    }

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
