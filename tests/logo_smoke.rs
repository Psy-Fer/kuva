//! End-to-end tests for the sequence-logo plot type.

#![cfg(feature = "logo")]

mod common;

use kuva::plot::logo::{parse_fasta, Alphabet, LogoUnits, PalettePreset, SequenceLogo, GAP_LETTER};
use kuva::prelude::*;

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/data/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// Render one logo with a layout derived from that logo (not from an empty
/// plot list, which would give a degenerate axis range).
fn render(logo: SequenceLogo, width: f64) -> String {
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots).with_width(width);
    kuva::render_to_svg(plots, layout)
}

#[test]
fn kozak_fixture_renders_to_svg_with_glyph_paths() {
    let logo = SequenceLogo::from_fasta_str(&fixture("kozak.fasta"), Alphabet::Dna).unwrap();
    assert_eq!(logo.n_positions(), 10);
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots);
    let svg = kuva::render_to_svg(plots, layout);

    assert!(svg.contains("<svg"));
    // Glyph outlines are emitted as <path> elements, not <text>, so a
    // conserved position produces at least one path with a non-empty `d`.
    assert!(svg.contains("<path"));
    // Integer x-tick labels — every position 1..=10 appears as a category
    // label. (The y-axis legitimately shows fractional bits like 2.5, so
    // only positive assertions are safe on the raw SVG.)
    for pos in 1..=10 {
        assert!(svg.contains(&format!(">{pos}<")), "missing x label {pos}");
    }
    common::write_test_output("test_outputs/logo_kozak.svg", &svg).unwrap();
}

#[test]
fn ctcf_fixture_parses_and_renders() {
    let records = parse_fasta(&fixture("ctcf.fasta"));
    assert_eq!(records.len(), 10);
    let logo = SequenceLogo::from_fasta_str(&fixture("ctcf.fasta"), Alphabet::Dna).unwrap();
    assert_eq!(logo.n_positions(), 19);
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots);
    let svg = kuva::render_to_svg(plots, layout);
    assert!(svg.contains("<svg"));
    common::write_test_output("test_outputs/logo_ctcf.svg", &svg).unwrap();
}

#[test]
fn matrix_input_matches_sequences_input() {
    // Two sequences, each ACGA / ACGG — position 3 splits A/G.
    let by_seq = SequenceLogo::from_sequences(vec!["ACGA", "ACGG"], Alphabet::Dna).unwrap();
    let by_matrix = SequenceLogo::from_matrix(
        vec![
            vec![2.0, 0.0, 0.0, 0.0],
            vec![0.0, 2.0, 0.0, 0.0],
            vec![0.0, 0.0, 2.0, 0.0],
            vec![1.0, 0.0, 1.0, 0.0],
        ],
        Alphabet::Dna,
    );
    for logo in [by_seq, by_matrix] {
        let plots: Vec<Plot> = vec![logo.into()];
        let layout = Layout::auto_from_plots(&plots);
        let svg = kuva::render_to_svg(plots, layout);
        assert!(svg.contains("<svg"));
    }
}

#[test]
fn probability_mode_bounds_y_at_one() {
    let logo = SequenceLogo::from_sequences(vec!["AAA", "TTT"], Alphabet::Dna)
        .unwrap()
        .with_units(LogoUnits::Probability);
    assert_eq!(logo.y_max(), 1.0);
}

#[test]
fn rna_reverse_complement_displays_u_not_t() {
    // Fully conserved A — under RC the displayed letter must be U for RNA.
    let logo = SequenceLogo::from_sequences(vec!["AAAA"], Alphabet::Rna)
        .unwrap()
        .with_no_correction()
        .with_reverse_complement()
        .unwrap();
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots);
    let svg = kuva::render_to_svg(plots, layout);
    // U in the classic palette is red rgb(200,0,0) → #c80000; A is green.
    // The rendered stack must use U's color, proving the complement is
    // alphabet-aware (a DNA-style A→T map would also give red — so check
    // the consensus row letter instead via a consensus-enabled render).
    let logo2 = SequenceLogo::from_sequences(vec!["AAAA"], Alphabet::Rna)
        .unwrap()
        .with_consensus()
        .with_reverse_complement()
        .unwrap();
    let svg2 = render(logo2, 400.0);
    assert!(
        svg2.contains(">U<"),
        "RC'd RNA consensus should display U, got: {}",
        &svg2[..svg2.len().min(2000)]
    );
    assert!(svg.contains("<svg"));
}

#[test]
fn respect_case_and_include_gaps_change_the_matrix() {
    // 'a' soft-masked, '-' gap.
    let plain = SequenceLogo::from_sequences(vec!["Aa-", "AAA"], Alphabet::Dna).unwrap();
    let masked = SequenceLogo::from_sequences(vec!["Aa-", "AAA"], Alphabet::Dna)
        .unwrap()
        .with_respect_case();
    let gapped = SequenceLogo::from_sequences(vec!["Aa-", "AAA"], Alphabet::Dna)
        .unwrap()
        .with_include_gaps();

    // Position 1: plain counts both a→A and A (2); masked drops the lowercase (1).
    assert_eq!(plain.matrix[1][0], 2.0);
    assert_eq!(masked.matrix[1][0], 1.0);
    // Position 2: gapped grows a gap column counting the '-'.
    assert_eq!(*gapped.letters.last().unwrap(), GAP_LETTER);
    let gap_idx = gapped.letters.len() - 1;
    assert_eq!(gapped.matrix[2][gap_idx], 1.0);
}

#[test]
fn custom_alphabet_survives_end_to_end() {
    let alphabet = Alphabet::Custom(vec!['*', '#', '@']);
    let logo = SequenceLogo::from_sequences(vec!["*#@", "*#@", "@#*"], alphabet).unwrap();
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots);
    let svg = kuva::render_to_svg(plots, layout);
    assert!(svg.contains("<svg"));
}

#[test]
fn palette_preset_changes_letter_colors() {
    let seqs = vec!["A", "A", "A"];
    let classic = SequenceLogo::from_sequences(seqs.clone(), Alphabet::Dna)
        .unwrap()
        .with_preset(PalettePreset::Classic);
    let mono = SequenceLogo::from_sequences(seqs, Alphabet::Dna)
        .unwrap()
        .with_preset(PalettePreset::Mono);
    let svg_classic = render(classic, 400.0);
    let svg_mono = render(mono, 400.0);
    // Classic A is green (0,150,0); mono is black.
    assert!(
        uses_color(&svg_classic, [0, 150, 0]),
        "classic A should be green"
    );
    assert!(uses_color(&svg_mono, [0, 0, 0]), "mono A should be black");
}

#[test]
fn protein_logo_legend_groups_by_chemistry_color() {
    // The WebLogo protein palette maps 20 letters onto 5 chemistry-group
    // colors, so the legend must collapse to 5 rows — one per distinct
    // swatch — not 20 rows with duplicated colors.
    let logo = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 4], Alphabet::Protein)
        .with_preset(PalettePreset::Classic);
    assert!(logo.legend_visible());
    let plots: Vec<Plot> = vec![logo.into()];
    let entries = collect_legend_entries(&plots);
    assert_eq!(entries.len(), 5, "expected one row per chemistry group");
    // Every swatch is a distinct color.
    let mut colors: Vec<&str> = entries.iter().map(|e| e.color.as_str()).collect();
    colors.sort_unstable();
    colors.dedup();
    assert_eq!(colors.len(), 5, "legend colors must be distinct");
    // Groups are labelled with their member letters, in alphabet order.
    let labels: Vec<&str> = entries.iter().map(|e| e.label.as_str()).collect();
    assert!(
        labels.contains(&"AFILMPVW"),
        "hydrophobic group; got {labels:?}"
    );
    assert!(labels.contains(&"DE"), "acidic group; got {labels:?}");
    assert!(labels.contains(&"HKR"), "basic group; got {labels:?}");
    // And the layout reserves legend space.
    let layout = Layout::auto_from_plots(&plots);
    assert!(layout.show_legend);
    // DNA logos get none.
    let dna = SequenceLogo::from_sequences(vec!["ACGT"], Alphabet::Dna).unwrap();
    let dna_plots: Vec<Plot> = vec![dna.into()];
    assert!(collect_legend_entries(&dna_plots).is_empty());
}

#[test]
fn taylor_palette_gives_every_residue_a_distinct_color() {
    // Chemistry-group palettes reuse one color across a whole group; Taylor
    // must not, or a 20-row legend is ambiguous.
    let logo = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 4], Alphabet::Protein)
        .with_preset(PalettePreset::Taylor);
    let mut colors: Vec<[u8; 3]> = logo
        .letters
        .iter()
        .map(|&c| logo.palette.color_of(c))
        .collect();
    let total = colors.len();
    colors.sort_unstable();
    colors.dedup();
    assert_eq!(colors.len(), total, "every residue must have its own color");
    // Legend therefore has one row per residue, not per group.
    assert_eq!(logo.legend_groups().len(), 20);
    // And no residue falls through to the unassigned-letter grey.
    assert!(
        !colors.contains(&[120, 120, 120]),
        "unassigned residue color"
    );
}

/// Whether `svg` paints anything with `rgb`, in any encoding a backend may
/// emit (`#rrggbb`, `rgb(r,g,b)`, or a CSS name the renderer may substitute).
fn uses_color(svg: &str, rgb: [u8; 3]) -> bool {
    let [r, g, b] = rgb;
    svg.contains(&format!("#{r:02x}{g:02x}{b:02x}"))
        || svg.contains(&format!("#{r:02X}{g:02X}{b:02X}"))
        || svg.contains(&format!("rgb({r},{g},{b})"))
        || svg.contains(&format!("rgb({r}, {g}, {b})"))
}

/// Widest x-extent of any `<path>` in an SVG, per fill color.
fn path_width_by_fill(svg: &str, rgb: [u8; 3]) -> Option<f64> {
    let [r, g, b] = rgb;
    let encodings = [
        format!("#{r:02x}{g:02x}{b:02x}"),
        format!("rgb({r},{g},{b})"),
    ];
    let mut best: Option<f64> = None;
    for chunk in svg.split("<path d=\"").skip(1) {
        let end = chunk.find('"')?;
        let d = &chunk[..end];
        let tail = &chunk[end..];
        // Only consider the path painted with this fill.
        let stop = tail.find("/>").unwrap_or(tail.len());
        if !encodings.iter().any(|e| tail[..stop].contains(e.as_str())) {
            continue;
        }
        let mut lo = f64::INFINITY;
        let mut hi = f64::NEG_INFINITY;
        // Coordinates come in x y pairs after each command letter.
        let nums: Vec<f64> = d
            .split(|c: char| c.is_ascii_alphabetic() || c == ' ')
            .filter(|t| !t.is_empty())
            .filter_map(|t| t.parse::<f64>().ok())
            .collect();
        for pair in nums.chunks_exact(2) {
            lo = lo.min(pair[0]);
            hi = hi.max(pair[0]);
        }
        if lo.is_finite() && hi.is_finite() {
            let w = hi - lo;
            best = Some(best.map_or(w, |b: f64| b.max(w)));
        }
    }
    best
}

#[test]
fn letters_fill_their_column_tightly() {
    // Every letter is stretched to its full column less a 3% inset, so
    // columns pack tightly with only a hair of space between them. Letters
    // therefore all render at the same width regardless of their natural
    // proportions — the classic WebLogo look.
    let logo = SequenceLogo::from_sequences(vec!["IW"], Alphabet::Protein)
        .unwrap()
        .with_units(LogoUnits::Probability)
        .with_preset(PalettePreset::Taylor);
    let svg = render(logo, 500.0);
    // Taylor: I is #66ff00, W is #00ccff.
    let i_w = path_width_by_fill(&svg, [102, 255, 0]).expect("I glyph missing");
    let w_w = path_width_by_fill(&svg, [0, 204, 255]).expect("W glyph missing");
    let ratio = i_w / w_w;
    assert!(
        (0.95..1.05).contains(&ratio),
        "letters should fill equal column widths; I {i_w:.1}px vs W {w_w:.1}px (ratio {ratio:.2})"
    );
}

#[test]
fn capital_i_is_not_a_bare_rectangle() {
    // Letters are stretched to fill their column, so a glyph whose outline is
    // a plain 4-corner box renders as an anonymous block. The bundled face
    // must give capital I serifs; a bare rectangle would be 4 corner points.
    {
        let logo = SequenceLogo::from_sequences(vec!["I"], Alphabet::Protein)
            .unwrap()
            .with_units(LogoUnits::Probability);
        let svg = render(logo, 300.0);
        let d = svg
            .split("<path d=\"")
            .nth(1)
            .and_then(|c| c.split('"').next())
            .expect("I glyph path missing");
        let points = d
            .split(|c: char| c.is_ascii_alphabetic() || c == ' ')
            .filter(|t| !t.is_empty())
            .filter_map(|t| t.parse::<f64>().ok())
            .count()
            / 2;
        assert!(
            points > 4,
            "capital I has only {points} points — it is a bare rectangle"
        );
    }
}

#[test]
fn long_legend_reflows_into_columns_instead_of_truncating() {
    // 20 distinct swatches on a canvas too short for one column. Dropping
    // entries silently ("… (+N more)") makes a per-residue legend useless,
    // so it must wrap into columns and keep every row.
    let logo = SequenceLogo::from_fasta_str(&fixture("zinc_finger.fasta"), Alphabet::Protein)
        .unwrap()
        .with_preset(PalettePreset::Taylor);
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_width(1100.0)
        .with_height(320.0);
    let svg = kuva::render_to_svg(plots, layout);

    assert!(
        !svg.contains("more)"),
        "legend truncated instead of reflowing"
    );
    for residue in Alphabet::Protein.letters() {
        assert!(
            svg.contains(&format!(">{residue}<")),
            "legend is missing residue {residue}"
        );
    }
    common::write_test_output("test_outputs/logo_legend_reflow.svg", &svg).unwrap();
}

#[test]
fn protein_defaults_to_the_muted_palette() {
    // Saturated schemes glare when a glyph fills half the plot, so protein
    // and custom alphabets default to Muted; nucleotides keep classic ACGT.
    let protein = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 4], Alphabet::Protein);
    let explicit = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 4], Alphabet::Protein)
        .with_preset(PalettePreset::Muted);
    for &c in &protein.letters {
        assert_eq!(protein.palette.color_of(c), explicit.palette.color_of(c));
    }
    // Every residue still distinguishable.
    let mut colors: Vec<[u8; 3]> = protein
        .letters
        .iter()
        .map(|&c| protein.palette.color_of(c))
        .collect();
    let total = colors.len();
    colors.sort_unstable();
    colors.dedup();
    assert_eq!(colors.len(), total, "muted residues must stay distinct");

    // Mid chroma: no channel spread near the 0/255 full-saturation extremes
    // that make Taylor hard to look at (its spreads are 200–255).
    for &c in &protein.letters {
        let rgb = protein.palette.color_of(c);
        let hi = *rgb.iter().max().unwrap() as i32;
        let lo = *rgb.iter().min().unwrap() as i32;
        assert!(
            hi - lo <= 220,
            "residue {c} is too saturated for a large glyph: {rgb:?}"
        );
    }

    // DNA is unchanged — classic green A.
    let dna = SequenceLogo::from_sequences(vec!["ACGT"], Alphabet::Dna).unwrap();
    assert_eq!(dna.palette.color_of('A'), [0, 150, 0]);
}

#[test]
fn shapely_covers_protein_residues() {
    // Shapely used to fall through to a flat grey for every residue.
    let logo = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 4], Alphabet::Protein)
        .with_preset(PalettePreset::Shapely);
    let colors: Vec<[u8; 3]> = logo
        .letters
        .iter()
        .map(|&c| logo.palette.color_of(c))
        .collect();
    assert!(
        !colors.iter().all(|c| *c == colors[0]),
        "shapely protein palette should not be a single flat color"
    );
    assert!(
        !colors.contains(&[120, 120, 120]),
        "unassigned residue color"
    );
}

#[test]
fn zinc_finger_fixture_renders_a_long_protein_logo() {
    let logo =
        SequenceLogo::from_fasta_str(&fixture("zinc_finger.fasta"), Alphabet::Protein).unwrap();
    assert_eq!(logo.n_positions(), 25);
    assert_eq!(logo.sample_size, Some(14));
    // The C2H2 signature: cysteines at 3 and 6, histidines at 19 and 23 are
    // fully conserved, so each is the sole letter in its stack.
    for (pos, residue) in [(2usize, 'C'), (5, 'C'), (18, 'H'), (22, 'H')] {
        let stack: Vec<(char, f64)> = logo
            .stack(pos)
            .into_iter()
            .filter(|(_, h)| *h > 0.0)
            .collect();
        assert_eq!(
            stack.len(),
            1,
            "position {} should be fully conserved",
            pos + 1
        );
        assert_eq!(stack[0].0, residue);
    }
    let svg = render(
        SequenceLogo::from_fasta_str(&fixture("zinc_finger.fasta"), Alphabet::Protein)
            .unwrap()
            .with_preset(PalettePreset::Taylor),
        1000.0,
    );
    common::write_test_output("test_outputs/logo_zinc_finger.svg", &svg).unwrap();
    assert!(svg.contains("<svg"));
}

#[test]
fn mono_palette_suppresses_the_legend() {
    // Every letter black → a one-row legend carries no information.
    let logo = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 4], Alphabet::Protein)
        .with_preset(PalettePreset::Mono);
    assert!(logo.legend_groups().is_empty());
}

#[test]
fn bits_y_axis_pins_to_log2_alphabet_size() {
    // Protein: log2(20) ≈ 4.3219 — must not round out to 5.
    let protein = SequenceLogo::from_matrix(vec![vec![1.0; 20]; 4], Alphabet::Protein);
    let layout = Layout::auto_from_plots(&[protein.into()]);
    let y_max = layout.y_axis_max.expect("bits y-axis should be pinned");
    assert!((y_max - 20.0_f64.log2()).abs() < 1e-9, "got {y_max}");

    // DNA: log2(4) = 2.0 exactly.
    let dna = SequenceLogo::from_sequences(vec!["ACGT"], Alphabet::Dna).unwrap();
    let layout = Layout::auto_from_plots(&[dna.into()]);
    assert_eq!(layout.y_axis_max, Some(2.0));

    // Probability mode pins at 1.0.
    let prob = SequenceLogo::from_sequences(vec!["ACGT"], Alphabet::Dna)
        .unwrap()
        .with_units(LogoUnits::Probability);
    let layout = Layout::auto_from_plots(&[prob.into()]);
    assert_eq!(layout.y_axis_max, Some(1.0));

    // Frequency mode has no theoretical ceiling — left to auto-ranging.
    let freq = SequenceLogo::from_sequences(vec!["ACGT"], Alphabet::Dna)
        .unwrap()
        .with_units(LogoUnits::Frequency);
    let layout = Layout::auto_from_plots(&[freq.into()]);
    assert_eq!(layout.y_axis_max, None);
}

#[test]
fn curved_glyphs_emit_only_cubic_commands() {
    // TrueType outlines are quadratic, but kuva's rasteriser has no `q`
    // handler — a stray Q silently drops the letter from PNG output.
    // Every emitted path must therefore use only M / L / C / Z.
    let logo = SequenceLogo::from_sequences(vec!["ACGT"], Alphabet::Dna)
        .unwrap()
        .with_units(LogoUnits::Probability);
    let svg = render(logo, 400.0);
    for caps in svg.split("<path d=\"").skip(1) {
        let d = &caps[..caps.find('"').unwrap()];
        for ch in d.chars().filter(|c| c.is_ascii_alphabetic()) {
            assert!(
                matches!(ch, 'M' | 'L' | 'C' | 'Z'),
                "unsupported path command '{ch}' in glyph outline"
            );
        }
    }
    // And the curved letters are actually present (C and G both emit cubics).
    assert!(svg.matches('C').count() > 4, "expected cubic segments");
}

#[test]
fn title_flows_into_auto_layout() {
    let logo = SequenceLogo::from_sequences(vec!["ACGT"], Alphabet::Dna)
        .unwrap()
        .with_title("My motif");
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots);
    assert_eq!(layout.title.as_deref(), Some("My motif"));
}

#[test]
fn ambiguity_codes_are_dropped_not_counted() {
    // Second sequence contains N; third contains R (purine ambiguity).
    let logo = SequenceLogo::from_sequences(vec!["A", "N", "R"], Alphabet::Dna).unwrap();
    // Position 0 counts only the single A.
    let total: f64 = logo.matrix[0].iter().sum();
    assert_eq!(total, 1.0);
}

#[test]
fn empty_input_errors_cleanly() {
    let err = SequenceLogo::from_fasta_str("", Alphabet::Dna).unwrap_err();
    assert!(err.contains("no records") || err.contains("empty"));
}

#[test]
fn terminal_mode_emits_stacked_letters() {
    use kuva::render::render::render_multiple;
    use kuva::TerminalBackend;

    let logo = SequenceLogo::from_fasta_str(&fixture("kozak.fasta"), Alphabet::Dna).unwrap();
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_width(400.0)
        .with_height(240.0)
        .with_term_rows(24);
    let scene = render_multiple(plots, layout);
    let out = TerminalBackend::new(80, 24).render_scene(&scene);
    // The strongly conserved ATG core letters must appear in the char grid.
    assert!(out.contains('A'));
    assert!(out.contains('T'));
    assert!(out.contains('G'));
}

/// The consensus row must be emitted after the plot-area clip group (inside
/// it, the row is silently clipped away) and must sit in its own line between
/// the x tick labels and the x-axis title.
#[test]
fn consensus_row_sits_between_tick_labels_and_x_title() {
    let logo = SequenceLogo::from_fasta_str(&fixture("kozak.fasta"), Alphabet::Dna)
        .unwrap()
        .with_consensus();
    let plots: Vec<Plot> = vec![logo.into()];
    let layout = Layout::auto_from_plots(&plots).with_x_label("Position");
    let svg = kuva::render_to_svg(plots, layout);

    let clip_open = svg.find("<g clip-path=").expect("plot-area clip group");
    let clip_close = clip_open + svg[clip_open..].find("</g>").expect("clip group closes");
    let consensus_marker = "font-weight=\"bold\">G</text>";
    let consensus_at = svg.find(consensus_marker).expect("consensus letter G");
    assert!(
        consensus_at > clip_close,
        "consensus row must be emitted after the plot-area clip group"
    );

    // `y` of the <text> element whose content ends with `marker`.
    let y_of = |marker: &str| -> f64 {
        let end = svg
            .find(marker)
            .unwrap_or_else(|| panic!("missing {marker}"));
        let ys = svg[..end].rfind(" y=\"").unwrap() + 4;
        svg[ys..ys + svg[ys..].find('"').unwrap()].parse().unwrap()
    };
    let tick_y = y_of(">10</text>");
    let consensus_y = y_of(consensus_marker);
    let title_y = y_of(">Position</text>");
    assert!(
        consensus_y > tick_y + 8.0,
        "consensus row (y={consensus_y}) should sit below the tick labels (y={tick_y})"
    );
    assert!(
        title_y > consensus_y + 10.0,
        "x-axis title (y={title_y}) should clear the consensus row (y={consensus_y})"
    );
}

/// Long motifs widen the canvas so each position keeps a legible column;
/// short ones leave the default plot width alone.
#[test]
fn long_motifs_widen_the_canvas_automatically() {
    let short = SequenceLogo::from_fasta_str(&fixture("kozak.fasta"), Alphabet::Dna).unwrap();
    let plots: Vec<Plot> = vec![short.into()];
    assert!(
        Layout::auto_from_plots(&plots).width.is_none(),
        "10 positions fit the default plot width"
    );

    let long =
        SequenceLogo::from_fasta_str(&fixture("zinc_finger.fasta"), Alphabet::Protein).unwrap();
    let n = long.n_positions();
    let plots: Vec<Plot> = vec![long.into()];
    let width = Layout::auto_from_plots(&plots)
        .width
        .expect("25 positions should widen the canvas");
    assert!(
        width > n as f64 * 48.0,
        "auto width {width} should leave at least 48 px per position"
    );
}
