use kuva::plot::BarPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::{Layout, ComputedLayout};
use kuva::render::plots::Plot;

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
    std::fs::write("test_outputs/bar_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_vec_svg_output_builder() {
    let barvec = vec![("A", 3.2), ("B", 4.7), ("Longform_C", 2.8)];
    let bar = BarPlot::new()
                        .with_bars(barvec)
                        .with_color("purple");
    
    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
                        .with_title("Exciting Bar Plot")
                        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_vec_builder.svg", svg.clone()).unwrap();

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
    std::fs::write("test_outputs/bar_categories_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_bar_stacked() {
    let bar = BarPlot::new()
        .with_group("Q1", vec![(10.0, "tomato"), (15.0, "skyblue"), (8.0, "gold")])
        .with_group("Q2", vec![(12.0, "tomato"), (10.0, "skyblue"), (14.0, "gold")])
        .with_group("Q3", vec![(8.0, "tomato"), (18.0, "skyblue"), (6.0, "gold")])
        .with_legend(vec!["Product A", "Product B", "Product C"])
        .with_stacked();

    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Bar Plot")
        .with_y_label("Revenue");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_stacked.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#ff6347"));
    assert!(svg.contains("skyblue"));
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
    let long_first  = "VeryLongFirstLabelThatWouldClipWithoutFix_AAAA"; // 46 chars
    let short_first = "A";

    let long_computed  = computed_for_bar(vec![long_first,  "B", "C"], Some(-45.0));
    let short_computed = computed_for_bar(vec![short_first, "B", "C"], Some(-45.0));

    assert!(
        long_computed.margin_left > short_computed.margin_left,
        "margin_left should be larger for a longer first label \
         (got long={}, short={})",
        long_computed.margin_left, short_computed.margin_left,
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
    std::fs::write("test_outputs/bar_long_first_neg45.svg", &svg).unwrap();
}

// Positive rotation (+45°): TextAnchor::Start — last label extends right of its tick.
// With a very long last label the right margin must grow to contain it.
#[test]
fn test_rotated_pos45_long_last_label_expands_right_margin() {
    let long_last  = "VeryLongLastLabelThatWouldClipWithoutFix_ZZZZ"; // 45 chars
    let short_last = "Z";

    let long_computed  = computed_for_bar(vec!["A", "B", long_last],  Some(45.0));
    let short_computed = computed_for_bar(vec!["A", "B", short_last], Some(45.0));

    assert!(
        long_computed.margin_right > short_computed.margin_right,
        "margin_right should be larger for a longer last label \
         (got long={}, short={})",
        long_computed.margin_right, short_computed.margin_right,
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
    std::fs::write("test_outputs/bar_long_last_pos45.svg", &svg).unwrap();
}

// Short labels should not cause unnecessary margin inflation — both rotations.
#[test]
fn test_rotated_short_labels_do_not_inflate_margins() {
    let with_rot    = computed_for_bar(vec!["A", "B", "C"], Some(-45.0));
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