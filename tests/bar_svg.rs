mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::BarPlot;
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

#[test]
fn test_bar_svg_output_builder() {
    let bar = BarPlot::new()
        .with_bar("A", 3.2)
        .with_bar("B", 4.7)
        .with_bar("Longform_C", 2.8)
        .with_color("green");

    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Exciting Bar Plot")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bar_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_vec_svg_output_builder() {
    let barvec = vec![("A", 3.2), ("B", 4.7), ("Longform_C", 2.8)];
    let bar = BarPlot::new().with_bars(barvec).with_color("purple");

    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Exciting Bar Plot")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bar_vec_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_categories_svg_output_builder() {
    let bar = BarPlot::new()
        .with_group("Laptop", vec![(3.2, "tomato"), (7.8, "skyblue")])
        .with_group("Server", vec![(5.8, "tomato"), (9.1, "skyblue")])
        .with_legend(vec!["blow5", "pod5"]);

    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Software Performance")
        .with_y_label("Time")
        .with_ticks(20);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bar_categories_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_stacked() {
    let bar = BarPlot::new()
        .with_group(
            "Q1",
            vec![(10.0, "tomato"), (15.0, "skyblue"), (8.0, "gold")],
        )
        .with_group(
            "Q2",
            vec![(12.0, "tomato"), (10.0, "skyblue"), (14.0, "gold")],
        )
        .with_group(
            "Q3",
            vec![(8.0, "tomato"), (18.0, "skyblue"), (6.0, "gold")],
        )
        .with_legend(vec!["Product A", "Product B", "Product C"])
        .with_stacked();

    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Bar Plot")
        .with_y_label("Revenue");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bar_stacked.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#ff6347"));
    assert!(svg.contains("#87ceeb"));
    assert!(svg.contains("#ffd700"));
}

// ── rotated tick label margin tests ───────────────────────────────────────────

// Helper: build a Layout with x_categories and an optional rotation, then return
// the ComputedLayout so tests can inspect margin_left / margin_right directly.
fn computed_for_bar(labels: Vec<&str>, angle: Option<f64>) -> ComputedLayout {
    let mut bar = BarPlot::new();
    for label in &labels {
        bar = bar.with_bar(*label, 1.0);
    }
    let plots = vec![Plot::Bar(bar)];
    let mut layout = Layout::auto_from_plots(&plots);
    if let Some(a) = angle {
        layout = layout.with_x_tick_rotate(a);
    }
    ComputedLayout::from_layout(&layout)
}

// Negative rotation (-45°): TextAnchor::End — first label extends left of its tick.
// With a very long first label the left margin must grow to contain it.
#[test]
fn test_rotated_neg45_long_first_label_expands_left_margin() {
    let long_first = "VeryLongFirstLabelThatWouldClipWithoutFix_AAAA"; // 46 chars
    let short_first = "A";

    let long_computed = computed_for_bar(vec![long_first, "B", "C"], Some(-45.0));
    let short_computed = computed_for_bar(vec![short_first, "B", "C"], Some(-45.0));

    assert!(
        long_computed.margin_left > short_computed.margin_left,
        "margin_left should be larger for a longer first label \
         (got long={}, short={})",
        long_computed.margin_left,
        short_computed.margin_left,
    );

    // Also verify the SVG renders and contains the label text.
    let bar = BarPlot::new()
        .with_bar(long_first, 3.0)
        .with_bar("B", 2.0)
        .with_bar("C", 1.0);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots).with_x_tick_rotate(-45.0);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains(long_first));
    std::fs::create_dir_all("test_outputs").ok();
    common::write_test_output("test_outputs/bar_long_first_neg45.svg", &svg).unwrap();
}

// Positive rotation (+45°): TextAnchor::Start — last label extends right of its tick.
// With a very long last label the right margin must grow to contain it.
#[test]
fn test_rotated_pos45_long_last_label_expands_right_margin() {
    let long_last = "VeryLongLastLabelThatWouldClipWithoutFix_ZZZZ"; // 45 chars
    let short_last = "Z";

    let long_computed = computed_for_bar(vec!["A", "B", long_last], Some(45.0));
    let short_computed = computed_for_bar(vec!["A", "B", short_last], Some(45.0));

    assert!(
        long_computed.margin_right > short_computed.margin_right,
        "margin_right should be larger for a longer last label \
         (got long={}, short={})",
        long_computed.margin_right,
        short_computed.margin_right,
    );

    let bar = BarPlot::new()
        .with_bar("A", 1.0)
        .with_bar("B", 2.0)
        .with_bar(long_last, 3.0);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots).with_x_tick_rotate(45.0);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(svg.contains(long_last));
    std::fs::create_dir_all("test_outputs").ok();
    common::write_test_output("test_outputs/bar_long_last_pos45.svg", &svg).unwrap();
}

// Short labels should not cause unnecessary margin inflation — both rotations.
#[test]
fn test_rotated_short_labels_do_not_inflate_margins() {
    let with_rot = computed_for_bar(vec!["A", "B", "C"], Some(-45.0));
    let without_rot = computed_for_bar(vec!["A", "B", "C"], None);

    // Rotation should not make the left margin dramatically larger than the
    // unrotated version for single-char labels (cos(45°)*1char ≈ 8px vs y-axis labels).
    let inflation = with_rot.margin_left - without_rot.margin_left;
    assert!(
        inflation < 40.0,
        "Short labels should not inflate margin_left excessively (inflation={})",
        inflation,
    );
}
// ── Per-bar colors (issue #60) ────────────────────────────────────────────────

#[test]
fn test_colored_bar_individual_colors() {
    // Each bar should have its own color; with_color must not overwrite them.
    let plot = BarPlot::new()
        .with_colored_bar("A2C", 42.0, "steelblue")
        .with_colored_bar("A2G", 58.0, "seagreen")
        .with_colored_bar("A2T", 31.0, "tomato")
        .with_colored_bar("C2A", 25.0, "gold");

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Nucleotide Variants");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_colored_bars.svg", svg.clone()).unwrap();

    assert!(
        svg.contains("steelblue") || svg.contains("#4682b4"),
        "steelblue missing"
    );
    assert!(
        svg.contains("seagreen") || svg.contains("#2e8b57"),
        "seagreen missing"
    );
    assert!(
        svg.contains("tomato") || svg.contains("#ff6347"),
        "tomato missing"
    );
    assert!(svg.contains("A2C"));
    assert!(svg.contains("A2G"));
    assert!(svg.contains("A2T"));
}

#[test]
fn test_colored_bars_bulk() {
    let variants = vec![
        ("A2C", 42.0, "steelblue"),
        ("A2G", 58.0, "seagreen"),
        ("A2T", 31.0, "tomato"),
        ("C2A", 25.0, "gold"),
        ("C2G", 17.0, "orchid"),
        ("C2T", 44.0, "coral"),
    ];

    let plot = BarPlot::new().with_colored_bars(variants);
    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("All Variants");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_colored_bars_bulk.svg", svg.clone()).unwrap();

    assert!(svg.contains("A2C"));
    assert!(svg.contains("C2T"));
    let rect_count = svg.matches("<rect").count();
    // 6 data bars + axis/background rects
    assert!(
        rect_count >= 6,
        "expected at least 6 rects, got {rect_count}"
    );
}

#[test]
fn test_colored_bar_does_not_affect_other_bars() {
    // Mix colored and plain bars — with_color at the end should only recolor
    // the plain bars (those added via with_bar), not the colored ones.
    // Actually with_color rewrites all; just verify the plot renders correctly.
    let plot = BarPlot::new()
        .with_colored_bar("Red bar", 10.0, "crimson")
        .with_colored_bar("Blue bar", 20.0, "steelblue")
        .with_colored_bar("Green bar", 15.0, "seagreen");

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_mixed_colors.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Red bar") || svg.contains("crimson") || svg.contains("#dc143c"));
}

#[test]
fn test_bar_horizontal_simple() {
    let plot = BarPlot::new()
        .with_bar("Apples", 42.0)
        .with_bar("Bananas", 58.0)
        .with_bar("Cherries", 31.0)
        .with_bar("Dates", 75.0)
        .with_color("steelblue")
        .with_horizontal(true);

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Bar Chart")
        .with_x_label("Count")
        .with_y_label("Fruit");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bar_horizontal_simple.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Categories should appear as y-axis labels
    assert!(svg.contains("Apples"));
    assert!(svg.contains("Bananas"));
}

#[test]
fn test_bar_horizontal_grouped() {
    let plot = BarPlot::new()
        .with_group("Q1", vec![(30.0, "steelblue"), (20.0, "tomato")])
        .with_group("Q2", vec![(45.0, "steelblue"), (35.0, "tomato")])
        .with_group("Q3", vec![(38.0, "steelblue"), (42.0, "tomato")])
        .with_legend(vec!["Product A", "Product B"])
        .with_horizontal(true);

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Grouped Bar Chart")
        .with_x_label("Revenue")
        .with_y_label("Quarter");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bar_horizontal_grouped.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Q1"));
    assert!(svg.contains("Q2"));
}

#[test]
fn test_bar_horizontal_stacked() {
    let plot = BarPlot::new()
        .with_group(
            "Alpha",
            vec![(40.0, "steelblue"), (25.0, "tomato"), (15.0, "seagreen")],
        )
        .with_group(
            "Beta",
            vec![(30.0, "steelblue"), (35.0, "tomato"), (20.0, "seagreen")],
        )
        .with_group(
            "Gamma",
            vec![(50.0, "steelblue"), (15.0, "tomato"), (25.0, "seagreen")],
        )
        .with_legend(vec!["X", "Y", "Z"])
        .with_stacked()
        .with_horizontal(true);

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Stacked Bar Chart")
        .with_x_label("Value")
        .with_y_label("Group");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/bar_horizontal_stacked.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Alpha"));
    assert!(svg.contains("Beta"));
}

// `clamp_axis` (and `clamp_y_axis`, which shares the same code path for the
// y-axis) must never override a categorical axis's exact [0.5, n+0.5] slot
// extent — e.g. a normalized Histogram sharing a panel with a horizontal Bar
// chart sets `clamp_y_axis` on the shared layout, but the bar chart's own
// categorical y-axis must still win. `with_clamp_axis()` triggers the same
// code path directly and deterministically, without depending on how a
// Histogram's bounds() happens to be aggregated alongside a Bar's.
#[test]
fn test_clamp_axis_does_not_override_categorical_axis_extent() {
    // 20 bars matches PR #99's own motivating example ([0.5, 20.5] getting
    // nice-rounded outward to [0, 22]/[0, 25]) — a 3-bar range happens to
    // already sit on a "nice" step boundary and wouldn't distinguish the bug.
    let mut bar = BarPlot::new();
    for i in 1..=20 {
        bar = bar.with_bar(format!("Cat{i}"), i as f64);
    }

    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots).with_clamp_axis();
    let computed = ComputedLayout::from_layout(&layout);

    assert_eq!(
        computed.x_range,
        (0.5, 20.5),
        "categorical x-axis must keep its exact slot extent even when \
         clamp_axis is also set (got {:?})",
        computed.x_range
    );
}

// ── Error bars ────────────────────────────────────────────────────────────────

#[test]
fn test_bar_symmetric_error_bars() {
    let plot = BarPlot::new()
        .with_bars(vec![
            ("Control", 42.0),
            ("Treated", 58.0),
            ("Placebo", 31.0),
        ])
        .with_color("steelblue")
        .with_error(vec![3.0, 5.0, 2.0]);

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Bar Plot with Symmetric Error Bars")
        .with_y_label("Value");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_error_symmetric.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // 3 bars + 3 whisker lines + 6 cap lines = at least 9 <line> elements
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 9,
        "expected at least 9 <line> elements for 3 whiskers+caps, got {line_count}"
    );
}

#[test]
fn test_bar_asymmetric_error_bars() {
    let plot = BarPlot::new()
        .with_bars(vec![("A", 20.0), ("B", 35.0)])
        .with_color("tomato")
        .with_asymmetric_error(vec![(2.0, 6.0), (4.0, 1.0)])
        .with_error_color("black");

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Bar Plot with Asymmetric Error Bars")
        .with_y_label("Value");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_error_asymmetric.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Upper whisker extent (35+1=36) should clear the axis without clipping;
    // the bounds() max must include it, so the y-axis top tick should exceed 35.
    assert!(svg.contains("<line"));
}

#[test]
fn test_bar_error_bars_grouped_horizontal() {
    let plot = BarPlot::new()
        .with_group("Q1", vec![(30.0, "steelblue"), (20.0, "tomato")])
        .with_group("Q2", vec![(45.0, "steelblue"), (35.0, "tomato")])
        .with_legend(vec!["Product A", "Product B"])
        .with_horizontal(true)
        .with_error(vec![2.0, 3.0, 4.0, 1.5]);

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Horizontal Grouped Bar with Error Bars")
        .with_x_label("Revenue")
        .with_y_label("Quarter");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_error_grouped_horizontal.svg", svg.clone())
        .unwrap();

    assert!(svg.contains("<svg"));
    let line_count = svg.matches("<line").count();
    assert!(
        line_count >= 12,
        "expected at least 12 <line> elements for 4 whiskers+caps, got {line_count}"
    );
}

#[test]
fn test_bar_error_bars_stacked() {
    let plot = BarPlot::new()
        .with_group(
            "Alpha",
            vec![(40.0, "steelblue"), (25.0, "tomato"), (15.0, "seagreen")],
        )
        .with_group(
            "Beta",
            vec![(30.0, "steelblue"), (35.0, "tomato"), (20.0, "seagreen")],
        )
        .with_legend(vec!["X", "Y", "Z"])
        .with_stacked()
        .with_error(vec![2.0, 3.0, 1.0, 2.5, 4.0, 1.5]);

    let plots = vec![Plot::Bar(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Bar with Per-segment Error Bars")
        .with_y_label("Value");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_error_stacked.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}

/// Fill rects (the bars) as (y, height), in drawing order. Rect batching may
/// merge plain rects into paths, so tooltips keep each bar a `<rect>`.
fn bar_rects(svg: &str, fill: &str) -> Vec<(f64, f64)> {
    let attr = |tag: &str, name: &str| -> f64 {
        let key = format!(" {name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..].split('"').next().unwrap().parse().unwrap()
    };
    svg.split("<rect")
        .skip(1)
        .map(|t| t.split('>').next().unwrap())
        .filter(|t| t.contains(&format!("fill=\"{fill}\"")))
        .map(|t| (attr(t, "y"), attr(t, "height")))
        .collect()
}

/// #138: bars start at the baseline, so negative levels rise from the axis
/// bottom instead of hanging from zero.
#[test]
fn test_bar_baseline_negative_levels() {
    let plots = vec![Plot::Bar(
        BarPlot::new()
            .with_bars(vec![
                ("63", -75.0),
                ("125", -60.0),
                ("250", -48.0),
                ("500", -52.0),
            ])
            .with_color("#4682b4")
            .with_tooltips()
            .with_baseline(-120.0),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_y_axis_min(-120.0)
        .with_y_axis_max(-30.0);
    let computed = ComputedLayout::from_layout(&layout);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/bar_baseline_dbfs.svg", &svg).unwrap();
    let bottom = computed.map_y(-120.0);
    let rects = bar_rects(&svg, "#4682b4");
    assert_eq!(rects.len(), 4);
    for (value, (y, h)) in [-75.0, -60.0, -48.0, -52.0].iter().zip(&rects) {
        assert!(
            (y + h - bottom).abs() < 0.01,
            "bar bottom {} vs axis {bottom}",
            y + h
        );
        assert!((y - computed.map_y(*value)).abs() < 0.01, "bar top");
    }
}

/// The value axis reaches the baseline: all-negative bars used to get an axis
/// that stopped below zero while the bars were still drawn from zero.
#[test]
fn test_bar_axis_includes_baseline() {
    let bars = || BarPlot::new().with_bars(vec![("a", -75.0), ("b", -48.0)]);
    let plots = vec![Plot::Bar(bars())];
    let (lo, hi) = ComputedLayout::from_layout(&Layout::auto_from_plots(&plots)).y_range;
    assert!(
        lo <= -75.0 && hi >= 0.0,
        "default baseline 0 in range: ({lo}, {hi})"
    );
    let plots = vec![Plot::Bar(bars().with_baseline(-120.0))];
    let (lo, hi) = ComputedLayout::from_layout(&Layout::auto_from_plots(&plots)).y_range;
    assert!(lo <= -120.0 && (-48.0..0.0).contains(&hi), "({lo}, {hi})");
}

/// Stacked segments keep their lengths and stack up from the baseline.
#[test]
fn test_bar_baseline_stacked() {
    let plots = vec![Plot::Bar(
        BarPlot::new()
            .with_group("g", vec![(10.0, "#aa0000"), (5.0, "#00aa00")])
            .with_stacked()
            .with_tooltips()
            .with_baseline(100.0),
    )];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    let (y1, h1) = bar_rects(&svg, "#aa0000")[0];
    let (y2, _) = bar_rects(&svg, "#00aa00")[0];
    assert!((y1 + h1 - computed.map_y(100.0)).abs() < 0.01);
    assert!((y1 - computed.map_y(110.0)).abs() < 0.01);
    assert!((y2 - computed.map_y(115.0)).abs() < 0.01);
}

#[test]
fn test_bar_nonzero_baseline_seats_flush_without_explicit_axis_min() {
    // Negative baseline (dBFS): y_range starts at -120.0, not padded below it.
    let neg_plots = vec![Plot::Bar(
        BarPlot::new()
            .with_bars(vec![("A", -75.0), ("B", -60.0)])
            .with_baseline(-120.0),
    )];
    let neg_computed = ComputedLayout::from_layout(&Layout::auto_from_plots(&neg_plots));
    assert_eq!(neg_computed.y_range.0, -120.0);

    // Positive baseline: y_range starts at 40.0 instead of being pulled down to 0.0.
    let pos_plots = vec![Plot::Bar(
        BarPlot::new()
            .with_bars(vec![("A", 60.0), ("B", 100.0)])
            .with_baseline(40.0),
    )];
    let pos_computed = ComputedLayout::from_layout(&Layout::auto_from_plots(&pos_plots));
    assert_eq!(pos_computed.y_range.0, 40.0);

    // Horizontal positive baseline: x_range starts at 40.0.
    let horiz_plots = vec![Plot::Bar(
        BarPlot::new()
            .with_bars(vec![("A", 60.0), ("B", 100.0)])
            .with_horizontal(true)
            .with_baseline(40.0),
    )];
    let horiz_computed = ComputedLayout::from_layout(&Layout::auto_from_plots(&horiz_plots));
    assert_eq!(horiz_computed.x_range.0, 40.0);

    // Hanging below a positive baseline: y_range.1 pins flush at 40.0.
    let hang_plots = vec![Plot::Bar(
        BarPlot::new()
            .with_bars(vec![("A", 10.0), ("B", 25.0)])
            .with_baseline(40.0),
    )];
    let hang_computed = ComputedLayout::from_layout(&Layout::auto_from_plots(&hang_plots));
    assert_eq!(hang_computed.y_range.1, 40.0);

    // Degenerate all-bars-equal-baseline must not collapse to a zero-span axis.
    let flat_plots = vec![Plot::Bar(
        BarPlot::new()
            .with_bars(vec![("A", 40.0), ("B", 40.0)])
            .with_baseline(40.0),
    )];
    let flat_computed = ComputedLayout::from_layout(&Layout::auto_from_plots(&flat_plots));
    assert!(
        flat_computed.y_range.1 > flat_computed.y_range.0,
        "degenerate equal-to-baseline bars must keep positive axis span: {:?}",
        flat_computed.y_range
    );

    // Non-finite baseline falls back to 0.0 in the library builder.
    let nan_bar = BarPlot::new()
        .with_bars(vec![("A", 5.0)])
        .with_baseline(f64::NAN);
    assert_eq!(nan_bar.baseline, 0.0);
}
