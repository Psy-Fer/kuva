use std::fs;
use visus::render::render::Scene;
use visus::backend::svg::SvgBackend;
use crate::layout_args::BaseArgs;

/// Write the scene to a file (format inferred from extension) or SVG to stdout.
pub fn write_output(mut scene: Scene, args: &BaseArgs) -> Result<(), String> {
    // Only override the theme background when the user explicitly passed --background.
    if let Some(ref bg) = args.background {
        scene.background_color = Some(bg.clone());
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
                        let bytes = visus::PngBackend.render_scene(&scene)?;
                        fs::write(path, bytes).map_err(|e| e.to_string())
                    }
                    #[cfg(not(feature = "png"))]
                    Err("PNG output requires the 'png' feature. \
                         Rebuild with: cargo build --bin visus --features png"
                        .to_string())
                }
                "pdf" => {
                    #[cfg(feature = "pdf")]
                    {
                        let bytes = visus::PdfBackend.render_scene(&scene)?;
                        fs::write(path, bytes).map_err(|e| e.to_string())
                    }
                    #[cfg(not(feature = "pdf"))]
                    Err("PDF output requires the 'pdf' feature. \
                         Rebuild with: cargo build --bin visus --features pdf"
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
