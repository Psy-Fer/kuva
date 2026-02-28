// use kuva::prelude::*;
use kuva::plot::ViolinPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;

use rand_distr::{Normal, Distribution};
use rand::prelude::*;


#[test]
fn test_violin_groups_svg_output_builder() {
    let violin = ViolinPlot::new()
    .with_group("A", vec![2.3, 2.5, 2.4,2.4,2.4,2.4,2.4,2.4, 2.4,2.4,2.4,2.4,2.4,3.1, 1.9])
    // .with_group("B", vec![2.0, 2.1, 3.5, 3.8, 4.0, 4.2])
    .with_width(10.0)
    .with_color("purple");

    // let x_labels: Vec<String> = boxplot.groups.iter().map(|g| g.label.clone()).collect();
    
    let plots = vec![Plot::Violin(violin)];

    let layout = Layout::auto_from_plots(&plots)
        .with_title("Viola Plot")
        .with_y_label("kde");
        // .with_x_categories(x_labels);

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_single_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}


#[test]
fn test_violin_random_data() {
    let mut rng = rand::rng();

    // Group A: Normal distribution centered at 0
    let normal = Normal::new(0.0, 1.0).unwrap();
    let a_values: Vec<f64> = (0..2000).map(|_| normal.sample(&mut rng.clone())).collect();

    // Group B: Bimodal distribution about 0
    let normal1 = Normal::new(-2.0, 0.5).unwrap();
    let normal2 = Normal::new(2.0, 0.5).unwrap();
    let b_values: Vec<f64> = (0..1000).map(|_| normal1.sample(&mut rng.clone()))
        .chain((0..1000).map(|_| normal2.sample(&mut rng.clone())))
        .collect();

    // Group C: Right-skewed (exponential-like)
    let c_values: Vec<f64> = (0..2000).map(|_| {
        let u: f64 = rng.random();
        -1.0 * (1.0 - u).ln() * 1.5 // inverse CDF for exponential
    }).collect();

    let violin =  ViolinPlot::new()
            .with_group("Normal", a_values)
            .with_group("Bimodal", b_values)
            .with_group("Skewed", c_values)
            .with_color("purple")
            .with_width(30.0);

    let plots = vec![Plot::Violin(violin)];

    let layout = Layout::auto_from_plots(&plots)
            .with_title("Viola Plots")
            .with_y_label("kde");

    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_groups_builder.svg", svg.clone()).unwrap();

    // Basic sanity assertion
    assert!(svg.contains("<svg"));
}

#[test]
fn test_violin_silverman_auto() {
    let mut rng = rand::rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let values: Vec<f64> = (0..500).map(|_| normal.sample(&mut rng)).collect();

    let violin = ViolinPlot::new()
        .with_group("Auto", values)
        .with_color("steelblue")
        .with_width(30.0);

    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots).with_title("Auto Bandwidth");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_silverman_auto.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
    assert!(svg.contains("<path"));
}

#[test]
fn test_violin_manual_bandwidth() {
    let mut rng = rand::rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let values: Vec<f64> = (0..300).map(|_| normal.sample(&mut rng)).collect();

    let violin = ViolinPlot::new()
        .with_group("Manual", values)
        .with_color("coral")
        .with_width(30.0)
        .with_bandwidth(0.5);

    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots).with_title("Manual Bandwidth 0.5");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_manual_bandwidth.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_violin_degenerate_constant() {
    // All identical values — previously caused step=0 and NaN x-values
    let values: Vec<f64> = vec![5.0; 50];

    let violin = ViolinPlot::new()
        .with_group("Constant", values)
        .with_color("green")
        .with_width(30.0);

    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots).with_title("Degenerate Constant");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_degenerate_constant.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}

#[test]
fn test_violin_single_value() {
    let violin = ViolinPlot::new()
        .with_group("Single", vec![3.14f64])
        .with_color("orange")
        .with_width(30.0);

    let plots = vec![Plot::Violin(violin)];
    let layout = Layout::auto_from_plots(&plots).with_title("Single Value");
    let scene = render_multiple(plots, layout);
    let svg = SvgBackend.render_scene(&scene);
    std::fs::write("test_outputs/violin_single_value.svg", svg.clone()).unwrap();

    assert!(svg.contains("<svg"));
}