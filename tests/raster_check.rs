#[cfg(feature = "png")]
mod checks {
    use kuva::prelude::*;

    fn save(plots: Vec<Plot>, title: &str, w: f64, h: f64, path: &str) {
        let layout = Layout::auto_from_plots(&plots)
            .with_title(title)
            .with_width(w)
            .with_height(h);
        let png = kuva::render_to_raster(plots, layout, 2.0).unwrap();
        std::fs::write(path, png).unwrap();
        println!("wrote {path}");
    }

    #[test]
    fn raster_visual() {
        std::fs::create_dir_all("raster_test").unwrap();

        let pie = PiePlot::new()
            .with_slice("A", 40.0, "steelblue")
            .with_slice("B", 30.0, "tomato")
            .with_slice("C", 20.0, "seagreen")
            .with_legend("legend");
        save(
            vec![Plot::Pie(pie)],
            "Pie",
            400.0,
            300.0,
            "raster_test/pie.png",
        );

        let bar = BarPlot::new()
            .with_group("A", vec![(3.0, "steelblue".to_string())])
            .with_group("B", vec![(5.0, "tomato".to_string())])
            .with_group("C", vec![(2.0, "seagreen".to_string())]);
        save(
            vec![Plot::Bar(bar)],
            "Bar",
            400.0,
            300.0,
            "raster_test/bar.png",
        );

        let vals: Vec<f64> = (0..500).map(|i| (i as f64 * 0.05).sin() * 2.0).collect();
        let hist = Histogram::new()
            .with_data(vals)
            .with_bins(20)
            .with_range((-2.5, 2.5));
        save(
            vec![Plot::Histogram(hist)],
            "Histogram",
            400.0,
            300.0,
            "raster_test/hist.png",
        );

        let sc =
            ScatterPlot::new().with_data((0..50).map(|i| (i as f64 * 0.1, (i as f64 * 0.2).sin())));
        save(
            vec![Plot::Scatter(sc)],
            "Scatter",
            400.0,
            300.0,
            "raster_test/scatter.png",
        );

        let xs: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let ys: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin()).collect();
        let line = LinePlot::new()
            .with_data(xs.into_iter().zip(ys).map(|(x, y)| (x, y)))
            .with_color("steelblue")
            .with_legend("sin");
        save(
            vec![Plot::Line(line)],
            "Line",
            400.0,
            300.0,
            "raster_test/line.png",
        );

        let r_vals: Vec<f64> = (0..36)
            .map(|i| {
                let theta = i as f64 * 10.0_f64;
                (theta.to_radians() * 2.0).sin().abs() + 0.2
            })
            .collect();
        let theta_vals: Vec<f64> = (0..36).map(|i| i as f64 * 10.0).collect();
        let polar = PolarPlot::new().with_series(r_vals, theta_vals);
        save(
            vec![Plot::Polar(polar)],
            "Polar",
            400.0,
            300.0,
            "raster_test/polar.png",
        );

        let data: Vec<Vec<f64>> = (0..5)
            .map(|r| (0..5).map(|c| (r * 5 + c) as f64).collect())
            .collect();
        let row_labels: Vec<String> = vec!["R1", "R2", "R3", "R4", "R5"]
            .into_iter()
            .map(String::from)
            .collect();
        let col_labels: Vec<String> = vec!["C1", "C2", "C3", "C4", "C5"]
            .into_iter()
            .map(String::from)
            .collect();
        let hm = Heatmap::new()
            .with_data(data)
            .with_labels(row_labels, col_labels);
        save(
            vec![Plot::Heatmap(hm)],
            "Heatmap",
            400.0,
            300.0,
            "raster_test/heatmap.png",
        );
    }
}
