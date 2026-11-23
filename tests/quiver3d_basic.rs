mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::scatter3d::Scatter3DPlot;
use kuva::plot::{ColorMap, Quiver3DPlot, QuiverPivot, View3D};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn render(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn svg_of(plot: Quiver3DPlot) -> String {
    let plots = vec![Plot::Quiver3D(plot)];
    let layout = Layout::auto_from_plots(&plots);
    render(plots, layout)
}

/// `n × n × n` vortex around the z axis with a small updraft.
fn vortex(n: usize) -> Quiver3DPlot {
    Quiver3DPlot::from_function((-1.0, 1.0, n), (-1.0, 1.0, n), (0.0, 2.0, n), |x, y, _| {
        (-y, x, 0.3)
    })
}

/// The `<path …>` elements that are arrow glyphs. Each arrow is one closed
/// seven-point outline (`M` + six `L` + `Z`); the 3D box's back-pane fills are
/// also paths but have four corners, so this filter separates the two.
fn arrow_elements(svg: &str) -> Vec<&str> {
    svg.split("<path")
        .skip(1)
        .filter_map(|chunk| chunk.split("/>").next())
        .filter(|el| {
            el.split("d=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .is_some_and(|d| d.matches(" L ").count() == 6 && d.ends_with('Z'))
        })
        .collect()
}

/// The `d="..."` attribute of every arrow glyph.
fn path_data(svg: &str) -> Vec<&str> {
    arrow_elements(svg)
        .into_iter()
        .filter_map(|el| el.split("d=\"").nth(1).and_then(|s| s.split('"').next()))
        .collect()
}

/// Every numeric token of every arrow `d` attribute, as `(x, y)` pairs.
fn path_points(svg: &str) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    for d in path_data(svg) {
        let nums: Vec<f64> = d
            .split_whitespace()
            .filter_map(|t| t.parse::<f64>().ok())
            .collect();
        for pair in nums.chunks(2) {
            if let [x, y] = pair {
                pts.push((*x, *y));
            }
        }
    }
    pts
}

fn attr_f64(svg: &str, attr: &str) -> f64 {
    svg.split(&format!("{attr}=\""))
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("missing {attr} attribute"))
}

#[test]
fn test_quiver3d_basic() {
    let plot = vortex(3).with_color("steelblue");
    let plots = vec![Plot::Quiver3D(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title("3D Quiver Basic");
    let svg = render(plots, layout);
    common::write_test_output("test_outputs/quiver3d_basic.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("3D Quiver Basic"));
    assert_eq!(
        path_data(&svg).len(),
        27,
        "one <path> glyph per arrow (3×3×3 grid)"
    );
    assert!(
        svg.contains("<line"),
        "SVG should contain line elements for the wireframe box"
    );
}

#[test]
fn test_quiver3d_empty() {
    let svg = svg_of(Quiver3DPlot::new());
    common::write_test_output("test_outputs/quiver3d_empty.svg", svg.clone()).unwrap();
    assert!(svg.contains("<svg"));
    assert!(path_data(&svg).is_empty(), "no arrows, no paths");
}

#[test]
fn test_quiver3d_axis_labels() {
    let svg = svg_of(
        vortex(2)
            .with_x_label("East")
            .with_y_label("North")
            .with_z_label("Altitude"),
    );
    common::write_test_output("test_outputs/quiver3d_labels.svg", svg.clone()).unwrap();
    for label in ["East", "North", "Altitude"] {
        assert!(svg.contains(label), "SVG should contain axis label {label}");
    }
}

#[test]
fn test_quiver3d_legend() {
    let svg = svg_of(vortex(2).with_legend("Wind"));
    common::write_test_output("test_outputs/quiver3d_legend.svg", svg.clone()).unwrap();
    assert!(svg.contains("Wind"), "SVG should contain legend label");
}

#[test]
fn test_quiver3d_colormap_draws_colorbar_and_varied_fills() {
    let plot =
        Quiver3DPlot::from_function((0.0, 3.0, 4), (0.0, 3.0, 4), (0.0, 3.0, 2), |x, y, _| {
            (x, y, 0.0)
        })
        .with_magnitude_colormap(ColorMap::Viridis, "Speed");
    let svg = svg_of(plot);
    common::write_test_output("test_outputs/quiver3d_colormap.svg", svg.clone()).unwrap();
    assert!(svg.contains("Speed"), "colorbar label should be rendered");
    let fills: std::collections::HashSet<&str> = arrow_elements(&svg)
        .into_iter()
        .filter_map(|el| el.split("fill=\"").nth(1).and_then(|s| s.split('"').next()))
        .collect();
    assert!(
        fills.len() >= 4,
        "magnitude colormap should yield several distinct arrow fills, got {fills:?}"
    );
}

#[test]
fn test_quiver3d_per_arrow_color_override() {
    let plot = Quiver3DPlot::new()
        .with_arrow(0.0, 0.0, 0.0, 1.0, 0.0, 0.0)
        .with_colored_arrow(1.0, 1.0, 1.0, 1.0, 0.0, 0.0, "#00ff00");
    let svg = svg_of(plot);
    assert!(
        svg.contains("#00ff00"),
        "per-arrow color override should reach the SVG"
    );
}

#[test]
fn test_quiver3d_depth_shade_varies_opacity() {
    let svg = svg_of(vortex(3).with_depth_shade());
    common::write_test_output("test_outputs/quiver3d_depth_shade.svg", svg.clone()).unwrap();
    let arrow_opacities = |svg: &str| -> Vec<f64> {
        arrow_elements(svg)
            .into_iter()
            .filter_map(|el| {
                el.split("fill-opacity=\"")
                    .nth(1)
                    .and_then(|s| s.split('"').next())
                    .and_then(|v| v.parse().ok())
            })
            .collect()
    };
    let opacities = arrow_opacities(&svg);
    assert_eq!(
        opacities.len(),
        27,
        "depth shade should emit one fill-opacity per arrow"
    );
    let lo = opacities.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = opacities.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        hi - lo > 0.3,
        "near and far arrows should differ in opacity, got {lo}..{hi}"
    );
    let plain = svg_of(vortex(3));
    assert!(
        arrow_opacities(&plain).is_empty(),
        "arrows carry no fill-opacity without depth shade"
    );
}

/// Painter's algorithm: the arrow nearer the viewer is drawn later. The
/// nearest floor corner for the default view comes from `View3D` itself, so
/// the test does not bake in which data corner the box puts at the front.
#[test]
fn test_quiver3d_far_arrows_are_drawn_first() {
    let (fc_x, fc_y) = View3D::default().front_bottom_corner();
    let near = (
        if fc_x > 0.0 { 10.0 } else { 0.0 },
        if fc_y > 0.0 { 10.0 } else { 0.0 },
    );
    let far = (10.0 - near.0, 10.0 - near.1);
    let plot = Quiver3DPlot::new()
        .with_colored_arrow(near.0, near.1, 0.0, 1.0, 0.0, 0.0, "#ff0000")
        .with_colored_arrow(far.0, far.1, 0.0, 1.0, 0.0, 0.0, "#0000ff")
        .with_arrow(5.0, 5.0, 5.0, 1.0, 0.0, 0.0);
    let svg = svg_of(plot);
    let near_pos = svg.find("#ff0000").expect("near arrow fill missing");
    let far_pos = svg.find("#0000ff").expect("far arrow fill missing");
    assert!(
        far_pos < near_pos,
        "far arrow (blue) should precede near arrow (red)"
    );
}

#[test]
fn test_quiver3d_single_huge_arrow_stays_on_canvas() {
    let svg = svg_of(Quiver3DPlot::new().with_arrow(0.0, 0.0, 0.0, 500.0, 0.0, 0.0));
    common::write_test_output("test_outputs/quiver3d_single.svg", svg.clone()).unwrap();
    let (w, h) = (attr_f64(&svg, "width"), attr_f64(&svg, "height"));
    let pts = path_points(&svg);
    assert!(!pts.is_empty(), "the arrow should be drawn");
    for (x, y) in pts {
        assert!(
            (0.0..=w).contains(&x) && (0.0..=h).contains(&y),
            "arrow point ({x}, {y}) lies outside the {w}×{h} canvas"
        );
    }
}

/// Two arrows of different length: with Tail pivot the short one ends where
/// the gap to the long one begins, with Tip pivot the long one ends at the
/// short one's anchor — different relative layouts, so different geometry.
/// (A box that follows the arrows makes a uniform translation invisible.)
#[test]
fn test_quiver3d_pivot_moves_arrow() {
    let arrows = [
        (0.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        (2.0, 0.0, 0.0, 2.0, 0.0, 0.0),
    ];
    let tail = svg_of(Quiver3DPlot::new().with_arrows(arrows).with_scale(1.0));
    let tip = svg_of(
        Quiver3DPlot::new()
            .with_arrows(arrows)
            .with_scale(1.0)
            .with_pivot(QuiverPivot::Tip),
    );
    assert_ne!(
        path_data(&tail),
        path_data(&tip),
        "pivot should change where the arrow sits"
    );
}

#[test]
fn test_quiver3d_no_grid_no_box_draws_fewer_lines() {
    let full = svg_of(vortex(2));
    let bare = svg_of(vortex(2).with_no_grid().with_no_box());
    assert!(
        bare.matches("<line").count() < full.matches("<line").count(),
        "no-grid/no-box should remove wireframe lines"
    );
    assert_eq!(path_data(&bare).len(), 8, "arrows are still drawn");
}

/// A Quiver3D and a Scatter3D in one figure share one coordinate box, drawn
/// once: the wireframe line count must match a single plot spanning the same
/// merged range.
#[test]
fn test_quiver3d_shares_one_box_with_scatter3d() {
    let scatter = Scatter3DPlot::new().with_data(vec![(0.0, 0.0, 0.0), (1.0, 1.0, 1.0)]);
    let quiver = Quiver3DPlot::new()
        .with_arrow(5.0, 5.0, 5.0, 1.0, 0.0, 0.0)
        .with_arrow(6.0, 6.0, 6.0, 0.0, 1.0, 0.0);
    let plots = vec![Plot::Scatter3D(scatter), Plot::Quiver3D(quiver)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = render(plots, layout);
    common::write_test_output("test_outputs/quiver3d_with_scatter3d.svg", svg.clone()).unwrap();
    assert!(svg.contains("<circle"), "scatter points drawn");
    assert_eq!(path_data(&svg).len(), 2, "both arrows drawn");

    let single = Scatter3DPlot::new().with_data(vec![(0.0, 0.0, 0.0), (6.0, 6.0, 6.0)]);
    let single_plots = vec![Plot::Scatter3D(single)];
    let single_layout = Layout::auto_from_plots(&single_plots);
    let single_svg = render(single_plots, single_layout);
    assert_eq!(
        svg.matches("<line").count(),
        single_svg.matches("<line").count(),
        "combined figure should draw exactly one wireframe box over the merged range"
    );
}

/// A counter-clockwise vortex about +z must read counter-clockwise when
/// viewed from above. The projection is right-handed (+x right, +y up near
/// elevation 90°), so on screen the cross product of each arrow's radius
/// vector and its direction is positive. A mirrored projection flips every
/// sign here; this is the plot type where that becomes visible.
#[test]
fn test_quiver3d_ccw_vortex_reads_ccw_from_above() {
    let ring: Vec<(f64, f64, f64, f64, f64, f64)> = (0..12)
        .map(|i| {
            let t = i as f64 / 12.0 * std::f64::consts::TAU;
            (t.cos(), t.sin(), 0.0, -t.sin(), t.cos(), 0.0)
        })
        .collect();
    let plot = Quiver3DPlot::new()
        .with_arrows(ring)
        .with_azimuth(0.0)
        .with_elevation(89.0);
    let svg = svg_of(plot);
    // Each glyph is M tail_l, L base_l, L wing_l, L tip, L wing_r, L base_r, L tail_r Z.
    let arrows: Vec<((f64, f64), (f64, f64))> = path_data(&svg)
        .iter()
        .map(|d| {
            let n: Vec<f64> = d
                .split_whitespace()
                .filter_map(|t| t.parse().ok())
                .collect();
            let p = |i: usize| (n[2 * i], n[2 * i + 1]);
            let (tl, tr, tip) = (p(0), p(6), p(3));
            (((tl.0 + tr.0) / 2.0, (tl.1 + tr.1) / 2.0), tip)
        })
        .collect();
    assert_eq!(arrows.len(), 12, "all twelve ring arrows drawn");
    let cx = arrows.iter().map(|(t, _)| t.0).sum::<f64>() / 12.0;
    let cy = arrows.iter().map(|(t, _)| t.1).sum::<f64>() / 12.0;
    for (tail, tip) in &arrows {
        // Screen-up is -y in SVG coordinates.
        let (rx, ry) = (tail.0 - cx, -(tail.1 - cy));
        let (dx, dy) = (tip.0 - tail.0, -(tip.1 - tail.1));
        assert!(
            rx * dy - ry * dx > 0.0,
            "arrow at ({rx:.1}, {ry:.1}) points clockwise: direction ({dx:.1}, {dy:.1})"
        );
    }
}

#[test]
fn test_quiver3d_interactive_tooltips() {
    let plots = vec![Plot::Quiver3D(vortex(2))];
    let layout = Layout::auto_from_plots(&plots).with_interactive();
    let svg = render(plots, layout);
    assert!(
        svg.contains("|v|="),
        "interactive mode should attach a magnitude tooltip per arrow"
    );
}

#[cfg(feature = "png")]
#[test]
fn test_quiver3d_png_renders() {
    use kuva::backend::png::PngBackend;
    let plots = vec![Plot::Quiver3D(vortex(3).with_depth_shade())];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let bytes = PngBackend::new().render_scene(&scene).expect("PNG render");
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
}
