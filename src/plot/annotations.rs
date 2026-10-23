//! Annotation *data* types (reference lines, shaded regions, text callouts) shared by the
//! plot and render layers. These are pure configuration structs (no render dependencies), so
//! they live in `src/plot/` per the one-way `plot → render` architecture rule. The drawing
//! functions that consume them stay in [`crate::render::annotations`], which re-exports these
//! types for backward compatibility.

#[derive(Clone, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct TextAnnotation {
    pub text: String,
    pub text_x: f64,
    pub text_y: f64,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub font_size: u32,
    pub color: String,
    pub arrow_padding: f64,
}

impl TextAnnotation {
    pub fn new<S: Into<String>>(text: S, x: f64, y: f64) -> Self {
        Self {
            text: text.into(),
            text_x: x,
            text_y: y,
            target_x: None,
            target_y: None,
            font_size: 12,
            color: "black".into(),
            arrow_padding: 6.0,
        }
    }

    pub fn with_arrow(mut self, target_x: f64, target_y: f64) -> Self {
        self.target_x = Some(target_x);
        self.target_y = Some(target_y);
        self
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_arrow_padding(mut self, padding: f64) -> Self {
        self.arrow_padding = padding;
        self
    }
}

#[derive(Clone, Debug)]
pub struct ReferenceLine {
    pub value: f64,
    pub orientation: Orientation,
    pub color: String,
    pub stroke_width: f64,
    pub dasharray: String,
    pub label: Option<String>,
}

impl ReferenceLine {
    pub fn horizontal(y: f64) -> Self {
        Self {
            value: y,
            orientation: Orientation::Horizontal,
            color: "red".into(),
            stroke_width: 1.0,
            dasharray: "6 4".into(),
            label: None,
        }
    }

    pub fn vertical(x: f64) -> Self {
        Self {
            value: x,
            orientation: Orientation::Vertical,
            color: "red".into(),
            stroke_width: 1.0,
            dasharray: "6 4".into(),
            label: None,
        }
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_dasharray<S: Into<String>>(mut self, dash: S) -> Self {
        self.dasharray = dash.into();
        self
    }
}

#[derive(Clone)]
pub struct ShadedRegion {
    pub orientation: Orientation,
    pub min_val: f64,
    pub max_val: f64,
    pub color: String,
    pub opacity: f64,
}

impl ShadedRegion {
    pub fn horizontal(y_min: f64, y_max: f64) -> Self {
        Self {
            orientation: Orientation::Horizontal,
            min_val: y_min,
            max_val: y_max,
            color: "blue".into(),
            opacity: 0.15,
        }
    }

    pub fn vertical(x_min: f64, x_max: f64) -> Self {
        Self {
            orientation: Orientation::Vertical,
            min_val: x_min,
            max_val: x_max,
            color: "blue".into(),
            opacity: 0.15,
        }
    }

    pub fn with_color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }
}

// ── Significance brackets ────────────────────────────────────────────────────

/// One end of a [`SignificanceBracket`]: which group it sits over, and how that
/// group is located on the category axis at render time.
#[derive(Clone, Debug, PartialEq)]
pub enum BracketAnchor {
    /// Zero-based category index. Bar, box, violin, strip and raincloud plots
    /// all place category `i` at data coordinate `i + 1`, so this needs no
    /// category list to resolve.
    Index(usize),
    /// Category label, resolved against the layout's `x_categories` (or
    /// `y_categories` for a horizontal bracket) when the plot is rendered.
    /// Falls back to no bracket, not a panic, if the label is unknown.
    Label(String),
    /// Raw coordinate on the category axis, for numeric axes or for a
    /// position between two categories.
    Value(f64),
}

impl From<usize> for BracketAnchor {
    fn from(i: usize) -> Self {
        BracketAnchor::Index(i)
    }
}

impl From<&str> for BracketAnchor {
    fn from(s: &str) -> Self {
        BracketAnchor::Label(s.to_string())
    }
}

impl From<String> for BracketAnchor {
    fn from(s: String) -> Self {
        BracketAnchor::Label(s)
    }
}

impl From<f64> for BracketAnchor {
    fn from(v: f64) -> Self {
        BracketAnchor::Value(v)
    }
}

/// How a [`SignificanceBracket`] turns its p-value into the text above the bar.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum BracketLabel {
    /// Star code: `****` below the fourth threshold down to `*` below the
    /// first, and `ns` otherwise. Thresholds come from
    /// [`SignificanceBracket::thresholds`].
    #[default]
    Stars,
    /// Numeric p-value: `p = 0.023`, or `p < 0.0001` below that.
    PValue,
    /// Stars followed by the numeric value in parentheses: `** (p = 0.0031)`.
    Both,
    /// Text supplied verbatim; the p-value, if any, is ignored.
    Text(String),
}

/// A bracket spanning two groups on a categorical axis, with a significance
/// label above it — the `stat_compare_means` / `statannotations` figure idiom.
///
/// Brackets are a [`Layout`](crate::render::layout::Layout) feature, like
/// [`TextAnnotation`] and [`ReferenceLine`], so they work on any plot whose
/// categories sit at `i + 1` on one axis: bar, box, violin, strip, raincloud.
/// The p-value is supplied, not computed — kuva draws the result of a test
/// you ran elsewhere.
///
/// ```
/// use kuva::plot::annotations::SignificanceBracket;
/// use kuva::render::layout::Layout;
///
/// let layout = Layout::new((0.5, 3.5), (0.0, 10.0))
///     .with_x_categories(vec!["WT".into(), "KO".into(), "Rescue".into()])
///     // category labels, auto-stacked above the data
///     .with_bracket(SignificanceBracket::new("WT", "KO", 0.0007))
///     // zero-based indices, numeric label, explicit height
///     .with_bracket(
///         SignificanceBracket::new(0usize, 2usize, 0.03)
///             .with_p_value_label()
///             .with_position(9.5),
///     );
/// assert_eq!(layout.brackets[0].label(), "***");
/// assert_eq!(layout.brackets[1].label(), "p = 0.030");
/// ```
#[derive(Clone, Debug)]
pub struct SignificanceBracket {
    pub a: BracketAnchor,
    pub b: BracketAnchor,
    /// The p-value the label is formatted from. `None` only for a verbatim
    /// [`BracketLabel::Text`] label.
    pub p_value: Option<f64>,
    pub format: BracketLabel,
    /// Explicit position on the value axis, in data units. `None` stacks the
    /// bracket automatically above the data, and the axis grows to fit it.
    pub position: Option<f64>,
    /// Force the bracket to run along the y axis (categories on y), drawn
    /// vertically to the right of the data with its label beside it. Normally
    /// inferred from the layout — a plot with `y_categories` and no
    /// `x_categories` is horizontal — so this is only an override.
    pub horizontal: bool,
    /// Stroke and label colour. `None` follows the theme's text colour.
    pub color: Option<String>,
    pub stroke_width: f64,
    /// Label font size in px. `None` uses the layout's body size.
    pub font_size: Option<u32>,
    /// Star thresholds: `p` earns one star per threshold it is below, so the
    /// order does not matter. Default `[0.05, 0.01, 0.001, 0.0001]`.
    pub thresholds: [f64; 4],
}

impl SignificanceBracket {
    /// Bracket between groups `a` and `b` labelled with the star code for `p`.
    /// Anchors may be category labels (`&str`), zero-based indices (`usize`),
    /// or raw axis values (`f64`).
    pub fn new(a: impl Into<BracketAnchor>, b: impl Into<BracketAnchor>, p: f64) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
            p_value: Some(p),
            format: BracketLabel::Stars,
            position: None,
            horizontal: false,
            color: None,
            stroke_width: 1.0,
            font_size: None,
            thresholds: [0.05, 0.01, 0.001, 0.0001],
        }
    }

    /// Bracket between `a` and `b` carrying `text` verbatim (e.g. `"n.s."`,
    /// `"q = 0.02"`, or a test name).
    pub fn text(
        a: impl Into<BracketAnchor>,
        b: impl Into<BracketAnchor>,
        text: impl Into<String>,
    ) -> Self {
        let mut s = Self::new(a, b, 1.0);
        s.p_value = None;
        s.format = BracketLabel::Text(text.into());
        s
    }

    /// Label with the star code (default).
    pub fn with_stars(mut self) -> Self {
        self.format = BracketLabel::Stars;
        self
    }

    /// Label with the numeric p-value, `p = 0.023`.
    pub fn with_p_value_label(mut self) -> Self {
        self.format = BracketLabel::PValue;
        self
    }

    /// Label with stars and the numeric value, `** (p = 0.0031)`.
    pub fn with_both_labels(mut self) -> Self {
        self.format = BracketLabel::Both;
        self
    }

    /// Replace the formatted label with `text`.
    pub fn with_label(mut self, text: impl Into<String>) -> Self {
        self.format = BracketLabel::Text(text.into());
        self
    }

    /// Pin the bracket at `pos` on the value axis (data units) instead of
    /// auto-stacking it above the data.
    pub fn with_position(mut self, pos: f64) -> Self {
        self.position = Some(pos);
        self
    }

    /// Force a vertical bracket (categories along y). Only needed when the
    /// layout cannot tell — orientation is inferred from `y_categories` /
    /// `x_categories` otherwise.
    pub fn with_horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Stroke and label colour (CSS). The default follows the theme's text
    /// colour.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn with_font_size(mut self, size: u32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Star thresholds: `p` earns one star per threshold it is below, in any
    /// order. Default `[0.05, 0.01, 0.001, 0.0001]`.
    pub fn with_thresholds(mut self, thresholds: [f64; 4]) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// The text drawn above the bracket.
    pub fn label(&self) -> String {
        match (&self.format, self.p_value) {
            (BracketLabel::Text(t), _) => t.clone(),
            (_, None) => String::new(),
            (BracketLabel::Stars, Some(p)) => stars(p, &self.thresholds).to_string(),
            (BracketLabel::PValue, Some(p)) => format_p_value(p),
            (BracketLabel::Both, Some(p)) => {
                format!("{} ({})", stars(p, &self.thresholds), format_p_value(p))
            }
        }
    }

    /// Whether the label is a bare star code (drawn tighter to the bar, since
    /// asterisks have no descenders and sit high in the em box).
    pub fn label_is_stars(&self) -> bool {
        matches!(self.format, BracketLabel::Stars) && self.p_value.is_some()
    }
}

/// Star code for `p`: one star per threshold `p` lies below (so the order of
/// `thresholds` does not matter), `ns` when it clears them all.
pub fn stars(p: f64, thresholds: &[f64; 4]) -> &'static str {
    const CODES: [&str; 5] = ["ns", "*", "**", "***", "****"];
    CODES[thresholds.iter().filter(|t| p < **t).count()]
}

/// `p = 0.023` style text: three decimals from 0.001 up, four below that, and
/// `p < 0.0001` under the fourth decimal.
pub fn format_p_value(p: f64) -> String {
    if p < 0.0001 {
        "p < 0.0001".to_string()
    } else if p < 0.001 {
        format!("p = {p:.4}")
    } else {
        format!("p = {p:.3}")
    }
}

/// Assign a stacking level to every auto-positioned bracket so that no two
/// brackets on the same level overlap or touch along the category axis.
///
/// `spans` holds each bracket's resolved `(lo, hi)` interval on the category
/// axis, or `None` for a bracket that is pinned with an explicit position (or
/// could not be resolved) and takes no part in stacking. Narrow brackets are
/// placed first so they sit lowest; the result is indexed like `spans`.
pub fn stack_levels(spans: &[Option<(f64, f64)>]) -> Vec<Option<usize>> {
    let mut order: Vec<usize> = spans
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.map(|_| i))
        .collect();
    order.sort_by(|&i, &j| {
        let wi = spans[i].map(|(lo, hi)| hi - lo).unwrap_or(0.0);
        let wj = spans[j].map(|(lo, hi)| hi - lo).unwrap_or(0.0);
        wi.partial_cmp(&wj)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(i.cmp(&j))
    });

    let mut levels: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut out = vec![None; spans.len()];
    for i in order {
        let (lo, hi) = spans[i].expect("filtered to Some above");
        let level = levels
            .iter()
            .position(|placed| placed.iter().all(|&(plo, phi)| hi < plo || lo > phi))
            .unwrap_or_else(|| {
                levels.push(Vec::new());
                levels.len() - 1
            });
        levels[level].push((lo, hi));
        out[i] = Some(level);
    }
    out
}

#[cfg(test)]
mod bracket_tests {
    use super::*;

    #[test]
    fn stars_follow_thresholds() {
        let t = [0.05, 0.01, 0.001, 0.0001];
        assert_eq!(stars(0.2, &t), "ns");
        assert_eq!(stars(0.05, &t), "ns");
        assert_eq!(stars(0.049, &t), "*");
        assert_eq!(stars(0.009, &t), "**");
        assert_eq!(stars(0.0009, &t), "***");
        assert_eq!(stars(0.00009, &t), "****");
    }

    #[test]
    fn threshold_order_does_not_matter() {
        let asc = [0.0001, 0.001, 0.01, 0.05];
        let desc = [0.05, 0.01, 0.001, 0.0001];
        for p in [0.2, 0.02, 0.002, 0.0002, 0.00002] {
            assert_eq!(stars(p, &asc), stars(p, &desc), "p = {p}");
        }
        assert_eq!(stars(0.02, &asc), "*");
    }

    #[test]
    fn p_value_text() {
        assert_eq!(format_p_value(0.5), "p = 0.500");
        assert_eq!(format_p_value(0.0231), "p = 0.023");
        assert_eq!(format_p_value(0.00031), "p = 0.0003");
        assert_eq!(format_p_value(0.00001), "p < 0.0001");
    }

    #[test]
    fn labels_by_format() {
        let b = SignificanceBracket::new(0usize, 1usize, 0.0031);
        assert_eq!(b.label(), "**");
        assert_eq!(b.clone().with_p_value_label().label(), "p = 0.003");
        assert_eq!(b.clone().with_both_labels().label(), "** (p = 0.003)");
        assert_eq!(b.with_label("n.s.").label(), "n.s.");
        assert_eq!(
            SignificanceBracket::text("a", "b", "q = 0.1").label(),
            "q = 0.1"
        );
    }

    #[test]
    fn stacking_separates_overlapping_and_touching_spans() {
        // A–B and B–C touch at B: different levels. A–C spans both: on top.
        let spans = [Some((1.0, 2.0)), Some((2.0, 3.0)), Some((1.0, 3.0)), None];
        let lv = stack_levels(&spans);
        assert_eq!(lv[0], Some(0));
        assert_eq!(lv[1], Some(1));
        assert_eq!(lv[2], Some(2));
        assert_eq!(lv[3], None);
    }

    #[test]
    fn stacking_shares_a_level_for_disjoint_spans() {
        let spans = [Some((1.0, 2.0)), Some((3.0, 4.0))];
        assert_eq!(stack_levels(&spans), vec![Some(0), Some(0)]);
    }
}
