//! Security regression: float sorts used `partial_cmp(..).unwrap_or(Equal)` (or
//! `.unwrap()`), which is not a total order once a key is NaN; the std sorts can
//! panic on such a comparator for slices over 20 elements. And the 3D projection
//! turned a finite axis range wider than `f64::MAX` into a scale of 0, so some
//! points projected to `inf * 0 = NaN` depths. Every render here must finish,
//! and the 3D ones must not emit NaN coordinates.
use kuva::backend::svg::SvgBackend;
use kuva::plot::bump::BumpPlot;
use kuva::plot::pareto::ParetoPlot;
use kuva::plot::pr::{PrGroup, PrPlot};
use kuva::plot::quiver3d::Quiver3DPlot;
use kuva::plot::roc::{RocGroup, RocPlot};
use kuva::plot::scatter3d::Scatter3DPlot;
use kuva::plot::streamgraph::StreamgraphPlot;
use kuva::plot::surface3d::Surface3DPlot;
use kuva::plot::survival::SurvivalPlot;
use kuva::plot::violin::ViolinPlot;
use kuva::plot::{ColorMap, QuiverPivot};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

fn svg(plot: Plot) -> String {
    let plots = vec![plot];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

/// Values alternating between the two ends of the f64 range, so the axis span
/// (`max - min`) overflows to infinity although every value is finite.
fn extreme(i: usize) -> f64 {
    let t = (i % 7) as f64 / 6.0;
    if i.is_multiple_of(2) {
        -1e308 * t
    } else {
        1e308 * t
    }
}

fn assert_no_nan(svg: &str, what: &str) {
    assert!(!svg.contains("NaN"), "{what}: NaN in output");
    assert!(
        !svg.contains("inf"),
        "{what}: infinite coordinate in output"
    );
}

#[test]
fn scatter3d_with_an_overflowing_axis_range() {
    let data: Vec<(f64, f64, f64)> = (0..60)
        .map(|i| (extreme(i), (i % 5) as f64, (i % 3) as f64))
        .collect();
    let out = svg(Plot::Scatter3D(Scatter3DPlot::new().with_data(data)));
    assert!(out.contains("<circle"));
    assert_no_nan(&out, "scatter3d");
}

#[test]
fn surface3d_with_an_overflowing_axis_range() {
    let n = 8;
    let z: Vec<Vec<f64>> = (0..n)
        .map(|r| (0..n).map(|c| ((r * c) % 5) as f64).collect())
        .collect();
    // -1e308 ..= 1e308, interpolated without overflowing.
    let xs: Vec<f64> = (0..n)
        .map(|i| {
            let t = i as f64 / (n - 1) as f64;
            -1e308 * (1.0 - t) + 1e308 * t
        })
        .collect();
    let ys: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let plot = Surface3DPlot::new(z).with_x_coords(xs).with_y_coords(ys);
    let out = svg(Plot::Surface3D(plot));
    assert_no_nan(&out, "surface3d");
}

#[test]
fn quiver3d_with_an_overflowing_axis_range() {
    let arrows: Vec<(f64, f64, f64, f64, f64, f64)> = (0..60)
        .map(|i| (extreme(i), (i % 5) as f64, (i % 3) as f64, 0.0, 1.0, 1.0))
        .collect();
    let out = svg(Plot::Quiver3D(
        Quiver3DPlot::new().with_arrows(arrows).with_tight_bounds(),
    ));
    assert_no_nan(&out, "quiver3d");
}

/// Arrow tips `x + u * scale` overflowed to inf for finite inputs, which made
/// the box range (and so every projected coordinate) NaN; vectors this large
/// also gave an infinite magnitude, and `inf / inf` in the colour mapping.
#[test]
fn quiver3d_with_overflowing_arrow_tips() {
    let cases: Vec<(&str, Quiver3DPlot)> = vec![
        (
            "far anchors, explicit scale",
            Quiver3DPlot::new()
                .with_arrows((0..20).map(|i| (1e308, i as f64, 0.0, 1e308, 1.0, 1.0)))
                .with_scale(10.0),
        ),
        (
            "tight bounds, explicit scale",
            Quiver3DPlot::new()
                .with_arrows((0..20).map(|i| (i as f64, i as f64, 0.0, 1e308, 1.0, 1.0)))
                .with_scale(10.0)
                .with_tight_bounds(),
        ),
        (
            "auto scale, tip pivot",
            Quiver3DPlot::new()
                .with_arrows((0..20).map(|i| (extreme(i), i as f64, 0.0, f64::MAX, -f64::MAX, 1.0)))
                .with_pivot(QuiverPivot::Tip),
        ),
        (
            "magnitude colormap",
            Quiver3DPlot::new()
                .with_arrows((0..20).map(|i| {
                    let u = if i % 2 == 0 { f64::MAX } else { 1.0 };
                    (i as f64, (i % 4) as f64, 0.0, u, u, u)
                }))
                .with_magnitude_colormap(ColorMap::Viridis, "|v|"),
        ),
    ];
    for (what, q) in cases {
        let out = svg(Plot::Quiver3D(q));
        assert!(out.contains("<path"), "{what}: no arrows drawn");
        assert_no_nan(&out, what);
    }
}

/// 40 values with NaN mixed in: long enough for the std sorts' merge path.
/// (Survival also used to spin forever on a NaN time: its tie scan relied on
/// `t == t`.)
fn with_nans() -> Vec<f64> {
    (0..40usize)
        .map(|i| {
            if i.is_multiple_of(3) {
                f64::NAN
            } else {
                (i * 7 % 13) as f64
            }
        })
        .collect()
}

#[test]
fn violin_with_nan_values() {
    svg(Plot::Violin(ViolinPlot::new().with_group("a", with_nans())));
}

#[test]
fn survival_with_nan_values() {
    let v = with_nans();
    let events: Vec<bool> = (0..v.len()).map(|i| i.is_multiple_of(2)).collect();
    let out = svg(Plot::Survival(
        SurvivalPlot::new().with_group("g", v, events),
    ));
    assert_no_nan(&out, "survival");
}

#[test]
fn pareto_with_nan_values() {
    let cats: Vec<(String, f64)> = with_nans()
        .into_iter()
        .enumerate()
        .map(|(i, x)| (format!("c{i}"), x))
        .collect();
    svg(Plot::Pareto(ParetoPlot::new().with_categories(cats)));
}

#[test]
fn bump_with_nan_values() {
    let v = with_nans();
    let mut bump = BumpPlot::new();
    for s in 0..25usize {
        bump = bump.with_series(format!("s{s}"), v.iter().take(4).map(|x| x + s as f64));
    }
    svg(Plot::Bump(bump));
}

#[test]
fn streamgraph_with_nan_values() {
    let mut stream = StreamgraphPlot::new().with_x((0..12).map(|i| i as f64).collect::<Vec<_>>());
    for s in 0..25usize {
        let ys: Vec<f64> = (0..12usize)
            .map(|i| {
                if (i + s).is_multiple_of(4) {
                    f64::NAN
                } else {
                    (i + s) as f64
                }
            })
            .collect();
        stream = stream.with_series(ys);
    }
    svg(Plot::Streamgraph(stream));
}

/// Under the NaN-last total order a NaN score would be the maximum, so the
/// optimal-point marker must skip NaN points rather than land on one.
#[test]
fn roc_and_pr_optimal_point_skips_nan() {
    let pts = [(0.0, 0.0), (0.2, 0.8), (0.5, f64::NAN), (1.0, 1.0)];
    let roc = |opt: bool| {
        let g = RocGroup::new("g").with_points(pts);
        let g = if opt { g.with_optimal_point() } else { g };
        svg(Plot::Roc(RocPlot::new().with_group(g)))
            .matches("NaN")
            .count()
    };
    assert_eq!(roc(true), roc(false), "ROC optimal marker drawn at NaN");
    let pr = |opt: bool| {
        let g = PrGroup::new("g").with_points(pts);
        let g = if opt { g.with_optimal_point() } else { g };
        svg(Plot::Pr(PrPlot::new().with_group(g)))
            .matches("NaN")
            .count()
    };
    assert_eq!(pr(true), pr(false), "PR optimal marker drawn at NaN");
}
