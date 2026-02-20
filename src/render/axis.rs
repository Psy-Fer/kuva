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
    });

    // Y axis
    scene.add(Primitive::Line {
        x1: computed.margin_left,
        y1: computed.margin_top,
        x2: computed.margin_left,
        y2: computed.height - computed.margin_bottom,
        stroke: "green".into(),
        stroke_width: 1.0,
    });

    // if y categories
    if let Some(categories) = &layout.y_categories {
        // draw x axis category labels and ticks
        for (i, label) in categories.iter().enumerate() {
            let y_val = i as f64 + 1.0; // match y-positioning
            let y_pos = computed.map_y(y_val);

            // Draw label
            scene.add(Primitive::Text {
                x: computed.margin_left - 10.0,
                y: y_pos + 20.0,
                content: label.clone(),
                size: 10,
                anchor: TextAnchor::End,
                rotate: None,
            });
    
            // Optional: draw a small tick
            scene.add(Primitive::Line {
                x1: computed.margin_left - 5.0,
                y1: y_pos,
                x2: computed.margin_left,
                y2: y_pos,
                stroke: "black".into(),
                stroke_width: 1.0,
            });
        }
        // x axis
        let x_ticks = if layout.log_x {
            render_utils::generate_ticks_log(computed.x_range.0, computed.x_range.1)
        } else {
            render_utils::generate_ticks(computed.x_range.0, computed.x_range.1, computed.ticks)
        };
        for (i, tx) in x_ticks.iter().enumerate() {

            let x = map_x(*tx);

            // X ticks
            scene.add(Primitive::Line {
                x1: x,
                y1: computed.height - computed.margin_bottom,
                x2: x,
                y2: computed.height - computed.margin_bottom + 5.0,
                stroke: "black".into(),
                stroke_width: 1.0,
            });

            // X tick labels
            let label = if layout.log_x {
                render_utils::format_log_tick(*tx)
            } else {
                format!("{:.1}", tx)
            };
            scene.add(Primitive::Text {
                x,
                y: computed.height - computed.margin_bottom + 15.0,
                content: label,
                size: 10,
                anchor: TextAnchor::Middle,
                rotate: None,
            });

            // Grid lines
            if layout.show_grid {
                if i != 0 {
                    // Vertical grid
                    scene.add(Primitive::Line {
                        x1: x,
                        y1: computed.margin_top,
                        x2: x,
                        y2: computed.height - computed.margin_bottom,
                        stroke: "#ccc".to_string(),
                        stroke_width: 1.0,
                    });
                }
            }
        }

    }
    // if x categories
    else if let Some(categories) = &layout.x_categories {
        // draw x axis category labels and ticks
        for (i, label) in categories.iter().enumerate() {
            let x_val = i as f64 + 1.0; // match x-positioning
            let x_pos = computed.map_x(x_val);
    
            // Draw label
            scene.add(Primitive::Text {
                x: x_pos,
                y: computed.height - computed.margin_bottom + 15.0,
                content: label.clone(),
                size: 10,
                anchor: TextAnchor::Middle,
                rotate: None,
            });
    
            // Optional: draw a small tick
            scene.add(Primitive::Line {
                x1: x_pos,
                y1: computed.height - computed.margin_bottom,
                x2: x_pos,
                y2: computed.height - computed.margin_bottom + 5.0,
                stroke: "black".into(),
                stroke_width: 1.0,
            });
        }

        let y_ticks = if layout.log_y {
            render_utils::generate_ticks_log(computed.y_range.0, computed.y_range.1)
        } else {
            render_utils::generate_ticks(computed.y_range.0, computed.y_range.1, computed.ticks)
        };

        for ty in y_ticks {
            let y = map_y(ty);
            // Y ticks
            scene.add(Primitive::Line {
                x1: computed.margin_left - 5.0,
                y1: y,
                x2: computed.margin_left,
                y2: y,
                stroke: "black".into(),
                stroke_width: 1.0,
            });

            // Y tick labels
            let label = if layout.log_y {
                render_utils::format_log_tick(ty)
            } else {
                format!("{:.1}", ty)
            };
            scene.add(Primitive::Text {
                x: computed.margin_left - 15.0,
                y,
                content: label,
                size: 10,
                anchor: TextAnchor::Middle,
                rotate: None,
            });
        }
    }
    // if regular axes
    else {
        // Draw value ticks and labels

        // x axis
        let x_ticks = if layout.log_x {
            render_utils::generate_ticks_log(computed.x_range.0, computed.x_range.1)
        } else {
            render_utils::generate_ticks(computed.x_range.0, computed.x_range.1, computed.ticks)
        };
        for (i, tx) in x_ticks.iter().enumerate() {

            let x = map_x(*tx);

            // X ticks
            scene.add(Primitive::Line {
                x1: x,
                y1: computed.height - computed.margin_bottom,
                x2: x,
                y2: computed.height - computed.margin_bottom + 5.0,
                stroke: "black".into(),
                stroke_width: 1.0,
            });

            // X tick labels
            let label = if layout.log_x {
                render_utils::format_log_tick(*tx)
            } else {
                format!("{:.1}", tx)
            };
            scene.add(Primitive::Text {
                x,
                y: computed.height - computed.margin_bottom + 15.0,
                content: label,
                size: 10,
                anchor: TextAnchor::Middle,
                rotate: None,
            });

            // Grid lines
            if layout.show_grid {
                if i != 0 {
                    // Vertical grid
                    scene.add(Primitive::Line {
                        x1: x,
                        y1: computed.margin_top,
                        x2: x,
                        y2: computed.height - computed.margin_bottom,
                        stroke: "#ccc".to_string(),
                        stroke_width: 1.0,
                    });
                }
            }
        }

        // y axis
        let y_ticks = if layout.log_y {
            render_utils::generate_ticks_log(computed.y_range.0, computed.y_range.1)
        } else {
            render_utils::generate_ticks(computed.y_range.0, computed.y_range.1, computed.ticks)
        };
        for (i, ty) in y_ticks.iter().enumerate() {

            let y = map_y(*ty);

            // Y ticks
            scene.add(Primitive::Line {
                x1: computed.margin_left - 5.0,
                y1: y,
                x2: computed.margin_left,
                y2: y,
                stroke: "black".into(),
                stroke_width: 1.0,
            });

            // Y tick labels
            let label = if layout.log_y {
                render_utils::format_log_tick(*ty)
            } else {
                format!("{:.1}", ty)
            };
            scene.add(Primitive::Text {
                x: computed.margin_left - 15.0,
                y,
                content: label,
                size: 10,
                anchor: TextAnchor::Middle,
                rotate: None,
            });

            if layout.show_grid {
                if i != 0 {
                    // Horizontal grid
                    scene.add(Primitive::Line {
                        x1: computed.margin_left,
                        y1: y,
                        x2: computed.width - computed.margin_right,
                        y2: y,
                        stroke: "#ccc".to_string(),
                        stroke_width: 1.0,
                    });
                }
            }
        }

    }
}

pub fn add_labels_and_title(scene: &mut Scene, computed: &ComputedLayout, layout: &Layout) {
    // X Axis Label
    if let Some(label) = &layout.x_label {
        scene.add(Primitive::Text {
            x: computed.width / 2.0,
            y: computed.height - computed.margin_bottom / 4.0,
            content: label.clone(),
            size: 14,
            anchor: TextAnchor::Middle,
            rotate: None,
        });
    }

    // Y Axis Label
    if let Some(label) = &layout.y_label {
        scene.add(Primitive::Text {
            x: 20.0,
            y: computed.height / 2.0,
            content: label.clone(),
            size: 14,
            anchor: TextAnchor::Middle,
            rotate: Some(-90.0),
        });
    }

    // Title
    if let Some(title) = &layout.title {
        scene.add(Primitive::Text {
            x: computed.width / 2.0,
            y: computed.margin_top / 2.0,
            content: title.clone(),
            size: 16,
            anchor: TextAnchor::Middle,
            rotate: None,
        });
    }
}