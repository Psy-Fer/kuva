# Architecture

The diagram below shows how a plot moves through the kuva rendering pipeline.
Each plot type implements only its own `render_*()` function — layout, scene
composition, and backend output are shared infrastructure.

```mermaid
flowchart TD
    subgraph entry ["Entry Points"]
        direction LR
        api["Rust API"]
        cli["CLI binary\nkuva scatter / manhattan / …"]
    end

    subgraph plots ["Plot Types (29)"]
        direction LR
        p1["ScatterPlot"]
        p2["ManhattanPlot"]
        p3["PhyloTreePlot"]
        p4["SankeyPlot"]
        p5["… 25 more"]
    end

    enum["Plot enum\nwraps all plot types"]

    subgraph layout_block ["Layout"]
        l1["Layout\ntitle · labels · ranges · palette · log scale"]
        l2["ComputedLayout\nmargins · tick positions · coordinate mapping"]
        l1 -->|"auto_from_plots()"| l2
    end

    render["render_*()\nper-plot renderer"]

    scene["Scene\nprimitives: Circle · Line · Path · Rect · Text"]

    subgraph backends ["Backends"]
        direction LR
        svg["SVG"]
        png["PNG\n(feature: png)"]
        pdf["PDF\n(feature: pdf)"]
        term["Terminal\n(braille)"]
    end

    api & cli --> plots
    plots --> enum
    enum --> layout_block
    l2 --> render
    enum --> render
    render --> scene
    scene --> backends
```
