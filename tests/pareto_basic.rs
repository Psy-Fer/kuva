mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::pareto::{ParetoBar, ParetoPlot};
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn error_categories() -> ParetoPlot {
    ParetoPlot::new().with_categories(vec![
        ("Missing field", 42.0),
        ("Typo", 31.0),
        ("Timeout", 18.0),
        ("Bad format", 12.0),
        ("Other", 6.0),
    ])
}

#[test]
fn test_pareto_basic() {
    let plot = error_categories();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Error Categories");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<rect"), "should contain bars");
    assert!(
        svg.contains("polyline") || svg.contains("<path"),
        "should contain the cumulative line"
    );
    assert!(svg.contains("Missing field"), "x-axis category labels");
    assert!(svg.contains("Cumulative"), "default y2 label");
}

#[test]
fn test_pareto_empty() {
    let plot = ParetoPlot::new();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_empty.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_pareto_sorts_descending_by_default() {
    let plot = ParetoPlot::new().with_categories(vec![("B", 10.0), ("A", 50.0), ("C", 5.0)]);
    let ordered: Vec<&str> = plot
        .ordered_categories()
        .iter()
        .map(|c| c.label.as_str())
        .collect();
    assert_eq!(ordered, vec!["A", "B", "C"]);
}

#[test]
fn test_pareto_with_sorted_false_preserves_insertion_order() {
    let plot = ParetoPlot::new()
        .with_categories(vec![("B", 10.0), ("A", 50.0), ("C", 5.0)])
        .with_sorted(false);
    let ordered: Vec<&str> = plot
        .ordered_categories()
        .iter()
        .map(|c| c.label.as_str())
        .collect();
    assert_eq!(ordered, vec!["B", "A", "C"]);
}

#[test]
fn test_pareto_cumulative_reaches_100() {
    let plot = error_categories();
    let cum = plot.cumulative_percentages();
    assert_eq!(cum.len(), 5);
    assert!(
        (cum.last().unwrap() - 100.0).abs() < 1e-9,
        "cumulative percentage should reach exactly 100%, got {}",
        cum.last().unwrap()
    );
    // Monotonically non-decreasing
    for i in 1..cum.len() {
        assert!(cum[i] >= cum[i - 1]);
    }
}

#[test]
fn test_pareto_cumulative_percentages_empty_when_total_zero() {
    let plot = ParetoPlot::new().with_categories(vec![("A", 0.0), ("B", 0.0)]);
    assert!(plot.cumulative_percentages().is_empty());
}

#[test]
fn test_pareto_y2_axis_starts_at_zero_with_headroom_above_100() {
    let plot = error_categories();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    let (y2_min, y2_max) = computed.y2_range.expect("pareto should set a y2 range");
    assert_eq!(y2_min, 0.0);
    // Cumulative always reaches exactly 100% at the last category, so the axis max
    // must clear 100 with room to spare -- otherwise the guaranteed-100% point (and
    // its label, if shown) sits flush against the plot's top edge and gets clipped.
    // Bounded above too, as a sanity check against over-provisioning the other way.
    assert!(
        y2_max > 100.0 && y2_max <= 120.0,
        "y2 max ({y2_max}) should clear 100 with headroom, but not by an excessive amount"
    );
}

/// The final cumulative point (always exactly 100%) must render with clearance
/// below the plot's top edge, not flush against it -- regression test for the
/// clipping bug fixed by giving the y2 axis headroom above 100.
#[test]
fn test_pareto_final_point_not_flush_with_plot_top() {
    let plot = error_categories();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    let final_y = computed.map_y2(100.0);
    let clearance = final_y - computed.margin_top;
    assert!(
        clearance > 8.0,
        "final cumulative point should have real clearance below the plot's top \
         edge, got {clearance:.1}px (margin_top={}, point_y={})",
        computed.margin_top,
        final_y
    );
}

/// The threshold reference line must be labeled with its percentage -- otherwise
/// it reads as ambiguous against the primary axis's own gridlines/values, which
/// commonly land at unrelated numbers in the same pixel positions (e.g. primary
/// axis maxing at 50 puts its "40" gridline exactly where an 80%-cumulative line
/// falls on the fixed 0..100 secondary axis).
#[test]
fn test_pareto_threshold_line_is_labeled() {
    let plot = error_categories();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_threshold_labeled.svg", svg.clone()).unwrap();
    assert!(
        svg.contains(">80%<"),
        "default 80% threshold line should be labeled with its percentage"
    );
}

#[test]
fn test_pareto_legend() {
    let plot = error_categories().with_legend("Count", "Cumulative %");
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_legend.svg", svg.clone()).unwrap();
    assert!(svg.contains("Count"));
    assert!(svg.contains("Cumulative %"));
}

#[test]
fn test_pareto_legend_shown_by_default() {
    // Bars and the cumulative line are two encodings that always coexist, so
    // (unlike most other plot types) Pareto shows a legend by default with
    // generic labels, rather than requiring an explicit .with_legend() call.
    let plot = error_categories();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    assert!(
        layout.show_legend,
        "legend should show by default even without an explicit .with_legend() call"
    );
    let svg = SvgBackend.render_scene(&render_multiple(
        vec![Plot::Pareto(error_categories())],
        Layout::auto_from_plots(&[Plot::Pareto(error_categories())]),
    ));
    common::write_test_output("test_outputs/pareto_default_legend.svg", svg.clone()).unwrap();
    assert!(svg.contains(">Value<"), "default bar legend label");
    assert!(svg.contains(">Cumulative %<"), "default line legend label");
}

#[test]
fn test_pareto_legend_can_be_hidden() {
    let plot = error_categories().with_show_legend(false);
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    assert!(
        !layout.show_legend,
        ".with_show_legend(false) should suppress the default legend"
    );
}

#[test]
fn test_pareto_threshold_line_shown_by_default() {
    let plot = error_categories();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_threshold.svg", svg.clone()).unwrap();
    assert!(
        svg.contains("4,3") || svg.contains("stroke-dasharray"),
        "default threshold reference line should be dashed"
    );
}

#[test]
fn test_pareto_threshold_line_can_be_hidden() {
    let plot = error_categories().with_show_threshold(false);
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg_with = SvgBackend.render_scene(&render_multiple(
        vec![Plot::Pareto(error_categories())],
        Layout::auto_from_plots(&[Plot::Pareto(error_categories())]),
    ));
    let svg_without = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_no_threshold.svg", svg_without.clone()).unwrap();
    let dash_with = svg_with.matches("4,3").count();
    let dash_without = svg_without.matches("4,3").count();
    assert!(
        dash_without < dash_with,
        "hiding the threshold line should remove its dashed stroke"
    );
}

#[test]
fn test_pareto_custom_threshold() {
    let plot = error_categories().with_threshold(90.0);
    assert_eq!(plot.threshold, 90.0);
    assert!(plot.show_threshold, "with_threshold implies show_threshold");
}

#[test]
fn test_pareto_cumulative_labels() {
    let plot = error_categories().with_cumulative_labels(true);
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_cumulative_labels.svg", svg.clone()).unwrap();
    assert!(svg.contains("%<"), "should render percentage labels");
}

#[test]
fn test_pareto_custom_colors() {
    // Named CSS colors are resolved to hex at construction (`Color::from`), so the
    // literal name never appears in the SVG — check the resolved hex value instead.
    let plot = error_categories()
        .with_color("seagreen")
        .with_line_color("darkorange");
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_custom_colors.svg", svg.clone()).unwrap();
    let seagreen_hex = kuva::render::color::Color::from("seagreen").to_svg_string();
    let darkorange_hex = kuva::render::color::Color::from("darkorange").to_svg_string();
    assert!(svg.contains(&seagreen_hex), "bar fill should use seagreen");
    assert!(
        svg.contains(&darkorange_hex),
        "cumulative line stroke should use darkorange"
    );
}

#[test]
fn test_pareto_bw_mode() {
    // Named CSS colors are always resolved to hex (see test_pareto_custom_colors),
    // so check the *resolved* default hex values, not the literal color names —
    // a literal-name check would pass vacuously even with BW mode reverted.
    let bar_hex = kuva::render::color::Color::from("steelblue").to_svg_string();
    let line_hex = kuva::render::color::Color::from("firebrick").to_svg_string();

    let plots = vec![Plot::Pareto(error_categories())];
    let layout = Layout::auto_from_plots(&plots).with_bw_mode();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_bw.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(
        !svg.contains(&bar_hex) && !svg.contains(&line_hex),
        "BW mode should not leak the configured bar/line colors into the SVG"
    );

    let plots_color = vec![Plot::Pareto(error_categories())];
    let layout_color = Layout::auto_from_plots(&plots_color);
    let svg_color = SvgBackend.render_scene(&render_multiple(plots_color, layout_color));
    assert!(
        svg_color.contains(&bar_hex) && svg_color.contains(&line_hex),
        "sanity check: color mode should use the real colors, confirming the BW \
         assertion above isn't vacuously true"
    );
}

#[test]
fn test_pareto_bounds_categorical_extent() {
    let plot = error_categories();
    let plots = [Plot::Pareto(plot)];
    let ((x_min, x_max), (y_min, y_max)) = plots[0].bounds().unwrap();
    assert_eq!((x_min, x_max), (0.5, 5.5));
    assert_eq!(y_min, 0.0);
    assert_eq!(y_max, 42.0);
}

#[test]
fn test_pareto_defaults_to_rotated_thinned_x_labels() {
    use kuva::AxisLabelOverlap;

    let plots = vec![Plot::Pareto(error_categories())];
    let layout = Layout::auto_from_plots(&plots);
    assert_eq!(
        layout.x_tick_rotate,
        Some(-45.0),
        "Pareto x-labels are always categorical and often numerous -- should \
         default to rotated, not upright, without the caller having to know to set it"
    );
    assert!(
        matches!(layout.x_label_overlap, AxisLabelOverlap::Thin),
        "should default to collision-thinning as a safety net beyond rotation alone"
    );
}

#[test]
fn test_pareto_x_tick_rotate_can_be_overridden() {
    let plots = vec![Plot::Pareto(error_categories())];
    let layout = Layout::auto_from_plots(&plots).with_x_tick_rotate(0.0);
    assert_eq!(
        layout.x_tick_rotate,
        Some(0.0),
        "explicit .with_x_tick_rotate() after auto_from_plots should override the default"
    );
}

// ── max_categories bucketing ────────────────────────────────────────────────

fn long_tail_categories() -> ParetoPlot {
    ParetoPlot::new().with_categories(vec![
        ("Missing field", 42.0),
        ("Typo", 31.0),
        ("Timeout", 18.0),
        ("Bad format", 12.0),
        ("Duplicate entry", 9.0),
        ("Wrong encoding", 7.0),
        ("Network blip", 5.0),
        ("Other misc", 3.0),
    ])
}

#[test]
fn test_pareto_no_bucketing_by_default() {
    let plot = long_tail_categories();
    let bars = plot.render_bars();
    assert_eq!(
        bars.len(),
        8,
        "no max_categories set -> one bar per category"
    );
    assert!(bars.iter().all(|b| matches!(b, ParetoBar::Single(_))));
}

#[test]
fn test_pareto_max_categories_buckets_the_tail() {
    let plot = long_tail_categories().with_max_categories(5);
    let bars = plot.render_bars();
    // Keeps the top 4 as-is, buckets the remaining 4 into one "Other" bar.
    assert_eq!(bars.len(), 5);
    assert!(matches!(bars[0], ParetoBar::Single(ref c) if c.label == "Missing field"));
    assert!(matches!(bars[3], ParetoBar::Single(ref c) if c.label == "Bad format"));
    match &bars[4] {
        ParetoBar::Bucketed { label, segments } => {
            assert_eq!(label, "Other");
            assert_eq!(segments.len(), 4);
            // Segments preserve descending order from the tail.
            assert_eq!(segments[0].label, "Duplicate entry");
            assert_eq!(segments[3].label, "Other misc");
        }
        ParetoBar::Single(_) => panic!("expected the last bar to be bucketed"),
    }
}

#[test]
fn test_pareto_max_categories_no_effect_when_within_limit() {
    let plot = error_categories().with_max_categories(10); // only 5 categories exist
    let bars = plot.render_bars();
    assert_eq!(bars.len(), 5);
    assert!(bars.iter().all(|b| matches!(b, ParetoBar::Single(_))));
}

#[test]
fn test_pareto_max_categories_zero_disables_bucketing() {
    let plot = long_tail_categories().with_max_categories(0);
    let bars = plot.render_bars();
    assert_eq!(
        bars.len(),
        8,
        "max_categories(0) should not bucket anything"
    );
}

#[test]
fn test_pareto_custom_other_label() {
    let plot = long_tail_categories()
        .with_max_categories(5)
        .with_other_label("Miscellaneous");
    let bars = plot.render_bars();
    assert!(matches!(bars[4], ParetoBar::Bucketed { ref label, .. } if label == "Miscellaneous"));
}

#[test]
fn test_pareto_bucketed_cumulative_treats_bucket_as_one_point() {
    let plot = long_tail_categories().with_max_categories(5);
    let cum = plot.cumulative_percentages();
    assert_eq!(
        cum.len(),
        5,
        "bucket should contribute exactly one cumulative point"
    );
    assert!((cum.last().unwrap() - 100.0).abs() < 1e-9);
    // Bucket total (9+7+5+3=24) should match the last jump.
    let total = plot.total();
    let expected_last_jump = 24.0 / total * 100.0;
    let jump = cum[4] - cum[3];
    assert!(
        (jump - expected_last_jump).abs() < 1e-9,
        "bucket's cumulative jump ({jump}) should equal its summed value's share \
         ({expected_last_jump})"
    );
}

#[test]
fn test_pareto_bounds_reflect_bucketed_bar_count_and_height() {
    let plot = long_tail_categories().with_max_categories(5);
    let plots = [Plot::Pareto(plot)];
    let ((x_min, x_max), (_, y_max)) = plots[0].bounds().unwrap();
    // 5 rendered bars, not 8 raw categories.
    assert_eq!((x_min, x_max), (0.5, 5.5));
    // Tallest bar is still "Missing field" (42), which exceeds the bucket's
    // stacked total (24) -- bounds must consider both.
    assert_eq!(y_max, 42.0);
}

#[test]
fn test_pareto_bucketed_svg_shows_stack_and_legend_entries() {
    let plot = long_tail_categories().with_max_categories(5);
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Bucketed");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_bucketed.svg", svg.clone()).unwrap();
    assert!(svg.contains("Other"), "bucketed bar's x-axis label");
    for label in [
        "Duplicate entry",
        "Wrong encoding",
        "Network blip",
        "Other misc",
    ] {
        assert!(
            svg.contains(label),
            "legend should decode hidden category '{label}'"
        );
    }
    // 4 segments should use 4 distinct fill colors (plus the main bar color).
    let rect_fills: std::collections::HashSet<&str> = svg
        .split("<rect")
        .skip(1)
        .filter_map(|chunk| {
            chunk
                .split("fill=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
        })
        .collect();
    assert!(
        rect_fills.len() >= 5,
        "expected at least 5 distinct rect fill colors (main bars + bucket \
         segments), got {}: {:?}",
        rect_fills.len(),
        rect_fills
    );
}

// ── horizontal mode ──────────────────────────────────────────────────────────

#[test]
fn test_pareto_horizontal_swaps_bounds_axes() {
    let plot = error_categories().with_horizontal(true);
    let plots = [Plot::Pareto(plot)];
    let ((x_min, x_max), (y_min, y_max)) = plots[0].bounds().unwrap();
    // Categories now on Y (0.5..5.5), values on X (0..42) -- the exact swap of
    // the default vertical bounds() test above.
    assert_eq!((y_min, y_max), (0.5, 5.5));
    assert_eq!(x_min, 0.0);
    assert_eq!(x_max, 42.0);
}

#[test]
fn test_pareto_horizontal_uses_x2_not_y2_axis() {
    let plot = error_categories().with_horizontal(true);
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    assert!(
        computed.x2_range.is_some(),
        "horizontal Pareto's cumulative line pairs with the secondary X-axis"
    );
    assert!(
        computed.y2_range.is_none(),
        "horizontal Pareto should not also set up a secondary Y-axis"
    );
    let (x2_min, x2_max) = computed.x2_range.unwrap();
    assert_eq!(x2_min, 0.0);
    assert!(x2_max > 100.0 && x2_max <= 120.0);
}

#[test]
fn test_pareto_vertical_still_uses_y2_not_x2_axis() {
    // Default (non-horizontal) must be unaffected by adding the x2 system.
    let plot = error_categories();
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    assert!(computed.y2_range.is_some());
    assert!(computed.x2_range.is_none());
}

#[test]
fn test_pareto_horizontal_renders_bars_and_top_axis() {
    let plot = error_categories().with_horizontal(true);
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Horizontal");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/pareto_horizontal.svg", svg.clone()).unwrap();
    assert!(svg.contains("<rect"), "should contain bars");
    assert!(svg.contains("Missing field"), "category label on y-axis");
    assert!(svg.contains(">100%<"), "secondary x-axis should reach 100%");
}

/// Regression test for a real clipping bug: the guaranteed-100% final point sits
/// at the plot's right edge in horizontal mode, and its label (drawn 10px to the
/// *right* of the point, growing further right) got cut off by the plot's own
/// clip-path since the x2-axis headroom only clears the marker, not ~30px more
/// of label text. Fixed by flipping just the last label to anchor `End` (grow
/// leftward, back into the plot) instead of `Start`.
#[test]
fn test_pareto_horizontal_final_cumulative_label_not_clipped() {
    let plot = error_categories()
        .with_horizontal(true)
        .with_cumulative_labels(true);
    let plots = vec![Plot::Pareto(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let computed = ComputedLayout::from_layout(&layout);
    let svg = SvgBackend.render_scene(&render_multiple(
        vec![Plot::Pareto(
            error_categories()
                .with_horizontal(true)
                .with_cumulative_labels(true),
        )],
        layout,
    ));
    common::write_test_output("test_outputs/pareto_horizontal_labels.svg", svg.clone()).unwrap();

    let clip_right = computed.margin_left + computed.plot_width();
    // Find the "100%" cumulative-line label (not the axis tick, which has no
    // preceding "text-anchor=\"end\"" from our flip) and check its anchor.
    let last_label_pos = svg.rfind(">100%<").expect("100% label should be present");
    let before = &svg[..last_label_pos];
    let tag_start = before.rfind("<text").expect("text tag");
    let tag = &svg[tag_start..last_label_pos];
    assert!(
        tag.contains("text-anchor=\"end\""),
        "final cumulative label should anchor 'end' (grow leftward, into the \
         plot) to avoid being clipped at the right edge; tag: {tag}"
    );
    let x_val: f64 = tag
        .split("x=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse().ok())
        .expect("x attribute");
    assert!(
        x_val <= clip_right,
        "label x ({x_val}) should be within the plot's clip region (right edge {clip_right})"
    );
}
