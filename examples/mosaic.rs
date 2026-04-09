//! Mosaic / Marimekko chart documentation examples.
//!
//! Generates canonical SVG outputs used in the kuva documentation.
//! Run with:
//!
//! ```bash
//! cargo run --example mosaic
//! ```
//!
//! SVGs are written to `docs/src/assets/mosaic/`.

use kuva::plot::mosaic::MosaicPlot;
use kuva::backend::svg::SvgBackend;
use kuva::render::render::render_multiple;
use kuva::render::layout::Layout;
use kuva::render::plots::Plot;
use std::fs;

const OUT: &str = "docs/src/assets/mosaic";

fn save(name: &str, plot: MosaicPlot, title: &str) {
    let plots = vec![Plot::Mosaic(plot)];
    let layout = Layout::auto_from_plots(&plots).with_title(title);
    let svg = SvgBackend.render_scene(&render_multiple(plots, layout));
    fs::create_dir_all(OUT).unwrap();
    fs::write(format!("{OUT}/{name}.svg"), svg).unwrap();
    println!("wrote {OUT}/{name}.svg");
}

fn main() {
    // ── 1. Treatment × Response (epidemiology) ────────────────────────────────
    let treatment = MosaicPlot::new()
        .with_cells([
            ("Control",  "Positive", 23.0),
            ("Control",  "Neutral",  45.0),
            ("Control",  "Negative", 32.0),
            ("Low Dose", "Positive", 41.0),
            ("Low Dose", "Neutral",  38.0),
            ("Low Dose", "Negative", 21.0),
            ("High Dose","Positive", 67.0),
            ("High Dose","Neutral",  22.0),
            ("High Dose","Negative", 11.0),
        ])
        .with_col_order(["Control", "Low Dose", "High Dose"])
        .with_row_order(["Positive", "Neutral", "Negative"])
        .with_legend("Response");
    save("treatment_response", treatment, "Treatment × Response");

    // ── 2. Titanic — Survival by Passenger Class ──────────────────────────────
    let titanic = MosaicPlot::new()
        .with_cells([
            ("1st Class", "Survived", 200.0),
            ("1st Class", "Died",     123.0),
            ("2nd Class", "Survived", 119.0),
            ("2nd Class", "Died",     158.0),
            ("3rd Class", "Survived", 181.0),
            ("3rd Class", "Died",     528.0),
        ])
        .with_row_order(["Survived", "Died"])
        .with_group_colors(["#4dac26", "#d01c8b"])
        .with_legend("Outcome");
    save("titanic", titanic, "Titanic — Survival by Passenger Class");

    // ── 3. Market Share (4 regions × 3 products) ─────────────────────────────
    let market = MosaicPlot::new()
        .with_cells([
            ("North", "Product A", 320.0),
            ("North", "Product B", 210.0),
            ("North", "Product C", 85.0),
            ("South", "Product A", 180.0),
            ("South", "Product B", 290.0),
            ("South", "Product C", 130.0),
            ("East",  "Product A", 240.0),
            ("East",  "Product B", 175.0),
            ("East",  "Product C", 195.0),
            ("West",  "Product A", 150.0),
            ("West",  "Product B", 320.0),
            ("West",  "Product C", 210.0),
        ])
        .with_legend("Product");
    save("market_share", market, "Regional Market Share by Product");

    // ── 4. Education × Employment — non-normalized ────────────────────────────
    let education = MosaicPlot::new()
        .with_cells([
            ("No degree",    "Employed",    420.0),
            ("No degree",    "Unemployed",  180.0),
            ("No degree",    "Inactive",    280.0),
            ("Bachelor's",   "Employed",    850.0),
            ("Bachelor's",   "Unemployed",  95.0),
            ("Bachelor's",   "Inactive",    110.0),
            ("Postgraduate", "Employed",    390.0),
            ("Postgraduate", "Unemployed",  30.0),
            ("Postgraduate", "Inactive",    45.0),
        ])
        .with_row_order(["Employed", "Unemployed", "Inactive"])
        .with_normalize(false)
        .with_legend("Status");
    save("education_employment", education, "Education × Employment (non-normalized)");

    // ── 5. Minimal — 2 columns, percents only ─────────────────────────────────
    let minimal = MosaicPlot::new()
        .with_cells([
            ("Vaccinated",   "Protected",    88.0),
            ("Vaccinated",   "Breakthrough", 12.0),
            ("Unvaccinated", "Protected",    30.0),
            ("Unvaccinated", "Breakthrough", 70.0),
        ])
        .with_row_order(["Protected", "Breakthrough"])
        .with_group_colors(["#1a9641", "#d7191c"])
        .with_legend("Outcome");
    save("vaccine_efficacy", minimal, "Vaccine Efficacy");
}
