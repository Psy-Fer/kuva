use crate::render::render::{Scene, Primitive, TextAnchor};
use crate::render::layout::{Layout, ComputedLayout, TickFormat};
use crate::render::render_utils;



pub fn add_axes_and_grid(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {

    let map_x = |x| computed.map_x(x);
    let map_y = |y| computed.map_y(y);

    let theme = &computed.theme;

    // Draw axes
    // X axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.height - computed.margin_bottom,
        x2: computed.width - computed.margin_right,
        y2: computed.height - computed.margin_bottom,
        stroke: theme.axis_color.clone(),
        stroke_width: 1.0,
        stroke_dasharray: None,
    });

    // Y axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.margin_top,
        x2: computed.margin_left,
        y2: computed.height - computed.margin_bottom,
        stroke: theme.axis_color.clone(),
        stroke_width: 1.0,
        stroke_dasharray: None,
    });

    // Always compute tick positions for grid lines
    let x_ticks: Vec<f64> = if let Some(ref dt) = layout.x_datetime {
        dt.generate_ticks(computed.x_range.0, computed.x_range.1)
    } else if layout.log_x {
        render_utils::generate_ticks_log(computed.x_range.0, computed.x_range.1)
    } else if let Some(bw) = layout.x_bin_width {
        render_utils::generate_ticks_bin_aligned(
            computed.x_range.0, computed.x_range.1, bw, computed.x_ticks)
    } else {
        render_utils::generate_ticks(computed.x_range.0, computed.x_range.1, computed.x_ticks)
    };
    let y_ticks: Vec<f64> = if let Some(ref dt) = layout.y_datetime {
        dt.generate_ticks(computed.y_range.0, computed.y_range.1)
    } else if layout.log_y {
        render_utils::generate_ticks_log(computed.y_range.0, computed.y_range.1)
    } else {
        render_utils::generate_ticks(computed.y_range.0, computed.y_range.1, computed.y_ticks)
    };

    // Draw grid lines (always, regardless of suppress flags)
    if layout.show_grid {
        // Vertical grid lines (skip for category x-axes like boxplot, bar, violin)
        if layout.x_categories.is_none() && layout.y_categories.is_none() {
            for (i, tx) in x_ticks.iter().enumerate() {
                // Skip first tick on linear axes (it sits on the axis line).
                // Datetime ticks are calendar-snapped and don't land on the axis edge.
                if i == 0 && !layout.log_x && layout.x_datetime.is_none() { continue; }
                let x = map_x(*tx);
                scene.add(Primitive::Line {
                    x1: x,
                    y1: computed.margin_top,
                    x2: x,
                    y2: computed.height - computed.margin_bottom,
                    stroke: theme.grid_color.clone(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });
            }
        }
        // Horizontal grid lines (draw when y-axis is numeric)
        if layout.y_categories.is_none() {
            for (i, ty) in y_ticks.iter().enumerate() {
                if i == 0 && !layout.log_y && layout.y_datetime.is_none() { continue; }
                let y = map_y(*ty);
                scene.add(Primitive::Line {
                    x1: computed.margin_left,
                    y1: y,
                    x2: computed.width - computed.margin_right,
                    y2: y,
                    stroke: theme.grid_color.clone(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });
            }
        }
    }

    // Draw tick marks and labels
    if let Some(categories) = &layout.y_categories {
        if !layout.suppress_y_ticks {
            for (i, label) in categories.iter().enumerate() {
                let y_val = i as f64 + 1.0;
                let y_pos = computed.map_y(y_val);

                scene.add(Primitive::Text {
                    x: computed.margin_left - 10.0,
                    y: y_pos + computed.tick_size as f64 * 0.35,
                    content: label.clone(),
                    size: computed.tick_size,
                    anchor: TextAnchor::End,
                    rotate: None,
                    bold: false,
                });

                scene.add(Primitive::Line {
                    x1: computed.margin_left - 5.0,
                    y1: y_pos,
                    x2: computed.margin_left,
                    y2: y_pos,
                    stroke: theme.tick_color.clone(),
                    stroke_width: 1.0,
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

                    scene.add(Primitive::Text {
                        x: x_pos,
                        y: computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64,
                        content: label.clone(),
                        size: computed.tick_size,
                        anchor: TextAnchor::Middle,
                        rotate: layout.x_tick_rotate,
                        bold: false,
                    });

                    scene.add(Primitive::Line {
                        x1: x_pos,
                        y1: computed.height - computed.margin_bottom,
                        x2: x_pos,
                        y2: computed.height - computed.margin_bottom + 5.0,
                        stroke: theme.tick_color.clone(),
                        stroke_width: 1.0,
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
                        y2: computed.height - computed.margin_bottom + 5.0,
                        stroke: theme.tick_color.clone(),
                        stroke_width: 1.0,
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
                        y: computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64,
                        content: label,
                        size: computed.tick_size,
                        anchor,
                        rotate,
                        bold: false,
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
                    Some(angle) => (TextAnchor::End, Some(angle)),
                    None        => (TextAnchor::Middle, None),
                };
                scene.add(Primitive::Text {
                    x: x_pos,
                    y: computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64,
                    content: label.clone(),
                    size: computed.tick_size,
                    anchor,
                    rotate,
                    bold: false,
                });

                scene.add(Primitive::Line {
                    x1: x_pos,
                    y1: computed.height - computed.margin_bottom,
                    x2: x_pos,
                    y2: computed.height - computed.margin_bottom + 5.0,
                    stroke: theme.tick_color.clone(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });
            }
        }

        if !layout.suppress_y_ticks {
            for ty in y_ticks.iter() {
                let y = map_y(*ty);
                scene.add(Primitive::Line {
                    x1: computed.margin_left - 5.0,
                    y1: y,
                    x2: computed.margin_left,
                    y2: y,
                    stroke: theme.tick_color.clone(),
                    stroke_width: 1.0,
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
                    x: computed.margin_left - 8.0,
                    y: y + computed.tick_size as f64 * 0.35,
                    content: label,
                    size: computed.tick_size,
                    anchor: TextAnchor::End,
                    rotate: None,
                    bold: false,
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
                    y2: computed.height - computed.margin_bottom + 5.0,
                    stroke: theme.tick_color.clone(),
                    stroke_width: 1.0,
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
                    y: computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64,
                    content: label,
                    size: computed.tick_size,
                    anchor,
                    rotate,
                    bold: false,
                });
            }
        }

        if !layout.suppress_y_ticks {
            for ty in y_ticks.iter() {
                let y = map_y(*ty);

                scene.add(Primitive::Line {
                    x1: computed.margin_left - 5.0,
                    y1: y,
                    x2: computed.margin_left,
                    y2: y,
                    stroke: theme.tick_color.clone(),
                    stroke_width: 1.0,
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
                    x: computed.margin_left - 8.0,
                    y: y + computed.tick_size as f64 * 0.35,
                    content: label,
                    size: computed.tick_size,
                    anchor: TextAnchor::End,
                    rotate: None,
                    bold: false,
                });
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
        stroke: theme.axis_color.clone(), stroke_width: 1.0, stroke_dasharray: None,
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
            x1: axis_x, y1: y, x2: axis_x + 5.0, y2: y,
            stroke: theme.tick_color.clone(), stroke_width: 1.0, stroke_dasharray: None,
        });

        let label = if layout.log_y2 && matches!(computed.y2_tick_format, TickFormat::Auto) {
            render_utils::format_log_tick(*ty)
        } else {
            computed.y2_tick_format.format(*ty)
        };
        scene.add(Primitive::Text {
            x: axis_x + 8.0,
            y: y + computed.tick_size as f64 * 0.35,
            content: label,
            size: computed.tick_size,
            anchor: TextAnchor::Start,
            rotate: None,
            bold: false,
        });
    }

    if let Some(ref label) = layout.y2_label {
        scene.add(Primitive::Text {
            x: axis_x + computed.y2_axis_width - computed.label_size as f64 * 0.5,
            y: computed.height / 2.0,
            content: label.clone(),
            size: computed.label_size,
            anchor: TextAnchor::Middle,
            rotate: Some(90.0),
            bold: false,
        });
    }
}

pub fn add_labels_and_title(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    // X Axis Label
    if !layout.suppress_x_ticks {
        if let Some(label) = &layout.x_label {
            scene.add(Primitive::Text {
                x: computed.width / 2.0,
                y: computed.height - computed.label_size as f64 * 0.5,
                content: label.clone(),
                size: computed.label_size,
                anchor: TextAnchor::Middle,
                rotate: None,
                bold: false,
            });
        }
    }

    // Y Axis Label
    if !layout.suppress_y_ticks {
        if let Some(label) = &layout.y_label {
            scene.add(Primitive::Text {
                x: computed.label_size as f64,
                y: computed.height / 2.0,
                content: label.clone(),
                size: computed.label_size,
                anchor: TextAnchor::Middle,
                rotate: Some(-90.0),
                bold: false,
            });
        }
    }

    // Title
    if let Some(title) = &layout.title {
        scene.add(Primitive::Text {
            x: computed.width / 2.0,
            y: computed.margin_top / 2.0,
            content: title.clone(),
            size: computed.title_size,
            anchor: TextAnchor::Middle,
            rotate: None,
            bold: false,
        });
    }
}