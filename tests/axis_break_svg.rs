mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::axis_break::AxisBreak;
use kuva::plot::{BarPlot, LinePlot, ScatterPlot};
use kuva::render::layout::{AxisLine, ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn bars() -> Vec<Plot> {
    vec![Plot::Bar(
        BarPlot::new()
            .with_bar("Liver", 8.2)
            .with_bar("Kidney", 6.9)
            .with_bar("Heart", 5.1)
            .with_bar("Brain", 7.4)
            .with_bar("Testis", 940.0),
    )]
}

fn render(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// `name="<number>"` attribute of an SVG tag body.
fn attr(tag: &str, name: &str) -> Option<f64> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    tag[start..end].parse().ok()
}

fn lines(svg: &str) -> Vec<(f64, f64, f64, f64)> {
    svg.split("<line")
        .skip(1)
        .filter_map(|t| {
            let t = &t[..t.find('>')?];
            Some((
                attr(t, "x1")?,
                attr(t, "y1")?,
                attr(t, "x2")?,
                attr(t, "y2")?,
            ))
        })
        .collect()
}

fn rects(svg: &str) -> Vec<(f64, f64, f64, f64)> {
    svg.split("<rect")
        .skip(1)
        .filter_map(|t| {
            let t = &t[..t.find('>')?];
            Some((
                attr(t, "x")?,
                attr(t, "y")?,
                attr(t, "width")?,
                attr(t, "height")?,
            ))
        })
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.05
}

#[test]
fn mapping_is_piecewise_continuous_and_monotonic() {
    let layout = Layout::auto_from_plots(&bars()).with_y_break(12.0, 880.0);
    let c = ComputedLayout::from_layout(&layout);
    let b = c.y_break_px.expect("break accepted");
    let bottom = c.height - c.margin_bottom;

    // Gap is 4% of the plot height; its edges are where lo and hi land.
    assert!(close(b.lo_px - b.hi_px, 0.04 * c.plot_height()));
    assert!(close(c.map_y(12.0), b.lo_px));
    assert!(close(c.map_y(880.0), b.hi_px));
    // The axis ends still sit on the plot edges.
    assert!(close(c.map_y(c.y_range.0), bottom));
    assert!(close(c.map_y(c.y_range.1), c.margin_top));
    // Strictly monotonic through the gap (pixel y grows downward).
    let ys: Vec<f64> = [0.0, 5.0, 12.0, 300.0, 600.0, 880.0, 950.0, 1000.0]
        .iter()
        .map(|v| c.map_y(*v))
        .collect();
    assert!(ys.windows(2).all(|w| w[0] > w[1]), "{ys:?}");
    // The short lower segment gets the 30% floor, not its 1:10 data share.
    let avail = c.plot_height() * 0.96;
    assert!(close((bottom - b.lo_px) / avail, 0.3));
}

#[test]
fn explicit_share_and_gap_are_honoured() {
    let layout = Layout::auto_from_plots(&bars())
        .with_y_break_config(AxisBreak::new(12.0, 880.0).with_share(0.5).with_gap(0.1));
    let c = ComputedLayout::from_layout(&layout);
    let b = c.y_break_px.unwrap();
    let bottom = c.height - c.margin_bottom;
    assert!(close(b.lo_px - b.hi_px, 0.1 * c.plot_height()));
    assert!(close((bottom - b.lo_px) / (c.plot_height() * 0.9), 0.5));
}

#[test]
fn ticks_cover_both_segments_and_skip_the_gap() {
    let svg = render(
        bars(),
        Layout::auto_from_plots(&bars()).with_y_break(12.0, 880.0),
    );
    common::write_test_output("test_outputs/axis_break_bar.svg", &svg).unwrap();
    for label in ["0", "5", "10", "900", "950", "1000"] {
        assert!(
            svg.contains(&format!(">{label}</text>")),
            "missing tick {label}"
        );
    }
    for label in ["250", "500", "750"] {
        assert!(
            !svg.contains(&format!(">{label}</text>")),
            "tick {label} inside the gap"
        );
    }
}

#[test]
fn y_axis_is_split_with_two_marks_and_a_cover() {
    let layout = Layout::auto_from_plots(&bars()).with_y_break(12.0, 880.0);
    let c = ComputedLayout::from_layout(&layout);
    let b = c.y_break_px.unwrap();
    let svg = render(bars(), layout);

    let all = lines(&svg);
    // Two vertical segments on the y-axis line, ending at the gap edges.
    let axis: Vec<_> = all
        .iter()
        .filter(|(x1, y1, x2, y2)| {
            close(*x1, c.margin_left) && close(*x2, c.margin_left) && (y2 - y1).abs() > 20.0
        })
        .collect();
    assert_eq!(axis.len(), 2, "{axis:?}");
    assert!(axis
        .iter()
        .any(|(_, y1, _, y2)| close(*y2, b.lo_px) || close(*y1, b.lo_px)));
    assert!(axis
        .iter()
        .any(|(_, y1, _, y2)| close(*y2, b.hi_px) || close(*y1, b.hi_px)));
    // Two slanted marks centred on the axis at the gap edges.
    let m = c.tick_mark_major.max(4.0);
    let marks: Vec<_> = all
        .iter()
        .filter(|(x1, y1, x2, y2)| {
            close(x2 - x1, 2.0 * m) && close(y1 - y2, m) && close((x1 + x2) / 2.0, c.margin_left)
        })
        .collect();
    assert_eq!(marks.len(), 2, "{marks:?}");
    // A background-coloured band over the plot area between the edges.
    let cover = rects(&svg).into_iter().find(|(x, y, w, h)| {
        close(*x, c.margin_left)
            && close(*y, b.hi_px)
            && close(*h, b.lo_px - b.hi_px)
            && close(*w, c.plot_width())
    });
    assert!(cover.is_some(), "no cover rect");

    // Without marks the axis is still split but the slashes are gone.
    let plain = render(
        bars(),
        Layout::auto_from_plots(&bars())
            .with_y_break_config(AxisBreak::new(12.0, 880.0).with_no_marks()),
    );
    let n_marks = lines(&plain)
        .iter()
        .filter(|(x1, y1, x2, y2)| close(x2 - x1, 2.0 * m) && close(y1 - y2, m))
        .count();
    assert_eq!(n_marks, 0);
}

#[test]
fn boxed_axes_split_the_right_side_too() {
    let layout = Layout::auto_from_plots(&bars())
        .with_axis_line(AxisLine::Box)
        .with_y_break(12.0, 880.0);
    let c = ComputedLayout::from_layout(&layout);
    let right = c.width - c.margin_right;
    let svg = render(bars(), layout);
    let n = lines(&svg)
        .iter()
        .filter(|(x1, y1, x2, y2)| close(*x1, right) && close(*x2, right) && (y2 - y1).abs() > 20.0)
        .count();
    assert_eq!(n, 2);
}

#[test]
fn bars_run_through_the_gap() {
    let layout = Layout::auto_from_plots(&bars()).with_y_break(12.0, 880.0);
    let c = ComputedLayout::from_layout(&layout);
    let b = c.y_break_px.unwrap();
    let svg = render(bars(), layout);
    let top = c.map_y(940.0);
    let tall = rects(&svg)
        .into_iter()
        .find(|(_, y, _, h)| close(*y, top) && close(y + h, c.map_y(0.0)))
        .expect("tall bar");
    // It starts above the gap and ends below it, so the cover slices it.
    assert!(tall.1 < b.hi_px && tall.1 + tall.3 > b.lo_px);
}

#[test]
fn x_break_on_a_line_splits_the_x_axis() {
    let mut pts: Vec<(f64, f64)> = (0..=10).map(|i| (i as f64, i as f64)).collect();
    pts.extend((0..=10).map(|i| (100.0 + i as f64, 10.0 - i as f64)));
    let plots = vec![Plot::Line(LinePlot::new().with_data(pts))];
    let layout = Layout::auto_from_plots(&plots).with_x_break(12.0, 98.0);
    let c = ComputedLayout::from_layout(&layout);
    let b = c.x_break_px.expect("x break accepted");
    assert!(c.y_break_px.is_none());
    let xs: Vec<f64> = [0.0, 5.0, 12.0, 50.0, 98.0, 105.0, 110.0]
        .iter()
        .map(|v| c.map_x(*v))
        .collect();
    assert!(xs.windows(2).all(|w| w[0] < w[1]), "{xs:?}");
    assert!(close(c.map_x(12.0), b.lo_px) && close(c.map_x(98.0), b.hi_px));

    let svg = render(plots, layout);
    common::write_test_output("test_outputs/axis_break_line.svg", &svg).unwrap();
    let bottom = c.height - c.margin_bottom;
    let axis: Vec<_> = lines(&svg)
        .into_iter()
        // Long horizontal runs on the axis line; the 5px tick at y = 0 is not one.
        .filter(|(x1, y1, x2, y2)| {
            close(*y1, bottom) && close(*y2, bottom) && (x2 - x1).abs() > 20.0
        })
        .collect();
    assert_eq!(axis.len(), 2, "x axis split in two: {axis:?}");
    assert!(svg.contains(">100</text>") && svg.contains(">5</text>"));
    assert!(!svg.contains(">50</text>"));
}

#[test]
fn unusable_breaks_are_ignored() {
    let plain = ComputedLayout::from_layout(&Layout::auto_from_plots(&bars()));
    let cases = [
        Layout::auto_from_plots(&bars()).with_y_break(880.0, 12.0), // reversed
        Layout::auto_from_plots(&bars()).with_y_break(2000.0, 3000.0), // outside the range
        Layout::auto_from_plots(&bars()).with_y_break(-5.0, 12.0),  // not strictly inside
        Layout::auto_from_plots(&bars())
            .with_log_y()
            .with_y_break(12.0, 880.0), // log axis
        Layout::auto_from_plots(&bars())
            .with_y_break_config(AxisBreak::new(12.0, 880.0).with_share(2.0)), // bad share
    ];
    for layout in cases {
        let c = ComputedLayout::from_layout(&layout);
        assert!(c.y_break.is_none() && c.y_break_px.is_none());
        if !layout.log_y {
            assert!(close(c.map_y(500.0), plain.map_y(500.0)));
        }
    }
    // Categorical y (horizontal bars): the break never applies to categories.
    let hbars = vec![Plot::Bar(
        BarPlot::new()
            .with_bar("a", 1.0)
            .with_bar("b", 900.0)
            .with_horizontal(true),
    )];
    let c = ComputedLayout::from_layout(&Layout::auto_from_plots(&hbars).with_y_break(1.2, 1.8));
    assert!(c.y_break.is_none());
}

#[test]
fn scatter_outliers_land_in_the_upper_segment() {
    let mut data: Vec<(f64, f64)> = (0..20).map(|i| (i as f64, (i % 5) as f64)).collect();
    data.push((10.0, 50.0));
    let plots = vec![Plot::Scatter(ScatterPlot::new().with_data(data))];
    let layout = Layout::auto_from_plots(&plots).with_y_break(6.0, 44.0);
    let c = ComputedLayout::from_layout(&layout);
    let b = c.y_break_px.unwrap();
    assert!(c.map_y(50.0) < b.hi_px);
    assert!(c.map_y(4.0) > b.lo_px);
}

#[test]
fn twin_y_and_candlestick_drop_the_break() {
    let mut layout = Layout::auto_from_plots(&bars()).with_y_break(12.0, 880.0);
    layout.y2_range = Some((0.0, 1.0));
    assert!(ComputedLayout::from_layout(&layout).y_break.is_none());

    let candles = || {
        vec![Plot::Candlestick(
            kuva::plot::CandlestickPlot::new()
                .with_candle("Mon", 10.0, 12.0, 9.0, 11.0)
                .with_candle("Tue", 11.0, 900.0, 10.0, 890.0),
        )]
    };
    let c = ComputedLayout::from_layout(&Layout::auto_from_plots(&candles()));
    let svg = render(
        candles(),
        Layout::auto_from_plots(&candles()).with_y_break(20.0, 800.0),
    );
    // The break is dropped: one unbroken y-axis segment, no break marks.
    let axis_segments = lines(&svg)
        .iter()
        .filter(|(x1, y1, x2, y2)| {
            close(*x1, c.margin_left) && close(*x2, c.margin_left) && (y2 - y1).abs() > 20.0
        })
        .count();
    assert_eq!(axis_segments, 1);
}

#[test]
fn pixel_space_plots_get_no_cover() {
    let pie = kuva::plot::PiePlot::new()
        .with_slice("a", 30.0, "tomato")
        .with_slice("b", 70.0, "steelblue");
    let plots = vec![Plot::Pie(pie)];
    let layout = Layout::auto_from_plots(&plots).with_y_break(0.3, 0.7);
    let c = ComputedLayout::from_layout(&layout);
    let svg = render(plots, layout);
    // Whatever the layout accepted, nothing paints a band over a pie.
    let band = rects(&svg)
        .into_iter()
        .any(|(x, _, w, _)| close(x, c.margin_left) && close(w, c.plot_width()));
    assert!(!band, "cover rect drawn on an axis-less plot");
}

fn cover_rect(svg: &str, b: &kuva::render::layout::BreakPx) -> Option<String> {
    svg.split("<rect").skip(1).find_map(|t| {
        let t = &t[..t.find('>')?];
        (attr(t, "y").is_some_and(|y| close(y, b.hi_px))
            && attr(t, "height").is_some_and(|h| close(h, b.lo_px - b.hi_px)))
        .then(|| t.to_string())
    })
}

#[test]
fn cover_matches_the_layout_background_and_skips_transparent() {
    let layout = Layout::auto_from_plots(&bars())
        .with_background("#123456")
        .with_y_break(12.0, 880.0);
    let c = ComputedLayout::from_layout(&layout);
    let b = c.y_break_px.unwrap();
    let svg = render(bars(), layout);
    let tag = cover_rect(&svg, &b).expect("cover rect");
    assert!(tag.to_lowercase().contains("#123456"), "{tag}");

    // A transparent canvas has no colour to paint with: no cover at all.
    let layout = Layout::auto_from_plots(&bars())
        .with_background("none")
        .with_y_break(12.0, 880.0);
    let svg = render(bars(), layout);
    assert!(
        cover_rect(&svg, &b).is_none(),
        "cover painted on a transparent canvas"
    );
}

#[test]
fn minor_ticks_follow_each_segments_step() {
    // Minor tick marks are only drawn on numeric x/y axes, so use a scatter.
    let mut data: Vec<(f64, f64)> = (0..20).map(|i| (i as f64, (i % 5) as f64)).collect();
    data.push((10.0, 50.0));
    let plots = || vec![Plot::Scatter(ScatterPlot::new().with_data(data.clone()))];
    let layout = Layout::auto_from_plots(&plots())
        .with_minor_ticks(5)
        .with_y_break(6.0, 44.0);
    let c = ComputedLayout::from_layout(&layout);
    let b = c.y_break_px.unwrap();
    let svg = render(plots(), layout);
    common::write_test_output("test_outputs/axis_break_minor.svg", &svg).unwrap();
    // Minor y ticks: short horizontal lines ending on the y axis, shorter
    // than a major tick.
    let minors: Vec<f64> = lines(&svg)
        .iter()
        .filter(|(x1, y1, x2, y2)| {
            close(*y1, *y2)
                && close(*x2, c.margin_left)
                && (x2 - x1).abs() < c.tick_mark_major - 0.01
        })
        .map(|(_, y, _, _)| *y)
        .collect();
    assert!(!minors.is_empty(), "no minor ticks drawn");
    assert!(
        minors
            .iter()
            .all(|y| *y <= b.hi_px + 0.01 || *y >= b.lo_px - 0.01),
        "minor tick inside the gap: {minors:?}"
    );
    let above = minors.iter().filter(|y| **y < b.hi_px).count();
    let below = minors.iter().filter(|y| **y > b.lo_px).count();
    assert!(above > 0 && below > 0, "above {above}, below {below}");
    // Within a segment the pitch is even (a double gap where a major tick
    // sits): no dense cluster from a phantom tick extrapolated with the other
    // segment's step.
    let mut up: Vec<f64> = minors.iter().copied().filter(|y| *y < b.hi_px).collect();
    up.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let gaps: Vec<f64> = up.windows(2).map(|w| w[1] - w[0]).collect();
    let pitch = gaps.iter().copied().fold(f64::MAX, f64::min);
    assert!(
        gaps.iter()
            .all(|g| (g - pitch).abs() < 0.6 || (g - 2.0 * pitch).abs() < 0.6),
        "uneven minor pitch above the gap: {gaps:?}"
    );
}
