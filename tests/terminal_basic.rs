use visus::backend::terminal::TerminalBackend;
use visus::plot::scatter::ScatterPlot;
use visus::plot::BarPlot;
use visus::render::layout::Layout;
use visus::render::plots::Plot;
use visus::render::render::{render_multiple, Scene};

#[test]
fn scatter_renders_non_empty() {
    // Use large marker size so circles are guaranteed to produce braille dots
    // at 80×24 resolution against an 800×500 scene.
    let scatter = ScatterPlot::new()
        .with_data(vec![
            (1.0_f64, 2.0), (2.0, 4.0), (3.0, 1.0), (4.0, 5.0),
            (5.0, 3.0),     (1.5, 4.5), (3.5, 2.5), (2.5, 1.5),
        ])
        .with_color("steelblue")
        .with_size(8.0);
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let out = TerminalBackend::new(80, 24).render_scene(&scene);
    assert!(!out.trim().is_empty());
    // Must contain braille characters (U+2800..U+28FF) from the scatter dots.
    assert!(
        out.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c)),
        "expected braille characters in scatter output"
    );
}

#[test]
fn bar_renders_block_chars() {
    let bar = BarPlot::new()
        .with_bar("A", 10.0)
        .with_bar("B", 20.0)
        .with_bar("C", 15.0);
    let plots = vec![Plot::Bar(bar)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let out = TerminalBackend::new(80, 24).render_scene(&scene);
    assert!(out.contains('█'), "expected block characters in bar output");
}

#[test]
fn text_labels_present() {
    let scatter = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (3.0, 4.0)])
        .with_color("steelblue");
    let plots = vec![Plot::Scatter(scatter)];
    let layout = Layout::auto_from_plots(&plots).with_title("Hi");
    let scene = render_multiple(plots, layout);
    let out = TerminalBackend::new(80, 24).render_scene(&scene);
    // Title characters 'H' and 'i' should appear somewhere in the output.
    assert!(
        out.contains('H') || out.contains('i'),
        "expected title text in output"
    );
}

#[test]
fn auto_size_default() {
    let scene = Scene::new(800.0, 500.0);
    let out = TerminalBackend::new(80, 24).render_scene(&scene);
    // An empty scene must still produce non-empty output (rows of spaces + newlines).
    assert!(!out.is_empty());
    // Should have exactly 24 newlines (one per row).
    assert_eq!(
        out.chars().filter(|&c| c == '\n').count(),
        24,
        "expected 24 newlines for 24-row terminal"
    );
}
