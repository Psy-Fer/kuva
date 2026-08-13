use crate::render::color::Color;
use crate::render::datetime::DateTimeAxis;
use crate::render::layout::{
    AxisLabelOverlap, AxisLine, ComputedLayout, Layout, TickAlign, TickFormat, TickPos,
    SUBTITLE_MUTE,
};
use crate::render::render::{Primitive, Scene, TextAnchor};
use crate::render::render_utils;
use crate::render::text_metrics::{
    ascent, center_offset, line_height, measure_text_width, FontStyle,
};

fn draw_x_tick(
    scene: &mut Scene,
    layout: &Layout,
    computed: &ComputedLayout,
    theme: &crate::render::theme::Theme,
    x: f64,
    is_minor: bool,
) {
    let tick_len = if is_minor {
        computed.tick_mark_minor
    } else {
        computed.tick_mark_major
    };
    let y_base = computed.height - computed.margin_bottom;
    let (y1, y2) = match layout.tick_align {
        TickAlign::Inside => (y_base - tick_len, y_base),
        TickAlign::Outside => (y_base, y_base + tick_len),
        TickAlign::Center => (y_base - tick_len * 0.5, y_base + tick_len * 0.5),
    };
    scene.add(Primitive::Line {
        x1: x,
        y1,
        x2: x,
        y2,
        stroke: Color::from(&theme.tick_color),
        stroke_width: computed.tick_stroke_width,
        stroke_dasharray: None,
    });
    if layout.tick_pos == TickPos::Both {
        let y_top = computed.margin_top;
        let (ty1, ty2) = match layout.tick_align {
            TickAlign::Inside => (y_top, y_top + tick_len),
            TickAlign::Outside => (y_top - tick_len, y_top),
            TickAlign::Center => (y_top - tick_len * 0.5, y_top + tick_len * 0.5),
        };
        scene.add(Primitive::Line {
            x1: x,
            y1: ty1,
            x2: x,
            y2: ty2,
            stroke: Color::from(&theme.tick_color),
            stroke_width: computed.tick_stroke_width,
            stroke_dasharray: None,
        });
    }
}

fn draw_y_tick(
    scene: &mut Scene,
    layout: &Layout,
    computed: &ComputedLayout,
    theme: &crate::render::theme::Theme,
    y: f64,
    is_minor: bool,
) {
    let tick_len = if is_minor {
        computed.tick_mark_minor
    } else {
        computed.tick_mark_major
    };
    let x_base = computed.margin_left;
    let (x1, x2) = match layout.tick_align {
        TickAlign::Inside => (x_base, x_base + tick_len),
        TickAlign::Outside => (x_base - tick_len, x_base),
        TickAlign::Center => (x_base - tick_len * 0.5, x_base + tick_len * 0.5),
    };
    scene.add(Primitive::Line {
        x1,
        y1: y,
        x2,
        y2: y,
        stroke: Color::from(&theme.tick_color),
        stroke_width: computed.tick_stroke_width,
        stroke_dasharray: None,
    });
    if layout.tick_pos == TickPos::Both && layout.y2_range.is_none() {
        let x_right = computed.width - computed.margin_right;
        let (tx1, tx2) = match layout.tick_align {
            TickAlign::Inside => (x_right - tick_len, x_right),
            TickAlign::Outside => (x_right, x_right + tick_len),
            TickAlign::Center => (x_right - tick_len * 0.5, x_right + tick_len * 0.5),
        };
        scene.add(Primitive::Line {
            x1: tx1,
            y1: y,
            x2: tx2,
            y2: y,
            stroke: Color::from(&theme.tick_color),
            stroke_width: computed.tick_stroke_width,
            stroke_dasharray: None,
        });
    }
}

/// Tracks state for x-axis label overlap handling across a tick loop.
pub(crate) struct XLabelPlacer {
    strategy: AxisLabelOverlap,
    tick_size: f64,
    rotate: Option<f64>,
    last_right: f64,
    row_right: [f64; 2],
}

impl XLabelPlacer {
    pub(crate) fn new(strategy: AxisLabelOverlap, tick_size: f64, rotate: Option<f64>) -> Self {
        Self {
            strategy,
            tick_size,
            rotate,
            last_right: f64::NEG_INFINITY,
            row_right: [f64::NEG_INFINITY; 2],
        }
    }

    fn footprint(&self, x: f64, label: &str, anchor: &TextAnchor) -> (f64, f64) {
        let text_w = measure_text_width(label, self.tick_size, FontStyle::Regular);
        let h_extent = match self.rotate {
            Some(angle) => text_w * angle.to_radians().cos().abs(),
            None => text_w,
        };
        match anchor {
            TextAnchor::End => (x - h_extent, x),
            TextAnchor::Start => (x, x + h_extent),
            TextAnchor::Middle => (x - h_extent / 2.0, x + h_extent / 2.0),
        }
    }

    /// Returns `Some(y_offset)` to draw the label at `base_y + y_offset`, or `None` to skip it.
    pub(crate) fn place(&mut self, x: f64, label: &str, anchor: &TextAnchor) -> Option<f64> {
        const GAP: f64 = 2.0;
        match &self.strategy {
            AxisLabelOverlap::Allow => Some(0.0),
            AxisLabelOverlap::Thin => {
                let (left, right) = self.footprint(x, label, anchor);
                if left < self.last_right + GAP {
                    None
                } else {
                    self.last_right = right;
                    Some(0.0)
                }
            }
            AxisLabelOverlap::Stagger => {
                let (left, right) = self.footprint(x, label, anchor);
                // Second row sits one real line height below the first; layout.rs
                // reserves the same so the lower row never clips the axis title.
                let row_h = line_height(self.tick_size, FontStyle::Regular);
                for row in 0..2usize {
                    if left >= self.row_right[row] + GAP {
                        self.row_right[row] = right;
                        return Some(row as f64 * row_h);
                    }
                }
                // Both rows occupied — still draw in row 0 to never drop labels.
                self.row_right[0] = right;
                Some(0.0)
            }
        }
    }
}

/// How a major-tick list was generated — determines how to find the tick
/// that would exist one step beyond each end, for [`extend_with_phantom_ticks`].
enum TickSpacing<'a> {
    /// Constant additive step: plain linear ticks, an explicit tick-step, or
    /// bin-aligned histogram ticks are all evenly spaced by construction.
    Arithmetic(f64),
    /// Log-scale ticks following the `[1, 2, 5] × 10^n` (or pure `10^n`)
    /// pattern selected by [`render_utils::log_multipliers`] for this span.
    Log { min: f64, max: f64 },
    /// Calendar-based ticks (one per day/week/month/... via [`DateTimeAxis`]).
    Calendar(&'a DateTimeAxis),
}

/// Extend a major-tick list with the tick that would exist one step beyond
/// each end, so [`render_utils::generate_minor_ticks`] also fills the
/// leading/trailing partial interval when the axis range doesn't stop on a
/// major tick. Ticks beyond the axis range are dropped by the caller after
/// minor generation.
///
/// Rather than extrapolating from the ratio/step of the outermost tick pair
/// (which breaks for log's non-constant `1,2,5` decade pattern and for
/// calendar units with irregular month/year lengths), this asks the same
/// rule that generated `ticks` for its actual next/previous value.
fn extend_with_phantom_ticks(ticks: &[f64], spacing: &TickSpacing) -> Vec<f64> {
    let (Some(&first), Some(&last)) = (ticks.first(), ticks.last()) else {
        return ticks.to_vec();
    };
    let (lo, hi) = match spacing {
        TickSpacing::Arithmetic(step) => (first - step, last + step),
        TickSpacing::Log { min, max } => {
            let multipliers = render_utils::log_multipliers(*min, *max);
            (
                render_utils::log_tick_before(first, multipliers),
                render_utils::log_tick_after(last, multipliers),
            )
        }
        TickSpacing::Calendar(dt) => (dt.tick_before(first), dt.tick_after(last)),
    };
    let mut out = Vec::with_capacity(ticks.len() + 2);
    out.push(lo);
    out.extend_from_slice(ticks);
    out.push(hi);
    out
}

/// Generate the minor-tick positions for one axis: extend `ticks` with the
/// phantom ends described by `spacing`, subdivide, then drop anything outside
/// the real axis `range`. Shared by both the x-axis and y-axis call sites so
/// they can't drift apart the way `resolve_axis_range` had to fix separately
/// for the x/y/y2 range-selection chains.
fn compute_minor_ticks(
    ticks: &[f64],
    spacing: &TickSpacing,
    range: (f64, f64),
    subdivisions: u32,
) -> Vec<f64> {
    render_utils::generate_minor_ticks(&extend_with_phantom_ticks(ticks, spacing), subdivisions)
        .into_iter()
        .filter(|t| *t >= range.0 && *t <= range.1)
        .collect()
}

/// The constant step of an evenly-spaced tick list (`generate_ticks`,
/// `generate_ticks_with_step`, and `generate_ticks_bin_aligned` are all
/// constant-step by construction), falling back to `fallback` when there
/// aren't enough ticks to measure a step directly.
fn arithmetic_step(ticks: &[f64], fallback: f64) -> f64 {
    if ticks.len() >= 2 {
        ticks[1] - ticks[0]
    } else {
        fallback
    }
}

/// Resolve one axis's major-tick list, tagged with how it was generated
/// (`TickSpacing`) so minor-tick extrapolation can later ask the same rule
/// for the tick that would exist just past each end. Shared by the x-axis
/// and y-axis call sites in `add_axes_and_grid` so they can't independently
/// drift out of sync — `bin_width` is `None` for y, which has no histogram
/// binning concept.
#[allow(clippy::too_many_arguments)]
fn resolve_axis_ticks(
    range: (f64, f64),
    tick_step: Option<f64>,
    bin_width: Option<f64>,
    datetime: Option<&DateTimeAxis>,
    log: bool,
    target_ticks: usize,
) -> (Vec<f64>, TickSpacing<'_>) {
    if let Some(step) = tick_step {
        (
            render_utils::generate_ticks_with_step(range.0, range.1, step),
            TickSpacing::Arithmetic(step),
        )
    } else if let Some(bw) = bin_width {
        let ticks = render_utils::generate_ticks_bin_aligned(range.0, range.1, bw, target_ticks);
        let step = arithmetic_step(&ticks, bw);
        (ticks, TickSpacing::Arithmetic(step))
    } else if let Some(dt) = datetime {
        (
            dt.generate_ticks(range.0, range.1),
            TickSpacing::Calendar(dt),
        )
    } else if log {
        (
            render_utils::generate_ticks_log(range.0, range.1),
            TickSpacing::Log {
                min: range.0,
                max: range.1,
            },
        )
    } else {
        let ticks = render_utils::generate_ticks(range.0, range.1, target_ticks);
        let fallback = render_utils::compute_tick_step(range.0, range.1, target_ticks);
        let step = arithmetic_step(&ticks, fallback);
        (ticks, TickSpacing::Arithmetic(step))
    }
}

pub fn add_axes_and_grid(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    let map_x = |x| computed.map_x(x);
    let map_y = |y| computed.map_y(y);

    let theme = &computed.theme;

    // Always compute tick positions for grid lines, tagged with how they were
    // generated so minor-tick extrapolation below can ask the same rule for
    // the tick that would exist just past each end (see `TickSpacing`).
    let (x_ticks, x_spacing) = resolve_axis_ticks(
        computed.x_range,
        computed.x_tick_step,
        computed.x_bin_width,
        layout.x_datetime.as_ref(),
        layout.log_x,
        computed.x_ticks,
    );
    let (y_ticks, y_spacing) = resolve_axis_ticks(
        computed.y_range,
        computed.y_tick_step,
        None,
        layout.y_datetime.as_ref(),
        layout.log_y,
        computed.y_ticks,
    );

    // Generate minors as if one more major existed just beyond each end so the
    // bands past the outer majors get filled, then drop any that fall outside the
    // axis range. The filtered lists feed both the gridlines and the tick marks.
    let x_minor = computed
        .minor_ticks
        .map(|n| compute_minor_ticks(&x_ticks, &x_spacing, computed.x_range, n));
    let y_minor = computed
        .minor_ticks
        .map(|n| compute_minor_ticks(&y_ticks, &y_spacing, computed.y_range, n));

    // Draw minor gridlines (before major so major renders on top)
    if computed.show_minor_grid && layout.x_categories.is_none() {
        if let Some(ref mx) = x_minor {
            for tx in mx {
                let x = map_x(*tx);
                scene.add(Primitive::Line {
                    x1: x,
                    y1: computed.margin_top,
                    x2: x,
                    y2: computed.height - computed.margin_bottom,
                    stroke: Color::from(&theme.grid_color),
                    stroke_width: computed.grid_stroke_width * 0.5,
                    stroke_dasharray: None,
                });
            }
        }
        if let Some(ref my) = y_minor {
            for ty in my {
                let y = map_y(*ty);
                scene.add(Primitive::Line {
                    x1: computed.margin_left,
                    y1: y,
                    x2: computed.width - computed.margin_right,
                    y2: y,
                    stroke: Color::from(&theme.grid_color),
                    stroke_width: computed.grid_stroke_width * 0.5,
                    stroke_dasharray: None,
                });
            }
        }
    }

    // Draw grid lines (always, regardless of suppress flags)
    if layout.show_grid {
        // Vertical grid lines (skip for category x-axes like boxplot, bar, violin)
        if layout.x_categories.is_none() && layout.y_categories.is_none() {
            let x_axis_edge = computed.margin_left;
            for tx in x_ticks.iter() {
                // Skip gridlines that land on (or within 1 px of) the y-axis line —
                // they would be invisible under the axis stroke.  Use pixel proximity
                // instead of `i == 0` so that equal_aspect-expanded ranges still draw
                // all interior ticks correctly.
                if !layout.log_x
                    && layout.x_datetime.is_none()
                    && (map_x(*tx) - x_axis_edge).abs() < 1.0
                {
                    continue;
                }
                let x = map_x(*tx);
                scene.add(Primitive::Line {
                    x1: x,
                    y1: computed.margin_top,
                    x2: x,
                    y2: computed.height - computed.margin_bottom,
                    stroke: Color::from(&theme.grid_color),
                    stroke_width: computed.grid_stroke_width,
                    stroke_dasharray: None,
                });
            }
        }
        // Horizontal grid lines (draw when y-axis is numeric)
        if layout.y_categories.is_none() {
            let y_axis_edge = computed.height - computed.margin_bottom;
            for ty in y_ticks.iter() {
                // Same proximity check for the x-axis edge.
                if !layout.log_y
                    && layout.y_datetime.is_none()
                    && (map_y(*ty) - y_axis_edge).abs() < 1.0
                {
                    continue;
                }
                let y = map_y(*ty);
                scene.add(Primitive::Line {
                    x1: computed.margin_left,
                    y1: y,
                    x2: computed.width - computed.margin_right,
                    y2: y,
                    stroke: Color::from(&theme.grid_color),
                    stroke_width: computed.grid_stroke_width,
                    stroke_dasharray: None,
                });
            }
        }
    }

    // Draw axes on top of grid lines so grid never bleeds over the axis borders.
    // X axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.height - computed.margin_bottom,
        x2: computed.width - computed.margin_right,
        y2: computed.height - computed.margin_bottom,
        stroke: Color::from(&theme.axis_color),
        stroke_width: computed.axis_line_width,
        stroke_dasharray: None,
    });

    // Y axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.margin_top,
        x2: computed.margin_left,
        y2: computed.height - computed.margin_bottom,
        stroke: Color::from(&theme.axis_color),
        stroke_width: computed.axis_line_width,
        stroke_dasharray: None,
    });

    // Draw tick marks and labels
    if let Some(categories) = &layout.y_categories {
        if !layout.suppress_y_ticks {
            for (i, label) in categories.iter().enumerate() {
                let y_val = i as f64 + 1.0;
                let y_pos = computed.map_y(y_val);

                scene.add(Primitive::Text {
                    x: computed.margin_left - computed.tick_label_margin,
                    y: y_pos + center_offset(computed.tick_size as f64, FontStyle::Regular),
                    content: label.clone(),
                    size: computed.tick_size,
                    anchor: TextAnchor::End,
                    rotate: None,
                    bold: false,
                    color: None,
                });

                draw_y_tick(scene, layout, computed, theme, y_pos, false);
            }
        }
        if !layout.suppress_x_ticks {
            if let Some(x_cats) = &layout.x_categories {
                // Both x and y are category axes (e.g. DotPlot): draw x category labels
                let mut placer = XLabelPlacer::new(
                    computed.x_label_overlap.clone(),
                    computed.tick_size as f64,
                    layout.x_tick_rotate,
                );
                for (i, label) in x_cats.iter().enumerate() {
                    let x_val = i as f64 + 1.0;
                    let x_pos = computed.map_x(x_val);
                    let (anchor, rotate) = match layout.x_tick_rotate {
                        Some(angle) if angle < 0.0 => (TextAnchor::End, Some(angle)),
                        Some(angle) => (TextAnchor::Start, Some(angle)),
                        None => (TextAnchor::Middle, None),
                    };
                    let base_y = computed.height - computed.margin_bottom
                        + computed.tick_mark_major
                        + ascent(computed.tick_size as f64, FontStyle::Regular);
                    if let Some(y_off) = placer.place(x_pos, label, &anchor) {
                        scene.add(Primitive::Text {
                            x: x_pos,
                            y: base_y + y_off,
                            content: label.clone(),
                            size: computed.tick_size,
                            anchor,
                            rotate,
                            bold: false,
                            color: None,
                        });
                    }

                    draw_x_tick(scene, layout, computed, theme, x_pos, false);
                }
            } else {
                let mut placer = XLabelPlacer::new(
                    computed.x_label_overlap.clone(),
                    computed.tick_size as f64,
                    layout.x_tick_rotate,
                );
                for tx in x_ticks.iter() {
                    let x = map_x(*tx);

                    draw_x_tick(scene, layout, computed, theme, x, false);

                    let label = if let Some(ref dt) = layout.x_datetime {
                        dt.format_tick(*tx)
                    } else if layout.log_x && matches!(computed.x_tick_format, TickFormat::Auto) {
                        render_utils::format_log_tick(*tx)
                    } else {
                        computed.x_tick_format.format(*tx)
                    };
                    let (anchor, rotate) = match layout.x_tick_rotate {
                        Some(angle) => (TextAnchor::End, Some(angle)),
                        None => (TextAnchor::Middle, None),
                    };
                    let base_y = computed.height - computed.margin_bottom
                        + computed.tick_mark_major
                        + ascent(computed.tick_size as f64, FontStyle::Regular);
                    if let Some(y_off) = placer.place(x, &label, &anchor) {
                        scene.add(Primitive::Text {
                            x,
                            y: base_y + y_off,
                            content: label,
                            size: computed.tick_size,
                            anchor,
                            rotate,
                            bold: false,
                            color: None,
                        });
                    }
                }
            }
        }
    } else if let Some(categories) = &layout.x_categories {
        if !layout.suppress_x_ticks {
            let mut placer = XLabelPlacer::new(
                computed.x_label_overlap.clone(),
                computed.tick_size as f64,
                layout.x_tick_rotate,
            );
            for (i, label) in categories.iter().enumerate() {
                let x_val = i as f64 + 1.0;
                let x_pos = computed.map_x(x_val);
                let (anchor, rotate) = match layout.x_tick_rotate {
                    Some(angle) if angle < 0.0 => (TextAnchor::End, Some(angle)),
                    Some(angle) => (TextAnchor::Start, Some(angle)),
                    None => (TextAnchor::Middle, None),
                };
                let base_y = computed.height - computed.margin_bottom
                    + computed.tick_mark_major
                    + ascent(computed.tick_size as f64, FontStyle::Regular);
                if let Some(y_off) = placer.place(x_pos, label, &anchor) {
                    scene.add(Primitive::Text {
                        x: x_pos,
                        y: base_y + y_off,
                        content: label.clone(),
                        size: computed.tick_size,
                        anchor,
                        rotate,
                        bold: false,
                        color: None,
                    });
                }

                draw_x_tick(scene, layout, computed, theme, x_pos, false);
            }
        }

        if !layout.suppress_y_ticks {
            for ty in y_ticks.iter() {
                let y = map_y(*ty);
                draw_y_tick(scene, layout, computed, theme, y, false);

                let label = if let Some(ref dt) = layout.y_datetime {
                    dt.format_tick(*ty)
                } else if layout.log_y && matches!(computed.y_tick_format, TickFormat::Auto) {
                    render_utils::format_log_tick(*ty)
                } else {
                    computed.y_tick_format.format(*ty)
                };
                scene.add(Primitive::Text {
                    x: computed.margin_left - computed.tick_label_margin,
                    y: y + center_offset(computed.tick_size as f64, FontStyle::Regular),
                    content: label,
                    size: computed.tick_size,
                    anchor: TextAnchor::End,
                    rotate: None,
                    bold: false,
                    color: None,
                });
            }
        }
    }
    // regular axes
    else {
        if !layout.suppress_x_ticks {
            let mut placer = XLabelPlacer::new(
                computed.x_label_overlap.clone(),
                computed.tick_size as f64,
                layout.x_tick_rotate,
            );
            for tx in x_ticks.iter() {
                let x = map_x(*tx);

                draw_x_tick(scene, layout, computed, theme, x, false);

                let label = if let Some(ref dt) = layout.x_datetime {
                    dt.format_tick(*tx)
                } else if layout.log_x && matches!(computed.x_tick_format, TickFormat::Auto) {
                    render_utils::format_log_tick(*tx)
                } else {
                    computed.x_tick_format.format(*tx)
                };
                let (anchor, rotate) = match layout.x_tick_rotate {
                    Some(angle) if angle < 0.0 => (TextAnchor::End, Some(angle)),
                    Some(angle) => (TextAnchor::Start, Some(angle)),
                    None => (TextAnchor::Middle, None),
                };
                let base_y = computed.height - computed.margin_bottom
                    + computed.tick_mark_major
                    + ascent(computed.tick_size as f64, FontStyle::Regular);
                if let Some(y_off) = placer.place(x, &label, &anchor) {
                    scene.add(Primitive::Text {
                        x,
                        y: base_y + y_off,
                        content: label,
                        size: computed.tick_size,
                        anchor,
                        rotate,
                        bold: false,
                        color: None,
                    });
                }
            }
        }

        if !layout.suppress_y_ticks {
            for ty in y_ticks.iter() {
                let y = map_y(*ty);

                draw_y_tick(scene, layout, computed, theme, y, false);

                let label = if let Some(ref dt) = layout.y_datetime {
                    dt.format_tick(*ty)
                } else if layout.log_y && matches!(computed.y_tick_format, TickFormat::Auto) {
                    render_utils::format_log_tick(*ty)
                } else {
                    computed.y_tick_format.format(*ty)
                };
                scene.add(Primitive::Text {
                    x: computed.margin_left - computed.tick_label_margin,
                    y: y + center_offset(computed.tick_size as f64, FontStyle::Regular),
                    content: label,
                    size: computed.tick_size,
                    anchor: TextAnchor::End,
                    rotate: None,
                    bold: false,
                    color: None,
                });
            }
        }

        // Minor tick marks (no label)
        if !layout.suppress_x_ticks {
            if let Some(ref mx) = x_minor {
                for tx in mx {
                    let x = map_x(*tx);
                    draw_x_tick(scene, layout, computed, theme, x, true);
                }
            }
        }
        if !layout.suppress_y_ticks {
            if let Some(ref my) = y_minor {
                for ty in my {
                    let y = map_y(*ty);
                    draw_y_tick(scene, layout, computed, theme, y, true);
                }
            }
        }
    }

    if layout.axis_line == AxisLine::Box || layout.tick_pos == TickPos::Both {
        // Top axis
        scene.add(Primitive::Line {
            x1: computed.margin_left,
            y1: computed.margin_top,
            x2: computed.width - computed.margin_right,
            y2: computed.margin_top,
            stroke: Color::from(&theme.axis_color),
            stroke_width: computed.axis_line_width,
            stroke_dasharray: None,
        });

        // Right axis (drawn here only if y2 axis is NOT present)
        if layout.y2_range.is_none() {
            scene.add(Primitive::Line {
                x1: computed.width - computed.margin_right,
                y1: computed.margin_top,
                x2: computed.width - computed.margin_right,
                y2: computed.height - computed.margin_bottom,
                stroke: Color::from(&theme.axis_color),
                stroke_width: computed.axis_line_width,
                stroke_dasharray: None,
            });
        }
    }
}

pub fn add_y2_axis(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    let Some((y2_min, y2_max)) = computed.y2_range else {
        return;
    };
    let theme = &computed.theme;
    let axis_x = computed.width - computed.margin_right;

    // Right y-axis line
    scene.add(Primitive::Line {
        x1: axis_x,
        y1: computed.margin_top,
        x2: axis_x,
        y2: computed.height - computed.margin_bottom,
        stroke: Color::from(&theme.axis_color),
        stroke_width: computed.axis_line_width,
        stroke_dasharray: None,
    });

    if layout.suppress_y2_ticks {
        return;
    }

    let y2_ticks = if layout.log_y2 {
        render_utils::generate_ticks_log(y2_min, y2_max)
    } else {
        render_utils::generate_ticks(y2_min, y2_max, computed.y_ticks)
    };

    for ty in y2_ticks.iter() {
        let y = computed.map_y2(*ty);

        let (tx1, tx2) = match layout.tick_align {
            TickAlign::Inside => (axis_x - computed.tick_mark_major, axis_x),
            TickAlign::Outside => (axis_x, axis_x + computed.tick_mark_major),
            TickAlign::Center => (
                axis_x - computed.tick_mark_major * 0.5,
                axis_x + computed.tick_mark_major * 0.5,
            ),
        };

        scene.add(Primitive::Line {
            x1: tx1,
            y1: y,
            x2: tx2,
            y2: y,
            stroke: Color::from(&theme.tick_color),
            stroke_width: computed.tick_stroke_width,
            stroke_dasharray: None,
        });

        let label = if layout.log_y2 && matches!(computed.y2_tick_format, TickFormat::Auto) {
            render_utils::format_log_tick(*ty)
        } else {
            computed.y2_tick_format.format(*ty)
        };
        scene.add(Primitive::Text {
            x: axis_x + computed.tick_label_margin,
            y: y + center_offset(computed.tick_size as f64, FontStyle::Regular),
            content: label,
            size: computed.tick_size,
            anchor: TextAnchor::Start,
            rotate: None,
            bold: false,
            color: None,
        });
    }

    if let Some(ref label) = layout.y2_label {
        let lines = render_utils::wrap_or_single(label, computed.y2_label_wrap);
        let ls = computed.label_size as f64;
        let lh = line_height(ls, FontStyle::Regular);
        let (dx, dy) = layout.y2_label_offset;
        // Base x for the rightmost (first) line; additional lines shift left.
        let base_x = axis_x + computed.y2_axis_width - ls * 0.5 + dx;
        let base_y = computed.margin_top + computed.plot_height() / 2.0 + dy;
        for (i, line) in lines.iter().enumerate() {
            scene.add(Primitive::Text {
                x: base_x - i as f64 * lh,
                y: base_y,
                content: line.clone(),
                size: computed.label_size,
                anchor: TextAnchor::Middle,
                rotate: Some(90.0),
                bold: false,
                color: None,
            });
        }
    }
}

/// Draws the secondary X-axis on top of the plot, mirroring [`add_y2_axis`] (which
/// draws the secondary Y-axis on the right) — used by horizontal `ParetoPlot` for
/// its cumulative-% line, since horizontal mode puts categories on Y and values on
/// X, so the cumulative line needs its own X-axis rather than the Y-based `y2`.
pub fn add_x2_axis(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    let Some((x2_min, x2_max)) = computed.x2_range else {
        return;
    };
    let theme = &computed.theme;
    let axis_y = computed.margin_top;

    // Top x-axis line
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: axis_y,
        x2: computed.width - computed.margin_right,
        y2: axis_y,
        stroke: Color::from(&theme.axis_color),
        stroke_width: computed.axis_line_width,
        stroke_dasharray: None,
    });

    if layout.suppress_x2_ticks {
        return;
    }

    let x2_ticks = if layout.log_x2 {
        render_utils::generate_ticks_log(x2_min, x2_max)
    } else {
        render_utils::generate_ticks(x2_min, x2_max, computed.x_ticks)
    };

    for tx in x2_ticks.iter() {
        let x = computed.map_x2(*tx);

        let (ty1, ty2) = match layout.tick_align {
            TickAlign::Inside => (axis_y, axis_y + computed.tick_mark_major),
            TickAlign::Outside => (axis_y - computed.tick_mark_major, axis_y),
            TickAlign::Center => (
                axis_y - computed.tick_mark_major * 0.5,
                axis_y + computed.tick_mark_major * 0.5,
            ),
        };

        scene.add(Primitive::Line {
            x1: x,
            y1: ty1,
            x2: x,
            y2: ty2,
            stroke: Color::from(&theme.tick_color),
            stroke_width: computed.tick_stroke_width,
            stroke_dasharray: None,
        });

        let label = if layout.log_x2 && matches!(computed.x2_tick_format, TickFormat::Auto) {
            render_utils::format_log_tick(*tx)
        } else {
            computed.x2_tick_format.format(*tx)
        };
        scene.add(Primitive::Text {
            x,
            y: axis_y - computed.tick_label_margin,
            content: label,
            size: computed.tick_size,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
            color: None,
        });
    }

    if let Some(ref label) = layout.x2_label {
        let lines = render_utils::wrap_or_single(label, computed.x2_label_wrap);
        let ls = computed.label_size as f64;
        let lh = line_height(ls, FontStyle::Regular);
        let (dx, dy) = layout.x2_label_offset;
        let base_x = computed.margin_left + computed.plot_width() / 2.0 + dx;
        let base_y = axis_y - computed.x2_axis_height + ls * 0.5 + dy;
        for (i, line) in lines.iter().enumerate() {
            scene.add(Primitive::Text {
                x: base_x,
                y: base_y + i as f64 * lh,
                content: line.clone(),
                size: computed.label_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
                color: None,
            });
        }
    }
}

pub fn add_labels_and_title(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    let ls = computed.label_size as f64;
    // Real line height for stacking wrapped label lines (1.0em let them overlap).
    let lh = line_height(ls, FontStyle::Regular);

    // X-axis title. Drawn whenever an x_label is set, even when the numeric ticks are
    // suppressed — Manhattan draws its own chromosome labels but still wants the
    // "Chromosome" title. Every branch that can reach here reserves a line for it in the
    // bottom margin (layout.rs), including the suppressed-tick + no-rotation case, so it
    // clears those labels rather than overprinting them. Figure subplots that hide their
    // x-axis clear x_label, so they render nothing here.
    if let Some(label) = &layout.x_label {
        let lines = render_utils::wrap_or_single(label, computed.x_label_wrap);
        let (dx, dy) = layout.x_label_offset;
        let default_x = computed.margin_left + computed.plot_width() / 2.0;
        // Subtract legend_bottom_extra and risk_table_extra so the x-label stays in the
        // axis area rather than drifting into the OutsideBottom legend or the survival
        // number-at-risk table band below it.
        let default_y = computed.height
            - computed.legend_bottom_extra
            - computed.risk_table_extra
            - ls * 0.5
            - (lines.len() as f64 - 1.0) * lh;
        let (lx, ly) = computed.dice_x_label_pos.unwrap_or((default_x, default_y));
        for (i, line) in lines.iter().enumerate() {
            scene.add(Primitive::Text {
                x: lx + dx,
                y: ly + dy + i as f64 * lh,
                content: line.clone(),
                size: computed.label_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
                color: None,
            });
        }
    }

    // Y Axis Label (rotated -90°; wrapped lines stack horizontally in unrotated space)
    if !layout.suppress_y_ticks {
        if let Some(label) = &layout.y_label {
            let lines = render_utils::wrap_or_single(label, computed.y_label_wrap);
            let (dx, dy) = layout.y_label_offset;
            // Base x for the leftmost (first) line; subsequent lines step right by `ls`.
            // The .max() floor keeps all lines on-canvas for very narrow plots, but on
            // plots that are too narrow for many wrapped lines the lines will overlap.
            let default_x = (computed.margin_left
                - 8.0
                - computed.y_tick_label_px
                - 5.0
                - ls * 0.5
                - (lines.len() as f64 - 1.0) * lh)
                .max(ls * 0.5 + 8.0);
            let default_y = computed.margin_top + computed.plot_height() / 2.0;
            let (lx, ly) = computed.dice_y_label_pos.unwrap_or((default_x, default_y));
            for (i, line) in lines.iter().enumerate() {
                scene.add(Primitive::Text {
                    x: lx + dx + i as f64 * lh,
                    y: ly + dy,
                    content: line.clone(),
                    size: computed.label_size,
                    anchor: TextAnchor::Middle,
                    rotate: Some(-90.0),
                    bold: false,
                    color: None,
                });
            }
        }
    }

    // Title
    if let Some(title) = &layout.title {
        let lines = render_utils::wrap_or_single(title, computed.title_wrap);
        let ts = computed.title_size as f64;
        // Stack wrapped title lines by the real line height (was 1.0em, which let
        // adjacent lines' ascenders/descenders touch) and drop the first baseline by
        // the real ascent.
        let tlh = line_height(ts, FontStyle::Regular);
        let total_height = lines.len() as f64 * tlh;
        let cx = computed.margin_left + computed.plot_width() / 2.0;
        // Use title_y (derived from base margin before notation tiers) so that
        // BrickPlot notation labels don't push the title into the annotation zone.
        let start_y = computed.title_y - total_height / 2.0 + ascent(ts, FontStyle::Regular);
        for (i, line) in lines.iter().enumerate() {
            scene.add(Primitive::Text {
                x: cx,
                y: start_y + i as f64 * tlh,
                content: line.clone(),
                size: computed.title_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
                color: None,
            });
        }
    }

    // Subtitle: a smaller, muted line (or lines) just below the title block.
    // Positioned with the same real DejaVu metrics as the title above (line
    // height + ascent); the height reserved in `ComputedLayout::from_layout`
    // uses the same line height, so the plot is pushed down, not overlapped.
    if let Some(subtitle) = &layout.subtitle {
        let lines = render_utils::wrap_or_single(subtitle, computed.subtitle_wrap);
        let ts = computed.title_size as f64;
        let sts = computed.subtitle_size as f64;
        let slh = line_height(sts, FontStyle::Regular);
        let cx = computed.margin_left + computed.plot_width() / 2.0;
        // The title occupies `title_lines · line_height(ts)` centred on `title_y`,
        // so its bottom edge is `title_y + that / 2`. Drop the first subtitle
        // baseline by its ascent so the line clears the title. `title_lines` is
        // computed once in `from_layout` and reused here to stay in lockstep with
        // the height it reserved.
        let title_block_h = computed.title_lines as f64 * line_height(ts, FontStyle::Regular);
        let start_y = computed.title_y + title_block_h / 2.0 + ascent(sts, FontStyle::Regular);
        let color = muted_subtitle_color(scene);
        for (i, line) in lines.iter().enumerate() {
            scene.add(Primitive::Text {
                x: cx,
                y: start_y + i as f64 * slh,
                content: line.clone(),
                size: sts as u32,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
                color: Some(color.clone()),
            });
        }
    }
}

/// The subtitle colour: the effective title colour (the scene's text colour, or
/// black) blended `SUBTITLE_MUTE` of the way toward the background (the scene's
/// background, or white), so it reads as a muted version of the title in both
/// light and dark themes rather than a fixed grey.
///
/// Muting needs both colours as solid RGB. If either is a CSS colour kuva can't
/// resolve (e.g. an `hsl()`/functional-notation theme), fall back to the title's
/// own colour un-muted — correct hue and readable, just not dimmed — rather than
/// fabricating a wrong grey.
pub(crate) fn muted_subtitle_color(scene: &Scene) -> Color {
    let title_color = scene
        .text_color
        .as_deref()
        .map(Color::from)
        .unwrap_or(Color::Rgb(0, 0, 0));
    let background = scene
        .background_color
        .as_deref()
        .map(Color::from)
        .unwrap_or(Color::Rgb(255, 255, 255));
    match (&title_color, &background) {
        (Color::Rgb(tr, tg, tb), Color::Rgb(br, bg, bb)) => {
            let mix = |from: u8, to: u8| {
                (from as f64 * (1.0 - SUBTITLE_MUTE) + to as f64 * SUBTITLE_MUTE).round() as u8
            };
            Color::Rgb(mix(*tr, *br), mix(*tg, *bg), mix(*tb, *bb))
        }
        _ => title_color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression (PR #101 follow-up): `extend_with_phantom_ticks` used to be
    // handed a bare `bool` (`layout.log_x`) that could disagree with which
    // generator actually produced the tick list — e.g. an explicit
    // `x_tick_step` on a log-scale layout produces a *linear* tick list, but
    // the old code still applied geometric (ratio-based) extrapolation to it
    // because `layout.log_x` was `true`. `TickSpacing` is now set from the
    // branch that actually generated the ticks, so this can't happen: an
    // explicit step always carries `TickSpacing::Arithmetic`, regardless of
    // whether logging is also enabled elsewhere on the layout.
    #[test]
    fn arithmetic_ticks_extrapolate_by_the_real_step_not_a_guessed_ratio() {
        let ticks = vec![5.0, 10.0, 15.0, 20.0]; // from an explicit --x-tick-step 5
        let extended = extend_with_phantom_ticks(&ticks, &TickSpacing::Arithmetic(5.0));
        assert_eq!(extended, vec![0.0, 5.0, 10.0, 15.0, 20.0, 25.0]);
    }

    // Regression: `resolve_axis_ticks` must resolve `TickSpacing::Arithmetic`
    // — not `TickSpacing::Log` — when `--log-x`/`--log-y` is combined with an
    // explicit `--x-tick-step`/`--y-tick-step`, since the explicit step takes
    // precedence over log-scale tick generation and produces a linear list.
    #[test]
    fn log_axis_with_explicit_tick_step_resolves_to_arithmetic_spacing() {
        let (ticks, spacing) = resolve_axis_ticks(
            (1.0, 20.0),
            Some(5.0), // explicit tick step
            None,
            None,
            true, // log_x also enabled
            5,
        );
        assert_eq!(ticks, vec![5.0, 10.0, 15.0, 20.0]);
        assert!(
            matches!(spacing, TickSpacing::Arithmetic(step) if step == 5.0),
            "explicit tick step must win over log-scale extrapolation"
        );
    }

    #[test]
    fn log_ticks_extrapolate_past_a_5x_multiplier_to_the_next_decade() {
        let ticks = vec![1.0, 2.0, 5.0, 10.0, 20.0]; // decades<=3 → [1,2,5] pattern
        let extended = extend_with_phantom_ticks(
            &ticks,
            &TickSpacing::Log {
                min: 1.0,
                max: 35.0,
            },
        );
        // Naive ratio extrapolation (20/10=2x) would wrongly give 40.0 here;
        // the real next tick in the 1-2-5 pattern is 50.0.
        assert_eq!(extended, vec![0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0]);
    }

    #[test]
    fn calendar_ticks_extrapolate_by_calendar_month_not_a_fixed_delta() {
        use crate::render::datetime::ymd;
        let dt = DateTimeAxis::months("%Y-%m-%d");
        let ticks = vec![ymd(2024, 12, 1), ymd(2025, 1, 1), ymd(2025, 2, 1)];
        let extended = extend_with_phantom_ticks(&ticks, &TickSpacing::Calendar(&dt));
        // A fixed 31-day step from Feb 1 would overshoot Mar 1; the calendar
        // step lands exactly on it.
        assert_eq!(extended[0], ymd(2024, 11, 1));
        assert_eq!(*extended.last().unwrap(), ymd(2025, 3, 1));
    }

    #[test]
    fn phantom_ticks_generate_minors_covering_the_leading_and_trailing_bands() {
        // End-to-end sanity: a scatter-style linear axis range that doesn't
        // land on a major tick (e.g. an explicit x_axis_min/max) must get
        // minors both before the first major and after the last one.
        let ticks = vec![10.0, 20.0, 30.0];
        let minors = compute_minor_ticks(&ticks, &TickSpacing::Arithmetic(10.0), (3.0, 34.0), 5);
        assert!(
            minors.iter().any(|&m| m < 10.0),
            "expected a minor tick in the leading band below 10.0, got {minors:?}"
        );
        assert!(
            minors.iter().any(|&m| m > 30.0),
            "expected a minor tick in the trailing band above 30.0, got {minors:?}"
        );
        assert!(
            minors.iter().all(|&m| m >= 3.0 && m <= 34.0),
            "no minor tick should fall outside the axis range, got {minors:?}"
        );
    }
}
