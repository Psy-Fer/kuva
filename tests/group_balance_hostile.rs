//! Every `<g>` the SVG backend writes must be closed exactly once.
//!
//! The vertical bar branches opened a tooltip group when a bar had a tooltip
//! label or the plot was interactive, but closed it when `with_tooltips()` was
//! set or the plot was interactive. Custom labels without `with_tooltips()`, or
//! fewer labels than bars, left groups open or closed groups that were never
//! opened. The SVG backend then took its depth counter below zero: a panic with
//! overflow checks, or an indent loop that ran out of memory in pretty mode.

mod hostile;

use kuva::backend::svg::SvgBackend;
use kuva::plot::BarPlot;
use kuva::render::render::{Primitive, Scene};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};

/// Checks that `<g>` opening and closing tags nest correctly, ignoring
/// script blocks.
fn assert_groups_balanced(svg: &str, what: &str) {
    let mut rest = svg;
    let mut text = String::new();
    while let Some(start) = rest.find("<![CDATA[") {
        text.push_str(&rest[..start]);
        let end = rest[start..].find("]]>").expect("unclosed CDATA") + start + 3;
        rest = &rest[end..];
    }
    text.push_str(rest);

    let mut depth = 0i64;
    let mut i = 0;
    while let Some(off) = text[i..].find('<') {
        i += off;
        let tag = &text[i..];
        if tag.starts_with("</g>") {
            depth -= 1;
            assert!(depth >= 0, "{what}: </g> with no open <g>");
        } else if tag.starts_with("<g>") || tag.starts_with("<g ") {
            depth += 1;
        }
        i += 1;
    }
    assert_eq!(depth, 0, "{what}: {depth} <g> left open");
    assert!(text.trim_end().ends_with("</svg>"), "{what}");
}

/// Checks that the scene closes every group and clip it opens, in order.
fn assert_scene_balanced(scene: &Scene, what: &str) {
    let mut depth = 0i64;
    for el in &scene.elements {
        match el {
            Primitive::GroupStart { .. } | Primitive::ClipStart { .. } => depth += 1,
            Primitive::GroupEnd | Primitive::ClipEnd => {
                depth -= 1;
                assert!(depth >= 0, "{what}: end with nothing open");
            }
            _ => {}
        }
    }
    assert_eq!(depth, 0, "{what}: {depth} left open");
}

fn bars(stacked: bool, horizontal: bool) -> BarPlot {
    let mut b = BarPlot::new()
        .with_group("a", vec![(3.0, "steelblue"), (2.0, "crimson")])
        .with_group("b", vec![(4.0, "steelblue"), (1.0, "crimson")])
        .with_group("c", vec![(5.0, "steelblue"), (2.5, "crimson")])
        .with_legend(vec!["s1", "s2"])
        .with_horizontal(horizontal);
    if stacked {
        b = b.with_stacked();
    }
    b
}

#[test]
fn bar_tooltip_groups_are_balanced_for_any_label_count() {
    // Six bars: no labels, fewer, as many and more labels than bars.
    let label_counts = [None, Some(0), Some(2), Some(6), Some(9)];
    for stacked in [false, true] {
        for horizontal in [false, true] {
            for show in [false, true] {
                for labels in label_counts {
                    for interactive in [false, true] {
                        let mut b = bars(stacked, horizontal);
                        if show {
                            b = b.with_tooltips();
                        }
                        if let Some(n) = labels {
                            b = b.with_tooltip_labels((0..n).map(|i| format!("tip{i}")));
                        }
                        let plots = vec![Plot::Bar(b)];
                        let mut layout = Layout::auto_from_plots(&plots);
                        if interactive {
                            layout = layout.with_interactive();
                        }
                        let scene = render_multiple(plots, layout);
                        let what = format!(
                            "stacked {stacked} horizontal {horizontal} show {show} \
                             labels {labels:?} interactive {interactive}"
                        );
                        // The backend now repairs an unbalanced scene, so check
                        // the scene itself.
                        assert_scene_balanced(&scene, &what);
                        for pretty in [false, true] {
                            let svg = SvgBackend::new().with_pretty(pretty).render_scene(&scene);
                            assert_groups_balanced(&svg, &what);
                        }
                        // Labels given without `with_tooltips()` still show,
                        // as they did before.
                        if labels.is_some_and(|n| n > 0) {
                            let svg = SvgBackend::new().render_scene(&scene);
                            assert!(svg.contains("<title>tip0</title>"), "{what}");
                        }
                    }
                }
            }
        }
    }
}

fn group() -> Primitive {
    Primitive::GroupStart {
        transform: None,
        title: Some("t".into()),
        extra_attrs: None,
    }
}

fn clip() -> Primitive {
    Primitive::ClipStart {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        id: "c".into(),
    }
}

#[test]
fn svg_backend_skips_surplus_ends_and_closes_open_groups() {
    use Primitive::{ClipEnd, GroupEnd};
    let cases: Vec<(&str, Vec<Primitive>, usize)> = vec![
        (
            "surplus ends",
            vec![GroupEnd, ClipEnd, GroupEnd, GroupEnd],
            0,
        ),
        (
            "one group, three ends",
            vec![group(), GroupEnd, GroupEnd, ClipEnd],
            1,
        ),
        (
            "clip then surplus",
            vec![clip(), ClipEnd, ClipEnd, GroupEnd],
            1,
        ),
        ("missing ends", vec![group(), clip(), group()], 3),
        (
            "mixed",
            vec![
                GroupEnd,
                group(),
                group(),
                GroupEnd,
                ClipEnd,
                GroupEnd,
                group(),
            ],
            3,
        ),
    ];
    for (what, elements, opened) in cases {
        let what = what.to_string();
        // Old code panicked on the underflow, or in a release build looped
        // writing indentation until memory ran out.
        hostile::bounded(8 << 20, move || {
            let mut scene = Scene::new(50.0, 50.0);
            scene.elements = elements;
            for pretty in [false, true] {
                let svg = SvgBackend::new().with_pretty(pretty).render_scene(&scene);
                assert_groups_balanced(&svg, &what);
                assert_eq!(svg.matches("</g>").count(), opened, "{what}");
            }
        });
    }
}
