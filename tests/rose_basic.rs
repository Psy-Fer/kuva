use kuva::backend::svg::SvgBackend;
use kuva::plot::rose::{compass_labels_for_n, RoseEncoding, RosePlot};
use kuva::render::render::render_rose;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render(rp: RosePlot, title: &str) -> String {
    let plots = vec![Plot::Rose(rp)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn write(path: &str, svg: &str) {
    if std::env::var_os("CI").is_some() {
        return;
    }
    std::fs::write(path, svg).unwrap();
}

#[test]
fn test_rose_basic() {
    let rp = RosePlot::new()
        .with_slice("Jan", 30.0)
        .with_slice("Feb", 20.0)
        .with_slice("Mar", 45.0)
        .with_slice("Apr", 38.0)
        .with_slice("May", 50.0)
        .with_slice("Jun", 42.0);
    let svg = render(rp, "Basic Rose");
    write("test_outputs/rose_basic.svg", &svg);
    assert!(svg.contains("<path"), "should have path elements");
}

#[test]
fn test_rose_empty() {
    let rp = RosePlot::new();
    let svg = render(rp, "Empty Rose");
    write("test_outputs/rose_empty.svg", &svg);
    assert!(svg.contains("<svg"), "should produce valid SVG");
}

#[test]
fn test_rose_single_slice() {
    let rp = RosePlot::new().with_slice("Only", 100.0);
    let svg = render(rp, "Single Slice");
    write("test_outputs/rose_single_slice.svg", &svg);
    assert!(svg.contains("<path"), "should have a wedge path");
}

#[test]
fn test_rose_area_encoding() {
    let rp = RosePlot::new()
        .with_encoding(RoseEncoding::Area)
        .with_slice("A", 10.0)
        .with_slice("B", 20.0)
        .with_slice("C", 40.0);
    let svg = render(rp, "Area Encoding");
    write("test_outputs/rose_area_encoding.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_radius_encoding() {
    let rp = RosePlot::new()
        .with_encoding(RoseEncoding::Radius)
        .with_slice("A", 10.0)
        .with_slice("B", 30.0)
        .with_slice("C", 60.0);
    let svg = render(rp, "Radius Encoding");
    write("test_outputs/rose_radius_encoding.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_stacked() {
    let rp = RosePlot::new()
        .with_x_labels(["Jan", "Feb", "Mar", "Apr", "May", "Jun"])
        .with_stack("Cats A", vec![10.0_f64, 15.0, 20.0, 25.0, 18.0, 12.0])
        .with_stack("Cats B", vec![5.0_f64, 8.0, 12.0, 10.0, 14.0, 9.0])
        .with_stack("Cats C", vec![2.0_f64, 4.0, 6.0, 8.0, 5.0, 3.0]);
    let svg = render(rp, "Stacked Rose");
    write("test_outputs/rose_stacked.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_grouped() {
    let rp = RosePlot::new()
        .with_x_labels(["Q1", "Q2", "Q3", "Q4"])
        .with_group("Series A", vec![20.0_f64, 35.0, 25.0, 40.0])
        .with_group("Series B", vec![15.0_f64, 22.0, 30.0, 28.0]);
    let svg = render(rp, "Grouped Rose");
    write("test_outputs/rose_grouped.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_inner_radius() {
    let rp = RosePlot::new()
        .with_inner_radius(0.3)
        .with_slice("A", 40.0)
        .with_slice("B", 60.0)
        .with_slice("C", 30.0)
        .with_slice("D", 50.0);
    let svg = render(rp, "Donut Rose");
    write("test_outputs/rose_inner_radius.svg", &svg);
    assert!(svg.contains("<path"), "should have annular wedge paths");
}

#[test]
fn test_rose_bearing_data() {
    let bearings: Vec<f64> = (0..24).map(|i| i as f64 * 15.0).collect();
    let rp = RosePlot::new().with_bearing_data(bearings, 12);
    let svg = render(rp, "Bearing Rose");
    write("test_outputs/rose_bearing_data.svg", &svg);
    assert!(
        svg.contains("<path"),
        "should have wedge paths from bearing data"
    );
}

#[test]
fn test_rose_compass_labels() {
    let rp = RosePlot::new()
        .with_slice("0", 10.0)
        .with_slice("1", 20.0)
        .with_slice("2", 15.0)
        .with_slice("3", 25.0)
        .with_slice("4", 18.0)
        .with_slice("5", 12.0)
        .with_slice("6", 30.0)
        .with_slice("7", 22.0)
        .with_compass_labels();
    let svg = render(rp, "Compass Rose");
    write("test_outputs/rose_compass_labels.svg", &svg);
    assert!(svg.contains("N"), "should contain N compass label");
}

#[test]
fn test_rose_no_grid() {
    let rp = RosePlot::new()
        .with_grid(false)
        .with_slice("A", 30.0)
        .with_slice("B", 50.0)
        .with_slice("C", 20.0);
    let svg = render(rp, "No Grid Rose");
    write("test_outputs/rose_no_grid.svg", &svg);
    assert!(svg.contains("<svg"), "should be valid SVG");
}

#[test]
fn test_rose_grid_lines() {
    let rp = RosePlot::new()
        .with_grid_lines(6)
        .with_slice("A", 10.0)
        .with_slice("B", 30.0)
        .with_slice("C", 20.0);
    let svg = render(rp, "6 Grid Lines");
    write("test_outputs/rose_grid_lines.svg", &svg);
    assert!(svg.contains("<path"), "should have grid ring paths");
}

#[test]
fn test_rose_no_labels() {
    let rp = RosePlot::new()
        .with_show_labels(false)
        .with_slice("A", 40.0)
        .with_slice("B", 60.0);
    let svg = render(rp, "No Labels Rose");
    write("test_outputs/rose_no_labels.svg", &svg);
    assert!(svg.contains("<svg"), "should produce valid SVG");
}

#[test]
fn test_rose_show_values() {
    let rp = RosePlot::new()
        .with_show_values(true)
        .with_slice("Jan", 120.0)
        .with_slice("Feb", 85.0)
        .with_slice("Mar", 200.0);
    let svg = render(rp, "Values Rose");
    write("test_outputs/rose_show_values.svg", &svg);
    assert!(
        svg.contains("<text"),
        "should have text elements for values"
    );
}

#[test]
fn test_rose_legend() {
    let rp = RosePlot::new()
        .with_legend("Causes")
        .with_x_labels(["Jan", "Feb", "Mar"])
        .with_stack("Preventable", vec![30.0_f64, 25.0, 40.0])
        .with_stack("Other", vec![10.0_f64, 15.0, 20.0]);
    let svg = render(rp, "Legend Rose");
    write("test_outputs/rose_legend.svg", &svg);
    assert!(svg.contains("<svg"), "should produce valid SVG");
}

#[test]
fn test_rose_start_angle() {
    let rp = RosePlot::new()
        .with_start_angle(45.0)
        .with_slice("A", 30.0)
        .with_slice("B", 50.0)
        .with_slice("C", 20.0);
    let svg = render(rp, "Start Angle 45");
    write("test_outputs/rose_start_angle.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_counterclockwise() {
    let rp = RosePlot::new()
        .with_clockwise(false)
        .with_slice("N", 10.0)
        .with_slice("NE", 20.0)
        .with_slice("E", 30.0)
        .with_slice("SE", 15.0);
    let svg = render(rp, "Counterclockwise Rose");
    write("test_outputs/rose_counterclockwise.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_gap() {
    let rp = RosePlot::new()
        .with_gap(5.0)
        .with_slice("A", 40.0)
        .with_slice("B", 30.0)
        .with_slice("C", 50.0)
        .with_slice("D", 25.0);
    let svg = render(rp, "Wide Gap Rose");
    write("test_outputs/rose_gap.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_slices_api() {
    let data = vec![
        ("Mon", 10.0),
        ("Tue", 20.0),
        ("Wed", 15.0),
        ("Thu", 25.0),
        ("Fri", 30.0),
        ("Sat", 18.0),
        ("Sun", 8.0),
    ];
    let rp = RosePlot::new().with_slices(data.into_iter().map(|(l, v)| (l.to_string(), v)));
    let svg = render(rp, "Slices API Rose");
    write("test_outputs/rose_slices_api.svg", &svg);
    assert!(svg.contains("<path"), "should have wedge paths");
}

#[test]
fn test_rose_into_plot() {
    let rp = RosePlot::new().with_slice("A", 10.0).with_slice("B", 20.0);
    let p: Plot = rp.into();
    assert!(matches!(p, Plot::Rose(_)), "should convert into Plot::Rose");
}

#[test]
fn test_rose_large() {
    let slices: Vec<(String, f64)> = (0..24)
        .map(|i| (format!("{}", i * 15), (i as f64 * 3.0 + 5.0)))
        .collect();
    let rp = RosePlot::new().with_slices(slices);
    let svg = render(rp, "24-Slice Rose");
    write("test_outputs/rose_large.svg", &svg);
    assert!(svg.contains("<path"), "should have many wedge paths");
}

#[test]
fn test_rose_equal_values() {
    let rp = RosePlot::new()
        .with_slice("A", 25.0)
        .with_slice("B", 25.0)
        .with_slice("C", 25.0)
        .with_slice("D", 25.0);
    let svg = render(rp, "Equal Values Rose");
    write("test_outputs/rose_equal_values.svg", &svg);
    assert!(
        svg.contains("<path"),
        "should have wedge paths for equal values"
    );
}

#[test]
fn test_rose_nightingale() {
    // Classic Nightingale diagram: 12 months × 3 stacked causes
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let preventable = vec![
        12.0, 11.0, 14.0, 10.0, 9.0, 7.0, 6.0, 5.0, 8.0, 10.0, 13.0, 15.0,
    ];
    let wounds = vec![3.0, 4.0, 2.0, 3.0, 2.0, 2.0, 1.0, 1.0, 2.0, 3.0, 3.0, 4.0];
    let other = vec![2.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0];

    let rp = RosePlot::new()
        .with_x_labels(months.iter().copied())
        .with_stack("Preventable diseases", preventable)
        .with_stack("Wounds", wounds)
        .with_stack("Other", other)
        .with_legend("Cause of death")
        .with_start_angle(0.0);
    let svg = render(rp, "Nightingale Polar Area Diagram");
    write("test_outputs/rose_nightingale.svg", &svg);
    assert!(svg.contains("<path"), "should have stacked wedge paths");
    assert!(
        svg.contains("Jan") || svg.contains("<text"),
        "should have month labels or text"
    );
}

#[test]
fn test_rose_render_rose_fn() {
    let rp = RosePlot::new()
        .with_slice("X", 50.0)
        .with_slice("Y", 30.0)
        .with_slice("Z", 70.0);
    let plots_tmp = vec![Plot::Rose(RosePlot::new())];
    let layout = Layout::auto_from_plots(&plots_tmp).with_title("render_rose helper");
    let scene = render_rose(rp, layout);
    let svg = SvgBackend.render_scene(&scene);
    write("test_outputs/rose_render_fn.svg", &svg);
    assert!(
        svg.contains("<path"),
        "render_rose should produce wedge paths"
    );
}

#[test]
fn test_rose_compass_labels_for_n() {
    let labels_4 = compass_labels_for_n(4);
    assert_eq!(labels_4, vec!["N", "E", "S", "W"]);

    let labels_8 = compass_labels_for_n(8);
    assert_eq!(labels_8, vec!["N", "NE", "E", "SE", "S", "SW", "W", "NW"]);

    let labels_16 = compass_labels_for_n(16);
    assert_eq!(labels_16[0], "N");
    assert_eq!(labels_16.len(), 16);

    // Non-divisor → degree strings
    let labels_5 = compass_labels_for_n(5);
    assert!(labels_5[0].ends_with('°') || labels_5[0] == "0°");
}
