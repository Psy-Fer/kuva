use crate::render::render::{Scene, Primitive, TextAnchor};
use crate::render::layout::{Layout, ComputedLayout};
use crate::render::render_utils;



pub fn add_axes_and_grid(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {

    let map_x = |x| computed.map_x(x);
    let map_y = |y| computed.map_y(y);

    // Draw axes
    // X axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.height - computed.margin_bottom,
        x2: computed.width - computed.margin_right,
        y2: computed.height - computed.margin_bottom,
        stroke: "red".into(),
        stroke_width: 1.0,
        stroke_dasharray: None,
    });

    // Y axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.margin_top,
        x2: computed.margin_left,
        y2: computed.height - computed.margin_bottom,
        stroke: "green".into(),
        stroke_width: 1.0,
        stroke_dasharray: None,
    });

    // Always compute tick positions for grid lines
    let x_ticks = if layout.log_x {
        render_utils::generate_ticks_log(computed.x_range.0, computed.x_range.1)
    } else {
        render_utils::generate_ticks(computed.x_range.0, computed.x_range.1, computed.x_ticks)
    };
    let y_ticks = if layout.log_y {
        render_utils::generate_ticks_log(computed.y_range.0, computed.y_range.1)
    } else {
        render_utils::generate_ticks(computed.y_range.0, computed.y_range.1, computed.y_ticks)
    };

    // Draw grid lines (always, regardless of suppress flags)
    if layout.show_grid {
        // Vertical grid lines (skip for category x-axes like boxplot, bar, violin)
        if layout.x_categories.is_none() && layout.y_categories.is_none() {
            for (i, tx) in x_ticks.iter().enumerate() {
                // Skip first tick on linear axes (it sits on the axis line)
                if i == 0 && !layout.log_x { continue; }
                let x = map_x(*tx);
                scene.add(Primitive::Line {
                    x1: x,
                    y1: computed.margin_top,
                    x2: x,
                    y2: computed.height - computed.margin_bottom,
                    stroke: "#ccc".to_string(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });
            }
        }
        // Horizontal grid lines (draw when y-axis is numeric)
        if layout.y_categories.is_none() {
            for (i, ty) in y_ticks.iter().enumerate() {
                if i == 0 && !layout.log_y { continue; }
                let y = map_y(*ty);
                scene.add(Primitive::Line {
                    x1: computed.margin_left,
                    y1: y,
                    x2: computed.width - computed.margin_right,
                    y2: y,
                    stroke: "#ccc".to_string(),
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
                    y: y_pos + 20.0,
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
                    stroke: "black".into(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });
            }
        }
        if !layout.suppress_x_ticks {
            for tx in x_ticks.iter() {
                let x = map_x(*tx);

                scene.add(Primitive::Line {
                    x1: x,
                    y1: computed.height - computed.margin_bottom,
                    x2: x,
                    y2: computed.height - computed.margin_bottom + 5.0,
                    stroke: "black".into(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });

                let label = if layout.log_x {
                    render_utils::format_log_tick(*tx)
                } else {
                    format!("{:.1}", tx)
                };
                scene.add(Primitive::Text {
                    x,
                    y: computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64,
                    content: label,
                    size: computed.tick_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                });
            }
        }
    }
    else if let Some(categories) = &layout.x_categories {
        if !layout.suppress_x_ticks {
            for (i, label) in categories.iter().enumerate() {
                let x_val = i as f64 + 1.0;
                let x_pos = computed.map_x(x_val);

                scene.add(Primitive::Text {
                    x: x_pos,
                    y: computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64,
                    content: label.clone(),
                    size: computed.tick_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
                    bold: false,
                });

                scene.add(Primitive::Line {
                    x1: x_pos,
                    y1: computed.height - computed.margin_bottom,
                    x2: x_pos,
                    y2: computed.height - computed.margin_bottom + 5.0,
                    stroke: "black".into(),
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
                    stroke: "black".into(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });

                let label = if layout.log_y {
                    render_utils::format_log_tick(*ty)
                } else {
                    format!("{:.1}", ty)
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
                    stroke: "black".into(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });

                let label = if layout.log_x {
                    render_utils::format_log_tick(*tx)
                } else {
                    format!("{:.1}", tx)
                };
                scene.add(Primitive::Text {
                    x,
                    y: computed.height - computed.margin_bottom + 5.0 + computed.tick_size as f64,
                    content: label,
                    size: computed.tick_size,
                    anchor: TextAnchor::Middle,
                    rotate: None,
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
                    stroke: "black".into(),
                    stroke_width: 1.0,
                    stroke_dasharray: None,
                });

                let label = if layout.log_y {
                    render_utils::format_log_tick(*ty)
                } else {
                    format!("{:.1}", ty)
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