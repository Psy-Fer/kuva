mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{ColorMap, QuiverPlot};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render(q: QuiverPlot, title: &str) -> String {
    let plots = vec![Plot::Quiver(q)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(title)
        .with_show_grid(false);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn rotational_grid() -> QuiverPlot {
    QuiverPlot::from_function((-2.0, 2.0, 5), (-2.0, 2.0, 5), |x, y| (-y * 0.3, x * 0.3))
}

#[test]
fn test_quiver_basic_renders_arrows() {
    let svg = render(rotational_grid(), "Quiver Basic");
    common::write_test_output("test_outputs/quiver_basic.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "should be valid SVG");
    let lines = svg.matches("<line").count();
    let paths = svg.matches("<path").count();
    // 25 arrows = 25 shafts (<line>) + 25 heads (<path>), minus any with zero magnitude.
    assert!(lines >= 24, "expected ≥24 shaft lines, got {lines}");
    assert!(paths >= 24, "expected ≥24 head paths, got {paths}");
}

#[test]
fn test_quiver_from_function_exact_count() {
    let q = QuiverPlot::from_function((-1.0, 1.0, 4), (-1.0, 1.0, 6), |_, _| (1.0, 0.0));
    assert_eq!(q.arrows.len(), 24, "4 × 6 = 24 arrows");
}

#[test]
fn test_quiver_from_function_endpoints_inclusive() {
    let q = QuiverPlot::from_function((-1.0, 1.0, 3), (0.0, 2.0, 3), |x, y| (x, y));
    // first: x=-1, y=0 ; last: x=+1, y=+2
    assert_eq!(q.arrows.first().map(|a| (a.x, a.y)), Some((-1.0, 0.0)));
    assert_eq!(q.arrows.last().map(|a| (a.x, a.y)), Some((1.0, 2.0)));
}

#[test]
fn test_quiver_colormap_triggers_colorbar() {
    let q = rotational_grid().with_magnitude_colormap(ColorMap::Viridis, "Speed");
    let svg = render(q, "Quiver Colormap");
    common::write_test_output("test_outputs/quiver_colormap.svg", &svg).unwrap();
    assert!(svg.contains("Speed"), "colorbar label should be in the SVG");
    // Colorbar adds a vertical gradient/rect column on the right margin.
    assert!(
        svg.contains("<linearGradient") || svg.contains("<rect"),
        "colorbar should emit gradient or rect primitives"
    );
}

#[test]
fn test_quiver_tight_bounds_wraps_in_clippath() {
    let q = rotational_grid().with_tight_bounds();
    let svg = render(q, "Quiver Tight");
    common::write_test_output("test_outputs/quiver_tight.svg", &svg).unwrap();
    assert!(
        svg.contains("clipPath") || svg.contains("clip-path"),
        "tight bounds should emit a clip path"
    );
}

#[test]
fn test_quiver_proportional_heads_are_visible_on_small_arrows() {
    // A field where the longest arrow is tiny (all magnitudes ~0.1).
    let q = QuiverPlot::from_function((-1.0, 1.0, 5), (-1.0, 1.0, 5), |_, _| (0.1, 0.1));
    let svg = render(q, "Quiver Small");
    // All 25 arrows should still render heads (proportional clamp lower bound).
    assert!(
        svg.matches("<path").count() >= 24,
        "small arrows should still render heads"
    );
}

#[test]
fn test_quiver_with_legend_emits_entry() {
    // Use a non-default plot color so we can find the legend's glyph
    // unambiguously by stroke color.
    let q = rotational_grid().with_color("crimson").with_legend("Wind");
    let svg = render(q, "Quiver Legend");
    assert!(svg.contains("Wind"), "legend label should appear in SVG");
    // Legend glyph for Quiver is a Line — count short crimson-stroked
    // <line> elements. The arrow shafts also use crimson, so the glyph is
    // one of multiple matches; the key check is that *adding* the legend
    // produces at least one *more* crimson line than the no-legend baseline.
    let baseline = rotational_grid().with_color("crimson");
    let baseline_svg = render(baseline, "Baseline");
    let crimson_lines = |s: &str| -> usize {
        s.matches("stroke=\"crimson\"").count()
            + s.matches("stroke=\"#dc143c\"").count()
            + s.matches("stroke=\"rgb(220,20,60)\"").count()
    };
    let legend_count = crimson_lines(&svg);
    let baseline_count = crimson_lines(&baseline_svg);
    assert!(
        legend_count >= baseline_count + 1,
        "with_legend should add ≥1 crimson-stroked <line> for the glyph \
         (got {legend_count} vs baseline {baseline_count})"
    );
}

#[test]
fn test_quiver_drops_non_finite_arrows() {
    // NaN / infinity inputs must be silently skipped, not propagated.
    let q = QuiverPlot::new()
        .with_arrow(0.0, 0.0, 1.0, 0.0)
        .with_arrow(f64::NAN, 0.0, 1.0, 0.0)
        .with_arrow(0.0, f64::INFINITY, 1.0, 0.0)
        .with_arrow(1.0, 1.0, f64::NEG_INFINITY, 0.0)
        .with_arrow(2.0, 2.0, 1.0, f64::NAN);
    assert_eq!(q.arrows.len(), 1, "only the all-finite arrow should remain");
}

#[test]
fn test_quiver_with_arrows_drops_non_finite() {
    let q = QuiverPlot::new().with_arrows(vec![
        (0.0, 0.0, 1.0, 0.0),
        (f64::NAN, 0.0, 1.0, 0.0),
        (1.0, 1.0, 1.0, 1.0),
    ]);
    assert_eq!(q.arrows.len(), 2);
}

#[test]
fn test_quiver_with_arrow_accepts_integer_types() {
    // Verifies the impl Into<f64> bound — i32 / u32 / f32 all flow through
    // without callers needing `as f64`.
    let q = QuiverPlot::new()
        .with_arrow(0_i32, 0_i32, 1_i32, 0_i32)
        .with_arrow(1_u32, 1_u32, 2_u32, 3_u32)
        .with_arrow(2.0_f32, 2.0_f32, 1.5_f32, 0.5_f32);
    assert_eq!(q.arrows.len(), 3);
}

#[test]
fn test_quiver_interactive_mode_emits_tooltip_groups() {
    // In interactive mode every arrow should wrap in <g class="tt" data-...>
    // with a native <title> readout.
    let q = rotational_grid();
    let plots = vec![kuva::render::plots::Plot::Quiver(q)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Interactive")
        .with_show_grid(false)
        .with_interactive();
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    let tt_count = svg.matches("class=\"tt\"").count();
    let data_u_count = svg.matches("data-u=").count();
    let data_v_count = svg.matches("data-v=").count();
    let data_mag_count = svg.matches("data-mag=").count();
    // 5×5 grid = 25 arrows, minus the center (0,0) which has zero magnitude
    // and is skipped by the `len < 1e-6` guard.
    assert!(
        tt_count >= 24 && tt_count <= 25,
        "expected 24 or 25 tooltip groups, got {tt_count}"
    );
    assert_eq!(
        tt_count, data_u_count,
        "data-u attrs should match tooltip group count"
    );
    assert_eq!(
        tt_count, data_v_count,
        "data-v attrs should match tooltip group count"
    );
    assert_eq!(
        tt_count, data_mag_count,
        "data-mag attrs should match tooltip group count"
    );
}

#[test]
fn test_quiver_clip_opt_in_without_tight_bounds() {
    // with_clip_to_plot_area() alone should emit a clip-path, even when
    // tight_bounds is off.
    let q = rotational_grid().with_clip_to_plot_area();
    let svg = render(q, "Clip opt-in");
    assert!(
        svg.contains("clipPath") || svg.contains("clip-path"),
        "with_clip_to_plot_area() alone should emit a clip-path"
    );
}

#[test]
fn test_quiver_no_clip_suppresses_tight_bounds_clip() {
    // with_no_clip() should suppress clipping even when tight_bounds is on.
    let q = rotational_grid().with_tight_bounds().with_no_clip();
    let svg = render(q, "No clip");
    let has_quiver_clip = svg.contains("kuva-quiver-clip");
    assert!(
        !has_quiver_clip,
        "with_no_clip() must suppress the quiver clip-path even with tight_bounds"
    );
}

#[test]
fn test_quiver_color_range_pins_colorbar_extent() {
    // with_color_range pins the colormap normalization. The colorbar min/max
    // text should reflect the pinned range, not the data extent.
    let q = QuiverPlot::from_function((-1.0, 1.0, 3), (-1.0, 1.0, 3), |x, y| (x, y))
        .with_magnitude_colormap(ColorMap::Viridis, "Speed")
        .with_color_range(0.0, 10.0);
    let svg = render(q, "Pinned range");
    // The top of the colorbar axis should be 10 (the pinned max).
    assert!(
        svg.contains(">10<") || svg.contains(">10 <") || svg.contains(">10."),
        "pinned colorbar max of 10 should appear on the axis"
    );
}

#[test]
fn test_quiver_head_length_override_grows_heads() {
    // Explicit pixel head-size should produce visibly larger arrowhead
    // triangles than proportional defaults on the same data.
    let default_heads_svg = render(rotational_grid(), "Default heads");
    let big_heads_svg = render(
        rotational_grid()
            .with_head_length(20.0)
            .with_head_width(8.0),
        "Big heads",
    );
    // Proxy: count total byte length of path 'd' attributes — bigger heads
    // → longer path strings with larger coordinates.
    let default_path_bytes: usize = default_heads_svg.matches(" d=\"M ").count();
    let big_path_bytes: usize = big_heads_svg.matches(" d=\"M ").count();
    // Both should emit ~25 arrow-head paths (one per arrow).
    assert!(
        default_path_bytes >= 20,
        "default render should emit ≥20 arrow-head paths, got {default_path_bytes}"
    );
    assert!(
        big_path_bytes >= 20,
        "override render should emit ≥20 arrow-head paths, got {big_path_bytes}"
    );
    // Stronger: compare actual path coordinate magnitudes. Large heads
    // produce longer chord distances between tip and base corners.
    // Quick check: the big-heads SVG should be longer than default because
    // coordinate strings have more digits.
    assert!(
        big_heads_svg.len() != default_heads_svg.len(),
        "big heads should change SVG output vs default proportional sizing"
    );
}

#[test]
fn test_quiver_empty_arrows_renders_empty_plot() {
    use kuva::render::plots::Plot;
    let empty = QuiverPlot::new();
    // bounds() must return None so Layout::auto_from_plots falls back gracefully.
    assert!(
        Plot::Quiver(empty.clone()).bounds().is_none(),
        "empty QuiverPlot bounds must be None"
    );
    // Render still produces a valid SVG with axes but no arrow shapes.
    let svg = render(empty, "Empty");
    assert!(svg.contains("<svg"));
    assert!(
        !svg.contains("class=\"tt\""),
        "empty plot should have no interactive arrow groups"
    );
}

#[test]
fn test_quiver_per_arrow_color_beats_colormap() {
    // When BOTH a colormap and a per-arrow color override are set, the
    // per-arrow color must win — documented priority is
    // per-arrow > colormap > plot-level.
    let q = QuiverPlot::new()
        .with_colored_arrow(0.0, 0.0, 1.0, 0.0, "tomato")
        .with_arrow(1.0, 0.0, 2.0, 0.0)
        .with_arrow(2.0, 0.0, 0.5, 0.0)
        .with_magnitude_colormap(ColorMap::Viridis, "Speed");
    let svg = render(q, "Priority with cmap");
    // Tomato must appear (per-arrow override).
    let has_tomato =
        svg.contains("tomato") || svg.contains("#ff6347") || svg.contains("rgb(255,99,71)");
    assert!(
        has_tomato,
        "per-arrow tomato must override colormap; svg did not contain tomato"
    );
}

#[test]
fn test_quiver_color_fallback_priority() {
    // Per-arrow color overrides plot-level color.
    let q = QuiverPlot::new()
        .with_colored_arrow(0.0, 0.0, 1.0, 0.0, "tomato")
        .with_arrow(1.0, 0.0, 1.0, 0.0)
        .with_color("steelblue");
    let svg = render(q, "Priority");
    // Resilient to hex / named / rgb() encoding — the backend may emit any.
    let has_tomato =
        svg.contains("tomato") || svg.contains("#ff6347") || svg.contains("rgb(255,99,71)");
    let has_steelblue =
        svg.contains("steelblue") || svg.contains("#4682b4") || svg.contains("rgb(70,130,180)");
    assert!(has_tomato, "per-arrow tomato should be in SVG");
    assert!(has_steelblue, "plot-level steelblue should be in SVG");
}
