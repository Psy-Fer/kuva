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
    let st = stacked.bracket_stack_y;
    let top = stacked.map_y(st.base) - st.offset_px(2);
    assert!(top > stacked.margin_top, "{top} vs {}", stacked.margin_top);
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

/// The y of each bracket's bar (the horizontal middle segment), sorted
/// bottom-up on screen (largest y first).
fn bar_ys(svg: &str) -> Vec<f64> {
    let mut ys: Vec<f64> = bracket_paths(svg).iter().map(|p| p[1].1).collect();
    ys.sort_by(|a, b| b.total_cmp(a));
    ys
}

fn three_brackets(layout: Layout) -> Layout {
    layout
        .with_significance("A", "B", 0.04)
        .with_significance("A", "C", 0.004)
        .with_significance("A", "D", 0.0004)
}

#[test]
fn pinned_axis_max_squeezes_the_stack_inside_the_plot() {
    let plots = || vec![Plot::Box(boxes())];
    let layout = three_brackets(Layout::auto_from_plots(&plots())).with_y_axis_max(12.0);
    let c = ComputedLayout::from_layout(&layout);
    assert_eq!(c.y_range.1, 12.0);
    assert!(!c.bracket_overflow);
    let svg = render(plots(), layout);
    let ys = bar_ys(&svg);
    assert_eq!(ys.len(), 3);
    // Every bar, and the top label, stays inside the plot area.
    assert!(ys.iter().all(|&y| y > c.margin_top), "{ys:?}");
    assert!(common::text_y(&svg, "***") > c.margin_top - 1.0);
    // Still evenly spaced.
    assert!(((ys[0] - ys[1]) - (ys[1] - ys[2])).abs() < 0.05, "{ys:?}");

    // A max that leaves no room at all is flagged rather than silently
    // drawing off the plot.
    let tight = three_brackets(Layout::auto_from_plots(&plots())).with_y_axis_max(8.0);
    assert!(ComputedLayout::from_layout(&tight).bracket_overflow);
}

#[test]
fn brackets_on_a_broken_axis_keep_an_even_pixel_pitch() {
    let tall = || {
        BoxPlot::new()
            .with_group("A", vec![1.0, 2.0, 3.0, 4.0, 5.0])
            .with_group("B", vec![4.0, 5.0, 6.0, 7.0, 8.0])
            .with_group("C", vec![2.0, 3.0, 4.0, 5.0, 6.0])
            .with_group("D", vec![560.0, 580.0, 600.0, 620.0, 640.0])
    };
    let plain = three_brackets(Layout::auto_from_plots(&[Plot::Box(tall())]));
    let broken =
        three_brackets(Layout::auto_from_plots(&[Plot::Box(tall())])).with_y_break(10.0, 500.0);
    let cp = ComputedLayout::from_layout(&plain);
    let cb = ComputedLayout::from_layout(&broken);
    assert!(cb.y_break.is_some(), "break applies");
    let ys_p = bar_ys(&render(vec![Plot::Box(tall())], plain));
    let svg = render(vec![Plot::Box(tall())], broken);
    common::write_test_output("test_outputs/significance_axis_break.svg", &svg).unwrap();
    let ys_b = bar_ys(&svg);
    // Same pixel pitch with and without the break, and the stack fits.
    let pitch = |ys: &[f64]| ys[0] - ys[1];
    assert!(
        (pitch(&ys_b) - pitch(&ys_p)).abs() < 0.05,
        "{ys_b:?} vs {ys_p:?}"
    );
    assert!((pitch(&ys_b) - cb.bracket_stack_y.pitch_px).abs() < 0.05);
    assert!(ys_b.iter().all(|&y| y > cb.margin_top), "{ys_b:?}");
    // The first bracket sits half a pitch above the top of the data, which
    // is above the break.
    assert!(cb.bracket_stack_y.base > 500.0);
    let top_px = cb.map_y(cb.bracket_stack_y.base);
    assert!((top_px - ys_b[0] - cb.bracket_stack_y.pitch_px / 2.0).abs() < 0.05);
    assert_eq!(cp.bracket_stack_y.pitch_px, cb.bracket_stack_y.pitch_px);
}

#[test]
fn brackets_survive_a_figure_and_shared_axes_stay_aligned() {
    use kuva::render::figure::Figure;
    let plots = vec![vec![Plot::Box(boxes())], vec![Plot::Box(boxes())]];
    let layouts = vec![
        three_brackets(Layout::auto_from_plots(&plots[0])),
        Layout::auto_from_plots(&plots[1]),
    ];
    let scene = Figure::new(1, 2)
        .with_plots(plots)
        .with_layouts(layouts)
        .with_shared_y_all()
        .render();
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/significance_figure_shared_y.svg", &svg).unwrap();
    assert_eq!(bracket_paths(&svg).len(), 3, "brackets drawn in the panel");
    // Identical data in both panels, and every group has the same IQR: with
    // a shared y range all eight box rects have one height. If the bracket
    // panel grew its own axis, its four boxes would come out shorter.
    let heights: Vec<&str> = svg
        .split("<rect")
        .skip(1)
        .filter_map(|t| {
            let h = t[..t.find('>').unwrap()].split(" height=\"").nth(1)?;
            Some(&h[..h.find('"').unwrap()])
        })
        .collect();
    let max_same = heights
        .iter()
        .map(|h| heights.iter().filter(|g| *g == h).count())
        .max()
        .unwrap();
    assert_eq!(max_same, 8, "{heights:?}");
}

#[test]
fn out_of_range_index_draws_nothing() {
    let plots = vec![Plot::Box(boxes())];
    let layout = Layout::auto_from_plots(&plots)
        .with_bracket(SignificanceBracket::new(0usize, 9usize, 0.01));
    assert!(bracket_paths(&render(plots, layout)).is_empty());
}

/// #136: a narrow log axis is tightened to 1-2-5 ticks (data from 33 gives a min of 20), but the
/// bracket stack stretches it past three decades, where only powers of ten are ticked. The min
/// must widen back to 10 so the axis starts on a tick, and the stack is resized for that min so
/// the top bracket still fits inside the plot.
#[test]
fn log_axis_min_returns_to_a_decade_when_brackets_widen_it() {
    let plot = BoxPlot::new()
        .with_group("Input", vec![80.0, 110.0, 120.0, 130.0, 160.0])
        .with_group("IP", vec![1700.0, 2200.0, 2400.0, 2600.0, 3300.0])
        .with_group("IgG", vec![33.0, 75.0, 90.0, 105.0, 145.0]);
    let plots = vec![Plot::Box(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_log_y()
        .with_significance("Input", "IP", 1e-6)
        .with_significance("IP", "IgG", 1e-6)
        .with_significance("Input", "IgG", 0.4);

    let computed = ComputedLayout::from_layout(&layout);
    assert_eq!(computed.y_range.0, 10.0, "min should be a power of ten");
    assert!(!computed.bracket_overflow);

    let svg = render(plots, layout);
    common::write_test_output("test_outputs/significance_log_widened_min.svg", &svg).unwrap();
    assert!(
        svg.contains(">10</text>"),
        "the axis should start on a labelled tick"
    );
    assert!(common::text_y(&svg, "ns") > computed.margin_top);
}
