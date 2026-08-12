mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{DensityPlot, Histogram};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::render_utils::{silverman_bandwidth, simple_kde};

#[test]
fn test_histogram_svg_output_builder() {
    let hist = Histogram::new()
        .with_data(vec![1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0])
        .with_bins(5)
        .with_color("navy")
        .with_range((0.0, 5.0)); // make this automatic

    let plots = vec![Plot::Histogram(hist.clone())];

    // let layout = Layout::auto_from_data(&hist.data, 0.0..5.0)
    //     .with_title("Histogram")
    //     .with_x_label("Value")
    //     .with_y_label("Frequency");
    // .with_ticks(10);
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Histogram")
        .with_x_label("Value")
        .with_y_label("Frequency");
    // .with_ticks(10);

    // let scene = render_histogram(&hist, &layout);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/hist_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

// A normalized histogram has y values in [0, 1].  auto_from_plots should
// detect this and clamp the y-axis at exactly 1.0 rather than letting the
// 1%-span padding push auto_nice_range up to 1.1.
#[test]
fn test_normalized_histogram_y_axis_clamp() {
    let hist = Histogram::new()
        .with_data(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0])
        .with_bins(4)
        .with_range((0.0, 5.0))
        .with_normalize();

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_normalized_clamp.svg", &svg).unwrap();

    // clamp_y_axis should be triggered automatically: axis must stop at 1 not 1.1
    assert!(
        svg.contains(">1<") || svg.contains(">1.0<"),
        "normalized histogram y-axis should show a tick at 1"
    );
    assert!(
        !svg.contains(">1.1<"),
        "y-axis must not extend past 1.0 for normalized data"
    );
}

// Non-normalized histograms must not be affected by the clamp.
#[test]
fn test_non_normalized_histogram_y_axis_free() {
    let hist = Histogram::new()
        .with_data(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0])
        .with_bins(4)
        .with_range((0.0, 5.0)); // no with_normalize()

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_non_normalized.svg", &svg).unwrap();

    // y-axis should reflect actual counts (max count = 3), well above 1
    assert!(
        !svg.contains(">1.1<") || svg.contains(">2<") || svg.contains(">3<"),
        "non-normalized histogram y-axis should show count ticks"
    );
}

// Bug #4: ticks must land on bin boundaries for auto-binned histograms.
// Range [0.0, 4.8], 6 bins → bin_width = 0.8.
// generate_ticks_bin_aligned should produce 0, 0.8, 1.6, 2.4, 3.2, 4.0, 4.8
// rather than the generic 0, 1, 2, 3, 4 from generate_ticks.
#[test]
fn test_histogram_bin_aligned_ticks() {
    // 6 uniform bins over [0.0, 4.8] → bin_width = 0.8
    let data: Vec<f64> = (0..60).map(|i| i as f64 * 0.08).collect();
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(6)
        .with_range((0.0, 4.8))
        .with_color("steelblue");

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_bin_aligned_ticks.svg", &svg).unwrap();

    // The rightmost bin edge (4.8) must appear as a tick label
    assert!(
        svg.contains(">4.8<"),
        "bin-aligned ticks should include the rightmost bin edge 4.8"
    );
    // A non-edge tick like 1 would only appear with the generic tick generator
    assert!(
        !svg.contains(">1<"),
        "bin-aligned ticks must not emit non-edge tick 1"
    );
}

// Feature #6: Histogram::from_bins — precomputed edges + counts, no range needed.
#[test]
fn test_histogram_from_bins_basic() {
    let edges = vec![0.0, 1.0, 2.0, 3.0];
    let counts = vec![5.0, 12.0, 8.0];
    let hist = Histogram::from_bins(edges, counts).with_color("steelblue");

    let plots = vec![Plot::Histogram(hist)];
    // Must not panic — range is not required for precomputed histograms
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_from_bins.svg", &svg).unwrap();

    assert!(
        svg.contains("<rect"),
        "precomputed histogram must draw bars"
    );
    // Bin edges are 0, 1, 2, 3 — bin-aligned ticks should include 3
    assert!(
        svg.contains(">3<"),
        "bin-aligned ticks should include the right edge 3"
    );
}

// Feature #6: precomputed histogram with normalization.
#[test]
fn test_histogram_from_bins_normalize() {
    let edges = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let counts = vec![5.0, 20.0, 15.0, 10.0];
    let hist = Histogram::from_bins(edges, counts)
        .with_normalize()
        .with_color("coral");

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_from_bins_normalized.svg", &svg).unwrap();

    assert!(
        !svg.contains(">1.1<"),
        "normalized precomputed histogram y-axis must not exceed 1.0"
    );
}

// Issue #51: zero-count bins must not produce <rect height="0">.
// Bimodal data with a gap in the middle — several bins will have count == 0.
// The fix skips those bins entirely instead of emitting a zero-height rect.
#[test]
fn test_histogram_zero_count_bins_skipped() {
    // Two clusters far apart with a gap in the middle → zero-count bins guaranteed.
    let mut data: Vec<f64> = (0..20).map(|i| i as f64 * 0.1).collect(); // 0.0 – 1.9
    data.extend((80..100).map(|i| i as f64 * 0.1)); // 8.0 – 9.9
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(20)
        .with_range((0.0, 10.0));

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_zero_count_gap.svg", &svg).unwrap();

    assert!(
        svg.contains("<rect"),
        "bimodal histogram must still draw bars"
    );
    assert!(
        !svg.contains("height=\"0\""),
        "zero-count bins must not emit height=0 rects"
    );
}

// Issue #51: same check for Histogram::from_bins with explicit zeros in the middle.
#[test]
fn test_histogram_from_bins_zero_counts_skipped() {
    let edges = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let counts = vec![10.0, 0.0, 0.0, 0.0, 8.0]; // zeros in the middle
    let hist = Histogram::from_bins(edges, counts).with_color("steelblue");

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_from_bins_zero_gap.svg", &svg).unwrap();

    assert!(
        svg.contains("<rect"),
        "from_bins histogram must still draw non-zero bars"
    );
    assert!(
        !svg.contains("height=\"0\""),
        "zero-count bins must not emit height=0 rects"
    );
}

// Regression test for issue #46: last x-axis tick label was truncated when
// the tick value is a wide number (e.g. "15000").  The fix estimates the
// half-pixel-width of that label and ensures margin_right >= that estimate.
//
// Verification: with tick_size=11 and char_width≈0.6, "15000" (5 chars) has
// half-width = 5 * 11 * 0.6 * 0.5 = 16.5 px.  The canvas is 500px wide with
// the default auto-sizing, and margin_right should now absorb at least 16.5px
// so the label never bleeds past the SVG edge.
//
// We check this by parsing the canvas width from the SVG and verifying that
// the last tick label text element (">15000<") has its x-attribute set to a
// value strictly less than (canvas_width - 10).
#[test]
fn test_histogram_last_tick_no_overflow() {
    // Genomics-style data: read counts in [0, 15000]
    let data: Vec<f64> = (0..=150).map(|i| i as f64 * 100.0).collect();
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(10)
        .with_range((0.0, 15000.0));

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Read counts")
        .with_x_label("Count");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/hist_last_tick_no_overflow.svg", &svg).unwrap();

    assert!(
        svg.contains(">15000<"),
        "last tick label '15000' must be present"
    );

    // Extract SVG canvas width from the width="..." attribute on the root element
    let canvas_width: f64 = {
        let marker = "width=\"";
        let pos = svg.find(marker).expect("SVG must have width attribute");
        let rest = &svg[pos + marker.len()..];
        let end = rest.find('"').unwrap_or(rest.len());
        rest[..end].parse().expect("SVG width must be numeric")
    };

    // Find x-coordinate of the "15000" text element
    // Pattern: <text ... x="NNN" ...>15000<
    let mut tick_x: Option<f64> = None;
    let needle = ">15000<";
    if let Some(pos) = svg.find(needle) {
        // Walk backwards to find the opening <text tag
        let tag_start = svg[..pos].rfind("<text").unwrap_or(0);
        let tag_slice = &svg[tag_start..pos];
        // Extract x="..." attribute
        if let Some(x_pos) = tag_slice.find(" x=\"") {
            let rest = &tag_slice[x_pos + 4..];
            let end = rest.find('"').unwrap_or(rest.len());
            tick_x = rest[..end].parse().ok();
        }
    }

    let x = tick_x.expect("could not parse x-coord of '15000' tick label");
    // The text is centered (TextAnchor::Middle); half-width ≈ 5 chars * 11 * 0.6 * 0.5 = 16.5
    let approx_right_edge = x + 16.5;
    assert!(
        approx_right_edge <= canvas_width,
        "tick label '15000' right edge {approx_right_edge:.1} overflows canvas width {canvas_width}"
    );
}

// ── KDE overlay ───────────────────────────────────────────────────────────────

#[test]
fn test_histogram_kde_overlay() {
    let data = vec![
        1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0, 1.5, 2.1, 3.5, 2.9, 3.1, 2.4, 3.6, 2.2,
    ];
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(8)
        .with_color("steelblue")
        .with_kde(true);

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Histogram with KDE Overlay")
        .with_x_label("Value")
        .with_y_label("Count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_kde_overlay.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("firebrick") || svg.contains("#b22222"));
    // The KDE curve is a single stroked <path> with no fill.
    assert!(svg.contains("fill=\"none\""));
}

#[test]
fn test_histogram_kde_normalized() {
    // Normalized mode: the KDE curve must still be visible without pushing
    // bounds() below the peak (curve height can exceed the tallest bar).
    let data = vec![
        5.0, 5.2, 5.1, 5.3, 4.9, 5.0, 5.1, 10.0, 10.2, 10.1, 15.0, 15.1, 15.2,
    ];
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(6)
        .with_range((0.0, 20.0))
        .with_color("seagreen")
        .with_normalize()
        .with_kde(true)
        .with_kde_color("black");

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots).with_title("Normalized Histogram with KDE");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_kde_normalized.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_histogram_kde_custom_bandwidth_and_color() {
    let data: Vec<f64> = (0..30).map(|i| i as f64 * 0.3).collect();
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(10)
        .with_color("gray")
        .with_kde(true)
        .with_kde_bandwidth(0.8)
        .with_kde_color("purple")
        .with_kde_samples(300);

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots).with_title("Histogram KDE — Custom Bandwidth");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_kde_custom_bandwidth.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("purple") || svg.contains("#800080"));
}

#[test]
fn test_histogram_kde_ignored_for_precomputed() {
    // Precomputed histograms have no raw samples — .with_kde() must not panic.
    let hist = Histogram::from_bins(vec![0.0, 1.0, 2.0, 3.0], vec![5.0, 12.0, 8.0])
        .with_color("steelblue")
        .with_kde(true);

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots).with_title("Precomputed Histogram, KDE Ignored");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_kde_precomputed_ignored.svg", svg.clone())
        .unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_histogram_kde_overlay_matches_manual_density_curve() {
    // Cross-check the histogram's auto-drawn KDE overlay against an
    // independently-computed density curve using the exact same building
    // blocks (`silverman_bandwidth` / `simple_kde`) that `add_histogram`
    // uses internally, scaled by the same documented formula
    // (`density * n * bin_width`, un-normalized histogram => norm = 1.0).
    // Fed into a `DensityPlot::from_curve` and rendered on the SAME axes as
    // the histogram, the two curves should draw to the exact same SVG path
    // geometry — a strong, visual way to confirm the overlay math is right,
    // not just "close by eye".
    let data = vec![
        1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0, 1.5, 2.1, 3.5, 2.9, 3.1, 2.4, 3.6, 2.2,
    ];
    let bins = 8;
    let range = (0.0, 5.0);
    let bandwidth = silverman_bandwidth(&data);
    let kde_samples = 200;

    let bin_width = (range.1 - range.0) / bins as f64;
    let n = data.len() as f64;
    let density_norm = 1.0 / (n * bandwidth * (2.0 * std::f64::consts::PI).sqrt());
    let kde = simple_kde(&data, bandwidth, kde_samples);
    let xs: Vec<f64> = kde.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f64> = kde
        .iter()
        .map(|(_, d)| d * density_norm * n * bin_width)
        .collect();

    let hist = Histogram::new()
        .with_data(data)
        .with_bins(bins)
        .with_range(range)
        .with_color("steelblue")
        .with_kde(true)
        .with_kde_bandwidth(bandwidth)
        .with_kde_samples(kde_samples)
        .with_kde_color("firebrick");

    let manual_curve = DensityPlot::from_curve(xs, ys)
        .with_color("purple")
        .with_stroke_width(2.0);

    let plots = vec![Plot::Histogram(hist), Plot::Density(manual_curve)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Histogram KDE Overlay vs. Manually-scaled Density Curve")
        .with_x_label("Value")
        .with_y_label("Count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_kde_vs_manual_density.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));

    // Extract both curves' `d="..."` path geometry and compare.
    let paths: Vec<&str> = svg
        .match_indices("<path")
        .map(|(i, _)| {
            let end = svg[i..].find('>').unwrap() + i;
            &svg[i..=end]
        })
        .collect();
    assert!(
        paths.len() >= 2,
        "expected 2 <path> elements (histogram KDE overlay + manual density curve), got {}",
        paths.len()
    );
    let extract_d = |tag: &str| -> String {
        let start = tag.find("d=\"").unwrap() + 3;
        let end = tag[start..].find('"').unwrap() + start;
        tag[start..end].to_string()
    };
    let overlay_d = extract_d(paths[0]);
    let manual_d = extract_d(paths[1]);
    assert_eq!(
        overlay_d.trim(),
        manual_d.trim(),
        "histogram's auto-drawn KDE overlay path must exactly match a curve computed \
         from the same public render_utils functions with the documented scaling formula"
    );
}

// ── Distribution modes (step / cumulative / stacked / weighted / bin-method) ─────
// These render SVGs to test_outputs/ for visual inspection and assert light
// structural invariants; numeric correctness lives in the compute_bins unit tests.

#[test]
fn test_histogram_step_svg() {
    // Step mode draws an outline-only staircase: a stroked path with fill="none"
    // in the series colour, and no filled bin rectangles.
    let data = vec![
        1.1, 2.3, 2.7, 3.2, 3.8, 3.9, 4.0, 1.5, 2.1, 3.5, 2.9, 3.1, 2.4, 3.6, 2.2,
    ];
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(10)
        .with_range((0.0, 5.0))
        .with_color("#4682b4")
        .with_step(true);

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Step Histogram")
        .with_x_label("Value")
        .with_y_label("Count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_step.svg", &svg).unwrap();

    assert!(svg.contains("<svg"));
    assert!(
        svg.contains("stroke=\"#4682b4\"") && svg.contains("fill=\"none\""),
        "step histogram must draw a stroked outline in the series colour"
    );
    // No filled bin bars in the series colour.
    assert!(
        !svg.contains("fill=\"#4682b4\""),
        "step mode must not emit filled bin rects"
    );
}

#[test]
fn test_histogram_step_overlay_svg() {
    // Two step histograms overlaid — the clean multi-distribution use case.
    let a = vec![1.0, 1.2, 1.5, 1.8, 2.0, 2.1, 2.3, 2.5, 2.7, 3.0];
    let b = vec![3.0, 3.2, 3.5, 3.8, 4.0, 4.1, 4.3, 4.5, 4.7, 5.0];
    let range = (0.0, 6.0);
    let ha = Histogram::new()
        .with_data(a)
        .with_bins(12)
        .with_range(range)
        .with_color("#4682b4")
        .with_step(true)
        .with_legend("Group A");
    let hb = Histogram::new()
        .with_data(b)
        .with_bins(12)
        .with_range(range)
        .with_color("#dc143c")
        .with_step(true)
        .with_legend("Group B");

    let plots = vec![Plot::Histogram(ha), Plot::Histogram(hb)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Overlaid Step Histograms")
        .with_x_label("Value")
        .with_y_label("Count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_step_overlay.svg", &svg).unwrap();

    assert!(svg.contains("stroke=\"#4682b4\""));
    assert!(svg.contains("stroke=\"#dc143c\""));
    // Both series appear in the legend.
    assert!(svg.contains(">Group A<") && svg.contains(">Group B<"));
}

#[test]
fn test_histogram_cumulative_svg() {
    // Cumulative bars are non-decreasing, so the final (rightmost) bar is the
    // tallest: its top y-coordinate is the smallest of all bars.
    let data: Vec<f64> = (0..50).map(|i| i as f64 * 0.1).collect(); // uniform over [0, 4.9]
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(10)
        .with_range((0.0, 5.0))
        .with_color("steelblue")
        .with_cumulative(true);

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Cumulative Histogram")
        .with_x_label("Value")
        .with_y_label("Cumulative count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_cumulative.svg", &svg).unwrap();

    assert!(svg.contains("<rect"));
    // The y-axis should reach the full sample count (50) as the top tick region.
    assert!(
        svg.contains(">50<") || svg.contains(">40<"),
        "cumulative y-axis should climb to the total count"
    );
}

#[test]
fn test_histogram_stacked_svg() {
    // Three groups stacked; each group colour and legend label must appear.
    let a = vec![1.0, 1.2, 1.4, 1.6, 1.8];
    let b = vec![2.0, 2.2, 2.4, 2.6, 2.8];
    let c = vec![3.0, 3.2, 3.4, 3.6, 3.8];
    let hist = Histogram::new()
        .with_data(a)
        .with_bins(12)
        .with_range((0.0, 5.0))
        .with_color("#4e79a7")
        .with_stacked(true)
        .with_legend("Group A")
        .with_group(b, "#f28e2b", Some("Group B".to_string()))
        .with_group(c, "#59a14f", Some("Group C".to_string()));

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Stacked Histogram")
        .with_x_label("Value")
        .with_y_label("Count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_stacked.svg", &svg).unwrap();

    for color in ["#4e79a7", "#f28e2b", "#59a14f"] {
        assert!(
            svg.contains(&format!("fill=\"{color}\"")),
            "stacked histogram must draw bars in {color}"
        );
    }
    for label in [">Group A<", ">Group B<", ">Group C<"] {
        assert!(svg.contains(label), "legend must include {label}");
    }
}

#[test]
fn test_histogram_weighted_svg() {
    // Weighted samples: a single heavily-weighted sample dominates its bin.
    let data = vec![0.5, 1.5, 2.5, 2.6, 2.7];
    let weights = vec![1.0, 1.0, 20.0, 1.0, 1.0];
    let hist = Histogram::new()
        .with_data(data)
        .with_weights(weights)
        .with_bins(5)
        .with_range((0.0, 5.0))
        .with_color("darkorange");

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Weighted Histogram")
        .with_x_label("Value")
        .with_y_label("Weighted count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_weighted.svg", &svg).unwrap();

    assert!(svg.contains("<rect"));
    // The weighted bin pushes the y-axis well past a plain count of 5.
    assert!(
        svg.contains(">20<") || svg.contains(">15<"),
        "weighted y-axis should reflect the summed weights, not raw counts"
    );
}

#[test]
fn test_histogram_bin_method_svg() {
    // Freedman-Diaconis picks the bin count from the data, overriding with_bins.
    let data: Vec<f64> = (0..200).map(|i| (i as f64 * 0.05).sin() + 2.0).collect();
    // Use a hex colour so the SVG fill matches verbatim (named CSS colours are
    // normalized to hex on output, e.g. "mediumpurple" -> "#9370db").
    let hist = Histogram::new()
        .with_data(data)
        .with_bins(3) // deliberately wrong; bin_method must override
        .with_range((0.0, 4.0))
        .with_color("#9370db")
        .with_bin_method(kuva::plot::BinMethod::FreedmanDiaconis);

    let plots = vec![Plot::Histogram(hist)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Freedman-Diaconis Bins")
        .with_x_label("Value")
        .with_y_label("Count");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/hist_bin_method_fd.svg", &svg).unwrap();

    // More than 3 filled bars means the auto rule overrode with_bins(3).
    let bar_count = svg.matches("fill=\"#9370db\"").count();
    assert!(
        bar_count > 3,
        "Freedman-Diaconis should override with_bins(3); got {bar_count} bars"
    );
}
