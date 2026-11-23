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

fn sankey_svg(sankey: SankeyPlot) -> String {
    let plots = vec![Plot::Sankey(sankey)];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend::new().render_scene(&render_multiple(plots, layout))
}

#[test]
fn sankey_gradient_node_colour_is_escaped() {
    let svg = sankey_svg(
        SankeyPlot::new()
            .with_link("A", "B", 3.0)
            .with_node_color("A", BREAKOUT)
            .with_gradient_links(),
    );
    common::write_test_output("test_outputs/svg_escaping_sankey_gradient.svg", &svg).unwrap();
    assert!(
        !svg.contains(BREAKOUT),
        "raw colour leaked into the gradient defs"
    );
    assert!(!svg.contains("<script>"));
    assert!(svg.contains(&format!(r#"stop-color="{ESCAPED_BREAKOUT}""#)));
}

#[test]
fn sankey_gradient_palette_entry_is_escaped() {
    let svg = sankey_svg(
        SankeyPlot::new()
            .with_link("A", "B", 3.0)
            .with_palette(vec![BREAKOUT.to_string(), "#00ff00".to_string()])
            .with_gradient_links(),
    );
    assert!(
        !svg.contains(BREAKOUT),
        "raw palette entry leaked into the gradient defs"
    );
    assert!(!svg.contains("<script>"));
    assert!(svg.contains(ESCAPED_BREAKOUT));
}

#[test]
fn sankey_gradients_in_two_figure_panels_do_not_share_ids() {
    let panel = |a: &str, b: &str| {
        vec![Plot::Sankey(
            SankeyPlot::new()
                .with_link("A", "B", 3.0)
                .with_node_color("A", a)
                .with_node_color("B", b)
                .with_gradient_links(),
        )]
    };
    let scene = Figure::new(1, 2)
        .with_plots(vec![
            panel("#ff0000", "#0000ff"),
            panel("#00ff00", "#ffff00"),
        ])
        .render();
    let svg = SvgBackend::new().render_scene(&scene);
    let ids: Vec<&str> = svg
        .match_indices(r#"<linearGradient id=""#)
        .map(|(i, m)| {
            let rest = &svg[i + m.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect();
    assert_eq!(ids.len(), 2, "{svg}");
    assert_ne!(ids[0], ids[1], "both panels define the same gradient id");
    for id in &ids {
        assert_eq!(
            svg.matches(&format!(r#"fill="url(#{id})""#)).count(),
            1,
            "{id}"
        );
    }
    // Ids are renumbered per document, so the output doesn't depend on earlier renders.
    assert_eq!(ids, ["kuva-grad-0", "kuva-grad-1"]);
}

// Control characters from data (ESC, BEL, C1 CSI, DEL) must never reach SVG output: they are
// illegal in XML 1.0, and the CLI prints SVG to the terminal by default.
const CONTROL: &str = "a\u{1b}]0;pwned\u{7}b\u{9b}31mc\u{7f}d";

fn has_unsafe_control(s: &str) -> Option<char> {
    s.chars()
        .find(|&c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
}

#[test]
fn control_characters_never_reach_svg_output() {
    let plots = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0_f64, 2.0), (2.0, 3.0)])
            .with_legend(CONTROL)
            .with_color(CONTROL)
            .with_tooltips()
            .with_tooltip_labels([CONTROL, CONTROL]),
    )];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(CONTROL)
        .with_x_label(CONTROL)
        .with_y_label(CONTROL)
        .with_interactive();
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend::new().render_scene(&scene);
    common::write_test_output("test_outputs/svg_escaping_control_chars.svg", &svg).unwrap();
    assert_eq!(has_unsafe_control(&svg), None);
    assert!(svg.contains("a\u{FFFD}]0;pwned\u{FFFD}b\u{FFFD}31mc\u{FFFD}d"));

    // The PDF backend parses the SVG as XML, so this also shows it is well-formed.
    #[cfg(feature = "pdf")]
    kuva::backend::pdf::PdfBackend::new()
        .render_scene(&scene)
        .expect("SVG with replaced control characters should parse");
}
