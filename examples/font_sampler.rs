/// Font sampler — generates font_sampler.svg showing each font at the sizes
/// actually used in kuva plots (tick labels, axis labels, title).
///
/// Run:
///   cargo run --example font_sampler
///   # then open font_sampler.svg in a browser
///
/// Fonts not installed on your system fall back to the generic (sans-serif /
/// serif / monospace). That is itself useful information.

fn main() {
    // (display name, CSS font-family value)
    let fonts: &[(&str, &str)] = &[
        // ── Generics ────────────────────────────────────────────────────────
        ("sans-serif",        "sans-serif"),
        ("serif",             "serif"),
        ("monospace",         "monospace"),
        // ── Common Linux sans-serif ─────────────────────────────────────────
        ("DejaVu Sans",       "DejaVu Sans, sans-serif"),
        ("Liberation Sans",   "Liberation Sans, sans-serif"),
        ("Noto Sans",         "Noto Sans, sans-serif"),
        ("Ubuntu",            "Ubuntu, sans-serif"),
        ("Cantarell",         "Cantarell, sans-serif"),
        ("FreeSans",          "FreeSans, sans-serif"),
        // ── Cross-platform sans-serif ───────────────────────────────────────
        ("Arial",             "Arial, sans-serif"),
        ("Helvetica Neue",    "Helvetica Neue, Helvetica, sans-serif"),
        ("Verdana",           "Verdana, sans-serif"),
        ("Tahoma",            "Tahoma, sans-serif"),
        ("Trebuchet MS",      "Trebuchet MS, sans-serif"),
        ("Calibri",           "Calibri, sans-serif"),
        ("Segoe UI",          "Segoe UI, sans-serif"),
        // ── Web / open-source sans-serif ────────────────────────────────────
        ("Open Sans",         "Open Sans, sans-serif"),
        ("Lato",              "Lato, sans-serif"),
        ("Inter",             "Inter, sans-serif"),
        ("Roboto",            "Roboto, sans-serif"),
        ("Fira Sans",         "Fira Sans, sans-serif"),
        ("Source Sans 3",     "Source Sans 3, Source Sans Pro, sans-serif"),
        // ── Serif ────────────────────────────────────────────────────────────
        ("Georgia",           "Georgia, serif"),
        ("Palatino Linotype", "Palatino Linotype, Palatino, Book Antiqua, serif"),
        ("Times New Roman",   "Times New Roman, Times, serif"),
        // ── Monospace ────────────────────────────────────────────────────────
        ("Courier New",       "Courier New, monospace"),
        ("DejaVu Sans Mono",  "DejaVu Sans Mono, monospace"),
        ("Fira Code",         "Fira Code, monospace"),
        ("Consolas",          "Consolas, monospace"),
        ("Menlo",             "Menlo, monospace"),
    ];

    // Text shown at each size — representative of kuva plot text
    let sample = "AaBbGg 0.0 0.5 1.0  p &lt; 0.05  μ σ α β";

    // Layout constants
    let label_col_w: f64 = 160.0;
    let size_col_w:  f64 = 330.0;
    let sizes: &[(u32, &str)] = &[
        (12, "12px — tick labels"),
        (14, "14px — axis labels"),
        (18, "18px — title"),
    ];
    let row_h:    f64 = 36.0;
    let header_h: f64 = 60.0;
    let pad:      f64 = 12.0;

    // Group separators: (first font index in group, group label)
    let groups: &[(usize, &str)] = &[
        (0,  "Generic"),
        (3,  "Common Linux"),
        (9,  "Cross-platform"),
        (16, "Web / Open-source"),
        (22, "Serif"),
        (25, "Monospace"),
    ];

    let total_rows = fonts.len() + groups.len();
    let svg_w = label_col_w + sizes.len() as f64 * size_col_w + pad * 2.0;
    let svg_h = header_h + total_rows as f64 * row_h + pad * 2.0;

    let mut out = String::with_capacity(256 * 1024);

    // ── SVG root ─────────────────────────────────────────────────────────────
    out += &format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_w}" height="{svg_h}">"##);
    out += "\n";
    out += r##"  <rect width="100%" height="100%" fill="white" />"##;
    out += "\n";

    // ── Title ─────────────────────────────────────────────────────────────────
    out += &format!(
        r##"  <text x="{pad}" y="28" font-size="20" font-family="sans-serif" font-weight="bold" fill="#111">Font sampler — kuva</text>"##,
    );
    out += "\n";
    out += &format!(
        r##"  <text x="{pad}" y="48" font-size="11" font-family="sans-serif" fill="#666">Fonts not installed fall back to the generic. Open in a browser for live rendering.</text>"##,
    );
    out += "\n";

    // ── Column headers ────────────────────────────────────────────────────────
    let hdr_y = header_h - 8.0;
    out += &format!(
        r##"  <text x="{x}" y="{hdr_y}" font-size="11" font-family="sans-serif" font-weight="bold" fill="#444">Font family</text>"##,
        x = pad + 4.0
    );
    out += "\n";
    for (i, (_sz, label)) in sizes.iter().enumerate() {
        let col_x = pad + label_col_w + i as f64 * size_col_w + 8.0;
        out += &format!(
            r##"  <text x="{col_x}" y="{hdr_y}" font-size="11" font-family="sans-serif" font-weight="bold" fill="#444">{label}</text>"##,
        );
        out += "\n";
    }
    // Separator under headers
    out += &format!(
        r##"  <line x1="{x}" y1="{y}" x2="{x2}" y2="{y}" stroke="#999" stroke-width="1" />"##,
        x = pad, x2 = svg_w - pad, y = header_h
    );
    out += "\n";

    // ── Font rows ─────────────────────────────────────────────────────────────
    let mut row: usize = 0;

    for (fi, (name, family)) in fonts.iter().enumerate() {
        // Group header row
        if let Some(&(_, group_name)) = groups.iter().find(|&&(start, _)| start == fi) {
            let bg_y  = header_h + pad + row as f64 * row_h;
            let text_y = bg_y + row_h * 0.65;
            let w = svg_w - pad * 2.0;
            out += &format!(
                r##"  <rect x="{pad}" y="{bg_y}" width="{w}" height="{row_h}" fill="#eef2f7" />"##,
            );
            out += "\n";
            out += &format!(
                r##"  <text x="{x}" y="{text_y}" font-size="11" font-family="sans-serif" font-weight="bold" fill="#555">{group_name}</text>"##,
                x = pad + 4.0
            );
            out += "\n";
            row += 1;
        }

        let row_y  = header_h + pad + row as f64 * row_h;
        let text_y = row_y + row_h * 0.65;
        let row_w  = svg_w - pad * 2.0;

        // Alternating row tint
        if fi % 2 == 0 {
            out += &format!(
                r##"  <rect x="{pad}" y="{row_y}" width="{row_w}" height="{row_h}" fill="#fafbfc" />"##,
            );
            out += "\n";
        }

        // Font name label (always sans-serif for readability)
        out += &format!(
            r##"  <text x="{x}" y="{text_y}" font-size="11" font-family="sans-serif" fill="#333">{name}</text>"##,
            x = pad + 4.0
        );
        out += "\n";

        // Vertical separator after label column
        let sep_x0 = pad + label_col_w;
        out += &format!(
            r##"  <line x1="{sep_x0}" y1="{row_y}" x2="{sep_x0}" y2="{bot}" stroke="#ddd" stroke-width="1" />"##,
            bot = row_y + row_h
        );
        out += "\n";

        // Sample text in the target font at each size
        for (i, (sz, _)) in sizes.iter().enumerate() {
            let col_x = pad + label_col_w + i as f64 * size_col_w + 8.0;
            out += &format!(
                r##"  <text x="{col_x}" y="{text_y}" font-size="{sz}" font-family="{family}" fill="#111">{sample}</text>"##,
            );
            out += "\n";

            // Vertical separator between size columns
            if i + 1 < sizes.len() {
                let vsep = pad + label_col_w + (i + 1) as f64 * size_col_w;
                out += &format!(
                    r##"  <line x1="{vsep}" y1="{row_y}" x2="{vsep}" y2="{bot}" stroke="#ddd" stroke-width="1" />"##,
                    bot = row_y + row_h
                );
                out += "\n";
            }
        }

        row += 1;
    }

    // Bottom border
    let bot_y = header_h + pad + row as f64 * row_h;
    out += &format!(
        r##"  <line x1="{x}" y1="{bot_y}" x2="{x2}" y2="{bot_y}" stroke="#999" stroke-width="1" />"##,
        x = pad, x2 = svg_w - pad
    );
    out += "\n";
    out += "</svg>\n";

    std::fs::write("font_sampler.svg", &out).expect("failed to write font_sampler.svg");
    println!("Written: font_sampler.svg  ({} fonts)", fonts.len());
    println!("Open in a browser to see which fonts are installed on your system.");
}
