use kuva::backend::terminal::TerminalBackend;
use kuva::plot::scatter::ScatterPlot;
use kuva::plot::BarPlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::{render_multiple, Scene};

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
fn legend_swatches_show_series_color() {
    // Two scatter series with distinct colors and legend labels.
    // The legend swatch for each series should appear in the terminal output
    // as a '█' with the series color, not the legend background color.
    let s1 = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 2.0), (2.0, 4.0), (3.0, 3.0)])
        .with_color("red")
        .with_legend("Red")
        .with_size(6.0);
    let s2 = ScatterPlot::new()
        .with_data(vec![(1.0_f64, 5.0), (2.0, 6.0), (3.0, 7.0)])
        .with_color("blue")
        .with_legend("Blue")
        .with_size(6.0);
    let plots = vec![Plot::Scatter(s1), Plot::Scatter(s2)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let out = TerminalBackend::new(80, 24).render_scene(&scene);

    // Legend background is typically a dark gray (e.g. rgb(45,45,45) → "38;2;45;45;45").
    // The swatch '█' must appear with a non-background color near the label text.
    // Find lines containing "Red" or "Blue" and verify they have a colored '█'
    // that is NOT the legend background gray.
    let legend_bg = "38;2;45;45;45";
    for label in &["Red", "Blue"] {
        let line = out.lines().find(|l| l.contains(label));
        assert!(line.is_some(), "expected legend label '{label}' in output");
        let line = line.unwrap();
        // Extract all ANSI color codes preceding '█' on this line.
        // At least one should differ from the legend background.
        let has_colored_swatch = line
            .match_indices('█')
            .any(|(i, _)| {
                // Look backwards from '█' for the nearest ANSI color code.
                let prefix = &line[..i];
                if let Some(esc_end) = prefix.rfind('m') {
                    if let Some(esc_start) = prefix[..esc_end].rfind("\x1b[") {
                        let code = &prefix[esc_start + 2..esc_end];
                        return code != legend_bg;
                    }
                }
                false
            });
        assert!(
            has_colored_swatch,
            "legend swatch for '{label}' should have a non-background color"
        );
    }
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
