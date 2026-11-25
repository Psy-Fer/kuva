//! Colour, date and k-mer strings with multi-byte characters must not panic.
//! Each parser checked the string's byte length and then sliced it at fixed
//! byte offsets, which panics when an offset falls inside a character.

use kuva::backend::svg::SvgBackend;
use kuva::backend::terminal::TerminalBackend;
use kuva::plot::{BrickPlot, CalendarPlot, HorizonPlot, PiePlot, StreamgraphPlot};
use kuva::render::color::Color;
use kuva::render::render::{Primitive, Scene, TextAnchor};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

/// Characters 2, 3 and 4 bytes long.
const WIDE: [char; 3] = ['é', '€', '😀'];

/// Every string of exactly `len` bytes made of `prefix`, then ASCII `fill`,
/// with one wide character at each possible byte offset after the prefix.
fn straddling(prefix: &str, fill: char, len: usize) -> Vec<String> {
    let mut out = Vec::new();
    for c in WIDE {
        let w = c.len_utf8();
        for at in prefix.len()..=len.saturating_sub(w) {
            let mut s = prefix.to_string();
            s.extend(std::iter::repeat_n(fill, at - prefix.len()));
            s.push(c);
            s.extend(std::iter::repeat_n(fill, len - at - w));
            assert_eq!(s.len(), len);
            out.push(s);
        }
    }
    out
}

/// `#` colours of 4 and 7 bytes, the lengths the hex parsers check for.
fn hex_like() -> Vec<String> {
    let mut v = straddling("#", 'a', 4);
    v.extend(straddling("#", 'a', 7));
    v
}

fn render_svg(plot: Plot, layout: impl FnOnce(Layout) -> Layout) -> String {
    let plots = vec![plot];
    let l = layout(Layout::auto_from_plots(&plots));
    SvgBackend.render_scene(&render_multiple(plots, l))
}

#[test]
fn colour_strings_with_wide_characters_parse() {
    for s in hex_like() {
        // Not hex, so they are kept as CSS strings.
        assert!(matches!(Color::from(s.as_str()), Color::Css(_)), "{s:?}");
    }
    // Valid hex still parses.
    assert_eq!(Color::from("#a1B2c3"), Color::Rgb(0xa1, 0xb2, 0xc3));
    assert_eq!(Color::from("#abc"), Color::Rgb(0xaa, 0xbb, 0xcc));
}

#[test]
fn terminal_colours_with_wide_characters_render() {
    // The terminal backend trims colours before parsing them. The CSV reader
    // only trims ASCII whitespace, so a cell padded with U+00A0 reaches it
    // with a different byte length than the trimmed string.
    for s in hex_like() {
        for padded in [s.clone(), format!(" {s} "), format!("\u{a0}{s}\u{a0}")] {
            let mut scene = Scene::new(100.0, 60.0);
            scene.text_color = Some(padded.clone());
            scene.add(Primitive::Text {
                x: 10.0,
                y: 20.0,
                content: "label".into(),
                size: 12,
                anchor: TextAnchor::Start,
                rotate: None,
                bold: false,
                color: Some(Color::from(padded.as_str())),
            });
            assert!(!TerminalBackend::new(30, 8).render_scene(&scene).is_empty());

            let pie = PiePlot::new()
                .with_slice("a", 1.0, padded.as_str())
                .with_slice("b", 2.0, "#336699");
            let plots = vec![Plot::Pie(pie)];
            let layout = Layout::auto_from_plots(&plots);
            let scene = render_multiple(plots, layout);
            assert!(!TerminalBackend::new(40, 12).render_scene(&scene).is_empty());
            assert!(SvgBackend.render_scene(&scene).contains("<svg"));
        }
    }
}

#[test]
fn horizon_colours_with_wide_characters_render() {
    for s in hex_like() {
        for (values, signs) in [(false, false), (true, true)] {
            let plot = HorizonPlot::new()
                .with_series_colored(
                    "s",
                    [0.0, 1.0, 2.0, 3.0],
                    [1.0, -2.0, 3.0, -1.0],
                    s.as_str(),
                    s.as_str(),
                )
                .with_value_labels(values)
                .with_sign_colors(signs);
            assert!(render_svg(Plot::Horizon(plot), |l| l).contains("<svg"));
        }
    }
}

#[test]
fn streamgraph_label_colours_with_wide_characters_render() {
    // The label colour is picked from the raw series colour in BW mode.
    for s in straddling("#", 'a', 7) {
        let plot = StreamgraphPlot::new()
            .with_x([0.0, 1.0, 2.0, 3.0])
            .with_series([1.0, 2.0, 3.0, 2.0])
            .with_label("one")
            .with_color(s.as_str())
            .with_series([2.0, 1.0, 2.0, 3.0])
            .with_label("two");
        let svg = render_svg(Plot::Streamgraph(plot), |l| l.with_bw_mode());
        assert!(svg.contains("<svg"));
    }
}

#[test]
fn calendar_dates_with_wide_characters_are_skipped() {
    // Every 10 to 13 byte string with a wide character after "2024".
    let mut bad = Vec::new();
    for len in 10..=13 {
        bad.extend(straddling("2024", '1', len));
    }
    let mut data: Vec<(String, f64)> = bad.iter().map(|d| (d.clone(), 5.0)).collect();
    data.push(("2024-03-15".into(), 1.0));
    let plot = CalendarPlot::new().with_data(data);
    let svg = render_svg(Plot::Calendar(plot), |l| l);
    assert!(svg.contains("<svg"));

    for d in &bad {
        let plot = CalendarPlot::new()
            .with_data([("2024-03-15", 1.0)])
            .with_period("p", d.as_str(), d.as_str());
        assert!(render_svg(Plot::Calendar(plot), |l| l).contains("<svg"));
    }
}

#[test]
fn brick_kmers_with_wide_characters_build() {
    for kmer in ["Aé", "€CAG", "AT😀G", "éé"] {
        let motif = format!("{kmer}:A,CAG:B");
        let plot = BrickPlot::new()
            .with_motif_colors([(kmer, "#e41a1c")])
            .with_strigars([(motif.clone(), "3A2B".to_string())])
            .with_flanked_strigars([("ACGT", motif.as_str(), "3A2B", "TTTT")]);
        let svg = render_svg(Plot::Brick(plot), |l| l);
        assert!(svg.contains("<svg"));
    }
}
