use kuva::plot::diceplot::DicePlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

#[test]
fn test_dice_categorical_basic() {
    let organs = vec!["Lung".into(), "Liver".into(), "Brain".into()];
    let data = vec![
        ("miR-1", "Cpd1", "Lung",  "#b2182b"),
        ("miR-1", "Cpd1", "Liver", "#2166ac"),
        ("miR-1", "Cpd1", "Brain", "#cccccc"),
        ("miR-1", "Cpd2", "Lung",  "#cccccc"),
        ("miR-1", "Cpd2", "Brain", "#b2182b"),
        ("miR-2", "Cpd1", "Lung",  "#2166ac"),
        ("miR-2", "Cpd1", "Liver", "#b2182b"),
        ("miR-2", "Cpd2", "Lung",  "#cccccc"),
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
    std::fs::write("test_outputs/dice_categorical_basic.svg", svg.clone()).unwrap();

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
    let data = vec![
        ("X1", "Y1", "A", "#ff0000"),
        ("X1", "Y1", "C", "#0000ff"),
    ];

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
        ("G1", "S2", vec![0, 2],       Some(0.3), Some(2.0)),
        ("G2", "S1", vec![1, 3],       Some(0.6), Some(8.0)),
        ("G2", "S2", vec![0, 1, 2, 3], Some(0.1), Some(3.0)),
    ];

    let dice = DicePlot::new(4).with_points(data);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots).with_title("Dice Continuous");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dice_continuous_tile.svg", svg.clone()).unwrap();

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
        ("X1", "Y1", 0_usize, Some(1.5),  Some(3.0)),
        ("X1", "Y1", 1,       Some(-0.8), Some(1.5)),
        ("X1", "Y1", 2,       Some(0.2),  Some(4.0)),
        ("X1", "Y2", 0,       Some(-1.2), Some(2.0)),
        ("X1", "Y2", 2,       Some(0.9),  Some(5.0)),
        ("X2", "Y1", 1,       Some(2.0),  Some(3.5)),
        ("X2", "Y1", 2,       Some(-0.3), Some(1.0)),
    ];

    let dice = DicePlot::new(3)
        .with_category_labels(cats)
        .with_dot_data(data);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dice_per_dot.svg", svg.clone()).unwrap();

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
    std::fs::write("test_outputs/dice_position_legend.svg", svg.clone()).unwrap();

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
        ("X1", "Y1", "Lung",  "#b2182b"),
        ("X1", "Y1", "Liver", "#2166ac"),
    ];

    let dice = DicePlot::new(2)
        .with_category_labels(organs)
        .with_records(data)
        .with_dot_legend(vec![
            ("Down", "#2166ac"),
            ("Up",   "#b2182b"),
        ]);

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dice_dot_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Down"));
    assert!(svg.contains("Up"));
}

#[test]
fn test_dice_size_legend() {
    let cats = vec!["A".into(), "B".into()];
    let data = vec![
        ("X1", "Y1", 0_usize, Some(1.0), Some(2.0)),
        ("X1", "Y1", 1,       Some(0.5), Some(8.0)),
    ];

    let dice = DicePlot::new(2)
        .with_category_labels(cats)
        .with_dot_data(data)
        .with_size_legend("-log10(FDR)");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dice_size_legend.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("-log10(FDR)"));
}

#[test]
fn test_dice_colorbar() {
    let data = vec![
        ("X1", "Y1", vec![0, 1], Some(0.2), None),
        ("X1", "Y2", vec![0],    Some(0.9), None),
        ("X2", "Y1", vec![1],    Some(0.5), None),
    ];

    let dice = DicePlot::new(2)
        .with_points(data)
        .with_fill_legend("Expression");

    let plots = vec![Plot::DicePlot(dice)];
    let layout = Layout::auto_from_plots(&plots);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/dice_colorbar.svg", svg.clone()).unwrap();

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
        assert_eq!(svg.matches("<circle").count(), n, "ndots={n} should have {n} circles");
    }
}

#[test]
fn test_dice_stacked_legends() {
    // Position + colour + size legends all at once
    let cats = vec!["A".into(), "B".into(), "C".into()];
    let data = vec![
        ("X1", "Y1", 0_usize, Some(1.0), Some(3.0)),
        ("X1", "Y1", 1,       Some(-0.5), Some(1.0)),
        ("X1", "Y1", 2,       Some(0.8), Some(5.0)),
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
    std::fs::write("test_outputs/dice_stacked_legends.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("Category"));
    assert!(svg.contains("logFC"));
    assert!(svg.contains("Significance"));
    // Position legend has category labels
    assert!(svg.contains(">A<"));
    assert!(svg.contains(">B<"));
    assert!(svg.contains(">C<"));
}
