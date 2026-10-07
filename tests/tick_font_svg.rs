mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::{BarPlot, ScatterPlot};
use kuva::render::figure::Figure;
use kuva::render::layout::Layout;
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
