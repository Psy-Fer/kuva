use kuva::plot::{UpSetPlot, UpSetSort};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

#[test]
fn test_upset_basic() {
    let up = UpSetPlot::new().with_sets(vec![
        ("Set A", vec!["apple", "banana", "cherry", "date"]),
        ("Set B", vec!["banana", "cherry", "elderberry", "fig"]),
        ("Set C", vec!["cherry", "fig", "grape"]),
    ]);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Basic (3 sets)");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_basic.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    assert!(svg.contains("<rect"));
}

#[test]
fn test_upset_four_sets() {
    let up = UpSetPlot::new().with_sets(vec![
        ("Alpha", vec![1u32, 2, 3, 4, 5, 6]),
        ("Beta",  vec![3, 4, 5, 6, 7, 8]),
        ("Gamma", vec![5, 6, 7, 8, 9, 10]),
        ("Delta", vec![1, 5, 9, 11, 12]),
    ]);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Four Sets");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_four_sets.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_upset_precomputed() {
    // Precomputed with_data: 3 sets, masks and counts provided directly.
    // mask: bit 0 = Set A, bit 1 = Set B, bit 2 = Set C
    let intersections = vec![
        (0b001u64, 10usize), // A only
        (0b010, 8),          // B only
        (0b100, 5),          // C only
        (0b011, 15),         // A ∩ B
        (0b101, 7),          // A ∩ C
        (0b110, 4),          // B ∩ C
        (0b111, 20),         // A ∩ B ∩ C
    ];

    let up = UpSetPlot::new().with_data(
        vec!["Set A", "Set B", "Set C"],
        vec![52usize, 47, 36],
        intersections,
    );

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Precomputed");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_precomputed.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("20")); // largest intersection count should appear
}

#[test]
fn test_upset_sort_degree() {
    let up = UpSetPlot::new()
        .with_sets(vec![
            ("X", vec![1u32, 2, 3, 4, 5]),
            ("Y", vec![3, 4, 5, 6, 7]),
            ("Z", vec![5, 6, 7, 8, 9]),
        ])
        .with_sort(UpSetSort::ByDegree);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Sorted by Degree");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_sort_degree.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_upset_max_visible() {
    let up = UpSetPlot::new()
        .with_sets(vec![
            ("A", vec![1u32, 2, 3, 4, 5, 6, 7]),
            ("B", vec![4, 5, 6, 7, 8, 9, 10]),
            ("C", vec![6, 7, 8, 9, 10, 11, 12]),
            ("D", vec![1, 6, 11, 13, 14]),
        ])
        .with_max_visible(5);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Max 5 Intersections");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_max_visible.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // With 4 sets there are up to 15 non-empty intersections, but we cap at 5 columns.
    // Each column has 4 dots, so at most 5*4 = 20 data circles.
    // (Legend may add more, but there's no size legend here.)
    assert!(svg.contains("<circle"));
}

#[test]
fn test_upset_no_set_sizes() {
    let up = UpSetPlot::new()
        .with_sets(vec![
            ("Genes up",   vec!["BRCA1", "TP53", "EGFR", "MYC"]),
            ("Genes down", vec!["BRCA1", "CDKN2A", "RB1"]),
            ("Mutated",    vec!["TP53", "EGFR", "RB1", "PTEN"]),
        ])
        .without_set_sizes();

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Without Set-Size Bars");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_no_set_sizes.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // "Set size" header should not appear
    assert!(!svg.contains("Set size"));
}

#[test]
fn test_upset_custom_colors() {
    let up = UpSetPlot::new()
        .with_sets(vec![
            ("Treatment A", vec![1u32, 2, 3, 4, 5]),
            ("Treatment B", vec![3, 4, 5, 6, 7]),
            ("Control",     vec![5, 6, 7, 8]),
        ])
        .with_bar_color("#2563eb")
        .with_dot_color("#1e3a8a");

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Custom Colors");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_custom_colors.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("#2563eb"));
    assert!(svg.contains("#1e3a8a"));
}

#[test]
fn test_upset_two_sets() {
    // Minimal: just two sets.
    let up = UpSetPlot::new().with_sets(vec![
        ("Left", vec!["a", "b", "c", "d"]),
        ("Right", vec!["c", "d", "e", "f"]),
    ]);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Two Sets");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_two_sets.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    // 3 intersections × 2 rows = 6 circles minimum
    let circle_count = svg.matches("<circle").count();
    assert!(circle_count >= 6);
}

#[test]
fn test_upset_natural_sort() {
    // with_data + Natural sort preserves input order.
    let up = UpSetPlot::new()
        .with_data(
            vec!["P", "Q", "R"],
            vec![30usize, 25, 20],
            vec![(0b001u64, 5), (0b010, 8), (0b100, 3), (0b011, 12), (0b111, 7)],
        )
        .with_sort(UpSetSort::Natural);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots).with_title("UpSet Natural Sort");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_natural_sort.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
}

#[test]
fn test_upset_large() {
    // 5 sets constructed so every non-empty bitmask has exactly one representative element.
    // Element i (1..=31) belongs to set j iff bit j is set in i.
    // This guarantees all 31 non-empty intersections are populated.
    let set_a: Vec<u32> = (1u32..32).filter(|i| i & 1 != 0).collect();  // 16 elements
    let set_b: Vec<u32> = (1u32..32).filter(|i| i & 2 != 0).collect();
    let set_c: Vec<u32> = (1u32..32).filter(|i| i & 4 != 0).collect();
    let set_d: Vec<u32> = (1u32..32).filter(|i| i & 8 != 0).collect();
    let set_e: Vec<u32> = (1u32..32).filter(|i| i & 16 != 0).collect();

    let up = UpSetPlot::new()
        .with_sets(vec![
            ("Gene set A", set_a),
            ("Gene set B", set_b),
            ("Gene set C", set_c),
            ("Gene set D", set_d),
            ("Gene set E", set_e),
        ])
        .with_max_visible(20);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("UpSet Large (5 gene sets, top 20 intersections)")
        .with_width(900.0)
        .with_height(500.0);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_large.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<circle"));
    // 20 columns × 5 rows = 100 dots (31 non-empty intersections exist, capped at 20)
    let circle_count = svg.matches("<circle").count();
    assert!(circle_count >= 100);
}

#[test]
fn test_upset_title_and_labels() {
    let up = UpSetPlot::new().with_sets(vec![
        ("A", vec![1u32, 2, 3]),
        ("B", vec![2u32, 3, 4]),
    ]);

    let plots = vec![Plot::UpSet(up)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("UpSet With Title");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/upset_title.svg", svg.clone()).unwrap();

    assert!(svg.contains("UpSet With Title"));
    assert!(svg.contains("Intersection size"));
    assert!(svg.contains("Set size"));
}
