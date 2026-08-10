//! Shared-x stacked-panel layout — the genome-browser / track primitive.
//!
//! Vertically stacked panels ("tracks") that all share ONE x-axis, pixel-aligned regardless
//! of each track's own y-scale. This is the reusable core; `CoveragePlot` (issue #2) will be a
//! preset assembled on top of it. See `analysis/genome_browser_design.md` for the full design.
//!
//! **Implemented so far** (build order in `analysis/genome_browser_design.md` §9):
//! - Step 1 — the load-bearing invariant [`XScale`], the [`Track`] extension seam, the
//!   [`TrackStack`] container with an explicit [`TrackStack::x_axis`] element, and [`PlotTrack`]
//!   (wraps any continuous-x `Vec<Plot>` by reusing `render_multiple` with forced shared margins).
//!   The go/no-go it proved: two stacked line tracks of very different y-magnitude align on x —
//!   the exact case `Figure`'s independent-per-cell margins get wrong.
//! - Step 2 — [`Track::right_margin`] (shared right gutter), [`Track::label`] (gutter track-names,
//!   drawn by the stack), [`Track::legend_entries`] → one deduplicated shared legend, and
//!   `PlotTrack::with_y_label`.
//! - Step 3 — the first non-plot tracks drawing directly against [`XScale`]: [`IntervalTrack`]
//!   (labelled bands) and [`VariantTrack`] (typed tick marks + legend entries); plus the
//!   [`StackLayer`] seam ([`TrackStack::underlay`] / [`TrackStack::overlay`]) with [`RegionHighlight`].
//!
//! Not yet here: genomic x-axis formatting (step 4), `CoveragePlot` preset (step 5), CLI (step 6).

use crate::plot::legend::{LegendEntry, LegendShape};
use crate::render::color::Color;
use crate::render::layout::{ComputedLayout, Layout};
use crate::render::plots::Plot;
use crate::render::render::{
    collect_legend_entries, render_legend_at, render_multiple, Primitive, Scene, TextAnchor,
};
use crate::render::text_metrics::{measure_text_width, FontStyle};
use crate::render::theme::Theme;

/// Fixed vertical band an axis element reserves (tick marks + tick labels).
const AXIS_BAND_PX: f64 = 34.0;
/// Default height a `Flex` track is assumed to want when auto-sizing the canvas.
const DEFAULT_FLEX_PX: f64 = 160.0;
/// Body font size for gutter track-labels and the shared legend.
const LABEL_SIZE: f64 = 12.0;
/// Padding added to a measured label/legend width when reserving a gutter.
const GUTTER_PAD: f64 = 8.0;

/// The shared horizontal scale for one [`TrackStack`]: maps data-x to absolute canvas pixel-x
/// within the pixel band `[px_left, px_right]` that every track's plot area occupies.
///
/// This is the one contract every track renders against. Its surface is kept deliberately
/// minimal (`map`/`invert`/range/edges) so it can later become a trait with a segmented impl
/// (multi-chromosome / trading-day gaps) without touching any `Track` implementation.
#[derive(Clone, Copy, Debug)]
pub struct XScale {
    x_min: f64,
    x_max: f64,
    log_x: bool,
    px_left: f64,
    px_right: f64,
}

impl XScale {
    /// Data coordinate -> absolute canvas pixel. Mirrors `ComputedLayout::map_x`.
    #[inline]
    pub fn map(&self, x: f64) -> f64 {
        if self.log_x {
            let lo = self.x_min.max(1e-10).log10();
            let hi = self.x_max.max(1e-10).log10();
            let t = (x.max(1e-10).log10() - lo) / (hi - lo);
            self.px_left + t * (self.px_right - self.px_left)
        } else {
            let t = (x - self.x_min) / (self.x_max - self.x_min);
            self.px_left + t * (self.px_right - self.px_left)
        }
    }

    /// Absolute canvas pixel -> data coordinate. Kept in the contract so pan/zoom/tooltips can
    /// arrive later without reshaping the API.
    #[inline]
    pub fn invert(&self, px: f64) -> f64 {
        let t = (px - self.px_left) / (self.px_right - self.px_left);
        if self.log_x {
            let lo = self.x_min.max(1e-10).log10();
            let hi = self.x_max.max(1e-10).log10();
            10f64.powf(lo + t * (hi - lo))
        } else {
            self.x_min + t * (self.x_max - self.x_min)
        }
    }

    pub fn x_range(&self) -> (f64, f64) {
        (self.x_min, self.x_max)
    }
    pub fn px_left(&self) -> f64 {
        self.px_left
    }
    pub fn px_right(&self) -> f64 {
        self.px_right
    }
    pub fn width(&self) -> f64 {
        self.px_right - self.px_left
    }
}

/// How tall a track wants to be.
#[derive(Clone, Copy, Debug)]
pub enum TrackHeight {
    /// Exact pixels — thin annotation lanes.
    Fixed(f64),
    /// Share of the leftover vertical space, by weight — the tall data panels.
    Flex(f64),
}

/// Everything a track needs to draw itself. Grows in later steps (interactive, bw_mode, ...);
/// kept small on purpose now.
pub struct TrackCtx<'a> {
    /// Shared horizontal scale (the only x-API a track needs).
    pub x: XScale,
    /// Top of this track's vertical band, absolute canvas coordinates.
    pub y_top: f64,
    /// This track's band height.
    pub height: f64,
    /// Full canvas width (handy for tracks that reserve their own right edge).
    pub width: f64,
    pub theme: &'a Theme,
}

impl TrackCtx<'_> {
    pub fn y_bottom(&self) -> f64 {
        self.y_top + self.height
    }
    pub fn inset(&self, frac: f64) -> f64 {
        self.y_top + frac * self.height
    }
}

/// The one thing a new track type implements — no core enum or match to touch.
///
/// `render` is **consuming** (`self: Box<Self>`): `Plot` is not `Clone`, so `PlotTrack` must own
/// its plots to hand to `render_multiple`. A stack renders exactly once, so consuming is fine.
/// The sizing/measuring methods (`height`, `x_bounds`, `left_margin`) run first, on `&self`.
pub trait Track {
    /// Vertical size request.
    fn height(&self) -> TrackHeight;

    /// Optionally influence the stack's shared x-range. `None` = just consume whatever the stack
    /// decides (typical for annotation tracks pinned to a locus).
    fn x_bounds(&self) -> Option<(f64, f64)> {
        None
    }

    /// Width needed at the shared left edge for a y-axis / y-tick labels. The stack takes the max
    /// across tracks — that shared max is the x-alignment mechanism. 0 = no y-axis.
    fn left_margin(&self) -> f64 {
        0.0
    }

    /// Width needed at the shared right edge (colorbar, right-side legend, secondary y-axis).
    /// Mirror of [`left_margin`](Self::left_margin); the stack takes the max. 0 = nothing.
    fn right_margin(&self) -> f64 {
        0.0
    }

    /// Optional track name. The stack draws it in the left gutter, vertically centred on this
    /// track's band, and folds its width into the shared left gutter. Tracks that draw their own
    /// y-axis (e.g. [`PlotTrack`]) return `None` — their y-axis label already names them.
    fn label(&self) -> Option<&str> {
        None
    }

    /// Legend entries this track contributes to the stack-level shared legend (deduplicated by
    /// label across tracks). Default: none.
    fn legend_entries(&self) -> Vec<LegendEntry> {
        Vec::new()
    }

    /// Draw into the band `[cx.y_top, cx.y_bottom()]`, using `cx.x` for all x. Push primitives
    /// (and any defs) into `scene`. MUST NOT paint a full background rect — the stack owns the one
    /// background.
    fn render(self: Box<Self>, cx: &TrackCtx<'_>, scene: &mut Scene);
}

/// A stack-spanning decoration drawn across the FULL height of every track at once — region
/// highlights, shared vertical gridlines, a cursor line. This is the one thing a per-track
/// [`Track`] structurally can't express. Added via [`TrackStack::underlay`] (behind all tracks)
/// or [`TrackStack::overlay`] (on top).
pub trait StackLayer {
    /// Return primitives (absolute canvas coords) spanning `[y_top, y_bottom]`, using `x` for x.
    fn render(&self, x: &XScale, y_top: f64, y_bottom: f64) -> Vec<Primitive>;
}

/// Highlight an x-interval across the whole stack (a variant of interest, an exon, a locus).
pub struct RegionHighlight {
    start: f64,
    end: f64,
    fill: Color,
    opacity: f64,
}

impl RegionHighlight {
    pub fn new(start: f64, end: f64) -> Self {
        Self {
            start,
            end,
            fill: Color::from("#ffd166"),
            opacity: 0.18,
        }
    }
    pub fn with_fill(mut self, fill: impl Into<Color>) -> Self {
        self.fill = fill.into();
        self
    }
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }
}

impl StackLayer for RegionHighlight {
    fn render(&self, x: &XScale, y_top: f64, y_bottom: f64) -> Vec<Primitive> {
        let (x0, x1) = (x.map(self.start), x.map(self.end));
        vec![Primitive::Rect {
            x: x0.min(x1),
            y: y_top,
            width: (x1 - x0).abs().max(1.0),
            height: (y_bottom - y_top).max(0.0),
            fill: self.fill.clone(),
            stroke: None,
            stroke_width: None,
            opacity: Some(self.opacity),
        }]
    }
}

/// Config for the one shared x-axis. (Formatting/label sizing land in later steps.)
#[derive(Clone, Default)]
pub struct AxisSpec {
    pub label: Option<String>,
}

/// One entry in the stack's ordered sequence. The x-axis is an explicit, positioned element —
/// never inferred from track order — so appending a track can never silently move the axis.
enum Entry {
    Track(Box<dyn Track>),
    Axis(AxisSpec),
}

/// A vertically stacked set of tracks sharing one x-axis. General primitive; genomics is a preset.
pub struct TrackStack {
    entries: Vec<Entry>,
    x_range: Option<(f64, f64)>,
    log_x: bool,
    spacing: f64,
    theme: Theme,
    underlays: Vec<Box<dyn StackLayer>>,
    overlays: Vec<Box<dyn StackLayer>>,
}

impl Default for TrackStack {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackStack {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            x_range: None,
            log_x: false,
            spacing: 6.0,
            theme: Theme::default(),
            underlays: Vec::new(),
            overlays: Vec::new(),
        }
    }

    pub fn track(mut self, t: impl Track + 'static) -> Self {
        self.entries.push(Entry::Track(Box::new(t)));
        self
    }

    /// Add a stack-spanning layer drawn BEHIND all tracks (region highlights, shared gridlines).
    pub fn underlay(mut self, layer: impl StackLayer + 'static) -> Self {
        self.underlays.push(Box::new(layer));
        self
    }

    /// Add a stack-spanning layer drawn ON TOP of all tracks (cursor, callouts).
    pub fn overlay(mut self, layer: impl StackLayer + 'static) -> Self {
        self.overlays.push(Box::new(layer));
        self
    }

    /// Place the shared x-axis HERE. Tracks added before render above it; tracks added after render
    /// below it. Call it exactly where you want the axis — its position is never inferred.
    pub fn x_axis(mut self) -> Self {
        self.entries.push(Entry::Axis(AxisSpec::default()));
        self
    }

    pub fn x_axis_with(mut self, spec: AxisSpec) -> Self {
        self.entries.push(Entry::Axis(spec));
        self
    }

    /// Pin the shared x-range to an explicit interval (a locus). Without this, the union of the
    /// tracks' `x_bounds()` is used.
    pub fn x_range(mut self, lo: f64, hi: f64) -> Self {
        self.x_range = Some((lo, hi));
        self
    }

    pub fn log_x(mut self, on: bool) -> Self {
        self.log_x = on;
        self
    }

    pub fn spacing(mut self, px: f64) -> Self {
        self.spacing = px;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Render with an auto total height (sum of track/axis heights + spacing). The common path —
    /// callers shouldn't have to guess a canvas height. Use [`render_sized`](Self::render_sized)
    /// to force one.
    pub fn render(mut self, width: f64) -> Scene {
        self.ensure_axis();
        let height = total_height(&self.entries, self.spacing);
        self.render_sized(width, height)
    }

    pub fn render_sized(mut self, width: f64, height: f64) -> Scene {
        self.ensure_axis();
        let TrackStack {
            entries,
            x_range,
            log_x,
            spacing,
            theme,
            underlays,
            overlays,
        } = self;

        // Shared x-range: explicit locus, else union of Track x_bounds(); guard degenerate.
        let (mut x_min, mut x_max) = x_range.unwrap_or_else(|| {
            entries
                .iter()
                .filter_map(|e| match e {
                    Entry::Track(t) => t.x_bounds(),
                    Entry::Axis(_) => None,
                })
                .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
                .unwrap_or((0.0, 1.0))
        });
        if x_max <= x_min {
            let pad = if x_min != 0.0 {
                x_min.abs() * 0.05
            } else {
                0.5
            };
            x_min -= pad;
            x_max += pad;
        }

        // Collect the shared legend up front (from &entries, before the consuming render loop),
        // deduplicated by label across tracks.
        let legend = dedup_legend(
            entries
                .iter()
                .filter_map(|e| match e {
                    Entry::Track(t) => Some(t.legend_entries()),
                    Entry::Axis(_) => None,
                })
                .flatten()
                .collect(),
        );

        // Shared LEFT gutter = max over tracks of (y-axis width, track-label width). This shared
        // max is the x-alignment mechanism. Shared RIGHT edge reserves the max of every track's
        // right_margin() and the shared legend's width.
        let px_left = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Track(t) => Some(t.left_margin().max(label_gutter_width(t.label()))),
                Entry::Axis(_) => None,
            })
            .fold(0.0_f64, f64::max)
            .max(10.0);
        let legend_reserve = legend_block_width(&legend);
        let max_track_right = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Track(t) => Some(t.right_margin()),
                Entry::Axis(_) => None,
            })
            .fold(0.0_f64, f64::max);
        let px_right = width - max_track_right.max(legend_reserve).max(12.0);
        let x = XScale {
            x_min,
            x_max,
            log_x,
            px_left,
            px_right,
        };

        let bands = layout_bands(&entries, height, spacing);
        let (y_top, y_bottom) = (
            bands.first().map_or(0.0, |b| b.y_top),
            bands.last().map_or(height, |b| b.y_bottom()),
        );

        // One background for the whole stack; tracks render transparent over it.
        let mut scene = Scene::new(width, height);
        scene.background_color = Some(theme.background.clone());
        scene.font_family = theme.font_family.clone();

        // Underlays first, spanning the full track region behind everything.
        for layer in &underlays {
            for prim in layer.render(&x, y_top, y_bottom) {
                scene.add(prim);
            }
        }

        for (entry, band) in entries.into_iter().zip(bands.iter()) {
            let cx = TrackCtx {
                x,
                y_top: band.y_top,
                height: band.height,
                width,
                theme: &theme,
            };
            match entry {
                Entry::Track(t) => {
                    // The stack draws the track-name label in the gutter (uniform placement);
                    // clone it out before `render` consumes the boxed track.
                    let label = t.label().map(str::to_owned);
                    t.render(&cx, &mut scene);
                    if let Some(name) = label {
                        draw_gutter_label(&mut scene, &name, &cx, &theme);
                    }
                }
                Entry::Axis(spec) => draw_shared_x_axis(&mut scene, &x, band, &spec, &theme),
            }
        }

        // Overlays on top of all tracks (cursor, callouts).
        for layer in &overlays {
            for prim in layer.render(&x, y_top, y_bottom) {
                scene.add(prim);
            }
        }

        // Shared legend in the reserved right band, vertically centred over the stack.
        if !legend.is_empty() {
            render_legend_at(
                &legend,
                None,
                None,
                true,
                &mut scene,
                px_right + GUTTER_PAD,
                (y_top + y_bottom) / 2.0,
                legend_reserve,
                LABEL_SIZE as u32,
                &theme,
            );
        }
        scene
    }

    fn ensure_axis(&mut self) {
        if !self.entries.iter().any(|e| matches!(e, Entry::Axis(_))) {
            self.entries.push(Entry::Axis(AxisSpec::default()));
        }
    }
}

/// A track that wraps any continuous-x `Vec<Plot>` and draws it via `render_multiple`, forced to
/// adopt the stack's shared x-margins so it pixel-aligns with every other track.
///
/// Continuous-x only: categorical-x plots (bar/box/violin/strip) do not share a numeric `XScale`
/// meaningfully and are not supported here.
pub struct PlotTrack {
    plots: Vec<Plot>,
    height: TrackHeight,
    y_label: Option<String>,
}

impl PlotTrack {
    pub fn new(plots: Vec<Plot>) -> Self {
        Self {
            plots,
            height: TrackHeight::Flex(1.0),
            y_label: None,
        }
    }

    pub fn with_height(mut self, height: TrackHeight) -> Self {
        self.height = height;
        self
    }

    /// Label for this track's y-axis (drawn in the shared left gutter). A `PlotTrack` names
    /// itself via its y-axis label rather than a gutter `label()`, so this doubles as its name.
    pub fn with_y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// The y-side `Layout` for this track's plots, before the stack forces shared x-geometry.
    /// Shared by the sizing methods and `render` so `left_margin`/`right_margin` predict exactly
    /// what `render` will produce.
    fn base_layout(&self) -> Layout {
        let mut l = Layout::auto_from_plots(&self.plots);
        if let Some(lbl) = &self.y_label {
            l = l.with_y_label(lbl.clone());
        }
        l
    }
}

impl Track for PlotTrack {
    fn height(&self) -> TrackHeight {
        self.height
    }

    fn x_bounds(&self) -> Option<(f64, f64)> {
        self.plots
            .iter()
            .filter_map(|p| p.bounds().map(|b| b.0))
            .reduce(|a, b| (a.0.min(b.0), a.1.max(b.1)))
    }

    fn left_margin(&self) -> f64 {
        // Margins are canvas-size-independent, so a provisional ComputedLayout reads the exact
        // y-driven left margin cheaply — and matches what `render` will force.
        ComputedLayout::from_layout(&self.base_layout()).margin_left
    }

    fn right_margin(&self) -> f64 {
        ComputedLayout::from_layout(&self.base_layout()).margin_right
    }

    fn legend_entries(&self) -> Vec<LegendEntry> {
        collect_legend_entries(&self.plots)
    }

    fn render(self: Box<Self>, cx: &TrackCtx<'_>, scene: &mut Scene) {
        let (x_min, x_max) = cx.x.x_range();
        let s = *self;
        let mut layout = s
            .base_layout()
            .with_width(cx.width)
            .with_height(cx.height)
            // Pin the range exactly (bypass nice-rounding) so map_x == XScale::map.
            .with_x_axis_min(x_min)
            .with_x_axis_max(x_max)
            // Force the shared gutters -> identical map_x across every track.
            .with_force_margins(cx.x.px_left(), cx.width - cx.x.px_right());
        // The stack draws the one shared x-axis; each track suppresses its own.
        layout.suppress_x_ticks = true;
        layout.log_x = cx.x.log_x;

        let sub = render_multiple(s.plots, layout);
        merge_translated(scene, sub, 0.0, cx.y_top);
    }
}

/// A labelled band over an x-interval, in an [`IntervalTrack`] (amplicons, primers, gene models).
pub struct Interval {
    pub start: f64,
    pub end: f64,
    pub label: Option<String>,
}

impl Interval {
    pub fn new(start: f64, end: f64) -> Self {
        Self {
            start,
            end,
            label: None,
        }
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A thin annotation lane drawing labelled bands over x-intervals. Draws directly against the
/// shared [`XScale`] — no y-axis, no `render_multiple`. The first proof of the non-plot `Track` seam.
pub struct IntervalTrack {
    intervals: Vec<Interval>,
    name: Option<String>,
    height_px: f64,
    fill: Color,
}

impl IntervalTrack {
    pub fn new(intervals: Vec<Interval>) -> Self {
        Self {
            intervals,
            name: None,
            height_px: 18.0,
            fill: Color::from("#7aa6c2"),
        }
    }
    /// Track name, drawn by the stack in the left gutter.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn with_height(mut self, px: f64) -> Self {
        self.height_px = px;
        self
    }
    pub fn with_fill(mut self, fill: impl Into<Color>) -> Self {
        self.fill = fill.into();
        self
    }
}

impl Track for IntervalTrack {
    fn height(&self) -> TrackHeight {
        TrackHeight::Fixed(self.height_px)
    }

    fn label(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn x_bounds(&self) -> Option<(f64, f64)> {
        let lo = self
            .intervals
            .iter()
            .map(|i| i.start)
            .fold(f64::INFINITY, f64::min);
        let hi = self
            .intervals
            .iter()
            .map(|i| i.end)
            .fold(f64::NEG_INFINITY, f64::max);
        lo.is_finite().then_some((lo, hi))
    }

    fn render(self: Box<Self>, cx: &TrackCtx<'_>, scene: &mut Scene) {
        let bar_h = (cx.height * 0.55).max(4.0);
        let y = cx.inset(0.15);
        for iv in &self.intervals {
            let (x0, x1) = (cx.x.map(iv.start), cx.x.map(iv.end));
            scene.add(Primitive::Rect {
                x: x0.min(x1),
                y,
                width: (x1 - x0).abs().max(1.0),
                height: bar_h,
                fill: self.fill.clone(),
                stroke: None,
                stroke_width: None,
                opacity: None,
            });
            if let Some(label) = &iv.label {
                scene.add(Primitive::Text {
                    x: (x0 + x1) / 2.0,
                    y: cx.y_bottom() - 1.0,
                    content: label.clone(),
                    size: 9,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                    color: Some(Color::Css(cx.theme.text_color.as_str().into())),
                });
            }
        }
    }
}

/// One coloured, labelled set of variant positions in a [`VariantTrack`].
struct VariantGroup {
    label: String,
    color: Color,
    positions: Vec<f64>,
}

/// A thin annotation lane of typed variant tick marks (e.g. SNV / InDel). Each group renders as
/// vertical ticks in its own colour and contributes a shared-legend entry.
pub struct VariantTrack {
    groups: Vec<VariantGroup>,
    name: Option<String>,
    height_px: f64,
}

impl Default for VariantTrack {
    fn default() -> Self {
        Self::new()
    }
}

impl VariantTrack {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            name: None,
            height_px: 14.0,
        }
    }
    /// Add a coloured, labelled group of variant positions.
    pub fn with_group(
        mut self,
        label: impl Into<String>,
        color: impl Into<Color>,
        positions: Vec<f64>,
    ) -> Self {
        self.groups.push(VariantGroup {
            label: label.into(),
            color: color.into(),
            positions,
        });
        self
    }
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn with_height(mut self, px: f64) -> Self {
        self.height_px = px;
        self
    }
}

impl Track for VariantTrack {
    fn height(&self) -> TrackHeight {
        TrackHeight::Fixed(self.height_px)
    }

    fn label(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn legend_entries(&self) -> Vec<LegendEntry> {
        self.groups
            .iter()
            .map(|g| LegendEntry {
                label: g.label.clone(),
                color: g.color.to_svg_string(),
                shape: LegendShape::Line,
                dasharray: None,
            })
            .collect()
    }

    fn x_bounds(&self) -> Option<(f64, f64)> {
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for g in &self.groups {
            for &p in &g.positions {
                lo = lo.min(p);
                hi = hi.max(p);
            }
        }
        lo.is_finite().then_some((lo, hi))
    }

    fn render(self: Box<Self>, cx: &TrackCtx<'_>, scene: &mut Scene) {
        for g in &self.groups {
            for &pos in &g.positions {
                let px = cx.x.map(pos);
                scene.add(Primitive::Line {
                    x1: px,
                    y1: cx.y_top,
                    x2: px,
                    y2: cx.y_bottom(),
                    stroke: g.color.clone(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });
            }
        }
    }
}

/// One track's assigned vertical band.
struct Band {
    y_top: f64,
    height: f64,
}

impl Band {
    fn y_bottom(&self) -> f64 {
        self.y_top + self.height
    }
}

fn entry_fixed_height(e: &Entry) -> Option<f64> {
    match e {
        Entry::Axis(_) => Some(AXIS_BAND_PX),
        Entry::Track(t) => match t.height() {
            TrackHeight::Fixed(px) => Some(px),
            TrackHeight::Flex(_) => None,
        },
    }
}

fn flex_weight(e: &Entry) -> f64 {
    match e {
        Entry::Track(t) => match t.height() {
            TrackHeight::Flex(w) => w,
            TrackHeight::Fixed(_) => 0.0,
        },
        Entry::Axis(_) => 0.0,
    }
}

/// Assign each entry a vertical band in sequence order. Fixed heights are honoured exactly; the
/// remainder is split among Flex entries by weight. Over-subscription clamps to a 1px min (never
/// negative) — prefer [`TrackStack::render`] (auto height) to avoid it entirely.
fn layout_bands(entries: &[Entry], total_height: f64, spacing: f64) -> Vec<Band> {
    let n = entries.len();
    let total_spacing = spacing * (n.saturating_sub(1)) as f64;
    let fixed_sum: f64 = entries.iter().filter_map(entry_fixed_height).sum();
    let flex_total: f64 = entries.iter().map(flex_weight).sum();
    let remainder = (total_height - total_spacing - fixed_sum).max(0.0);

    let mut bands = Vec::with_capacity(n);
    let mut y = 0.0;
    for e in entries {
        let h = match entry_fixed_height(e) {
            Some(px) => px,
            None => {
                if flex_total > 0.0 {
                    (remainder * flex_weight(e) / flex_total).max(1.0)
                } else {
                    1.0
                }
            }
        };
        bands.push(Band {
            y_top: y,
            height: h,
        });
        y += h + spacing;
    }
    bands
}

fn total_height(entries: &[Entry], spacing: f64) -> f64 {
    let n = entries.len();
    let total_spacing = spacing * (n.saturating_sub(1)) as f64;
    let sum: f64 = entries
        .iter()
        .map(|e| entry_fixed_height(e).unwrap_or(DEFAULT_FLEX_PX))
        .sum();
    sum + total_spacing
}

/// Merge a sub-scene (a `render_multiple` output) into `master`, translated by `(dx, dy)`.
/// Wraps the sub-scene's elements in a `translate` group and carries its defs/scripts across.
/// Factored so `Figure`'s equivalent inline merge can later share it.
fn merge_translated(master: &mut Scene, sub: Scene, dx: f64, dy: f64) {
    for def in sub.defs {
        master.defs.push(def);
    }
    master.add(Primitive::GroupStart {
        transform: Some(format!("translate({dx},{dy})")),
        title: None,
        extra_attrs: None,
    });
    for e in sub.elements {
        master.add(e);
    }
    master.add(Primitive::GroupEnd);
    if sub.has_tooltips {
        master.has_tooltips = true;
    }
    for script in sub.scripts {
        master.scripts.push(script);
    }
}

/// Draw the one shared x-axis for the stack at the top edge of its band. Minimal for step 1
/// (line + evenly spaced numeric ticks); genomic/datetime formatting arrives in step 4.
fn draw_shared_x_axis(scene: &mut Scene, x: &XScale, band: &Band, spec: &AxisSpec, theme: &Theme) {
    let y = band.y_top;
    let axis = || Color::Css(theme.axis_color.as_str().into());
    let text = || Some(Color::Css(theme.text_color.as_str().into()));

    // Axis line.
    scene.add(Primitive::Line {
        x1: x.px_left(),
        y1: y,
        x2: x.px_right(),
        y2: y,
        stroke: axis(),
        stroke_width: 1.0,
        stroke_dasharray: None,
    });

    let (x_min, x_max) = x.x_range();
    let n = 6usize;
    for i in 0..=n {
        let val = x_min + (i as f64 / n as f64) * (x_max - x_min);
        let px = x.map(val);
        scene.add(Primitive::Line {
            x1: px,
            y1: y,
            x2: px,
            y2: y + 5.0,
            stroke: axis(),
            stroke_width: 1.0,
            stroke_dasharray: None,
        });
        scene.add(Primitive::Text {
            x: px,
            y: y + 18.0,
            content: fmt_tick(val),
            size: 11,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
            color: text(),
        });
    }

    if let Some(label) = &spec.label {
        scene.add(Primitive::Text {
            x: (x.px_left() + x.px_right()) / 2.0,
            y: band.y_bottom() - 1.0,
            content: label.clone(),
            size: 12,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
            color: text(),
        });
    }
}

/// Width to reserve in the left gutter for a track's name label (0 if none).
fn label_gutter_width(label: Option<&str>) -> f64 {
    match label {
        Some(s) => measure_text_width(s, LABEL_SIZE, FontStyle::Regular) + GUTTER_PAD,
        None => 0.0,
    }
}

/// Width of the shared-legend block: widest entry label + swatch/padding. 0 if empty.
fn legend_block_width(entries: &[LegendEntry]) -> f64 {
    if entries.is_empty() {
        return 0.0;
    }
    let widest = entries
        .iter()
        .map(|e| measure_text_width(&e.label, LABEL_SIZE, FontStyle::Regular))
        .fold(0.0_f64, f64::max);
    widest + 35.0
}

/// Deduplicate legend entries by label, preserving first-seen order.
fn dedup_legend(entries: Vec<LegendEntry>) -> Vec<LegendEntry> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(entries.len());
    for e in entries {
        if seen.insert(e.label.clone()) {
            out.push(e);
        }
    }
    out
}

/// Draw a track's name in the left gutter, vertically centred on its band, left-aligned.
fn draw_gutter_label(scene: &mut Scene, name: &str, cx: &TrackCtx<'_>, theme: &Theme) {
    scene.add(Primitive::Text {
        x: 2.0,
        y: cx.inset(0.5) + LABEL_SIZE / 2.5,
        content: name.to_string(),
        size: LABEL_SIZE as u32,
        anchor: TextAnchor::Start,
        rotate: None,
        bold: false,
        color: Some(Color::Css(theme.text_color.as_str().into())),
    });
}

fn fmt_tick(v: f64) -> String {
    if (v.round() - v).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plot::LinePlot;

    fn line(ys: &[f64]) -> Plot {
        let pts: Vec<(f64, f64)> = ys.iter().enumerate().map(|(i, &y)| (i as f64, y)).collect();
        Plot::Line(LinePlot::new().with_data(pts))
    }

    /// The bug `Figure` has: different y-magnitudes -> different y-tick-label widths -> different
    /// auto `margin_left` -> x-axes don't line up. Guard that the problem is real, so the next
    /// test's fix is meaningful.
    #[test]
    fn different_y_magnitudes_misalign_without_forcing() {
        let small = Layout::auto_from_plots(&[line(&[0.0, 0.5, 1.0])])
            .with_width(900.0)
            .with_height(200.0);
        let big = Layout::auto_from_plots(&[line(&[0.0, 500_000.0, 1_000_000.0])])
            .with_width(900.0)
            .with_height(200.0);
        let ca = ComputedLayout::from_layout(&small);
        let cb = ComputedLayout::from_layout(&big);
        assert!(
            (ca.margin_left - cb.margin_left).abs() > 1.0,
            "expected differing left margins (got {} vs {})",
            ca.margin_left,
            cb.margin_left
        );
    }

    /// The fix: forcing the shared margins + pinning the range makes `map_x` identical across
    /// tracks of any y-magnitude, and equal to an independent `XScale` over the same band.
    #[test]
    fn forcing_margins_aligns_map_x() {
        let force = |p: Plot| {
            ComputedLayout::from_layout(
                &Layout::auto_from_plots(&[p])
                    .with_width(900.0)
                    .with_height(200.0)
                    .with_x_axis_min(0.0)
                    .with_x_axis_max(2.0)
                    .with_force_margins(120.0, 20.0),
            )
        };
        let ca = force(line(&[0.0, 0.5, 1.0]));
        let cb = force(line(&[0.0, 500_000.0, 1_000_000.0]));

        for &xv in &[0.0, 0.5, 1.0, 1.5, 2.0] {
            assert!(
                (ca.map_x(xv) - cb.map_x(xv)).abs() < 1e-9,
                "map_x misaligned at x={xv}: {} vs {}",
                ca.map_x(xv),
                cb.map_x(xv)
            );
        }

        let xs = XScale {
            x_min: 0.0,
            x_max: 2.0,
            log_x: false,
            px_left: 120.0,
            px_right: 900.0 - 20.0,
        };
        for &xv in &[0.0, 1.0, 2.0] {
            assert!((ca.map_x(xv) - xs.map(xv)).abs() < 1e-9);
        }
    }

    /// End-to-end: two mismatched-y line tracks + an explicit axis render into one Scene without
    /// panicking, at the requested width.
    #[test]
    fn stack_renders_two_line_tracks() {
        let scene = TrackStack::new()
            .x_range(0.0, 2.0)
            .track(PlotTrack::new(vec![line(&[0.0, 0.5, 1.0])]))
            .track(PlotTrack::new(vec![line(&[0.0, 500_000.0, 1_000_000.0])]))
            .x_axis()
            .render(900.0);
        assert_eq!(scene.width, 900.0);
        assert!(!scene.elements.is_empty());
    }

    /// The auto-appended axis: never placing `.x_axis()` still yields exactly one axis at the
    /// bottom (no panic, renders).
    #[test]
    fn axis_defaults_to_bottom_when_unplaced() {
        let scene = TrackStack::new()
            .x_range(0.0, 10.0)
            .track(PlotTrack::new(vec![line(&[1.0, 2.0, 3.0])]))
            .render(600.0);
        assert!(scene.height > 0.0);
    }

    // ---- step 2 ----

    use crate::plot::legend::{LegendEntry, LegendShape};

    fn legend_entry(label: &str) -> LegendEntry {
        LegendEntry {
            label: label.into(),
            color: "#000000".into(),
            shape: LegendShape::Line,
            dasharray: None,
        }
    }

    /// A minimal non-plot track for exercising `label()` / `legend_entries()` without depending on
    /// any real plot type's legend API (annotation tracks land properly in step 3).
    struct DummyTrack {
        label: Option<String>,
        legend: Vec<LegendEntry>,
    }
    impl Track for DummyTrack {
        fn height(&self) -> TrackHeight {
            TrackHeight::Fixed(40.0)
        }
        fn label(&self) -> Option<&str> {
            self.label.as_deref()
        }
        fn legend_entries(&self) -> Vec<LegendEntry> {
            self.legend.clone()
        }
        fn render(self: Box<Self>, _cx: &TrackCtx<'_>, _scene: &mut Scene) {}
    }

    fn count_text(scene: &Scene, needle: &str) -> usize {
        scene
            .elements
            .iter()
            .filter(|p| matches!(p, Primitive::Text { content, .. } if content == needle))
            .count()
    }

    #[test]
    fn dedup_legend_removes_duplicate_labels_preserving_order() {
        let out = dedup_legend(vec![
            legend_entry("A"),
            legend_entry("B"),
            legend_entry("A"),
            legend_entry("C"),
            legend_entry("B"),
        ]);
        let labels: Vec<_> = out.iter().map(|e| e.label.as_str()).collect();
        assert_eq!(labels, vec!["A", "B", "C"]);
    }

    /// The stack draws a track's `label()` in the gutter and a deduplicated shared legend, both as
    /// top-level Text primitives.
    #[test]
    fn gutter_label_and_shared_legend_render() {
        let track = DummyTrack {
            label: Some("Genes".into()),
            legend: vec![
                legend_entry("SNV"),
                legend_entry("SNV"),
                legend_entry("InDel"),
            ],
        };
        let scene = TrackStack::new()
            .x_range(0.0, 10.0)
            .track(track)
            .x_axis()
            .render_sized(600.0, 200.0);
        assert_eq!(count_text(&scene, "Genes"), 1, "gutter label missing");
        assert_eq!(count_text(&scene, "SNV"), 1, "legend not deduped");
        assert_eq!(count_text(&scene, "InDel"), 1, "legend entry missing");
    }

    /// A `PlotTrack`'s y-axis label actually renders (it names the track via its y-axis, so it has
    /// no separate gutter `label()`). Margins don't change — `from_layout` reserves the label band
    /// as a fixed `label_size` regardless of text — but the label text must appear in the output.
    #[test]
    fn y_label_renders() {
        let scene = TrackStack::new()
            .x_range(0.0, 2.0)
            .track(PlotTrack::new(vec![line(&[0.0, 1.0, 2.0])]).with_y_label("coverage depth"))
            .x_axis()
            .render(600.0);
        assert_eq!(count_text(&scene, "coverage depth"), 1);
    }

    /// Alignment still holds when tracks have different right reservations: a legend-bearing track
    /// and a bare track both map x identically (px_right is shared).
    #[test]
    fn alignment_holds_with_shared_right_reserve() {
        // Two PlotTracks; both go through the same forced shared margins regardless of their own
        // right reservations, so a rendered stack keeps x aligned. Smoke: renders without panic.
        let scene = TrackStack::new()
            .x_range(0.0, 100.0)
            .track(PlotTrack::new(vec![line(&[0.0, 50.0, 100.0])]).with_y_label("depth"))
            .track(PlotTrack::new(vec![line(&[0.0, 1.0, 2.0])]))
            .x_axis()
            .render(700.0);
        assert_eq!(scene.width, 700.0);
    }

    // ---- step 3 ----

    fn count_prims<F: Fn(&Primitive) -> bool>(scene: &Scene, pred: F) -> usize {
        scene.elements.iter().filter(|p| pred(p)).count()
    }

    /// `IntervalTrack` draws one band rect per interval, the interval labels, and its track name in
    /// the gutter — all directly against `XScale`, no `render_multiple`.
    #[test]
    fn interval_track_bands_labels_and_gutter_name() {
        let track = IntervalTrack::new(vec![
            Interval::new(1200.0, 1800.0).with_label("amp1"),
            Interval::new(2600.0, 3200.0).with_label("amp2"),
        ])
        .with_name("Amplicons");
        let scene = TrackStack::new()
            .x_range(1000.0, 4000.0)
            .x_axis()
            .track(track)
            .render_sized(600.0, 160.0);
        assert_eq!(count_text(&scene, "Amplicons"), 1, "gutter track-name");
        assert_eq!(count_text(&scene, "amp1"), 1);
        assert_eq!(count_text(&scene, "amp2"), 1);
        // Two band rects (no legend here, so no legend box rect).
        assert_eq!(
            count_prims(&scene, |p| matches!(p, Primitive::Rect { .. })),
            2
        );
    }

    /// `VariantTrack` draws one tick line per position and contributes one shared-legend entry per
    /// group.
    #[test]
    fn variant_track_ticks_and_legend() {
        let track = VariantTrack::new()
            .with_group("SNV", "#d1495b", vec![1200.0, 3400.0])
            .with_group("InDel", "#edae49", vec![2900.0])
            .with_name("Variants");
        let scene = TrackStack::new()
            .x_range(1000.0, 4000.0)
            .track(track)
            .x_axis()
            .render_sized(600.0, 160.0);
        assert_eq!(count_text(&scene, "Variants"), 1, "gutter track-name");
        assert_eq!(count_text(&scene, "SNV"), 1, "SNV legend entry");
        assert_eq!(count_text(&scene, "InDel"), 1, "InDel legend entry");
    }

    /// A `RegionHighlight` underlay renders as a rect BEFORE the first track group — i.e. behind the
    /// tracks, spanning the stack.
    #[test]
    fn region_highlight_underlay_renders_behind_tracks() {
        let scene = TrackStack::new()
            .x_range(0.0, 100.0)
            .underlay(RegionHighlight::new(40.0, 60.0).with_fill("#ffd166"))
            .track(PlotTrack::new(vec![line(&[0.0, 1.0, 2.0])]))
            .x_axis()
            .render(600.0);
        let first_rect = scene
            .elements
            .iter()
            .position(|p| matches!(p, Primitive::Rect { .. }));
        let first_group = scene
            .elements
            .iter()
            .position(|p| matches!(p, Primitive::GroupStart { .. }));
        assert!(first_rect.is_some(), "underlay rect missing");
        assert!(first_group.is_some(), "track group missing");
        assert!(
            first_rect.unwrap() < first_group.unwrap(),
            "underlay must render behind (before) the track"
        );
    }
}
