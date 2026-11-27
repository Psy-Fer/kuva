//! STRIGAR run counts come from the data, and each copy becomes one brick.
//! A row such as `18446744073709551615A` asked `with_strigars` to build that
//! many bricks, and two such runs overflowed the motif frequency counters
//! (a panic in debug builds). Rows over the brick limits are now rejected
//! before anything is expanded.

mod hostile;

use kuva::backend::svg::SvgBackend;
use kuva::plot::brick::{StrigarIssue, MAX_STRIGAR_PLOT_BRICKS, MAX_STRIGAR_ROW_BRICKS};
use kuva::plot::BrickPlot;
use kuva::render::render::{render_multiple, Scene};
use kuva::render::{layout::Layout, plots::Plot};

fn row(motifs: &str, strigar: String) -> (String, String) {
    (motifs.to_string(), strigar)
}

fn render(plot: BrickPlot) -> Scene {
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots);
    render_multiple(plots, layout)
}

#[test]
fn huge_and_overflowing_run_counts_are_rejected() {
    let max = usize::MAX;
    let (plot, scene) = hostile::bounded(64 << 20, move || {
        let plot = BrickPlot::new().with_strigars(vec![
            row("CAG:A", "5A".into()),
            row("CAG:A", format!("{max}A")),
            // Two runs whose sum overflows usize.
            row("CAG:A,AATGG:B", format!("{max}A{max}B")),
        ]);
        let scene = render(plot.clone());
        (plot, scene)
    });
    assert_eq!(
        plot.strigar_issues(),
        vec![
            StrigarIssue::TooManyBricks {
                row: 1,
                bricks: max
            },
            StrigarIssue::TooManyBricks {
                row: 2,
                bricks: max
            },
        ]
    );
    let expanded = plot.strigar_exp.as_ref().unwrap();
    assert_eq!(expanded[0].chars().count(), 5);
    assert!(expanded[1].is_empty() && expanded[2].is_empty());

    let w: Vec<_> = scene
        .warnings
        .iter()
        .filter(|w| w.code == "brick.strigar_rejected")
        .collect();
    assert_eq!(w.len(), 1, "{:?}", scene.warnings);
    assert!(
        w[0].message.starts_with("2 STRIGAR row(s)"),
        "{}",
        w[0].message
    );
    assert!(w[0].message.contains("row 1"), "{}", w[0].message);
    assert!(SvgBackend.render_scene(&scene).contains("<svg"));
}

#[test]
fn the_row_limit_is_inclusive_and_counts_only_drawn_runs() {
    let n = MAX_STRIGAR_ROW_BRICKS;
    let plot = hostile::bounded(64 << 20, move || {
        BrickPlot::new().with_strigars(vec![
            row("CAG:A", format!("{n}A")),
            row("CAG:A", format!("{}A", n - 1) + "2A"),
            // `Z` is not in the motif map, so its run draws nothing and does
            // not count.
            row("CAG:A", format!("{}Z1A", n * 10)),
        ])
    });
    assert_eq!(
        plot.strigar_issues(),
        vec![StrigarIssue::TooManyBricks {
            row: 1,
            bricks: n + 1
        }]
    );
    let expanded = plot.strigar_exp.as_ref().unwrap();
    assert_eq!(expanded[0].chars().count(), n);
    assert_eq!(expanded[2].chars().count(), 1);
}

#[test]
fn rows_past_the_plot_limit_are_rejected_in_order() {
    let per_row = MAX_STRIGAR_ROW_BRICKS;
    let full_rows = MAX_STRIGAR_PLOT_BRICKS / per_row;
    let plot = hostile::bounded(64 << 20, move || {
        let mut rows: Vec<_> = (0..full_rows)
            .map(|_| row("A:A", format!("{per_row}A")))
            .collect();
        rows.push(row("A:A", "1A".into()));
        // A row that draws nothing still fits.
        rows.push(row("A:A", String::new()));
        BrickPlot::new().with_strigars(rows)
    });
    assert_eq!(
        plot.strigar_issues(),
        vec![StrigarIssue::PlotTooLarge {
            row: full_rows,
            bricks: 1
        }]
    );
    let expanded = plot.strigar_exp.as_ref().unwrap();
    assert_eq!(expanded.len(), full_rows + 2);
    assert!(expanded[full_rows].is_empty());
}

#[test]
fn flanked_strigars_use_the_same_limits() {
    let max = usize::MAX;
    let plot = hostile::bounded(64 << 20, move || {
        BrickPlot::new().with_flanked_strigars([
            ("ACGT", "CAG:A", "3A".to_string(), "TTGA"),
            ("ACGT", "CAG:A", format!("{max}A"), "TTGA"),
        ])
    });
    assert_eq!(
        plot.strigar_issues(),
        vec![StrigarIssue::TooManyBricks {
            row: 1,
            bricks: max
        }]
    );
}

#[test]
fn clean_rows_give_no_warning() {
    let scene = render(BrickPlot::new().with_strigars(vec![row("CAG:A", "5A".into())]));
    assert!(scene.warnings.is_empty(), "{:?}", scene.warnings);
}
