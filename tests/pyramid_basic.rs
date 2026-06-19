mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::pyramid::{PopulationPyramid, PyramidMode};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render(pp: PopulationPyramid, title: &str) -> String {
    let plots = vec![Plot::Pyramid(pp)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn render_size(pp: PopulationPyramid, title: &str, w: f64, h: f64) -> String {
    let plots = vec![Plot::Pyramid(pp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(title)
        .with_width(w)
        .with_height(h);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Build a typical 5-band single-series pyramid.
fn simple_pyramid() -> PopulationPyramid {
    PopulationPyramid::new()
        .with_left_label("Male")
        .with_right_label("Female")
        .with_group("0–4", 6.5, 6.2)
        .with_group("5–14", 6.8, 6.5)
        .with_group("15–29", 10.2, 9.8)
        .with_group("30–44", 9.5, 9.4)
        .with_group("45–64", 8.1, 8.6)
        .with_group("65+", 3.1, 4.2)
}

#[test]
fn test_pyramid_basic() {
    let svg = render(simple_pyramid(), "Basic Pyramid");
    common::write_test_output("test_outputs/pyramid_basic.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should have rect elements");
    assert!(svg.contains("Male"), "should contain left label");
    assert!(svg.contains("Female"), "should contain right label");
}

#[test]
fn test_pyramid_empty() {
    let pp = PopulationPyramid::new();
    let plots = vec![Plot::Pyramid(pp)];
    let layout = Layout::new((-1.0, 1.0), (0.5, 1.5));
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pyramid_empty.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "should produce valid SVG");
}

#[test]
fn test_pyramid_with_legend() {
    let pp = simple_pyramid().with_legend(true);
    let svg = render(pp, "Pyramid with Legend");
    common::write_test_output("test_outputs/pyramid_legend.svg", &svg).unwrap();
    assert!(svg.contains("Male"), "legend entry for left");
    assert!(svg.contains("Female"), "legend entry for right");
}

#[test]
fn test_pyramid_normalized() {
    let pp = simple_pyramid().with_normalize(true);
    let svg = render(pp, "Normalized Pyramid");
    common::write_test_output("test_outputs/pyramid_normalized.svg", &svg).unwrap();
    // Normalized: x-tick labels should have '%'
    assert!(svg.contains('%'), "should have percent tick labels");
}

#[test]
fn test_pyramid_show_values() {
    let pp = simple_pyramid().with_show_values(true);
    let svg = render_size(pp, "Pyramid with Values", 700.0, 500.0);
    common::write_test_output("test_outputs/pyramid_values.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should have bars");
}

#[test]
fn test_pyramid_custom_colors() {
    let pp = PopulationPyramid::new()
        .with_left_label("Male")
        .with_right_label("Female")
        .with_left_color("#3498db")
        .with_right_color("#e74c3c")
        .with_group("0–4", 6.5, 6.2)
        .with_group("5–14", 6.8, 6.5)
        .with_group("15+", 9.5, 9.4);
    let svg = render(pp, "Custom Colors");
    common::write_test_output("test_outputs/pyramid_custom_colors.svg", &svg).unwrap();
    assert!(
        svg.contains("#3498db") || svg.contains("3498db"),
        "should use left color"
    );
    assert!(
        svg.contains("#e74c3c") || svg.contains("e74c3c"),
        "should use right color"
    );
}

#[test]
fn test_pyramid_multi_series_grouped() {
    let pp = PopulationPyramid::new()
        .with_left_label("Male")
        .with_right_label("Female")
        .with_series(
            "1960",
            vec![
                ("0–14", 12.5f64, 12.0f64),
                ("15–44", 18.0f64, 17.5f64),
                ("45–64", 10.0f64, 10.8f64),
                ("65+", 3.5f64, 4.5f64),
            ],
        )
        .with_series(
            "2020",
            vec![
                ("0–14", 9.5f64, 9.0f64),
                ("15–44", 19.0f64, 19.2f64),
                ("45–64", 14.0f64, 15.0f64),
                ("65+", 7.0f64, 9.5f64),
            ],
        )
        .with_legend(true)
        .with_mode(PyramidMode::Grouped);
    let svg = render(pp, "Multi-Series Grouped");
    common::write_test_output("test_outputs/pyramid_multi_grouped.svg", &svg).unwrap();
    assert!(svg.contains("1960"), "should contain first series label");
    assert!(svg.contains("2020"), "should contain second series label");
}

#[test]
fn test_pyramid_multi_series_overlap() {
    let pp = PopulationPyramid::new()
        .with_left_label("Male")
        .with_right_label("Female")
        .with_series(
            "2000",
            vec![
                ("0–14", 10.0f64, 9.5f64),
                ("15–64", 28.0f64, 28.5f64),
                ("65+", 5.0f64, 7.0f64),
            ],
        )
        .with_series(
            "2020",
            vec![
                ("0–14", 9.0f64, 8.5f64),
                ("15–64", 27.0f64, 28.0f64),
                ("65+", 8.0f64, 11.0f64),
            ],
        )
        .with_mode(PyramidMode::Overlap)
        .with_legend(true);
    let svg = render(pp, "Multi-Series Overlap");
    common::write_test_output("test_outputs/pyramid_multi_overlap.svg", &svg).unwrap();
    assert!(svg.contains("<rect"), "should have rect elements");
}

#[test]
fn test_pyramid_x_ticks_are_unsigned() {
    // The x-axis should show unsigned values (no minus signs on tick labels)
    let pp = simple_pyramid();
    let svg = render(pp, "Tick Sign Check");
    common::write_test_output("test_outputs/pyramid_tick_sign.svg", &svg).unwrap();
    // Simple heuristic: no tick label should be "-"
    // The tick format custom fn strips the sign; "−" or "-" followed by digits should not appear
    // as a standalone tick label. We check the SVG doesn't have a lone "-" tick label.
    assert!(
        !svg.contains(">-<"),
        "negative tick labels should not appear"
    );
}

#[test]
fn test_pyramid_three_series_grouped() {
    let pp = PopulationPyramid::new()
        .with_left_label("Male")
        .with_right_label("Female")
        .with_series(
            "1960",
            vec![
                ("0–14", 12.0f64, 11.5f64),
                ("15–64", 20.0f64, 20.5f64),
                ("65+", 3.0f64, 4.0f64),
            ],
        )
        .with_series(
            "1990",
            vec![
                ("0–14", 10.0f64, 9.5f64),
                ("15–64", 24.0f64, 24.5f64),
                ("65+", 5.0f64, 6.5f64),
            ],
        )
        .with_series(
            "2020",
            vec![
                ("0–14", 9.0f64, 8.5f64),
                ("15–64", 26.0f64, 27.0f64),
                ("65+", 8.0f64, 11.0f64),
            ],
        )
        .with_legend(true)
        .with_group_gap(0.1)
        .with_bar_gap(0.03);
    let svg = render_size(pp, "Three Census Years", 700.0, 500.0);
    common::write_test_output("test_outputs/pyramid_three_series.svg", &svg).unwrap();
    assert!(svg.contains("1960"), "should contain 1960 label");
    assert!(svg.contains("1990"), "should contain 1990 label");
    assert!(svg.contains("2020"), "should contain 2020 label");
}

#[test]
fn test_pyramid_auto_from_plots_sets_y_categories() {
    let pp = simple_pyramid();
    let plots = vec![Plot::Pyramid(pp)];
    let layout = Layout::auto_from_plots(&plots);
    // y_categories should have 6 entries (one per age group)
    assert_eq!(
        layout.y_categories.as_ref().map(|v| v.len()),
        Some(6),
        "auto_from_plots should set 6 y_categories"
    );
    assert_eq!(
        layout
            .y_categories
            .as_ref()
            .and_then(|v| v.first())
            .map(|s| s.as_str()),
        Some("0–4"),
        "first y_category should be the youngest age group"
    );
}

#[test]
fn test_pyramid_bar_width() {
    // with_bar_width(w) controls bar thickness: bars fill `w` fraction of each row.
    // Two pyramids identical except for bar_width should produce rects whose pixel
    // heights are in the same ratio as their bar_widths.
    let make = |bw: f64| {
        let pp = PopulationPyramid::new()
            .with_left_label("Male")
            .with_right_label("Female")
            .with_bar_width(bw)
            .with_group("0–14", 10.0, 9.5)
            .with_group("15–64", 20.0, 20.5)
            .with_group("65+", 5.0, 7.0);
        render_size(pp, "Bar Width Test", 600.0, 400.0)
    };

    let svg_narrow = make(0.4);
    let svg_wide = make(0.8);

    common::write_test_output("test_outputs/pyramid_bar_width_narrow.svg", &svg_narrow).unwrap();
    common::write_test_output("test_outputs/pyramid_bar_width_wide.svg", &svg_wide).unwrap();

    // Extract the first rect height from each SVG (the left bar of group 0).
    // Format in SVG output is: height="NNN.NN"
    fn first_rect_height(svg: &str) -> f64 {
        // Bar rects have fractional pixel heights (e.g. "40.67"); skip integer
        // heights which belong to the canvas, clip rect, and axis lines.
        for cap in svg.split("height=\"") {
            let s = cap.split('"').next().unwrap_or("");
            if s.contains('.') {
                if let Ok(v) = s.parse::<f64>() {
                    if v > 1.0 {
                        return v;
                    }
                }
            }
        }
        panic!("no bar rect found in SVG");
    }

    let h_narrow = first_rect_height(&svg_narrow);
    let h_wide = first_rect_height(&svg_wide);

    assert!(
        h_wide > h_narrow,
        "wide bars ({h_wide:.1}px) should be taller than narrow bars ({h_narrow:.1}px)"
    );

    // The ratio of pixel heights should match the ratio of bar_widths (map_y is linear).
    let ratio = h_wide / h_narrow;
    let expected = 0.8 / 0.4; // = 2.0
    assert!(
        (ratio - expected).abs() < 0.05,
        "height ratio {ratio:.3} should be ~{expected:.1} (bar_width ratio)"
    );
}

#[test]
fn test_pyramid_bounds_symmetric() {
    let pp = simple_pyramid();
    let plot = Plot::Pyramid(pp);
    let bounds = plot.bounds().expect("bounds should be Some");
    let (x_min, x_max) = bounds.0;
    // x range should be symmetric around 0
    assert!(
        (x_min + x_max).abs() < 1e-9,
        "x range should be symmetric: got ({x_min}, {x_max})"
    );
    assert!(x_max > 0.0, "x_max should be positive");
}
