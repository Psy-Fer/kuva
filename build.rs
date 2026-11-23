use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_FEATURE_DOOM").is_err() {
        return;
    }

    // docs.rs: source is read-only and network is disabled -- write empty
    // placeholder files to OUT_DIR so include_bytes! compiles, then bail.
    if std::env::var("DOCS_RS").is_ok() {
        let out = std::env::var("OUT_DIR").unwrap();
        let out = Path::new(&out);
        std::fs::write(out.join("doom.wasm"), b"").unwrap();
        std::fs::write(out.join("doom.js"), "").unwrap();
        std::fs::write(out.join("doom1.wad"), b"").unwrap();
        return;
    }

    // Asset cache lives next to Cargo.toml so it survives `cargo clean`.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let cache = PathBuf::from(&manifest_dir).join("doom-assets");
    std::fs::create_dir_all(&cache).expect("failed to create doom-assets/");

    let wasm = cache.join("doom.wasm");
    let js = cache.join("doom.js");
    let wad = cache.join("doom1.wad");

    // Tell Cargo to re-run this script if the cached files disappear.
    println!("cargo:rerun-if-changed=doom-assets/doom.wasm");
    println!("cargo:rerun-if-changed=doom-assets/doom.js");
    println!("cargo:rerun-if-changed=doom-assets/doom1.wad");

    // Download missing assets from the kuva doom-assets release.
    // These are uploaded once by .github/workflows/build-doom-assets.yml.
    const BASE: &str = "https://github.com/Psy-Fer/kuva/releases/download/doom-assets-v1";

    if !wasm.exists() {
        download(&format!("{BASE}/doom.wasm"), &wasm);
    }
    if !js.exists() {
        download(&format!("{BASE}/doom.js"), &js);
    }
    if !wad.exists() {
        download(&format!("{BASE}/doom1.wad"), &wad);
    }

    // The files end up as script inside `kuva doom`'s SVG, so check them against the digests
    // of the published release, whether they were just downloaded or were already in
    // doom-assets/ (put there by hand or restored from a cache).
    verify(&wasm, DOOM_WASM_SHA256);
    verify(&js, DOOM_JS_SHA256);
    verify(&wad, DOOM1_WAD_SHA256);

    // Copy into OUT_DIR so doom.rs can include_bytes! them.
    let out = std::env::var("OUT_DIR").unwrap();
    let out = Path::new(&out);
    copy_asset(&wasm, out.join("doom.wasm"));
    copy_asset(&js, out.join("doom.js"));
    copy_asset(&wad, out.join("doom1.wad"));
}

// SHA-256 of the doom-assets-v1 release files. A rebuild with
// .github/workflows/build-doom-assets.yml prints new digests to copy in here.
const DOOM_WASM_SHA256: &str = "4ac406bf31468d46a293382f2432fe8da593bb2c227ae1b0040555ec56215616";
const DOOM_JS_SHA256: &str = "264188ae0388d1d6d9d7585600b0ae94e66af47e7dd0d7e5bce5e2be05b58dfc";
// Shareware DOOM1.WAD v1.9 (SHA-1 5b2e249b9c5133ec987b3ea77596381dc0d6bc1d).
const DOOM1_WAD_SHA256: &str = "1d7d43be501e67d927e415e0b8f3e29c3bf33075e859721816f652a526cac771";

fn verify(path: &Path, expected: &str) {
    let bytes =
        std::fs::read(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let got = sha256_hex(&bytes);
    if got != expected {
        panic!(
            "\n\nkuva doom: {} has SHA-256 {got}, expected {expected}.\n\
             Delete the doom-assets/ directory next to Cargo.toml and build again to\n\
             re-download it. If the doom-assets-v1 release was rebuilt on purpose,\n\
             update the digests in build.rs.\n",
            path.display()
        );
    }
}

/// SHA-256 (FIPS 180-4) as lowercase hex. Written out here so the doom feature needs no
/// build dependency.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for block in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
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
    std::fs::copy(src, &dest)
        .unwrap_or_else(|e| panic!("failed to copy {} → {}: {e}", src.display(), dest.display()));
}
