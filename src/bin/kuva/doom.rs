use clap::Args;

// Assets compiled in at build time by build.rs (only present with --features doom).
const DOOM_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/doom.wasm"));
const DOOM_WAD:  &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/doom1.wad"));
const DOOM_JS:   &str  = include_str!(concat!(env!("OUT_DIR"), "/doom.js"));

/// Generate a self-contained DOOM SVG playable in any browser.
#[derive(Args, Debug)]
pub struct DoomArgs {
    /// Output SVG file.
    #[arg(short, long, default_value = "doom.svg")]
    pub output: String,
}

pub fn run(args: DoomArgs) -> Result<(), String> {
    let svg = generate_svg(DOOM_WAD, DOOM_WASM, DOOM_JS);
    std::fs::write(&args.output, &svg)
        .map_err(|e| format!("failed to write {}: {e}", args.output))?;
    eprintln!("wrote {} ({:.1} MB) — open in any browser to play",
        args.output, svg.len() as f64 / 1_048_576.0);
    Ok(())
}

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(CHARS[b0 >> 2] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(if chunk.len() > 1 { CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[b2 & 0x3f] as char } else { '=' });
    }
    out
}

fn generate_svg(wad: &[u8], wasm: &[u8], doom_js: &str) -> String {
    let wad_b64  = base64_encode(wad);
    let wasm_b64 = base64_encode(wasm);

    // Escape any </script> occurrences in the Emscripten glue so they don't
    // terminate the enclosing <script> tag prematurely.
    let doom_js_safe = doom_js
        .replace("</script", "<\\/script")
        .replace("]]>", "]] >");  // prevent premature CDATA close

    // Capacity estimate: base64 + JS + ~2 KB template.
    let cap = wad_b64.len() + wasm_b64.len() + doom_js_safe.len() + 2048;
    let mut s = String::with_capacity(cap);

    s.push_str(concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
        "<!-- Engine: Chocolate Doom (GPL v2, github.com/cloudflare/doom-wasm).\n",
        "     WAD: DOOM shareware \u{00a9} id Software / ZeniMax Media.\n",
        "     Free redistribution permitted under original shareware terms. -->\n",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"800\" height=\"600\" viewBox=\"0 0 800 600\">\n",
        "  <rect width=\"800\" height=\"600\" fill=\"#000\"/>\n",
        "  <foreignObject x=\"0\" y=\"0\" width=\"800\" height=\"600\">\n",
        "    <body xmlns=\"http://www.w3.org/1999/xhtml\"\n",
        "          style=\"margin:0;padding:0;background:#000;overflow:hidden\">\n",
        "      <canvas id=\"doom-canvas\" width=\"800\" height=\"600\" tabindex=\"-1\"\n",
        "              oncontextmenu=\"event.preventDefault()\"\n",
        "              style=\"display:block;width:800px;height:600px;\"/>\n",
        "      <script type=\"text/javascript\">//<![CDATA[\n",
        "function _b64(s){var b=atob(s),a=new Uint8Array(b.length);\n",
        "  for(var i=0;i<b.length;i++)a[i]=b.charCodeAt(i);return a;}\n",
        "var _WAD='",
    ));
    s.push_str(&wad_b64);
    s.push_str("';\nvar _WASM='");
    s.push_str(&wasm_b64);
    s.push_str(concat!(
        "';\n",
        "var Module={\n",
        "  wasmBinary:_b64(_WASM),\n",
        "  canvas:document.getElementById('doom-canvas'),\n",
        "  noInitialRun:true,\n",
        "  preRun:[function(){\n",
        "    FS.writeFile('doom1.wad',_b64(_WAD));\n",
        "  }],\n",
        "  onRuntimeInitialized:function(){\n",
        "    callMain(['-iwad','doom1.wad','-window','-nogui','-nomusic']);\n",
        "    document.getElementById('doom-canvas').focus();\n",
        "  },\n",
        "  print:function(){},\n",
        "  printErr:function(){}\n",
        "};\n",
    ));
    s.push_str(&doom_js_safe);
    s.push_str(concat!(
        "\n//]]>\n      </script>\n",
        "    </body>\n",
        "  </foreignObject>\n",
        "</svg>\n",
    ));

    s
}
