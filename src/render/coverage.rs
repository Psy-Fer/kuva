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
use crate::render::color::Color;
use crate::render::palette::Palette;
use crate::render::plots::Plot;
use crate::render::render::Scene;
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
    locus: Option<(f64, f64)>,
    x_label: String,
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
            locus: None,
            x_label: "position".to_string(),
        }
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

    /// Name for the below-axis feature lane (default `"features"`).
    pub fn with_feature_track_name(mut self, name: impl Into<String>) -> Self {
        self.feature_track_name = name.into();
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
    /// Layout: one filled-area depth `PlotTrack` per sample (above the axis), then a variant lane
    /// if any, then the genomic x-axis, then the feature bands below it.
    pub fn build(self) -> TrackStack {
        let palette = Palette::wong();
        let colors = palette.colors();

        let mut stack = TrackStack::new();
        if let Some((lo, hi)) = self.locus {
            stack = stack.x_range(lo, hi);
        }

        for (i, (name, depth)) in self.samples.into_iter().enumerate() {
            let color = colors[i % colors.len()].clone();
            let line = LinePlot::new()
                .with_data(depth)
                .with_fill()
                .with_fill_opacity(0.6)
                .with_color(color);
            stack = stack.track(PlotTrack::new(vec![Plot::Line(line)]).with_y_label(name));
        }

        if !self.variant_groups.is_empty() {
            let mut vt = VariantTrack::new().with_name("variants");
            for g in self.variant_groups {
                vt = vt.with_group(g.label, g.color, g.positions);
            }
            stack = stack.track(vt);
        }

        stack = stack.x_axis_with(AxisSpec::genomic(self.x_label));

        if !self.features.is_empty() {
            stack = stack.track(
                IntervalTrack::new(self.features).with_name(self.feature_track_name.clone()),
            );
        }

        stack
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

        // Sample names appear as y-axis labels.
        assert_eq!(count_text(&scene, "tumour"), 1);
        assert_eq!(count_text(&scene, "normal"), 1);
        // Variant types appear in the shared legend.
        assert_eq!(count_text(&scene, "SNV"), 1);
        assert_eq!(count_text(&scene, "InDel"), 1);
        // Feature bands + their gutter track-name.
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
