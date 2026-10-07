//! CIGAR bars on `BrickPlot` — a thin band above a row coloured by the read's
//! CIGAR operations.
//!
//! Rows here are real HG002 SCA17_TBP data: 18 reads on one haplotype sharing
//! `108M`, 12 on the other sharing `42M3D63M` (a 3 bp contraction), plus
//! singletons carrying an insertion and a second deletion.

mod common;
use kuva::plot::{BrickPlot, BrickSort, CigarIssue, CigarOp};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

const MOTIFS: &str = "CAG:A,CAA:B";
const HAP1: &str = "3A3B9A1B1A1B18A"; // 108 bp
const HAP2: &str = "3A3B9A1B1A1B17A"; // 105 bp

/// `(motif_map, strigar)` pairs, row names, and per-row CIGARs.
type Rows = (Vec<(String, String)>, Vec<String>, Vec<Option<String>>);

/// Real locus rows, in a deliberately unsorted order.
fn hg002_rows() -> Rows {
    let mut s = Vec::new();
    let mut n = Vec::new();
    let mut c = Vec::new();
    let mut push = |strig: &str, cig: &str, name: &str, k: usize| {
        for _ in 0..k {
            s.push((MOTIFS.to_string(), strig.to_string()));
            n.push(name.to_string());
            c.push(Some(cig.to_string()));
        }
    };
    push(HAP1, "108M", "Hap1", 18);
    push(HAP2, "42M3D63M", "Hap2", 12);
    push("3A3B9A1B1A1B16A", "42M3D6M3D54M", "Hap2", 1);
    push("3A3B10A1B1A1B18A", "54M3I54M", "noHap", 1);
    (s, n, c)
}

fn build(sort: BrickSort) -> BrickPlot {
    let (s, n, c) = hg002_rows();
    BrickPlot::new()
        .with_strigars(s)
        .with_names(n)
        .with_cigars(c)
        .with_sort(sort)
}

#[test]
fn real_locus_cigars_all_reconcile_with_their_rows() {
    let plot = build(BrickSort::Cigar);
    assert_eq!(
        plot.cigar_issues(),
        Vec::<CigarIssue>::new(),
        "every CIGAR's read length must equal its row's expanded length"
    );
}

#[test]
fn renders_bars_and_writes_svg_for_inspection() {
    let plot = build(BrickSort::Cigar);
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("SCA17_TBP - CIGAR bars")
        .with_width(820.0)
        .with_height(520.0);
    let svg = kuva::backend::svg::SvgBackend.render_scene(&render_multiple(plots, layout));

    assert!(svg.contains("<svg"));
    // Bar fills for the ops present, and none for ops that are not.
    assert!(svg.contains(CigarOp::Match.default_color()), "match bar");
    assert!(
        svg.contains(CigarOp::Insertion.default_color()),
        "insertion bar"
    );
    assert!(
        svg.contains(CigarOp::Deletion.default_color()),
        "deletion caret"
    );
    assert!(
        !svg.contains(CigarOp::SoftClip.default_color()),
        "no soft clips in this data, so no softclip swatch"
    );
    // Run brackets carry their count.
    assert!(svg.contains(">x18<"), "18-read run bracket");
    assert!(svg.contains(">x12<"), "12-read run bracket");

    common::write_test_output("test_outputs/brick_cigar_bars.svg", svg).unwrap();
}

// Dedup is what makes the plot readable: 32 reads carry only 4 distinct
// CIGARs, so sorting by CIGAR must leave 4 bars rather than 32.
#[test]
fn sorting_by_cigar_draws_one_bar_per_distinct_cigar() {
    // 32 reads carrying only 4 distinct CIGARs.
    let plot = build(BrickSort::Cigar);
    assert_eq!(plot.cigar_bar_count(), 4);
    // Unsorted, the same data needs more bars because a CIGAR can be split
    // across non-adjacent rows.
    assert!(build(BrickSort::None).cigar_bar_count() >= 4);
}

// Unsorted input splits a CIGAR across runs and repeats its bar, which is the
// argument for BrickSort::Cigar rather than a claim that other orders break.
#[test]
fn unsorted_input_repeats_bars_that_sorting_would_merge() {
    let (s, n, mut c) = hg002_rows();
    // Interleave one odd read into the Hap1 block.
    c[5] = Some("54M3I54M".to_string());
    let mut s2 = s;
    s2[5] = (MOTIFS.to_string(), "3A3B10A1B1A1B18A".to_string());
    let plot = BrickPlot::new()
        .with_strigars(s2)
        .with_names(n)
        .with_cigars(c)
        .with_sort(BrickSort::None);
    let cl = plot.cigar_issues();
    assert!(cl.is_empty(), "{cl:?}");
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = kuva::backend::svg::SvgBackend.render_scene(&render_multiple(plots, layout));
    // Hap1's 108M is now split either side of the interloper, so it is drawn
    // twice; sorted, it would be drawn once.
    assert!(
        svg.matches(CigarOp::Match.default_color()).count() > 5,
        "unsorted order should repeat at least one bar"
    );
}

// A row whose CIGAR disagrees with its bricks must not be drawn: a bar that is
// 9 bp short misaligns every operation after the first while still looking
// like a valid annotation.
#[test]
fn mismatched_cigar_is_reported_and_not_drawn() {
    let plot = BrickPlot::new()
        .with_strigars([(MOTIFS, HAP1)])
        .with_names(["read"])
        .with_cigars(["99M"]);
    let issues = plot.cigar_issues();
    assert_eq!(issues.len(), 1);
    assert!(matches!(issues[0], CigarIssue::LengthMismatch { .. }));

    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let svg = kuva::backend::svg::SvgBackend.render_scene(&render_multiple(plots, layout));
    assert!(
        !svg.contains(CigarOp::Match.default_color()),
        "a rejected CIGAR must draw no bar at all"
    );
}

// Without CIGARs the plot must be byte-identical to before the feature: the
// bar allowance is charged per drawn bar, so zero bars means zero extra space.
#[test]
fn plots_without_cigars_are_unchanged() {
    let render = |with_cigars: bool| {
        let (s, n, c) = hg002_rows();
        let mut p = BrickPlot::new().with_strigars(s).with_names(n);
        if with_cigars {
            p = p.with_cigars(c);
        }
        let plots = vec![Plot::Brick(p)];
        let layout = Layout::auto_from_plots(&plots)
            .with_width(820.0)
            .with_height(520.0);
        kuva::backend::svg::SvgBackend.render_scene(&render_multiple(plots, layout))
    };
    let plain = render(false);
    assert!(!plain.contains(CigarOp::Deletion.default_color()));
    // And the bars really do change the layout when present.
    assert_ne!(plain, render(true));
}

// ── flanked rows: the CIGAR covers every base the row draws ──────────────────
//
// A flanked row is drawn as left flank, then STRIGAR bricks, then right flank.
// The caller supplies one CIGAR spanning all three, so validation and drawing
// both have to start at the left flank rather than at the STRIGAR.

/// Left flank 6 bp, STRIGAR 5 units x 3 bp = 15 bp, right flank 4 bp = 25 bp.
fn flanked_plot(cigar: &str) -> BrickPlot {
    BrickPlot::new()
        .with_flanked_strigars([("ACGTAC", "CAG:A", "5A", "TTGA")])
        .with_names(["read"])
        .with_cigars([cigar])
}

#[test]
fn flanked_row_validates_against_flanks_plus_strigar() {
    // 5M2I10M3D8M consumes 5+2+10+8 = 25 read bases, matching 6 + 15 + 4.
    let plot = flanked_plot("5M2I10M3D8M");
    assert_eq!(
        plot.cigar_issues(),
        Vec::<CigarIssue>::new(),
        "a CIGAR spanning flanks + STRIGAR must validate"
    );
    assert_eq!(plot.cigar_bar_count(), 1);
}

// The STRIGAR alone is 15 bp. A CIGAR covering only that no longer validates,
// because the bar would be drawn 10 bases short of the row it annotates.
#[test]
fn cigar_covering_only_the_strigar_is_rejected_for_a_flanked_row() {
    let issues = flanked_plot("15M").cigar_issues();
    assert_eq!(issues.len(), 1);
    match &issues[0] {
        CigarIssue::LengthMismatch {
            cigar_read_bases,
            row_bases,
            ..
        } => {
            assert_eq!(*cigar_read_bases, 15.0);
            assert_eq!(*row_bases, 25.0, "flanks are part of the row");
        }
        other => panic!("wrong issue: {other:?}"),
    }
}

/// x of every filled `<rect>` in the SVG, in document order. Excludes the
/// background (no `x`) and the clip rect (no `fill`).
fn all_filled_rect_xs(svg: &str) -> Vec<f64> {
    svg.split("<rect")
        .skip(1)
        .filter(|r| r.contains("fill=\""))
        .filter_map(|r| r.split("x=\"").nth(1)?.split('"').next()?.parse().ok())
        .collect()
}

/// x of every `<rect>` in the SVG whose fill is `fill`, in document order.
fn rect_xs(svg: &str, fill: &str) -> Vec<f64> {
    svg.split("<rect")
        .skip(1)
        .filter(|r| r.contains(&format!("fill=\"{fill}\"")))
        .filter_map(|r| r.split("x=\"").nth(1)?.split('"').next()?.parse().ok())
        .collect()
}

fn render_svg(plot: BrickPlot) -> String {
    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_width(700.0)
        .with_height(220.0);
    kuva::backend::svg::SvgBackend.render_scene(&render_multiple(plots, layout))
}

// The bar must begin at the row's first drawn base (the left flank), not at
// the STRIGAR. Compared against the leftmost drawn rect rather than an
// absolute pixel, so it survives margin and sizing changes.
#[test]
fn flanked_bar_starts_at_the_rows_first_drawn_base() {
    let svg = render_svg(flanked_plot("5M2I10M3D8M"));
    let bar_start = rect_xs(&svg, CigarOp::Match.default_color())
        .into_iter()
        .fold(f64::INFINITY, f64::min);
    // The leftmost drawn rect is the left flank's first base. The legend sits
    // to the right, so it cannot win this minimum.
    let row_start = all_filled_rect_xs(&svg)
        .into_iter()
        .fold(f64::INFINITY, f64::min);
    assert!(row_start.is_finite(), "no drawn rects");
    assert!(
        (bar_start - row_start).abs() < 1.0,
        "bar starts at {bar_start}, row's first drawn base at {row_start}"
    );
}

// Right anchoring shifts whole rows; the bar uses the same per-row offset as
// the bricks, so the two must still line up.
#[test]
fn right_anchored_flanked_row_keeps_bar_and_bricks_aligned() {
    use kuva::plot::brick::BrickAnchor;
    let plot = BrickPlot::new()
        .with_flanked_strigars([
            ("ACGTAC", "CAG:A", "5A", "TTGA"),
            ("ACGTAC", "CAG:A", "3A", "TTGA"),
        ])
        .with_names(["long", "short"])
        .with_cigars(["25M", "19M"])
        .with_anchor(BrickAnchor::Right)
        .with_cigar_dedup(false);
    assert!(plot.cigar_issues().is_empty(), "{:?}", plot.cigar_issues());

    let svg = render_svg(plot);
    // Two bars (dedup off) plus the legend swatch, which sits far to the right.
    let mut bars = rect_xs(&svg, CigarOp::Match.default_color());
    assert_eq!(bars.len(), 3, "one bar per row, plus the legend swatch");
    bars.sort_by(|a, b| a.partial_cmp(b).unwrap());
    bars.truncate(2);
    // Rows end flush on the right, so the shorter row must start further right.
    let (b0, b1) = (bars[0], bars[1]);
    assert!(
        b1 > b0,
        "right-anchored shorter row should start further right ({b0} vs {b1})"
    );
}

// ── row-height sizing must include the bars ─────────────────────────────────

#[test]
fn row_height_sizing_accounts_for_cigar_bars() {
    let rows: Vec<(String, String)> = (0..10)
        .map(|_| (MOTIFS.to_string(), HAP1.to_string()))
        .collect();
    // Distinct CIGARs on 3 rows so dedup cannot merge them; the rest have none.
    let cigars: Vec<Option<String>> = (0..10)
        .map(|i| match i {
            0 => Some("108M".to_string()),
            4 => Some("54M3I51M".to_string()),
            8 => Some("40M3D68M".to_string()),
            _ => None,
        })
        .collect();
    let plot = BrickPlot::new()
        .with_strigars(rows)
        .with_names((0..10).map(|i| format!("r{i}")))
        .with_cigars(cigars)
        .with_row_height(10.0);
    assert_eq!(plot.cigar_bar_count(), 3);

    let plots = vec![Plot::Brick(plot)];
    let layout = Layout::auto_from_plots(&plots);
    let height = layout.height.expect("row_height sizes the canvas");

    // Rebuild the same plot to measure the overhead the sizing used.
    let probe_plots = vec![Plot::Brick(
        BrickPlot::new()
            .with_strigars((0..10).map(|_| (MOTIFS, HAP1)))
            .with_names((0..10).map(|i| format!("r{i}")))
            .with_row_height(10.0),
    )];
    let probe = Layout::auto_from_plots(&probe_plots);
    let plain_height = probe.height.expect("height");

    // 10 rows with no bars would be 10 * 10 + overhead; with 3 bars the canvas
    // must be 3 * 0.45 * 10 px taller, leaving each brick row 10px.
    let expected_extra = 3.0 * 0.45 * 10.0;
    assert!(
        (height - plain_height - expected_extra).abs() < 1e-6,
        "expected {expected_extra}px extra, got {}",
        height - plain_height
    );
}

// ── permute_rows must carry the CIGARs with their rows ──────────────────────

#[test]
fn permute_rows_keeps_each_cigar_with_its_read() {
    // Row lengths differ, so a CIGAR landing on the wrong row fails validation.
    let plot = BrickPlot::new()
        .with_strigars([
            (MOTIFS, "1A"), // 3 bp
            (MOTIFS, "2A"), // 6 bp
            (MOTIFS, "3A"), // 9 bp
        ])
        .with_names(["a", "b", "c"])
        .with_cigars(["3M", "6M", "9M"]);
    assert!(plot.cigar_issues().is_empty(), "{:?}", plot.cigar_issues());

    let permuted = plot.permute_rows(&[2, 0, 1]);
    assert!(
        permuted.cigar_issues().is_empty(),
        "CIGARs must follow their rows through a permutation: {:?}",
        permuted.cigar_issues()
    );
    // And the rows really did move.
    assert_eq!(permuted.row_base_len(0), 9.0);
    assert_eq!(permuted.row_base_len(1), 3.0);
}
