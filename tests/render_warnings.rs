//! Things rendering changes or leaves out are recorded on `Scene::warnings`
//! (they used to go straight to stderr, out of reach of library callers),
//! carried up from `Figure` panels and printed by the CLI.

use std::io::Write;
use std::process::{Command, Stdio};

use kuva::backend::svg::SvgBackend;
use kuva::plot::{BarPlot, BoxPlot, ScatterPlot};
use kuva::render::figure::Figure;
use kuva::render::render::{render_multiple, RenderWarning, Scene};
use kuva::render::{layout::Layout, plots::Plot};

fn codes(scene: &Scene) -> Vec<&'static str> {
    scene.warnings.iter().map(|w| w.code).collect()
}

fn bars() -> Vec<Plot> {
    vec![Plot::Bar(
        BarPlot::new().with_bar("a", 1.0).with_bar("b", 2.0),
    )]
}

#[test]
fn a_clean_render_has_no_warnings() {
    let plots = bars();
    let layout = Layout::auto_from_plots(&plots);
    assert!(render_multiple(plots, layout).warnings.is_empty());
}

#[test]
fn an_ignored_axis_break_is_recorded() {
    // The break lies above the data, so it can't apply.
    let plots = bars();
    let layout = Layout::auto_from_plots(&plots).with_y_break(5.0, 50.0);
    let scene = render_multiple(plots, layout);
    assert_eq!(codes(&scene), ["axis_break.ignored"]);
    assert!(scene.warnings[0]
        .message
        .starts_with("y-axis break ignored"));
    assert_eq!(scene.warnings[0].to_string(), scene.warnings[0].message);
}

#[test]
fn points_inside_a_break_are_counted() {
    let scatter =
        ScatterPlot::new().with_data([(0.0, 1.0), (1.0, 50.0), (2.0, 60.0), (3.0, 900.0)]);
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots).with_y_break(10.0, 800.0);
    let scene = render_multiple(plots, layout);
    assert_eq!(codes(&scene), ["axis_break.points_in_gap"]);
    assert!(scene.warnings[0].message.starts_with("2 point(s)"));
}

#[test]
fn bracket_overflow_is_recorded() {
    let boxes = || {
        vec![Plot::Box(
            BoxPlot::new()
                .with_group("A", vec![1.0, 2.0, 3.0, 4.0, 5.0])
                .with_group("B", vec![4.0, 5.0, 6.0, 7.0, 8.0])
                .with_group("C", vec![2.0, 3.0, 4.0, 5.0, 6.0])
                .with_group("D", vec![3.0, 4.0, 5.0, 6.0, 7.0]),
        )]
    };
    let layout = |max| {
        Layout::auto_from_plots(&boxes())
            .with_significance("A", "B", 0.04)
            .with_significance("A", "C", 0.004)
            .with_significance("A", "D", 0.0004)
            .with_y_axis_max(max)
    };
    assert!(render_multiple(boxes(), layout(12.0)).warnings.is_empty());
    let scene = render_multiple(boxes(), layout(8.0));
    assert_eq!(codes(&scene), ["brackets.overflow"]);
}

#[test]
fn figure_panels_pass_warnings_up_once_each() {
    let broken = || {
        let plots = bars();
        Layout::auto_from_plots(&plots).with_y_break(5.0, 50.0)
    };
    let scatter = || {
        vec![Plot::Scatter(ScatterPlot::new().with_data([
            (0.0, 1.0),
            (1.0, 50.0),
            (2.0, 900.0),
        ]))]
    };
    let gap = Layout::auto_from_plots(&scatter()).with_y_break(10.0, 800.0);
    let scene = Figure::new(1, 3)
        .with_plots(vec![bars(), bars(), scatter()])
        .with_layouts(vec![broken(), broken(), gap])
        .render();
    // Two panels hit the same ignored break: listed once.
    assert_eq!(
        codes(&scene),
        ["axis_break.ignored", "axis_break.points_in_gap"]
    );
}

#[test]
fn warn_keeps_one_copy_of_identical_warnings() {
    let mut scene = Scene::new(10.0, 10.0);
    scene.warn("x.one", "first");
    scene.warn("x.one", "first");
    scene.warn("x.one", "other text");
    scene.warn("x.two", "first");
    assert_eq!(
        scene.warnings,
        [
            RenderWarning::new("x.one", "first"),
            RenderWarning::new("x.one", "other text"),
            RenderWarning::new("x.two", "first"),
        ]
    );
    // Warnings don't change the drawing.
    assert!(SvgBackend.render_scene(&scene).contains("<svg"));
}

#[test]
fn cli_prints_warnings_on_stderr_only() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .args([
            "bar",
            "--label-col",
            "0",
            "--value-col",
            "1",
            "--y-break",
            "5,50",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kuva");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"name\tv\na\t1\nb\t2\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert_eq!(
        stderr.matches("warning: y-axis break ignored").count(),
        1,
        "{stderr}"
    );
    assert!(stdout.starts_with("<svg") && !stdout.contains("warning"));
}
