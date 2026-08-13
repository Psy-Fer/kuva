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
//! - Step 4 — shared x-axis formatting ([`XAxisFormat`] on [`AxisSpec`]): nice 1-2-5 tick positions
//!   (via `render_utils::generate_ticks`) and a `Genomic` mode picking one bp/kb/Mb/Gb unit for the
//!   whole axis.
//!
//! - Step 5 — the [`CoveragePlot`](crate::render::coverage::CoveragePlot) genomics preset
//!   (issue #2), assembled entirely from this primitive (depth tracks + variant lane + feature
//!   bands + genomic axis). Lives in `src/render/coverage.rs`.
//! - Step 6 — CLI `kuva coverage` subcommand (`src/bin/kuva/coverage.rs`).
//!
//! Visual-inspection SVGs for the library path are emitted to `test_outputs/` by
//! `tests/track_stack_svg.rs` / `tests/coverage_svg.rs` (skipped under CI). Still open:
//! `--emit-code`, docs pages, gallery, man-page regen.

use crate::plot::legend::{LegendEntry, LegendGroup, LegendShape};
use crate::render::annotations::ReferenceLine;
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
/// Gap between the plot area's right edge and the shared legend box.
const LEGEND_LEFT_GAP: f64 = 10.0;
/// Padding kept clear to the right of the legend box so its border is visible.
const LEGEND_RIGHT_PAD: f64 = 12.0;
/// Vertical band reserved for the figure title when one is set.
const TITLE_BAND_PX: f64 = 30.0;
/// Title font size.
const TITLE_SIZE: u32 = 18;

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

    /// Legend entries this track contributes. Rendered as one titled section (group) per track in
    /// the shared right-margin legend. Default: none.
    fn legend_entries(&self) -> Vec<LegendEntry> {
        Vec::new()
    }

    /// Title for this track's legend section. Defaults to the track's gutter [`label`](Self::label),
    /// so a named track gets a titled section for free; override to decouple them, or return `None`
    /// for an untitled section (entries with no heading).
    fn legend_group_title(&self) -> Option<String> {
        self.label().map(str::to_string)
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

/// How the shared x-axis formats its tick labels.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum XAxisFormat {
    /// Plain numbers (nice-rounded; integers or up to 2 decimals).
    #[default]
    Numeric,
    /// Genomic coordinates with a single bp/kb/Mb/Gb unit chosen for the whole axis
    /// (e.g. `1.5 Mb`, `25 kb`, `800 bp`).
    Genomic,
    /// Date/time axis. X values are Unix timestamps (seconds); tick positions and labels are
    /// derived by [`DateTimeAxis::auto`](crate::render::datetime::DateTimeAxis) — the non-genomic
    /// use case (financial / sensor time series).
    DateTime,
}

/// Config for the one shared x-axis.
#[derive(Clone, Default)]
pub struct AxisSpec {
    pub label: Option<String>,
    pub format: XAxisFormat,
}

impl AxisSpec {
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn with_format(mut self, format: XAxisFormat) -> Self {
        self.format = format;
        self
    }
    /// Shorthand for `AxisSpec { format: Genomic, .. }` with a label.
    pub fn genomic(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            format: XAxisFormat::Genomic,
        }
    }
    /// Shorthand for `AxisSpec { format: DateTime, .. }` with a label (x = Unix seconds).
    pub fn datetime(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            format: XAxisFormat::DateTime,
        }
    }
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
    title: Option<String>,
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
            title: None,
            underlays: Vec::new(),
            overlays: Vec::new(),
        }
    }

    pub fn track(mut self, t: impl Track + 'static) -> Self {
        self.entries.push(Entry::Track(Box::new(t)));
        self
    }

    /// Figure title, drawn centred in a reserved band at the top (pushes the tracks down).
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
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
        let title_h = if self.title.is_some() {
            TITLE_BAND_PX
        } else {
            0.0
        };
        let height = total_height(&self.entries, self.spacing) + title_h;
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
            title,
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

        // Build the shared legend as one titled SECTION (group) per track that contributes entries
        // — so each track shows its own info (e.g. sample colours, variant types, per-gene colours)
        // instead of everything being merged into one flat list. Consecutive tracks with the same
        // section title merge (e.g. the depth tracks form one untitled "samples" block).
        let legend_groups: Vec<LegendGroup> = {
            let mut groups: Vec<LegendGroup> = Vec::new();
            for e in &entries {
                if let Entry::Track(t) = e {
                    let entries = t.legend_entries();
                    if entries.is_empty() {
                        continue;
                    }
                    let title = t.legend_group_title().unwrap_or_default();
                    match groups.last_mut() {
                        Some(g) if g.title == title => g.entries.extend(entries),
                        _ => groups.push(LegendGroup { title, entries }),
                    }
                }
            }
            groups
        };

        // Shared LEFT gutter = max over ALL entries (tracks' y-axis/name width AND the axis
        // entry's own label — the x-axis label lives in the gutter like a track label now, not
        // centred under the axis). This shared max is the x-alignment mechanism.
        let px_left = entries
            .iter()
            .map(|e| match e {
                Entry::Track(t) => t.left_margin().max(label_gutter_width(t.label())),
                Entry::Axis(s) => label_gutter_width(s.label.as_deref()),
            })
            .fold(0.0_f64, f64::max)
            .max(10.0);

        // Shared RIGHT edge reserves the widest track right_margin OR the legend block (its own
        // content width + a left gap + right padding so the box border stays clear of the canvas).
        let legend_w = legend_groups_width(&legend_groups);
        let legend_reserve = if legend_w > 0.0 {
            legend_w + LEGEND_LEFT_GAP + LEGEND_RIGHT_PAD
        } else {
            0.0
        };
        let max_track_right = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Track(t) => Some(t.right_margin()),
                Entry::Axis(_) => None,
            })
            .fold(0.0_f64, f64::max);
        let px_right = width - max_track_right.max(legend_reserve).max(LEGEND_RIGHT_PAD);
        let x = XScale {
            x_min,
            x_max,
            log_x,
            px_left,
            px_right,
        };

        // Reserve a title band at the top; tracks lay out in the space below it.
        let title_h = if title.is_some() { TITLE_BAND_PX } else { 0.0 };
        let bands = layout_bands(&entries, height - title_h, spacing);
        let (y_top, y_bottom) = (
            title_h + bands.first().map_or(0.0, |b| b.y_top),
            title_h + bands.last().map_or(height - title_h, |b| b.y_bottom()),
        );

        // One background for the whole stack; tracks render transparent over it.
        let mut scene = Scene::new(width, height);
        scene.background_color = Some(theme.background.clone());
        scene.font_family = theme.font_family.clone();

        if let Some(title) = &title {
            scene.add(Primitive::Text {
                x: (px_left + px_right) / 2.0,
                y: TITLE_BAND_PX * 0.62,
                content: title.clone(),
                size: TITLE_SIZE,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: true,
                color: Some(Color::Css(theme.text_color.as_str().into())),
            });
        }

        // Underlays first, spanning the full track region behind everything.
        for layer in &underlays {
            for prim in layer.render(&x, y_top, y_bottom) {
                scene.add(prim);
            }
        }

        for (entry, band) in entries.into_iter().zip(bands.iter()) {
            // Offset each band below the title band.
            let cx = TrackCtx {
                x,
                y_top: band.y_top + title_h,
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
                Entry::Axis(spec) => {
                    let axis_band = Band {
                        y_top: band.y_top + title_h,
                        height: band.height,
                    };
                    draw_shared_x_axis(&mut scene, &x, &axis_band, &spec, &theme);
                }
            }
        }

        // Overlays on top of all tracks (cursor, callouts).
        for layer in &overlays {
            for prim in layer.render(&x, y_top, y_bottom) {
                scene.add(prim);
            }
        }

        // Shared legend (one titled section per track) in the reserved right band, vertically
        // centred over the stack, with a gap from the plot area and padding kept clear on its right.
        if !legend_groups.is_empty() {
            let total_rows: usize = legend_groups
                .iter()
                .map(|g| g.entries.len() + usize::from(!g.title.is_empty()))
                .sum();
            let legend_h = total_rows as f64 * (LABEL_SIZE * 1.5).max(12.0);
            render_legend_at(
                &[],
                Some(&legend_groups),
                None,
                true,
                &mut scene,
                px_right + LEGEND_LEFT_GAP,
                (y_top + y_bottom - legend_h) / 2.0,
                legend_w,
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
    reference_lines: Vec<ReferenceLine>,
}

impl PlotTrack {
    pub fn new(plots: Vec<Plot>) -> Self {
        Self {
            plots,
            height: TrackHeight::Flex(1.0),
            y_label: None,
            reference_lines: Vec::new(),
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

    /// Draw a dashed horizontal reference line across this track at y = `value` (in the track's
    /// own y units, e.g. a minimum-coverage threshold on a depth track).
    pub fn with_hline(mut self, value: f64) -> Self {
        self.reference_lines.push(ReferenceLine::horizontal(value));
        self
    }

    /// Add a fully-styled reference line ([`ReferenceLine::horizontal`]/`vertical` with colour,
    /// dash, and label).
    pub fn with_reference_line(mut self, line: ReferenceLine) -> Self {
        self.reference_lines.push(line);
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
        // The stack draws ONE shared legend (collected via `legend_entries`); suppress the
        // per-plot legend `render_multiple` would otherwise draw inside this track's band.
        layout.show_legend = false;
        // Reference lines (e.g. a coverage threshold) draw in this track's own y-scale.
        layout.reference_lines = s.reference_lines;

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
/// shared [`XScale`] — no y-axis, no `render_multiple`.
///
/// **Overlapping intervals tile onto multiple rows** (greedy minimum-row packing, the same idea
/// as brick/terminal label packing): each interval takes the lowest row where it doesn't overlap
/// one already placed, so N mutually-overlapping intervals need N rows. This gives the classic
/// tiled-amplicon layout (e.g. ARTIC's two alternating primer pools land on two rows). Rows can be
/// coloured independently via [`with_row_colors`](Self::with_row_colors) — a natural fit for
/// per-pool colouring. The track's height grows with the row count (`row_height_px` per row).
pub struct IntervalTrack {
    intervals: Vec<Interval>,
    name: Option<String>,
    row_height_px: f64,
    fill: Color,
    row_colors: Option<Vec<Color>>,
    item_colors: Option<Vec<Color>>,
}

impl IntervalTrack {
    pub fn new(intervals: Vec<Interval>) -> Self {
        Self {
            intervals,
            name: None,
            row_height_px: 20.0,
            fill: Color::from("#7aa6c2"),
            row_colors: None,
            item_colors: None,
        }
    }
    /// Track name, drawn by the stack in the left gutter.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Per-row band height in pixels (total track height = this × row count). Default 20.
    pub fn with_row_height(mut self, px: f64) -> Self {
        self.row_height_px = px;
        self
    }
    /// Single fill colour for every band (used when no per-row / per-item colours are set).
    pub fn with_fill(mut self, fill: impl Into<Color>) -> Self {
        self.fill = fill.into();
        self
    }
    /// Colour bands by their packed row, cycling through these colours — e.g. two colours for the
    /// two alternating amplicon pools in a tiled scheme.
    pub fn with_row_colors<C: Into<Color>>(mut self, colors: Vec<C>) -> Self {
        self.row_colors = Some(colors.into_iter().map(Into::into).collect());
        self
    }
    /// Colour each interval individually, cycling through these colours by item index, AND expose
    /// each interval as a legend entry (label + its colour). Use for a gene/region track where the
    /// legend is the key — small bands stay identifiable by colour even when their label doesn't
    /// fit inside. Takes precedence over `with_row_colors`/`with_fill`.
    pub fn with_item_colors<C: Into<Color>>(mut self, colors: Vec<C>) -> Self {
        self.item_colors = Some(colors.into_iter().map(Into::into).collect());
        self
    }

    /// Assign each interval to a row via greedy minimum-row packing (interval-graph colouring):
    /// sort by start, then place each into the lowest row whose last interval ends at or before
    /// this one's start. Returns `(row_per_interval, row_count)`; `row_count` is the maximum
    /// overlap depth, so the layout uses exactly as many rows as needed.
    fn pack_rows(&self) -> (Vec<usize>, usize) {
        let mut order: Vec<usize> = (0..self.intervals.len()).collect();
        order.sort_by(|&a, &b| {
            self.intervals[a]
                .start
                .partial_cmp(&self.intervals[b].start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut row_end: Vec<f64> = Vec::new(); // end of the last interval placed in each row
        let mut row_of = vec![0usize; self.intervals.len()];
        for &i in &order {
            let iv = &self.intervals[i];
            match row_end.iter().position(|&end| iv.start >= end) {
                Some(r) => {
                    row_end[r] = iv.end;
                    row_of[i] = r;
                }
                None => {
                    row_of[i] = row_end.len();
                    row_end.push(iv.end);
                }
            }
        }
        (row_of, row_end.len().max(1))
    }
}

impl Track for IntervalTrack {
    fn height(&self) -> TrackHeight {
        // Height scales with the number of packed rows (overlap depth).
        let (_, rows) = self.pack_rows();
        TrackHeight::Fixed(self.row_height_px * rows as f64)
    }

    fn label(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn legend_entries(&self) -> Vec<LegendEntry> {
        // Only a per-item-coloured track carries a legend (one entry per interval). Row-coloured /
        // single-fill tracks are self-labelled inline and contribute nothing.
        match &self.item_colors {
            Some(cs) if !cs.is_empty() => self
                .intervals
                .iter()
                .enumerate()
                .map(|(i, iv)| LegendEntry {
                    label: iv.label.clone().unwrap_or_default(),
                    color: cs[i % cs.len()].to_svg_string(),
                    shape: LegendShape::Rect,
                    dasharray: None,
                })
                .collect(),
            _ => Vec::new(),
        }
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
        let (row_of, rows) = self.pack_rows();
        let row_h = cx.height / rows as f64;
        let bar_h = (row_h * 0.7).max(4.0);
        let text_color = Color::Css(cx.theme.text_color.as_str().into());

        for (i, iv) in self.intervals.iter().enumerate() {
            let row = row_of[i];
            let row_top = cx.y_top + row as f64 * row_h;
            let y = row_top + (row_h - bar_h) * 0.5;
            let (x0, x1) = (cx.x.map(iv.start), cx.x.map(iv.end));
            // Per-item colours win, then per-row (pool) colours, then the single fill.
            let fill = if let Some(cs) = self.item_colors.as_ref().filter(|c| !c.is_empty()) {
                cs[i % cs.len()].clone()
            } else if let Some(cs) = self.row_colors.as_ref().filter(|c| !c.is_empty()) {
                cs[row % cs.len()].clone()
            } else {
                self.fill.clone()
            };
            scene.add(Primitive::Rect {
                x: x0.min(x1),
                y,
                width: (x1 - x0).abs().max(1.0),
                height: bar_h,
                fill,
                stroke: None,
                stroke_width: None,
                opacity: None,
            });
            // Draw the label centred inside the band only if it fits; tiny bands (e.g. the small
            // 3' SARS-CoV-2 genes) stay as bare coloured boxes rather than spilling overlapping text.
            if let Some(label) = &iv.label {
                let box_w = (x1 - x0).abs();
                if measure_text_width(label, 9.0, FontStyle::Regular) + 4.0 <= box_w {
                    scene.add(Primitive::Text {
                        x: (x0 + x1) / 2.0,
                        y: row_top + row_h * 0.5 + 3.0,
                        content: label.clone(),
                        size: 9,
                        anchor: TextAnchor::Middle,
                        rotate: None,
                        bold: false,
                        color: Some(text_color.clone()),
                    });
                }
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

/// Draw the one shared x-axis for the stack at the top edge of its band. The axis *label* is drawn
/// in the LEFT GUTTER (left-aligned, like a track name) rather than centred under the axis — it
/// then reads consistently with every other track label and stays clear of any track below it.
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
    // Tick positions + labels depend on the format.
    let (ticks, labels): (Vec<f64>, Vec<String>) = match spec.format {
        XAxisFormat::DateTime => {
            // X values are Unix seconds; DateTimeAxis picks a calendar-aligned unit + format.
            let axis = crate::render::datetime::DateTimeAxis::auto(x_min, x_max);
            let t = axis.generate_ticks(x_min, x_max);
            let l = t.iter().map(|&v| axis.format_tick(v)).collect();
            (t, l)
        }
        fmt => {
            // Nice 1-2-5 tick positions, shared with the rest of kuva's axes.
            let t = crate::render::render_utils::generate_ticks(x_min, x_max, 6);
            // For Genomic, pick ONE unit for the whole axis so labels don't mix bp/kb/Mb.
            let unit = genomic_unit(x_min, x_max);
            let l = t.iter().map(|&v| format_tick(v, fmt, unit)).collect();
            (t, l)
        }
    };

    for (&val, label) in ticks.iter().zip(labels.iter()) {
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
            content: label.clone(),
            size: 11,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
            color: text(),
        });
    }

    // Axis label in the left gutter, on the SAME baseline as the tick labels (y + 18) so it lines
    // up with the tick numbers and the axis line rather than floating at the band centre.
    if let Some(label) = &spec.label {
        scene.add(Primitive::Text {
            x: 2.0,
            y: y + 18.0,
            content: label.clone(),
            size: 11,
            anchor: TextAnchor::Start,
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

/// Width of the shared-legend box: widest entry-or-title label across all groups + swatch/padding.
/// 0 if there are no entries.
fn legend_groups_width(groups: &[LegendGroup]) -> f64 {
    let mut widest = 0.0_f64;
    for g in groups {
        if !g.title.is_empty() {
            widest = widest.max(measure_text_width(&g.title, LABEL_SIZE, FontStyle::Regular));
        }
        for e in &g.entries {
            // Entries are inset past the swatch, so account for that in the content width.
            widest =
                widest.max(measure_text_width(&e.label, LABEL_SIZE, FontStyle::Regular) + 20.0);
        }
    }
    if widest == 0.0 {
        0.0
    } else {
        widest + 20.0
    }
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

/// A genomic unit chosen for a whole axis: the divisor + suffix that keeps tick numbers small.
#[derive(Clone, Copy)]
struct GenomicUnit {
    divisor: f64,
    suffix: &'static str,
}

/// Pick one bp/kb/Mb/Gb unit for the axis from its largest-magnitude endpoint, so every tick
/// label shares a unit instead of mixing (e.g. an axis to 5 Mb never shows a tick in "kb").
fn genomic_unit(x_min: f64, x_max: f64) -> GenomicUnit {
    let mag = x_min.abs().max(x_max.abs());
    if mag >= 1e9 {
        GenomicUnit {
            divisor: 1e9,
            suffix: "Gb",
        }
    } else if mag >= 1e6 {
        GenomicUnit {
            divisor: 1e6,
            suffix: "Mb",
        }
    } else if mag >= 1e3 {
        GenomicUnit {
            divisor: 1e3,
            suffix: "kb",
        }
    } else {
        GenomicUnit {
            divisor: 1.0,
            suffix: "bp",
        }
    }
}

fn format_tick(v: f64, format: XAxisFormat, unit: GenomicUnit) -> String {
    match format {
        // DateTime is formatted upstream via DateTimeAxis and never reaches here; fall back to
        // plain numbers if it ever does.
        XAxisFormat::Numeric | XAxisFormat::DateTime => fmt_numeric(v),
        XAxisFormat::Genomic => format!("{} {}", fmt_numeric(v / unit.divisor), unit.suffix),
    }
}

/// Integer when whole, else up to 3 decimals with trailing zeros trimmed.
fn fmt_numeric(v: f64) -> String {
    if (v.round() - v).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
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

    /// A minimal non-plot track for exercising `label()` / `legend_entries()` /
    /// `legend_group_title()` without depending on any real plot type's legend API.
    struct DummyTrack {
        label: Option<String>,
        legend_title: Option<String>,
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
        fn legend_group_title(&self) -> Option<String> {
            self.legend_title.clone()
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

    /// The stack draws a track's `label()` in the gutter and its legend as a titled section
    /// (group heading + entries), all as top-level Text primitives.
    #[test]
    fn gutter_label_and_legend_section_render() {
        let track = DummyTrack {
            label: Some("Genes".into()),
            legend_title: Some("variants".into()),
            legend: vec![legend_entry("SNV"), legend_entry("InDel")],
        };
        let scene = TrackStack::new()
            .x_range(0.0, 10.0)
            .track(track)
            .x_axis()
            .render_sized(600.0, 200.0);
        assert_eq!(count_text(&scene, "Genes"), 1, "gutter label missing");
        assert_eq!(
            count_text(&scene, "variants"),
            1,
            "legend section title missing"
        );
        assert_eq!(count_text(&scene, "SNV"), 1, "legend entry missing");
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
        // "Variants" appears twice: the gutter track-name AND the legend section title (which
        // defaults to the track name).
        assert_eq!(
            count_text(&scene, "Variants"),
            2,
            "gutter name + legend section title"
        );
        assert_eq!(count_text(&scene, "SNV"), 1, "SNV legend entry");
        assert_eq!(count_text(&scene, "InDel"), 1, "InDel legend entry");
    }

    // ---- step 4 ----

    #[test]
    fn genomic_unit_is_chosen_from_axis_magnitude() {
        assert_eq!(genomic_unit(0.0, 800.0).suffix, "bp");
        assert_eq!(genomic_unit(1000.0, 5000.0).suffix, "kb");
        assert_eq!(genomic_unit(1_000_000.0, 5_000_000.0).suffix, "Mb");
        assert_eq!(genomic_unit(0.0, 3.2e9).suffix, "Gb");
    }

    #[test]
    fn format_tick_genomic_and_numeric() {
        let mb = genomic_unit(0.0, 5_000_000.0);
        assert_eq!(format_tick(1_500_000.0, XAxisFormat::Genomic, mb), "1.5 Mb");
        assert_eq!(format_tick(2_000_000.0, XAxisFormat::Genomic, mb), "2 Mb");
        let kb = genomic_unit(0.0, 5000.0);
        assert_eq!(format_tick(2500.0, XAxisFormat::Genomic, kb), "2.5 kb");
        // Numeric ignores the unit.
        assert_eq!(format_tick(2500.0, XAxisFormat::Numeric, kb), "2500");
    }

    /// A `Genomic` axis emits at least one unit-suffixed tick label in the rendered scene.
    #[test]
    fn genomic_axis_labels_render_with_unit() {
        let scene = TrackStack::new()
            .x_range(1_000_000.0, 5_000_000.0)
            .track(PlotTrack::new(vec![line(&[0.0, 1.0, 2.0])]))
            .x_axis_with(AxisSpec::genomic("chr1"))
            .render(700.0);
        let has_mb = scene
            .elements
            .iter()
            .any(|p| matches!(p, Primitive::Text { content, .. } if content.ends_with(" Mb")));
        assert!(has_mb, "expected an axis tick labelled in Mb");
        assert_eq!(count_text(&scene, "chr1"), 1, "axis label present");
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
