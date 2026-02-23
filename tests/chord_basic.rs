use visus::plot::ChordPlot;
use visus::render::{plots::Plot, layout::Layout, render::render_multiple};
use visus::backend::svg::SvgBackend;

#[test]
fn chord_basic() {
    let matrix = vec![
        vec![0.0, 10.0, 5.0],
        vec![10.0, 0.0, 8.0],
        vec![5.0, 8.0, 0.0],
    ];
    let chord = ChordPlot::new()
        .with_matrix(matrix)
        .with_labels(["A", "B", "C"])
        .with_opacity(0.7);
    let plots = vec![Plot::Chord(chord)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Chord Diagram");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/chord_basic.svg", svg).unwrap();
}

#[test]
fn chord_asymmetric() {
    let matrix = vec![
        vec![0.0, 15.0, 3.0, 7.0],
        vec![5.0, 0.0, 12.0, 0.0],
        vec![10.0, 4.0, 0.0, 6.0],
        vec![2.0, 8.0, 1.0, 0.0],
    ];
    let chord = ChordPlot::new()
        .with_matrix(matrix)
        .with_labels(["Alpha", "Beta", "Gamma", "Delta"])
        .with_colors(["#e41a1c", "#377eb8", "#4daf4a", "#984ea3"])
        .with_gap(3.0)
        .with_opacity(0.65)
        .with_legend("Flows");
    let plots = vec![Plot::Chord(chord)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Asymmetric Chord Diagram");
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    std::fs::create_dir_all("test_outputs").unwrap();
    std::fs::write("test_outputs/chord_asymmetric.svg", svg).unwrap();
}
