"""
Generate all example TSV data files for the kuva plotting library.
All files written to the same directory as this script.
"""

import numpy as np
import os
from pathlib import Path

np.random.seed(42)
OUT = Path(__file__).parent


def write_tsv(filename, header, rows):
    path = OUT / filename
    with open(path, "w") as f:
        f.write("\t".join(header) + "\n")
        for row in rows:
            f.write("\t".join(str(v) for v in row) + "\n")
    return len(rows)


counts = {}

# ---------------------------------------------------------------------------
# scatter.tsv — three bivariate clusters for scatter / color-by demos
# ---------------------------------------------------------------------------
rows_sc = []
for group, (cx, cy, sx, sy, rho) in [
    ("Group_A", (3.0, 5.5, 1.2, 0.9,  0.60)),
    ("Group_B", (7.5, 7.0, 1.0, 1.1,  0.45)),
    ("Group_C", (5.0, 2.0, 0.8, 0.75, -0.30)),
]:
    cov = np.array([[sx**2, rho*sx*sy], [rho*sx*sy, sy**2]])
    pts = np.random.multivariate_normal([cx, cy], cov, 80)
    for x, y in pts:
        rows_sc.append((round(float(x), 3), round(float(y), 3), group))
np.random.shuffle(rows_sc)
counts["scatter.tsv"] = write_tsv("scatter.tsv", ["x", "y", "group"], rows_sc)

# ---------------------------------------------------------------------------
# volcano.tsv — crafted for a clear V-shape (gene name / log2fc / pvalue)
# ---------------------------------------------------------------------------
_vc_names = [f"Gene_{i:03d}" for i in range(1, 201)]
_vc_fc  = np.zeros(200)
_vc_pv  = np.zeros(200)

# Null genes — dense grey cluster at center-bottom
_vc_fc[:70]   = np.random.normal(0, 0.32, 70)
_vc_pv[:70]   = np.random.uniform(0.15, 0.99, 70)

# Up-regulated — top-right cluster
_vc_fc[70:110]  = np.abs(np.random.normal(2.8, 0.65, 40))
_vc_pv[70:110]  = np.random.uniform(1e-9, 0.005, 40)

# Down-regulated — top-left cluster
_vc_fc[110:150] = -np.abs(np.random.normal(2.8, 0.65, 40))
_vc_pv[110:150] = np.random.uniform(1e-9, 0.005, 40)

# Borderline up — just crossing both thresholds (lower arms of V)
_vc_fc[150:162] = np.abs(np.random.normal(1.15, 0.12, 12))
_vc_pv[150:162] = np.random.uniform(0.008, 0.048, 12)

# Borderline down
_vc_fc[162:174] = -np.abs(np.random.normal(1.15, 0.12, 12))
_vc_pv[162:174] = np.random.uniform(0.008, 0.048, 12)

# High FC, not significant — outer arms of V
_vc_fc[174:187] = np.abs(np.random.normal(1.8, 0.45, 13))
_vc_pv[174:187] = np.random.uniform(0.07, 0.55, 13)

_vc_fc[187:200] = -np.abs(np.random.normal(1.8, 0.45, 13))
_vc_pv[187:200] = np.random.uniform(0.07, 0.55, 13)

_vc_idx = np.random.permutation(200)
_vc_names = [_vc_names[i] for i in _vc_idx]
_vc_fc    = _vc_fc[_vc_idx]
_vc_pv    = _vc_pv[_vc_idx]
rows_vc = [(_vc_names[i], round(float(_vc_fc[i]), 4), f"{_vc_pv[i]:.6e}") for i in range(200)]
counts["volcano.tsv"] = write_tsv("volcano.tsv", ["gene", "log2fc", "pvalue"], rows_vc)

# ---------------------------------------------------------------------------
# samples.tsv
# ---------------------------------------------------------------------------
rows = []
# Control: normal(5.0, 1.2)
for v in np.random.normal(5.0, 1.2, 120):
    rows.append(("Control", round(float(v), 3)))
# Drug_A: normal(7.2, 1.5)
for v in np.random.normal(7.2, 1.5, 120):
    rows.append(("Drug_A", round(float(v), 3)))
# Drug_B: bimodal
for v in np.concatenate([np.random.normal(3.8, 0.7, 70), np.random.normal(8.2, 0.9, 50)]):
    rows.append(("Drug_B", round(float(v), 3)))
# Drug_C: clipped normal
for v in np.clip(np.random.normal(3.5, 2.0, 120), 0, 15):
    rows.append(("Drug_C", round(float(v), 3)))
# Drug_D: right-skewed
for v in np.random.exponential(1.5, 120) + 4.5:
    rows.append(("Drug_D", round(float(v), 3)))
counts["samples.tsv"] = write_tsv("samples.tsv", ["group", "expression"], rows)

# ---------------------------------------------------------------------------
# measurements.tsv — three sigmoid growth curves, clearly separated, no crossings
# ---------------------------------------------------------------------------
_t = np.linspace(0, 20, 50)
_sig = lambda t, k, m: 1.0 / (1.0 + np.exp(-k * (t - m)))
rows = []
for group, base, scale, k, m, noise_sd in [
    ("Condition_A", 1.5, 2.0, 0.40, 10.0, 0.15),   # low band, rises from ~1.5 to ~3.5
    ("Condition_B", 4.2, 2.5, 0.30, 10.0, 0.18),   # mid band, rises from ~4.2 to ~6.7
    ("Condition_C", 7.0, 2.0, 0.35, 10.0, 0.15),   # high band, rises from ~7.0 to ~9.0
]:
    v = base + scale * _sig(_t, k, m) + np.random.normal(0, noise_sd, 50)
    for ti, vi in zip(_t, v):
        rows.append((group, round(float(ti), 2), round(float(vi), 3)))
counts["measurements.tsv"] = write_tsv(
    "measurements.tsv",
    ["group", "time", "value"],
    rows,
)

# ---------------------------------------------------------------------------
# gene_stats.tsv
# ---------------------------------------------------------------------------
# Chromosome weights (proportional to size)
chrom_names = [f"chr{i}" for i in range(1, 23)] + ["chrX", "chrY"]
# Approximate lengths in Mb
chrom_lengths_mb = [
    248, 242, 198, 190, 181, 171, 159, 145, 138, 133,
    135, 133, 114, 107, 102, 90, 83, 80, 58, 63, 47, 51,
    155, 57
]
total_len = sum(chrom_lengths_mb)
chrom_weights = [l / total_len for l in chrom_lengths_mb]

n_genes = 8000
n_de = 400  # differentially expressed
n_null = n_genes - n_de

gene_names = [f"Gene_{i:04d}" for i in range(1, n_genes + 1)]
chroms = np.random.choice(chrom_names, size=n_genes, p=chrom_weights)
chrom_len_map = {c: l * 1_000_000 for c, l in zip(chrom_names, chrom_lengths_mb)}
positions = np.array([np.random.randint(1, chrom_len_map[c]) for c in chroms])
basemeans = np.round(np.random.lognormal(mean=5, sigma=2, size=n_genes), 1)

# log2fc and pvalue
log2fc = np.zeros(n_genes)
pvalue = np.zeros(n_genes)

# null genes — uniform p-values bounded away from 0 so none accidentally
# cross the significance threshold and fill the V-notch
log2fc[:n_null] = np.random.normal(0, 0.3, n_null)
pvalue[:n_null] = np.random.uniform(0.05, 1.0, n_null)

# DE genes
signs = np.random.choice([-1, 1], size=n_de)
log2fc[n_null:] = signs * np.random.normal(3.5, 0.8, n_de)
pvalue[n_null:] = np.random.uniform(1e-8, 0.001, n_de)

# Shuffle
idx = np.random.permutation(n_genes)
gene_names = [gene_names[i] for i in idx]
chroms = chroms[idx]
positions = positions[idx]
basemeans = basemeans[idx]
log2fc = log2fc[idx]
pvalue = pvalue[idx]

# BH-adjusted p-values (approximation)
ranks = np.argsort(pvalue) + 1  # rank 1 = smallest
padj = np.minimum(pvalue * n_genes / ranks, 1.0)
# Re-order padj to match gene order
padj_ordered = np.empty(n_genes)
for rank_pos, gene_pos in enumerate(np.argsort(pvalue)):
    padj_ordered[gene_pos] = padj[rank_pos]
padj = padj_ordered

rows = []
for i in range(n_genes):
    rows.append((
        gene_names[i],
        chroms[i],
        int(positions[i]),
        basemeans[i],
        round(float(log2fc[i]), 4),
        f"{pvalue[i]:.6e}",
        f"{padj[i]:.6e}",
    ))
counts["gene_stats.tsv"] = write_tsv(
    "gene_stats.tsv",
    ["gene", "chr", "pos", "basemean", "log2fc", "pvalue", "padj"],
    rows,
)

# ---------------------------------------------------------------------------
# bar.tsv
# ---------------------------------------------------------------------------
go_terms = [
    "cell cycle regulation",
    "DNA repair",
    "immune response",
    "apoptosis",
    "protein folding",
    "mRNA splicing",
    "oxidative phosphorylation",
    "chromatin remodeling",
    "signal transduction",
    "vesicle-mediated transport",
    "cytoskeleton organization",
    "transcription regulation",
    "protein ubiquitination",
    "lipid metabolism",
    "RNA processing",
    "cell adhesion",
    "autophagy",
    "mitochondrial organization",
    "ion transport",
    "protein phosphorylation",
]
# Decreasing counts from ~320 down to ~15
gene_counts = [320, 285, 257, 231, 208, 187, 168, 152, 137, 123,
               111, 100, 90, 81, 68, 55, 42, 33, 22, 15]
rows = list(zip(go_terms, gene_counts))
counts["bar.tsv"] = write_tsv("bar.tsv", ["category", "count"], rows)

# ---------------------------------------------------------------------------
# histogram.tsv
# ---------------------------------------------------------------------------
vals = np.concatenate([
    np.random.normal(42, 8, 550),
    np.random.normal(68, 6, 350),
])
rows = [(round(float(v), 2),) for v in vals]
counts["histogram.tsv"] = write_tsv("histogram.tsv", ["value"], rows)

# ---------------------------------------------------------------------------
# pie.tsv
# ---------------------------------------------------------------------------
rows = [
    ("Intron", 37.0),
    ("Intergenic", 28.0),
    ("Repeat", 15.0),
    ("Exon", 9.0),
    ("3'UTR", 4.0),
    ("Promoter", 3.0),
    ("5'UTR", 2.0),
    ("Other", 2.0),
]
counts["pie.tsv"] = write_tsv("pie.tsv", ["feature", "percentage"], rows)

# ---------------------------------------------------------------------------
# heatmap.tsv
# ---------------------------------------------------------------------------
gene_list = [
    "TP53", "BRCA1", "EGFR", "MYC", "CDK2", "RB1", "PTEN", "AKT1", "KRAS", "BRAF",
    "MDM2", "CCND1", "E2F1", "PCNA", "GAPDH", "ACTB", "STAT3", "JAK2", "PIK3CA", "MTOR",
    "ATM", "CHEK2", "RAD51", "BRCA2", "BCL2", "MCL1", "CASP3", "CASP9", "BAX", "BID",
]
n_genes_hm = 30
n_samples = 12
# Group structure: samples 0-2 Control, 3-5 TreatA, 6-8 TreatB, 9-11 TreatC
sample_names = [f"Sample_{i:02d}" for i in range(1, 13)]
# 10 "interesting" genes (indices 0-9) have group-dependent means
interesting = set(range(10))
group_means = {
    "Control":  [0.0] * 10,
    "TreatA":   [2.0] * 10,
    "TreatB":   [-1.5] * 10,
    "TreatC":   [1.0] * 10,
}
group_membership = (
    ["Control"] * 3 + ["TreatA"] * 3 + ["TreatB"] * 3 + ["TreatC"] * 3
)
rows = []
for gi, gene in enumerate(gene_list):
    row = [gene]
    for si, group in enumerate(group_membership):
        if gi in interesting:
            mean = group_means[group][gi]
        else:
            mean = 0.0
        val = round(float(np.random.normal(mean, 1.0)), 2)
        row.append(val)
    rows.append(row)
counts["heatmap.tsv"] = write_tsv(
    "heatmap.tsv",
    ["gene"] + sample_names,
    rows,
)

# ---------------------------------------------------------------------------
# waterfall.tsv
# ---------------------------------------------------------------------------
waterfall_data = [
    ("Glycolysis",          3.2),
    ("DNA repair",         -2.1),
    ("Cell proliferation",  2.8),
    ("Apoptosis",          -1.8),
    ("mTOR signaling",      2.5),
    ("Autophagy",          -2.3),
    ("Angiogenesis",        1.9),
    ("T cell activity",    -2.7),
    ("Ribosome biogenesis", 1.6),
    ("Mitochondrial resp.", -1.4),
    ("Protein synthesis",   1.4),
    ("Oxidative stress",   -2.0),
    ("Cell cycle",          2.2),
    ("Ion transport",      -1.1),
    ("Chromatin remodel.",  1.3),
    ("Interferon signaling",-2.8),
    ("Vesicle transport",   1.0),
    ("Complement cascade", -3.1),
    ("Lipid biosynthesis",  1.8),
    ("Antiviral defense",  -2.4),
]
counts["waterfall.tsv"] = write_tsv(
    "waterfall.tsv",
    ["process", "log2fc"],
    waterfall_data,
)

# ---------------------------------------------------------------------------
# stacked_area.tsv — raw read counts (not pre-normalized) so basic vs
# normalized views look meaningfully different
# ---------------------------------------------------------------------------
species_list = [
    "Firmicutes", "Bacteroidetes", "Proteobacteria",
    "Actinobacteria", "Fusobacteria", "Verrucomicrobia",
]
weeks = list(range(1, 53))
rows = []
t = np.linspace(0, 2 * np.pi, 52)
for wi, week in enumerate(weeks):
    # Raw read counts per species — totals vary per week (800–1 200)
    firm = int(max(1, 350 + 80 * np.sin(t[wi])   + np.random.normal(0, 20)))
    bact = int(max(1, 250 - 60 * np.sin(t[wi])   + np.random.normal(0, 20)))
    prot = int(max(1, 150 + 30 * np.cos(t[wi]*2) + np.random.normal(0, 15)))
    acti = int(max(1, 120 + 20 * np.sin(t[wi]*1.5) + np.random.normal(0, 10)))
    fuso = int(max(1,  80                          + np.random.normal(0, 10)))
    verr = int(max(1,  50                          + np.random.normal(0,  8)))
    for sp, ab in zip(species_list, [firm, bact, prot, acti, fuso, verr]):
        rows.append((week, sp, ab))
counts["stacked_area.tsv"] = write_tsv(
    "stacked_area.tsv",
    ["week", "species", "abundance"],
    rows,
)

# ---------------------------------------------------------------------------
# candlestick.tsv
# ---------------------------------------------------------------------------
from datetime import date, timedelta

def next_weekday(d, delta=1):
    d += timedelta(days=delta)
    while d.weekday() >= 5:
        d += timedelta(days=1)
    return d

price = 142.50
current_date = date(2023, 1, 2)
rows = []
for i in range(200):
    daily_return = np.random.normal(0.0003, 0.018)
    close = round(price * (1 + daily_return), 2)
    open_ = round(price * (1 + np.random.normal(0, 0.003)), 2)
    high = round(max(open_, close) * (1 + abs(np.random.normal(0, 0.008))), 2)
    low = round(min(open_, close) * (1 - abs(np.random.normal(0, 0.008))), 2)
    volume = int(np.round(np.random.lognormal(15.5, 0.4)))
    rows.append((current_date.strftime("%Y-%m-%d"), open_, high, low, close, volume))
    price = close
    if i < 199:
        current_date = next_weekday(current_date)
counts["candlestick.tsv"] = write_tsv(
    "candlestick.tsv",
    ["date", "open", "high", "low", "close", "volume"],
    rows,
)

# ---------------------------------------------------------------------------
# contour.tsv
# ---------------------------------------------------------------------------
def gauss2d(x, y, cx, cy, sx, sy):
    return np.exp(-0.5 * ((x - cx) ** 2 / sx**2 + (y - cy) ** 2 / sy**2))

x_c = np.random.uniform(0, 10, 600)
y_c = np.random.uniform(1, 10, 600)
density = (
    0.6 * gauss2d(x_c, y_c, 3, 4, 1.5, 1.2)
    + 0.4 * gauss2d(x_c, y_c, 7, 6, 1.0, 1.8)
    + np.random.normal(0, 0.02, 600)
)
density = np.clip(density, 0, None)
rows = [
    (round(float(x), 2), round(float(y), 2), round(float(d), 4))
    for x, y, d in zip(x_c, y_c, density)
]
counts["contour.tsv"] = write_tsv("contour.tsv", ["x", "y", "density"], rows)

# ---------------------------------------------------------------------------
# hist2d.tsv — two bivariate clusters with clear density structure
# ---------------------------------------------------------------------------
pts1 = np.random.multivariate_normal([25, 30], [[40, 30], [30, 40]], 350)
pts2 = np.random.multivariate_normal([70, 75], [[35, 25], [25, 35]], 250)
all_pts = np.vstack([pts1, pts2])
rows = [(round(float(x), 2), round(float(y), 2)) for x, y in all_pts]
counts["hist2d.tsv"] = write_tsv("hist2d.tsv", ["x", "y"], rows)

# ---------------------------------------------------------------------------
# dot.tsv
# ---------------------------------------------------------------------------
pathways = [
    "Glycolysis",
    "TCA cycle",
    "Oxidative phosphorylation",
    "Fatty acid oxidation",
    "Pentose phosphate",
    "Amino acid synthesis",
    "Nucleotide synthesis",
    "One-carbon metabolism",
]
cell_types = [
    "Hepatocyte",
    "Neuron",
    "Cardiomyocyte",
    "Skeletal muscle",
    "Adipocyte",
    "Epithelial",
    "Immune cell",
]
# Base expression matrix (pathway × cell type) — biologically informed
base_expr = np.array([
    # Glyco  TCA   OxPh  FAO   PPP   AASyn NucSyn 1C
    [3.8,   3.5,  3.2,  4.0,  3.5,  3.0,  2.8,  2.5],  # Hepatocyte
    [2.0,   2.5,  4.2,  2.8,  1.8,  2.2,  2.0,  2.0],  # Neuron
    [3.0,   4.0,  4.5,  4.2,  2.5,  2.8,  2.5,  2.2],  # Cardiomyocyte
    [3.5,   3.8,  4.0,  3.8,  2.8,  3.0,  2.8,  2.5],  # Skeletal muscle
    [2.5,   3.0,  3.0,  4.5,  2.0,  2.5,  2.0,  2.2],  # Adipocyte
    [2.8,   2.5,  2.8,  2.5,  2.2,  2.5,  2.5,  2.0],  # Epithelial
    [2.2,   2.0,  2.5,  2.0,  2.5,  2.2,  3.0,  1.8],  # Immune cell
]).T  # now shape (pathway × cell type)

rows = []
for pi, pathway in enumerate(pathways):
    for ci, cell in enumerate(cell_types):
        mean_expr = round(float(np.clip(base_expr[pi, ci] + np.random.normal(0, 0.15), 0.5, 4.5)), 2)
        pct = round(float(np.clip(mean_expr / 4.5 * 90 + np.random.normal(0, 5), 5, 95)), 1)
        rows.append((pathway, cell, mean_expr, pct))
counts["dot.tsv"] = write_tsv(
    "dot.tsv",
    ["pathway", "cell_type", "mean_expr", "pct_expressed"],
    rows,
)

# ---------------------------------------------------------------------------
# upset.tsv
# ---------------------------------------------------------------------------
n_variants = 400
# Marginal probs
p_gwas = 0.30
p_eqtl = 0.45
p_splicing = 0.20
p_methyl = 0.35
p_conservation = 0.55
p_clinvar = 0.15

# Generate with mild correlations using a latent variable approach
z = np.random.normal(0, 1, (n_variants, 6))
# Add correlation via shared latent factor
shared1 = np.random.normal(0, 1, n_variants)  # GWAS + eQTL
shared2 = np.random.normal(0, 1, n_variants)  # Conservation + ClinVar
z[:, 0] += 0.3 * shared1
z[:, 1] += 0.3 * shared1
z[:, 4] += 0.4 * shared2
z[:, 5] += 0.4 * shared2

thresholds = [
    np.percentile(z[:, 0], (1 - p_gwas) * 100),
    np.percentile(z[:, 1], (1 - p_eqtl) * 100),
    np.percentile(z[:, 2], (1 - p_splicing) * 100),
    np.percentile(z[:, 3], (1 - p_methyl) * 100),
    np.percentile(z[:, 4], (1 - p_conservation) * 100),
    np.percentile(z[:, 5], (1 - p_clinvar) * 100),
]
binary = (z > thresholds).astype(int)
rows = [tuple(row) for row in binary]
counts["upset.tsv"] = write_tsv(
    "upset.tsv",
    ["GWAS_hit", "eQTL", "Splicing_QTL", "Methylation_QTL", "Conservation", "ClinVar"],
    rows,
)

# ---------------------------------------------------------------------------
# chord.tsv
# ---------------------------------------------------------------------------
regions = ["Cortex", "Hippocampus", "Amygdala", "Thalamus",
           "Cerebellum", "Striatum", "Brainstem", "Hypothalamus"]
n_r = len(regions)
# Build symmetric matrix
mat = np.zeros((n_r, n_r), dtype=int)
# Define some strong connections
strong = {
    (0, 3): 450,  # Cortex <-> Thalamus
    (0, 1): 320,  # Cortex <-> Hippocampus
    (0, 4): 280,  # Cortex <-> Cerebellum
    (1, 2): 210,  # Hippocampus <-> Amygdala
    (3, 6): 190,  # Thalamus <-> Brainstem
    (4, 5): 175,  # Cerebellum <-> Striatum
    (5, 6): 160,  # Striatum <-> Brainstem
    (2, 7): 145,  # Amygdala <-> Hypothalamus
    (3, 7): 130,  # Thalamus <-> Hypothalamus
    (0, 5): 120,  # Cortex <-> Striatum
}
for (i, j), v in strong.items():
    mat[i, j] = v
    mat[j, i] = v
# Fill remaining with random moderate values
for i in range(n_r):
    for j in range(i + 1, n_r):
        if mat[i, j] == 0:
            v = int(np.random.randint(10, 100))
            mat[i, j] = v
            mat[j, i] = v
# Zero diagonal
np.fill_diagonal(mat, 0)
rows = []
for i, region in enumerate(regions):
    rows.append([region] + list(mat[i]))
counts["chord.tsv"] = write_tsv("chord.tsv", ["region"] + regions, rows)

# ---------------------------------------------------------------------------
# sankey.tsv
# ---------------------------------------------------------------------------
sankey_rows = [
    ("Raw_reads",        "Trimmed",          82),
    ("Raw_reads",        "Discarded",         3),
    ("Trimmed",          "Genome_aligned",   68),
    ("Trimmed",          "rRNA",              8),
    ("Trimmed",          "Unmapped",          6),
    ("Genome_aligned",   "Exonic",           42),
    ("Genome_aligned",   "Intronic",         18),
    ("Genome_aligned",   "Intergenic",        8),
    ("Exonic",           "Protein_coding",   31),
    ("Exonic",           "lncRNA",            7),
    ("Exonic",           "Other_RNA",         4),
    ("Protein_coding",   "High_conf",        24),
    ("Protein_coding",   "Low_conf",          7),
]
counts["sankey.tsv"] = write_tsv("sankey.tsv", ["source", "target", "value"], sankey_rows)

# ---------------------------------------------------------------------------
# phylo.tsv
# ---------------------------------------------------------------------------
# Topology (rough mammalian phylogeny):
# node_1 (root) splits into: primates_clade(node_2) and others(node_3)
# Primates (node_2): hominids(node_4) + other_primates(node_5)
# Hominids (node_4): human_pan(node_6) + gorilla_pongo(node_7)
# node_6: Homo_sapiens, Pan_troglodytes
# node_7: Gorilla_gorilla, Pongo_pygmaeus
# Other_primates (node_5): OW_monkeys(node_8) + NW_monkeys(node_9)
# node_8: Macaca_mulatta, Papio_anubis
# node_9: Callithrix_jacchus
# Others (node_3): rodents_clade(node_10) + laurasiatheria(node_11)
# Rodents (node_10): muridae(node_12) + others(node_13)
# node_12: Mus_musculus, Rattus_norvegicus
# node_13: Cavia_porcellus, Oryctolagus_cuniculus
# Laurasiatheria (node_11): carnivores(node_14) + others(node_15)
# node_14: Canis_lupus, Felis_catus
# node_15: artiodactyls(node_16) + Equus_caballus
# node_16: Sus_scrofa, Bos_taurus
# Outgroups (node_3 splits to node_17)
# Actually let node_1 -> node_2(primates+rodents+laurasiatheria) + node_17(outgroups)
# node_17: birds_reptiles(node_18) + fish_insects(node_19)
# node_18: Gallus_gallus, Xenopus_tropicalis
# node_19: Danio_rerio, Drosophila_melanogaster

phylo_edges = [
    # parent, child, length
    ("node_1", "node_2", 0.05),   # root -> placentals
    ("node_1", "node_17", 0.12),  # root -> outgroups (reduced for balanced phylogram)
    # Placentals
    ("node_2", "node_3", 0.04),   # placentals -> primates+euarchontoglires
    ("node_2", "node_11", 0.06),  # placentals -> laurasiatheria
    # Primates + rodents
    ("node_3", "node_4", 0.03),   # -> primates
    ("node_3", "node_10", 0.05),  # -> rodents
    # Primates
    ("node_4", "node_5", 0.02),   # -> hominoids
    ("node_4", "node_8", 0.04),   # -> old world monkeys + NW
    # Hominoids
    ("node_5", "node_6", 0.01),   # -> human+pan
    ("node_5", "node_7", 0.015),  # -> gorilla+pongo
    ("node_6", "Homo_sapiens", 0.008),
    ("node_6", "Pan_troglodytes", 0.010),
    ("node_7", "Gorilla_gorilla", 0.015),
    ("node_7", "Pongo_pygmaeus", 0.030),
    # OW monkeys + NW
    ("node_8", "node_9", 0.03),
    ("node_8", "Callithrix_jacchus", 0.06),
    ("node_9", "Macaca_mulatta", 0.02),
    ("node_9", "Papio_anubis", 0.025),
    # Rodents
    ("node_10", "node_12", 0.05),
    ("node_10", "node_13", 0.06),
    ("node_12", "Mus_musculus", 0.04),
    ("node_12", "Rattus_norvegicus", 0.045),
    ("node_13", "Cavia_porcellus", 0.08),
    ("node_13", "Oryctolagus_cuniculus", 0.07),
    # Laurasiatheria
    ("node_11", "node_14", 0.05),
    ("node_11", "node_15", 0.07),
    ("node_14", "Canis_lupus", 0.05),
    ("node_14", "Felis_catus", 0.06),
    ("node_15", "node_16", 0.04),
    ("node_15", "Equus_caballus", 0.08),
    ("node_16", "Sus_scrofa", 0.06),
    ("node_16", "Bos_taurus", 0.05),
    # Outgroups (scaled so max depth ≈ 2× mammalian max, giving a balanced phylogram)
    ("node_17", "node_18", 0.06),
    ("node_17", "node_19", 0.15),
    ("node_18", "Gallus_gallus", 0.05),
    ("node_18", "Xenopus_tropicalis", 0.09),
    ("node_19", "Danio_rerio", 0.12),
    ("node_19", "Drosophila_melanogaster", 0.22),
]
counts["phylo.tsv"] = write_tsv(
    "phylo.tsv",
    ["parent", "child", "length"],
    phylo_edges,
)

# ---------------------------------------------------------------------------
# synteny_seqs.tsv
# ---------------------------------------------------------------------------
seq_rows = [
    ("Chr1A", 2800000),
    ("Chr1B", 2650000),
    ("Chr2A", 1900000),
    ("Chr2B", 1750000),
]
counts["synteny_seqs.tsv"] = write_tsv("synteny_seqs.tsv", ["name", "length"], seq_rows)

# ---------------------------------------------------------------------------
# synteny_blocks.tsv
# ---------------------------------------------------------------------------
# Generate non-overlapping blocks between pairs
def generate_synteny_blocks(seq1, len1, seq2, len2, n_blocks, inversion_rate=0.2, seed_offset=0):
    rng = np.random.default_rng(42 + seed_offset)
    # Divide seq1 into n_blocks segments with gaps
    min_block = 50_000
    max_block = 400_000
    blocks = []
    pos1 = int(len1 * 0.02)
    pos2 = int(len2 * 0.02)
    for i in range(n_blocks):
        block_len = int(rng.integers(min_block, max_block))
        if pos1 + block_len > len1 * 0.97:
            break
        end1 = pos1 + block_len
        # Add small jitter to seq2 position
        offset = int(rng.integers(-20_000, 20_000))
        start2 = max(0, pos2 + offset)
        end2 = start2 + int(block_len * rng.uniform(0.85, 1.15))
        if end2 > len2 * 0.97:
            break
        strand = "-" if rng.random() < inversion_rate else "+"
        blocks.append((seq1, pos1, end1, seq2, start2, end2, strand))
        gap1 = int(rng.integers(10_000, 50_000))
        gap2 = int(rng.integers(10_000, 50_000))
        pos1 = end1 + gap1
        pos2 = end2 + gap2
    return blocks

blocks_1ab = generate_synteny_blocks("Chr1A", 2_800_000, "Chr1B", 2_650_000, 12, seed_offset=0)
blocks_2ab = generate_synteny_blocks("Chr2A", 1_900_000, "Chr2B", 1_750_000, 10, seed_offset=10)
# A few cross-chromosome blocks
cross_blocks = [
    ("Chr1A", 2_400_000, 2_550_000, "Chr2B", 1_550_000, 1_700_000, "+"),
    ("Chr2A", 100_000,   250_000,   "Chr1B", 2_400_000, 2_550_000, "-"),
    ("Chr1A", 100_000,   220_000,   "Chr2A", 1_600_000, 1_720_000, "+"),
]
all_blocks = blocks_1ab + blocks_2ab + cross_blocks
counts["synteny_blocks.tsv"] = write_tsv(
    "synteny_blocks.tsv",
    ["seq1", "start1", "end1", "seq2", "start2", "end2", "strand"],
    all_blocks,
)

# ---------------------------------------------------------------------------
# reads.tsv
# ---------------------------------------------------------------------------
n_reads = 350
# Cluster center around 2000-4000 (peak), some scattered
peak_reads = int(n_reads * 0.65)
scatter_reads = n_reads - peak_reads

peak_starts = np.random.normal(2800, 600, peak_reads).astype(int)
peak_starts = np.clip(peak_starts, 0, 7800)
scatter_starts = np.random.randint(0, 7900, scatter_reads)
all_starts = np.concatenate([peak_starts, scatter_starts])
lengths = np.random.randint(80, 251, n_reads)
all_ends = all_starts + lengths
all_ends = np.minimum(all_ends, 8000)
strands = np.random.choice(["+", "-"], n_reads)
order = np.argsort(all_starts)
rows = []
for rank, i in enumerate(order):
    name = f"read_{rank+1:04d}"
    rows.append((name, int(all_starts[i]), int(all_ends[i]), strands[i]))
counts["reads.tsv"] = write_tsv("reads.tsv", ["name", "start", "end", "strand"], rows)

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print(f"\n{'Filename':<30} {'Rows':>6}")
print("-" * 38)
total_rows = 0
for fname in sorted(counts):
    print(f"{fname:<30} {counts[fname]:>6}")
    total_rows += counts[fname]
print("-" * 38)
print(f"{'TOTAL':<30} {total_rows:>6}")
print(f"\nAll files written to: {OUT}")
