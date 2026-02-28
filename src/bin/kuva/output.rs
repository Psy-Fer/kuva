use std::fs;
use kuva::render::render::Scene;
use kuva::backend::svg::SvgBackend;
use crate::layout_args::BaseArgs;

/// Write the scene to a file (format inferred from extension) or SVG to stdout.
pub fn write_output(mut scene: Scene, args: &BaseArgs) -> Result<(), String> {
    // Only override the theme background when the user explicitly passed --background.
    if let Some(ref bg) = args.background {
        scene.background_color = Some(bg.clone());
    }

    if args.terminal {
        let cols = args
            .term_width
            .map(|w| w as usize)
            .or_else(|| std::env::var("COLUMNS").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(80);
        let rows = args
            .term_height
            .map(|h| h as usize)
            .or_else(|| std::env::var("LINES").ok().and_then(|s| s.parse().ok()))
            .unwrap_or(24);
        print!("{}", kuva::TerminalBackend::new(cols, rows).render_scene(&scene));
        return Ok(());
    }

    match &args.output {
        None => {
            print!("{}", SvgBackend.render_scene(&scene));
            Ok(())
        }
        Some(path) => {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("svg");
            match ext {
                "png" => {
                    #[cfg(feature = "png")]
                    {
                        let bytes = kuva::PngBackend::new().render_scene(&scene)?;
                        fs::write(path, bytes).map_err(|e| e.to_string())
                    }
                    #[cfg(not(feature = "png"))]
                    Err("PNG output requires the 'png' feature. \
                         Rebuild with: cargo build --bin kuva --features png"
                        .to_string())
                }
                "pdf" => {
                    #[cfg(feature = "pdf")]
                    {
                        let bytes = kuva::PdfBackend.render_scene(&scene)?;
                        fs::write(path, bytes).map_err(|e| e.to_string())
                    }
                    #[cfg(not(feature = "pdf"))]
                    Err("PDF output requires the 'pdf' feature. \
                         Rebuild with: cargo build --bin kuva --features pdf"
                        .to_string())
                }
                _ => {
                    fs::write(path, SvgBackend.render_scene(&scene))
                        .map_err(|e| e.to_string())
                }
            }
        }
    }
}
