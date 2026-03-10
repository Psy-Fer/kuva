use kuva::backend::svg::SvgBackend;
use kuva::render::render::{Scene, Primitive, TextAnchor};

fn minimal_scene() -> Scene {
    let mut scene = Scene::new(200.0, 100.0);
    scene.add(Primitive::Circle { cx: 50.0, cy: 50.0, r: 10.0, fill: "red".into() });
    scene.add(Primitive::Text {
        x: 100.0,
        y: 50.0,
        content: "hello".to_string(),
        size: 12,
        anchor: TextAnchor::Middle,
        rotate: None,
        bold: false,
    });
    scene
}

#[test]
fn test_svg_compact() {
    let scene = minimal_scene();
    let svg = SvgBackend::new().render_scene(&scene);

    // Compact mode: no newlines inside the element body (between <svg …> and </svg>)
    let body_start = svg.find('>').expect("<svg> opening tag must be present") + 1;
    let body_end = svg.rfind('<').expect("</svg> closing tag must be present");
    let body = &svg[body_start..body_end];
    assert!(
        !body.contains('\n'),
        "compact SVG body must contain no newlines; got: {:?}",
        body
    );
}

#[test]
fn test_svg_pretty() {
    let scene = minimal_scene();
    let svg = SvgBackend::new().with_pretty(true).render_scene(&scene);

    // Pretty mode: elements are indented with "  " (2 spaces at depth 1)
    assert!(
        svg.contains("\n  <"),
        "pretty SVG must contain newline-indented elements; got:\n{}",
        svg
    );
}

#[test]
fn test_svg_pretty_groups() {
    // Verify that GroupStart increments depth and GroupEnd decrements it.
    let mut scene = Scene::new(200.0, 100.0);
    scene.add(Primitive::GroupStart { transform: Some("translate(10,10)".to_string()) });
    scene.add(Primitive::Circle { cx: 5.0, cy: 5.0, r: 3.0, fill: "blue".into() });
    scene.add(Primitive::GroupEnd);

    let svg = SvgBackend::new().with_pretty(true).render_scene(&scene);

    // Outer <g> is at depth 1 (2 spaces); child circle is at depth 2 (4 spaces).
    assert!(svg.contains("\n  <g "), "group open must be indented at depth 1");
    assert!(svg.contains("\n    <circle"), "child of group must be indented at depth 2");
    assert!(svg.contains("\n  </g>"), "group close must be indented at depth 1");
}

#[test]
fn test_svg_compat_shim() {
    // Old call-site style: `SvgBackend.render_scene(...)` using the const shim.
    // This must compile and produce the same output as SvgBackend::new().render_scene(...).
    let scene = minimal_scene();
    #[allow(non_upper_case_globals)]
    let svg_via_shim = SvgBackend.render_scene(&scene);
    let svg_via_new  = SvgBackend::new().render_scene(&scene);
    assert_eq!(svg_via_shim, svg_via_new);
}
