use crate::render::render::{Scene, Primitive, TextAnchor};
use crate::render::layout::{Layout, ComputedLayout, TickFormat};
use crate::render::render_utils;
use crate::render::color::Color;



pub fn add_axes_and_grid(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {

    let map_x = |x| computed.map_x(x);
    let map_y = |y| computed.map_y(y);

    let theme = &computed.theme;

    // Always compute tick positions for grid lines
    let x_ticks: Vec<f64> = if let Some(step) = computed.x_tick_step {
        render_utils::generate_ticks_with_step(computed.x_range.0, computed.x_range.1, step)
    } else if let Some(bw) = computed.x_bin_width {
        render_utils::generate_ticks_bin_aligned(
            computed.x_range.0, computed.x_range.1, bw, computed.x_ticks)
    } else if let Some(ref dt) = layout.x_datetime {
        dt.generate_ticks(computed.x_range.0, computed.x_range.1)
    } else if layout.log_x {
        render_utils::generate_ticks_log(computed.x_range.0, computed.x_range.1)
    } else {
        render_utils::generate_ticks(computed.x_range.0, computed.x_range.1, computed.x_ticks)
    };
    let y_ticks: Vec<f64> = if let Some(step) = computed.y_tick_step {
        render_utils::generate_ticks_with_step(computed.y_range.0, computed.y_range.1, step)
    } else if let Some(ref dt) = layout.y_datetime {
        dt.generate_ticks(computed.y_range.0, computed.y_range.1)
    } else if layout.log_y {
        render_utils::generate_ticks_log(computed.y_range.0, computed.y_range.1)
    } else {
        render_utils::generate_ticks(computed.y_range.0, computed.y_range.1, computed.y_ticks)
    };

    let x_minor = computed.minor_ticks.map(|n| render_utils::generate_minor_ticks(&x_ticks, n));
    let y_minor = computed.minor_ticks.map(|n| render_utils::generate_minor_ticks(&y_ticks, n));

    // Draw minor gridlines (before major so major renders on top)
    if computed.show_minor_grid && layout.x_categories.is_none() {
        if let Some(ref mx) = x_minor {
            for tx in mx {
                let x = map_x(*tx);
                scene.add(Primitive::Line {
                    x1: x, y1: computed.margin_top,
                    x2: x, y2: computed.height - computed.margin_bottom,
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
                    x1: computed.margin_left, y1: y,
                    x2: computed.width - computed.margin_right, y2: y,
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
                if !layout.log_x && layout.x_datetime.is_none()
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
                if !layout.log_y && layout.y_datetime.is_none()
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
                    y: y_pos + computed.tick_size as f64 * 0.35,
                    content: label.clone(),
                    size: computed.tick_size,
                    anchor: TextAnchor::End,
                    rotate: None,
                    bold: false,
                    color: None,
                });

                scene.add(Primitive::Line {
                    x1: computed.margin_left - computed.tick_mark_major,
                    y1: y_pos,
                    x2: computed.margin_left,
                    y2: y_pos,
                    stroke: Color::from(&theme.tick_color),
                    stroke_width: computed.tick_stroke_width,
                    stroke_dasharray: None,
                });
            }
        }
        if !layout.suppress_x_ticks {
            if let Some(x_cats) = &layout.x_categories {
                // Both x and y are category axes (e.g. DotPlot): draw x category labels
                for (i, label) in x_cats.iter().enumerate() {
                    let x_val = i as f64 + 1.0;
                    let x_pos = computed.map_x(x_val);
                    let (anchor, rotate) = match layout.x_tick_rotate {
                        Some(angle) if angle < 0.0 => (TextAnchor::End,   Some(angle)),
                        Some(angle)                => (TextAnchor::Start,  Some(angle)),
                        None                       => (TextAnchor::Middle, None),
                    };

                    scene.add(Primitive::Text {
                        x: x_pos,
                        y: computed.height - computed.margin_bottom + computed.tick_mark_major + computed.tick_size as f64,
                        content: label.clone(),
                        size: computed.tick_size,
                        anchor,
                        rotate,
                        bold: false,
                        color: None,
                    });

                    scene.add(Primitive::Line {
                        x1: x_pos,
                        y1: computed.height - computed.margin_bottom,
                        x2: x_pos,
                        y2: computed.height - computed.margin_bottom + computed.tick_mark_major,
                        stroke: Color::from(&theme.tick_color),
                        stroke_width: computed.tick_stroke_width,
                        stroke_dasharray: None,
                    });
                }
            } else {
                for tx in x_ticks.iter() {
                    let x = map_x(*tx);

                    scene.add(Primitive::Line {
                        x1: x,
                        y1: computed.height - computed.margin_bottom,
                        x2: x,
                        y2: computed.height - computed.margin_bottom + computed.tick_mark_major,
                        stroke: Color::from(&theme.tick_color),
                        stroke_width: computed.tick_stroke_width,
                        stroke_dasharray: None,
                    });

                    let label = if let Some(ref dt) = layout.x_datetime {
                        dt.format_tick(*tx)
                    } else if layout.log_x && matches!(computed.x_tick_format, TickFormat::Auto) {
                        render_utils::format_log_tick(*tx)
                    } else {
                        computed.x_tick_format.format(*tx)
                    };
                    let (anchor, rotate) = match layout.x_tick_rotate {
                        Some(angle) => (TextAnchor::End, Some(angle)),
                        None        => (TextAnchor::Middle, None),
                    };
                    scene.add(Primitive::Text {
                        x,
                        y: computed.height - computed.margin_bottom + computed.tick_mark_major + computed.tick_size as f64,
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
    else if let Some(categories) = &layout.x_categories {
        if !layout.suppress_x_ticks {
            for (i, label) in categories.iter().enumerate() {
                let x_val = i as f64 + 1.0;
                let x_pos = computed.map_x(x_val);
                let (anchor, rotate) = match layout.x_tick_rotate {
                    Some(angle) if angle < 0.0 => (TextAnchor::End,   Some(angle)),
                    Some(angle)                => (TextAnchor::Start,  Some(angle)),
                    None                       => (TextAnchor::Middle, None),
                };
                scene.add(Primitive::Text {
                    x: x_pos,
                    y: computed.height - computed.margin_bottom + computed.tick_mark_major + computed.tick_size as f64,
                    content: label.clone(),
                    size: computed.tick_size,
                    anchor,
                    rotate,
                    bold: false,
                    color: None,
                });

                scene.add(Primitive::Line {
                    x1: x_pos,
                    y1: computed.height - computed.margin_bottom,
                    x2: x_pos,
                    y2: computed.height - computed.margin_bottom + computed.tick_mark_major,
                    stroke: Color::from(&theme.tick_color),
                    stroke_width: computed.tick_stroke_width,
                    stroke_dasharray: None,
                });
            }
        }

        if !layout.suppress_y_ticks {
            for ty in y_ticks.iter() {
                let y = map_y(*ty);
                scene.add(Primitive::Line {
                    x1: computed.margin_left - computed.tick_mark_major,
                    y1: y,
                    x2: computed.margin_left,
                    y2: y,
                    stroke: Color::from(&theme.tick_color),
                    stroke_width: computed.tick_stroke_width,
                    stroke_dasharray: None,
                });

                let label = if let Some(ref dt) = layout.y_datetime {
                    dt.format_tick(*ty)
                } else if layout.log_y && matches!(computed.y_tick_format, TickFormat::Auto) {
                    render_utils::format_log_tick(*ty)
                } else {
                    computed.y_tick_format.format(*ty)
                };
                scene.add(Primitive::Text {
                    x: computed.margin_left - computed.tick_label_margin,
                    y: y + computed.tick_size as f64 * 0.35,
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
            for tx in x_ticks.iter() {
                let x = map_x(*tx);

                scene.add(Primitive::Line {
                    x1: x,
                    y1: computed.height - computed.margin_bottom,
                    x2: x,
                    y2: computed.height - computed.margin_bottom + computed.tick_mark_major,
                    stroke: Color::from(&theme.tick_color),
                    stroke_width: computed.tick_stroke_width,
                    stroke_dasharray: None,
                });

                let label = if let Some(ref dt) = layout.x_datetime {
                    dt.format_tick(*tx)
                } else if layout.log_x && matches!(computed.x_tick_format, TickFormat::Auto) {
                    render_utils::format_log_tick(*tx)
                } else {
                    computed.x_tick_format.format(*tx)
                };
                let (anchor, rotate) = match layout.x_tick_rotate {
                    Some(angle) if angle < 0.0 => (TextAnchor::End,   Some(angle)),
                    Some(angle)                => (TextAnchor::Start,  Some(angle)),
                    None                       => (TextAnchor::Middle, None),
                };
                scene.add(Primitive::Text {
                    x,
                    y: computed.height - computed.margin_bottom + computed.tick_mark_major + computed.tick_size as f64,
                    content: label,
                    size: computed.tick_size,
                    anchor,
                    rotate,
                    bold: false,
                    color: None,
                });
            }
        }

        if !layout.suppress_y_ticks {
            for ty in y_ticks.iter() {
                let y = map_y(*ty);

                scene.add(Primitive::Line {
                    x1: computed.margin_left - computed.tick_mark_major,
                    y1: y,
                    x2: computed.margin_left,
                    y2: y,
                    stroke: Color::from(&theme.tick_color),
                    stroke_width: computed.tick_stroke_width,
                    stroke_dasharray: None,
                });

                let label = if let Some(ref dt) = layout.y_datetime {
                    dt.format_tick(*ty)
                } else if layout.log_y && matches!(computed.y_tick_format, TickFormat::Auto) {
                    render_utils::format_log_tick(*ty)
                } else {
                    computed.y_tick_format.format(*ty)
                };
                scene.add(Primitive::Text {
                    x: computed.margin_left - computed.tick_label_margin,
                    y: y + computed.tick_size as f64 * 0.35,
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
                    scene.add(Primitive::Line {
                        x1: x,
                        y1: computed.height - computed.margin_bottom,
                        x2: x,
                        y2: computed.height - computed.margin_bottom + computed.tick_mark_minor,
                        stroke: Color::from(&theme.tick_color),
                        stroke_width: computed.tick_stroke_width,
                        stroke_dasharray: None,
                    });
                }
            }
        }
        if !layout.suppress_y_ticks {
            if let Some(ref my) = y_minor {
                for ty in my {
                    let y = map_y(*ty);
                    scene.add(Primitive::Line {
                        x1: computed.margin_left - computed.tick_mark_minor,
                        y1: y,
                        x2: computed.margin_left,
                        y2: y,
                        stroke: Color::from(&theme.tick_color),
                        stroke_width: computed.tick_stroke_width,
                        stroke_dasharray: None,
                    });
                }
            }
        }
    }
}

pub fn add_y2_axis(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    let Some((y2_min, y2_max)) = computed.y2_range else { return; };
    let theme = &computed.theme;
    let axis_x = computed.width - computed.margin_right;

    // Right y-axis line
    scene.add(Primitive::Line {
        x1: axis_x, y1: computed.margin_top,
        x2: axis_x, y2: computed.height - computed.margin_bottom,
        stroke: Color::from(&theme.axis_color), stroke_width: computed.axis_line_width, stroke_dasharray: None,
    });

    if layout.suppress_y2_ticks { return; }

    let y2_ticks = if layout.log_y2 {
        render_utils::generate_ticks_log(y2_min, y2_max)
    } else {
        render_utils::generate_ticks(y2_min, y2_max, computed.y_ticks)
    };

    for ty in y2_ticks.iter() {
        let y = computed.map_y2(*ty);

        scene.add(Primitive::Line {
            x1: axis_x, y1: y, x2: axis_x + computed.tick_mark_major, y2: y,
            stroke: Color::from(&theme.tick_color), stroke_width: computed.tick_stroke_width, stroke_dasharray: None,
        });

        let label = if layout.log_y2 && matches!(computed.y2_tick_format, TickFormat::Auto) {
            render_utils::format_log_tick(*ty)
        } else {
            computed.y2_tick_format.format(*ty)
        };
        scene.add(Primitive::Text {
            x: axis_x + computed.tick_label_margin,
            y: y + computed.tick_size as f64 * 0.35,
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
        let (dx, dy) = layout.y2_label_offset;
        // Base x for the rightmost (first) line; additional lines shift left.
        let base_x = axis_x + computed.y2_axis_width - ls * 0.5 + dx;
        let base_y = computed.height / 2.0 + dy;
        for (i, line) in lines.iter().enumerate() {
            scene.add(Primitive::Text {
                x: base_x - i as f64 * ls,
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

pub fn add_labels_and_title(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    let ls = computed.label_size as f64;

    // X Axis Label
    if !layout.suppress_x_ticks {
        if let Some(label) = &layout.x_label {
            let lines = render_utils::wrap_or_single(label, computed.x_label_wrap);
            let (dx, dy) = layout.x_label_offset;
            let default_x = computed.margin_left + computed.plot_width() / 2.0;
            // Position the first line so the whole block ends at the expected bottom.
            let default_y = computed.height - ls * 0.5 - (lines.len() as f64 - 1.0) * ls;
            let (lx, ly) = computed.dice_x_label_pos.unwrap_or((default_x, default_y));
            for (i, line) in lines.iter().enumerate() {
                scene.add(Primitive::Text {
                    x: lx + dx, y: ly + dy + i as f64 * ls,
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

    // Y Axis Label (rotated -90°; wrapped lines stack horizontally in unrotated space)
    if !layout.suppress_y_ticks {
        if let Some(label) = &layout.y_label {
            let lines = render_utils::wrap_or_single(label, computed.y_label_wrap);
            let (dx, dy) = layout.y_label_offset;
            // Base x for the leftmost (first) line.
            let default_x = (computed.margin_left - 8.0 - computed.y_tick_label_px - 5.0
                - ls * 0.5 - (lines.len() as f64 - 1.0) * ls)
                .max(ls * 0.5 + 8.0);
            let default_y = computed.height / 2.0;
            let (lx, ly) = computed.dice_y_label_pos.unwrap_or((default_x, default_y));
            for (i, line) in lines.iter().enumerate() {
                scene.add(Primitive::Text {
                    x: lx + dx + i as f64 * ls, y: ly + dy,
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
        let total_height = lines.len() as f64 * ts;
        let cx = computed.margin_left + computed.plot_width() / 2.0;
        // Vertically centre the text block within margin_top/2.
        let start_y = computed.margin_top / 2.0 - total_height / 2.0 + ts * 0.8;
        for (i, line) in lines.iter().enumerate() {
            scene.add(Primitive::Text {
                x: cx,
                y: start_y + i as f64 * ts,
                content: line.clone(),
                size: computed.title_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
                color: None,
            });
        }
    }
}
