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
        PaletteCycleIter { palette: self, index: 0 }
    }

    // ── Colorblind-safe palettes ──

    /// Bang Wong, Nature Methods 2011 — 8 colors, colorblind-safe.
    pub fn wong() -> Self {
        Self {
            name: "wong",
            colors: vec![
                "#E69F00", "#56B4E9", "#009E73", "#F0E442",
                "#0072B2", "#D55E00", "#CC79A7", "#000000",
            ].into_iter().map(Into::into).collect(),
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
                "#4477AA", "#EE6677", "#228833", "#CCBB44",
                "#66CCEE", "#AA3377", "#BBBBBB",
            ].into_iter().map(Into::into).collect(),
        }
    }

    /// Paul Tol qualitative muted — 10 colors.
    pub fn tol_muted() -> Self {
        Self {
            name: "tol_muted",
            colors: vec![
                "#CC6677", "#332288", "#DDCC77", "#117733",
                "#88CCEE", "#882255", "#44AA99", "#999933",
                "#AA4499", "#DDDDDD",
            ].into_iter().map(Into::into).collect(),
        }
    }

    /// Paul Tol qualitative light — 9 colors.
    pub fn tol_light() -> Self {
        Self {
            name: "tol_light",
            colors: vec![
                "#77AADD", "#EE8866", "#EEDD88", "#FFAABB",
                "#99DDFF", "#44BB99", "#BBCC33", "#AAAA00",
                "#DDDDDD",
            ].into_iter().map(Into::into).collect(),
        }
    }

    /// IBM Design Language — 5 colors.
    pub fn ibm() -> Self {
        Self {
            name: "ibm",
            colors: vec![
                "#648FFF", "#785EF0", "#DC267F", "#FE6100", "#FFB000",
            ].into_iter().map(Into::into).collect(),
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
                "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728",
                "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
                "#bcbd22", "#17becf",
            ].into_iter().map(Into::into).collect(),
        }
    }

    /// Softer pastel version — 10 colors.
    pub fn pastel() -> Self {
        Self {
            name: "pastel",
            colors: vec![
                "#aec7e8", "#ffbb78", "#98df8a", "#ff9896",
                "#c5b0d5", "#c49c94", "#f7b6d2", "#c7c7c7",
                "#dbdb8d", "#9edae5",
            ].into_iter().map(Into::into).collect(),
        }
    }

    /// High-saturation vivid — 10 colors.
    pub fn bold() -> Self {
        Self {
            name: "bold",
            colors: vec![
                "#e41a1c", "#377eb8", "#4daf4a", "#984ea3",
                "#ff7f00", "#a65628", "#f781bf", "#999999",
                "#66c2a5", "#fc8d62",
            ].into_iter().map(Into::into).collect(),
        }
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
