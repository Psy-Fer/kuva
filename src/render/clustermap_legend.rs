//! Category resolution and legend entries for [`Clustermap`] annotation tracks and
//! categorical / binned cells. The types and builders live in `plot::clustermap`; these methods
//! need render-side pieces (the default palette, normalization), so they live here to keep the
//! plot layer free of render imports.

use std::collections::{HashMap, HashSet};

use crate::plot::clustermap::{
    is_na, AnnotationTrack, CategoryColors, CategoryEntry, CategoryStyle, Clustermap, NA_COLOR,
};
use crate::render::palette::Palette;

impl CategoryStyle {
    /// The categories present in `values`, in display order, with their colours and labels.
    ///
    /// Order is `order` first (only categories that occur), then the others in order of first
    /// appearance, so it never depends on hash order. Palette colours are handed out in that
    /// order, skipping categories with an explicit colour. A missing value (empty or blank) is
    /// listed last as `NA` in [`NA_COLOR`], and only if it occurs.
    pub fn resolve<'a>(&self, values: impl IntoIterator<Item = &'a str>) -> Vec<CategoryEntry> {
        self.resolve_from(values, 0)
    }

    /// [`resolve`](Self::resolve), taking palette colours from index `start` onward. Used to
    /// keep several tracks on the default palette from reusing the same colours.
    pub(crate) fn resolve_from<'a>(
        &self,
        values: impl IntoIterator<Item = &'a str>,
        start: usize,
    ) -> Vec<CategoryEntry> {
        // Sets and maps beside the lists keep this linear in the number of
        // values; with mostly distinct values the list searches were quadratic.
        let mut seen: Vec<&str> = Vec::new();
        let mut seen_set: HashSet<&str> = HashSet::new();
        let mut has_na = false;
        for v in values {
            if is_na(v) {
                has_na = true;
            } else if seen_set.insert(v) {
                seen.push(v);
            }
        }
        // A name repeated in `order` is listed each time, as before.
        let mut ordered: Vec<&str> = self
            .order
            .iter()
            .map(String::as_str)
            .filter(|c| seen_set.contains(c))
            .collect();
        let in_order: HashSet<&str> = ordered.iter().copied().collect();
        ordered.extend(seen.iter().copied().filter(|c| !in_order.contains(c)));
        // Explicit colours and labels; the first one given for a value wins.
        let mut colors: HashMap<&str, &str> = HashMap::new();
        for (k, col) in &self.colors {
            colors.entry(k.as_str()).or_insert(col.as_str());
        }
        let mut labels: HashMap<&str, &str> = HashMap::new();
        for (k, l) in &self.labels {
            labels.entry(k.as_str()).or_insert(l.as_str());
        }
        let default_palette;
        let palette: &[String] = match self.palette {
            Some(ref p) if !p.0.is_empty() => &p.0,
            _ => {
                default_palette = Palette::category10();
                default_palette.colors()
            }
        };
        let mut next = start;
        let mut out: Vec<CategoryEntry> = ordered
            .into_iter()
            .map(|c| {
                let color = match colors.get(c) {
                    Some(&col) => col.to_string(),
                    None => {
                        let col = palette[next % palette.len().max(1)].clone();
                        next += 1;
                        col
                    }
                };
                let label = labels
                    .get(c)
                    .map_or_else(|| c.to_string(), |l| l.to_string());
                CategoryEntry {
                    value: c.to_string(),
                    label,
                    color,
                }
            })
            .collect();
        if has_na {
            out.push(CategoryEntry {
                value: String::new(),
                label: self.na_label.clone().unwrap_or_else(|| "NA".to_string()),
                color: NA_COLOR.to_string(),
            });
        }
        out
    }
}

impl AnnotationTrack {
    /// The resolved categories of a categorical track, in legend order; empty otherwise.
    ///
    /// On its own a track starts at the palette's first colour. Inside a [`Clustermap`],
    /// tracks on the default palette share it instead (see
    /// [`Clustermap::annotation_entries`]), so different tracks don't reuse colours.
    pub fn category_entries(&self) -> Vec<CategoryEntry> {
        self.category_entries_from(0)
    }

    pub(crate) fn category_entries_from(&self, start: usize) -> Vec<CategoryEntry> {
        match &self.categories {
            Some(values) => self
                .style
                .resolve_from(values.iter().map(String::as_str), start),
            None => Vec::new(),
        }
    }

    /// How many palette colours this track's categories take (those without an explicit
    /// colour, NA excluded).
    pub(crate) fn palette_colors_used(&self, entries: &[CategoryEntry]) -> usize {
        let explicit: HashSet<&str> = self.style.colors.iter().map(|(k, _)| k.as_str()).collect();
        entries
            .iter()
            .filter(|e| !e.value.is_empty() && !explicit.contains(e.value.as_str()))
            .count()
    }

    /// The strip colour for the item at original index `i`. `colors` is built
    /// once from the track's resolved entries.
    pub(crate) fn color_at(&self, i: usize, colors: &CategoryColors) -> String {
        match &self.categories {
            Some(values) => colors
                .get(values.get(i).map_or("", String::as_str))
                .to_string(),
            None => self
                .colors
                .get(i)
                .cloned()
                .unwrap_or_else(|| NA_COLOR.to_string()),
        }
    }
}

impl Clustermap {
    /// Legend entries for the cells in categorical or discrete mode (`None` for a colorbar):
    /// `(title, entries)`, titled by [`with_legend`](Self::with_legend) or `"Value"`.
    pub fn cell_legend_group(&self) -> Option<(String, Vec<CategoryEntry>)> {
        let entries = if let Some(ref d) = self.categorical_data {
            self.cell_style
                .resolve(d.iter().flatten().map(String::as_str))
        } else if let Some(ref map) = self.discrete_color_map {
            let fixed = self.shape_fixed();
            let cm = fixed.as_ref().unwrap_or(self);
            let values =
                crate::render::render::apply_normalization(cm.data.clone(), &cm.normalization);
            let mut entries = map.entries(values.iter().flatten());
            if let Some(ref na) = self.cell_style.na_label {
                for e in entries.iter_mut().filter(|e| e.value.is_empty()) {
                    e.label = na.clone();
                }
            }
            entries
        } else {
            return None;
        };
        let title = self
            .legend_label
            .clone()
            .unwrap_or_else(|| "Value".to_string());
        (!entries.is_empty()).then_some((title, entries))
    }

    /// Legend groups for the categorical annotation tracks: `(title, entries)` per track,
    /// column tracks first, in the order they were added. Empty when
    /// [`with_annotation_legend(false)`](Self::with_annotation_legend).
    pub fn annotation_legend_groups(&self) -> Vec<(String, Vec<CategoryEntry>)> {
        if !self.annotation_legend {
            return Vec::new();
        }
        let (col, row) = self.annotation_entries();
        self.col_annotations
            .iter()
            .zip(col)
            .chain(self.row_annotations.iter().zip(row))
            .filter(|(t, e)| t.categories.is_some() && !e.is_empty())
            .map(|(t, e)| {
                (
                    t.label.clone().unwrap_or_else(|| "Annotation".to_string()),
                    e,
                )
            })
            .collect()
    }

    /// Resolved categories for every annotation track, `(column tracks, row tracks)`, each in
    /// the order added (empty for a raw-colour track).
    ///
    /// Tracks that set no palette share the default one (category10): each continues from
    /// where the previous one stopped, column tracks first, so no two tracks reuse a colour
    /// until the palette runs out. A track with its own palette starts at that palette's
    /// first colour.
    pub fn annotation_entries(&self) -> (Vec<Vec<CategoryEntry>>, Vec<Vec<CategoryEntry>>) {
        let mut cursor = 0usize;
        let mut resolve = |t: &AnnotationTrack| {
            if t.style.palette.is_some() {
                return t.category_entries_from(0);
            }
            let entries = t.category_entries_from(cursor);
            cursor += t.palette_colors_used(&entries);
            entries
        };
        let col = self.col_annotations.iter().map(&mut resolve).collect();
        let row = self.row_annotations.iter().map(&mut resolve).collect();
        (col, row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list-search version `resolve_from` replaced.
    fn resolve_ref<'a>(
        style: &CategoryStyle,
        values: impl IntoIterator<Item = &'a str>,
        start: usize,
    ) -> Vec<CategoryEntry> {
        let mut seen: Vec<&str> = Vec::new();
        let mut has_na = false;
        for v in values {
            if is_na(v) {
                has_na = true;
            } else if !seen.contains(&v) {
                seen.push(v);
            }
        }
        let mut ordered: Vec<&str> = style
            .order
            .iter()
            .map(String::as_str)
            .filter(|c| seen.contains(c))
            .collect();
        for &c in &seen {
            if !ordered.contains(&c) {
                ordered.push(c);
            }
        }
        let default_palette = Palette::category10();
        let palette: &[String] = match style.palette {
            Some(ref p) => &p.0,
            None => default_palette.colors(),
        };
        let mut next = start;
        let mut out: Vec<CategoryEntry> = ordered
            .into_iter()
            .map(|c| {
                let color = match style.colors.iter().find(|(k, _)| k == c) {
                    Some((_, col)) => col.clone(),
                    None => {
                        let col = palette[next % palette.len().max(1)].clone();
                        next += 1;
                        col
                    }
                };
                let label = style
                    .labels
                    .iter()
                    .find(|(k, _)| k == c)
                    .map_or_else(|| c.to_string(), |(_, l)| l.clone());
                CategoryEntry {
                    value: c.to_string(),
                    label,
                    color,
                }
            })
            .collect();
        if has_na {
            out.push(CategoryEntry {
                value: String::new(),
                label: style.na_label.clone().unwrap_or_else(|| "NA".to_string()),
                color: NA_COLOR.to_string(),
            });
        }
        out
    }

    #[test]
    fn hashed_resolution_matches_the_list_searches() {
        let pool = ["a", "b", "c", "d", "e", "", " "];
        let state = std::cell::Cell::new(0x0bad_cafe_f00d_1234_u64);
        let next = || {
            let mut x = state.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            state.set(x);
            x
        };
        let pick = |n: u64| -> Vec<String> {
            (0..(next() % n))
                .map(|_| pool[(next() % 5) as usize].to_string())
                .collect()
        };
        for _ in 0..400 {
            // Repeated names in `order`, and repeated keys in colours and
            // labels, where the first one wins.
            let style = CategoryStyle {
                palette: None,
                colors: pick(4)
                    .into_iter()
                    .enumerate()
                    .map(|(i, k)| (k, format!("#00000{i}")))
                    .collect(),
                labels: pick(4)
                    .into_iter()
                    .enumerate()
                    .map(|(i, k)| (k, format!("L{i}")))
                    .collect(),
                order: pick(5),
                na_label: None,
            };
            let values: Vec<&str> = (0..(next() % 15))
                .map(|_| pool[(next() % pool.len() as u64) as usize])
                .collect();
            let start = (next() % 3) as usize;
            let got = style.resolve_from(values.iter().copied(), start);
            assert_eq!(
                got,
                resolve_ref(&style, values.iter().copied(), start),
                "{style:?} {values:?}"
            );
            let colors = CategoryColors::new(&got);
            for v in &values {
                let want = if is_na(v) {
                    NA_COLOR
                } else {
                    got.iter()
                        .find(|e| e.value == *v)
                        .map_or(NA_COLOR, |e| e.color.as_str())
                };
                assert_eq!(colors.get(v), want);
            }
        }
    }
}
