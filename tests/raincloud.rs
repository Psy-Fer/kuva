use kuva::plot::RaincloudPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

fn make_values(seed: u64, n: usize, shift: f64) -> Vec<f64> {
    // Simple deterministic pseudo-random values for tests — no rand dependency.
    let mut state = seed ^ 0x9e3779b97f4a7c15u64;
    (0..n).map(|_| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let u = (state >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        // Box-Muller (approximate): 2*u - 1 as a simple uniform offset
        (u - 0.5) * 4.0 + shift
    }).collect()
}

// ── basic rendering ──────────────────────────────────────────────────────────

#[test]
fn test_raincloud_basic() {
    let plot = RaincloudPlot::new()
        .with_group("Control", make_values(1, 30, 5.0))
        .with_group("Treated", make_values(2, 30, 7.0));

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Basic")
        .with_x_label("Group")
        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"), "SVG should start with svg element");
    // Cloud is a filled path
    assert!(svg.contains("<path"), "SVG should contain path elements (clouds)");
    // Box is rendered as a rect
    assert!(svg.contains("<rect"), "SVG should contain rect elements (boxes)");
    // Rain is rendered as circles
    assert!(svg.contains("<circle"), "SVG should contain circle elements (rain)");
    // Group labels should appear on x-axis
    assert!(svg.contains("Control"), "SVG should contain group label 'Control'");
    assert!(svg.contains("Treated"), "SVG should contain group label 'Treated'");
}

#[test]
fn test_raincloud_single_group() {
    let plot = RaincloudPlot::new()
        .with_group("Only", make_values(3, 20, 3.0));

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Single Group");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_single.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Only"), "SVG should contain the single group label");
}

// ── element toggling ─────────────────────────────────────────────────────────

#[test]
fn test_raincloud_no_cloud() {
    let plot = RaincloudPlot::new()
        .with_group("A", make_values(4, 25, 5.0))
        .with_group("B", make_values(5, 25, 7.0))
        .with_cloud(false);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud No Cloud");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_no_cloud.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Without cloud, no path elements should represent the KDE shape
    // (there may still be axis paths, so just ensure we have rect/circle)
    assert!(svg.contains("<rect"), "should still have box rects");
    assert!(svg.contains("<circle"), "should still have rain circles");
}

#[test]
fn test_raincloud_no_box() {
    let plot = RaincloudPlot::new()
        .with_group("A", make_values(6, 25, 5.0))
        .with_group("B", make_values(7, 25, 7.0))
        .with_box(false);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud No Box");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_no_box.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Background rect is still present; just verify it renders OK
    assert!(svg.contains("<circle"), "should have rain points");
    assert!(svg.contains("<path"), "should have cloud paths");
}

#[test]
fn test_raincloud_no_rain() {
    let plot = RaincloudPlot::new()
        .with_group("A", make_values(8, 25, 5.0))
        .with_group("B", make_values(9, 25, 7.0))
        .with_rain(false);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud No Rain");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_no_rain.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // No circle elements when rain is disabled
    assert!(!svg.contains("<circle"), "should have no rain circles");
    assert!(svg.contains("<rect"), "should still have box rects");
    assert!(svg.contains("<path"), "should still have cloud paths");
}

// ── appearance options ───────────────────────────────────────────────────────

#[test]
fn test_raincloud_group_colors() {
    let plot = RaincloudPlot::new()
        .with_group("A", make_values(10, 20, 4.0))
        .with_group("B", make_values(11, 20, 6.0))
        .with_group("C", make_values(12, 20, 8.0))
        .with_group_colors(["tomato", "steelblue", "seagreen"]);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Group Colors");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_group_colors.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // All three custom colors (or their hex equivalents) should appear
    // "seagreen" is not in color table so stays as-is; others may be resolved to hex
    assert!(
        svg.contains("seagreen") || svg.contains("#2e8b57"),
        "SVG should contain seagreen color"
    );
}

#[test]
fn test_raincloud_flipped() {
    let plot = RaincloudPlot::new()
        .with_group("A", make_values(13, 30, 5.0))
        .with_group("B", make_values(14, 30, 7.0))
        .with_flip(true);

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Flipped");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_flipped.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"), "cloud path present when flipped");
    assert!(svg.contains("<circle"), "rain circles present when flipped");
}

#[test]
fn test_raincloud_legend() {
    let plot = RaincloudPlot::new()
        .with_group("Control", make_values(15, 20, 4.0))
        .with_group("Treated", make_values(16, 20, 6.5))
        .with_legend("Condition");

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud With Legend");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Legend entries are per-group: group labels appear as legend text
    assert!(svg.contains("Control"), "SVG should contain legend entry 'Control'");
    assert!(svg.contains("Treated"), "SVG should contain legend entry 'Treated'");
}

#[test]
fn test_raincloud_five_groups() {
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri"];
    let mut plot = RaincloudPlot::new();
    for (i, label) in labels.iter().enumerate() {
        plot = plot.with_group(*label, make_values(i as u64 + 100, 40, i as f64 * 1.5 + 3.0));
    }

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Five Groups")
        .with_x_label("Day")
        .with_y_label("Score");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_five_groups.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    for label in &labels {
        assert!(svg.contains(label), "SVG should contain group label '{label}'");
    }
    // Five groups → many circles and paths
    let circle_count = svg.matches("<circle").count();
    assert!(circle_count >= 5, "expected at least 5 rain circles, got {circle_count}");
}

#[test]
fn test_raincloud_bandwidth_scale() {
    // Write two variants side-by-side: default bandwidth and a tighter one.
    // Both should render without panicking; the tight one should have more
    // KDE path points in the SVG due to finer structure.
    let groups = [("A", 300u64, 5.0f64), ("B", 301, 7.5), ("C", 302, 10.0)];

    let mut default_plot = RaincloudPlot::new();
    let mut tight_plot   = RaincloudPlot::new().with_bandwidth_scale(0.4);
    for (label, seed, shift) in &groups {
        let vals = make_values(*seed, 60, *shift);
        default_plot = default_plot.with_group(*label, vals.clone());
        tight_plot   = tight_plot.with_group(*label, vals);
    }

    for (plot, name) in [(default_plot, "default"), (tight_plot, "tight")] {
        let plots = vec![Plot::Raincloud(plot)];
        let layout = Layout::auto_from_plots(&plots)
            .with_title(format!("Raincloud bandwidth_scale={name}"))
            .with_x_label("Group")
            .with_y_label("Value");
        let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
        std::fs::write(format!("test_outputs/raincloud_bw_{name}.svg"), svg.clone()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"), "cloud path should be present ({name})");
    }
}

#[test]
fn test_raincloud_large_dataset() {
    // 1000 points per group — stress-tests KDE path building and rendering performance.
    let plot = RaincloudPlot::new()
        .with_group("Control", make_values(200, 1000, 5.0))
        .with_group("Low dose", make_values(201, 1000, 6.5))
        .with_group("High dose", make_values(202, 1000, 8.5));

    let plots = vec![Plot::Raincloud(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Raincloud Large Dataset (1000 pts/group)")
        .with_x_label("Treatment")
        .with_y_label("Response");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/raincloud_large.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"), "cloud paths present");
    assert!(svg.contains("<rect"), "box rects present");
    let circle_count = svg.matches("<circle").count();
    assert!(circle_count >= 3000, "expected 3000+ rain circles for 3×1000 pts, got {circle_count}");
}
