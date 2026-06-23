/// Pattern fill for filled areas, bars, and other solid regions.
///
/// The default is [`FillPattern::Solid`] (no pattern, plain color fill).
///
/// Patterns are fully independent of the fill color — the hatch lines are
/// always black and render as an overlay on top of whatever color fill is
/// applied. This means every combination of color and pattern is valid, and
/// patterns remain useful even after converting a figure to greyscale.
///
/// When used with [`crate::render::Layout::with_bw_mode()`], patterns and grey
/// shades are assigned automatically to maximise distinguishability without
/// relying on color.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FillPattern {
    /// No pattern — plain solid fill (default).
    #[default]
    Solid,
    /// Horizontal parallel lines (═══).
    Horizontal,
    /// Vertical parallel lines (|||).
    Vertical,
    /// Forward-diagonal lines (///).
    DiagonalForward,
    /// Back-diagonal lines (\\\).
    DiagonalBack,
    /// Horizontal + vertical grid (+++).
    Crosshatch,
    /// Forward + back diagonal grid (×××).
    DiagonalCrosshatch,
    /// Scattered filled dots (···).
    Dots,
}

impl FillPattern {
    /// The SVG `id` used to reference this pattern via `fill="url(#id)"`.
    ///
    /// Returns an empty string for [`FillPattern::Solid`].
    pub fn id(self) -> &'static str {
        match self {
            FillPattern::Solid              => "",
            FillPattern::Horizontal         => "kuva-fp-horiz",
            FillPattern::Vertical           => "kuva-fp-vert",
            FillPattern::DiagonalForward    => "kuva-fp-diag-fwd",
            FillPattern::DiagonalBack       => "kuva-fp-diag-back",
            FillPattern::Crosshatch         => "kuva-fp-crosshatch",
            FillPattern::DiagonalCrosshatch => "kuva-fp-diag-cross",
            FillPattern::Dots               => "kuva-fp-dots",
        }
    }

    /// The complete SVG `<pattern>…</pattern>` element to place in `<defs>`.
    ///
    /// The pattern has a transparent background so the base fill color shows
    /// through. Hatch lines are black and sized for readability at typical plot
    /// resolutions and at 300 DPI print output.
    ///
    /// Returns an empty string for [`FillPattern::Solid`].
    pub fn svg_def(self) -> &'static str {
        match self {
            FillPattern::Solid => "",

            FillPattern::Horizontal => concat!(
                r#"<pattern id="kuva-fp-horiz" patternUnits="userSpaceOnUse" width="8" height="6">"#,
                r#"<line x1="0" y1="3" x2="8" y2="3" stroke="black" stroke-width="1.2"/>"#,
                r#"</pattern>"#,
            ),

            FillPattern::Vertical => concat!(
                r#"<pattern id="kuva-fp-vert" patternUnits="userSpaceOnUse" width="6" height="8">"#,
                r#"<line x1="3" y1="0" x2="3" y2="8" stroke="black" stroke-width="1.2"/>"#,
                r#"</pattern>"#,
            ),

            // Three path segments tile a continuous diagonal line across any
            // shape without gaps: top-left corner, main stripe, bottom-right corner.
            FillPattern::DiagonalForward => concat!(
                r#"<pattern id="kuva-fp-diag-fwd" patternUnits="userSpaceOnUse" width="6" height="6">"#,
                r#"<path d="M-1,1 l2,-2 M0,6 l6,-6 M5,7 l2,-2" stroke="black" stroke-width="1.2" fill="none"/>"#,
                r#"</pattern>"#,
            ),

            FillPattern::DiagonalBack => concat!(
                r#"<pattern id="kuva-fp-diag-back" patternUnits="userSpaceOnUse" width="6" height="6">"#,
                r#"<path d="M-1,5 l2,2 M0,0 l6,6 M5,-1 l2,2" stroke="black" stroke-width="1.2" fill="none"/>"#,
                r#"</pattern>"#,
            ),

            FillPattern::Crosshatch => concat!(
                r#"<pattern id="kuva-fp-crosshatch" patternUnits="userSpaceOnUse" width="8" height="8">"#,
                r#"<path d="M0,4 H8 M4,0 V8" stroke="black" stroke-width="1.2" fill="none"/>"#,
                r#"</pattern>"#,
            ),

            FillPattern::DiagonalCrosshatch => concat!(
                r#"<pattern id="kuva-fp-diag-cross" patternUnits="userSpaceOnUse" width="6" height="6">"#,
                r#"<path d="M-1,1 l2,-2 M0,6 l6,-6 M5,7 l2,-2 M-1,5 l2,2 M0,0 l6,6 M5,-1 l2,2" stroke="black" stroke-width="1.2" fill="none"/>"#,
                r#"</pattern>"#,
            ),

            FillPattern::Dots => concat!(
                r#"<pattern id="kuva-fp-dots" patternUnits="userSpaceOnUse" width="8" height="8">"#,
                r#"<circle cx="4" cy="4" r="1.8" fill="black"/>"#,
                r#"</pattern>"#,
            ),
        }
    }

    /// Returns `true` for any variant other than [`FillPattern::Solid`].
    pub fn is_patterned(self) -> bool {
        self != FillPattern::Solid
    }
}
