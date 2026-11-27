//! Long-format pivots build a dense matrix of distinct rows by distinct
//! columns, so each input line can add a row and a column and the matrix grows
//! with the square of the input. `heatmap --long-format`, `rose --group-by`
//! and `surface3d` now refuse a pivot over the 1,000,000-cell limit before
//! allocating it. kuva runs here under a 1 GB address-space limit, so a
//! regression fails on the allocation instead of filling memory.
#![cfg(unix)]

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Runs kuva on `input` under a 1 GB memory limit; returns (success, stdout,
/// stderr). Fails if it takes longer than 60 s.
fn run(args: &[&str], input: String) -> (bool, String, String) {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("ulimit -v 1000000; exec \"$0\" \"$@\"")
        .arg(env!("CARGO_BIN_EXE_kuva"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kuva");
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || {
        let mut b = Vec::new();
        stdout.read_to_end(&mut b).map(|_| b)
    });
    let err = std::thread::spawn(move || {
        let mut b = Vec::new();
        stderr.read_to_end(&mut b).map(|_| b)
    });
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if start.elapsed() > Duration::from_secs(60) {
            child.kill().ok();
            panic!("kuva {args:?} took longer than 60 s");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    // kuva may exit before reading all of stdin.
    let _ = writer.join().unwrap();
    let out = String::from_utf8_lossy(&out.join().unwrap().unwrap()).into_owned();
    let err = String::from_utf8_lossy(&err.join().unwrap().unwrap()).into_owned();
    (status.success(), out, err)
}

/// 20,000 lines, each with its own row and column key: a 4 x 10^8 cell pivot.
fn distinct_triples() -> String {
    let mut t = String::from("a\tb\tv\n");
    for i in 0..20_000 {
        t.push_str(&format!("{i}\t{}\t1\n", i + 100_000));
    }
    t
}

fn assert_refused(args: &[&str], input: String) {
    let (ok, out, err) = run(args, input);
    assert!(!ok, "{args:?} succeeded");
    assert!(out.is_empty());
    assert!(err.contains("over the 1000000 cell limit"), "{err}");
    assert!(err.contains("20000 x 20000"), "{err}");
}

#[test]
fn heatmap_long_format_pivot_is_limited() {
    assert_refused(&["heatmap", "--long-format"], distinct_triples());
}

#[test]
fn rose_group_by_pivot_is_limited() {
    assert_refused(
        &["rose", "--label", "a", "--group-by", "b", "--value", "v"],
        distinct_triples(),
    );
}

#[test]
fn surface3d_pivot_is_limited() {
    assert_refused(
        &["surface3d", "--x", "a", "--y", "b", "--z", "v"],
        distinct_triples(),
    );
}

#[test]
fn ordinary_pivots_still_render() {
    let mut t = String::from("a\tb\tv\n");
    for r in 0..20 {
        for c in 0..30 {
            t.push_str(&format!("{r}\t{c}\t{}\n", r * c));
        }
    }
    for args in [
        &["heatmap", "--long-format"][..],
        &["rose", "--label", "a", "--group-by", "b", "--value", "v"],
        &["surface3d", "--x", "a", "--y", "b", "--z", "v"],
    ] {
        let (ok, out, err) = run(args, t.clone());
        assert!(ok, "{args:?}: {err}");
        assert!(out.contains("<svg"), "{args:?}");
    }
}
