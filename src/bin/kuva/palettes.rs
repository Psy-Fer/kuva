//! `kuva palettes`: preview the named palettes, colormaps and logo colour
//! schemes in the terminal, so picking one doesn't need a trip to the docs.
//!
//! Every list here is read from the same tables the plotting subcommands
//! parse names from (`NAMED_PALETTES`, `CVD_PALETTES`, `NAMED_COLORMAPS`,
//! `LOGO_PRESETS`), so a new palette shows up here automatically.

use std::io::{IsTerminal, Write};

use clap::{Args, ValueEnum};
use kuva::plot::logo::{Alphabet, AlphabetPalette};

use crate::data::{NamedColormap, NAMED_COLORMAPS};
use crate::layout_args::{find_cvd_palette, find_named_palette, CVD_PALETTES, NAMED_PALETTES};
use crate::logo::LOGO_PRESETS;

/// Width of a colormap gradient bar, in terminal cells (two samples per cell).
const GRADIENT_CELLS: usize = 40;
/// Stops listed for a colormap when colour output is off or `--hex` is set.
const GRADIENT_HEX_STOPS: usize = 5;
/// Column the swatches start at: two-space indent, the longest name, two spaces.
const NAME_WIDTH: usize = 2 + "yellow-orange-brown".len() + 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    /// Colour when stdout is a terminal and `NO_COLOR` is unset.
    Auto,
    Always,
    Never,
}

/// Preview the named colour palettes, colormaps and logo schemes.
///
/// With no arguments, lists the `--palette` and `--cvd-palette` names.
/// Name one or more entries (from any list) to show just those with their
/// hex codes. Swatches use 24-bit terminal colour; when output is piped or
/// `NO_COLOR` is set, names and hex codes are printed instead.
#[derive(Args, Debug)]
pub struct PalettesArgs {
    /// Show only these palettes, colormaps or logo schemes (implies --hex).
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,

    /// List the `--colormap` names (heatmap, hist2d, contour, ...) instead.
    #[arg(long, conflicts_with = "names")]
    pub colormaps: bool,

    /// List the `kuva logo --palette` colour schemes instead.
    #[arg(long, conflicts_with = "names")]
    pub logo: bool,

    /// List palettes, colormaps and logo schemes together.
    #[arg(long, conflicts_with_all = ["names", "colormaps", "logo"])]
    pub all: bool,

    /// Print hex codes under each swatch.
    #[arg(long)]
    pub hex: bool,

    /// When to use terminal colour.
    #[arg(long, value_enum, default_value_t = ColorWhen::Auto)]
    pub color: ColorWhen,
}

pub fn run(args: PalettesArgs) -> Result<(), String> {
    let color = match args.color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => {
            std::io::stdout().is_terminal()
                && std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
        }
    };
    let mut out = Printer {
        buf: String::new(),
        color,
        hex: args.hex || !args.names.is_empty() || !color,
    };

    if !args.names.is_empty() {
        out.named(&args.names)?;
    } else {
        let palettes = args.all || !(args.colormaps || args.logo);
        let mut first = true;
        if palettes {
            out.palettes();
            first = false;
        }
        if args.all || args.colormaps {
            if !first {
                out.buf.push('\n');
            }
            out.colormaps(NAMED_COLORMAPS.iter());
            first = false;
        }
        if args.all || args.logo {
            if !first {
                out.buf.push('\n');
            }
            out.logo_presets(LOGO_PRESETS.iter());
        }
    }

    let mut stdout = std::io::stdout().lock();
    // A closed pipe (`kuva palettes | head`) is not an error worth reporting.
    let _ = stdout.write_all(out.buf.as_bytes());
    Ok(())
}

struct Printer {
    buf: String,
    color: bool,
    hex: bool,
}

impl Printer {
    fn heading(&mut self, text: &str) {
        if self.color {
            self.buf.push_str(&format!("\x1b[1m{text}\x1b[0m\n"));
        } else {
            self.buf.push_str(&format!("{text}\n"));
        }
    }

    fn name(&mut self, name: &str) {
        self.buf
            .push_str(&format!("  {name:<w$}", w = NAME_WIDTH - 2));
    }

    fn hex_line(&mut self, colors: &[String]) {
        if self.hex {
            // Plain output already has the name on this line; coloured
            // output put the swatches there, so indent under them.
            if self.color {
                self.buf.push_str(&" ".repeat(NAME_WIDTH));
            }
            self.buf.push_str(&colors.join(" "));
            self.buf.push('\n');
        }
    }

    /// One categorical palette: name, a block per colour, colour count.
    fn palette_row(&mut self, name: &str, colors: &[String]) {
        self.name(name);
        if self.color {
            for c in colors {
                let (r, g, b) = parse_hex(c);
                self.buf
                    .push_str(&format!("\x1b[38;2;{r};{g};{b}m██\x1b[0m "));
            }
            self.buf.push_str(&format!(" ({})\n", colors.len()));
        }
        self.hex_line(colors);
    }

    fn palettes(&mut self) {
        let mut group = "";
        for p in NAMED_PALETTES {
            if p.group != group {
                if !group.is_empty() {
                    self.buf.push('\n');
                }
                group = p.group;
                let title = match group {
                    "LTC" => "LTC palettes (also accept an ltc- prefix)",
                    g => g,
                };
                self.heading(&format!("--palette: {title}"));
            }
            self.palette_row(p.name, (p.build)().colors());
        }
        self.buf.push('\n');
        self.heading("--cvd-palette (overrides --palette)");
        for c in CVD_PALETTES {
            self.palette_row(c.name, (c.build)().colors());
        }
    }

    fn colormaps<'a>(&mut self, maps: impl Iterator<Item = &'a NamedColormap>) {
        let mut group = "";
        for m in maps {
            if m.group != group {
                if !group.is_empty() {
                    self.buf.push('\n');
                }
                group = m.group;
                self.heading(&format!("--colormap: {group}"));
            }
            let cmap = (m.build)();
            let rgb = |t: f64| cmap.map_rgb(t).unwrap_or((0, 0, 0));
            self.name(m.name);
            if self.color {
                // Half blocks: the left half takes the foreground colour and
                // the right half the background, so each cell shows two samples.
                let n = GRADIENT_CELLS * 2 - 1;
                for i in 0..GRADIENT_CELLS {
                    let (r1, g1, b1) = rgb((2 * i) as f64 / n as f64);
                    let (r2, g2, b2) = rgb((2 * i + 1) as f64 / n as f64);
                    self.buf
                        .push_str(&format!("\x1b[38;2;{r1};{g1};{b1};48;2;{r2};{g2};{b2}m▌"));
                }
                self.buf.push_str("\x1b[0m");
                if !m.aliases.is_empty() {
                    self.buf.push_str(&format!("  {}", m.aliases.join(", ")));
                }
                self.buf.push('\n');
            }
            if self.hex {
                let stops: Vec<String> = (0..GRADIENT_HEX_STOPS)
                    .map(|i| {
                        let (r, g, b) = rgb(i as f64 / (GRADIENT_HEX_STOPS - 1) as f64);
                        format!("#{r:02x}{g:02x}{b:02x}")
                    })
                    .collect();
                self.hex_line(&stops);
            }
        }
    }

    fn logo_presets<'a>(
        &mut self,
        presets: impl Iterator<Item = &'a (&'static str, kuva::plot::logo::PalettePreset)>,
    ) {
        self.heading("kuva logo --palette (default: classic for DNA/RNA, muted for protein)");
        for (name, preset) in presets {
            for (i, alphabet) in [Alphabet::Dna, Alphabet::Protein].iter().enumerate() {
                if i == 0 {
                    self.name(name);
                } else {
                    self.buf.push_str(&" ".repeat(NAME_WIDTH));
                }
                let pal = AlphabetPalette::preset(*preset, alphabet);
                let letters = alphabet.letters();
                if self.color {
                    for &c in &letters {
                        let [r, g, b] = pal.color_of(c);
                        // Pick black or white text by luminance so every
                        // letter stays readable on its own background.
                        let lum = 0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64;
                        let fg = if lum > 140.0 { "30" } else { "97" };
                        self.buf
                            .push_str(&format!("\x1b[1;{fg};48;2;{r};{g};{b}m {c} \x1b[0m"));
                    }
                    self.buf.push('\n');
                }
                if self.hex {
                    if self.color {
                        self.buf.push_str(&" ".repeat(NAME_WIDTH));
                    }
                    let pairs: Vec<String> = letters
                        .iter()
                        .map(|&c| {
                            let [r, g, b] = pal.color_of(c);
                            format!("{c}=#{r:02x}{g:02x}{b:02x}")
                        })
                        .collect();
                    self.buf.push_str(&pairs.join(" "));
                    self.buf.push('\n');
                }
            }
        }
    }

    /// Show the named entries, searching every list. Unknown names are an
    /// error that lists where to look.
    fn named(&mut self, names: &[String]) -> Result<(), String> {
        let mut unknown = Vec::new();
        for name in names {
            if !self.buf.is_empty() && !self.buf.ends_with("\n\n") {
                self.buf.push('\n');
            }
            let lower = name.to_ascii_lowercase();
            let mut found = false;
            if let Some(p) = find_named_palette(&lower) {
                self.heading(&format!("--palette {}", p.name));
                self.palette_row(p.name, (p.build)().colors());
                found = true;
            }
            if let Some(c) = find_cvd_palette(&lower) {
                self.heading(&format!("--cvd-palette {}", c.name));
                self.palette_row(c.name, (c.build)().colors());
                found = true;
            }
            let key = lower.replace('_', "-");
            if let Some(m) = NAMED_COLORMAPS
                .iter()
                .find(|m| m.name == key || m.aliases.contains(&key.as_str()))
            {
                self.colormaps(std::iter::once(m));
                found = true;
            }
            if let Some(entry) = LOGO_PRESETS.iter().find(|(n, _)| *n == lower) {
                self.logo_presets(std::iter::once(entry));
                found = true;
            }
            if !found {
                unknown.push(name.as_str());
            }
        }
        if unknown.is_empty() {
            Ok(())
        } else {
            // Still print what was found before failing.
            let _ = std::io::stdout().lock().write_all(self.buf.as_bytes());
            self.buf.clear();
            Err(format!(
                "unknown palette, colormap or logo scheme: {}. \
                Run `kuva palettes --all` to list every name.",
                unknown.join(", ")
            ))
        }
    }
}

/// `#rrggbb` → bytes. Every built-in palette uses this form (checked by a test).
fn parse_hex(s: &str) -> (u8, u8, u8) {
    let h = s.trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    (byte(0), byte(2), byte(4))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_palette_colour_is_rrggbb_hex() {
        let all = NAMED_PALETTES
            .iter()
            .map(|p| (p.name, (p.build)()))
            .chain(CVD_PALETTES.iter().map(|c| (c.name, (c.build)())));
        for (name, pal) in all {
            for c in pal.colors() {
                assert!(
                    c.len() == 7
                        && c.starts_with('#')
                        && c[1..].chars().all(|ch| ch.is_ascii_hexdigit()),
                    "{name}: '{c}' is not #rrggbb"
                );
            }
        }
    }

    #[test]
    fn every_listed_name_resolves() {
        for p in NAMED_PALETTES {
            assert!(find_named_palette(p.name).is_some(), "{}", p.name);
            assert!(
                find_named_palette(&p.name.replace('-', "_")).is_some(),
                "{}",
                p.name
            );
        }
        for m in NAMED_COLORMAPS {
            assert!(
                crate::data::colormap_from_name(m.name).is_some(),
                "{}",
                m.name
            );
            for a in m.aliases {
                assert!(crate::data::colormap_from_name(a).is_some(), "{a}");
            }
        }
        assert!(find_named_palette("ltc-paloma").is_some());
        assert!(
            find_named_palette("ltc-wong").is_none(),
            "ltc- prefix is LTC-only"
        );
        assert!(find_cvd_palette("deuter").is_some());
        let longest = NAMED_PALETTES
            .iter()
            .map(|p| p.name)
            .chain(NAMED_COLORMAPS.iter().map(|m| m.name))
            .chain(LOGO_PRESETS.iter().map(|(n, _)| *n))
            .map(str::len)
            .max()
            .unwrap();
        assert_eq!(
            NAME_WIDTH,
            longest + 4,
            "NAME_WIDTH must fit the longest name"
        );
    }
}
