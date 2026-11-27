//! Builds minimal TrueType fonts for tests: just the tables ttf-parser and
//! the font registry need, with a caller-supplied `cmap`.
#![allow(dead_code)]

fn be16(v: u16) -> [u8; 2] {
    v.to_be_bytes()
}
fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}

/// A minimal TrueType font: `head`, `hhea`, `maxp`, `hmtx`, `name` (family
/// `family`) and the given `cmap`. Two glyphs, both 500 units wide.
pub fn font(family: &str, cmap: Vec<u8>) -> Vec<u8> {
    font_with_advances(family, cmap, &[500, 500])
}

/// [`font`] with one glyph per entry of `advances`, in font units.
pub fn font_with_advances(family: &str, cmap: Vec<u8>, advances: &[u16]) -> Vec<u8> {
    font_parts(family, cmap, advances, 0)
}

/// [`font`] whose `name` table is followed by `name_padding` zero bytes, so
/// reading it costs that much more.
pub fn font_with_name_padding(family: &str, name_padding: usize) -> Vec<u8> {
    font_parts(
        family,
        cmap(1, format12(&[(0x41, 0x5A, 1)])),
        &[500, 500],
        name_padding,
    )
}

fn font_parts(family: &str, cmap: Vec<u8>, advances: &[u16], name_padding: usize) -> Vec<u8> {
    let mut head = Vec::new();
    head.extend(be32(0x0001_0000)); // version
    head.extend(be32(0x0001_0000)); // revision
    head.extend(be32(0)); // checksum adjustment
    head.extend(be32(0x5F0F_3CF5)); // magic
    head.extend(be16(0)); // flags
    head.extend(be16(1000)); // units per em
    head.extend([0; 16]); // created, modified
    for v in [0i16, -200, 500, 800] {
        head.extend(v.to_be_bytes()); // bounding box
    }
    head.extend([0; 6]); // mac style, lowest ppem, direction hint
    head.extend(be16(0)); // index to loc format
    head.extend(be16(0)); // glyph data format

    let mut hhea = Vec::new();
    hhea.extend(be32(0x0001_0000));
    for v in [800i16, -200, 0] {
        hhea.extend(v.to_be_bytes()); // ascender, descender, line gap
    }
    hhea.extend([0; 24]);
    hhea.extend(be16(advances.len() as u16)); // number of h metrics

    let mut maxp = Vec::new();
    maxp.extend(be32(0x0000_5000));
    maxp.extend(be16(advances.len() as u16));

    let mut hmtx = Vec::new();
    for &a in advances {
        hmtx.extend(be16(a));
        hmtx.extend(be16(0));
    }

    let utf16: Vec<u8> = family
        .encode_utf16()
        .flat_map(|u| u.to_be_bytes())
        .collect();
    let mut name = Vec::new();
    name.extend(be16(0)); // format
    name.extend(be16(1)); // count
    name.extend(be16(6 + 12)); // string offset
    name.extend(be16(3)); // platform: Windows
    name.extend(be16(1)); // encoding: Unicode BMP
    name.extend(be16(0x0409)); // language: en-US
    name.extend(be16(1)); // name id: family
    name.extend(be16(utf16.len() as u16));
    name.extend(be16(0));
    name.extend(utf16);
    name.resize(name.len() + name_padding, 0);

    let tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"cmap", cmap),
        (b"head", head),
        (b"hhea", hhea),
        (b"hmtx", hmtx),
        (b"maxp", maxp),
        (b"name", name),
    ];
    let mut out = Vec::new();
    out.extend(be32(0x0001_0000));
    out.extend(be16(tables.len() as u16));
    out.extend([0; 6]); // search range, entry selector, range shift
    let mut offset = 12 + 16 * tables.len();
    let mut body = Vec::new();
    for (tag, data) in &tables {
        out.extend(*tag);
        out.extend(be32(0)); // checksum
        out.extend(be32(offset as u32));
        out.extend(be32(data.len() as u32));
        let mut padded = data.clone();
        padded.resize(data.len().div_ceil(4) * 4, 0);
        offset += padded.len();
        body.extend(padded);
    }
    out.extend(body);
    out
}

/// A format-12 subtable with the given `(start, end, start_glyph)` groups.
pub fn format12(groups: &[(u32, u32, u32)]) -> Vec<u8> {
    let mut t = Vec::new();
    t.extend(be16(12));
    t.extend(be16(0));
    t.extend(be32(16 + 12 * groups.len() as u32));
    t.extend(be32(0)); // language
    t.extend(be32(groups.len() as u32));
    for &(start, end, glyph) in groups {
        t.extend(be32(start));
        t.extend(be32(end));
        t.extend(be32(glyph));
    }
    t
}

/// A cmap with `records` Windows/UCS-4 encoding records, all pointing at the
/// one `subtable`.
pub fn cmap(records: u16, subtable: Vec<u8>) -> Vec<u8> {
    let mut t = Vec::new();
    t.extend(be16(0));
    t.extend(be16(records));
    let offset = 4 + 8 * u32::from(records);
    for _ in 0..records {
        t.extend(be16(3)); // Windows
        t.extend(be16(10)); // UCS-4
        t.extend(be32(offset));
    }
    t.extend(subtable);
    t
}

/// A cmap with one Windows/UCS-4 record per subtable, in order.
pub fn cmap_of(subtables: &[Vec<u8>]) -> Vec<u8> {
    let mut t = Vec::new();
    t.extend(be16(0));
    t.extend(be16(subtables.len() as u16));
    let mut offset = 4 + 8 * subtables.len() as u32;
    for s in subtables {
        t.extend(be16(3));
        t.extend(be16(10));
        t.extend(be32(offset));
        offset += s.len() as u32;
    }
    for s in subtables {
        t.extend(s);
    }
    t
}

/// `font` with every table offset moved by `base`, for placing it inside a
/// collection.
fn rebase(font: &[u8], base: u32) -> Vec<u8> {
    let mut out = font.to_vec();
    let num_tables = u16::from_be_bytes([font[4], font[5]]) as usize;
    for i in 0..num_tables {
        let at = 12 + 16 * i + 8;
        let offset = u32::from_be_bytes([out[at], out[at + 1], out[at + 2], out[at + 3]]);
        out[at..at + 4].copy_from_slice(&(offset + base).to_be_bytes());
    }
    out
}

fn collection_header(offsets: &[u32]) -> Vec<u8> {
    let mut t = Vec::new();
    t.extend(*b"ttcf");
    t.extend(be32(0x0001_0000));
    t.extend(be32(offsets.len() as u32));
    for &o in offsets {
        t.extend(be32(o));
    }
    t
}

/// A TrueType collection holding `fonts`, one after another.
pub fn collection(fonts: &[Vec<u8>]) -> Vec<u8> {
    let mut base = 12 + 4 * fonts.len() as u32;
    let mut offsets = Vec::new();
    let mut body = Vec::new();
    for f in fonts {
        offsets.push(base);
        let mut placed = rebase(f, base);
        placed.resize(placed.len().div_ceil(4) * 4, 0);
        base += placed.len() as u32;
        body.extend(placed);
    }
    let mut out = collection_header(&offsets);
    out.extend(body);
    out
}

/// A collection claiming `count` faces that all point at the one `font`.
pub fn collection_repeating(font: &[u8], count: u32) -> Vec<u8> {
    let base = 12 + 4 * count;
    let mut out = collection_header(&vec![base; count as usize]);
    out.extend(rebase(font, base));
    out
}

/// A collection of `count` faces, each with its own table directory, all
/// pointing at the one set of tables from `font`.
pub fn collection_sharing_tables(font: &[u8], count: u32) -> Vec<u8> {
    let num_tables = u16::from_be_bytes([font[4], font[5]]) as u32;
    let dir_len = 12 + 16 * num_tables;
    let header = 12 + 4 * count;
    let tables_at = header + count * dir_len;
    let dir = rebase(&font[..dir_len as usize], tables_at - dir_len);
    let offsets: Vec<u32> = (0..count).map(|i| header + i * dir_len).collect();
    let mut out = collection_header(&offsets);
    for _ in 0..count {
        out.extend(&dir);
    }
    out.extend(&font[dir_len as usize..]);
    out
}
