use visus::plot::BarPlot;
use visus::backend::svg::SvgBackend;
use visus::render::{render_bar_categories, Layout};

// #[test]
// fn test_bar_svg_output_builder() {
//     let bars = BarPlot::new()
//                         .with_data(vec![(1.0, 3.0), (2.0, 5.0), (3.0, 2.0)])
//                         .with_color("orange");

//     let layout = Layout::new((0.0, 5.0), (0.0, 6.0))
//                         .with_title("Exciting Bar Plot")
//                         .with_x_label("Category")
//                         .with_y_label("Value")
//                         .with_ticks(5);

//     let scene = render_bar(&bars, layout);
//     let svg = SvgBackend.render_scene(&scene);
//     std::fs::write("test_outputs/bar_builder.svg", svg.clone()).unwrap();

//     // Basic sanity assertion
//     assert!(svg.contains("<svg"));
// }
#[test]
fn test_bar_categories_svg_output_builder() {
    let bars = BarPlot::with_categories_and_values(
        vec!["team-kill", "Okay", "Lovely"],
        vec![2.5, 3.5, 4.0]
        )
        .with_color("darkgreen");
    
    let layout = Layout::new((0.0, 5.0), (0.0, 5.0))
                        .with_title("Grenade Stats")
                        .with_y_label("Lobbed nades");

    let scene = render_bar_categories(&bars, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/bar_categories_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}