use std::io::Read;
use std::path::PathBuf;

use clap::Args;

use kuva::plot::logo::{
    detect_orientation, Alphabet, LogoUnits, MatrixOrientation, PalettePreset, SequenceLogo,
};
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use kuva::render::render::render_multiple;

use crate::layout_args::{apply_axis_args, apply_base_args, AxisArgs, BaseArgs};
use crate::output::write_output;

/// Sequence logo — stacked-letter plot of a PWM or aligned FASTA set.
#[derive(Args, Debug)]
pub struct LogoArgs {
    /// Input file. Extensions .fa/.fasta/.aln = FASTA; .csv/.tsv/.txt = matrix.
    /// Pass "-" or omit to read from stdin.
    pub input: Option<PathBuf>,

    /// Force input interpretation regardless of extension.
    #[arg(long, value_parser = ["matrix", "fasta"])]
    pub from: Option<String>,

    /// Alphabet: dna, rna, protein, or a raw letter string for custom alphabets
    /// (e.g. --alphabet MHUL for methylation states).
    #[arg(long, default_value = "dna")]
    pub alphabet: String,

    /// Y-axis units: bits (default), probability, or frequency.
    #[arg(long, value_parser = ["bits", "probability", "frequency"], default_value = "bits")]
    pub units: String,

    /// Per-letter color overrides, comma-separated. Example:
    /// --colors "A=#00CC00,C=#0000CC,G=#FFB300,T=#CC0000".
    #[arg(long)]
    pub colors: Option<String>,

    /// Starting position label (default: 1). Position 1 is displayed as this value.
    #[arg(long, default_value_t = 1)]
    pub start: i64,

    /// Reverse-complement the motif (nucleic alphabets only).
    #[arg(long)]
    pub reverse_complement: bool,

    /// Show the consensus-letter row below the plot.
    #[arg(long)]
    pub consensus: bool,

    /// Consensus threshold — letters above this fraction render uppercase (default: 0.5).
    #[arg(long, default_value_t = 0.5)]
    pub consensus_threshold: f64,

    /// Treat lowercase input as soft-masked (gaps) instead of uppercasing.
    #[arg(long)]
    pub respect_case: bool,

    /// Include gap characters as their own column instead of excluding them.
    #[arg(long)]
    pub include_gaps: bool,

    /// Skip Schneider's small-sample correction (FASTA input only, bits mode).
    #[arg(long)]
    pub no_correction: bool,

    /// Matrix orientation: rows=positions×cols=letters (WebLogo/MEME) or
    /// rows=letters×cols=positions (JASPAR). Autodetected from the header
    /// if omitted.
    #[arg(long, value_parser = ["rows", "cols"])]
    pub orientation: Option<String>,

    #[command(flatten)]
    pub base: BaseArgs,
    #[command(flatten)]
    pub axis: AxisArgs,
}

pub fn run(mut args: LogoArgs) -> Result<(), String> {
    let alphabet = parse_alphabet(&args.alphabet)?;
    let units = parse_units(&args.units);

    // `--palette` is repurposed for logo presets (classic, weblogo, shapely,
    // clustal, mono).  Take it out of BaseArgs so apply_base_args doesn't
    // try to interpret a preset name as a chrome palette.
    let preset = match args.base.palette.take() {
        Some(name) => parse_preset(&name)?,
        None => PalettePreset::default_for(&alphabet),
    };

    let content = read_input(args.input.as_deref())?;
    let mode = resolve_mode(args.from.as_deref(), args.input.as_deref(), &content);

    let is_matrix = matches!(mode, InputMode::Matrix);
    let mut logo = match mode {
        InputMode::Fasta => SequenceLogo::from_fasta_str(&content, alphabet.clone())?,
        InputMode::Matrix => build_from_matrix(&content, &alphabet, args.orientation.as_deref())?,
    };

    if is_matrix && (args.respect_case || args.include_gaps) {
        eprintln!(
            "kuva logo: --respect-case / --include-gaps have no effect on matrix input \
             (case and gap information is already gone from a matrix)"
        );
    }

    logo = logo
        .with_preset(preset)
        .with_units(units)
        .with_start(args.start)
        .with_consensus_threshold(args.consensus_threshold);
    if args.consensus {
        logo = logo.with_consensus();
    }
    if args.respect_case {
        logo = logo.with_respect_case();
    }
    if args.include_gaps {
        logo = logo.with_include_gaps();
    }
    if args.no_correction {
        logo = logo.with_no_correction();
    }
    if args.reverse_complement {
        logo = logo.with_reverse_complement()?;
    }
    let overrides = match args.colors.as_deref() {
        Some(spec) => parse_color_overrides(spec)?,
        None => Vec::new(),
    };
    if !overrides.is_empty() {
        let mut palette = logo.palette.clone();
        palette.override_colors(overrides.iter().copied());
        logo = logo.with_palette(palette);
    }

    #[cfg(feature = "emit_code")]
    if args.base.emit_code {
        // Mirror the axis-label defaults the rendered layout gets below.
        if args.axis.y_label.is_none() {
            args.axis.y_label = Some(default_y_label(units).to_string());
        }
        if args.axis.x_label.is_none() {
            args.axis.x_label = Some("Position".to_string());
        }
        print!(
            "{}",
            crate::emit_code::assemble(
                &[
                    "kuva::plot::logo::SequenceLogo",
                    "kuva::plot::logo::Alphabet",
                    "kuva::plot::logo::LogoUnits",
                    "kuva::plot::logo::PalettePreset",
                ],
                "SequenceLogo",
                &[crate::emit_code::emit_sequence_logo(
                    &logo, preset, &overrides
                )],
                &args.base,
                Some(&args.axis),
                None,
            )
        );
        return Ok(());
    }

    let plots: Vec<Plot> = vec![logo.into()];
    let mut layout = Layout::auto_from_plots(&plots);
    layout = apply_base_args(layout, &args.base);
    layout = apply_axis_args(layout, &args.axis);
    if layout.y_label.is_none() {
        layout.y_label = Some(default_y_label(units).to_string());
    }
    if layout.x_label.is_none() {
        layout.x_label = Some("Position".to_string());
    }
    let scene = render_multiple(plots, layout);
    write_output(scene, &args.base)
}

// ── Input handling ──────────────────────────────────────────────────────────

enum InputMode {
    Fasta,
    Matrix,
}

fn resolve_mode(
    explicit: Option<&str>,
    path: Option<&std::path::Path>,
    content: &str,
) -> InputMode {
    if let Some(kind) = explicit {
        return match kind {
            "fasta" => InputMode::Fasta,
            _ => InputMode::Matrix,
        };
    }
    if let Some(p) = path {
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            match ext.to_ascii_lowercase().as_str() {
                "fa" | "fasta" | "fna" | "faa" | "aln" | "afa" => return InputMode::Fasta,
                "csv" | "tsv" | "txt" | "pwm" | "ppm" => return InputMode::Matrix,
                _ => {}
            }
        }
    }
    // Content sniff: a FASTA blob has a leading '>' on the first non-empty line.
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        return if t.starts_with('>') {
            InputMode::Fasta
        } else {
            InputMode::Matrix
        };
    }
    InputMode::Fasta
}

fn read_input(path: Option<&std::path::Path>) -> Result<String, String> {
    match path {
        Some(p) if p.to_str() != Some("-") => {
            std::fs::read_to_string(p).map_err(|e| format!("Cannot read {}: {e}", p.display()))
        }
        _ => {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .map_err(|e| format!("Cannot read stdin: {e}"))?;
            Ok(s)
        }
    }
}

// ── Matrix parsing ──────────────────────────────────────────────────────────

fn build_from_matrix(
    text: &str,
    alphabet: &Alphabet,
    orientation_flag: Option<&str>,
) -> Result<SequenceLogo, String> {
    let (delim, _) = sniff_delimiter(text);
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        rows.push(t.split(delim).map(|s| s.trim().to_string()).collect());
    }
    if rows.is_empty() {
        return Err("Matrix input has no non-empty rows".to_string());
    }

    let letters = alphabet.letters();
    let orientation = if let Some(k) = orientation_flag {
        match k {
            "rows" => MatrixOrientation::RowsArePositions,
            _ => MatrixOrientation::RowsAreLetters,
        }
    } else {
        // Try autodetect from row/col headers.
        let first_row = &rows[0];
        let first_col: Vec<String> = rows
            .iter()
            .map(|r| r.first().cloned().unwrap_or_default())
            .collect();
        detect_orientation(first_row, &first_col, &letters)
            .unwrap_or(MatrixOrientation::RowsArePositions)
    };

    let matrix = shape_matrix(rows, orientation, &letters)?;
    Ok(SequenceLogo::from_matrix(matrix, alphabet.clone()))
}

fn shape_matrix(
    mut rows: Vec<Vec<String>>,
    orientation: MatrixOrientation,
    letters: &[char],
) -> Result<Vec<Vec<f64>>, String> {
    if orientation == MatrixOrientation::RowsArePositions {
        // Row 0 might be letters (header) — strip it if so.
        if row_is_all_alphabet(&rows[0], letters) {
            rows.remove(0);
        }
    } else {
        // Column 0 might be letter labels; strip it.
        if column_is_all_alphabet(&rows, 0, letters) {
            for r in rows.iter_mut() {
                if !r.is_empty() {
                    r.remove(0);
                }
            }
        }
    }

    match orientation {
        MatrixOrientation::RowsArePositions => {
            let mut matrix = Vec::with_capacity(rows.len());
            for (i, r) in rows.iter().enumerate() {
                let row_f = parse_row_f64(r, i)?;
                if row_f.len() != letters.len() {
                    return Err(format!(
                        "Row {i}: expected {} values (alphabet size), got {}",
                        letters.len(),
                        row_f.len()
                    ));
                }
                matrix.push(row_f);
            }
            Ok(matrix)
        }
        MatrixOrientation::RowsAreLetters => {
            if rows.len() != letters.len() {
                return Err(format!(
                    "JASPAR orientation: expected {} rows (one per letter), got {}",
                    letters.len(),
                    rows.len()
                ));
            }
            let n_positions = rows[0].len();
            let mut matrix = vec![vec![0.0_f64; letters.len()]; n_positions];
            for (li, row) in rows.iter().enumerate() {
                let row_f = parse_row_f64(row, li)?;
                if row_f.len() != n_positions {
                    return Err(format!(
                        "Letter row {li}: expected {n_positions} values, got {}",
                        row_f.len()
                    ));
                }
                for (pos, v) in row_f.iter().enumerate() {
                    matrix[pos][li] = *v;
                }
            }
            Ok(matrix)
        }
    }
}

fn parse_row_f64(row: &[String], row_idx: usize) -> Result<Vec<f64>, String> {
    row.iter()
        .enumerate()
        .map(|(ci, s)| {
            s.parse::<f64>()
                .map_err(|_| format!("Row {row_idx} col {ci}: cannot parse '{s}' as a number"))
        })
        .collect()
}

fn row_is_all_alphabet(row: &[String], letters: &[char]) -> bool {
    !row.is_empty()
        && row.iter().all(|s| {
            let mut cs = s.chars();
            match (cs.next(), cs.next()) {
                (Some(c), None) => letters.contains(&c.to_ascii_uppercase()),
                _ => false,
            }
        })
}

fn column_is_all_alphabet(rows: &[Vec<String>], col: usize, letters: &[char]) -> bool {
    !rows.is_empty()
        && rows.iter().all(|r| {
            r.get(col)
                .map(|s| {
                    let mut cs = s.chars();
                    matches!(
                        (cs.next(), cs.next()),
                        (Some(c), None) if letters.contains(&c.to_ascii_uppercase())
                    )
                })
                .unwrap_or(false)
        })
}

fn sniff_delimiter(text: &str) -> (char, usize) {
    let mut tab = 0;
    let mut comma = 0;
    for line in text.lines().take(20) {
        tab += line.matches('\t').count();
        comma += line.matches(',').count();
    }
    if tab > comma {
        ('\t', tab)
    } else {
        (',', comma)
    }
}

// ── Enum + preset + color parsing ───────────────────────────────────────────

fn parse_alphabet(s: &str) -> Result<Alphabet, String> {
    match s.to_ascii_lowercase().as_str() {
        "dna" => Ok(Alphabet::Dna),
        "rna" => Ok(Alphabet::Rna),
        "protein" | "aa" | "amino" => Ok(Alphabet::Protein),
        other => {
            let letters: Vec<char> = other.chars().filter(|c| !c.is_whitespace()).collect();
            if letters.is_empty() {
                return Err(format!("Unrecognised alphabet: '{s}'"));
            }
            Ok(Alphabet::Custom(letters))
        }
    }
}

fn parse_units(s: &str) -> LogoUnits {
    match s {
        "probability" => LogoUnits::Probability,
        "frequency" => LogoUnits::Frequency,
        _ => LogoUnits::Bits,
    }
}

fn parse_preset(s: &str) -> Result<PalettePreset, String> {
    match s.to_ascii_lowercase().as_str() {
        "classic" => Ok(PalettePreset::Classic),
        "weblogo" => Ok(PalettePreset::WebLogo),
        "shapely" => Ok(PalettePreset::Shapely),
        "clustal" => Ok(PalettePreset::Clustal),
        "taylor" => Ok(PalettePreset::Taylor),
        "muted" => Ok(PalettePreset::Muted),
        "mono" => Ok(PalettePreset::Mono),
        other => Err(format!(
            "Unknown logo palette '{other}'. Available: muted, classic, weblogo, shapely, clustal, taylor, mono"
        )),
    }
}

fn parse_color_overrides(spec: &str) -> Result<Vec<(char, [u8; 3])>, String> {
    let mut out = Vec::new();
    for entry in spec.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (letter, color) = entry
            .split_once('=')
            .ok_or_else(|| format!("--colors entry '{entry}' missing '='; expected 'A=#RRGGBB'"))?;
        let mut cs = letter.trim().chars();
        let c = match (cs.next(), cs.next()) {
            (Some(c), None) => c.to_ascii_uppercase(),
            _ => return Err(format!("--colors: '{letter}' is not a single character")),
        };
        let rgb = parse_hex_color(color.trim())?;
        out.push((c, rgb));
    }
    Ok(out)
}

fn parse_hex_color(s: &str) -> Result<[u8; 3], String> {
    let hex = s.trim().trim_start_matches('#');
    if hex.len() != 6 {
        return Err(format!("expected #RRGGBB, got '{s}'"));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| format!("bad hex byte in '{s}'"))?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| format!("bad hex byte in '{s}'"))?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| format!("bad hex byte in '{s}'"))?;
    Ok([r, g, b])
}

fn default_y_label(units: LogoUnits) -> &'static str {
    match units {
        LogoUnits::Probability => "Probability",
        LogoUnits::Frequency => "Count",
        LogoUnits::Bits => "Information (bits)",
    }
}
