//! CLI table handling that used to be quadratic in the row count must finish
//! quickly on large inputs. `rose --group-by` rescanned every row for each
//! (group, label) pair, and `radar` parsed every axis column once per row.
//! In a release build each case took 17 to 38 s before and now takes under 1.5 s.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Runs kuva on `input` and fails if it hasn't finished within `limit`.
fn run_within(args: &[&str], input: String, limit: Duration) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kuva");
    let mut stdin = child.stdin.take().expect("stdin");
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    // Read the output while waiting, or a large SVG fills the pipe and kuva
    // blocks on the write.
    let mut stdout = child.stdout.take().expect("stdout");
    let mut stderr = child.stderr.take().expect("stderr");
    let out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).map(|_| buf)
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        stderr.read_to_end(&mut buf).map(|_| buf)
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("wait") {
            break status;
        }
        if start.elapsed() > limit {
            child.kill().ok();
            panic!("kuva {args:?} took longer than {limit:?}");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    writer.join().unwrap().expect("write stdin");
    let out = out_reader.join().unwrap().expect("read stdout");
    let err = err_reader.join().unwrap().expect("read stderr");
    assert!(status.success(), "{}", String::from_utf8_lossy(&err));
    String::from_utf8_lossy(&out).into_owned()
}

// Generous, so a loaded machine doesn't fail the test. A debug build of
// the old code needed minutes for each case.
const LIMIT: Duration = Duration::from_secs(60);

#[test]
fn rose_group_by_with_many_distinct_labels_is_fast() {
    let mut tsv = String::from("dir\tgrp\tv\n");
    for i in 0..40_000 {
        tsv.push_str(&format!("L{i}\t{}\t1\n", ["a", "b"][i % 2]));
    }
    let svg = run_within(
        &[
            "rose",
            "--label",
            "dir",
            "--value",
            "v",
            "--group-by",
            "grp",
        ],
        tsv,
        LIMIT,
    );
    assert!(svg.contains("L39999"));
}

#[test]
fn radar_one_series_per_row_is_fast() {
    let mut tsv = String::from("name\tx\ty\tz\n");
    for i in 0..10_000 {
        tsv.push_str(&format!("R{i}\t{}\t{}\t{}\n", i % 7, i % 5, i % 3));
    }
    let svg = run_within(
        &[
            "radar",
            "--axes",
            "x",
            "y",
            "z",
            "--label-col",
            "name",
            "--legend",
        ],
        tsv,
        LIMIT,
    );
    assert!(svg.contains("R9999"));
}
