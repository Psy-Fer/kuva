use visus::plot::BarPlot;
use visus::backend::svg::SvgBackend;
use visus::render::{render_multiple, render_bar_categories, Layout, Plot};

#[test]
fn test_bar_svg_output_builder() {
    let bar = BarPlot::new()
                        .with_group("A", 3.2)
                        .with_group("B", 4.7)
                        .with_group("Longform_C", 2.8)
                        .with_color("orange");
    
    let plots = vec![Plot::Bar(bar)];

    let layout = Layout::auto_from_plots(&plots)
                        .with_title("Exciting Bar Plot")
                        .with_y_label("Value");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}
// #[test]
// fn test_bar_categories_svg_output_builder() {
//     let bars = BarPlot::with_categories_and_values(
//         vec!["team-kill", "Okay", "Lovely"],
//         vec![2.5, 3.5, 4.0]
//         )
//         .with_color("darkgreen");
    
//     let layout = Layout::new((0.0, 5.0), (0.0, 5.0))
//                         .with_title("Grenade Stats")
//                         .with_y_label("Lobbed nades");

//     let scene = render_bar_categories(&bars, layout);
//     let svg = SvgBackend.render_scene(&scene);
//     std::fs::write("test_outputs/bar_categories_builder.svg", svg.clone()).unwrap();

//     // Basic sanity assertion
//     assert!(svg.contains("<svg"));
// }