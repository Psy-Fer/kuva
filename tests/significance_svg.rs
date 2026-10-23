mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::annotations::SignificanceBracket;
use kuva::plot::{BarPlot, BoxPlot, ViolinPlot};
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::theme::Theme;

fn boxes() -> BoxPlot {
    BoxPlot::new()
        .with_group("A", vec![1.0, 2.0, 3.0, 4.0, 5.0])
        .with_group("B", vec![4.0, 5.0, 6.0, 7.0, 8.0])
        .with_group("C", vec![2.0, 3.0, 4.0, 5.0, 6.0])
        .with_group("D", vec![3.0, 4.0, 5.0, 6.0, 7.0])
}

fn render(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Every 4-point `M L L L` path in the SVG, as its points. Brackets are the
/// only primitive drawn that way.
fn bracket_paths(svg: &str) -> Vec<Vec<(f64, f64)>> {
    let mut out = Vec::new();
    for tag in svg.split("<path").skip(1) {
        let Some(start) = tag.find(" d=\"") else {
            continue;
        };
        let d = &tag[start + 4..];
        let Some(end) = d.find('"') else { continue };
        let d = &d[..end];
        let toks: Vec<&str> = d.split_whitespace().collect();
        if toks.len() != 12 || toks[0] != "M" || toks[3] != "L" || toks[6] != "L" || toks[9] != "L"
        {
            continue;
        }
        let pt =
            |i: usize| -> (f64, f64) { (toks[i].parse().unwrap(), toks[i + 1].parse().unwrap()) };
        out.push(vec![pt(1), pt(4), pt(7), pt(10)]);
    }
    out
}

#[test]
fn bracket_draws_path_and_star_label() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots).with_significance("A", "B", 0.0004);
    let svg = render(plots, layout);
    common::write_test_output("test_outputs/significance_basic.svg", &svg).unwrap();

    let paths = bracket_paths(&svg);
    assert_eq!(paths.len(), 1, "one bracket path");
    let p = &paths[0];
    // Two downward ticks joined by a horizontal bar: ends share x with their
    // tick, bar is level, ticks point toward the data (larger pixel y).
    assert_eq!(p[0].0, p[1].0);
    assert_eq!(p[2].0, p[3].0);
    assert_eq!(p[1].1, p[2].1);
    assert!(p[0].1 > p[1].1 && p[3].1 > p[2].1);
    assert!(svg.contains(">***</text>"), "star label present");
    // Label is centred on the bar and sits above it.
    let label_y = common::text_y(&svg, "***");
    assert!(label_y < p[1].1);
}

#[test]
fn anchors_resolve_by_label_index_and_value_identically() {
    let plots = vec![Plot::Box(boxes())];
    let by_label = Layout::auto_from_plots(&plots).with_significance("A", "C", 0.01);
    let by_index = Layout::auto_from_plots(&plots).with_significance(0usize, 2usize, 0.01);
    let by_value = Layout::auto_from_plots(&plots).with_significance(1.0, 3.0, 0.01);
    let a = bracket_paths(&render(vec![Plot::Box(boxes())], by_label));
    let b = bracket_paths(&render(vec![Plot::Box(boxes())], by_index));
    let c = bracket_paths(&render(plots, by_value));
    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn unknown_label_draws_nothing() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots).with_significance("A", "nope", 0.01);
    let svg = render(plots, layout);
    assert!(bracket_paths(&svg).is_empty());
    assert!(!svg.contains(">*</text>"));
}

#[test]
fn touching_brackets_stack_and_the_spanning_one_sits_on_top() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_significance("A", "B", 0.04) // *
        .with_significance("B", "C", 0.004) // **
        .with_significance("A", "C", 0.0004); // ***
    let svg = render(plots, layout);
    common::write_test_output("test_outputs/significance_stack.svg", &svg).unwrap();

    let y_ab = common::text_y(&svg, "*");
    let y_bc = common::text_y(&svg, "**");
    let y_ac = common::text_y(&svg, "***");
    // Pixel y grows downward: A–B lowest, B–C above it (they touch at B),
    // A–C spans both and is highest.
    assert!(y_ab > y_bc, "A–B ({y_ab}) below B–C ({y_bc})");
    assert!(y_bc > y_ac, "B–C ({y_bc}) below A–C ({y_ac})");
    // Even pitch between consecutive levels.
    assert!(((y_ab - y_bc) - (y_bc - y_ac)).abs() < 0.6);
}

#[test]
fn disjoint_brackets_share_the_lowest_level() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_significance("A", "B", 0.04)
        .with_significance("C", "D", 0.004);
    let svg = render(plots, layout);
    assert_eq!(common::text_y(&svg, "*"), common::text_y(&svg, "**"));
}

#[test]
fn value_axis_grows_to_fit_the_stack_but_not_a_pinned_bracket() {
    let plots = vec![Plot::Box(boxes())];
    let plain = ComputedLayout::from_layout(&Layout::auto_from_plots(&plots));
    let stacked = ComputedLayout::from_layout(
        &Layout::auto_from_plots(&plots)
            .with_significance("A", "B", 0.04)
            .with_significance("B", "C", 0.04)
            .with_significance("A", "D", 0.04),
    );
    let pinned = ComputedLayout::from_layout(
        &Layout::auto_from_plots(&plots)
            .with_bracket(SignificanceBracket::new("A", "B", 0.04).with_position(2.0)),
    );
    assert!(stacked.y_range.1 > plain.y_range.1);
    assert_eq!(pinned.y_range.1, plain.y_range.1);
    assert_eq!(stacked.y_range.0, plain.y_range.0, "min is untouched");
    // The top bracket and its label fit inside the grown axis.
    let top = stacked.bracket_stack_y.position(2);
    assert!(top < stacked.y_range.1);
}

#[test]
fn explicit_axis_max_is_respected() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_y_axis_max(9.0)
        .with_significance("A", "B", 0.04)
        .with_significance("A", "D", 0.04);
    let computed = ComputedLayout::from_layout(&layout);
    assert_eq!(computed.y_range.1, 9.0);
}

#[test]
fn horizontal_plots_are_inferred_and_brackets_run_along_y() {
    let hplots = || vec![Plot::Box(boxes().with_horizontal(true))];
    // No `with_horizontal()` on the bracket: the layout has y_categories and
    // no x_categories, so orientation is inferred.
    let layout = Layout::auto_from_plots(&hplots()).with_significance("A", "B", 0.04);
    let svg = render(hplots(), layout);
    common::write_test_output("test_outputs/significance_horizontal.svg", &svg).unwrap();

    let paths = bracket_paths(&svg);
    assert_eq!(paths.len(), 1);
    let p = &paths[0];
    // Vertical bar with two ticks pointing left, toward the data.
    assert_eq!(p[1].0, p[2].0);
    assert_eq!(p[0].1, p[1].1);
    assert_eq!(p[2].1, p[3].1);
    assert!(p[0].0 < p[1].0 && p[3].0 < p[2].0);
    assert!(svg.contains(">*</text>"));

    // The explicit override draws the same thing.
    let forced = Layout::auto_from_plots(&hplots())
        .with_bracket(SignificanceBracket::new("A", "B", 0.04).with_horizontal());
    assert_eq!(bracket_paths(&render(hplots(), forced)), paths);

    // The x axis grew to make room on the right.
    let plain = ComputedLayout::from_layout(&Layout::auto_from_plots(&hplots()));
    let grown = ComputedLayout::from_layout(
        &Layout::auto_from_plots(&hplots()).with_significance("A", "B", 0.04),
    );
    assert!(grown.x_range.1 > plain.x_range.1);
}

#[test]
fn horizontal_labels_get_headroom_for_their_width() {
    let plots = vec![Plot::Box(boxes().with_horizontal(true))];
    let layout = Layout::auto_from_plots(&plots)
        .with_significance("A", "B", 0.012)
        .with_significance("C", "D", 0.3)
        .with_bracket(SignificanceBracket::text(
            "A",
            "D",
            "Mann–Whitney U, p = 0.012",
        ));
    let computed = ComputedLayout::from_layout(&layout);
    let svg = render(plots, layout);
    let plot_right = computed.width - computed.margin_right;
    let font = computed.body_size as f64;
    for label in ["*", "ns", "Mann–Whitney U, p = 0.012"] {
        let needle = format!(">{label}</text>");
        let end = svg.find(&needle).unwrap_or_else(|| panic!("no {label}"));
        let seg = &svg[..end];
        let xs = seg.rfind(r#" x=""#).unwrap() + 4;
        let xe = seg[xs..].find('"').unwrap() + xs;
        let x: f64 = seg[xs..xe].parse().unwrap();
        // A conservative 0.5 em per glyph: the real metric is used for the
        // headroom, so the label must end inside the plot area.
        assert!(
            x + 0.5 * font * label.chars().count() as f64 <= plot_right + 0.5,
            "{label} starts at {x} and would cross {plot_right}"
        );
    }
}

#[test]
fn short_canvas_keeps_labels_off_the_bar_above() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_height(200.0)
        .with_significance("A", "B", 0.04)
        .with_significance("B", "C", 0.004)
        .with_significance("A", "C", 0.0004);
    let computed = ComputedLayout::from_layout(&layout);
    let font = computed.body_size as f64;
    let svg = render(plots, layout);
    common::write_test_output("test_outputs/significance_short.svg", &svg).unwrap();
    let y0 = common::text_y(&svg, "*");
    let y1 = common::text_y(&svg, "**");
    let y2 = common::text_y(&svg, "***");
    // Pitch is at least 2.2 label heights, so a label never touches the bar
    // of the level above; the top label stays inside the plot area.
    assert!(y0 - y1 >= 2.0 * font, "pitch {} vs font {font}", y0 - y1);
    assert!(y1 - y2 >= 2.0 * font);
    assert!(
        y2 - font >= computed.margin_top,
        "top label clipped: {y2} < {}",
        computed.margin_top
    );
}

#[test]
fn default_colour_follows_the_theme() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_theme(Theme::dark())
        .with_significance("A", "B", 0.04);
    let svg = render(plots, layout);
    let tag = svg
        .split("<path")
        .find(|t| t.contains(" L ") && t.contains("fill=\"none\""))
        .expect("bracket path");
    let tag = &tag[..tag.find('>').unwrap()];
    assert!(
        !tag.contains("stroke=\"black\"") && !tag.contains("stroke=\"#000000\""),
        "bracket stroke is black on a dark theme: {tag}"
    );
    // An explicit colour is honoured as given, even black.
    let layout = Layout::auto_from_plots(&[Plot::Box(boxes())])
        .with_theme(Theme::dark())
        .with_bracket(SignificanceBracket::new("A", "B", 0.04).with_color("black"));
    let svg = render(vec![Plot::Box(boxes())], layout);
    let tag = svg
        .split("<path")
        .find(|t| t.contains(" L ") && t.contains("fill=\"none\""))
        .unwrap();
    let tag = &tag[..tag.find('>').unwrap()];
    assert!(
        tag.contains("stroke=\"black\"") || tag.contains("stroke=\"#000000\""),
        "{tag}"
    );
}

#[test]
fn p_value_both_and_text_labels_render() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_bracket(SignificanceBracket::new("A", "B", 0.041).with_p_value_label())
        .with_bracket(SignificanceBracket::new("C", "D", 0.0031).with_both_labels())
        .with_bracket(SignificanceBracket::text("A", "D", "n.s."));
    let svg = render(plots, layout);
    assert!(svg.contains(">p = 0.041</text>"), "{svg}");
    assert!(svg.contains(">** (p = 0.003)</text>"));
    assert!(svg.contains(">n.s.</text>"));
}

#[test]
fn log_axis_keeps_an_even_pixel_pitch() {
    let plot = BoxPlot::new()
        .with_group("A", vec![10.0, 20.0, 30.0, 40.0])
        .with_group("B", vec![100.0, 200.0, 300.0, 400.0])
        .with_group("C", vec![1000.0, 2000.0, 3000.0, 4000.0]);
    let plots = vec![Plot::Box(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_log_y()
        .with_significance("A", "B", 0.04)
        .with_significance("B", "C", 0.004)
        .with_significance("A", "C", 0.0004);
    let svg = render(plots, layout);
    common::write_test_output("test_outputs/significance_log.svg", &svg).unwrap();
    let y0 = common::text_y(&svg, "*");
    let y1 = common::text_y(&svg, "**");
    let y2 = common::text_y(&svg, "***");
    assert!(y0 > y1 && y1 > y2);
    assert!(((y0 - y1) - (y1 - y2)).abs() < 0.6, "{y0} {y1} {y2}");
}

#[test]
fn works_on_bar_and_violin() {
    let bar = BarPlot::new()
        .with_bar("ctrl", 3.0)
        .with_bar("drug", 5.0)
        .with_error(vec![0.5, 0.4]);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots).with_significance("ctrl", "drug", 0.0004);
    let svg = render(plots, layout);
    assert_eq!(bracket_paths(&svg).len(), 1);
    assert!(svg.contains(">***</text>"));

    let violin = ViolinPlot::new()
        .with_group("ctrl", vec![1.0, 2.0, 2.5, 3.0, 4.0, 5.0])
        .with_group("drug", vec![3.0, 4.0, 4.5, 5.0, 6.0, 7.0]);
    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots).with_significance(0usize, 1usize, 0.2);
    let svg = render(plots, layout);
    assert_eq!(bracket_paths(&svg).len(), 1);
    assert!(svg.contains(">ns</text>"));
}

#[test]
fn terminal_mode_keeps_labels_a_row_clear_of_the_bar() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_term_rows(30)
        .with_significance("A", "B", 0.04)
        .with_significance("B", "C", 0.004)
        .with_significance("A", "C", 0.0004);
    let computed = ComputedLayout::from_layout(&layout);
    let cell_h = computed.height / 30.0;
    let svg = render(plots, layout);
    let paths = bracket_paths(&svg);
    assert_eq!(paths.len(), 3);
    for (label, p) in [("*", &paths[0]), ("**", &paths[1]), ("***", &paths[2])] {
        let bar_y = p[1].1;
        let label_y = common::text_y(&svg, label);
        assert!(
            bar_y - label_y >= cell_h - 1e-6,
            "{label}: bar {bar_y}, label {label_y}, row {cell_h}"
        );
    }
    // Bar, label, blank row: at least three rows per level.
    assert!(paths[0][1].1 - paths[1][1].1 >= 3.0 * cell_h - 1e-6);
    assert!(paths[1][1].1 - paths[2][1].1 >= 3.0 * cell_h - 1e-6);
}

#[test]
fn horizontal_label_never_crosses_the_next_level() {
    let plots = vec![Plot::Box(boxes().with_horizontal(true))];
    let long = "Mann–Whitney U, p = 0.012";
    let layout = Layout::auto_from_plots(&plots)
        .with_bracket(SignificanceBracket::text("A", "B", long))
        .with_significance("A", "D", 0.3);
    let computed = ComputedLayout::from_layout(&layout);
    let font = computed.body_size as f64;
    let svg = render(plots, layout);
    let paths = bracket_paths(&svg);
    assert_eq!(paths.len(), 2);
    let next_bar_x = paths[1][1].0;
    let needle = format!(">{long}</text>");
    let end = svg.find(&needle).expect("long label");
    let seg = &svg[..end];
    let xs = seg.rfind(r#" x=""#).unwrap() + 4;
    let xe = seg[xs..].find('"').unwrap() + xs;
    let x: f64 = seg[xs..xe].parse().unwrap();
    assert!(
        x + 0.45 * font * long.chars().count() as f64 <= next_bar_x,
        "label from {x} crosses the next bar at {next_bar_x}"
    );
}
