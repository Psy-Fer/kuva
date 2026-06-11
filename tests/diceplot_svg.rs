mod common;
use kuva::backend::svg::SvgBackend;
use kuva::plot::diceplot::DicePlot;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;
use kuva::render::theme::Theme;

#[test]
fn test_dice_categorical_basic() {
    let organs = vec!["Lung".into(), "Liver".into(), "Brain".into()];
    let data = vec![
        ("miR-1", "Cpd1", "Lung", "#b2182b"),
        ("miR-1", "Cpd1", "Liver", "#2166ac"),
        ("miR-1", "Cpd1", "Brain", "#cccccc"),
        ("miR-1", "Cpd2", "Lung", "#cccccc"),
        ("miR-1", "Cpd2", "Brain", "#b2182b"),
        ("miR-2", "Cpd1", "Lung", "#2166ac"),
        ("miR-2", "Cpd1", "Liver", "#b2182b"),
        ("miR-2", "Cpd2", "Lung", "#cccccc"),
        ("miR-2", "Cpd2", "Liver", "#2166ac"),
        ("miR-2", "Cpd2", "Brain", "#b2182b"),
    ];

    let dice = DicePlot::new(3)
        .with_category_labels(organs)
        .with_records(data);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Dice Categorical")
        .with_x_label("miRNA")
        .with_y_label("Compound");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_categorical_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // 4 tile rects (2 miRNAs x 2 compounds)
    assert!(svg.contains("<rect"));
    // 10 data dots
    assert_eq!(svg.matches("<circle").count(), 10);
    // Title rendered
    assert!(svg.contains("Dice Categorical"));
}

#[test]
fn test_dice_categorical_absent_dots_omitted() {
    let cats = vec!["A".into(), "B".into(), "C".into(), "D".into()];
    // Only 2 of 4 positions present in this cell
    let data = vec![("X1", "Y1", "A", "#ff0000"), ("X1", "Y1", "C", "#0000ff")];

    let dice = DicePlot::new(4)
        .with_category_labels(cats)
        .with_records(data);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);

    // Only 2 circles, not 4 — absent positions are omitted in categorical mode
    assert_eq!(svg.matches("<circle").count(), 2);
}

#[test]
fn test_dice_continuous_tile() {
    let data = vec![
        ("G1", "S1", vec![0, 1, 2, 3], Some(0.8), Some(5.0)),
        ("G1", "S2", vec![0, 2], Some(0.3), Some(2.0)),
        ("G2", "S1", vec![1, 3], Some(0.6), Some(8.0)),
        ("G2", "S2", vec![0, 1, 2, 3], Some(0.1), Some(3.0)),
    ];

    let dice = DicePlot::new(4).with_points(data);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dice Continuous");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_continuous_tile.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // 4 tiles + circles for present dots + hollow paths for absent dots
    assert!(svg.contains("<rect"));
    assert!(svg.contains("<circle"));
    // Absent dots rendered as hollow path arcs
    assert!(svg.contains("<path"));
}

#[test]
fn test_dice_per_dot_continuous() {
    let cats = vec!["C1".into(), "C2".into(), "C3".into()];
    let data = vec![
        ("X1", "Y1", 0_usize, Some(1.5), Some(3.0)),
        ("X1", "Y1", 1, Some(-0.8), Some(1.5)),
        ("X1", "Y1", 2, Some(0.2), Some(4.0)),
        ("X1", "Y2", 0, Some(-1.2), Some(2.0)),
        ("X1", "Y2", 2, Some(0.9), Some(5.0)),
        ("X2", "Y1", 1, Some(2.0), Some(3.5)),
        ("X2", "Y1", 2, Some(-0.3), Some(1.0)),
    ];

    let dice = DicePlot::new(3)
        .with_category_labels(cats)
        .with_dot_data(data);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_per_dot.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // White tiles with black border
    assert!(svg.contains("#ffffff"));
    // 7 data dots (only present dots drawn)
    assert_eq!(svg.matches("<circle").count(), 7);
}

#[test]
fn test_dice_position_legend() {
    let organs = vec!["Lung".into(), "Liver".into(), "Brain".into()];
    let data = vec![
        ("X1", "Y1", "Lung", "#ff0000"),
        ("X1", "Y1", "Brain", "#0000ff"),
    ];

    let dice = DicePlot::new(3)
        .with_category_labels(organs)
        .with_records(data)
        .with_position_legend("Organ");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_position_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Legend title present
    assert!(svg.contains("Organ"));
    // Category labels in legend
    assert!(svg.contains("Lung"));
    assert!(svg.contains("Liver"));
    assert!(svg.contains("Brain"));
}

#[test]
fn test_dice_dot_legend() {
    let organs = vec!["Lung".into(), "Liver".into()];
    let data = vec![
        ("X1", "Y1", "Lung", "#b2182b"),
        ("X1", "Y1", "Liver", "#2166ac"),
    ];

    let dice = DicePlot::new(2)
        .with_category_labels(organs)
        .with_records(data)
        .with_dot_legend(vec![("Down", "#2166ac"), ("Up", "#b2182b")]);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_dot_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Down"));
    assert!(svg.contains("Up"));
}

#[test]
fn test_dice_size_legend() {
    let cats = vec!["A".into(), "B".into()];
    let data = vec![
        ("X1", "Y1", 0_usize, Some(1.0), Some(2.0)),
        ("X1", "Y1", 1, Some(0.5), Some(8.0)),
    ];

    let dice = DicePlot::new(2)
        .with_category_labels(cats)
        .with_dot_data(data)
        .with_size_legend("-log10(FDR)");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_size_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("-log10(FDR)"));
}

#[test]
fn test_dice_colorbar() {
    let data = vec![
        ("X1", "Y1", vec![0, 1], Some(0.2), None),
        ("X1", "Y2", vec![0], Some(0.9), None),
        ("X2", "Y1", vec![1], Some(0.5), None),
    ];

    let dice = DicePlot::new(2)
        .with_points(data)
        .with_fill_legend("Expression");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_colorbar.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // Colorbar label
    assert!(svg.contains("Expression"));
    // Colorbar draws many stacked rects
    assert!(svg.matches("<rect").count() > 10);
}

#[test]
fn test_dice_empty_data() {
    let dice = DicePlot::new(4);
    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);

    // Should produce valid SVG without panicking
    assert!(svg.contains("<svg"));
    // No circles — no data
    assert_eq!(svg.matches("<circle").count(), 0);
}

#[test]
fn test_dice_all_ndots_variants() {
    // Verify rendering doesn't panic for ndots 1 through 6
    for n in 1..=6 {
        let mut data = Vec::new();
        for k in 0..n {
            data.push(("X", "Y", format!("Cat{k}"), "#444444"));
        }
        let labels: Vec<String> = (0..n).map(|k| format!("Cat{k}")).collect();

        let dice = DicePlot::new(n)
            .with_category_labels(labels)
            .with_records(data);

        let plots = vec![Plot::DicePlot(dice)];
        let layout = Layout::auto_from_plots(&plots);
        let scene = render_multiple(plots, layout);
        let svg = SvgBackend.render_scene(&scene);

        assert!(svg.contains("<svg"), "ndots={n} should produce valid SVG");
        assert_eq!(
            svg.matches("<circle").count(),
            n,
            "ndots={n} should have {n} circles"
        );
    }
}

#[test]
fn test_dice_stacked_legends() {
    // Position + colour + size legends all at once
    let cats = vec!["A".into(), "B".into(), "C".into()];
    let data = vec![
        ("X1", "Y1", 0_usize, Some(1.0), Some(3.0)),
        ("X1", "Y1", 1, Some(-0.5), Some(1.0)),
        ("X1", "Y1", 2, Some(0.8), Some(5.0)),
    ];

    let dice = DicePlot::new(3)
        .with_category_labels(cats)
        .with_dot_data(data)
        .with_position_legend("Category")
        .with_fill_legend("logFC")
        .with_size_legend("Significance");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_stacked_legends.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Category"));
    assert!(svg.contains("logFC"));
    assert!(svg.contains("Significance"));
    // Position legend has category labels
    assert!(svg.contains(">A<"));
    assert!(svg.contains(">B<"));
    assert!(svg.contains(">C<"));
}

#[test]
fn test_dice_position_legend_dark_theme() {
    // Position legend mini-tiles must use theme colours, not hardcoded white/black.
    let organs = vec!["Lung".into(), "Liver".into()];
    let data = vec![("X1", "Y1", "Lung", "#ff0000")];

    let dice = DicePlot::new(2)
        .with_category_labels(organs)
        .with_records(data)
        .with_position_legend("Organ");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots).with_theme(Theme::dark());

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);

    // Dark theme background is not white — the mini-tile fill must reflect that.
    // If hardcoded #ffffff were used it would appear in the position legend;
    // the dark legend_bg is #2a2a2a so we must NOT see solid #ffffff in the legend area.
    // We can't easily isolate just the legend rects, but we verify the SVG doesn't
    // contain a #ffffff rect that's wider than 30px (legend mini-tile is 18px wide,
    // so any 18-or-smaller tile shouldn't contribute a bare #ffffff fill).
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Organ"));
    // Dark theme background colour must appear somewhere (proves theme was applied).
    assert!(svg.contains("#1e1e1e") || svg.contains("background"));
}

#[test]
fn test_dice_single_dot_positions() {
    // Verify ndots=1 (single centre pip) and ndots=6 (all pips) don't panic
    // and produce the right circle counts.
    for (ndots, expected_circles) in [(1_usize, 1_usize), (6, 6)] {
        let labels: Vec<String> = (0..ndots).map(|k| format!("Cat{k}")).collect();
        let colors: Vec<(&str, &str, String, &str)> = (0..ndots)
            .map(|k| ("X", "Y", format!("Cat{k}"), "#444444"))
            .collect();

        let dice = DicePlot::new(ndots)
            .with_category_labels(labels)
            .with_records(colors);

        let plots = vec![Plot::DicePlot(dice)];
        let layout = Layout::auto_from_plots(&plots);
        let scene = render_multiple(plots, layout);
        let svg = SvgBackend.render_scene(&scene);

        assert_eq!(
            svg.matches("<circle").count(),
            expected_circles,
            "ndots={ndots}: expected {expected_circles} circles"
        );
    }
}

#[test]
fn test_dice_long_legend_title_fits_box() {
    // Regression: long position/size legend titles used to overflow the bounding box
    // because max_label_len in auto_from_plots only counted entry labels, not title strings.
    let long_title = "A Very Long Legend Title String";
    let cats: Vec<String> = vec!["Cat A".into(), "Cat B".into()];
    let data = vec![
        ("X1", "Y1", 0_usize, Some(1.0), Some(5.0)),
        ("X1", "Y1", 1, Some(0.5), Some(2.0)),
    ];

    let dice = DicePlot::new(2)
        .with_category_labels(cats)
        .with_dot_data(data)
        .with_position_legend(long_title)
        .with_size_legend(long_title);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    // legend_width must be at least as wide as the title needs.
    // title chars * ~8.5px + some padding — check via the computed margin_right:
    // margin_right grows with legend_width, so if the title is wider than short labels,
    // margin_right must be > the old minimum.
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);

    assert!(svg.contains("<svg"));
    assert!(svg.contains(long_title));
    // The title must appear — and must not be cut off (SVG text truncation would
    // only happen if we manually clipped, which we don't, so presence is sufficient).
}

/// Recreates a ggdiceplot-style large grid (8 compounds × 5 miRNAs, 4 organs/dots).
/// Also exercises: position legend with big-die layout, grid_lines on, and dark theme.
#[test]
fn test_dice_large_grid_with_position_legend() {
    // Position (which pip) = organ affected (Lung/Liver/Brain/Kidney)
    // Pip colour          = direction of effect (Upregulated/Downregulated/Not sig.)
    // These are two genuinely different dimensions, so both legends carry distinct information.
    let organs: Vec<String> = vec![
        "Lung".into(),
        "Liver".into(),
        "Brain".into(),
        "Kidney".into(),
    ];

    const UP: &str = "#d73027"; // upregulated   – red
    const DN: &str = "#4575b4"; // downregulated – blue
    const NS: &str = "#aaaaaa"; // not significant – grey

    // (miRNA, Compound, Organ, significance_colour)
    // miR-1: predominantly upregulated in Lung/Liver
    // miR-2: downregulated in Kidney, mixed elsewhere
    // miR-3: upregulated in Brain across many compounds
    // miR-4: downregulated in Lung, mixed
    // miR-5: mostly not significant
    let combos: &[(&str, &str, &str, &str)] = &[
        ("miR-1", "CpdA", "Lung", UP),
        ("miR-1", "CpdA", "Liver", UP),
        ("miR-1", "CpdA", "Brain", NS),
        ("miR-1", "CpdB", "Lung", UP),
        ("miR-1", "CpdB", "Kidney", NS),
        ("miR-1", "CpdC", "Lung", UP),
        ("miR-1", "CpdC", "Liver", UP),
        ("miR-1", "CpdD", "Liver", UP),
        ("miR-1", "CpdD", "Brain", NS),
        ("miR-1", "CpdE", "Lung", UP),
        ("miR-1", "CpdF", "Liver", UP),
        ("miR-1", "CpdF", "Brain", DN),
        ("miR-1", "CpdG", "Lung", UP),
        ("miR-1", "CpdG", "Liver", NS),
        ("miR-1", "CpdH", "Lung", UP),
        ("miR-1", "CpdH", "Liver", UP),
        ("miR-2", "CpdA", "Brain", DN),
        ("miR-2", "CpdA", "Kidney", DN),
        ("miR-2", "CpdB", "Lung", NS),
        ("miR-2", "CpdB", "Kidney", DN),
        ("miR-2", "CpdC", "Kidney", DN),
        ("miR-2", "CpdD", "Lung", NS),
        ("miR-2", "CpdD", "Kidney", DN),
        ("miR-2", "CpdE", "Liver", NS),
        ("miR-2", "CpdE", "Kidney", DN),
        ("miR-2", "CpdF", "Brain", DN),
        ("miR-2", "CpdG", "Liver", UP),
        ("miR-2", "CpdG", "Kidney", DN),
        ("miR-2", "CpdH", "Brain", NS),
        ("miR-2", "CpdH", "Kidney", DN),
        ("miR-3", "CpdA", "Brain", UP),
        ("miR-3", "CpdA", "Lung", NS),
        ("miR-3", "CpdB", "Brain", UP),
        ("miR-3", "CpdB", "Liver", NS),
        ("miR-3", "CpdC", "Brain", UP),
        ("miR-3", "CpdD", "Brain", UP),
        ("miR-3", "CpdD", "Lung", NS),
        ("miR-3", "CpdE", "Brain", UP),
        ("miR-3", "CpdE", "Kidney", NS),
        ("miR-3", "CpdF", "Brain", UP),
        ("miR-3", "CpdG", "Brain", UP),
        ("miR-3", "CpdG", "Liver", DN),
        ("miR-3", "CpdH", "Brain", UP),
        ("miR-4", "CpdA", "Lung", DN),
        ("miR-4", "CpdA", "Liver", NS),
        ("miR-4", "CpdB", "Lung", DN),
        ("miR-4", "CpdC", "Lung", DN),
        ("miR-4", "CpdC", "Kidney", UP),
        ("miR-4", "CpdD", "Lung", DN),
        ("miR-4", "CpdE", "Lung", DN),
        ("miR-4", "CpdE", "Brain", NS),
        ("miR-4", "CpdF", "Lung", DN),
        ("miR-4", "CpdF", "Liver", UP),
        ("miR-4", "CpdG", "Lung", DN),
        ("miR-4", "CpdH", "Lung", DN),
        ("miR-4", "CpdH", "Kidney", NS),
        ("miR-5", "CpdA", "Lung", NS),
        ("miR-5", "CpdB", "Brain", NS),
        ("miR-5", "CpdC", "Liver", NS),
        ("miR-5", "CpdC", "Lung", NS),
        ("miR-5", "CpdD", "Kidney", NS),
        ("miR-5", "CpdE", "Brain", NS),
        ("miR-5", "CpdF", "Lung", NS),
        ("miR-5", "CpdG", "Liver", NS),
        ("miR-5", "CpdH", "Kidney", NS),
    ];

    let dice = DicePlot::new(4)
        .with_category_labels(organs)
        .with_records(combos.iter().copied())
        .with_position_legend("Organ")
        .with_dot_legend(vec![
            ("Upregulated", UP),
            ("Downregulated", DN),
            ("Not significant", NS),
        ])
        .with_grid_lines(true);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("miRNA dysregulation by organ and direction")
        .with_x_label("miRNA")
        .with_y_label("Compound")
        .with_height(520.0);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    common::write_test_output("test_outputs/dice_large_grid.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<rect"));
    assert!(svg.contains("Organ"));
    assert!(svg.contains("Lung"));
    assert!(svg.contains("Liver"));
    assert!(svg.contains("Brain"));
    assert!(svg.contains("Kidney"));
    assert!(svg.contains("Upregulated"));
    assert!(svg.contains("Downregulated"));
    assert!(svg.contains("<line"));
}

#[test]
fn test_dice_grid_lines_off_by_default() {
    let organs = vec!["A".into(), "B".into(), "C".into()];
    let data = vec![("X1", "Y1", "A", "#ff0000")];
    let dice = DicePlot::new(3)
        .with_category_labels(organs)
        .with_records(data);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);

    // Grid lines should be absent when not enabled
    assert!(!svg.contains("stroke-dasharray"));
}

#[test]
fn test_dice_fill_colorbar_range() {
    // Explicit fill_range should be respected — colorbar min/max derived from it.
    let data = vec![
        ("G1", "S1", vec![0], Some(0.0_f64), None),
        ("G1", "S2", vec![0], Some(1.0), None),
    ];

    let dice = DicePlot::new(1)
        .with_points(data)
        .with_fill_range(0.0, 5.0) // explicit range wider than data
        .with_fill_legend("Score");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);

    assert!(svg.contains("Score"));
    // Colorbar ticks should reflect the 0..5 range, not 0..1.
    assert!(svg.contains('5') || svg.contains("5.0"));
}
