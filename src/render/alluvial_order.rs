use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::plot::sankey::{SankeyAlluvium, SankeyLink};

const MATRIX_INIT: f64 = 1.0e6;
const SAME_SIDE_MATRIX_INIT: f64 = 1.0e6;
const WEIGHT_SCALAR: f64 = 5.0e5;

#[derive(Debug, Clone)]
pub(crate) struct SankeyOrderingResult {
    pub col: Vec<usize>,
    pub nodes_in_col: Vec<Vec<usize>>,
    #[allow(dead_code)]
    pub column_order: Vec<usize>,
}

struct FenwickTree {
    tree: Vec<f64>,
}

struct DeterministicRng {
    mt: [u32; 624],
    mti: usize,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        let mut seed = seed as u32;
        for _ in 0..50 {
            seed = seed.wrapping_mul(69069).wrapping_add(1);
        }

        let mut mt = [0u32; 624];
        for j in 0..625 {
            seed = seed.wrapping_mul(69069).wrapping_add(1);
            if j > 0 {
                mt[j - 1] = seed;
            }
        }

        Self { mt, mti: 624 }
    }

    fn unif_rand(&mut self) -> f64 {
        const N: usize = 624;
        const M: usize = 397;
        const MATRIX_A: u32 = 0x9908_B0DF;
        const UPPER_MASK: u32 = 0x8000_0000;
        const LOWER_MASK: u32 = 0x7FFF_FFFF;
        const I2_32M1: f64 = 2.328_306_437_080_797e-10;

        if self.mti >= N {
            let mag01 = [0u32, MATRIX_A];
            for kk in 0..(N - M) {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M] ^ (y >> 1) ^ mag01[(y & 0x1) as usize];
            }
            for kk in (N - M)..(N - 1) {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M - N] ^ (y >> 1) ^ mag01[(y & 0x1) as usize];
            }
            let y = (self.mt[N - 1] & UPPER_MASK) | (self.mt[0] & LOWER_MASK);
            self.mt[N - 1] = self.mt[M - 1] ^ (y >> 1) ^ mag01[(y & 0x1) as usize];
            self.mti = 0;
        }

        let mut y = self.mt[self.mti];
        self.mti += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9D2C_5680;
        y ^= (y << 15) & 0xEFC6_0000;
        y ^= y >> 18;

        let x = y as f64 * 2.328_306_436_538_696_3e-10;
        if x <= 0.0 {
            0.5 * I2_32M1
        } else if (1.0 - x) <= 0.0 {
            1.0 - 0.5 * I2_32M1
        } else {
            x
        }
    }

    fn rbits(&mut self, bits: usize) -> u64 {
        let mut v = 0u64;
        let mut n = 0usize;
        while n <= bits {
            let v1 = (self.unif_rand() * 65536.0).floor() as u64;
            v = 65536u64.wrapping_mul(v).wrapping_add(v1);
            n += 16;
        }
        let mask = if bits >= 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };
        v & mask
    }

    fn gen_index(&mut self, upper: usize) -> usize {
        if upper == 0 {
            return 0;
        }
        let bits = (upper as f64).log2().ceil() as usize;
        loop {
            let v = self.rbits(bits);
            if v < upper as u64 {
                return v as usize;
            }
        }
    }

    fn sample_permutation(&mut self, n: usize) -> Vec<usize> {
        let mut pool: Vec<usize> = (0..n).collect();
        let mut remaining = n;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let j = self.gen_index(remaining);
            out.push(pool[j]);
            remaining -= 1;
            if j < remaining {
                pool[j] = pool[remaining];
            }
        }
        out
    }
}

impl FenwickTree {
    fn new(size: usize) -> Self {
        Self {
            tree: vec![0.0; size + 1],
        }
    }

    fn add(&mut self, index: usize, value: f64) {
        let mut i = index + 1;
        while i < self.tree.len() {
            self.tree[i] += value;
            i += i & i.wrapping_neg();
        }
    }

    fn prefix_sum(&self, end_exclusive: usize) -> f64 {
        let mut total = 0.0;
        let mut i = end_exclusive;
        while i > 0 {
            total += self.tree[i];
            i &= i - 1;
        }
        total
    }
}

fn crossing_objective_from_pairs(pairs: &mut Vec<(f64, f64, f64)>) -> f64 {
    if pairs.len() <= 1 {
        return 0.0;
    }

    pairs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut y2_sorted: Vec<f64> = pairs.iter().map(|(_, y2, _)| *y2).collect();
    y2_sorted.sort_by(|a, b| a.total_cmp(b));
    y2_sorted.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    let mut bit = FenwickTree::new(y2_sorted.len());
    let mut seen_weight = 0.0;
    let mut total = 0.0;
    for (_, y2, weight) in pairs.iter().copied() {
        let target_pos = y2_sorted
            .binary_search_by(|probe| probe.total_cmp(&y2))
            .expect("compressed y rank exists");
        let not_above = bit.prefix_sum(target_pos + 1);
        let above = seen_weight - not_above;
        total += weight * above;
        bit.add(target_pos, weight);
        seen_weight += weight;
    }
    total
}

fn build_node_positions(nodes_in_col: &[Vec<usize>], n_nodes: usize) -> Vec<usize> {
    let mut pos = vec![usize::MAX; n_nodes];
    for members in nodes_in_col {
        for (i, &node) in members.iter().enumerate() {
            pos[node] = i;
        }
    }
    pos
}

fn make_display_col(column_order: &[usize], col_orig: &[usize]) -> Vec<usize> {
    let mut axis_to_display = vec![usize::MAX; column_order.len()];
    for (display_idx, &axis) in column_order.iter().enumerate() {
        axis_to_display[axis] = display_idx;
    }
    col_orig.iter().map(|&axis| axis_to_display[axis]).collect()
}

fn reorder_nodes_into_display_columns(
    column_order: &[usize],
    nodes_by_axis: &[Vec<usize>],
) -> Vec<Vec<usize>> {
    column_order
        .iter()
        .map(|&axis| nodes_by_axis[axis].clone())
        .collect()
}

fn axis_nodes_from_cycle(cycle: &[usize], col_orig: &[usize], n_axes: usize) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new(); n_axes];
    for &node in cycle {
        out[col_orig[node]].push(node);
    }
    out
}

fn barycentric_refine_links(
    mut nodes_in_axis: Vec<Vec<usize>>,
    col_orig: &[usize],
    links: &[SankeyLink],
) -> Vec<Vec<usize>> {
    if nodes_in_axis.len() <= 1 {
        return nodes_in_axis;
    }
    for _ in 0..4 {
        let pos = build_node_positions(&nodes_in_axis, col_orig.len());
        for axis in 1..nodes_in_axis.len() {
            let prev = axis - 1;
            nodes_in_axis[axis].sort_by(|&a, &b| {
                let bary = |node: usize| {
                    let mut weighted_sum = 0.0;
                    let mut weight_total = 0.0;
                    for link in links {
                        if link.target == node && col_orig[link.source] == prev {
                            weighted_sum += pos[link.source] as f64 * link.value;
                            weight_total += link.value;
                        }
                        if link.source == node && col_orig[link.target] == prev {
                            weighted_sum += pos[link.target] as f64 * link.value;
                            weight_total += link.value;
                        }
                    }
                    if weight_total > 0.0 {
                        weighted_sum / weight_total
                    } else {
                        pos[node] as f64
                    }
                };
                bary(a).total_cmp(&bary(b)).then(pos[a].cmp(&pos[b]))
            });
        }

        let pos = build_node_positions(&nodes_in_axis, col_orig.len());
        for axis in (0..nodes_in_axis.len() - 1).rev() {
            let next = axis + 1;
            nodes_in_axis[axis].sort_by(|&a, &b| {
                let bary = |node: usize| {
                    let mut weighted_sum = 0.0;
                    let mut weight_total = 0.0;
                    for link in links {
                        if link.source == node && col_orig[link.target] == next {
                            weighted_sum += pos[link.target] as f64 * link.value;
                            weight_total += link.value;
                        }
                        if link.target == node && col_orig[link.source] == next {
                            weighted_sum += pos[link.source] as f64 * link.value;
                            weight_total += link.value;
                        }
                    }
                    if weight_total > 0.0 {
                        weighted_sum / weight_total
                    } else {
                        pos[node] as f64
                    }
                };
                bary(a).total_cmp(&bary(b)).then(pos[a].cmp(&pos[b]))
            });
        }
    }
    nodes_in_axis
}

fn build_distance_matrix_from_links(col_orig: &[usize], links: &[SankeyLink]) -> Vec<Vec<f64>> {
    let n = col_orig.len();
    let mut mat = vec![vec![MATRIX_INIT; n]; n];
    for (i, row) in mat.iter_mut().enumerate() {
        row[i] = 0.0;
    }

    if (SAME_SIDE_MATRIX_INIT - MATRIX_INIT).abs() > f64::EPSILON {
        for i in 0..n {
            for j in 0..n {
                if i != j && col_orig[i] == col_orig[j] {
                    mat[i][j] = SAME_SIDE_MATRIX_INIT;
                }
            }
        }
    }

    for link in links {
        if link.value > 0.0 {
            let d = WEIGHT_SCALAR * -link.value.ln();
            mat[link.source][link.target] = d;
            mat[link.target][link.source] = d;
        }
    }

    let min_val = mat
        .iter()
        .flat_map(|row| row.iter().copied())
        .fold(f64::INFINITY, f64::min);
    let shift = if min_val <= 0.0 {
        min_val.abs() + 1.0
    } else {
        0.0
    };
    if shift > 0.0 {
        for row in &mut mat {
            for val in row {
                *val += shift;
            }
        }
    }
    mat
}

fn build_distance_matrix_from_alluvia(
    col_orig: &[usize],
    alluvia: &[SankeyAlluvium],
) -> Vec<Vec<f64>> {
    let n = col_orig.len();
    let mut mat = vec![vec![MATRIX_INIT; n]; n];
    for (i, row) in mat.iter_mut().enumerate() {
        row[i] = 0.0;
    }

    if (SAME_SIDE_MATRIX_INIT - MATRIX_INIT).abs() > f64::EPSILON {
        for i in 0..n {
            for j in 0..n {
                if i != j && col_orig[i] == col_orig[j] {
                    mat[i][j] = SAME_SIDE_MATRIX_INIT;
                }
            }
        }
    }

    let mut pair_weights = HashMap::<(usize, usize), f64>::new();
    for row in alluvia {
        for i in 0..row.nodes.len() {
            for j in i + 1..row.nodes.len() {
                let a = row.nodes[i];
                let b = row.nodes[j];
                let key = if a < b { (a, b) } else { (b, a) };
                *pair_weights.entry(key).or_insert(0.0) += row.value;
            }
        }
    }
    for ((a, b), weight) in pair_weights {
        if weight > 0.0 {
            let d = WEIGHT_SCALAR * -weight.ln();
            mat[a][b] = d;
            mat[b][a] = d;
        }
    }

    let min_val = mat
        .iter()
        .flat_map(|row| row.iter().copied())
        .fold(f64::INFINITY, f64::min);
    let shift = if min_val <= 0.0 {
        min_val.abs() + 1.0
    } else {
        0.0
    };
    if shift > 0.0 {
        for row in &mut mat {
            for val in row {
                *val += shift;
            }
        }
    }
    mat
}

#[allow(dead_code)]
fn tour_cost(cycle: &[usize], mat: &[Vec<f64>]) -> f64 {
    if cycle.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..cycle.len() {
        let a = cycle[i];
        let b = cycle[(i + 1) % cycle.len()];
        total += mat[a][b];
    }
    total
}

fn rotate_left<T: Copy>(v: &[T], k: usize) -> Vec<T> {
    if v.is_empty() {
        return Vec::new();
    }
    let k = k % v.len();
    let mut out = Vec::with_capacity(v.len());
    out.extend_from_slice(&v[k..]);
    out.extend_from_slice(&v[..k]);
    out
}

fn exact_tsp_cycle_from_start(mat: &[Vec<f64>], start: usize) -> Option<Vec<usize>> {
    let n = mat.len();
    if n == 0 {
        return Some(Vec::new());
    }
    if n > 12 {
        return None;
    }

    let size = 1usize << n;
    let inf = f64::INFINITY;
    let mut dp = vec![vec![inf; n]; size];
    let mut parent = vec![vec![usize::MAX; n]; size];
    dp[1 << start][start] = 0.0;

    for mask in 1usize..size {
        if mask & (1 << start) == 0 {
            continue;
        }
        for last in 0..n {
            let cur = dp[mask][last];
            if !cur.is_finite() {
                continue;
            }
            for next in 0..n {
                if mask & (1 << next) != 0 {
                    continue;
                }
                let new_mask = mask | (1 << next);
                let cand = cur + mat[last][next];
                if cand < dp[new_mask][next] {
                    dp[new_mask][next] = cand;
                    parent[new_mask][next] = last;
                }
            }
        }
    }

    let full = size - 1;
    let mut best_last = start;
    let mut best_cost = inf;
    for last in 0..n {
        if last == start {
            continue;
        }
        let cand = dp[full][last] + mat[last][start];
        if cand < best_cost {
            best_cost = cand;
            best_last = last;
        }
    }
    if !best_cost.is_finite() {
        return None;
    }

    let mut order = vec![0usize; n];
    let mut mask = full;
    let mut last = best_last;
    order[0] = start;
    for idx in (1..n).rev() {
        order[idx] = last;
        let prev = parent[mask][last];
        mask ^= 1 << last;
        last = prev;
    }
    let start_pos = order.iter().position(|&node| node == start).unwrap_or(0);
    Some(rotate_left(&order, start_pos))
}

#[allow(dead_code)]
fn canonicalize_cycle(cycle: &[usize]) -> Vec<usize> {
    if cycle.is_empty() {
        return Vec::new();
    }
    let min_pos = cycle
        .iter()
        .enumerate()
        .min_by_key(|(_, node)| *node)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let forward = rotate_left(cycle, min_pos);
    let mut reversed = cycle.to_vec();
    reversed.reverse();
    let min_pos_rev = reversed
        .iter()
        .enumerate()
        .min_by_key(|(_, node)| *node)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let reverse_canonical = rotate_left(&reversed, min_pos_rev);
    if reverse_canonical < forward {
        reverse_canonical
    } else {
        forward
    }
}

#[allow(dead_code)]
fn exact_tsp_cycles(mat: &[Vec<f64>]) -> Option<Vec<Vec<usize>>> {
    let n = mat.len();
    if n == 0 {
        return Some(vec![Vec::new()]);
    }
    if n > 12 {
        return None;
    }

    let mut cycles = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for start in 0..n {
        if let Some(cycle) = exact_tsp_cycle_from_start(mat, start) {
            let canonical = canonicalize_cycle(&cycle);
            if seen.insert(canonical.clone()) {
                cycles.push(canonical);
            }
        }
    }
    Some(cycles)
}

#[allow(dead_code)]
fn heuristic_tsp_cycle(mat: &[Vec<f64>]) -> Vec<usize> {
    let n = mat.len();
    if n == 0 {
        return Vec::new();
    }

    let mut best_cycle = Vec::new();
    let mut best_cost = f64::INFINITY;
    for start in 0..n {
        let mut visited = vec![false; n];
        let mut cycle = Vec::with_capacity(n);
        let mut cur = start;
        visited[cur] = true;
        cycle.push(cur);
        while cycle.len() < n {
            let mut next = None;
            let mut next_cost = f64::INFINITY;
            for (candidate, seen) in visited.iter().enumerate() {
                if *seen {
                    continue;
                }
                let cost = mat[cur][candidate];
                if cost < next_cost {
                    next_cost = cost;
                    next = Some(candidate);
                }
            }
            let next = next.expect("unvisited node exists");
            visited[next] = true;
            cycle.push(next);
            cur = next;
        }
        let cycle = two_opt_best_improvement(cycle, mat);
        let cost = tour_cost(&cycle, mat);
        if cost < best_cost {
            best_cost = cost;
            best_cycle = cycle;
        }
    }
    best_cycle
}

fn arbitrary_insertion_cycle(mat: &[Vec<f64>], rng: &mut DeterministicRng) -> Vec<usize> {
    let n = mat.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![0];
    }
    if n == 2 {
        let mut order = vec![0, 1];
        order.swap(1, rng.gen_index(2));
        return order;
    }

    let rorder = rng.sample_permutation(n);

    // Mirror TSP::tsp_insertion_arbitrary(): operate on the randomly permuted
    // matrix and then map the final cycle back through rorder.
    let mut order = vec![0usize; n];
    order[0] = 0;
    order[1] = 1;
    for city in 2..n {
        let used = &order[..city];
        let mut best_pos = 0usize;
        let mut best_cost = f64::INFINITY;
        for pos in 0..used.len() {
            let a = rorder[used[pos]];
            let b = rorder[used[(pos + 1) % used.len()]];
            let k = rorder[city];
            let cost = mat[a][k] + mat[k][b] - mat[a][b];
            if cost < best_cost {
                best_cost = cost;
                best_pos = pos + 1;
            }
        }
        for idx in (best_pos..city).rev() {
            order[idx + 1] = order[idx];
        }
        order[best_pos] = city;
    }

    order[..n].iter().map(|&idx| rorder[idx]).collect()
}

fn two_opt_best_improvement(mut cycle: Vec<usize>, mat: &[Vec<f64>]) -> Vec<usize> {
    if cycle.len() < 4 {
        return cycle;
    }
    let n = cycle.len();
    loop {
        let mut swaps = 0usize;
        let mut swap1 = 0usize;
        let mut swap2 = 0usize;
        let mut imp_best = 0.0;

        for i in 1..n - 1 {
            let mut imp = 0.0;
            imp += mat[cycle[i - 1]][cycle[i]];
            imp += mat[cycle[i]][cycle[i + 1]];

            for j in i + 1..n - 1 {
                imp += mat[cycle[j]][cycle[j + 1]];
                imp -= mat[cycle[j]][cycle[j - 1]];

                let imp_tmp = imp - mat[cycle[i - 1]][cycle[j]] - mat[cycle[i]][cycle[j + 1]];
                if imp_tmp > 1.0e-7 {
                    swaps += 1;
                    if imp_tmp > imp_best {
                        imp_best = imp_tmp;
                        swap1 = i;
                        swap2 = j;
                    }
                }
            }

            let j = n - 1;
            imp -= mat[cycle[j]][cycle[j - 1]];
            let imp_tmp = imp - mat[cycle[i - 1]][cycle[j]] - mat[cycle[i]][cycle[0]];
            if imp_tmp > 1.0e-7 {
                swaps += 1;
                if imp_tmp > imp_best {
                    imp_best = imp_tmp;
                    swap1 = i;
                    swap2 = j;
                }
            }
        }

        if swaps == 0 {
            break;
        }
        cycle[swap1..=swap2].reverse();
    }
    cycle
}

fn tsp_cycle(mat: &[Vec<f64>], rng: &mut DeterministicRng) -> Vec<usize> {
    let cycle = arbitrary_insertion_cycle(mat, rng);
    two_opt_best_improvement(cycle, mat)
}

fn tsp_cycle_links(mat: &[Vec<f64>], rng: &mut DeterministicRng) -> Vec<usize> {
    let cycle = arbitrary_insertion_cycle(mat, rng);
    two_opt_best_improvement(cycle, mat)
}

fn neighbornet_cycle(mat: &[Vec<f64>]) -> Option<Vec<usize>> {
    let n = mat.len();
    if n == 0 {
        return Some(Vec::new());
    }
    if n <= 3 {
        return Some((0..n).collect());
    }

    let script = r#"
import sys
import numpy as np
from splitspy.nnet import nnet_cycle

n = int(sys.stdin.readline().strip())
rows = []
for _ in range(n):
    rows.append([float(x) for x in sys.stdin.readline().strip().split()])
mat = np.array(rows, dtype=np.float64)

max_nodes = max(3, 3 * n - 5)
full = np.zeros((max_nodes + 1, max_nodes + 1), dtype=np.float64)
full[1:n+1, 1:n+1] = mat
nodes_head = nnet_cycle.__setup_nodes(n)
joins = nnet_cycle.__join_nodes(n, full, nodes_head)
cycle = nnet_cycle.__expand_nodes(joins, nodes_head)
cycle = nnet_cycle.__normalize_cycle(cycle)
print(" ".join(str(int(x)) for x in cycle))
"#;

    let mut child = Command::new("python3")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    {
        let stdin = child.stdin.as_mut()?;
        writeln!(stdin, "{n}").ok()?;
        for row in mat {
            for (j, value) in row.iter().enumerate() {
                if j > 0 {
                    write!(stdin, " ").ok()?;
                }
                write!(stdin, "{value}").ok()?;
            }
            writeln!(stdin).ok()?;
        }
    }

    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let cycle: Vec<usize> = stdout
        .split_whitespace()
        .map(|s| s.parse::<isize>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?
        .into_iter()
        .filter(|&idx| idx > 0)
        .map(|idx| (idx - 1) as usize)
        .collect();
    if cycle.len() == n {
        Some(cycle)
    } else {
        None
    }
}

#[allow(dead_code)]
fn largest_gap_cycle_start(cycle: &[usize], mat: &[Vec<f64>]) -> Vec<usize> {
    if cycle.len() <= 1 {
        return cycle.to_vec();
    }
    let mut max_index = 0usize;
    let mut max_value = f64::NEG_INFINITY;
    for i in 0..cycle.len() {
        let a = cycle[i];
        let b = cycle[(i + 1) % cycle.len()];
        let value = mat[a][b];
        if value > max_value {
            max_value = value;
            max_index = i;
        }
    }
    rotate_left(cycle, (max_index + 1) % cycle.len())
}

fn node_positions(nodes_in_col: &[Vec<usize>], n_nodes: usize) -> Vec<usize> {
    build_node_positions(nodes_in_col, n_nodes)
}

fn make_lode_y_by_axis(
    alluvia: &[SankeyAlluvium],
    column_order: &[usize],
    col_orig: &[usize],
    nodes_in_col: &[Vec<usize>],
) -> Vec<Vec<f64>> {
    let n_axes = column_order.len();
    let n_rows = alluvia.len();
    let node_pos = node_positions(nodes_in_col, col_orig.len());
    let mut y = vec![vec![0.0; n_rows]; n_axes];

    for axis_pos in 0..n_axes {
        let axis = column_order[axis_pos];
        let tiebreak_axis = if axis_pos + 1 < n_axes {
            column_order[axis_pos + 1]
        } else {
            column_order[axis_pos - 1]
        };
        let mut order: Vec<usize> = (0..n_rows).collect();
        order.sort_by(|&a, &b| {
            let node_a = alluvia[a]
                .nodes
                .iter()
                .copied()
                .find(|&node| col_orig[node] == axis)
                .expect("alluvium contains axis");
            let node_b = alluvia[b]
                .nodes
                .iter()
                .copied()
                .find(|&node| col_orig[node] == axis)
                .expect("alluvium contains axis");
            let tie_a = alluvia[a]
                .nodes
                .iter()
                .copied()
                .find(|&node| col_orig[node] == tiebreak_axis)
                .expect("alluvium contains tiebreak axis");
            let tie_b = alluvia[b]
                .nodes
                .iter()
                .copied()
                .find(|&node| col_orig[node] == tiebreak_axis)
                .expect("alluvium contains tiebreak axis");
            node_pos[node_a]
                .cmp(&node_pos[node_b])
                .then(node_pos[tie_a].cmp(&node_pos[tie_b]))
                .then(a.cmp(&b))
        });
        let mut cum = 0.0;
        for row_idx in order {
            cum += alluvia[row_idx].value;
            y[axis_pos][row_idx] = cum;
        }
    }
    y
}

fn total_crossing_objective_alluvia(
    alluvia: &[SankeyAlluvium],
    column_order: &[usize],
    col_orig: &[usize],
    nodes_in_col: &[Vec<usize>],
) -> f64 {
    if alluvia.is_empty() || column_order.len() <= 1 {
        return 0.0;
    }
    let y = make_lode_y_by_axis(alluvia, column_order, col_orig, nodes_in_col);
    let mut total = 0.0;
    for axis_pos in 0..column_order.len() - 1 {
        let mut pairs: Vec<(f64, f64, f64)> = alluvia
            .iter()
            .enumerate()
            .map(|(i, row)| (y[axis_pos][i], y[axis_pos + 1][i], row.value))
            .collect();
        total += crossing_objective_from_pairs(&mut pairs);
    }
    total
}

pub(crate) fn optimize_sankey_alluvial_order(
    col_orig: &[usize],
    initial_nodes_in_axis: &[Vec<usize>],
    alluvia: &[SankeyAlluvium],
    links: &[SankeyLink],
    seed: u64,
    node_sort_keys: Option<&[String]>,
    use_neighbornet: bool,
) -> SankeyOrderingResult {
    if initial_nodes_in_axis.is_empty() {
        return SankeyOrderingResult {
            col: col_orig.to_vec(),
            nodes_in_col: Vec::new(),
            column_order: Vec::new(),
        };
    }

    if alluvia.is_empty() {
        let mat = build_distance_matrix_from_links(col_orig, links);
        let mut rng = DeterministicRng::new(seed);
        let cycle = if use_neighbornet {
            neighbornet_cycle(&mat).unwrap_or_else(|| tsp_cycle_links(&mat, &mut rng))
        } else {
            tsp_cycle_links(&mat, &mut rng)
        };
        let axis_nodes = barycentric_refine_links(
            axis_nodes_from_cycle(&cycle, col_orig, initial_nodes_in_axis.len()),
            col_orig,
            links,
        );
        let column_order: Vec<usize> = (0..initial_nodes_in_axis.len()).collect();
        let col = make_display_col(&column_order, col_orig);
        let nodes_in_col = reorder_nodes_into_display_columns(&column_order, &axis_nodes);
        return SankeyOrderingResult {
            col,
            nodes_in_col,
            column_order,
        };
    }

    let node_mat = build_distance_matrix_from_alluvia(col_orig, alluvia);
    let mut rng = DeterministicRng::new(seed);
    let node_cycle = if let Some(keys) = node_sort_keys {
        let mut order: Vec<usize> = (0..col_orig.len()).collect();
        order.sort_by(|&a, &b| keys[a].cmp(&keys[b]).then(a.cmp(&b)));
        let mut permuted = vec![vec![0.0; order.len()]; order.len()];
        for (i, &src_i) in order.iter().enumerate() {
            for (j, &src_j) in order.iter().enumerate() {
                permuted[i][j] = node_mat[src_i][src_j];
            }
        }
        let permuted_cycle = if use_neighbornet {
            neighbornet_cycle(&permuted).unwrap_or_else(|| tsp_cycle(&permuted, &mut rng))
        } else {
            tsp_cycle(&permuted, &mut rng)
        };
        permuted_cycle.into_iter().map(|i| order[i]).collect()
    } else {
        if use_neighbornet {
            neighbornet_cycle(&node_mat).unwrap_or_else(|| tsp_cycle(&node_mat, &mut rng))
        } else {
            tsp_cycle(&node_mat, &mut rng)
        }
    };
    let n_axes = initial_nodes_in_axis.len();

    let mut best = SankeyOrderingResult {
        col: col_orig.to_vec(),
        nodes_in_col: initial_nodes_in_axis.to_vec(),
        column_order: (0..n_axes).collect(),
    };
    let mut best_obj = f64::INFINITY;

    let fixed_column_order: Vec<usize> = (0..n_axes).collect();
    for k in 0..node_cycle.len() {
        let rotated_nodes = rotate_left(&node_cycle, k);
        let axis_nodes = axis_nodes_from_cycle(&rotated_nodes, col_orig, n_axes);

        let nodes_in_col = reorder_nodes_into_display_columns(&fixed_column_order, &axis_nodes);
        let obj =
            total_crossing_objective_alluvia(alluvia, &fixed_column_order, col_orig, &nodes_in_col);
        if obj + 1e-9 < best_obj {
            best_obj = obj;
            best = SankeyOrderingResult {
                col: make_display_col(&fixed_column_order, col_orig),
                nodes_in_col,
                column_order: fixed_column_order.clone(),
            };
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::{
        build_distance_matrix_from_alluvia, optimize_sankey_alluvial_order,
        total_crossing_objective_alluvia, tsp_cycle, DeterministicRng, SankeyOrderingResult,
    };
    use crate::plot::sankey::SankeyAlluvium;
    use std::collections::HashMap;

    fn build_fixture(
        rows: &[(&str, &str, &str)],
    ) -> (Vec<usize>, Vec<Vec<usize>>, Vec<SankeyAlluvium>) {
        let n_axes = 3usize;
        let axis_names = ["tissue", "cluster", "sex"];
        let mut axis_levels = vec![Vec::<String>::new(); n_axes];
        let mut combo_weights = HashMap::<(String, String, String), f64>::new();

        for &(a, b, c) in rows {
            for (axis, label) in [a, b, c].into_iter().enumerate() {
                axis_levels[axis].push(label.to_string());
            }
            *combo_weights
                .entry((a.to_string(), b.to_string(), c.to_string()))
                .or_insert(0.0) += 1.0;
        }

        for levels in &mut axis_levels {
            levels.sort();
            levels.dedup();
        }

        let mut node_ids = HashMap::<(usize, String), usize>::new();
        let mut ordered_nodes = Vec::new();
        for axis in 0..n_axes {
            for label in &axis_levels[axis] {
                ordered_nodes.push((
                    axis,
                    format!("{}~~{}", axis_names[axis], label),
                    label.clone(),
                ));
            }
        }
        ordered_nodes.sort_by(|a, b| a.1.cmp(&b.1));

        let mut col_orig = Vec::with_capacity(ordered_nodes.len());
        let mut nodes_in_col = vec![Vec::new(); n_axes];
        for (idx, (axis, _, label)) in ordered_nodes.into_iter().enumerate() {
            node_ids.insert((axis, label), idx);
            col_orig.push(axis);
            nodes_in_col[axis].push(idx);
        }

        let mut combos: Vec<_> = combo_weights.into_iter().collect();
        combos.sort_by(|a, b| a.0.cmp(&b.0));

        let mut alluvia = Vec::new();
        for ((a, b, c), value) in combos {
            let nodes = vec![node_ids[&(0, a)], node_ids[&(1, b)], node_ids[&(2, c)]];
            alluvia.push(SankeyAlluvium { nodes, value });
        }

        (col_orig, nodes_in_col, alluvia)
    }

    fn build_weighted_fixture3(
        rows: &[([&str; 3], f64)],
    ) -> (
        Vec<usize>,
        Vec<Vec<usize>>,
        Vec<SankeyAlluvium>,
        Vec<Vec<String>>,
    ) {
        let n_axes = 3usize;
        let mut axis_levels = vec![Vec::<String>::new(); n_axes];

        for (vals, _) in rows {
            for (axis, label) in vals.iter().enumerate() {
                axis_levels[axis].push((*label).to_string());
            }
        }

        for levels in &mut axis_levels {
            levels.sort();
            levels.dedup();
        }

        let mut node_ids = HashMap::<(usize, String), usize>::new();
        let mut col_orig = Vec::new();
        let mut nodes_in_col = vec![Vec::new(); n_axes];
        let mut labels_by_axis = vec![Vec::new(); n_axes];
        for axis in 0..n_axes {
            for label in &axis_levels[axis] {
                let idx = col_orig.len();
                node_ids.insert((axis, label.clone()), idx);
                col_orig.push(axis);
                nodes_in_col[axis].push(idx);
                labels_by_axis[axis].push(label.clone());
            }
        }

        let mut alluvia = Vec::with_capacity(rows.len());
        for (vals, value) in rows {
            let nodes = vals
                .iter()
                .enumerate()
                .map(|(axis, label)| node_ids[&(axis, (*label).to_string())])
                .collect();
            alluvia.push(SankeyAlluvium {
                nodes,
                value: *value,
            });
        }

        (col_orig, nodes_in_col, alluvia, labels_by_axis)
    }

    fn node_sort_keys(axis_names: &[&str], labels_by_axis: &[Vec<String>]) -> Vec<String> {
        let mut keys = Vec::new();
        for (axis, labels) in labels_by_axis.iter().enumerate() {
            for label in labels {
                keys.push(format!("{}~~{}", axis_names[axis], label));
            }
        }
        keys
    }

    fn canonical_three_layer_rows() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("BRAIN", "1", "male"),
            ("BRAIN", "1", "female"),
            ("BRAIN", "2", "male"),
            ("STOMACH", "1", "female"),
            ("STOMACH", "2", "male"),
            ("STOMACH", "2", "female"),
            ("STOMACH", "2", "female"),
            ("STOMACH", "2", "male"),
            ("STOMACH", "2", "female"),
            ("HEART", "1", "male"),
            ("HEART", "3", "female"),
            ("HEART", "3", "male"),
            ("HEART", "3", "female"),
            ("HEART", "3", "male"),
            ("HEART", "3", "female"),
            ("HEART", "3", "male"),
            ("T CELL", "4", "female"),
            ("T CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
            ("B CELL", "4", "male"),
        ]
    }

    fn identical_layer_rows() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("BRAIN", "BRAIN", "male"),
            ("BRAIN", "BRAIN", "female"),
            ("BRAIN", "BRAIN", "male"),
            ("STOMACH", "STOMACH", "female"),
            ("STOMACH", "STOMACH", "male"),
            ("STOMACH", "STOMACH", "female"),
            ("STOMACH", "STOMACH", "female"),
            ("STOMACH", "STOMACH", "male"),
            ("STOMACH", "STOMACH", "female"),
            ("HEART", "HEART", "male"),
            ("HEART", "HEART", "female"),
            ("HEART", "HEART", "male"),
            ("HEART", "HEART", "female"),
            ("HEART", "HEART", "male"),
            ("HEART", "HEART", "female"),
            ("HEART", "HEART", "male"),
            ("T CELL", "T CELL", "female"),
            ("T CELL", "T CELL", "male"),
            ("B CELL", "B CELL", "female"),
            ("B CELL", "B CELL", "male"),
            ("B CELL", "B CELL", "female"),
            ("B CELL", "B CELL", "male"),
            ("B CELL", "B CELL", "female"),
            ("B CELL", "B CELL", "male"),
            ("B CELL", "B CELL", "female"),
            ("B CELL", "B CELL", "female"),
            ("B CELL", "B CELL", "male"),
        ]
    }

    fn four_axis_rows() -> Vec<([&'static str; 4], f64)> {
        vec![
            (["A", "K1", "M1", "Z"], 8.0),
            (["A", "K2", "M2", "Y"], 7.0),
            (["B", "K1", "M2", "Z"], 6.0),
            (["B", "K2", "M1", "Y"], 6.0),
            (["C", "K3", "M3", "X"], 9.0),
            (["C", "K2", "M1", "X"], 4.0),
            (["A", "K3", "M3", "Y"], 3.0),
            (["B", "K3", "M2", "X"], 5.0),
            (["C", "K1", "M1", "Z"], 2.0),
        ]
    }

    fn canonical_weighted_rows() -> Vec<([&'static str; 3], f64)> {
        vec![
            (["B CELL", "4", "male"], 9.0),
            (["BRAIN", "1", "female"], 1.0),
            (["BRAIN", "1", "male"], 1.0),
            (["BRAIN", "2", "male"], 1.0),
            (["HEART", "1", "male"], 1.0),
            (["HEART", "3", "female"], 3.0),
            (["HEART", "3", "male"], 3.0),
            (["STOMACH", "1", "female"], 1.0),
            (["STOMACH", "2", "female"], 3.0),
            (["STOMACH", "2", "male"], 2.0),
            (["T CELL", "4", "female"], 1.0),
            (["T CELL", "4", "male"], 1.0),
        ]
    }

    fn build_fixture4(
        rows: &[([&str; 4], f64)],
    ) -> (Vec<usize>, Vec<Vec<usize>>, Vec<SankeyAlluvium>) {
        let n_axes = 4usize;
        let axis_names = ["axis1", "axis2", "axis3", "axis4"];
        let mut axis_levels = vec![Vec::<String>::new(); n_axes];
        let mut combo_weights = HashMap::<(String, String, String, String), f64>::new();

        for (vals, weight) in rows {
            for (axis, label) in vals.iter().enumerate() {
                axis_levels[axis].push((*label).to_string());
            }
            *combo_weights
                .entry((
                    vals[0].to_string(),
                    vals[1].to_string(),
                    vals[2].to_string(),
                    vals[3].to_string(),
                ))
                .or_insert(0.0) += *weight;
        }

        for levels in &mut axis_levels {
            levels.sort();
            levels.dedup();
        }

        let mut node_ids = HashMap::<(usize, String), usize>::new();
        let mut ordered_nodes = Vec::new();
        for axis in 0..n_axes {
            for label in &axis_levels[axis] {
                ordered_nodes.push((
                    axis,
                    format!("{}~~{}", axis_names[axis], label),
                    label.clone(),
                ));
            }
        }
        ordered_nodes.sort_by(|a, b| a.1.cmp(&b.1));

        let mut col_orig = Vec::with_capacity(ordered_nodes.len());
        let mut nodes_in_col = vec![Vec::new(); n_axes];
        for (idx, (axis, _, label)) in ordered_nodes.into_iter().enumerate() {
            node_ids.insert((axis, label), idx);
            col_orig.push(axis);
            nodes_in_col[axis].push(idx);
        }

        let mut combos: Vec<_> = combo_weights.into_iter().collect();
        combos.sort_by(|a, b| a.0.cmp(&b.0));

        let mut alluvia = Vec::new();
        for ((a, b, c, d), value) in combos {
            let nodes = vec![
                node_ids[&(0, a)],
                node_ids[&(1, b)],
                node_ids[&(2, c)],
                node_ids[&(3, d)],
            ];
            alluvia.push(SankeyAlluvium { nodes, value });
        }

        (col_orig, nodes_in_col, alluvia)
    }

    fn build_weighted_fixture4(
        rows: &[([&str; 4], f64)],
    ) -> (
        Vec<usize>,
        Vec<Vec<usize>>,
        Vec<SankeyAlluvium>,
        Vec<Vec<String>>,
    ) {
        let n_axes = 4usize;
        let mut axis_levels = vec![Vec::<String>::new(); n_axes];

        for (vals, _) in rows {
            for (axis, label) in vals.iter().enumerate() {
                axis_levels[axis].push((*label).to_string());
            }
        }

        for levels in &mut axis_levels {
            levels.sort();
            levels.dedup();
        }

        let mut node_ids = HashMap::<(usize, String), usize>::new();
        let mut col_orig = Vec::new();
        let mut nodes_in_col = vec![Vec::new(); n_axes];
        let mut labels_by_axis = vec![Vec::new(); n_axes];
        for axis in 0..n_axes {
            for label in &axis_levels[axis] {
                let idx = col_orig.len();
                node_ids.insert((axis, label.clone()), idx);
                col_orig.push(axis);
                nodes_in_col[axis].push(idx);
                labels_by_axis[axis].push(label.clone());
            }
        }

        let mut alluvia = Vec::with_capacity(rows.len());
        for (vals, value) in rows {
            let nodes = vals
                .iter()
                .enumerate()
                .map(|(axis, label)| node_ids[&(axis, (*label).to_string())])
                .collect();
            alluvia.push(SankeyAlluvium {
                nodes,
                value: *value,
            });
        }

        (col_orig, nodes_in_col, alluvia, labels_by_axis)
    }

    fn ordered_label_columns<'a>(
        ordered: &SankeyOrderingResult,
        col_orig: &[usize],
        labels_by_axis: &'a [Vec<String>],
    ) -> Vec<Vec<&'a str>> {
        let mut label_for_node = vec![""; col_orig.len()];
        let mut axis_offsets = vec![0usize; labels_by_axis.len()];
        for axis in 0..labels_by_axis.len() {
            axis_offsets[axis] = col_orig.iter().take_while(|&&col| col != axis).count();
        }
        for (axis, labels) in labels_by_axis.iter().enumerate() {
            for (i, label) in labels.iter().enumerate() {
                label_for_node[axis_offsets[axis] + i] = label.as_str();
            }
        }
        ordered
            .nodes_in_col
            .iter()
            .map(|nodes| nodes.iter().map(|&node| label_for_node[node]).collect())
            .collect()
    }

    fn identity_ordering(col_orig: &[usize], nodes_in_col: &[Vec<usize>]) -> SankeyOrderingResult {
        SankeyOrderingResult {
            col: col_orig.to_vec(),
            nodes_in_col: nodes_in_col.to_vec(),
            column_order: (0..nodes_in_col.len()).collect(),
        }
    }

    #[test]
    fn wompwomp_fixture_three_layer_unsorted_objective_matches() {
        let (col_orig, nodes_in_col, alluvia) = build_fixture(&canonical_three_layer_rows());
        let identity = identity_ordering(&col_orig, &nodes_in_col);
        let objective = total_crossing_objective_alluvia(
            &alluvia,
            &identity.column_order,
            &col_orig,
            &identity.nodes_in_col,
        );
        assert_eq!(objective, 225.0);
    }

    #[test]
    fn wompwomp_fixture_three_layer_tsp_objective_matches() {
        let (col_orig, nodes_in_col, alluvia) = build_fixture(&canonical_three_layer_rows());
        let ordered = optimize_sankey_alluvial_order(
            &col_orig,
            &nodes_in_col,
            &alluvia,
            &[],
            42,
            None,
            false,
        );
        let objective = total_crossing_objective_alluvia(
            &alluvia,
            &ordered.column_order,
            &col_orig,
            &ordered.nodes_in_col,
        );
        let identity_column_order: Vec<usize> = (0..nodes_in_col.len()).collect();
        let identity_column_obj = total_crossing_objective_alluvia(
            &alluvia,
            &identity_column_order,
            &col_orig,
            &ordered.nodes_in_col,
        );
        assert_eq!(
            objective, 57.0,
            "column_order={:?} nodes_in_col={:?} identity_column_obj={}",
            ordered.column_order, ordered.nodes_in_col, identity_column_obj
        );
    }

    #[test]
    fn wompwomp_fixture_identical_layer_unsorted_objective_matches() {
        let (col_orig, nodes_in_col, alluvia) = build_fixture(&identical_layer_rows());
        let identity = identity_ordering(&col_orig, &nodes_in_col);
        let objective = total_crossing_objective_alluvia(
            &alluvia,
            &identity.column_order,
            &col_orig,
            &identity.nodes_in_col,
        );
        assert_eq!(objective, 74.0);
    }

    #[test]
    fn wompwomp_fixture_identical_layer_tsp_objective_matches() {
        let (col_orig, nodes_in_col, alluvia) = build_fixture(&identical_layer_rows());
        let ordered = optimize_sankey_alluvial_order(
            &col_orig,
            &nodes_in_col,
            &alluvia,
            &[],
            42,
            None,
            false,
        );
        let objective = total_crossing_objective_alluvia(
            &alluvia,
            &ordered.column_order,
            &col_orig,
            &ordered.nodes_in_col,
        );
        let identity_column_order: Vec<usize> = (0..nodes_in_col.len()).collect();
        let identity_column_obj = total_crossing_objective_alluvia(
            &alluvia,
            &identity_column_order,
            &col_orig,
            &ordered.nodes_in_col,
        );
        assert_eq!(
            objective, 56.0,
            "column_order={:?} nodes_in_col={:?} identity_column_obj={}",
            ordered.column_order, ordered.nodes_in_col, identity_column_obj
        );
    }

    #[test]
    fn r_sample_permutation_seed_42_matches() {
        let mut rng = DeterministicRng::new(42);
        assert_eq!(
            rng.sample_permutation(11),
            vec![0, 4, 10, 8, 1, 3, 6, 9, 7, 5, 2]
        );
        let mut rng = DeterministicRng::new(42);
        assert_eq!(
            rng.sample_permutation(12),
            vec![0, 4, 11, 8, 1, 3, 7, 5, 9, 2, 10, 6]
        );
        assert_eq!(rng.sample_permutation(4), vec![3, 0, 2, 1]);
    }

    #[test]
    fn r_tsp_three_layer_tour_matches() {
        let (col_orig, _, alluvia) = build_fixture(&canonical_three_layer_rows());
        let mat = build_distance_matrix_from_alluvia(&col_orig, &alluvia);
        let mut rng = DeterministicRng::new(42);
        let cycle = tsp_cycle(&mat, &mut rng);
        assert_eq!(cycle, vec![0, 7, 10, 3, 6, 5, 1, 9, 4, 2, 8]);
    }

    #[test]
    fn r_tsp_identical_layer_tour_matches() {
        let (col_orig, _, alluvia) = build_fixture(&identical_layer_rows());
        let mat = build_distance_matrix_from_alluvia(&col_orig, &alluvia);
        let mut rng = DeterministicRng::new(42);
        let cycle = tsp_cycle(&mat, &mut rng);
        assert_eq!(cycle, vec![0, 7, 5, 10, 3, 6, 2, 9, 1, 8, 11, 4]);
    }

    #[test]
    fn r_tsp_larger_matrix_8_matches() {
        let mat = vec![
            vec![0.0, 24.0, 33.0, 5.0, 14.0, 23.0, 13.0, 22.0],
            vec![24.0, 0.0, 32.0, 3.0, 14.0, 25.0, 14.0, 25.0],
            vec![33.0, 32.0, 0.0, 19.0, 14.0, 24.0, 15.0, 10.0],
            vec![5.0, 3.0, 19.0, 0.0, 11.0, 23.0, 16.0, 10.0],
            vec![14.0, 14.0, 14.0, 11.0, 0.0, 25.0, 17.0, 13.0],
            vec![23.0, 25.0, 24.0, 23.0, 25.0, 0.0, 18.0, 16.0],
            vec![13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 0.0, 16.0],
            vec![22.0, 25.0, 10.0, 10.0, 13.0, 16.0, 16.0, 0.0],
        ];
        let mut rng = DeterministicRng::new(42);
        let cycle = tsp_cycle(&mat, &mut rng);
        assert_eq!(cycle, vec![0, 6, 5, 7, 2, 4, 1, 3]);
    }

    #[test]
    fn r_tsp_larger_matrix_6_matches() {
        let mat = vec![
            vec![0.0, 24.0, 33.0, 5.0, 14.0, 23.0],
            vec![24.0, 0.0, 32.0, 3.0, 14.0, 25.0],
            vec![33.0, 32.0, 0.0, 19.0, 14.0, 24.0],
            vec![5.0, 3.0, 19.0, 0.0, 11.0, 23.0],
            vec![14.0, 14.0, 14.0, 11.0, 0.0, 25.0],
            vec![23.0, 25.0, 24.0, 23.0, 25.0, 0.0],
        ];
        let mut rng = DeterministicRng::new(42);
        let cycle = tsp_cycle(&mat, &mut rng);
        assert_eq!(cycle, vec![0, 5, 2, 4, 1, 3]);
    }

    #[test]
    fn larger_four_axis_alluvium_improves_crossing_objective() {
        let (col_orig, nodes_in_col, alluvia) = build_fixture4(&four_axis_rows());
        let identity = identity_ordering(&col_orig, &nodes_in_col);
        let identity_obj = total_crossing_objective_alluvia(
            &alluvia,
            &identity.column_order,
            &col_orig,
            &identity.nodes_in_col,
        );
        let ordered = optimize_sankey_alluvial_order(
            &col_orig,
            &nodes_in_col,
            &alluvia,
            &[],
            42,
            None,
            false,
        );
        let ordered_obj = total_crossing_objective_alluvia(
            &alluvia,
            &ordered.column_order,
            &col_orig,
            &ordered.nodes_in_col,
        );
        assert!(
            ordered_obj < identity_obj,
            "optimized alluvium ordering should improve the weighted crossing objective"
        );
        assert_eq!(ordered.column_order.len(), 4);
        assert_eq!(ordered.nodes_in_col.len(), 4);
    }

    #[test]
    fn wompwomp_package_canonical_output_matches() {
        let (col_orig, nodes_in_col, alluvia, labels_by_axis) =
            build_weighted_fixture3(&canonical_weighted_rows());
        let axis_labels = ["tissue", "cluster", "sex"];
        let node_sort_keys = node_sort_keys(&axis_labels, &labels_by_axis);
        let ordered = optimize_sankey_alluvial_order(
            &col_orig,
            &nodes_in_col,
            &alluvia,
            &[],
            42,
            Some(&node_sort_keys),
            false,
        );
        let objective = total_crossing_objective_alluvia(
            &alluvia,
            &ordered.column_order,
            &col_orig,
            &ordered.nodes_in_col,
        );
        let column_names: Vec<&str> = ordered
            .column_order
            .iter()
            .map(|&i| axis_labels[i])
            .collect();
        let node_labels = ordered_label_columns(&ordered, &col_orig, &labels_by_axis);
        assert_eq!(
            column_names,
            vec!["tissue", "cluster", "sex"],
            "column order mismatch; labels={:?}",
            node_labels
        );
        assert_eq!(
            node_labels[0],
            vec!["STOMACH", "HEART", "BRAIN", "T CELL", "B CELL"]
        );
        assert_eq!(node_labels[1], vec!["2", "3", "1", "4"]);
        assert_eq!(node_labels[2], vec!["female", "male"]);
        assert_eq!(objective, 57.0);
    }

    #[test]
    fn wompwomp_package_four_axis_output_matches() {
        let (col_orig, nodes_in_col, alluvia, labels_by_axis) =
            build_weighted_fixture4(&four_axis_rows());
        let axis_labels = ["axis1", "axis2", "axis3", "axis4"];
        let node_sort_keys = node_sort_keys(&axis_labels, &labels_by_axis);
        let ordered = optimize_sankey_alluvial_order(
            &col_orig,
            &nodes_in_col,
            &alluvia,
            &[],
            42,
            Some(&node_sort_keys),
            false,
        );
        let objective = total_crossing_objective_alluvia(
            &alluvia,
            &ordered.column_order,
            &col_orig,
            &ordered.nodes_in_col,
        );
        let column_names: Vec<&str> = ordered
            .column_order
            .iter()
            .map(|&i| axis_labels[i])
            .collect();
        let node_labels = ordered_label_columns(&ordered, &col_orig, &labels_by_axis);
        assert_eq!(
            column_names,
            vec!["axis2", "axis4", "axis3", "axis1"],
            "column order mismatch; labels={:?}",
            node_labels
        );
        assert_eq!(node_labels[0], vec!["K3", "K2", "K1"]);
        assert_eq!(node_labels[1], vec!["X", "Y", "Z"]);
        assert_eq!(node_labels[2], vec!["M3", "M1", "M2"]);
        assert_eq!(node_labels[3], vec!["C", "A", "B"]);
        assert_eq!(objective, 464.0);
    }
}
