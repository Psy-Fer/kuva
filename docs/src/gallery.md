# Gallery

A quick overview of every plot type. Click any image to go to the full documentation page.

<style>
.gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.25rem;
  margin-top: 1.5rem;
}
.gallery-card {
  border: 1px solid var(--sidebar-bg, #e0e0e0);
  border-radius: 6px;
  overflow: hidden;
  transition: box-shadow 0.15s;
}
.gallery-card:hover {
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
}
.gallery-card a {
  display: block;
  text-decoration: none;
  color: inherit;
}
.gallery-card img {
  width: 100%;
  height: auto;
  display: block;
}
.gallery-caption {
  padding: 0.5rem 0.75rem;
  font-weight: 600;
  font-size: 0.9rem;
}
.gallery-desc {
  padding: 0 0.75rem 0.6rem;
  font-size: 0.8rem;
  color: var(--fg, #555);
  opacity: 0.8;
}
</style>

<div class="gallery">

<div class="gallery-card"><a href="./plots/scatter.html">
<img src="./assets/scatter/trend.svg" alt="Scatter Plot">
<div class="gallery-caption">Scatter Plot</div>
<div class="gallery-desc">x/y points with optional trend line, error bars, and variable size.</div>
</a></div>

<div class="gallery-card"><a href="./plots/line.html">
<img src="./assets/line/styles.svg" alt="Line Plot">
<div class="gallery-caption">Line Plot</div>
<div class="gallery-desc">Connected series with solid, dashed, dotted, and dash-dot styles.</div>
</a></div>

<div class="gallery-card"><a href="./plots/bar.html">
<img src="./assets/bar/stacked.svg" alt="Bar Chart">
<div class="gallery-caption">Bar Chart</div>
<div class="gallery-desc">Categorical bars — grouped, stacked, and single-series.</div>
</a></div>

<div class="gallery-card"><a href="./plots/histogram.html">
<img src="./assets/histogram/overlapping.svg" alt="Histogram">
<div class="gallery-caption">Histogram</div>
<div class="gallery-desc">Distribution of a numeric column with configurable bins and normalisation.</div>
</a></div>

<div class="gallery-card"><a href="./plots/histogram2d.html">
<img src="./assets/histogram2d/correlation.svg" alt="2D Histogram">
<div class="gallery-caption">2D Histogram</div>
<div class="gallery-desc">Joint density of two numeric columns with colourmap and optional correlation.</div>
</a></div>

<div class="gallery-card"><a href="./plots/boxplot.html">
<img src="./assets/boxplot/swarm_overlay.svg" alt="Box Plot">
<div class="gallery-caption">Box Plot</div>
<div class="gallery-desc">Median, IQR, and whiskers per group — with optional strip or swarm overlay.</div>
</a></div>

<div class="gallery-card"><a href="./plots/violin.html">
<img src="./assets/violin/swarm_overlay.svg" alt="Violin Plot">
<div class="gallery-caption">Violin Plot</div>
<div class="gallery-desc">KDE density mirrored per group, with configurable bandwidth and swarm overlay.</div>
</a></div>

<div class="gallery-card"><a href="./plots/strip.html">
<img src="./assets/strip/composed.svg" alt="Strip Plot">
<div class="gallery-caption">Strip Plot</div>
<div class="gallery-desc">Individual points jittered or swarmed by group; composes with box/violin.</div>
</a></div>

<div class="gallery-card"><a href="./plots/pie.html">
<img src="./assets/pie/outside_labels.svg" alt="Pie Chart">
<div class="gallery-caption">Pie Chart</div>
<div class="gallery-desc">Pie or donut chart with inside, outside, or legend-only labels.</div>
</a></div>

<div class="gallery-card"><a href="./plots/heatmap.html">
<img src="./assets/heatmap/values.svg" alt="Heatmap">
<div class="gallery-caption">Heatmap</div>
<div class="gallery-desc">Matrix of values with colourmap, optional cell labels, and row/col clustering.</div>
</a></div>

<div class="gallery-card"><a href="./plots/dotplot.html">
<img src="./assets/dotplot/size_legend.svg" alt="Dot Plot">
<div class="gallery-caption">Dot Plot</div>
<div class="gallery-desc">Size and colour encoding on a categorical grid — common in single-cell analysis.</div>
</a></div>

<div class="gallery-card"><a href="./plots/contour.html">
<img src="./assets/contour/filled.svg" alt="Contour Plot">
<div class="gallery-caption">Contour Plot</div>
<div class="gallery-desc">Filled or line contours from gridded or scattered x/y/z data via IDW interpolation.</div>
</a></div>

<div class="gallery-card"><a href="./plots/stacked_area.html">
<img src="./assets/stacked_area/normalized.svg" alt="Stacked Area Plot">
<div class="gallery-caption">Stacked Area Plot</div>
<div class="gallery-desc">Stacked time-series with absolute or 100%-normalised mode.</div>
</a></div>

<div class="gallery-card"><a href="./plots/waterfall.html">
<img src="./assets/waterfall/connectors_values.svg" alt="Waterfall Chart">
<div class="gallery-caption">Waterfall Chart</div>
<div class="gallery-desc">Running total with positive/negative deltas, connectors, and value labels.</div>
</a></div>

<div class="gallery-card"><a href="./plots/candlestick.html">
<img src="./assets/candlestick/volume.svg" alt="Candlestick Chart">
<div class="gallery-caption">Candlestick Chart</div>
<div class="gallery-desc">OHLC bars with optional volume panel and datetime axis.</div>
</a></div>

<div class="gallery-card"><a href="./plots/volcano.html">
<img src="./assets/volcano/labeled.svg" alt="Volcano Plot">
<div class="gallery-caption">Volcano Plot</div>
<div class="gallery-desc">log₂FC vs −log₁₀(p) with threshold lines and top-N gene labels.</div>
</a></div>

<div class="gallery-card"><a href="./plots/manhattan.html">
<img src="./assets/manhattan/gene_labels.svg" alt="Manhattan Plot">
<div class="gallery-caption">Manhattan Plot</div>
<div class="gallery-desc">GWAS p-values across chromosomes with genome-wide threshold and gene labels.</div>
</a></div>

<div class="gallery-card"><a href="./plots/upset.html">
<img src="./assets/upset/custom.svg" alt="UpSet Plot">
<div class="gallery-caption">UpSet Plot</div>
<div class="gallery-desc">Set intersection sizes as a bar chart with a membership matrix below.</div>
</a></div>

<div class="gallery-card"><a href="./plots/chord.html">
<img src="./assets/chord/styled.svg" alt="Chord Diagram">
<div class="gallery-caption">Chord Diagram</div>
<div class="gallery-desc">Pairwise flows between N groups as arc segments and Bézier ribbons.</div>
</a></div>

<div class="gallery-card"><a href="./plots/sankey.html">
<img src="./assets/sankey/variant_filter.svg" alt="Sankey Diagram">
<div class="gallery-caption">Sankey Diagram</div>
<div class="gallery-desc">Multi-stage flow with tapered Bézier links and source/gradient colouring.</div>
</a></div>

<div class="gallery-card"><a href="./plots/phylo.html">
<img src="./assets/phylo/clade_color.svg" alt="Phylogenetic Tree">
<div class="gallery-caption">Phylogenetic Tree</div>
<div class="gallery-desc">Newick, edge-list, distance-matrix, or linkage input; rectangular, slanted, or circular layout.</div>
</a></div>

<div class="gallery-card"><a href="./plots/synteny.html">
<img src="./assets/synteny/three_seq.svg" alt="Synteny Plot">
<div class="gallery-caption">Synteny Plot</div>
<div class="gallery-desc">Genome synteny ribbons between sequences with forward and inverted blocks.</div>
</a></div>

<div class="gallery-card"><a href="./plots/band.html">
<img src="./assets/band/multi.svg" alt="Band Plot">
<div class="gallery-caption">Band Plot</div>
<div class="gallery-desc">Shaded confidence or credible interval bands, standalone or attached to a line/scatter.</div>
</a></div>

<div class="gallery-card"><a href="./plots/series.html">
<img src="./assets/series/multi.svg" alt="Series Plot">
<div class="gallery-caption">Series Plot</div>
<div class="gallery-desc">Multiple named data series on one canvas with per-series line and marker styles.</div>
</a></div>

<div class="gallery-card"><a href="./plots/brick.html">
<img src="./assets/brick/strigar.svg" alt="Brick Plot">
<div class="gallery-caption">Brick Plot</div>
<div class="gallery-desc">Read-level alignment bricks with per-base colouring and STRIGAR string support.</div>
</a></div>

</div>
