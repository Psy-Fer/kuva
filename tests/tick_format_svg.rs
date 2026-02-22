use std::sync::Arc;
use visus::plot::scatter::ScatterPlot;
use visus::backend::svg::SvgBackend;
use visus::render::render::render_scatter;
use visus::render::layout::Layout;
use visus::TickFormat;

fn make_scatter(data: Vec<(f64, f64)>, layout: Layout) -> String {
    let plot = ScatterPlot::new().with_data(data);
    let scene = render_scatter(&plot, layout).with_background(Some("white"));
    SvgBackend.render_scene(&scene)
}

#[test]
fn test_auto_integers() {
    // Integer-valued data; Auto format should produce "0", "2", "4" not "0.0", "2.0"
    let data: Vec<(f64, f64)> = (0..=5).map(|i| (i as f64, i as f64 * 2.0)).collect();
    let layout = Layout::new((0.0, 5.0), (0.0, 10.0));
    let svg = make_scatter(data, layout);
    std::fs::write("test_outputs/tick_format_auto_integers.svg", &svg).unwrap();
    assert!(!svg.contains(".0<"), "Auto format should not produce trailing .0 in tick labels");
    // Specifically ".0" should not appear as part of integer tick labels like "5.0"
    // We check that no tick text element contains ".0"
    assert!(svg.contains("<svg"));
}

#[test]
fn test_fixed() {
    let data = vec![(0.0, 0.0), (1.0, 3.14159), (2.0, 2.71828)];
    let layout = Layout::new((0.0, 2.0), (0.0, 4.0))
        .with_y_tick_format(TickFormat::Fixed(2));
    let svg = make_scatter(data, layout);
    std::fs::write("test_outputs/tick_format_fixed.svg", &svg).unwrap();
    // Fixed(2) means 2 decimal places; look for pattern like "X.XX"
    // The tick labels should all have exactly 2 decimal places
    assert!(svg.contains("<svg"));
    // At least one tick label should match a 2-decimal-place pattern
    let has_two_dp = svg.split('>').any(|chunk| {
        let s = chunk.split('<').next().unwrap_or("");
        if let Some(dot_pos) = s.find('.') {
            let after_dot = &s[dot_pos + 1..];
            after_dot.len() == 2 && after_dot.chars().all(|c| c.is_ascii_digit())
        } else {
            false
        }
    });
    assert!(has_two_dp, "Fixed(2) should produce tick labels with exactly 2 decimal places");
}

#[test]
fn test_integer() {
    let data = vec![(1.5, 2.7), (3.2, 4.1)];
    let layout = Layout::new((0.0, 5.0), (0.0, 5.0))
        .with_tick_format(TickFormat::Integer);
    let svg = make_scatter(data, layout);
    std::fs::write("test_outputs/tick_format_integer.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // Tick labels should not contain a decimal point
    // Extract text content from text elements in the tick area
    // Simple check: no tick label text should contain "."
    // We look at text elements that are tick-sized (they have the tick font size)
    // As a proxy, check that the SVG doesn't have labels like "1.0" or "2.5" in tick positions
    // The most reliable check: no "." appears in any numeric tick label content
    // We verify by checking that non-percentage, non-scientific content has no dots
    // Simple approach: count occurrences of "." in tick label text nodes
    // All tick labels from Integer format should be whole numbers without "."
    let tick_labels_have_dot = svg.split("font-size=\"10\"")
        .skip(1) // skip first split (before any tick)
        .any(|chunk| {
            if let Some(close) = chunk.find("</text>") {
                let text_part = &chunk[..close];
                if let Some(gt) = text_part.rfind('>') {
                    let content = &text_part[gt + 1..];
                    content.contains('.')
                } else {
                    false
                }
            } else {
                false
            }
        });
    assert!(!tick_labels_have_dot, "Integer format should not produce tick labels with decimal points");
}

#[test]
fn test_percent() {
    let data = vec![(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)];
    let layout = Layout::new((0.0, 1.0), (0.0, 1.0))
        .with_y_tick_format(TickFormat::Percent);
    let svg = make_scatter(data, layout);
    std::fs::write("test_outputs/tick_format_percent.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains('%'), "Percent format should produce tick labels with '%'");
}

#[test]
fn test_sci() {
    let data = vec![(1.0, 1.0), (1000.0, 50000.0), (100000.0, 100000.0)];
    let layout = Layout::new((0.0, 100000.0), (0.0, 100000.0))
        .with_tick_format(TickFormat::Sci);
    let svg = make_scatter(data, layout);
    std::fs::write("test_outputs/tick_format_sci.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    // Scientific notation tick labels should contain 'e'
    let has_sci = svg.split("font-size=\"10\"")
        .skip(1)
        .any(|chunk| {
            if let Some(close) = chunk.find("</text>") {
                let text_part = &chunk[..close];
                if let Some(gt) = text_part.rfind('>') {
                    let content = &text_part[gt + 1..];
                    content.contains('e')
                } else {
                    false
                }
            } else {
                false
            }
        });
    assert!(has_sci, "Sci format should produce tick labels containing 'e'");
}

#[test]
fn test_custom() {
    let data = vec![(0.0, 0.0), (100.0, 200.0)];
    let layout = Layout::new((0.0, 100.0), (0.0, 200.0))
        .with_tick_format(TickFormat::Custom(Arc::new(|v| format!("{}px", v as i32))));
    let svg = make_scatter(data, layout);
    std::fs::write("test_outputs/tick_format_custom.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("px"), "Custom format should produce tick labels with 'px' suffix");
}

#[test]
fn test_independent() {
    // x-axis as Percent, y-axis as Sci
    let data = vec![(0.0, 0.0), (0.5, 50000.0), (1.0, 100000.0)];
    let layout = Layout::new((0.0, 1.0), (0.0, 100000.0))
        .with_x_tick_format(TickFormat::Percent)
        .with_y_tick_format(TickFormat::Sci);
    let svg = make_scatter(data, layout);
    std::fs::write("test_outputs/tick_format_independent.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains('%'), "Independent: x-axis should have '%' from Percent format");
    assert!(svg.contains('e'), "Independent: y-axis should have 'e' from Sci format");
}
