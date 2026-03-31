use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_FEATURE_DOOM").is_err() {
        return;
    }

    // Asset cache lives next to Cargo.toml so it survives `cargo clean`.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let cache = PathBuf::from(&manifest_dir).join("doom-assets");
    std::fs::create_dir_all(&cache).expect("failed to create doom-assets/");

    let wasm = cache.join("doom.wasm");
    let js   = cache.join("doom.js");
    let wad  = cache.join("doom1.wad");

    // Tell Cargo to re-run this script if the cached files disappear.
    println!("cargo:rerun-if-changed=doom-assets/doom.wasm");
    println!("cargo:rerun-if-changed=doom-assets/doom.js");
    println!("cargo:rerun-if-changed=doom-assets/doom1.wad");

    // Download missing assets from the kuva doom-assets release.
    // These are uploaded once by .github/workflows/build-doom-assets.yml.
    const BASE: &str =
        "https://github.com/Psy-Fer/kuva/releases/download/doom-assets-v1";

    if !wasm.exists() {
        download(&format!("{BASE}/doom.wasm"), &wasm);
    }
    if !js.exists() {
        download(&format!("{BASE}/doom.js"), &js);
    }
    if !wad.exists() {
        download(&format!("{BASE}/doom1.wad"), &wad);
    }

    // Copy into OUT_DIR so doom.rs can include_bytes! them.
    let out = std::env::var("OUT_DIR").unwrap();
    let out = Path::new(&out);
    copy_asset(&wasm, out.join("doom.wasm"));
    copy_asset(&js,   out.join("doom.js"));
    copy_asset(&wad,  out.join("doom1.wad"));
}

fn download(url: &str, dest: &Path) {
    eprintln!("kuva doom: downloading {} …", url);
    let status = std::process::Command::new("curl")
        .args(["-L", "-f", "--retry", "3", "--progress-bar", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .unwrap_or_else(|_| {
            panic!(
                "\n\nkuva doom: `curl` not found.\n\
                 Install curl or manually place doom.wasm, doom.js, and doom1.wad\n\
                 into the doom-assets/ directory next to Cargo.toml.\n"
            )
        });
    if !status.success() {
        panic!(
            "\n\nkuva doom: failed to download {url}\n\
             The doom-assets release may not exist yet.\n\
             Run the 'Build doom assets' GitHub Actions workflow first:\n\
             https://github.com/Psy-Fer/kuva/actions/workflows/build-doom-assets.yml\n"
        );
    }
}

fn copy_asset(src: &Path, dest: PathBuf) {
    std::fs::copy(src, &dest).unwrap_or_else(|e| {
        panic!("failed to copy {} → {}: {e}", src.display(), dest.display())
    });
}
