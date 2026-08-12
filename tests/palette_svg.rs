mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::line::LinePlot;
use kuva::plot::scatter::ScatterPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::{Palette, Theme};

#[test]
fn test_palette_dark_theme_with_colorblind() {
    let pal = Palette::wong();

    let s1 = ScatterPlot::new()
        .with_data(vec![(1.0, 2.0), (2.0, 3.0), (3.0, 5.0)])
        .with_color(&pal[0])
        .with_legend("Group A");

    let s2 = ScatterPlot::new()
        .with_data(vec![(1.0, 4.0), (2.0, 1.0), (3.0, 3.0)])
        .with_color(&pal[1])
        .with_legend("Group B");

    let plots = vec![Plot::Scatter(s1), Plot::Scatter(s2)];
    let layout = Layout::auto_from_plots(&plots)
        .with_theme(Theme::dark())
        .with_title("Wong Palette + Dark Theme");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/palette_dark_colorblind.svg", &svg).unwrap();

    // Wong palette colors appear in the SVG
    assert!(svg.contains("#e69f00"), "expected wong color 0");
    assert!(svg.contains("#56b4e9"), "expected wong color 1");
    // Dark background
    assert!(
        svg.contains(r##"fill="#1e1e1e""##),
        "expected dark background"
    );
}

#[test]
fn test_palette_auto_cycle() {
    let s1 = ScatterPlot::new()
        .with_data(vec![(0.0, 1.0), (1.0, 2.0)])
        .with_legend("A");

    let s2 = ScatterPlot::new()
        .with_data(vec![(0.0, 3.0), (1.0, 4.0)])
        .with_legend("B");

    let s3 = ScatterPlot::new()
        .with_data(vec![(0.0, 5.0), (1.0, 6.0)])
        .with_legend("C");

    let plots = vec![Plot::Scatter(s1), Plot::Scatter(s2), Plot::Scatter(s3)];
    let layout = Layout::auto_from_plots(&plots)
        .with_palette(Palette::wong())
        .with_title("Auto-Cycle Wong");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/palette_auto_cycle.svg", &svg).unwrap();

    // First three Wong colors assigned automatically
    assert!(svg.contains("#e69f00"), "expected wong[0]");
    assert!(svg.contains("#56b4e9"), "expected wong[1]");
    assert!(svg.contains("#009e73"), "expected wong[2]");
}

#[test]
fn test_palette_indexing() {
    let pal = Palette::wong();

    assert_eq!(pal.len(), 8);
    assert_eq!(&pal[0], "#E69F00");
    assert_eq!(&pal[7], "#000000");
    // Wraps on overflow
    assert_eq!(&pal[8], "#E69F00");
    assert_eq!(&pal[10], "#009E73");
}

#[test]
fn test_palette_custom() {
    let pal = Palette::custom("mine", vec!["red".into(), "green".into(), "blue".into()]);

    assert_eq!(pal.name, "mine");
    assert_eq!(pal.len(), 3);
    assert_eq!(&pal[0], "red");
    assert_eq!(&pal[1], "green");
    assert_eq!(&pal[2], "blue");
    // Wraps
    assert_eq!(&pal[3], "red");
}

#[test]
fn test_palette_tritanopia() {
    let pal = Palette::tritanopia();

    // Tritanopia maps to Tol Bright
    assert_eq!(pal.len(), 7);
    assert_eq!(&pal[0], "#4477AA");
    assert_eq!(&pal[1], "#EE6677");

    // Use it with auto-cycle
    let l1 = LinePlot::new()
        .with_data(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.5)])
        .with_legend("X");

    let l2 = LinePlot::new()
        .with_data(vec![(0.0, 1.0), (1.0, 0.5), (2.0, 1.5)])
        .with_legend("Y");

    let plots = vec![Plot::Line(l1), Plot::Line(l2)];
    let layout = Layout::auto_from_plots(&plots)
        .with_palette(Palette::tritanopia())
        .with_title("Tritanopia Safe");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/palette_tritanopia.svg", &svg).unwrap();

    assert!(svg.contains("#4477aa"), "expected tol_bright[0]");
    assert!(svg.contains("#ee6677"), "expected tol_bright[1]");
}

// ── LTC palettes (#112) ─────────────────────────────────────────────────────────
// Render representative small/medium/large LTC palettes into test_outputs/ for
// visual inspection; assert the palette colours actually reach the SVG. Hex in the
// output is lowercased, so assertions use the lowercase form.

#[test]
fn test_palette_ltc_maya_auto_cycle() {
    // 5-colour palette across 5 line series.
    let series: Vec<Plot> = (0..5)
        .map(|i| {
            let base = i as f64;
            Plot::Line(
                LinePlot::new()
                    .with_data(vec![(0.0, base), (1.0, base + 1.0), (2.0, base + 0.5)])
                    .with_legend(format!("S{i}")),
            )
        })
        .collect();

    let layout = Layout::auto_from_plots(&series)
        .with_palette(Palette::maya())
        .with_title("LTC maya");
    let svg = SvgBackend.render_scene(&render_multiple(series, layout));
    common::write_test_output("test_outputs/palette_ltc_maya.svg", &svg).unwrap();

    // maya = 3d5a80, 98c1d9, e0fbfc, ee6c4d, 293241
    for c in ["#3d5a80", "#98c1d9", "#ee6c4d", "#293241"] {
        assert!(svg.contains(c), "expected maya colour {c}");
    }
}

#[test]
fn test_palette_ltc_minou_auto_cycle() {
    // 6-colour palette across 6 scatter series.
    let series: Vec<Plot> = (0..6)
        .map(|i| {
            let base = i as f64;
            Plot::Scatter(
                ScatterPlot::new()
                    .with_data(vec![(0.0, base), (1.0, base + 1.0), (2.0, base + 2.0)])
                    .with_legend(format!("G{i}")),
            )
        })
        .collect();

    let layout = Layout::auto_from_plots(&series)
        .with_palette(Palette::minou())
        .with_title("LTC minou");
    let svg = SvgBackend.render_scene(&render_multiple(series, layout));
    common::write_test_output("test_outputs/palette_ltc_minou.svg", &svg).unwrap();

    // minou = 00798c, d1495b, edae49, 66a182, 2e4057, 8d96a3
    for c in [
        "#00798c", "#d1495b", "#edae49", "#66a182", "#2e4057", "#8d96a3",
    ] {
        assert!(svg.contains(c), "expected minou colour {c}");
    }
}

#[test]
fn test_palette_ltc_casa_natal_auto_cycle() {
    // Largest LTC palette (9 colours) across 9 line series — exercises the full set.
    let series: Vec<Plot> = (0..9)
        .map(|i| {
            let base = i as f64;
            Plot::Line(
                LinePlot::new()
                    .with_data(vec![(0.0, base), (1.0, base + 1.0), (2.0, base + 0.7)])
                    .with_legend(format!("L{i}")),
            )
        })
        .collect();

    let layout = Layout::auto_from_plots(&series)
        .with_palette(Palette::casa_natal())
        .with_title("LTC casa_natal");
    let svg = SvgBackend.render_scene(&render_multiple(series, layout));
    common::write_test_output("test_outputs/palette_ltc_casa_natal.svg", &svg).unwrap();

    // casa_natal first/last few: 245e55 ... 1d1d1b, eae4da
    for c in ["#245e55", "#ed773c", "#808bc5", "#1d1d1b", "#eae4da"] {
        assert!(svg.contains(c), "expected casa_natal colour {c}");
    }
}

#[test]
fn test_palette_ltc_constructors_are_valid() {
    // Every LTC constructor: non-empty, valid #rrggbb, name matches.
    let pals = [
        (Palette::paloma(), "paloma", 5),
        (Palette::maya(), "maya", 5),
        (Palette::dora(), "dora", 5),
        (Palette::ploen(), "ploen", 5),
        (Palette::olga(), "olga", 5),
        (Palette::mterese(), "mterese", 5),
        (Palette::franscoise(), "franscoise", 5),
        (Palette::fernande(), "fernande", 4),
        (Palette::sylvie(), "sylvie", 5),
        (Palette::expevo(), "expevo", 6),
        (Palette::minou(), "minou", 6),
        (Palette::alger(), "alger", 5),
        (Palette::seafarer(), "seafarer", 5),
        (Palette::luminaries(), "luminaries", 6),
        (Palette::casa_natal(), "casa_natal", 9),
    ];
    for (pal, name, n) in pals {
        assert_eq!(pal.name, name);
        assert_eq!(pal.len(), n, "{name} colour count");
        for c in pal.colors() {
            assert_eq!(c.len(), 7, "{name}: {c} not #rrggbb");
            assert!(c.starts_with('#'));
            assert!(
                c[1..].chars().all(|ch| ch.is_ascii_hexdigit()),
                "{name}: {c}"
            );
        }
    }
}
