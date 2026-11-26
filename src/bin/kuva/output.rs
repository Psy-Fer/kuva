use crate::layout_args::BaseArgs;
use clap::builder::{PathBufValueParser, TypedValueParser};
use kuva::backend::svg::SvgBackend;
use kuva::render::render::Scene;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Svg,
    Svgz,
    Html,
    Png,
    Pdf,
}

impl OutputFormat {
    fn from_path(path: &Path) -> Result<Self, String> {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return Err(unsupported_output_path(path));
        };

        match extension.to_ascii_lowercase().as_str() {
            "svg" => Ok(Self::Svg),
            "svgz" => Ok(Self::Svgz),
            "html" | "htm" => Ok(Self::Html),
            "png" => Ok(Self::Png),
            "pdf" => Ok(Self::Pdf),
            _ => Err(unsupported_output_path(path)),
        }
    }
}

fn unsupported_output_path(path: &Path) -> String {
    format!(
        "unsupported output path '{}': expected an extension of .svg, .svgz, .html, .png, or .pdf",
        path.display()
    )
}

fn validate_output_path(path: PathBuf) -> Result<PathBuf, std::io::Error> {
    OutputFormat::from_path(&path)
        .map(|_| path)
        .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))
}

pub(crate) fn output_path_parser() -> impl TypedValueParser<Value = PathBuf> {
    PathBufValueParser::new().try_map(validate_output_path)
}

/// Write the scene to a file (format inferred from extension) or SVG to stdout.
pub fn write_output(mut scene: Scene, args: &BaseArgs) -> Result<(), String> {
    // Anything rendering changed or left out, on stderr so it never mixes with
    // an SVG written to stdout.
    for w in &scene.warnings {
        eprintln!("warning: {w}");
    }
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
        print!(
            "{}",
            kuva::TerminalBackend::new(cols, rows).render_scene(&scene)
        );
        return Ok(());
    }

    let svg_backend = SvgBackend::new()
        .with_embedded_font(args.embed_font)
        .with_rect_batching(!args.no_rect_batching);

    match &args.output {
        None => {
            print!("{}", svg_backend.render_scene(&scene));
            Ok(())
        }
        Some(path) => match OutputFormat::from_path(path)? {
            OutputFormat::Svg => {
                fs::write(path, svg_backend.render_scene(&scene)).map_err(|e| e.to_string())
            }
            OutputFormat::Svgz => {
                #[cfg(feature = "svgz")]
                {
                    fs::write(path, svg_backend.render_scene_svgz(&scene))
                        .map_err(|e| e.to_string())
                }
                #[cfg(not(feature = "svgz"))]
                Err(svgz_feature_missing(".svgz"))
            }
            OutputFormat::Html => {
                #[cfg(feature = "svgz")]
                {
                    // One self-contained page holding the gzipped figure; it opens from disk in
                    // any current browser. Named after the file, titled by --title if given.
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("figure");
                    let title = args.title.as_deref().unwrap_or(name);
                    let page = kuva::html::standalone_page(
                        title,
                        &svg_backend.render_scene_svgz(&scene),
                        name,
                    );
                    fs::write(path, page).map_err(|e| e.to_string())
                }
                #[cfg(not(feature = "svgz"))]
                Err(svgz_feature_missing(".html"))
            }
            OutputFormat::Png => {
                #[cfg(feature = "png")]
                {
                    let bytes = kuva::PngBackend::new().render_scene(&scene)?;
                    fs::write(path, bytes).map_err(|e| e.to_string())
                }
                #[cfg(not(feature = "png"))]
                Err("PNG output requires the 'png' feature. \
                         Rebuild with: cargo build --bin kuva --features cli,png"
                    .to_string())
            }
            OutputFormat::Pdf => {
                #[cfg(feature = "pdf")]
                {
                    let bytes = kuva::PdfBackend::new().render_scene(&scene)?;
                    fs::write(path, bytes).map_err(|e| e.to_string())
                }
                #[cfg(not(feature = "pdf"))]
                Err("PDF output requires the 'pdf' feature. \
                         Rebuild with: cargo build --bin kuva --features cli,pdf"
                    .to_string())
            }
        },
    }
}

#[cfg(not(feature = "svgz"))]
fn svgz_feature_missing(ext: &str) -> String {
    format!(
        "{ext} output requires the 'svgz' feature. \
         Rebuild with: cargo build --bin kuva --features cli,svgz"
    )
}

#[cfg(test)]
mod tests {
    use super::OutputFormat;
    use std::path::Path;

    #[test]
    fn output_format_recognizes_supported_extensions_case_insensitively() {
        let cases = [
            ("plot.svg", OutputFormat::Svg),
            ("plot.SvG", OutputFormat::Svg),
            ("plot.svgz", OutputFormat::Svgz),
            ("plot.SVGZ", OutputFormat::Svgz),
            ("plot.html", OutputFormat::Html),
            ("plot.htm", OutputFormat::Html),
            ("plot.png", OutputFormat::Png),
            ("plot.PnG", OutputFormat::Png),
            ("plot.pdf", OutputFormat::Pdf),
            ("plot.PdF", OutputFormat::Pdf),
        ];

        for (path, expected) in cases {
            assert_eq!(OutputFormat::from_path(Path::new(path)), Ok(expected));
        }
    }

    #[test]
    fn output_format_rejects_missing_and_unsupported_extensions() {
        for path in ["plot", "plot.", "plot.txt", ".svg"] {
            let error = OutputFormat::from_path(Path::new(path)).unwrap_err();
            assert!(error.contains(".svg"), "unexpected error: {error}");
            assert!(error.contains(".svgz"), "unexpected error: {error}");
            assert!(error.contains(".html"), "unexpected error: {error}");
            assert!(error.contains(".png"), "unexpected error: {error}");
            assert!(error.contains(".pdf"), "unexpected error: {error}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn output_format_rejects_non_unicode_extension() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        use std::path::PathBuf;

        let mut bytes = b"plot.".to_vec();
        bytes.push(0xff);
        let path = PathBuf::from(OsString::from_vec(bytes));

        assert!(OutputFormat::from_path(&path).is_err());
    }
}
