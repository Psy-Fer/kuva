/// Write `content` to `path` unless the `CI` environment variable is set.
///
/// GitHub Actions (and most CI systems) export `CI=true`. Locally the variable
/// is absent, so the file is written as usual and can be inspected in a browser.
/// In CI the write is silently skipped — no test_outputs directory is created
/// and no artefacts accumulate on the runner.
///
/// The function mirrors the signature of `std::fs::write` and returns a
/// `Result` so existing `.unwrap()` / `?` call-sites continue to compile
/// without any other changes.
pub fn write_test_output(
    path: impl AsRef<std::path::Path>,
    content: impl AsRef<[u8]>,
) -> std::io::Result<()> {
    if std::env::var_os("CI").is_some() {
        return Ok(());
    }
    let _ = std::fs::create_dir("test_outputs");
    std::fs::write(path, content)
}
