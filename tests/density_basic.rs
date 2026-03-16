use kuva::plot::DensityPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::palette::Palette;
use kuva::backend::svg::SvgBackend;
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
    let dp = DensityPlot::new()
        .with_data(data)
        .with_color("steelblue");
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Basic")
        .with_x_label("Value")
        .with_y_label("Density");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_basic.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "output should contain <svg tag");
    assert!(svg.contains("<path"), "output should contain a <path element");
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
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Filled");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_filled.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // The filled path should include a Z (close path) command
    assert!(svg.contains('Z'), "filled density path should contain 'Z' close command");
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
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Custom Bandwidth");
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
    assert!(svg.contains("Group A"), "legend label 'Group A' should appear in SVG");
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
            DensityPlot::new().with_data(group_a).with_color(pal[0].to_string()).with_legend("A")
        ),
        Plot::Density(
            DensityPlot::new().with_data(group_b).with_color(pal[1].to_string()).with_legend("B")
        ),
        Plot::Density(
            DensityPlot::new().with_data(group_c).with_color(pal[2].to_string()).with_legend("C")
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
    assert!(path_count >= 3, "expected at least 3 path elements, got {path_count}");
}

#[test]
fn test_density_precomputed() {
    outdir();
    let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![0.05, 0.2, 0.5, 0.4, 0.2, 0.05];
    let dp = DensityPlot::from_curve(x, y).with_color("orange");
    let plots = vec![Plot::Density(dp)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density Precomputed");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_precomputed.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"), "precomputed density should emit a path");
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
    assert!(x_min < 0.0,
        "without x_range, KDE tail should extend below data_min=0.0; got x_min={x_min}");
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
    assert_eq!(x_min, 0.0, "x_min should be exactly the lower bound of x_range");
    assert_eq!(x_max, 1.0, "x_max should be exactly the upper bound of x_range");
    assert_eq!(y_min, 0.0, "y_min should be 0.0 for a density");
    assert!(y_max > 0.0, "y_max should be positive; KDE peak was not found");
}

/// The rendered SVG should not contain NaN or empty paths when x_range is set
/// and the data lives entirely within the clamped region.
#[test]
fn test_density_x_range_renders_cleanly() {
    outdir();
    // Simulate methylation β-value data: bimodal near 0 and 1
    let mut data: Vec<f64> = (0..50).map(|i| i as f64 * 0.01).collect();      // 0.0 – 0.49
    data.extend((51..100).map(|i| i as f64 * 0.01));                           // 0.51 – 0.99
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
    assert!(svg.contains("<path"), "density with x_range should produce a path");
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
    assert!(y_max > 1.0,
        "bounds() y_max should reflect the KDE peak; got y_max={y_max}. \
         This fails if bounds() uses too few samples and misses the peak.");
}

/// Multi-group density with x_range: each group curve should be clamped,
/// the plot should render without errors.
#[test]
fn test_density_multigroup_x_range() {
    outdir();
    let pal = Palette::category10();
    let group_a: Vec<f64> = (0..50).map(|i| i as f64 * 0.01).collect();       // [0, 0.49]
    let group_b: Vec<f64> = (50..100).map(|i| i as f64 * 0.01).collect();     // [0.50, 0.99]
    let plots = vec![
        Plot::Density(
            DensityPlot::new()
                .with_data(group_a)
                .with_color(pal[0].to_string())
                .with_legend("Low")
                .with_x_range(0.0, 1.0)
        ),
        Plot::Density(
            DensityPlot::new()
                .with_data(group_b)
                .with_color(pal[1].to_string())
                .with_legend("High")
                .with_x_range(0.0, 1.0)
        ),
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Density multigroup bounded");
    let svg = render_svg(plots, layout);
    fs::write("test_outputs/density_multigroup_x_range.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"), "no NaN in multigroup bounded density");
    let path_count = svg.matches("<path").count();
    assert!(path_count >= 2, "expected at least one path per group; got {path_count}");
}
