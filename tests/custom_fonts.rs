//! Registering fonts at runtime (feature `custom_fonts`, #134).
//!
//! The registry is process-wide and every test here registers the same DejaVu
//! Serif files (re-registering replaces), so the tests can run in any order.
#![cfg(feature = "custom_fonts")]

mod common;

use kuva::fonts::{self, FontError};
use std::path::PathBuf;

fn font_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/fonts")
}

fn serif() -> PathBuf {
    font_dir().join("DejaVuSerif.ttf")
}

fn serif_bold() -> PathBuf {
    font_dir().join("DejaVuSerif-Bold.ttf")
}

/// A fresh scratch directory for one test.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kuva_fonts_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn register_file_reads_family_weight_and_style() {
    let infos = fonts::register_font_file(serif()).unwrap();
    assert_eq!(infos.len(), 1);
    let info = &infos[0];
    assert_eq!(info.family, "DejaVu Serif");
    assert_eq!(info.weight, 400);
    assert!(!info.italic && !info.monospace && !info.variable);
    assert!(!info.restricted_embedding);

    let bold = fonts::register_font_file(serif_bold()).unwrap();
    assert_eq!(bold[0].family, "DejaVu Serif", "bold joins the same family");
    assert_eq!(bold[0].weight, 700);
}

#[test]
fn family_styles_reports_missing_italics() {
    fonts::register_font_file(serif()).unwrap();
    fonts::register_font_file(serif_bold()).unwrap();
    // Case-insensitive family lookup.
    let styles = fonts::family_styles("dejavu serif").unwrap();
    assert_eq!(styles.regular.weight, 400);
    assert_eq!(styles.bold.weight, 700);
    // No italic faces registered: italic falls back to the upright faces, by weight.
    assert!(!styles.italic.italic);
    assert_eq!(styles.italic.weight, 400);
    assert_eq!(styles.bold_italic.weight, 700);
    assert_eq!(styles.missing(), ["italic", "bold italic"]);

    assert!(fonts::family_styles("No Such Family").is_none());
}

#[test]
fn re_registering_replaces_instead_of_duplicating() {
    for _ in 0..3 {
        fonts::register_font_file(serif()).unwrap();
    }
    let n = fonts::registered_fonts()
        .iter()
        .filter(|f| f.family == "DejaVu Serif" && f.weight == 400 && !f.italic)
        .count();
    assert_eq!(n, 1);
}

#[test]
fn register_bytes() {
    let bytes = std::fs::read(serif()).unwrap();
    let infos = fonts::register_font_bytes(bytes).unwrap();
    assert_eq!(infos[0].family, "DejaVu Serif");
}

#[test]
fn register_dir_finds_fonts_and_ignores_other_files() {
    // tests/data/fonts also holds LICENSE, which is not a font file.
    let report = fonts::register_fonts_dir(font_dir()).unwrap();
    let mut weights: Vec<u16> = report.registered.iter().map(|f| f.weight).collect();
    weights.sort();
    assert_eq!(weights, [400, 700]);
    assert!(report.skipped.is_empty());
}

#[test]
fn register_dir_skips_bad_files_and_recurses() {
    let dir = scratch("mixed");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::copy(serif(), dir.join("sub/Serif.TTF")).unwrap();
    std::fs::write(dir.join("broken.ttf"), b"not a font").unwrap();
    std::fs::write(dir.join("notes.txt"), b"ignored").unwrap();

    let report = fonts::register_fonts_dir(&dir).unwrap();
    assert_eq!(report.registered.len(), 1, "found in the subdirectory");
    assert_eq!(report.skipped.len(), 1);
    assert!(report.skipped[0].0.ends_with("broken.ttf"));
    assert!(matches!(report.skipped[0].1, FontError::Parse { .. }));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn register_dir_with_nothing_usable_is_an_error() {
    let dir = scratch("empty");
    std::fs::write(dir.join("broken.otf"), b"nope").unwrap();
    let err = fonts::register_fonts_dir(&dir).unwrap_err();
    match &err {
        FontError::NoFonts { skipped, .. } => assert_eq!(skipped.len(), 1),
        other => panic!("expected NoFonts, got {other:?}"),
    }
    assert!(err.to_string().contains("no usable fonts"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn errors_name_what_failed() {
    let err = fonts::register_font_file("/no/such/font.ttf").unwrap_err();
    assert!(matches!(err, FontError::Io { .. }));
    assert!(err.to_string().contains("/no/such/font.ttf"), "{err}");

    let err = fonts::register_font_bytes(b"definitely not a font".to_vec()).unwrap_err();
    assert!(matches!(err, FontError::Parse { .. }), "{err:?}");
    assert!(
        err.to_string().contains("not a font kuva can read"),
        "{err}"
    );

    let err = fonts::register_fonts_dir("/no/such/dir").unwrap_err();
    assert!(matches!(err, FontError::Io { .. }));
}

// ── Layout measures registered fonts (#134 step 3) ───────────────────────────

use kuva::plot::legend::LegendShape;
use kuva::plot::{BoxPlot, LegendEntry, ScatterPlot};
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;

fn register_serif() {
    fonts::register_font_file(serif()).unwrap();
    fonts::register_font_file(serif_bold()).unwrap();
}

fn scatter_with_legend() -> Vec<Plot> {
    vec![ScatterPlot::new()
        .with_data(vec![(1.0, 10.0), (2.0, 2000.0), (3.0, 30000.0)])
        .with_legend("Measured abundance (sample 7)")
        .into()]
}

fn legend_entries() -> Vec<LegendEntry> {
    ["Wild type control", "Knock-out, replicate 2"]
        .iter()
        .map(|l| LegendEntry {
            label: l.to_string(),
            color: "#336699".into(),
            shape: LegendShape::Rect,
            dasharray: None,
        })
        .collect()
}

#[test]
fn legend_width_follows_a_registered_font_set_after_auto_from_plots() {
    register_serif();
    let plots = scatter_with_legend();
    let sans = ComputedLayout::from_layout(&Layout::auto_from_plots(&plots));
    // The family is set *after* auto_from_plots measured the legend labels.
    let serif = ComputedLayout::from_layout(
        &Layout::auto_from_plots(&plots).with_font_family("DejaVu Serif, serif"),
    );
    assert!(
        (serif.legend_width - sans.legend_width).abs() > 1.0,
        "serif {} vs sans {}",
        serif.legend_width,
        sans.legend_width
    );
}

#[test]
fn legend_width_does_not_depend_on_builder_order() {
    register_serif();
    let before = Layout::new((0.0, 1.0), (0.0, 1.0))
        .with_font_family("DejaVu Serif")
        .with_legend_entries(legend_entries())
        .with_legend_title("Genotype");
    let after = Layout::new((0.0, 1.0), (0.0, 1.0))
        .with_legend_entries(legend_entries())
        .with_legend_title("Genotype")
        .with_font_family("DejaVu Serif");
    let sans = Layout::new((0.0, 1.0), (0.0, 1.0))
        .with_legend_entries(legend_entries())
        .with_legend_title("Genotype");
    let w = |l: &Layout| ComputedLayout::from_layout(l).legend_width;
    assert_eq!(w(&before), w(&after));
    assert!((w(&after) - w(&sans)).abs() > 1.0);
    // The public field tracks the family too.
    assert_eq!(after.legend_width, before.legend_width);
}

#[test]
fn tick_label_margin_follows_a_registered_tick_family() {
    register_serif();
    // Text category labels: DejaVu Serif and Sans share their digit widths, so
    // numeric ticks would not tell them apart.
    let plots: Vec<Plot> = vec![BoxPlot::new()
        .with_group("Wild type control", vec![1.0, 2.0, 3.0])
        .with_group("Knock-out, replicate 2", vec![2.0, 3.0, 4.0])
        .with_horizontal(true)
        .into()];
    let sans = ComputedLayout::from_layout(&Layout::auto_from_plots(&plots));
    let serif = ComputedLayout::from_layout(
        &Layout::auto_from_plots(&plots).with_tick_font_family("DejaVu Serif"),
    );
    assert!(
        (serif.y_tick_label_px - sans.y_tick_label_px).abs() > 1.0,
        "serif {} vs sans {}",
        serif.y_tick_label_px,
        sans.y_tick_label_px
    );
    assert!((serif.margin_left - sans.margin_left).abs() > 1.0);
}

#[test]
fn an_unregistered_family_still_measures_as_dejavu() {
    let plots = scatter_with_legend();
    let default = ComputedLayout::from_layout(&Layout::auto_from_plots(&plots));
    let other = ComputedLayout::from_layout(
        &Layout::auto_from_plots(&plots)
            .with_font_family("Some Font Nobody Registered, sans-serif"),
    );
    assert_eq!(default.legend_width, other.legend_width);
    assert_eq!(default.margin_left, other.margin_left);
}

/// For checking by eye: the same plot laid out in DejaVu Sans and in DejaVu Serif.
/// Labels, legend and margins should each hug their own font (the SVG names the
/// family; it renders as Serif where DejaVu Serif is installed).
#[test]
fn write_serif_and_sans_layouts_for_inspection() {
    register_serif();
    let plots = || -> Vec<Plot> {
        vec![BoxPlot::new()
            .with_group("Wild type control", vec![1.0, 2.0, 3.0, 2.5])
            .with_group("Knock-out, replicate 2", vec![2.0, 3.0, 4.0, 3.2])
            .with_group("Rescue (low dose)", vec![1.5, 2.2, 2.9, 2.4])
            .with_horizontal(true)
            .into()]
    };
    let render = |layout: Layout| {
        kuva::backend::svg::SvgBackend::new()
            .render_scene(&kuva::render::render::render_multiple(plots(), layout))
    };
    let titled = |l: Layout| {
        l.with_title("Expression by genotype")
            .with_x_label("Normalised expression")
            .with_legend_entries(legend_entries())
            .with_legend_title("Genotype")
    };
    let sans = render(titled(Layout::auto_from_plots(&plots())));
    let serif = render(titled(Layout::auto_from_plots(&plots())).with_font_family("DejaVu Serif"));
    assert!(serif.contains("font-family=\"DejaVu Serif\""));
    assert_ne!(sans, serif, "the layouts should differ");
    common::write_test_output("test_outputs/custom_font_sans.svg", sans).unwrap();
    common::write_test_output("test_outputs/custom_font_serif.svg", serif).unwrap();
}

// ── PNG draws registered fonts (#134 step 4) ─────────────────────────────────

#[cfg(feature = "png")]
fn genotype_png(family: Option<&str>) -> Vec<u8> {
    let plots = || -> Vec<Plot> {
        vec![BoxPlot::new()
            .with_group("Wild type control", vec![1.0, 2.0, 3.0, 2.5])
            .with_group("Knock-out, replicate 2", vec![2.0, 3.0, 4.0, 3.2])
            .with_horizontal(true)
            .into()]
    };
    let mut layout = Layout::auto_from_plots(&plots())
        .with_title("Expression by genotype")
        .with_legend_entries(legend_entries())
        .with_legend_title("Genotype");
    if let Some(family) = family {
        layout = layout.with_font_family(family);
    }
    let scene = kuva::render::render::render_multiple(plots(), layout);
    kuva::backend::png::PngBackend::new()
        .render_scene(&scene)
        .unwrap()
}

#[cfg(feature = "png")]
#[test]
fn png_draws_a_registered_font() {
    register_serif();
    let sans = genotype_png(None);
    let serif = genotype_png(Some("DejaVu Serif"));
    assert_ne!(sans, serif, "the PNG should be drawn in DejaVu Serif");
    // An unregistered family is drawn (and measured) as the bundled DejaVu Sans.
    assert_eq!(
        sans,
        genotype_png(Some("Nobody Registered This, sans-serif"))
    );
    common::write_test_output("test_outputs/custom_font_serif.png", serif).unwrap();
    common::write_test_output("test_outputs/custom_font_sans.png", sans).unwrap();
}

// ── PDF embeds registered fonts (#134 step 5) ────────────────────────────────

#[cfg(feature = "pdf")]
#[test]
fn pdf_embeds_a_registered_font() {
    register_serif();
    let plots = || -> Vec<Plot> { scatter_with_legend() };
    let layout = Layout::auto_from_plots(&plots())
        .with_title("Expression by genotype")
        .with_font_family("DejaVu Serif");
    let scene = kuva::render::render::render_multiple(plots(), layout);
    let pdf = kuva::backend::pdf::PdfBackend::new()
        .render_scene(&scene)
        .unwrap();
    // krilla subsets the font and names it in the font dictionary.
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("DejaVuSerif"),
        "PDF should embed DejaVu Serif"
    );
    common::write_test_output("test_outputs/custom_font_serif.pdf", pdf).unwrap();
}

// ── SVG: font-synthesis, embedding, mono family (#134 step 6) ────────────────

fn svg_of(plots: Vec<Plot>, layout: Layout, embed: bool) -> String {
    let scene = kuva::render::render::render_multiple(plots, layout);
    let backend = kuva::backend::svg::SvgBackend::new();
    #[cfg(feature = "embed_font")]
    let backend = backend.with_embedded_font(embed);
    #[cfg(not(feature = "embed_font"))]
    let _ = embed;
    backend.render_scene(&scene)
}

#[test]
fn registered_fonts_turn_off_style_synthesis() {
    register_serif();
    let plain = svg_of(
        scatter_with_legend(),
        Layout::auto_from_plots(&scatter_with_legend()),
        false,
    );
    assert!(
        !plain.contains("font-synthesis"),
        "default output unchanged"
    );
    let serif = svg_of(
        scatter_with_legend(),
        Layout::auto_from_plots(&scatter_with_legend()).with_font_family("DejaVu Serif"),
        false,
    );
    let root = &serif[..serif.find('>').unwrap()];
    assert!(root.contains(r#"style="font-synthesis:none""#), "{root}");
}

#[cfg(feature = "embed_font")]
#[test]
fn embedding_adds_exactly_the_registered_faces_drawn() {
    register_serif();
    let rules = |svg: &str| -> Vec<String> {
        svg.match_indices("@font-face{font-family:'DejaVu Serif'")
            .map(|(i, _)| {
                let end = i + svg[i..].find('}').unwrap();
                svg[i..end].to_string()
            })
            .collect()
    };
    // No bold text: only the regular face is embedded.
    let regular_only = svg_of(
        scatter_with_legend(),
        Layout::auto_from_plots(&scatter_with_legend()).with_font_family("DejaVu Serif"),
        true,
    );
    let r = rules(&regular_only);
    assert_eq!(r.len(), 1, "{r:?}");
    assert!(r[0].contains("font-weight:400") && r[0].contains("font/ttf"));
    // A bold legend title adds the bold face.
    let with_bold = svg_of(
        scatter_with_legend(),
        Layout::auto_from_plots(&scatter_with_legend())
            .with_font_family("DejaVu Serif")
            .with_legend_title("Series"),
        true,
    );
    let r = rules(&with_bold);
    assert_eq!(r.len(), 2, "{r:?}");
    assert!(r.iter().any(|rule| rule.contains("font-weight:700")));
    // Not embedding: no rules, though the family is still named.
    let not_embedded = svg_of(
        scatter_with_legend(),
        Layout::auto_from_plots(&scatter_with_legend()).with_font_family("DejaVu Serif"),
        false,
    );
    assert!(rules(&not_embedded).is_empty());
    common::write_test_output("test_outputs/custom_font_serif_embedded.svg", with_bold).unwrap();
}

#[test]
fn code_spans_use_the_mono_family() {
    register_serif();
    let note = || -> Vec<Plot> {
        vec![kuva::plot::TextPlot::new()
            .with_body("Run `kuva clustermap` on the table")
            .into()]
    };
    let default = svg_of(note(), Layout::new((0.0, 1.0), (0.0, 1.0)), false);
    assert!(default.contains(r#"font-family="DejaVu Sans Mono,monospace""#));
    let custom = svg_of(
        note(),
        Layout::new((0.0, 1.0), (0.0, 1.0)).with_mono_font_family("DejaVu Serif"),
        false,
    );
    assert!(
        custom.contains(r#"font-family="DejaVu Serif,DejaVu Sans Mono,monospace""#),
        "{custom}"
    );
    // The registered mono family is a registered face in use.
    assert!(custom.contains("font-synthesis:none"));
}

// ── Lazy loading (performance pass after #134) ───────────────────────────────

/// Registering a file reads only the font's header tables; that must describe
/// every face exactly as parsing the whole file does. Covers TrueType outlines
/// (DejaVu) and CFF (`.otf`, New Computer Modern Math).
#[test]
fn header_only_registration_matches_a_full_parse() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for path in [
        serif(),
        serif_bold(),
        root.join("assets/fonts/NewCMMath-Regular.otf"),
    ] {
        let from_file = fonts::register_font_file(&path).unwrap();
        let from_bytes = fonts::register_font_bytes(std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(from_file, from_bytes, "{}", path.display());
    }
    register_serif();
}

/// A collection header claiming billions of faces used to make registration
/// reserve memory for all of them and abort.
#[test]
fn collection_with_a_huge_face_count_is_an_error_not_an_abort() {
    let mut ttc = b"ttcf\x00\x01\x00\x00\xff\xff\xff\xff".to_vec();
    ttc.extend_from_slice(&[0; 16]);
    assert!(fonts::register_font_bytes(ttc.clone()).is_err());
    let dir = scratch("huge_ttc");
    let path = dir.join("huge.ttc");
    std::fs::write(&path, &ttc).unwrap();
    assert!(fonts::register_font_file(&path).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

/// A registered file is read in full only when one of its faces is first used.
/// If it has gone by then, its text measures and draws as DejaVu Sans instead
/// of failing. (A family no other test uses, so this can't disturb them.)
#[test]
fn a_registered_file_removed_before_first_use_falls_back() {
    let dir = scratch("removed_font");
    let path = dir.join("math.otf");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/NewCMMath-Regular.otf");
    std::fs::copy(src, &path).unwrap();
    let infos = fonts::register_font_file(&path).unwrap();
    let family = infos[0].family.clone();
    std::fs::remove_file(&path).unwrap();
    let plots = scatter_with_legend();
    let layout = Layout::auto_from_plots(&plots).with_font_family(family.as_str());
    let svg = svg_of(plots, layout, false);
    assert!(svg.contains("<svg"));
    let _ = std::fs::remove_dir_all(&dir);
}
