mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::parallel::ParallelPlot;
use kuva::plot::polar::PolarPlot;
use kuva::plot::radar::RadarPlot;
use kuva::plot::scatter3d::Scatter3DPlot;
use kuva::plot::ternary::TernaryPlot;
use kuva::plot::{BarPlot, DotPlot, ScatterPlot};
use kuva::render::coverage::CoveragePlot;
use kuva::render::figure::Figure;
use kuva::render::layout::{ComputedLayout, Layout, TickFormat};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

const MONO_GROUP: &str = r#"<g font-family="monospace">"#;

fn context_bars() -> Vec<Plot> {
    vec![Plot::Bar(BarPlot::new().with_bars(vec![
        ("ACA", 3.0),
        ("ACG", 5.0),
        ("TCG", 2.0),
        ("TCT", 4.0),
    ]))]
}

/// The SVG between the tick-font group's opening tag and its closing `</g>`.
fn tick_font_group(svg: &str) -> &str {
    let start = svg.find(MONO_GROUP).expect("no tick font group in SVG") + MONO_GROUP.len();
    let end = svg[start..].find("</g>").unwrap() + start;
    &svg[start..end]
}

#[test]
fn tick_font_family_applies_to_rotated_tick_labels_only() {
    let plots = context_bars();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Contexts")
        .with_x_label("Context")
        .with_y_label("Count")
        .with_x_tick_rotate(-90.0)
        .with_tick_font_family("monospace");
    let svg = SvgBackend::new().render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/tick_font_rotated.svg", &svg).unwrap();

    assert!(
        svg.contains(r#"font-family="DejaVu Sans, Verdana, Liberation Sans, Arial, sans-serif""#),
        "the root keeps the default family"
    );
    assert_eq!(svg.matches(MONO_GROUP).count(), 1);

    let ticks = tick_font_group(&svg);
    for context in ["ACA", "ACG", "TCG", "TCT"] {
        assert!(
            ticks.contains(&format!(">{context}</text>")),
            "rotated tick label {context} should be inside the tick font group"
        );
    }
    assert!(
        ticks.contains("rotate(-90"),
        "tick labels should stay rotated"
    );
    for text in ["Contexts", "Context", "Count"] {
        assert!(
            !ticks.contains(&format!(">{text}</text>")),
            "{text} should keep the main font family"
        );
    }
}

#[test]
fn tick_font_family_covers_numeric_axes() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0_f64, 0.0_f64), (10.0, 5.0)]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_x_axis_min(0.0)
        .with_x_axis_max(10.0)
        .with_tick_font_family("monospace");
    let svg = SvgBackend::new().render_scene(&render_multiple(plots, layout));
    common::write_test_output("test_outputs/tick_font_numeric.svg", &svg).unwrap();

    let ticks = tick_font_group(&svg);
    assert!(
        ticks.contains(">10</text>"),
        "x tick labels inside the group"
    );
    assert!(
        ticks.contains(">5</text>"),
        "y tick labels inside the group"
    );
}

#[test]
fn tick_font_family_is_absent_by_default() {
    let plots = context_bars();
    let layout = Layout::auto_from_plots(&plots).with_x_tick_rotate(-90.0);
    let svg = SvgBackend::new().render_scene(&render_multiple(plots, layout));
    assert!(
        !svg.contains("<g font-family="),
        "no tick font group without with_tick_font_family"
    );
}

#[test]
fn tick_font_family_is_escaped() {
    let plots = context_bars();
    let layout = Layout::auto_from_plots(&plots).with_tick_font_family(r#""Fira Code", monospace"#);
    let svg = SvgBackend::new().render_scene(&render_multiple(plots, layout));
    assert!(svg.contains(r#"<g font-family="&quot;Fira Code&quot;, monospace">"#));
}

#[test]
fn tick_font_family_survives_figure_panels() {
    let mono = context_bars();
    let mono_layout = Layout::auto_from_plots(&mono)
        .with_title("Monospace ticks")
        .with_tick_font_family("monospace");
    let sans = context_bars();
    let sans_layout = Layout::auto_from_plots(&sans).with_title("Default ticks");

    let scene = Figure::new(1, 2)
        .with_plots(vec![mono, sans])
        .with_layouts(vec![mono_layout, sans_layout])
        .render();
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/tick_font_figure.svg", &svg).unwrap();

    assert_eq!(
        svg.matches(MONO_GROUP).count(),
        1,
        "only the panel that set a tick font gets the group"
    );
    assert!(tick_font_group(&svg).contains(">ACG</text>"));
}

// ── Complex / pixel-space plots ────────────────────────────────────────────────
//
// These plots draw their own tick labels instead of going through the cartesian
// axis renderer, so each needs the tick-font group wired at its own call site.
// Their *titles* (ternary corner labels, parallel axis names, 3D axis labels, the
// colorbar's rotated label) must stay in the main family — that split is what
// these tests pin.

/// Renders `plots` with a monospace tick family and returns the SVG.
fn mono_svg(plots: Vec<Plot>) -> String {
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Title")
        .with_tick_font_family("monospace");
    SvgBackend::new().render_scene(&render_multiple(plots, layout))
}

#[test]
fn tick_font_family_covers_polar_ring_and_spoke_labels() {
    let theta: Vec<f64> = (0..8).map(|i| i as f64 * 45.0).collect();
    let r: Vec<f64> = theta.iter().map(|&t| 1.0 + t.to_radians().cos()).collect();
    let svg = mono_svg(vec![Plot::Polar(PolarPlot::new().with_series(r, theta))]);
    common::write_test_output("test_outputs/tick_font_polar.svg", &svg).unwrap();

    let ticks = tick_font_group(&svg);
    assert!(
        ticks.contains("</text>"),
        "polar r/theta labels should sit inside the tick font group"
    );
    assert!(
        !ticks.contains(">Title</text>"),
        "the title keeps the main family"
    );
}

#[test]
fn tick_font_family_covers_ternary_percent_ticks_but_not_corner_labels() {
    let plot = TernaryPlot::new()
        .with_point(0.7, 0.2, 0.1)
        .with_point(0.1, 0.7, 0.2);
    let svg = mono_svg(vec![Plot::Ternary(plot)]);
    common::write_test_output("test_outputs/tick_font_ternary.svg", &svg).unwrap();

    let ticks = tick_font_group(&svg);
    assert!(
        ticks.contains(">20%</text>"),
        "edge percentage ticks should be inside the group"
    );
    // Corner labels name the axes — they are titles, not ticks.
    assert!(
        !ticks.contains(">A</text>"),
        "corner labels should keep the main family"
    );
}

#[test]
fn tick_font_family_covers_radar_rings_and_spoke_names() {
    let plot = RadarPlot::new(["ACG", "TCG", "GCA", "CCT"])
        .with_series_labeled(vec![0.8_f64, 0.6, 0.9, 0.7], "Group A");
    let svg = mono_svg(vec![Plot::Radar(plot)]);
    common::write_test_output("test_outputs/tick_font_radar.svg", &svg).unwrap();

    // Two groups: the ring value labels and the spoke names.
    assert_eq!(svg.matches(MONO_GROUP).count(), 2);
    assert!(
        svg.contains(&format!("{MONO_GROUP}<text")),
        "a group should open directly onto tick text"
    );
}

#[test]
fn tick_font_family_covers_parallel_axis_values_but_not_axis_names() {
    let plot = ParallelPlot::new()
        .with_axis_names(["Sepal.L", "Sepal.W"])
        .with_row(vec![5.1, 3.5])
        .with_row(vec![4.9, 3.0]);
    let svg = mono_svg(vec![Plot::Parallel(plot)]);
    common::write_test_output("test_outputs/tick_font_parallel.svg", &svg).unwrap();

    let ticks = tick_font_group(&svg);
    assert!(
        !ticks.contains(">Sepal.L</text>"),
        "axis names are titles and keep the main family"
    );
}

#[test]
fn tick_font_family_covers_3d_axis_ticks() {
    let plot = Scatter3DPlot::new().with_data(vec![(1.0, 2.0, 3.0), (2.0, 3.0, 4.0)]);
    let svg = mono_svg(vec![Plot::Scatter3D(plot)]);
    common::write_test_output("test_outputs/tick_font_scatter3d.svg", &svg).unwrap();

    // One group per axis (x, y, z).
    assert_eq!(svg.matches(MONO_GROUP).count(), 3);
}

#[test]
fn tick_font_family_covers_colorbar_ticks_but_not_its_label() {
    let dot = DotPlot::new()
        .with_data(vec![
            ("TypeA", "GeneX", 80.0_f64, 2.5_f64),
            ("TypeB", "GeneY", 50.0, 3.1),
        ])
        .with_colorbar("Mean expression");
    let svg = mono_svg(vec![Plot::DotPlot(dot)]);
    common::write_test_output("test_outputs/tick_font_colorbar.svg", &svg).unwrap();

    assert!(
        svg.contains("Mean expression</text>"),
        "sanity: the colorbar label must be present for the exclusion to mean anything"
    );
    // The rotated colorbar label is a title: it must not be inside any tick group.
    for group in svg.match_indices(MONO_GROUP) {
        let rest = &svg[group.0 + MONO_GROUP.len()..];
        let end = rest.find("</g>").unwrap_or(rest.len());
        assert!(
            !rest[..end].contains("Mean expression"),
            "the colorbar label should keep the main family"
        );
    }
}

// ── Width reservation follows the tick face ────────────────────────────────────

/// A y-axis whose tick labels (`0.0000`, `0.2000`, …) are long enough to clear the
/// two-character floor the gutter reservation applies, so the measured width is
/// what actually decides `margin_left`.
fn decimal_y_layout() -> Layout {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new().with_data(vec![(0.0_f64, 0.0_f64), (1.0, 1.0)]),
    )];
    Layout::auto_from_plots(&plots).with_y_tick_format(TickFormat::Fixed(4))
}

#[test]
fn monospace_tick_labels_reserve_more_left_margin_than_sans() {
    // Mono is only narrower than sans for digits; '.' is nearly twice as wide, so
    // a decimal y-axis measured with sans metrics under-reserves and clips. The
    // margin must come from the face the labels actually render in.
    let sans = ComputedLayout::from_layout(&decimal_y_layout());
    let mono = ComputedLayout::from_layout(&decimal_y_layout().with_tick_font_family("monospace"));
    assert!(
        mono.margin_left > sans.margin_left,
        "monospace decimal ticks need a wider gutter ({} vs {})",
        mono.margin_left,
        sans.margin_left
    );
}

#[test]
fn a_monospace_general_family_also_reserves_tick_labels_as_mono() {
    // `with_font_family("monospace")` renders ticks in mono too, so the same
    // reservation applies even though no tick-specific family was set.
    let sans = ComputedLayout::from_layout(&decimal_y_layout());
    let mono = ComputedLayout::from_layout(&decimal_y_layout().with_font_family("monospace"));
    assert!(mono.margin_left > sans.margin_left);
}

#[test]
fn a_proportional_tick_family_keeps_the_sans_reservation() {
    // Only the face changes the reservation, not the mere presence of the option.
    let sans = ComputedLayout::from_layout(&decimal_y_layout());
    let named = ComputedLayout::from_layout(&decimal_y_layout().with_tick_font_family("Helvetica"));
    assert_eq!(named.margin_left, sans.margin_left);
}

// ── Genome-browser composite ───────────────────────────────────────────────────

#[test]
fn tick_font_family_covers_the_coverage_track_stack() {
    let depth: Vec<(f64, f64)> = (0..=50)
        .map(|i| {
            let x = 1_000_000.0 + i as f64 * 1_000.0;
            (x, 40.0 + (i as f64 / 7.0).sin() * 20.0)
        })
        .collect();
    let scene = CoveragePlot::new()
        .with_title("Coverage")
        .with_locus(1_000_000.0, 1_050_000.0)
        .with_sample("tumour", depth)
        .with_x_label("chr7 position")
        .with_tick_font_family("monospace")
        .render_sized(900.0, 300.0);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/tick_font_coverage.svg", &svg).unwrap();

    assert!(
        svg.contains(MONO_GROUP),
        "the genomic axis and depth ticks should be grouped"
    );
    for group in svg.match_indices(MONO_GROUP) {
        let rest = &svg[group.0 + MONO_GROUP.len()..];
        let end = rest.find("</g>").unwrap_or(rest.len());
        let inside = &rest[..end];
        assert!(
            !inside.contains("chr7 position") && !inside.contains(">Coverage<"),
            "the axis label and title keep the main family"
        );
    }
}
