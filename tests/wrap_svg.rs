mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::legend::{LegendEntry, LegendShape};
use kuva::plot::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn svg(plots: Vec<Plot>, layout: Layout) -> String {
    std::fs::create_dir_all("test_outputs").ok();
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

fn scatter_plots() -> Vec<Plot> {
    vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)])
            .with_color("steelblue"),
    )]
}

// ── Title wrapping ───────────────────────────────────────────────────────────

#[test]
fn title_wrap_splits_long_title() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This is a very long title that should wrap")
        .with_title_wrap(20);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/wrap_title.svg", &out).unwrap();

    // Should produce multiple <text> elements for the title lines.
    // "This is a very long" (19 chars) and "title that should" (17) and "wrap" (4)
    assert!(out.contains("This is a very long"), "first wrapped line");
    assert!(out.contains("title that should"), "second wrapped line");
    assert!(out.contains(">wrap<"), "third wrapped line");
}

#[test]
fn title_no_wrap_when_short() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Short")
        .with_title_wrap(20);
    let out = svg(plots, layout);

    // Title fits in 20 chars — should appear as a single text element.
    assert!(out.contains(">Short<"));
}

#[test]
fn title_no_wrap_without_setting() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This is a very long title that should not wrap");
    let out = svg(plots, layout);

    // No wrap setting → title stays on one line.
    assert!(out.contains("This is a very long title that should not wrap"));
}

// ── X-label wrapping ─────────────────────────────────────────────────────────

#[test]
fn x_label_wrap_splits() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_x_label("A long x-axis label for testing wrapping behavior")
        .with_x_label_wrap(25);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/wrap_x_label.svg", &out).unwrap();

    // Should split into multiple lines.
    assert!(
        out.contains("A long x-axis label for"),
        "first x-label line"
    );
    assert!(out.contains("testing wrapping"), "second x-label line");
}

// ── Y-label wrapping ─────────────────────────────────────────────────────────

#[test]
fn y_label_wrap_produces_multiple_rotated_texts() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_y_label("Long rotated y-axis label text here")
        .with_y_label_wrap(15);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/wrap_y_label.svg", &out).unwrap();

    // Each wrapped line should be a separate rotated text.
    let rotated_count = out.matches("rotate(-90").count();
    assert!(
        rotated_count >= 2,
        "expected multiple rotated y-label lines, got {rotated_count}"
    );
}

// ── Legend wrapping ──────────────────────────────────────────────────────────

#[test]
fn legend_wrap_splits_long_labels() {
    let plots = scatter_plots();
    let entries = vec![
        LegendEntry {
            label: "A very long legend label that should wrap".into(),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
        LegendEntry {
            label: "Short".into(),
            color: "tomato".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_wrap(15);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/wrap_legend.svg", &out).unwrap();

    // The long label should be split across multiple text elements.
    assert!(out.contains("A very long"), "first legend line");
    assert!(out.contains("legend label"), "continuation line");
    // Short label should appear as-is.
    assert!(out.contains(">Short<"));
}

#[test]
fn legend_wrap_title() {
    let plots = scatter_plots();
    let entries = vec![LegendEntry {
        label: "Item".into(),
        color: "steelblue".into(),
        shape: LegendShape::Rect,
        dasharray: None,
    }];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_title("A long legend title that wraps")
        .with_legend_wrap(15);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/wrap_legend_title.svg", &out).unwrap();

    // At 15 chars: "A long legend" / "title that" / "wraps"
    assert!(out.contains("A long legend"), "first legend title line");
    assert!(out.contains("title that"), "second legend title line");
    assert!(out.contains(">wraps<"), "third legend title line");
}

// ── with_wrap (global) ───────────────────────────────────────────────────────

#[test]
fn global_wrap_applies_to_all() {
    let plots = scatter_plots();
    let entries = vec![LegendEntry {
        label: "This is a long legend entry label".into(),
        color: "steelblue".into(),
        shape: LegendShape::Rect,
        dasharray: None,
    }];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This is a long title that should wrap on all elements")
        .with_x_label("This is a long x-axis label")
        .with_legend_entries(entries)
        .with_wrap(20);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/wrap_global.svg", &out).unwrap();

    // Title should wrap.
    assert!(
        !out.contains("This is a long title that should wrap on all elements"),
        "title should NOT appear as single line"
    );
    // Legend should wrap.
    assert!(
        !out.contains("This is a long legend entry label"),
        "legend label should NOT appear as single line"
    );
}

#[test]
fn per_element_overrides_global() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("A medium-length title for testing")
        .with_wrap(10) // global: aggressive wrap
        .with_title_wrap(40); // override: title gets more room
    let out = svg(plots, layout);

    // Title is 32 chars, title_wrap is 40 → should NOT wrap.
    assert!(
        out.contains("A medium-length title for testing"),
        "title should stay on one line with per-element override"
    );
}

#[test]
fn per_element_set_before_global_still_wins() {
    // Calling with_title_wrap BEFORE with_wrap — with_wrap must not overwrite it.
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("A medium-length title for testing")
        .with_title_wrap(40) // per-element first: title gets 40 chars
        .with_wrap(10); // global second: must not override title_wrap
    let out = svg(plots, layout);

    // Title is 32 chars, effective title_wrap should still be 40 → no wrap.
    assert!(
        out.contains("A medium-length title for testing"),
        "title should stay on one line when per-element is set before global"
    );
}

// ── Edge cases ───────────────────────────────────────────────────────────────

#[test]
fn wrap_with_zero_is_disabled() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_title("This should not wrap at all even though wrap is called")
        .with_wrap(0);
    let out = svg(plots, layout);

    assert!(out.contains("This should not wrap at all even though wrap is called"));
}

#[test]
fn wrap_margin_grows_for_multiline_title() {
    let plots1 = scatter_plots();
    let layout_no_wrap = Layout::auto_from_plots(&plots1).with_title("Short title");
    let svg_no_wrap = svg(plots1, layout_no_wrap);

    let plots2 = scatter_plots();
    let layout_wrap = Layout::auto_from_plots(&plots2)
        .with_title("A very long title that definitely needs to wrap onto many lines")
        .with_title_wrap(15);
    let svg_wrap = svg(plots2, layout_wrap);

    // Extract the SVG height to verify margins grew.
    let height_no_wrap = extract_height(&svg_no_wrap);
    let height_wrap = extract_height(&svg_wrap);
    // The wrapped version should have the same or larger canvas
    // (margin_top grows to accommodate extra title lines).
    assert!(
        height_wrap >= height_no_wrap,
        "wrapped height ({height_wrap}) should be >= no-wrap height ({height_no_wrap})"
    );
}

fn extract_height(svg: &str) -> f64 {
    // Parse height="NNN" from the SVG root element.
    let start = svg.find("height=\"").unwrap() + 8;
    let end = start + svg[start..].find('"').unwrap();
    svg[start..end].parse().unwrap()
}

// ── Y2-label wrapping ────────────────────────────────────────────────────────

#[test]
fn y2_label_wrap_produces_multiple_rotated_texts() {
    use kuva::render::render::render_twin_y;

    let primary = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 2.0), (3.0, 4.0)])
            .with_color("steelblue"),
    )];
    let secondary = vec![Plot::Scatter(
        ScatterPlot::new()
            .with_data(vec![(1.0, 10.0), (3.0, 20.0)])
            .with_color("tomato"),
    )];

    let layout = Layout::new((0.0, 5.0), (0.0, 5.0))
        .with_y_label("Primary axis")
        .with_y2_range(0.0, 25.0)
        .with_y2_label("A very long secondary y-axis label that wraps")
        .with_y2_label_wrap(15);

    let scene = render_twin_y(primary, secondary, layout);
    let out = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/wrap_y2_label.svg", &out).unwrap();

    let rotated_90_count = out.matches("rotate(90").count();
    assert!(
        rotated_90_count >= 2,
        "expected multiple +90° rotated y2-label lines, got {rotated_90_count}"
    );
}

// ── Grouped legend wrapping ──────────────────────────────────────────────────

#[test]
fn grouped_legend_wrap() {
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_group(
            "A long group title that wraps",
            vec![
                LegendEntry {
                    label: "Entry with a long label text".into(),
                    color: "steelblue".into(),
                    shape: LegendShape::Rect,
                    dasharray: None,
                },
                LegendEntry {
                    label: "Short".into(),
                    color: "tomato".into(),
                    shape: LegendShape::Rect,
                    dasharray: None,
                },
            ],
        )
        .with_legend_wrap(15);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/wrap_grouped_legend.svg", &out).unwrap();

    // Group title wraps: "A long group" / "title that" / "wraps"
    assert!(out.contains("A long group"), "group title first line");
    assert!(out.contains("title that"), "group title second line");
    // Long entry wraps.
    assert!(out.contains("Entry with a"), "entry first line");
    assert!(out.contains("long label text"), "entry continuation");
    // Short entry stays on one line.
    assert!(out.contains(">Short<"));
}

// ── Outside-bottom legend with wrapping ──────────────────────────────────────

#[test]
fn outside_bottom_legend_wrap_adjusts_height() {
    use kuva::plot::LegendPosition;

    let plots1 = scatter_plots();
    let entries = vec![
        LegendEntry {
            label: "A very long legend entry that should definitely wrap".into(),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
        LegendEntry {
            label: "Another long entry for good measure here".into(),
            color: "tomato".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
    ];
    let layout = Layout::auto_from_plots(&plots1)
        .with_legend_entries(entries.clone())
        .with_legend_position(LegendPosition::OutsideBottomCenter)
        .with_legend_wrap(20);
    let svg_wrap = svg(plots1, layout);
    common::write_test_output("test_outputs/wrap_outside_bottom.svg", &svg_wrap).unwrap();

    let plots2 = scatter_plots();
    let layout_no = Layout::auto_from_plots(&plots2)
        .with_legend_entries(entries)
        .with_legend_position(LegendPosition::OutsideBottomCenter);
    let svg_no_wrap = svg(plots2, layout_no);

    // The wrapped version needs more vertical space for the taller legend.
    let h_wrap = extract_height(&svg_wrap);
    let h_no = extract_height(&svg_no_wrap);
    assert!(
        h_wrap >= h_no,
        "wrapped outside-bottom height ({h_wrap}) should be >= no-wrap ({h_no})"
    );
}

#[test]
fn outside_bottom_legend_no_wrap_renders_entries() {
    // Regression guard for the OutsideBottom positioning fix: verify that a plain
    // OutsideBottom legend (no wrapping) still renders its entries and doesn't
    // extend off the bottom of the canvas.
    use kuva::plot::LegendPosition;

    let plots = scatter_plots();
    let entries = vec![
        LegendEntry {
            label: "Alpha".into(),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
        LegendEntry {
            label: "Beta".into(),
            color: "tomato".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        },
    ];
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_position(LegendPosition::OutsideBottomCenter);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/outside_bottom_no_wrap.svg", &out).unwrap();

    // Both entries must appear.
    assert!(
        out.contains(">Alpha<"),
        "Alpha entry missing from OutsideBottom legend"
    );
    assert!(
        out.contains(">Beta<"),
        "Beta entry missing from OutsideBottom legend"
    );

    // Canvas must have grown beyond the default plot height to fit the legend.
    let height = extract_height(&out);
    assert!(
        height > 380.0,
        "canvas height ({height}) should exceed default 380px when OutsideBottom legend is present"
    );
}

// ── Legend column reflow ─────────────────────────────────────────────────────

/// Canvas width from the root `<svg width="…">`.
fn canvas_width(out: &str) -> f64 {
    let start = out.find("width=\"").unwrap() + 7;
    let end = start + out[start..].find('"').unwrap();
    out[start..end].parse().unwrap()
}

/// `(x, y, content)` of every `<text>` whose content starts with `prefix`.
fn texts_starting_with(out: &str, prefix: &str) -> Vec<(f64, f64, String)> {
    let attr = |head: &str, name: &str| -> Option<f64> {
        let key = format!(" {name}=\"");
        let i = head.find(&key)? + key.len();
        head[i..i + head[i..].find('"')?].parse().ok()
    };
    out.split("<text")
        .skip(1)
        .filter_map(|cap| {
            let gt = cap.find('>')?;
            let (head, rest) = (&cap[..gt], &cap[gt + 1..]);
            let content = &rest[..rest.find('<').unwrap_or(rest.len())];
            if !content.starts_with(prefix) {
                return None;
            }
            Some((attr(head, "x")?, attr(head, "y")?, content.to_string()))
        })
        .collect()
}

/// Every label starting with `prefix` must end inside the canvas. Width is
/// estimated at 8 px/char, an upper bound for DejaVu Sans at the default
/// 12 px body size (digits are 0.636 em ≈ 7.6 px).
fn assert_labels_on_canvas(out: &str, prefix: &str) {
    let width = canvas_width(out);
    let labels = texts_starting_with(out, prefix);
    assert!(!labels.is_empty(), "no labels starting with {prefix:?}");
    for (x, _, content) in labels {
        let right = x + content.chars().count() as f64 * 8.0;
        assert!(
            x > 0.0 && right <= width + 1.0,
            "legend label {content:?} spans x={x:.1}..{right:.1}, off the canvas (width {width})"
        );
    }
}

fn group_entries(n: usize) -> Vec<LegendEntry> {
    (1..=n)
        .map(|i| LegendEntry {
            label: format!("Group {i}"),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
        .collect()
}

fn grouped_scatters(n: usize) -> Vec<Plot> {
    (0..n)
        .map(|g| {
            Plot::Scatter(
                ScatterPlot::new()
                    .with_data(vec![(g as f64, g as f64)])
                    .with_legend(format!("group_{g:02}")),
            )
        })
        .collect()
}

#[test]
fn long_flat_legend_reflows_into_columns() {
    // 80 entries are far more than one column holds on a default canvas. The
    // renderer reflows them into columns instead of truncating: silently
    // dropping entries makes a legend of distinct categories misleading.
    // Entries supplied *after* auto_from_plots must still get their width.
    let plots = scatter_plots();
    let layout = Layout::auto_from_plots(&plots).with_legend_entries(group_entries(80));
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/legend_reflow_columns.svg", &out).unwrap();

    assert!(!out.contains("more)"), "legend should reflow, not truncate");
    for i in [1, 40, 80] {
        assert!(out.contains(&format!(">Group {i}<")), "entry {i} missing");
    }
    assert_labels_on_canvas(&out, "Group ");
}

/// Auto-collected legends (a scatter coloured per group never maintained the
/// per-plot-type entry tally) reserve one column width per reflowed column.
#[test]
fn reflowed_legend_columns_fit_on_canvas() {
    let plots = grouped_scatters(28);
    let layout = Layout::auto_from_plots(&plots).with_height(380.0);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/legend_reflow_fits.svg", &out).unwrap();

    for g in 0..28 {
        assert!(
            out.contains(&format!(">group_{g:02}<")),
            "group_{g:02} missing"
        );
    }
    assert_labels_on_canvas(&out, "group_");
}

/// `with_legend_title` after `auto_from_plots` re-derives the legend width; the
/// reflowed columns must survive that (they are reserved at render time, not by
/// scaling `legend_width` in `auto_from_plots`).
#[test]
fn reflowed_legend_survives_later_legend_builders() {
    let plots = grouped_scatters(28);
    let layout = Layout::auto_from_plots(&plots)
        .with_height(380.0)
        .with_legend_title("Samples");
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/legend_reflow_title.svg", &out).unwrap();
    assert_labels_on_canvas(&out, "group_");
    // Columns start below the title row: no entry sits on the title's line.
    let title_y = texts_starting_with(&out, "Samples")[0].1;
    for (_, y, c) in texts_starting_with(&out, "group_") {
        assert!(
            y > title_y + 1.0,
            "{c:?} at y={y} overlaps the title (y={title_y})"
        );
    }
}

/// Wrapped labels take several lines; the column split must count lines, not
/// entries, or the continuation lines of one entry overprint the next.
#[test]
fn reflowed_legend_with_wrap_does_not_overprint() {
    let plots = scatter_plots();
    let entries: Vec<LegendEntry> = (1..=40)
        .map(|i| LegendEntry {
            label: format!("Sample {i} long descriptive name"),
            color: "steelblue".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
        .collect();
    let layout = Layout::auto_from_plots(&plots)
        .with_legend_entries(entries)
        .with_legend_wrap(12);
    let out = svg(plots, layout);
    common::write_test_output("test_outputs/legend_reflow_wrap.svg", &out).unwrap();

    let mut seen = std::collections::HashSet::new();
    let width = canvas_width(&out);
    for (x, y, c) in texts_starting_with(&out, "") {
        // Legend text only: the legend sits right of the plot area.
        if x < width * 0.5 || c.is_empty() {
            continue;
        }
        let key = ((x * 10.0).round() as i64, (y * 10.0).round() as i64);
        assert!(
            seen.insert(key),
            "two legend lines drawn at x={x}, y={y} ({c:?})"
        );
    }
    assert!(out.contains(">Sample 40<"), "last entry missing");
    for (x, _, c) in texts_starting_with(&out, "Sample ") {
        assert!(
            x + c.chars().count() as f64 * 8.0 <= width + 1.0,
            "{c:?} at x={x} runs off the canvas"
        );
    }
}

/// A bottom-anchored legend is sized before it is positioned, so a reflowed box
/// sits in the reserved band below the plot instead of floating over it.
#[test]
fn reflowed_bottom_center_legend_stays_below_plot() {
    use kuva::plot::legend::LegendPosition;
    let make_layout = || {
        Layout::auto_from_plots(&scatter_plots())
            .with_legend_entries(group_entries(60))
            .with_legend_position(LegendPosition::OutsideBottomCenter)
    };
    let out = svg(scatter_plots(), make_layout());
    common::write_test_output("test_outputs/legend_reflow_bottom_center.svg", &out).unwrap();

    let computed = kuva::render::layout::ComputedLayout::from_layout(&make_layout());
    let plot_bottom = computed.height - computed.margin_bottom;
    let labels = texts_starting_with(&out, "Group ");
    assert_eq!(labels.len(), 60, "every entry drawn once");
    for (_, y, c) in labels {
        assert!(
            y > plot_bottom,
            "{c:?} at y={y} is inside the plot (bottom {plot_bottom})"
        );
    }
    assert_labels_on_canvas(&out, "Group ");
}

/// A legend that fits in one column is drawn exactly as before the reflow.
#[test]
fn short_legend_is_single_column() {
    let plots = grouped_scatters(5);
    let out = svg(plots, Layout::auto_from_plots(&grouped_scatters(5)));
    let xs: std::collections::BTreeSet<i64> = texts_starting_with(&out, "group_")
        .into_iter()
        .map(|(x, _, _)| x.round() as i64)
        .collect();
    assert_eq!(xs.len(), 1, "a short legend must stay one column");
}
