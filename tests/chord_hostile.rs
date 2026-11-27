//! Chord matrices that aren't square. Laying out n rows took an n x n table
//! and n^2 loops, whatever the rows held, so 50,000 one-value rows asked for
//! 20 GB. A non-square matrix is now not drawn and is reported as a warning,
//! and the CLI refuses one.

mod hostile;

use kuva::plot::ChordPlot;
use kuva::render::render::{render_multiple, Scene};
use kuva::render::{layout::Layout, plots::Plot};

fn render(chord: ChordPlot) -> Scene {
    let plots = vec![Plot::Chord(chord)];
    let layout = Layout::auto_from_plots(&plots);
    render_multiple(plots, layout)
}

fn not_square(scene: &Scene) -> Vec<&str> {
    scene
        .warnings
        .iter()
        .filter(|w| w.code == "chord.not_square")
        .map(|w| w.message.as_str())
        .collect()
}

#[test]
fn many_short_rows_are_not_laid_out() {
    let scene = hostile::bounded(64 << 20, || {
        render(ChordPlot::new().with_matrix(vec![vec![1.0]; 50_000]))
    });
    let msgs = not_square(&scene);
    assert_eq!(msgs.len(), 1);
    assert!(
        msgs[0].contains("50000 rows but row 0 has 1 values"),
        "{}",
        msgs[0]
    );
}

#[test]
fn a_ragged_matrix_is_reported_and_a_square_one_is_drawn() {
    let ragged = ChordPlot::new().with_matrix(vec![
        vec![0.0, 1.0, 2.0],
        vec![1.0, 0.0],
        vec![2.0, 1.0, 0.0],
    ]);
    assert!(ragged
        .check_square()
        .unwrap_err()
        .contains("row 1 has 2 values"));
    let scene = render(ragged);
    assert_eq!(not_square(&scene).len(), 1);

    let square = ChordPlot::new().with_matrix(vec![
        vec![0.0, 1.0, 2.0],
        vec![1.0, 0.0, 3.0],
        vec![2.0, 1.0, 0.0],
    ]);
    assert!(square.check_square().is_ok());
    let drawn = render(square);
    assert!(drawn.warnings.is_empty());
    assert!(drawn.elements.len() > scene.elements.len());
}

#[cfg(feature = "cli")]
fn cli(input: String) -> (bool, String) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_kuva"))
        .arg("chord")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let out = child.wait_with_output().unwrap();
    let _ = writer.join();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[cfg(feature = "cli")]
#[test]
fn cli_refuses_a_non_square_table() {
    // Two data rows, three value columns.
    let (ok, err) = cli("node\ta\tb\tc\na\t0\t1\t2\nb\t1\t0\t3\n".into());
    assert!(!ok);
    assert!(err.contains("2 rows but row 0 has 3 values"), "{err}");

    // Many rows with one value each.
    let mut t = String::from("node\tv\n");
    for i in 0..50_000 {
        t.push_str(&format!("n{i}\t1\n"));
    }
    let (ok, err) = cli(t);
    assert!(!ok);
    assert!(err.contains("50000 rows"), "{err}");

    let (ok, err) = cli("node\ta\tb\na\t0\t1\nb\t1\t0\n".into());
    assert!(ok, "{err}");
}
