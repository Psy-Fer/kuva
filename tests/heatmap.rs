use visus::plot::{Heatmap, ColorMap};
use visus::backend::svg::SvgBackend;
use visus::render::render::render_multiple;
use visus::render::layout::Layout;
use visus::render::plots::Plot;


#[test]
fn test_boxplot_groups_svg_output_builder() {


    let data = vec![
        vec![10.0, 20.0, 30.0],
        vec![4.0, 50.0, 6.0],
        vec![7.0, 8.0, 90.0],
    ];

    let heatmap = Heatmap::new()
                        .with_data(data)
                        .show_values()
                        // .with_color_map(ColorMap::Grayscale);
                        .with_color_map(ColorMap::Viridis);
                        // .with_color_map(ColorMap::Inferno);

    
    let plots = vec![Plot::Heatmap(heatmap)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Heatmap");
        // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/heatmap_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}
