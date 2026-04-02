# DOOM

kuva generates scientific plots. It also generates a fully self-contained, playable DOOM SVG.

The file below is a single `.svg`. No server, no network requests, no external dependencies. Open it in any browser and play. Everything (engine, game data, all ~15 MB of it) is embedded inside.

<div id="doom-container" style="width:100%;max-width:800px;margin:0 auto;overflow:hidden;">
<iframe src="doom.svg" id="doom-frame" width="800" height="600" scrolling="no" style="border:none;display:block;transform-origin:top left;"></iframe>
</div>
<script>
(function(){
  var c = document.getElementById('doom-container');
  var f = document.getElementById('doom-frame');
  function scale(){
    var s = Math.min(1, c.clientWidth / 800);
    f.style.transform = 'scale(' + s + ')';
    c.style.height = Math.round(600 * s) + 'px';
  }
  scale();
  if (window.ResizeObserver) {
    new ResizeObserver(scale).observe(c);
  } else {
    window.addEventListener('resize', scale);
  }
})();
</script>

*Click the game to focus it, then use arrow keys or WASD to move, Ctrl to shoot, Space to open doors, Enter to start.*

---

## Generate your own

The `doom` feature is opt-in and separate from the plotting library. Building it downloads a pre-compiled [Chocolate Doom](https://github.com/cloudflare/doom-wasm) engine (GPL v2) and the shareware DOOM WAD (© id Software, free redistribution permitted) from the kuva GitHub releases on first build, then compiles them directly into the binary.

```bash
cargo build --bin kuva --features cli,doom
./target/debug/kuva doom -o doom.svg
```

Open `doom.svg` in Chrome or Firefox. That's it.

The output is ~15 MB. It's mostly the game data base64-encoded into the SVG. The file is self-contained and works offline.

### Release build

```bash
cargo build --release --bin kuva --features cli,doom
./target/release/kuva doom -o doom.svg
```

---

## How it works

`kuva doom` generates an SVG with a `<foreignObject>` containing an HTML canvas and an embedded `<script>`. The script base64-decodes the WASM engine and WAD at load time, writes the WAD into Emscripten's virtual filesystem, and calls `callMain` to start the game. The whole thing is valid SVG-XML. Any browsers that support `foreignObject` (Chrome, Firefox, Safari, Edge) render it as a fully interactive page.

This means a kuva doom SVG is fully self-contained and portable.
---

## Licenses

- **kuva** — MIT
- **Chocolate Doom engine** (embedded WASM) — GPL v2 · [cloudflare/doom-wasm](https://github.com/cloudflare/doom-wasm)
- **DOOM shareware WAD** — © id Software / ZeniMax Media · free redistribution permitted under original shareware terms
