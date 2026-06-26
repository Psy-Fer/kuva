//! The colorbar reserves right-margin space sized to its widest tick label (after the
//! colorbar tick format is applied), so wide labels like `100000` no longer clip at the
//! canvas edge. When the canvas width is fixed too narrow for the full reservation, the
//! tick-label font is shrunk to fit the available band instead of clipping.

mod common;

use kuva::backend::svg::SvgBackend;
use kuva::plot::hexbin::HexbinPlot;
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

fn render_svg(plots: Vec<Plot>, layout: Layout) -> String {
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Parse the `width="N"` of the root `<svg>` element.
fn canvas_width(svg: &str) -> f64 {
    let start = svg.find("width=\"").expect("svg width attribute") + "width=\"".len();
    let end = svg[start..].find('"').unwrap() + start;
    svg[start..end].parse().expect("numeric width")
}

/// Find the tick-label `<text>` whose content is exactly `content` and return its
/// `(x, font_size)`.
fn text_x_and_font(svg: &str, content: &str) -> (f64, f64) {
    let needle = format!(">{content}</text>");
    let close = svg
        .find(&needle)
        .unwrap_or_else(|| panic!("no <text> with content {content:?}"));
    let open = svg[..close].rfind("<text ").expect("opening <text>");
    let tag = &svg[open..close];
    let attr = |name: &str| -> f64 {
        let key = format!("{name}=\"");
        let s = tag
            .find(&key)
            .unwrap_or_else(|| panic!("attr {name} on {tag}"))
            + key.len();
        let e = tag[s..].find('"').unwrap() + s;
        tag[s..e].parse().unwrap()
    };
    (attr("x"), attr("font-size"))
}

/// `peak` coincident points (one dense hex) plus far-apart singletons that pin v_min at 1.
fn make_hexbin_peak(peak: usize) -> Vec<Plot> {
    let mut x = vec![2.5_f64; peak];
    let mut y = vec![2.5_f64; peak];
    for (sx, sy) in [(0.2, 0.2), (4.8, 0.3), (0.3, 4.7), (4.6, 4.8)] {
        x.push(sx);
        y.push(sy);
    }
    vec![Plot::Hexbin(
        HexbinPlot::new().with_data(x, y).with_log_color(true),
    )]
}

#[test]
fn test_colorbar_reservation_grows_with_label_width() {
    // 2-char max label ("10") vs 6-char ("100000"): the auto-sized canvas must be wider
    // for the longer labels because the right margin is sized to them.
    let narrow_plots = make_hexbin_peak(11);
    let narrow_layout = Layout::auto_from_plots(&narrow_plots);
    let narrow = render_svg(narrow_plots, narrow_layout);
    let wide_plots = make_hexbin_peak(100_001);
    let wide_layout = Layout::auto_from_plots(&wide_plots);
    let wide = render_svg(wide_plots, wide_layout);
    common::write_test_output("test_outputs/colorbar_label_width_wide.svg", &wide).unwrap();
    assert!(
        canvas_width(&wide) > canvas_width(&narrow),
        "6-digit colorbar labels should widen the canvas vs 2-digit ({} !> {})",
        canvas_width(&wide),
        canvas_width(&narrow),
    );
}

#[test]
fn test_six_digit_colorbar_label_fits_within_canvas() {
    let plots = make_hexbin_peak(100_001);
    let layout = Layout::auto_from_plots(&plots);
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/colorbar_label_width_fits.svg", &svg).unwrap();
    let width = canvas_width(&svg);
    let (x, font) = text_x_and_font(&svg, "100000");
    // Start-anchored label extends right of x by ~chars * font * 0.6.
    let right_edge = x + "100000".len() as f64 * font * 0.6;
    assert!(
        right_edge <= width,
        "label right edge {right_edge} must not exceed canvas width {width}"
    );
}

#[test]
fn test_colorbar_label_font_shrinks_at_constrained_width() {
    // A fixed, narrow canvas can't grant the full reservation, so the 6-digit labels are
    // drawn at a smaller font rather than clipping.
    let plots = make_hexbin_peak(100_001);
    let layout = Layout::auto_from_plots(&plots).with_width(300.0);
    let svg = render_svg(plots, layout);
    common::write_test_output("test_outputs/colorbar_label_width_constrained.svg", &svg).unwrap();
    let (_x, font) = text_x_and_font(&svg, "100000");
    assert!(
        font < 12.0,
        "constrained colorbar label font {font} should shrink below the default 12"
    );
    // ...but it should still fit inside the canvas.
    let width = canvas_width(&svg);
    let (x, font) = text_x_and_font(&svg, "100000");
    assert!(x + "100000".len() as f64 * font * 0.6 <= width);
}
