use kuva::backend::svg::SvgBackend;
use kuva::plot::DensityPlot;
use kuva::render::layout::Layout;
use kuva::render::palette::Palette;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use std::fs;

// ── Regression tests for #37: density plot unexpectedly behaving on [0,1] data ─

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

fn outdir() {
    fs::create_dir_all("test_outputs").ok();
}

#[test]
fn test_density_basic() {
    outdir();
    let data: Vec<f64> = (0..100).map(|i| (i as f64) * 0.1).collect();
    let dp = DensityPlot::new().with_data(data).with_color("steelblue");
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Basic")
        .with_x_label("Value")
        .with_y_label("Density");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_basic.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "output should contain <svg tag");
    assert!(
        svg.contains("<path"),
        "output should contain a <path element"
    );
}

#[test]
fn test_density_filled() {
    outdir();
    let data: Vec<f64> = vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 2.0, 2.5, 3.0];
    let dp = DensityPlot::new()
        .with_data(data)
        .with_color("coral")
        .with_filled(true)
        .with_opacity(0.4);
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Density Filled");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_filled.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // The filled path should include a Z (close path) command
    assert!(
        svg.contains('Z'),
        "filled density path should contain 'Z' close command"
    );
}

#[test]
fn test_density_bandwidth() {
    outdir();
    let data: Vec<f64> = vec![1.0, 2.0, 2.1, 2.9, 3.0, 3.1, 4.0, 4.5, 5.0];
    let dp = DensityPlot::new()
        .with_data(data)
        .with_color("purple")
        .with_bandwidth(0.3);
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Density Custom Bandwidth");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_bandwidth.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
}

#[test]
fn test_density_legend() {
    outdir();
    let data: Vec<f64> = vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 2.2, 2.8, 3.2];
    let dp = DensityPlot::new()
        .with_data(data)
        .with_color("teal")
        .with_legend("Group A");
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Legend")
        .with_legend_position(kuva::plot::LegendPosition::OutsideRightTop);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_legend.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("Group A"),
        "legend label 'Group A' should appear in SVG"
    );
}

#[test]
fn test_density_multigroup() {
    outdir();
    let pal = Palette::category10();
    let group_a: Vec<f64> = vec![1.0, 1.5, 2.0, 2.5, 3.0];
    let group_b: Vec<f64> = vec![2.5, 3.0, 3.5, 4.0, 4.5];
    let group_c: Vec<f64> = vec![0.5, 1.0, 1.5, 2.0, 2.5, 3.0];

    let plots = vec![
        Plot::Density(
            DensityPlot::new()
                .with_data(group_a)
                .with_color(pal[0].to_string())
                .with_legend("A"),
        ),
        Plot::Density(
            DensityPlot::new()
                .with_data(group_b)
                .with_color(pal[1].to_string())
                .with_legend("B"),
        ),
        Plot::Density(
            DensityPlot::new()
                .with_data(group_c)
                .with_color(pal[2].to_string())
                .with_legend("C"),
        ),
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Multigroup")
        .with_legend_position(kuva::plot::LegendPosition::OutsideRightTop);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_multigroup.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // Should have multiple path elements
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 3,
        "expected at least 3 path elements, got {path_count}"
    );
}

#[test]
fn test_density_precomputed() {
    outdir();
    let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![0.05, 0.2, 0.5, 0.4, 0.2, 0.05];
    let dp = DensityPlot::from_curve(x, y).with_color("orange");
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Density Precomputed");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_precomputed.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("<path"),
        "precomputed density should emit a path"
    );
}

/// With no x_range set, the KDE evaluation extends 3×bandwidth below the data
/// minimum, so bounds() x_min should be negative for data starting at 0.0.
/// This is the default behaviour (tails taper smoothly), but it means the curve
/// bleeds into negative territory for bounded data like methylation frequencies.
#[test]
fn test_density_unbounded_extends_below_zero() {
    let data: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect(); // [0.0, 0.99]
    let dp = DensityPlot::new().with_data(data);
    let plot = Plot::Density(dp);
    let ((x_min, _), _) = plot.bounds().unwrap();
    assert!(
        x_min < 0.0,
        "without x_range, KDE tail should extend below data_min=0.0; got x_min={x_min}"
    );
}

/// with_x_range(0.0, 1.0) clamps the KDE evaluation range.  bounds() must
/// return exactly (0.0, 1.0) as the x extent, preventing the curve from
/// extending into negative values — the root cause of #37 on [0,1] data.
#[test]
fn test_density_x_range_clamps_bounds() {
    let data: Vec<f64> = (0..100).map(|i| i as f64 / 100.0).collect();
    let dp = DensityPlot::new().with_data(data).with_x_range(0.0, 1.0);
    let plot = Plot::Density(dp);
    let ((x_min, x_max), (y_min, y_max)) = plot.bounds().unwrap();
    assert_eq!(
        x_min, 0.0,
        "x_min should be exactly the lower bound of x_range"
    );
    assert_eq!(
        x_max, 1.0,
        "x_max should be exactly the upper bound of x_range"
    );
    assert_eq!(y_min, 0.0, "y_min should be 0.0 for a density");
    assert!(
        y_max > 0.0,
        "y_max should be positive; KDE peak was not found"
    );
}

/// The rendered SVG should not contain NaN or empty paths when x_range is set
/// and the data lives entirely within the clamped region.
#[test]
fn test_density_x_range_renders_cleanly() {
    outdir();
    // Simulate methylation β-value data: bimodal near 0 and 1
    let mut data: Vec<f64> = (0..50).map(|i| i as f64 * 0.01).collect(); // 0.0 – 0.49
    data.extend((51..100).map(|i| i as f64 * 0.01)); // 0.51 – 0.99
    let dp = DensityPlot::new()
        .with_data(data)
        .with_x_range(0.0, 1.0)
        .with_filled(true);
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Methylation-like density")
        .with_x_label("β-value")
        .with_y_label("Density");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_x_range.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("<path"),
        "density with x_range should produce a path"
    );
    assert!(!svg.contains("NaN"), "SVG should not contain NaN");
}

/// For narrow-bandwidth data, bounds() must correctly capture the KDE peak.
/// Before the fix, bounds() used only 50 sample points; the renderer used
/// dp.kde_samples (200).  When the evaluation step was coarser than the peak
/// width the 50-sample version could miss the peak entirely, returning y_max≈0
/// and causing the axis to clip the top of the rendered curve.
///
/// This test uses an explicit narrow bandwidth and verifies that bounds()
/// returns a strictly positive y_max — which it would fail to do if it still
/// used 50 samples with a step larger than the bandwidth.
#[test]
fn test_density_narrow_bandwidth_bounds_nonzero() {
    // 60 points clustered in a narrow range — with bw=0.02 the peak is sharp.
    let data: Vec<f64> = (0..60).map(|i| 2.0 + i as f64 * 0.01).collect(); // [2.0, 2.59]
    let dp = DensityPlot::new()
        .with_data(data)
        .with_bandwidth(0.02)
        .with_kde_samples(200);
    let plot = Plot::Density(dp);
    let (_, (_, y_max)) = plot.bounds().unwrap();
    // With bw=0.02 and n=60, the KDE peak ≈ 60/(0.02*√(2π)) ≈ 1194.
    // Even with 10% headroom the y_max should be >> 1.
    assert!(
        y_max > 1.0,
        "bounds() y_max should reflect the KDE peak; got y_max={y_max}. \
         This fails if bounds() uses too few samples and misses the peak."
    );
}

/// Multi-group density with x_range: each group curve should be clamped,
/// the plot should render without errors.
#[test]
fn test_density_multigroup_x_range() {
    outdir();
    let pal = Palette::category10();
    let group_a: Vec<f64> = (0..50).map(|i| i as f64 * 0.01).collect(); // [0, 0.49]
    let group_b: Vec<f64> = (50..100).map(|i| i as f64 * 0.01).collect(); // [0.50, 0.99]
    let plots = vec![
        Plot::Density(
            DensityPlot::new()
                .with_data(group_a)
                .with_color(pal[0].to_string())
                .with_legend("Low")
                .with_x_range(0.0, 1.0),
        ),
        Plot::Density(
            DensityPlot::new()
                .with_data(group_b)
                .with_color(pal[1].to_string())
                .with_legend("High")
                .with_x_range(0.0, 1.0),
        ),
    ];
    let layout = Layout::auto_from_plots(&plots).with_title("Density multigroup bounded");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_multigroup_x_range.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"), "no NaN in multigroup bounded density");
    let path_count = svg.matches("<path").count();
    assert!(
        path_count >= 2,
        "expected at least one path per group; got {path_count}"
    );
}

// ── Boundary reflection tests (#47) ──────────────────────────────────────────

/// Reflection: the KDE evaluated at the lower bound should be non-trivially
/// higher than without reflection, because the ghost points restore probability
/// mass that a plain Gaussian kernel would lose past x=0.
/// We verify this by comparing density at x=0 with and without the bound set.
#[test]
fn test_density_reflection_raises_boundary_density() {
    // Data concentrated near 0 — standard KDE would lose ~50% of kernel mass
    // below x=0 at points right at the boundary.
    let data: Vec<f64> = (0..50).map(|i| i as f64 * 0.02).collect(); // [0.0, 0.98]

    // Without reflection: evaluate normally (tails extend past 0)
    let dp_plain = DensityPlot::new()
        .with_data(data.clone())
        .with_bandwidth(0.1);
    // With reflection at x=0: ghost points restore lost mass
    let dp_reflect = DensityPlot::new()
        .with_data(data)
        .with_bandwidth(0.1)
        .with_x_lo(0.0);

    // bounds() uses the same KDE path as the renderer; peak y with reflection
    // should be higher (boundary is not underestimated).
    let (_, (_, y_plain)) = Plot::Density(dp_plain).bounds().unwrap();
    let (_, (_, y_reflect)) = Plot::Density(dp_reflect).bounds().unwrap();

    // The reflected version should produce a meaningfully higher peak
    // (reflection corrects the ~50% loss at the boundary).
    assert!(
        y_reflect > y_plain,
        "reflection should raise peak density; plain={y_plain:.4} reflect={y_reflect:.4}"
    );
}

/// One-sided x_lo: the left boundary is clamped and reflected; the right tail
/// is deliberately free to extend past data_max (> 0.79, possibly past 1.0).
/// Use with_x_range(0.0, 1.0) when you need both sides clamped.
#[test]
fn test_density_x_lo_only() {
    outdir();
    let data: Vec<f64> = (0..80).map(|i| i as f64 * 0.01).collect(); // [0.0, 0.79]
    let dp = DensityPlot::new()
        .with_data(data)
        .with_x_lo(0.0)
        .with_filled(true);
    let plot = Plot::Density(dp);
    let ((x_min, x_max), _) = plot.bounds().unwrap();
    assert_eq!(x_min, 0.0, "x_lo should clamp x_min to exactly 0.0");
    assert!(
        x_max > 0.79,
        "right tail should still extend past data_max (one-sided bound)"
    );

    // Also renders without NaN
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_x_lo_only.svg", &svg).unwrap();
    assert!(!svg.contains("NaN"));
    assert!(svg.contains("<path"));
}

/// One-sided x_hi: the right boundary is clamped and reflected; the left tail
/// is deliberately free to extend past data_min (< 0.2, possibly below 0.0).
/// Use with_x_range(0.0, 1.0) when you need both sides clamped.
#[test]
fn test_density_x_hi_only() {
    outdir();
    let data: Vec<f64> = (20..100).map(|i| i as f64 * 0.01).collect(); // [0.2, 0.99]
    let dp = DensityPlot::new().with_data(data).with_x_hi(1.0);
    let plot = Plot::Density(dp);
    let ((x_min, x_max), _) = plot.bounds().unwrap();
    assert_eq!(x_max, 1.0, "x_hi should clamp x_max to exactly 1.0");
    assert!(
        x_min < 0.2,
        "left tail should still extend past data_min (one-sided bound)"
    );

    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_x_hi_only.svg", &svg).unwrap();
    assert!(!svg.contains("NaN"));
    assert!(svg.contains("<path"));
}

/// Bimodal bounded data: identity scores with peaks at 0.1 and 0.9.
/// With x_range(0.0, 1.0), neither peak should bleed past the boundaries.
#[test]
fn test_density_bounded_bimodal_identity_scores() {
    outdir();
    let mut data: Vec<f64> = (0..40).map(|i| 0.05 + i as f64 * 0.005).collect(); // near 0
    data.extend((0..40).map(|i| 0.85 + i as f64 * 0.005)); // near 1
    let dp = DensityPlot::new()
        .with_data(data)
        .with_x_range(0.0, 1.0)
        .with_filled(true)
        .with_color("steelblue");
    let plot = Plot::Density(dp);
    let ((x_min, x_max), _) = plot.bounds().unwrap();
    assert_eq!(x_min, 0.0);
    assert_eq!(x_max, 1.0);

    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_bounded_bimodal.svg", &svg).unwrap();
    assert!(!svg.contains("NaN"));
    assert!(svg.contains("<path"));
}

// ── with_fit() / anchor_y_zero tests ─────────────────────────────────────────

/// Without with_fit(), auto_from_plots anchors the y-axis at zero.
#[test]
fn test_density_default_anchors_y_zero() {
    let data: Vec<f64> = (0..50).map(|i| 5.0 + i as f64 * 0.1).collect();
    let dp = DensityPlot::new().with_data(data);
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots);
    assert!(
        layout.anchor_y_zero,
        "density without with_fit() should anchor y at zero"
    );
    assert_eq!(
        layout.y_range.0, 0.0,
        "y_min should be 0.0 when anchored (got {})",
        layout.y_range.0
    );
}

/// with_fit() disables the zero anchor; layout.anchor_y_zero is false and
/// y_min is computed from the data range rather than clamped to 0.
#[test]
fn test_density_fit_disables_zero_anchor() {
    let data: Vec<f64> = (0..50).map(|i| 5.0 + i as f64 * 0.1).collect();
    let dp = DensityPlot::new().with_data(data).with_fit();
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots);
    assert!(
        !layout.anchor_y_zero,
        "density with with_fit() should not anchor y at zero"
    );
}

/// For a precomputed curve whose y values start well above zero, with_fit()
/// produces a y_min that tracks the data rather than clamping to 0.
#[test]
fn test_density_fit_ymin_tracks_data() {
    // Precomputed curve: y values range from 0.5 to 1.0 — never touches zero.
    let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let y = vec![0.5, 0.8, 1.0, 0.8, 0.5];
    let dp_default = DensityPlot::from_curve(x.clone(), y.clone());
    let dp_fit = DensityPlot::from_curve(x, y).with_fit();

    let layout_default = Layout::auto_from_plots(&[Plot::Density(dp_default)]);
    let layout_fit = Layout::auto_from_plots(&[Plot::Density(dp_fit)]);

    assert_eq!(
        layout_default.y_range.0, 0.0,
        "without with_fit(), y_min should be 0.0"
    );
    assert!(
        layout_fit.y_range.0 > 0.0,
        "with with_fit(), y_min should be > 0.0 for data that never touches zero; got {}",
        layout_fit.y_range.0
    );
}

/// When with_fit() density plots are mixed with an anchoring plot type (bar),
/// anchor_y_zero stays true — the bar plot wins.
#[test]
fn test_density_fit_overridden_by_bar() {
    use kuva::plot::BarPlot;
    let dp = DensityPlot::new().with_data(vec![1.0, 2.0, 3.0]).with_fit();
    let bar = BarPlot::new().with_bar("A", 5.0);
    let plots = vec![Plot::Density(dp), Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots);
    assert!(
        layout.anchor_y_zero,
        "bar plot should keep anchor_y_zero true even when a density uses with_fit()"
    );
}

/// Rendered SVG with with_fit() on a precomputed curve (y values never reach 0)
/// should start the y-axis above zero and contain no NaN.
#[test]
fn test_density_fit_renders_cleanly() {
    outdir();
    // Precomputed curve whose baseline is at 0.3 — never touches zero.
    let x: Vec<f64> = (0..9).map(|i| i as f64 * 0.5).collect();
    let y = vec![0.30, 0.45, 0.70, 0.90, 1.00, 0.90, 0.70, 0.45, 0.30];
    let dp = DensityPlot::from_curve(x, y)
        .with_color("steelblue")
        .with_filled(true)
        .with_fit();
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots).with_title("Density fit-to-data");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_fit.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
    assert!(!svg.contains("NaN"), "SVG should not contain NaN");
    // The y-axis should not start at 0 — fit_y tracks the curve minimum (0.3).
    let layout2 = Layout::auto_from_plots(&[Plot::Density(
        DensityPlot::from_curve(
            (0..9).map(|i| i as f64 * 0.5).collect(),
            vec![0.30, 0.45, 0.70, 0.90, 1.00, 0.90, 0.70, 0.45, 0.30],
        )
        .with_fit(),
    )]);
    assert!(
        layout2.y_range.0 > 0.0,
        "y-axis should start above 0 when fit_y is set; got {}",
        layout2.y_range.0
    );
}
