//! The legend box width must not depend on the order in which the `with_legend_*`
//! builders are called. Regression for a clipped legend title (title set before
//! entries used to be discarded), generalised to all contributors.

use kuva::prelude::{LegendEntry, LegendShape};
use kuva::render::layout::Layout;

fn layout() -> Layout {
    Layout::new((0.0, 1.0), (0.0, 1.0))
}

fn entry(label: &str) -> LegendEntry {
    LegendEntry {
        label: label.into(),
        color: "steelblue".into(),
        shape: LegendShape::Rect,
        dasharray: None,
    }
}

const LONG_TITLE: &str = "Experimental cohorts (2024 pilot study)";
const LONG_GROUP_TITLE: &str = "Group with a rather long descriptive heading";

#[test]
fn title_and_entries_order_independent() {
    let title_first = layout()
        .with_legend_title(LONG_TITLE)
        .with_legend_entries(vec![entry("A"), entry("B")])
        .legend_width;
    let entries_first = layout()
        .with_legend_entries(vec![entry("A"), entry("B")])
        .with_legend_title(LONG_TITLE)
        .legend_width;
    assert_eq!(title_first, entries_first, "legend width must not depend on call order");
}

#[test]
fn long_title_widens_box_in_either_order() {
    // The long title is wider than the short entries, so it must drive the width
    // regardless of order — the original clipping bug occurred when entries were
    // set after the title and overwrote the reserved width.
    let entries_only = layout().with_legend_entries(vec![entry("A"), entry("B")]).legend_width;
    let title_first = layout()
        .with_legend_title(LONG_TITLE)
        .with_legend_entries(vec![entry("A"), entry("B")])
        .legend_width;
    let entries_first = layout()
        .with_legend_entries(vec![entry("A"), entry("B")])
        .with_legend_title(LONG_TITLE)
        .legend_width;
    assert!(title_first > entries_only, "title should widen the box");
    assert!(entries_first > entries_only, "title should widen the box in either order");
}

#[test]
fn group_and_entries_order_independent() {
    let group_first = layout()
        .with_legend_group(LONG_GROUP_TITLE, vec![entry("a")])
        .with_legend_entries(vec![entry("b")])
        .legend_width;
    let entries_first = layout()
        .with_legend_entries(vec![entry("b")])
        .with_legend_group(LONG_GROUP_TITLE, vec![entry("a")])
        .legend_width;
    assert_eq!(group_first, entries_first);
}

#[test]
fn explicit_width_wins_regardless_of_order() {
    let override_first = layout()
        .with_legend_width(250.0)
        .with_legend_entries(vec![entry("short")])
        .with_legend_title(LONG_TITLE)
        .legend_width;
    let override_last = layout()
        .with_legend_entries(vec![entry("short")])
        .with_legend_title(LONG_TITLE)
        .with_legend_width(250.0)
        .legend_width;
    assert_eq!(override_first, 250.0, "explicit width must win when set first");
    assert_eq!(override_last, 250.0, "explicit width must win when set last");
}

#[test]
fn widest_contributor_wins_across_permutations() {
    // Whatever order title, entries, and a group are added, the widest contributor
    // determines the width.
    let a = layout()
        .with_legend_title(LONG_TITLE)
        .with_legend_entries(vec![entry("x")])
        .with_legend_group(LONG_GROUP_TITLE, vec![entry("yy")])
        .legend_width;
    let b = layout()
        .with_legend_group(LONG_GROUP_TITLE, vec![entry("yy")])
        .with_legend_title(LONG_TITLE)
        .with_legend_entries(vec![entry("x")])
        .legend_width;
    let c = layout()
        .with_legend_entries(vec![entry("x")])
        .with_legend_group(LONG_GROUP_TITLE, vec![entry("yy")])
        .with_legend_title(LONG_TITLE)
        .legend_width;
    assert_eq!(a, b);
    assert_eq!(b, c);
}
