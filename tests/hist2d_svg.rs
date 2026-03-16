use kuva::plot::Histogram2D;
use kuva::plot::histogram2d::ColorMap;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

use rand_distr::{Normal, Distribution};

fn outdir() {
    std::fs::create_dir_all("test_outputs").ok();
}

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    let scene = render_multiple(plots, layout);
    SvgBackend.render_scene(&scene)
}

#[test]
fn test_histogram2d_svg_output_builder() {
    outdir();
    let normal_x = Normal::new(10.0, 2.0).unwrap();
    let normal_y = Normal::new(12.0, 3.0).unwrap();
    let mut rng = rand::rng();
    let data: Vec<(f64, f64)> = (0..10000)
        .map(|_| (normal_x.sample(&mut rng), normal_y.sample(&mut rng)))
        .collect();

    let hist2d = Histogram2D::new()
        .with_data(data, (0.0, 30.0), (0.0, 30.0), 30, 30)
        .with_color_map(ColorMap::Inferno)
        .with_correlation();

    let plots = vec![Plot::Histogram2d(hist2d)];
    let layout = Layout::auto_from_plots(&plots).with_title("Histogram2D");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hist2d_builder.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
}

// ── Regression tests for #39: hist2d not well-behaving on real data ───────────

/// A data point at exactly x_range.1 (the maximum boundary) must land in the
/// last bin, not be silently dropped.  Before the fix the condition
/// `x >= x_range.1` excluded it; now it is clamped with `.min(bins_x - 1)`.
#[test]
fn test_hist2d_max_boundary_point_in_last_bin() {
    let data = vec![(0.0f64, 0.0f64), (5.0, 5.0), (10.0, 10.0)];
    let hist = Histogram2D::new().with_data(data, (0.0, 10.0), (0.0, 10.0), 10, 10);

    let total: usize = hist.bins.iter().flatten().sum();
    assert_eq!(total, 3, "all 3 points should be binned, including the one at x_range.1");

    // The point (10, 10) should end up in the last cell [9][9].
    assert_eq!(hist.bins[9][9], 1, "point at (10,10) should be in the last bin [9][9]");
}

/// All data points within the stated range (inclusive) must appear exactly once
/// across all bins.
#[test]
fn test_hist2d_bin_total_equals_in_range_count() {
    // 25 points on a 5×5 grid within [0, 4] × [0, 4] — all within range.
    let data: Vec<(f64, f64)> = (0..5).flat_map(|i| {
        (0..5).map(move |j| (i as f64, j as f64))
    }).collect();

    let hist = Histogram2D::new().with_data(data, (0.0, 4.0), (0.0, 4.0), 4, 4);
    let total: usize = hist.bins.iter().flatten().sum();
    // All 25 points are within [0,4]×[0,4]; expect all to be counted.
    assert_eq!(total, 25, "sum of all bins should equal the number of in-range points");
}

/// When data has outliers far outside the stated range, they are excluded from
/// the bins.  An explicit range is what lets the user focus on the dense region
/// of real data without sparse bins caused by a few extreme values.
#[test]
fn test_hist2d_outlier_excluded_by_explicit_range() {
    // 100 points uniformly in [0, 9.9] × [0, 9.9], plus a single outlier at 1000.
    let mut data: Vec<(f64, f64)> = (0..100).map(|i| (i as f64 * 0.1, i as f64 * 0.1)).collect();
    data.push((1000.0, 1000.0));

    // With explicit range [0, 10]: outlier is outside the range and excluded.
    let hist_ranged = Histogram2D::new().with_data(data.clone(), (0.0, 10.0), (0.0, 10.0), 10, 10);
    let total_ranged: usize = hist_ranged.bins.iter().flatten().sum();
    assert_eq!(total_ranged, 100, "outlier at 1000 should be excluded by the [0,10] range");

    // The 100 in-range points form a diagonal; bins should be spread across the
    // grid, not piled into the first column (which is what happens when the
    // outlier forces a [0,1000] range with width-100 bins).
    let first_col_total: usize = hist_ranged.bins.iter().map(|row| row[0]).sum();
    assert!(first_col_total <= 10,
        "with explicit range, data should spread across bins; first col={first_col_total}");

    // Without explicit range (use full data range 0..1000): all 100 main-cluster
    // points collapse into the first bin because bin_width = 100.
    let hist_wide = Histogram2D::new().with_data(data, (0.0, 1000.0), (0.0, 1000.0), 10, 10);
    let total_wide: usize = hist_wide.bins.iter().flatten().sum();
    assert_eq!(total_wide, 101, "full range should include all 101 points");
    assert_eq!(hist_wide.bins[0][0], 100,
        "with [0,1000] range and 10 bins (width=100), all main points pile into bin [0][0]");
}

/// Explicit range affects the binning, not just the display: the rendered SVG
/// should contain coloured cells when the range is set appropriately.
#[test]
fn test_hist2d_explicit_range_renders_filled_bins() {
    outdir();
    // Skewed data: 99 points near 0, one outlier at 100.
    let mut data: Vec<(f64, f64)> = (0..99).map(|i| (i as f64 * 0.1, i as f64 * 0.1)).collect();
    data.push((100.0, 100.0));

    // With the outlier forcing (0, 100) range: almost all points pile in bin 0,
    // the SVG will barely have any coloured cells outside the first.
    // With explicit (0, 10) range: bins are well-distributed, SVG has many rects.
    let hist = Histogram2D::new()
        .with_data(data, (0.0, 10.0), (0.0, 10.0), 10, 10)
        .with_color_map(ColorMap::Viridis);
    let plots = vec![Plot::Histogram2d(hist)];
    let layout = Layout::auto_from_plots(&plots).with_title("hist2d explicit range");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hist2d_explicit_range.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(!svg.contains("NaN"), "SVG should contain no NaN values");
    // Multiple coloured rects expected (one per non-empty bin).
    let rect_count = svg.matches("<rect").count();
    assert!(rect_count >= 5, "expected multiple coloured bins; got {rect_count} rects");
}
