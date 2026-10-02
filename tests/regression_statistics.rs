use kuva::plot::scatter::TrendLine;
use kuva::plot::ScatterPlot;
use kuva::render::layout::{ComputedLayout, Layout};
use kuva::render::plots::Plot;
use kuva::render::render::{render_multiple, Primitive};
use kuva::render::render_utils::{linear_regression, pearson_corr};

#[test]
fn regression_preserves_translated_and_small_unit_fits() {
    let offsets = [
        (1e8, 1.0),
        (1e9, 1.0),
        (1e12, 1.0),
        (0.0, 1e-6),
        (1.7e9, 2f64.powi(-10)),
        (1.7e18, 1e9),
    ];
    for (offset, unit) in offsets {
        let data = (0..=10).map(|t| (offset + t as f64 * unit, 3.0 + 2.0 * t as f64));
        let (slope, intercept, r) = linear_regression(data).expect("nonconstant finite fit");
        let expected_slope = 2.0 / unit;
        assert!((slope / expected_slope - 1.0).abs() < 1e-14);
        assert!((intercept / (3.0 - expected_slope * offset) - 1.0).abs() < 1e-14);
        assert!((r - 1.0).abs() < 1e-14);
    }
}

#[test]
fn statistics_omit_spreads_beyond_f64_squares() {
    for unit in [1e-100, 1e100] {
        let data: Vec<_> = (-4..=4)
            .map(|t| (t as f64 * unit, -2.0 * t as f64 * unit))
            .collect();
        assert!((pearson_corr(&data).unwrap() + 1.0).abs() < 1e-14);
        let (slope, intercept, r) = linear_regression(data).unwrap();
        assert!((slope + 2.0).abs() < 1e-14);
        assert!(intercept.abs() < 1e-14 * unit);
        assert!((r + 1.0).abs() < 1e-14);
    }
    for unit in [f64::from_bits(1), 1e-200, 1e200] {
        let data: Vec<_> = (-4..=4)
            .map(|t| (t as f64 * unit, -2.0 * t as f64 * unit))
            .collect();
        assert_eq!(pearson_corr(&data), None);
        assert_eq!(linear_regression(data), None);
    }
    for data in [
        [(-f64::MAX, -f64::MAX), (f64::MAX, f64::MAX)],
        [(0.0, 0.0), (1e-200, 1e200)],
        [(0.0, 0.0), (1e200, 1e-200)],
    ] {
        assert_eq!(pearson_corr(&data), None);
        assert_eq!(linear_regression(data), None);
    }
}

#[test]
fn statistics_reject_undefined_results() {
    for data in [
        vec![],
        vec![(1.0, 2.0)],
        vec![(1.0, 2.0), (1.0, 3.0)],
        vec![(1.0, 2.0), (3.0, 2.0)],
        vec![(0.1, 1.0), (0.1, 2.0), (0.1, 4.0)],
        vec![(1.7e9 + 0.1, 1.0), (1.7e9 + 0.1, 2.0), (1.7e9 + 0.1, 4.0)],
        vec![(1.0, 2.0), (f64::NAN, 3.0)],
        vec![(1.0, 2.0), (3.0, f64::INFINITY)],
        vec![(f64::NEG_INFINITY, 2.0), (3.0, 4.0)],
    ] {
        assert_eq!(linear_regression(data.iter().copied()), None);
        assert_eq!(pearson_corr(&data), None);
    }
}

#[test]
fn scatter_uses_centered_predictions_for_bounds_and_line() {
    let x_min = 1e16;
    let x_max = x_min + 60.0;
    let scatter = ScatterPlot::new()
        .with_data((0..=10).map(|t| (x_min + 6.0 * t as f64, 1.0 + 2.0 * t as f64)))
        .with_trend(TrendLine::Linear)
        .with_trend_color("#d62728")
        .with_equation()
        .with_correlation();
    let plot = Plot::Scatter(scatter);
    let (_, (y_min, y_max)) = plot.bounds().unwrap();
    assert!((y_min - 1.0).abs() < 1e-13);
    assert!((y_max - 21.0).abs() < 1e-13);
    let layout = Layout::new((x_min, x_max), (0.0, 24.0));
    let computed = ComputedLayout::from_layout(&layout);
    let scene = render_multiple(vec![plot], layout);
    let line = scene
        .elements
        .iter()
        .find_map(|primitive| match primitive {
            Primitive::Line { y1, y2, stroke, .. } if stroke.to_svg_string() == "#d62728" => {
                Some((*y1, *y2))
            }
            _ => None,
        })
        .expect("finite trend line");
    assert!((line.0 - computed.map_y(1.0)).abs() < 1e-10);
    assert!((line.1 - computed.map_y(21.0)).abs() < 1e-10);
    assert!(scene.elements.iter().any(|primitive| matches!(primitive,
        Primitive::Text { content, .. } if content.contains("y = 0.33x") && content.contains("r = 1.00")
    )));
}

#[test]
fn regression_handles_mostly_zero_data() {
    for sign in [-1.0, 1.0] {
        let mut data = vec![(0.0, 0.0); 10_001];
        data[0] = (sign, 0.0);
        data[1] = (0.0, sign);
        let (slope, intercept, r) = linear_regression(data).unwrap();
        assert!((slope + 1e-4).abs() < 1e-15);
        assert!((intercept - sign * 1e-4).abs() < 1e-15);
        assert!((r + 1e-4).abs() < 1e-15);
    }
}
