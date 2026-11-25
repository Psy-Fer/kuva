//! `$...$` labels with deeply nested or unclosed groups must not overflow the
//! stack or take quadratic time. The lookup-tier lowering (`to_unicode`) and
//! the Typst translation (`to_typst_math`) used to recurse once per nested
//! group, and rescanned to the end of the label for every group opener that
//! had no closer.

mod hostile;

use hostile::bounded;
use kuva::backend::svg::SvgBackend;
use kuva::backend::terminal::TerminalBackend;
use kuva::render::math::{to_typst_math, to_unicode};
use kuva::render::render::{Primitive, Scene, TextAnchor};

/// `levels` copies of `open`, then `x`, then `levels` copies of `close`, as
/// one math region.
fn nested(open: &str, close: &str, levels: usize) -> String {
    format!("${}x{}$", open.repeat(levels), close.repeat(levels))
}

fn deep_labels(levels: usize) -> Vec<String> {
    vec![
        nested("x^{", "}", levels),
        nested("x_{", "}", levels),
        nested("\\frac{", "}{2}", levels),
        nested("\\sqrt{", "}", levels),
        nested("\\sqrt[", "]{y}", levels),
        nested("{", "}", levels),
    ]
}

/// Group openers that are never closed, repeated to about 2 MB.
fn unclosed_labels() -> Vec<String> {
    [
        "x^{",
        "x_{",
        "\\frac{",
        "\\frac{a}",
        "\\sqrt{",
        "\\sqrt[",
        "{",
        "[",
    ]
    .iter()
    .map(|open| format!("${}$", open.repeat((2 << 20) / open.len())))
    .collect()
}

#[test]
fn deeply_nested_math_lowers_on_a_small_stack() {
    for label in deep_labels(100_000) {
        bounded(256 << 10, move || {
            let uni = to_unicode(&label);
            assert!(!uni.contains('\\') && !uni.contains('$'));
            // `to_typst_math` takes the body of one region.
            let typ = to_typst_math(&label[1..label.len() - 1]);
            assert!(!typ.is_empty());
        });
    }
}

#[test]
fn nesting_past_the_limit_is_flattened() {
    // 32 levels are lowered as structure. Past that `\frac` is dropped and its
    // braces stripped, so the innermost 8 fractions become `x22222222`.
    let uni = to_unicode(&nested("\\frac{", "}{2}", 32));
    let want = format!("{}x/2{}", "(".repeat(31), ")/2".repeat(31));
    assert_eq!(uni, want);
    let uni = to_unicode(&nested("\\frac{", "}{2}", 40));
    let want = format!("{}x22222222{}", "(".repeat(32), ")/2".repeat(32));
    assert_eq!(uni, want);
}

#[test]
fn unclosed_group_openers_lower_in_linear_time() {
    // Each opener used to rescan to the end of the label: about 1e12 steps for
    // these. The 10 second limit in `bounded` fails the test if that returns.
    bounded(1 << 20, || {
        for label in unclosed_labels() {
            let uni = to_unicode(&label);
            assert!(!uni.contains('\\'));
            to_typst_math(&label[1..label.len() - 1]);
        }
    });
}

fn text(content: String) -> Primitive {
    Primitive::Text {
        x: 10.0,
        y: 20.0,
        content,
        size: 12,
        anchor: TextAnchor::Start,
        rotate: None,
        bold: false,
        color: None,
    }
}

#[test]
fn deeply_nested_math_labels_render_in_every_backend() {
    for label in deep_labels(20_000) {
        bounded(1 << 20, move || {
            let mut scene = Scene::new(200.0, 100.0);
            scene.add(text(label));
            assert!(SvgBackend.render_scene(&scene).contains("<svg"));
            assert!(!TerminalBackend::new(40, 10).render_scene(&scene).is_empty());
            #[cfg(feature = "png")]
            assert!(!kuva::backend::png::PngBackend::new()
                .render_scene(&scene)
                .unwrap()
                .is_empty());
            #[cfg(feature = "typst")]
            assert!(!kuva::backend::typst::TypstBackend::default()
                .render_scene(&scene)
                .is_empty());
        });
    }
}
