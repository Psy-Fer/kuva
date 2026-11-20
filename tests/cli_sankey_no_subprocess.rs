//! Security regression: Sankey `neighbornet` ordering must never start another program.
//!
//! It used to run `python3 -c` with a script importing `splitspy`, so a `splitspy` module
//! planted in the working directory (or a `python3` earlier on PATH) ran as the user.
#![cfg(all(feature = "cli", unix))]
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn run_sankey(cwd: &Path, path_env: &str, order: &str) -> String {
    let data = format!(
        "{}/examples/data/sankey_alluvium.tsv",
        env!("CARGO_MANIFEST_DIR")
    );
    let out = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .current_dir(cwd)
        .env("PATH", path_env)
        .args([
            "sankey",
            data.as_str(),
            "--axis-col",
            "tissue",
            "--axis-col",
            "cluster",
            "--axis-col",
            "sex",
            "--value-col",
            "count",
            "--node-order",
            order,
        ])
        .output()
        .expect("spawn kuva");
    assert!(
        out.status.success(),
        "kuva sankey --node-order {order} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8 svg")
}

#[test]
fn neighbornet_ordering_runs_no_external_program() {
    let dir = std::env::temp_dir().join(format!("kuva_sankey_nosub_{}", std::process::id()));
    let bin = dir.join("bin");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&bin).unwrap();
    let marker = dir.join("PWNED");

    // A fake interpreter first on PATH, and planted modules in the working directory.
    let fake = bin.join("python3");
    fs::write(
        &fake,
        format!("#!/bin/sh\ntouch '{}'\nexit 1\n", marker.display()),
    )
    .unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let plant = format!("open({:?}, 'w').close()\n", marker.display().to_string());
    fs::write(dir.join("numpy.py"), &plant).unwrap();
    fs::create_dir_all(dir.join("splitspy")).unwrap();
    fs::write(dir.join("splitspy").join("__init__.py"), &plant).unwrap();

    let path_env = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let nn = run_sankey(&dir, &path_env, "neighbornet");
    let nn_short = run_sankey(&dir, &path_env, "nn");
    let crossings = run_sankey(&dir, &path_env, "crossings");

    let pwned = marker.exists();
    let _ = fs::remove_dir_all(&dir);
    assert!(
        !pwned,
        "rendering started python3 or imported a planted module"
    );
    // neighbornet is an alias for crossing reduction.
    assert_eq!(nn, crossings);
    assert_eq!(nn_short, crossings);
}
