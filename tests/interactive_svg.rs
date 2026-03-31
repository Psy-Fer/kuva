use kuva::prelude::*;
use kuva::backend::svg::SvgBackend;
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
use kuva::plot::volcano::VolcanoPlot;

fn make_scatter_svg(interactive: bool) -> String {
    let mut layout = Layout::auto_from_plots(&[Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0), (5.0, 1.0)]),
    )]);
    if interactive {
        layout = layout.with_interactive();
    }
    let plots = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0), (5.0, 1.0)]),
    )];
    let scene = render_multiple(plots, layout);
    SvgBackend::new().render_scene(&scene)
}

fn make_multi_group_scatter_svg() -> String {
    let plots: Vec<Plot> = vec![
        ScatterPlot::new()
            .with_data(vec![(1.0_f64, 2.0), (2.0, 3.0)])
            .with_color("steelblue")
            .with_legend("Group A")
            .into(),
        ScatterPlot::new()
            .with_data(vec![(3.0_f64, 4.0), (4.0, 5.0)])
            .with_color("tomato")
            .with_legend("Group B")
            .into(),
    ];
    // auto_from_plots sets show_legend = true when plots have legend labels
    let layout = Layout::auto_from_plots(&plots).with_interactive();
    let scene = render_multiple(plots, layout);
    SvgBackend::new().render_scene(&scene)
}

fn make_volcano_svg(interactive: bool) -> String {
    let mut vp = VolcanoPlot::new();
    vp = vp
        .with_point("gene1", 2.5_f64, 0.001_f64)
        .with_point("gene2", -3.0_f64, 0.0001_f64)
        .with_point("gene3", 0.5_f64, 0.5_f64);

    let plots = vec![Plot::Volcano(vp)];
    let mut layout = Layout::auto_from_plots(&plots);
    if interactive {
        layout = layout.with_interactive();
    }
    let scene = render_multiple(plots, layout);
    SvgBackend::new().render_scene(&scene)
}

// ── 1. interactive off by default ────────────────────────────────────────────

#[test]
fn test_interactive_off_by_default() {
    let svg = make_scatter_svg(false);
    std::fs::write("test_outputs/interactive_off.svg", &svg).unwrap();
    assert!(!svg.contains("data-xmin"), "non-interactive SVG must not have data-xmin");
    assert!(!svg.contains("<script"), "non-interactive SVG must not have <script");
    assert!(!svg.contains("<foreignObject"), "non-interactive SVG must not have <foreignObject");
}

// ── 2. SVG root has all 8 data-* attributes ───────────────────────────────────

#[test]
fn test_interactive_root_attrs() {
    let svg = make_scatter_svg(true);
    std::fs::write("test_outputs/interactive_root_attrs.svg", &svg).unwrap();
    assert!(svg.contains("data-xmin="), "missing data-xmin");
    assert!(svg.contains("data-xmax="), "missing data-xmax");
    assert!(svg.contains("data-ymin="), "missing data-ymin");
    assert!(svg.contains("data-ymax="), "missing data-ymax");
    assert!(svg.contains("data-plot-left="), "missing data-plot-left");
    assert!(svg.contains("data-plot-top="), "missing data-plot-top");
    assert!(svg.contains("data-plot-right="), "missing data-plot-right");
    assert!(svg.contains("data-plot-bottom="), "missing data-plot-bottom");
}

// ── 3. log axis flag ──────────────────────────────────────────────────────────

#[test]
fn test_interactive_log_axis_flag() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(1.0_f64, 2.0), (10.0, 100.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_interactive()
        .with_log_y();
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    std::fs::write("test_outputs/interactive_log_y.svg", &svg).unwrap();
    assert!(svg.contains(r#"data-log-y="1""#), "expected data-log-y=\"1\" in SVG root");
}

// ── 4. scatter data-x and data-y attrs ───────────────────────────────────────

#[test]
fn test_interactive_scatter_data_attrs() {
    let svg = make_scatter_svg(true);
    std::fs::write("test_outputs/interactive_scatter_data_attrs.svg", &svg).unwrap();
    assert!(svg.contains("data-x="), "expected data-x= on scatter point groups");
    assert!(svg.contains("data-y="), "expected data-y= on scatter point groups");
}

// ── 5. no circle batch in interactive mode ────────────────────────────────────

#[test]
fn test_interactive_no_circle_batch() {
    let svg = make_scatter_svg(true);
    // CircleBatch renders as multiple individual <circle> elements without wrapping <g class="tt">.
    // In interactive mode every point is wrapped in a GroupStart, so circles appear inside <g>.
    // We verify individual wrapped groups exist by checking for data-x attr on group elements.
    assert!(svg.contains("data-x="), "scatter points should be individually wrapped in interactive mode");
    // Non-interactive mode has no data-x
    let svg_off = make_scatter_svg(false);
    assert!(!svg_off.contains("data-x="), "non-interactive scatter must not have data-x");
}

// ── 6. volcano threshold class ────────────────────────────────────────────────

#[test]
fn test_interactive_volcano_threshold() {
    let svg = make_volcano_svg(true);
    std::fs::write("test_outputs/interactive_volcano_threshold.svg", &svg).unwrap();
    assert!(
        svg.contains(r#"class="kuva-threshold""#),
        "expected class=\"kuva-threshold\" on volcano threshold lines"
    );
}

// ── 7. volcano point data-logfc ───────────────────────────────────────────────

#[test]
fn test_interactive_volcano_point_logfc() {
    let svg = make_volcano_svg(true);
    assert!(svg.contains("data-logfc="), "expected data-logfc= on volcano point groups");
}

// ── 8. legend data-group ──────────────────────────────────────────────────────

#[test]
fn test_interactive_legend_data_group() {
    let svg = make_multi_group_scatter_svg();
    std::fs::write("test_outputs/interactive_legend_data_group.svg", &svg).unwrap();
    assert!(
        svg.contains(r#"class="legend-entry""#),
        "expected class=\"legend-entry\" on legend entry groups"
    );
    assert!(svg.contains("data-group="), "expected data-group= on legend entry groups");
}

// ── 9. <script> present ───────────────────────────────────────────────────────

#[test]
fn test_interactive_script_present() {
    let svg = make_scatter_svg(true);
    assert!(svg.contains("<script"), "expected <script in interactive SVG");
    assert!(svg.contains("CDATA"), "expected CDATA section in script");
}

// ── 10. <foreignObject> present ───────────────────────────────────────────────

#[test]
fn test_interactive_foreignobject() {
    let svg = make_scatter_svg(true);
    assert!(svg.contains("<foreignObject"), "expected <foreignObject in interactive SVG");
    assert!(svg.contains("kuva-search"), "expected search input in foreignObject");
}

// ── 11. CSS pinned class ──────────────────────────────────────────────────────

#[test]
fn test_interactive_css_pinned() {
    let svg = make_scatter_svg(true);
    assert!(svg.contains(".pinned"), "expected .pinned CSS rule in interactive SVG");
}

// ── 12. non-interactive unchanged ─────────────────────────────────────────────

#[test]
fn test_noninteractive_unchanged() {
    let svg = make_scatter_svg(false);
    // Basic sanity: still has SVG root and circles, no interactive extras
    assert!(svg.contains("<svg"), "SVG root missing");
    assert!(svg.contains("<circle"), "expected circles in scatter");
    assert!(!svg.contains("data-xmin"), "non-interactive must not have axis metadata");
    assert!(!svg.contains(".pinned"), "non-interactive must not have pinned CSS");
}

// ── 13. Figure propagates interactive ─────────────────────────────────────────

#[test]
fn test_interactive_figure_propagates() {
    let make_scatter = || vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)]),
    )];
    let make_layout = || {
        let plots = make_scatter();
        Layout::auto_from_plots(&plots).with_interactive()
    };

    let figure = Figure::new(1, 2)
        .with_plots(vec![make_scatter(), make_scatter()])
        .with_layouts(vec![make_layout(), make_layout()]);

    let scene = figure.render();
    let svg = SvgBackend::new().render_scene(&scene);
    std::fs::write("test_outputs/interactive_figure.svg", &svg).unwrap();
    // Figure merges all panel SVGs into one; the child panel scenes are rendered with
    // interactive=true so they should have data-x attrs on scatter point groups.
    assert!(svg.contains("data-x="), "figure panels should have data-x on scatter points when interactive");
}
