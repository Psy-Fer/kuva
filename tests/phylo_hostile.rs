//! Tree input that isn't a tree, or is nested very deeply, must not crash or
//! hang the renderer.

use kuva::backend::svg::SvgBackend;
use kuva::plot::{PhyloNode, PhyloTree};
use kuva::render::{layout::Layout, plots::Plot, render::render_multiple};
use std::alloc::{GlobalAlloc, Layout as AllocLayout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

/// Allocator for this test binary that refuses to hold more than 2 GB at once.
/// If a regression makes a tree walk loop on a cycle, the walk's stack grows
/// until an allocation fails and the process aborts with "memory allocation
/// failed", instead of using all the machine's memory. The tests here peak at
/// about 400 MB.
struct Capped;

const ALLOC_CAP: usize = 2 << 30;
static IN_USE: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Capped {
    unsafe fn alloc(&self, layout: AllocLayout) -> *mut u8 {
        if IN_USE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size() > ALLOC_CAP {
            IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
            return std::ptr::null_mut();
        }
        let p = unsafe { System.alloc(layout) };
        if p.is_null() {
            IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: AllocLayout) {
        unsafe { System.dealloc(ptr, layout) };
        IN_USE.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: Capped = Capped;

/// Run `f` on a thread with a small stack and fail if it overflows, panics or
/// takes longer than 10 seconds.
///
/// On a timeout the whole test process exits. Panicking would only fail this
/// test and leave the stuck thread running (and allocating) while the other
/// tests finish.
fn bounded<T: Send + 'static>(stack: usize, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || {
            let _ = tx.send(f());
        })
        .unwrap();
    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(v) => v,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("did not finish (panic or stack overflow)")
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("did not finish within 10 seconds (hang); exiting the test process");
            std::process::exit(1);
        }
    }
}

fn render(tree: PhyloTree) -> String {
    let plots = vec![Plot::PhyloTree(tree)];
    let layout = Layout::auto_from_plots(&plots);
    SvgBackend.render_scene(&render_multiple(plots, layout))
}

#[test]
fn deeply_nested_newick_parses_on_a_small_stack() {
    // Unbalanced: a long run of '(' with no closing ')'.
    let n = bounded(256 * 1024, || {
        PhyloTree::from_newick(&"(".repeat(2_000_000)).nodes.len()
    });
    assert_eq!(n, 2_000_001);

    // Balanced, with a leaf at the bottom.
    let depth = 200_000;
    let tree = bounded(256 * 1024, move || {
        let s = format!("{}A{};", "(".repeat(depth), "):1".repeat(depth));
        PhyloTree::from_newick(&s)
    });
    assert_eq!(tree.nodes.len(), depth + 1);
    assert_eq!(tree.leaf_labels_top_to_bottom(), vec!["A".to_string()]);
}

#[test]
fn newick_structure_is_unchanged() {
    // Labels, support values, branch lengths, empty leaves, whitespace and
    // unbalanced input, written out as (label, support, length, children).
    let cases: &[(&str, &str)] = &[
        (
            "((A:1,B:2)95:1.5,(C,D)x:0.5,E:3);",
            "0[;;0;1,4,7] 1[;95;1.5;2,3] 2[A;;1;] 3[B;;2;] 4[x;;0.5;5,6] 5[C;;0;] 6[D;;0;] 7[E;;3;]",
        ),
        ("( A , ( B:1 , C ) ) root ;", "0[root;;0;1,2] 1[A;;0;] 2[;;0;3,4] 3[B;;1;] 4[C;;0;]"),
        ("(,());", "0[;;0;1,2] 1[;;0;] 2[;;0;3] 3[;;0;]"),
        ("((A,B", "0[;;0;1] 1[;;0;2,3] 2[A;;0;] 3[B;;0;]"),
        ("A:2;", "0[A;;2;]"),
        ("(A;B)", "0[;;0;1] 1[A;;0;]"),
    ];
    for (input, want) in cases {
        let tree = PhyloTree::from_newick(input);
        assert_eq!(tree.root, 0);
        let got: Vec<String> = tree
            .nodes
            .iter()
            .map(|n| {
                format!(
                    "{}[{};{};{};{}]",
                    n.id,
                    n.label.as_deref().unwrap_or(""),
                    n.support.map(|s| s.to_string()).unwrap_or_default(),
                    n.branch_length,
                    n.children
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect();
        assert_eq!(&got.join(" "), want, "input {input:?}");
        for n in &tree.nodes {
            for &c in &n.children {
                assert_eq!(tree.nodes[c].parent, Some(n.id), "input {input:?}");
            }
        }
    }
}

/// Every node is reached once from the root and parent links match.
fn assert_is_tree(tree: &PhyloTree) {
    let mut seen = vec![false; tree.nodes.len()];
    let mut stack = vec![tree.root];
    seen[tree.root] = true;
    while let Some(id) = stack.pop() {
        for &c in &tree.nodes[id].children {
            assert!(!seen[c], "node {c} reached twice");
            assert_eq!(tree.nodes[c].parent, Some(id));
            seen[c] = true;
            stack.push(c);
        }
    }
}

type Edge = (&'static str, &'static str, f64);

#[test]
fn edge_lists_that_are_not_trees_still_render() {
    let cases: Vec<(&str, Vec<Edge>)> = vec![
        (
            "self edge",
            vec![("R", "A", 1.0), ("A", "A", 1.0), ("A", "B", 1.0)],
        ),
        ("only a self edge", vec![("A", "A", 1.0)]),
        (
            "mutual",
            vec![("R", "A", 1.0), ("A", "B", 1.0), ("B", "A", 1.0)],
        ),
        (
            "pure cycle",
            vec![("A", "B", 1.0), ("B", "C", 1.0), ("C", "A", 1.0)],
        ),
        (
            "longer cycle fed by a root",
            vec![
                ("R", "A", 1.0),
                ("A", "B", 1.0),
                ("B", "C", 1.0),
                ("C", "D", 1.0),
                ("D", "A", 1.0),
                ("D", "L", 1.0),
            ],
        ),
        (
            "repeated rows",
            vec![
                ("R", "A", 1.0),
                ("R", "A", 1.0),
                ("R", "B", 1.0),
                ("R", "B", 2.0),
            ],
        ),
        (
            "two parents",
            vec![
                ("R", "A", 1.0),
                ("R", "B", 1.0),
                ("A", "C", 1.0),
                ("B", "C", 1.0),
            ],
        ),
    ];
    for (name, edges) in cases {
        let edges: Vec<(String, String, f64)> = edges
            .into_iter()
            .map(|(p, c, l)| (p.to_string(), c.to_string(), l))
            .collect();
        let (tree, svg) = bounded(8 << 20, move || {
            let tree = PhyloTree::from_edges(&edges);
            let svg = render(tree.clone().with_phylogram());
            (tree, svg)
        });
        assert_is_tree(&tree);
        assert!(svg.contains("<svg"), "{name}");
    }

    // The first parent given wins, and the repeated row's second length is ignored.
    let t = PhyloTree::from_edges(&[
        ("R", "A", 1.0),
        ("R", "A", 1.0),
        ("R", "B", 1.0),
        ("R", "B", 2.0),
    ]);
    assert_eq!(t.nodes[t.root].children.len(), 2);
    assert!(t
        .nodes
        .iter()
        .all(|n| n.branch_length == 1.0 || n.id == t.root));
    let t = PhyloTree::from_edges(&[("R", "A", 1.0), ("A", "B", 1.0), ("B", "A", 1.0)]);
    assert_eq!(t.leaf_labels_top_to_bottom(), vec!["B".to_string()]);
}

#[test]
fn many_repeated_edges_build_in_linear_time() {
    // A 200k-node chain plus a back edge from the bottom to the top, then every
    // row repeated. Quadratic cycle checks would take minutes here.
    let n = 200_000;
    let mut edges: Vec<(String, String, f64)> = (0..n)
        .map(|i| (format!("n{i}"), format!("n{}", i + 1), 1.0))
        .collect();
    edges.push((format!("n{n}"), "n0".to_string(), 1.0));
    edges.extend(edges.clone());
    let tree = bounded(8 << 20, move || PhyloTree::from_edges(&edges));
    assert_eq!(tree.nodes.len(), n + 1);
    assert_eq!(tree.leaf_labels_top_to_bottom(), vec![format!("n{n}")]);
}

#[test]
fn hand_built_cyclic_tree_renders() {
    let node = |id: usize, label: &str, children: Vec<usize>| PhyloNode {
        id,
        label: Some(label.to_string()),
        parent: None,
        children,
        branch_length: 1.0,
        support: None,
    };
    let mut tree = PhyloTree::from_newick("(A,B);");
    // 0 -> 1, 2; 1 -> 0 (back edge), 2; 2 -> 2 (self), 99 (out of range).
    tree.nodes = vec![
        node(0, "root", vec![1, 2, 2]),
        node(1, "A", vec![0, 2]),
        node(2, "B", vec![2, 99]),
    ];
    tree.root = 0;
    let t2 = tree.clone();
    let labels = bounded(8 << 20, move || t2.leaf_labels_top_to_bottom());
    // The back edge, the self edge, the repeat and the out-of-range id are
    // dropped, which leaves A and B as leaves under the root.
    assert_eq!(labels, vec!["A".to_string(), "B".to_string()]);
    let svg = bounded(8 << 20, move || {
        render(tree.clone().with_phylogram().with_clade_color(0, "red"))
    });
    assert!(svg.contains("<svg"));
}

#[test]
fn linkage_rows_pointing_forward_or_reusing_a_node_still_render() {
    // Row 0 points at node 4 (made by row 1) and reuses leaf 0; row 1 points
    // back at node 3 and at an index out of range.
    let linkage = [
        [4.0, 0.0, 1.0, 2.0],
        [3.0, 0.0, 2.0, 2.0],
        [1.0, 1e9, 3.0, 2.0],
    ];
    let tree = bounded(8 << 20, move || {
        PhyloTree::from_linkage(&["a", "b", "c"], &linkage)
    });
    assert_is_tree(&tree);
    let svg = bounded(8 << 20, move || render(tree));
    assert!(svg.contains("<svg"));
}
