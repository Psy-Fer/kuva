//! `CoveragePlot` — the genomics preset that closes issue #2, built entirely on top of the
//! generic [`TrackStack`](crate::render::track_stack) primitive.
//!
//! It is deliberately NOT a `Plot` enum variant: it's a thin builder that assembles a
//! `TrackStack` from sequencing-depth samples, typed variant marks, and below-axis feature bands
//! (amplicons / primers / genes), with a genomic x-axis. Everything it does is expressible with
//! the public `TrackStack` API — [`CoveragePlot::build`] hands back that stack so callers can
//! further customise it (add a [`RegionHighlight`](crate::render::track_stack::RegionHighlight)
//! underlay, extra tracks, etc.) before rendering.

use crate::plot::LinePlot;
use crate::render::annotations::ReferenceLine;
use crate::render::color::Color;
use crate::render::palette::Palette;
use crate::render::plots::Plot;
use crate::render::render::Scene;
use crate::render::theme::Theme;
use crate::render::track_stack::{
    AxisSpec, Interval, IntervalTrack, PlotTrack, TrackStack, VariantTrack,
};

/// One coloured, labelled group of variant positions (e.g. SNVs, InDels).
struct VariantGroup {
    label: String,
    color: Color,
    positions: Vec<f64>,
}

/// Builder for a multi-sample sequencing-coverage figure.
///
/// ```ignore
/// let scene = CoveragePlot::new()
///     .with_locus(1_000_000.0, 1_050_000.0)
///     .with_sample("tumour", tumour_depth)   // Vec<(pos, depth)>
///     .with_sample("normal", normal_depth)
///     .with_variants("SNV", "#d1495b", snv_positions)
///     .with_variants("InDel", "#e9c46a", indel_positions)
///     .with_feature(1_010_000.0, 1_020_000.0, "amplicon_3")
///     .render(1000.0);
/// ```
pub struct CoveragePlot {
    samples: Vec<(String, Vec<(f64, f64)>)>,
    variant_groups: Vec<VariantGroup>,
    features: Vec<Interval>,
    feature_track_name: String,
    regions: Vec<Interval>,
    region_track_name: String,
    locus: Option<(f64, f64)>,
    x_label: String,
    theme: Option<Theme>,
    title: Option<String>,
    overlay_samples: bool,
    depth_thresholds: Vec<ReferenceLine>,
}

impl Default for CoveragePlot {
    fn default() -> Self {
        Self::new()
    }
}

impl CoveragePlot {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            variant_groups: Vec::new(),
            features: Vec::new(),
            feature_track_name: "features".to_string(),
            regions: Vec::new(),
            region_track_name: "genes".to_string(),
            locus: None,
            x_label: "position".to_string(),
            theme: None,
            title: None,
            overlay_samples: false,
            depth_thresholds: Vec::new(),
        }
    }

    /// Draw a dashed horizontal reference line across every depth track at `depth`, e.g. a
    /// minimum-coverage threshold. Call more than once for multiple thresholds.
    pub fn with_coverage_threshold(mut self, depth: f64) -> Self {
        self.depth_thresholds.push(ReferenceLine::horizontal(depth));
        self
    }

    /// Draw a labelled coverage threshold line at `depth`.
    pub fn with_coverage_threshold_labeled(mut self, depth: f64, label: impl Into<String>) -> Self {
        self.depth_thresholds
            .push(ReferenceLine::horizontal(depth).with_label(label));
        self
    }

    /// Visual theme for the whole figure (default light).
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Draw all sample depths overlaid in ONE shared track (each a coloured filled line on a common
    /// y-axis) instead of one stacked track per sample. Useful for directly comparing pools/samples.
    pub fn with_overlaid_samples(mut self) -> Self {
        self.overlay_samples = true;
        self
    }

    /// Figure title, drawn centred above the tracks.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add one sample's depth track: `(genomic position, depth)` pairs. Rendered as a filled-area
    /// depth track, one per sample, coloured from a colourblind-safe palette and named on its y-axis.
    pub fn with_sample(mut self, name: impl Into<String>, depth: Vec<(f64, f64)>) -> Self {
        self.samples.push((name.into(), depth));
        self
    }

    /// Add a typed set of variant positions (all groups share one variant lane; each contributes a
    /// legend entry). Call once per type.
    pub fn with_variants(
        mut self,
        label: impl Into<String>,
        color: impl Into<Color>,
        positions: Vec<f64>,
    ) -> Self {
        self.variant_groups.push(VariantGroup {
            label: label.into(),
            color: color.into(),
            positions,
        });
        self
    }

    /// Add a below-axis feature band (amplicon / primer / gene).
    pub fn with_feature(mut self, start: f64, end: f64, label: impl Into<String>) -> Self {
        self.features
            .push(Interval::new(start, end).with_label(label));
        self
    }

    /// Add several feature bands at once.
    pub fn with_features(mut self, features: Vec<Interval>) -> Self {
        self.features.extend(features);
        self
    }

    /// Name for the feature lane (default `"features"`).
    pub fn with_feature_track_name(mut self, name: impl Into<String>) -> Self {
        self.feature_track_name = name.into();
        self
    }

    /// Add a labelled genome region (gene / ORF / annotation), drawn in a lane just above the axis.
    /// Regions are a *reference* annotation of the coordinate (distinct from `--features`/amplicons,
    /// which annotate the assay). Labels that don't fit their band are omitted (kept as bare boxes).
    pub fn with_region(mut self, start: f64, end: f64, label: impl Into<String>) -> Self {
        self.regions
            .push(Interval::new(start, end).with_label(label));
        self
    }

    /// Add several genome regions at once.
    pub fn with_regions(mut self, regions: Vec<Interval>) -> Self {
        self.regions.extend(regions);
        self
    }

    /// Name for the region/gene lane (default `"genes"`).
    pub fn with_region_track_name(mut self, name: impl Into<String>) -> Self {
        self.region_track_name = name.into();
        self
    }

    /// Pin the genomic locus (x-range). Without it, the union of all supplied data is used.
    pub fn with_locus(mut self, start: f64, end: f64) -> Self {
        self.locus = Some((start, end));
        self
    }

    /// X-axis label (default `"position"`).
    pub fn with_x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = label.into();
        self
    }

    /// Assemble the underlying [`TrackStack`] without rendering — an escape hatch for callers who
    /// want to add underlays / extra tracks before `render`.
    ///
    /// Layout, top to bottom: title (if any), one filled-area depth `PlotTrack` per sample, a
    /// variant lane (if any), the feature-band lane (amplicons tile onto rows when they overlap,
    /// coloured by pool), then the genomic x-axis at the bottom. Features sit *above* the axis
    /// (with the data), not below it.
    pub fn build(self) -> TrackStack {
        // Two-colour pool palette for tiled feature rows (e.g. ARTIC's alternating primer pools).
        // Ordered warm-then-blue to match the sample depth tracks, which take the Wong palette
        // (colors[0] = orange, colors[1] = sky blue): the first pool's amplicons then read the same
        // hue as the first sample's coverage, and likewise for the second.
        const POOL_COLORS: [&str; 2] = ["#ffb703", "#8ecae6"];
        // A 12-colour categorical palette for per-gene colouring of the region track.
        const GENE_COLORS: [&str; 12] = [
            "#4e79a7", "#f28e2b", "#59a14f", "#e15759", "#76b7b2", "#edc948", "#b07aa1", "#ff9da7",
            "#9c755f", "#bab0ac", "#86bcb6", "#d37295",
        ];

        let palette = Palette::wong();
        let colors = palette.colors();

        let mut stack = TrackStack::new();
        if let Some(theme) = self.theme {
            stack = stack.with_theme(theme);
        }
        if let Some(title) = self.title {
            stack = stack.with_title(title);
        }
        if let Some((lo, hi)) = self.locus {
            stack = stack.x_range(lo, hi);
        }

        let line = |name: String, depth: Vec<(f64, f64)>, color: String, opacity: f64| {
            // `with_legend` makes each sample a coloured entry in the shared legend.
            Plot::Line(
                LinePlot::new()
                    .with_data(depth)
                    .with_fill()
                    .with_fill_opacity(opacity)
                    .with_color(color)
                    .with_legend(name),
            )
        };
        // Coverage-threshold lines (e.g. minimum depth) applied to every depth track.
        let thresholds = self.depth_thresholds;
        let apply_thresholds = |mut pt: PlotTrack| -> PlotTrack {
            for line in &thresholds {
                pt = pt.with_reference_line(line.clone());
            }
            pt
        };
        if self.overlay_samples {
            // All samples overlaid in one shared "depth" track on a common y-axis.
            let plots: Vec<Plot> = self
                .samples
                .into_iter()
                .enumerate()
                .map(|(i, (name, depth))| line(name, depth, colors[i % colors.len()].clone(), 0.5))
                .collect();
            if !plots.is_empty() {
                stack = stack.track(apply_thresholds(
                    PlotTrack::new(plots).with_y_label("depth"),
                ));
            }
        } else {
            // One stacked track per sample; the y-axis label names it.
            for (i, (name, depth)) in self.samples.into_iter().enumerate() {
                let color = colors[i % colors.len()].clone();
                let plot = line(name.clone(), depth, color, 0.6);
                stack = stack.track(apply_thresholds(
                    PlotTrack::new(vec![plot]).with_y_label(name),
                ));
            }
        }

        if !self.variant_groups.is_empty() {
            let mut vt = VariantTrack::new().with_name("variants");
            for g in self.variant_groups {
                vt = vt.with_group(g.label, g.color, g.positions);
            }
            stack = stack.track(vt);
        }

        // Feature bands go ABOVE the axis, tiled onto pool-coloured rows when they overlap.
        if !self.features.is_empty() {
            stack = stack.track(
                IntervalTrack::new(self.features)
                    .with_name(self.feature_track_name.clone())
                    .with_row_colors(POOL_COLORS.to_vec()),
            );
        }

        // Genome regions (genes/ORFs) sit just above the axis — the reference annotation closest to
        // the coordinate. Per-gene colours + a titled legend section, so even the tiny 3' ORFs
        // (whose labels don't fit inside their band) stay identifiable by colour.
        if !self.regions.is_empty() {
            stack = stack.track(
                IntervalTrack::new(self.regions)
                    .with_name(self.region_track_name.clone())
                    .with_item_colors(GENE_COLORS.to_vec()),
            );
        }

        stack.x_axis_with(AxisSpec::genomic(self.x_label))
    }

    /// Render at the given width with an auto total height.
    pub fn render(self, width: f64) -> Scene {
        self.build().render(width)
    }

    /// Render at an explicit width and height.
    pub fn render_sized(self, width: f64, height: f64) -> Scene {
        self.build().render_sized(width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::render::Primitive;

    fn count_text(scene: &Scene, needle: &str) -> usize {
        scene
            .elements
            .iter()
            .filter(|p| matches!(p, Primitive::Text { content, .. } if content == needle))
            .count()
    }

    fn depth(n: usize) -> Vec<(f64, f64)> {
        (0..=n)
            .map(|i| {
                let x = 1_000_000.0 + i as f64 / n as f64 * 200_000.0;
                (x, (i as f64).sin().abs() * 100.0 + 10.0)
            })
            .collect()
    }

    #[test]
    fn assembles_samples_variants_and_features() {
        let scene = CoveragePlot::new()
            .with_locus(1_000_000.0, 1_200_000.0)
            .with_sample("tumour", depth(60))
            .with_sample("normal", depth(60))
            .with_variants("SNV", "#d1495b", vec![1_050_000.0, 1_150_000.0])
            .with_variants("InDel", "#e9c46a", vec![1_090_000.0])
            .with_feature(1_020_000.0, 1_080_000.0, "amp1")
            .with_feature(1_110_000.0, 1_170_000.0, "amp2")
            .render(1000.0);

        // Sample names appear twice: the y-axis label AND a shared-legend entry (untitled samples
        // section).
        assert_eq!(count_text(&scene, "tumour"), 2);
        assert_eq!(count_text(&scene, "normal"), 2);
        // Variant types appear in the shared legend.
        assert_eq!(count_text(&scene, "SNV"), 1);
        assert_eq!(count_text(&scene, "InDel"), 1);
        // Feature bands (labelled inline) + their gutter track-name.
        assert_eq!(count_text(&scene, "amp1"), 1);
        assert_eq!(count_text(&scene, "amp2"), 1);
        assert_eq!(count_text(&scene, "features"), 1);
        // Genomic axis: unit is chosen from absolute coordinate magnitude (~1.2 Mb here), so a
        // 200 kb window at Mb-scale coordinates labels ticks in Mb ("1 Mb", "1.05 Mb", ...).
        assert!(scene
            .elements
            .iter()
            .any(|p| matches!(p, Primitive::Text { content, .. } if content.ends_with(" Mb"))));
    }

    #[test]
    fn empty_is_harmless() {
        let scene = CoveragePlot::new().with_locus(0.0, 100.0).render(400.0);
        assert!(scene.width == 400.0);
    }

    #[test]
    fn build_exposes_stack_for_customisation() {
        // `build()` returns a TrackStack the caller can extend (e.g. add an underlay) before render.
        let scene = CoveragePlot::new()
            .with_locus(0.0, 1000.0)
            .with_sample("s1", vec![(0.0, 1.0), (500.0, 5.0), (1000.0, 2.0)])
            .build()
            .render(600.0);
        assert!(!scene.elements.is_empty());
    }
}
