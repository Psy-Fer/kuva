use std::ops::Index;

pub struct Palette {
    pub name: &'static str,
    colors: Vec<String>,
}

impl Palette {
    pub fn custom(name: &'static str, colors: Vec<String>) -> Self {
        Self { name, colors }
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn colors(&self) -> &[String] {
        &self.colors
    }

    pub fn iter(&self) -> PaletteCycleIter<'_> {
        PaletteCycleIter {
            palette: self,
            index: 0,
        }
    }

    // ── Colorblind-safe palettes ──

    /// Bang Wong, Nature Methods 2011 — 8 colors, colorblind-safe.
    pub fn wong() -> Self {
        Self {
            name: "wong",
            colors: vec![
                "#E69F00", "#56B4E9", "#009E73", "#F0E442", "#0072B2", "#D55E00", "#CC79A7",
                "#000000",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        }
    }

    /// Alias for Wong (same palette, widely known as Okabe-Ito).
    pub fn okabe_ito() -> Self {
        let mut p = Self::wong();
        p.name = "okabe_ito";
        p
    }

    /// Paul Tol qualitative bright — 7 colors.
    pub fn tol_bright() -> Self {
        Self {
            name: "tol_bright",
            colors: vec![
                "#4477AA", "#EE6677", "#228833", "#CCBB44", "#66CCEE", "#AA3377", "#BBBBBB",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        }
    }

    /// Paul Tol qualitative muted — 10 colors.
    pub fn tol_muted() -> Self {
        Self {
            name: "tol_muted",
            colors: vec![
                "#CC6677", "#332288", "#DDCC77", "#117733", "#88CCEE", "#882255", "#44AA99",
                "#999933", "#AA4499", "#DDDDDD",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        }
    }

    /// Paul Tol qualitative light — 9 colors.
    pub fn tol_light() -> Self {
        Self {
            name: "tol_light",
            colors: vec![
                "#77AADD", "#EE8866", "#EEDD88", "#FFAABB", "#99DDFF", "#44BB99", "#BBCC33",
                "#AAAA00", "#DDDDDD",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        }
    }

    /// IBM Design Language — 5 colors.
    pub fn ibm() -> Self {
        Self {
            name: "ibm",
            colors: vec!["#648FFF", "#785EF0", "#DC267F", "#FE6100", "#FFB000"]
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }

    // ── By colorblind condition ──

    /// Safe for deuteranopia (red-green, most common ~6% males).
    pub fn deuteranopia() -> Self {
        let mut p = Self::wong();
        p.name = "deuteranopia";
        p
    }

    /// Safe for protanopia (red-green, ~1% males).
    pub fn protanopia() -> Self {
        let mut p = Self::wong();
        p.name = "protanopia";
        p
    }

    /// Safe for tritanopia (blue-yellow, rare).
    pub fn tritanopia() -> Self {
        let mut p = Self::tol_bright();
        p.name = "tritanopia";
        p
    }

    // ── General-purpose palettes ──

    /// Tableau 10 / D3 Category10 — 10 colors.
    pub fn category10() -> Self {
        Self {
            name: "category10",
            colors: vec![
                "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2",
                "#7f7f7f", "#bcbd22", "#17becf",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        }
    }

    /// Softer pastel version — 10 colors.
    pub fn pastel() -> Self {
        Self {
            name: "pastel",
            colors: vec![
                "#aec7e8", "#ffbb78", "#98df8a", "#ff9896", "#c5b0d5", "#c49c94", "#f7b6d2",
                "#c7c7c7", "#dbdb8d", "#9edae5",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        }
    }

    /// High-saturation vivid — 10 colors.
    pub fn bold() -> Self {
        Self {
            name: "bold",
            colors: vec![
                "#e41a1c", "#377eb8", "#4daf4a", "#984ea3", "#ff7f00", "#a65628", "#f781bf",
                "#999999", "#66c2a5", "#fc8d62",
            ]
            .into_iter()
            .map(Into::into)
            .collect(),
        }
    }

    // ── LTC palettes (Louis's Themed Colours) ──
    //
    // Curated qualitative palettes from loukesio/ltc-color-palettes (MIT licensed), named after
    // Picasso's muses and a handful of places. Hex values copied verbatim from the source's
    // `R/ltc_functions.R`. Great for small-cardinality categorical series where the default
    // ColorBrewer/Tableau scales feel clinical.

    /// Build a palette from static hex literals (internal helper for the fixed LTC set).
    fn from_hexes(name: &'static str, hexes: &[&str]) -> Self {
        Self {
            name,
            colors: hexes.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// LTC "paloma" — soft coral, sage, and butter; 5 colors.
    pub fn paloma() -> Self {
        Self::from_hexes(
            "paloma",
            &["#83AF9B", "#C8C8A9", "#F8DA8A", "#F7BF95", "#FE8CA1"],
        )
    }

    /// LTC "maya" — navy, sky, ice, coral, ink; 5 colors.
    pub fn maya() -> Self {
        Self::from_hexes(
            "maya",
            &["#3D5A80", "#98C1D9", "#E0FBFC", "#EE6C4D", "#293241"],
        )
    }

    /// LTC "dora" — teal, plum, and warm reds; 5 colors.
    pub fn dora() -> Self {
        Self::from_hexes(
            "dora",
            &["#52777A", "#542437", "#C02942", "#D95B43", "#ECD078"],
        )
    }

    /// LTC "ploen" — slate blues, mauve, and peach; 5 colors.
    pub fn ploen() -> Self {
        Self::from_hexes(
            "ploen",
            &["#3F5671", "#83A1C3", "#CEB5C8", "#FAC898", "#B17776"],
        )
    }

    /// LTC "olga" — pastel green, teal, gold, apricot, mauve; 5 colors.
    pub fn olga() -> Self {
        Self::from_hexes(
            "olga",
            &["#C9E3C2", "#8BC8CB", "#ECCD80", "#F5AB70", "#9C87A1"],
        )
    }

    /// LTC "mterese" — cream, apricot, pink, and cornflower; 5 colors.
    pub fn mterese() -> Self {
        Self::from_hexes(
            "mterese",
            &["#F7DDAA", "#FAC3AD", "#F897A1", "#9298BA", "#9CBEED"],
        )
    }

    /// LTC "franscoise" — blues, mauve, and warm reds; 5 colors.
    pub fn franscoise() -> Self {
        Self::from_hexes(
            "franscoise",
            &["#5980B1", "#B96A8D", "#A55062", "#E05256", "#E9A986"],
        )
    }

    /// LTC "fernande" — coral, yellow, green, blue; 4 colors.
    pub fn fernande() -> Self {
        Self::from_hexes("fernande", &["#FF7676", "#F9D662", "#7CAB7D", "#75B7D1"])
    }

    /// LTC "sylvie" — gold, coral, lavender, teal, pink; 5 colors.
    pub fn sylvie() -> Self {
        Self::from_hexes(
            "sylvie",
            &["#E8B961", "#E88170", "#C6BDE8", "#5DB7C4", "#FD95BC"],
        )
    }

    /// LTC "expevo" — high-contrast orange, gold, teal, plum, navy, grey; 6 colors.
    pub fn expevo() -> Self {
        Self::from_hexes(
            "expevo",
            &[
                "#FC4E07", "#E7B800", "#00AFBB", "#8B4769", "#1D457F", "#808080",
            ],
        )
    }

    /// LTC "minou" — teal, red, gold, green, ink, slate; 6 colors.
    pub fn minou() -> Self {
        Self::from_hexes(
            "minou",
            &[
                "#00798C", "#D1495B", "#EDAE49", "#66A182", "#2E4057", "#8D96A3",
            ],
        )
    }

    /// LTC "alger" — black, teal, sage, gold, red; 5 colors.
    pub fn alger() -> Self {
        Self::from_hexes(
            "alger",
            &["#000000", "#1A5B5B", "#ACC8BE", "#F4AB5C", "#D1422F"],
        )
    }

    /// LTC "seafarer" — deep teal, cream, sage, moss, sand; 5 colors.
    pub fn seafarer() -> Self {
        Self::from_hexes(
            "seafarer",
            &["#013D5A", "#FCF3E3", "#BDD3CE", "#708C69", "#E4A25B"],
        )
    }

    /// LTC "luminaries" — orange, teal, charcoal, ivory, gold, mist; 6 colors.
    pub fn luminaries() -> Self {
        Self::from_hexes(
            "luminaries",
            &[
                "#FF5B04", "#075056", "#233038", "#FDF6E3", "#F4D47C", "#D3DBDD",
            ],
        )
    }

    /// LTC "casa_natal" — vibrant 9-colour set for larger categorical scales.
    pub fn casa_natal() -> Self {
        Self::from_hexes(
            "casa_natal",
            &[
                "#245E55", "#ED773C", "#808BC5", "#C63F3E", "#EAC119", "#EAA7C7", "#9ED6DF",
                "#1D1D1B", "#EAE4DA",
            ],
        )
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::category10()
    }
}

impl Index<usize> for Palette {
    type Output = str;

    fn index(&self, index: usize) -> &str {
        &self.colors[index % self.colors.len()]
    }
}

/// Cycling iterator that wraps around the palette endlessly.
pub struct PaletteCycleIter<'a> {
    palette: &'a Palette,
    index: usize,
}

impl<'a> Iterator for PaletteCycleIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let color = &self.palette[self.index];
        self.index += 1;
        Some(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every LTC palette: non-empty, valid `#rrggbb` hex, and a name matching its constructor.
    #[test]
    fn ltc_palettes_are_well_formed() {
        let expected: &[(Palette, &str, usize)] = &[
            (Palette::paloma(), "paloma", 5),
            (Palette::maya(), "maya", 5),
            (Palette::dora(), "dora", 5),
            (Palette::ploen(), "ploen", 5),
            (Palette::olga(), "olga", 5),
            (Palette::mterese(), "mterese", 5),
            (Palette::franscoise(), "franscoise", 5),
            (Palette::fernande(), "fernande", 4),
            (Palette::sylvie(), "sylvie", 5),
            (Palette::expevo(), "expevo", 6),
            (Palette::minou(), "minou", 6),
            (Palette::alger(), "alger", 5),
            (Palette::seafarer(), "seafarer", 5),
            (Palette::luminaries(), "luminaries", 6),
            (Palette::casa_natal(), "casa_natal", 9),
        ];
        for (pal, name, n) in expected {
            assert_eq!(pal.name, *name, "name mismatch for {name}");
            assert_eq!(pal.len(), *n, "colour count mismatch for {name}");
            for c in pal.colors() {
                assert_eq!(c.len(), 7, "{name}: {c} is not #rrggbb");
                assert!(c.starts_with('#'), "{name}: {c} missing #");
                assert!(
                    c[1..].chars().all(|ch| ch.is_ascii_hexdigit()),
                    "{name}: {c} has non-hex digits"
                );
            }
        }
    }

    /// Indexing wraps and never panics past the end.
    #[test]
    fn ltc_indexing_wraps() {
        let pal = Palette::fernande();
        assert_eq!(&pal[0], &pal[4]); // 4 colours, so index 4 wraps to 0
    }
}
