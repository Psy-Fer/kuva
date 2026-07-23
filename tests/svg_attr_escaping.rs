mod common;
use kuva::backend::svg::SvgBackend;
use kuva::prelude::*;
use kuva::render::layout::Layout;
use kuva::render::render::render_multiple;

// GHSA-3c48-9r95-hqhr — data-derived text (group names, legend labels, colors)
// must never be interpolated unescaped into a raw SVG attribute string, since
// that lets a crafted data file break out of the attribute and inject markup
// (e.g. an `onmouseover` handler or a `<script>` element) into the SVG DOM.

const BREAKOUT: &str = "evil\"><script>alert(1)</script>";
const ESCAPED_BREAKOUT: &str = "evil&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;";

#[test]
fn scatter_group_name_is_escaped() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0_f64, 2.0), (2.0, 3.0)])
            .with_legend(BREAKOUT),
    )];
    let layout = Layout::auto_from_plots(&plots).with_interactive();
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/svg_escaping_scatter_group.svg", &svg).unwrap();
    assert!(
        !svg.contains(BREAKOUT),
        "raw attribute-breakout string leaked into scatter group SVG"
    );
    assert!(
        svg.contains(ESCAPED_BREAKOUT),
        "expected escaped form of the malicious group name"
    );
}

#[test]
fn line_legend_label_is_escaped() {
    let plots = vec![Plot::Line(
        LinePlot::new()
            .with_data(vec![(1.0_f64, 2.0), (2.0, 3.0)])
            .with_legend(BREAKOUT),
    )];
    let layout = Layout::auto_from_plots(&plots).with_interactive();
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    assert!(
        !svg.contains(BREAKOUT),
        "raw attribute-breakout string leaked into line group SVG"
    );
    assert!(svg.contains(ESCAPED_BREAKOUT));
}

#[test]
fn bar_category_label_is_escaped() {
    let bar = BarPlot::new().with_bar(BREAKOUT, 10.0).with_bar("B", 20.0);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots).with_interactive();
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/svg_escaping_bar_category.svg", &svg).unwrap();
    assert!(
        !svg.contains(BREAKOUT),
        "raw attribute-breakout string leaked into bar category SVG"
    );
    assert!(svg.contains(ESCAPED_BREAKOUT));
}

#[test]
fn strip_group_label_is_escaped() {
    let strip = StripPlot::new().with_group(BREAKOUT, vec![1.0, 2.0, 3.0]);
    let plots = vec![Plot::Strip(strip)];
    let layout = Layout::auto_from_plots(&plots).with_interactive();
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    assert!(
        !svg.contains(BREAKOUT),
        "raw attribute-breakout string leaked into strip group SVG"
    );
    assert!(svg.contains(ESCAPED_BREAKOUT));
}

#[test]
fn legend_entry_label_is_escaped() {
    let plots: Vec<Plot> = vec![
        ScatterPlot::new()
            .with_data(vec![(1.0_f64, 2.0)])
            .with_legend(BREAKOUT)
            .into(),
        ScatterPlot::new()
            .with_data(vec![(2.0_f64, 3.0)])
            .with_legend("Group B")
            .into(),
    ];
    let layout = Layout::auto_from_plots(&plots).with_interactive();
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/svg_escaping_legend_entry.svg", &svg).unwrap();
    assert!(
        !svg.contains(BREAKOUT),
        "raw attribute-breakout string leaked into legend entry SVG"
    );
    assert!(svg.contains(ESCAPED_BREAKOUT));
}

#[test]
fn css_color_value_is_escaped() {
    // Doesn't match hex/rgb()/named-color parsing, so falls through to the
    // arbitrary-string `Color::Css` variant.
    let malicious = r#"red" onmouseover="alert(1)"#;
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0)])
        .with_color(malicious);
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/svg_escaping_css_color.svg", &svg).unwrap();
    assert!(
        !svg.contains(r#"" onmouseover="alert"#),
        "unescaped attribute breakout via Color::Css fill value"
    );
    assert!(svg.contains("&quot; onmouseover=&quot;alert(1)"));
}
