use kuva::plot::hexbin::{HexbinPlot, ZReduce, ColorMap};
use kuva::render::{plots::Plot, layout::Layout, render::render_multiple};
use kuva::backend::svg::SvgBackend;

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Simple deterministic point generator — produces a ring of clusters.
fn make_points(n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / n as f64 * std::f64::consts::TAU;
        x.push(t.cos() * (1.0 + 0.3 * (i as f64 * 0.1).sin()) + (i % 7) as f64 * 0.1);
        y.push(t.sin() * (1.0 + 0.3 * (i as f64 * 0.1).cos()) + (i % 5) as f64 * 0.1);
    }
    (x, y)
}

/// Three-cluster dataset spread deterministically.
fn make_clusters(n_each: usize) -> (Vec<f64>, Vec<f64>) {
    let centres = [(2.0_f64, 3.0_f64), (-1.0, -1.0), (4.0, -2.0)];
    let mut x = Vec::with_capacity(n_each * 3);
    let mut y = Vec::with_capacity(n_each * 3);
    for (cx, cy) in centres {
        for i in 0..n_each {
            let t = i as f64 / n_each as f64 * std::f64::consts::TAU;
            let r = 0.4 + 0.3 * (i as f64 * 0.17).sin().abs();
            x.push(cx + r * t.cos());
            y.push(cy + r * t.sin());
        }
    }
    (x, y)
}

#[test]
fn test_hexbin_basic() {
    let (x, y) = make_clusters(60);
    let plot = HexbinPlot::new().with_data(x, y);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Hexbin Basic")
        .with_x_label("X")
        .with_y_label("Y");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_basic.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "should contain hex path elements");
    assert!(svg.contains("<svg"), "should be valid SVG");
}

#[test]
fn test_hexbin_log_color() {
    let (x, y) = make_clusters(80);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_log_color(true);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Log Color");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_log_color.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "should render hexes");
}

#[test]
fn test_hexbin_min_count() {
    // Dense cluster data ensures many bins exceed the threshold.
    let (x, y) = make_clusters(150);
    let plot_low  = HexbinPlot::new().with_data(x.clone(), y.clone()).with_n_bins(10).with_min_count(1);
    let plot_high = HexbinPlot::new().with_data(x, y).with_n_bins(10).with_min_count(8);

    let plots_low  = vec![Plot::Hexbin(plot_low)];
    let plots_high = vec![Plot::Hexbin(plot_high)];
    let layout_low  = Layout::auto_from_plots(&plots_low).with_title("Hexbin min_count=1");
    let layout_high = Layout::auto_from_plots(&plots_high).with_title("Hexbin min_count=8");

    let svg_low  = render_svg(plots_low,  layout_low);
    let svg_high = render_svg(plots_high, layout_high);
    std::fs::write("test_outputs/hexbin_min_count_1.svg",  &svg_low).unwrap();
    std::fs::write("test_outputs/hexbin_min_count_8.svg", &svg_high).unwrap();

    let count_low  = svg_low.matches("<path").count();
    let count_high = svg_high.matches("<path").count();
    assert!(count_low > 0, "min_count=1 should render some hexes");
    assert!(count_low >= count_high, "higher min_count should produce fewer hexes");
}

#[test]
fn test_hexbin_flat_top() {
    let (x, y) = make_clusters(50);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_flat_top(true);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Flat-Top");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_flat_top.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "flat-top hexes should render");
}

#[test]
fn test_hexbin_inferno_colormap() {
    let (x, y) = make_clusters(60);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_color_map(ColorMap::Inferno);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Inferno");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_inferno.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "inferno colormap hexes should render");
}

#[test]
fn test_hexbin_with_stroke() {
    let (x, y) = make_clusters(60);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_stroke("#333333")
        .with_stroke_width(1.0);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Stroke");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_stroke.svg", &svg).unwrap();
    assert!(svg.contains("#333333"), "stroke color should appear in SVG");
}

#[test]
fn test_hexbin_normalize() {
    let (x, y) = make_clusters(60);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_normalize(true)
        .with_colorbar_label("Density");
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Normalized");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_normalize.svg", &svg).unwrap();
    assert!(svg.contains("Density"), "colorbar label 'Density' should appear");
}

#[test]
fn test_hexbin_z_mean() {
    let (x, y) = make_clusters(60);
    let z: Vec<f64> = x.iter().zip(y.iter()).map(|(xi, yi)| xi + yi).collect();
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_z(z, ZReduce::Mean);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Z Mean");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_z_mean.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "z-mean hexes should render");
    assert!(svg.contains("Mean"), "colorbar should show 'Mean' label");
}

#[test]
fn test_hexbin_z_median() {
    let (x, y) = make_clusters(60);
    let z: Vec<f64> = x.iter().map(|xi| xi.abs()).collect();
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_z(z, ZReduce::Median);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Z Median");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_z_median.svg", &svg).unwrap();
    assert!(svg.contains("Median"), "colorbar label should be 'Median'");
}

#[test]
fn test_hexbin_z_sum() {
    let (x, y) = make_clusters(60);
    let z: Vec<f64> = y.iter().map(|yi| yi.abs()).collect();
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_z(z, ZReduce::Sum);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Z Sum");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_z_sum.svg", &svg).unwrap();
    assert!(svg.contains("Sum"), "colorbar label should be 'Sum'");
}

#[test]
fn test_hexbin_color_range() {
    let (x, y) = make_clusters(80);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_color_range(2.0, 8.0);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Color Range");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_color_range.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "hexes should render with clamped color scale");
}

#[test]
fn test_hexbin_into_plot() {
    let (x, y) = make_points(50);
    let p: Plot = HexbinPlot::new().with_data(x, y).into();
    assert!(matches!(p, Plot::Hexbin(_)), "From<HexbinPlot> should produce Plot::Hexbin");
}

#[test]
fn test_hexbin_large() {
    let (x, y) = make_points(1000);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_n_bins(30);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Hexbin Large (1000 pts)")
        .with_x_label("X")
        .with_y_label("Y");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_large.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "large dataset should produce valid SVG");
    assert!(svg.contains("<path"), "should have hex paths");
}

#[test]
fn test_hexbin_n_bins_coarse() {
    let (x, y) = make_clusters(80);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_n_bins(10);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin n_bins=10");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_n_bins_10.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "coarse bins should render");
}

#[test]
fn test_hexbin_n_bins_fine() {
    let (x, y) = make_clusters(80);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_n_bins(40);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin n_bins=40");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_n_bins_40.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "fine bins should render");
}

#[test]
fn test_hexbin_x_range() {
    let (x, y) = make_clusters(80);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_x_range(-0.5, 3.0);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin x_range clipped");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_x_range.svg", &svg).unwrap();
    assert!(svg.contains("<svg"), "x_range clip should still produce valid SVG");
}

#[test]
fn test_hexbin_no_colorbar() {
    let (x, y) = make_clusters(60);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_colorbar(false);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin No Colorbar");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_no_colorbar.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "hexes should still render");
    assert!(!svg.contains("Count"), "colorbar label should be absent");
}

#[test]
fn test_hexbin_z_min_max() {
    let (x, y) = make_clusters(60);
    let z: Vec<f64> = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).collect();

    let plot_min = HexbinPlot::new()
        .with_data(x.clone(), y.clone())
        .with_z(z.clone(), ZReduce::Min);
    let plot_max = HexbinPlot::new()
        .with_data(x, y)
        .with_z(z, ZReduce::Max);

    let plots_min = vec![Plot::Hexbin(plot_min)];
    let plots_max = vec![Plot::Hexbin(plot_max)];
    let layout_min = Layout::auto_from_plots(&plots_min).with_title("Hexbin Z Min");
    let layout_max = Layout::auto_from_plots(&plots_max).with_title("Hexbin Z Max");

    let svg_min = render_svg(plots_min, layout_min);
    let svg_max = render_svg(plots_max, layout_max);
    std::fs::write("test_outputs/hexbin_z_min.svg", &svg_min).unwrap();
    std::fs::write("test_outputs/hexbin_z_max.svg", &svg_max).unwrap();
    assert!(svg_min.contains("Min"), "Min label");
    assert!(svg_max.contains("Max"), "Max label");
}

#[test]
fn test_hexbin_grayscale_colormap() {
    let (x, y) = make_clusters(60);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_color_map(ColorMap::Grayscale);
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Grayscale");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_grayscale.svg", &svg).unwrap();
    assert!(svg.contains("<path"), "grayscale hexes should render");
}

#[test]
fn test_hexbin_custom_colorbar_label() {
    let (x, y) = make_clusters(60);
    let plot = HexbinPlot::new()
        .with_data(x, y)
        .with_colorbar_label("My Custom Label");
    let plots = vec![Plot::Hexbin(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hexbin Custom Label");
    let svg = render_svg(plots, layout);
    std::fs::write("test_outputs/hexbin_custom_label.svg", &svg).unwrap();
    assert!(svg.contains("My Custom Label"), "custom colorbar label should appear");
}
