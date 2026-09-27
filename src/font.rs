use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

pub type Glyph = u16;
pub const LEGACY_PROFILE: &str = "ditto-moderndos-square-v1";
pub const PROFILE: &str = "ditto-blocks-square-v2";
pub const CELL_SIZE: u32 = 16;
pub const ATLAS_SIDE: u32 = 512;
pub const ATLAS_COLUMNS: u32 = ATLAS_SIDE / CELL_SIZE;
pub const BITMAP: &[u8; 4096] = include_bytes!("../assets/moderndos-8x16.bin");
/// Append-only catalogue. The first 256 IDs and their bitmaps remain byte-compatible with V1.
pub static CHARS: LazyLock<Vec<char>> = LazyLock::new(|| {
    let mut chars: Vec<char> = include_str!("../assets/cp437.txt").chars().collect();
    for c in (0x2580..=0x259f)
        .chain(0x1fb70..=0x1fb9f)
        .filter(|c| *c != 0x1fb93)
        .chain(0x1fb00..=0x1fb3b)
        .chain(0x2800..=0x28ff)
        .filter_map(char::from_u32)
        .chain("━┃╱╲╳╭╮╯╰┄┅┆┇┈┉┊┋╌╍╎╏€".chars())
    {
        if !chars.contains(&c) {
            chars.push(c);
        }
    }
    assert!(chars.len() < (ATLAS_COLUMNS * ATLAS_COLUMNS) as usize);
    chars
});
pub fn character(glyph: Glyph) -> char {
    CHARS.get(glyph as usize).copied().unwrap_or('\u{fffd}')
}
pub fn valid(glyph: Glyph) -> bool {
    (glyph as usize) < CHARS.len()
}
pub fn glyph(c: char) -> Option<Glyph> {
    if c == ' ' {
        return Some(32);
    }
    CHARS.iter().position(|v| *v == c).map(|i| i as Glyph)
}
pub fn is_block(c: char) -> bool {
    matches!(c as u32, 0x2580..=0x259f | 0x1fb00..=0x1fb3b | 0x1fb70..=0x1fb9f | 0x2800..=0x28ff)
        || c == '■'
}
pub fn block_glyphs() -> Vec<Glyph> {
    let mut out = vec![32];
    for cp in (0x2580..=0x259f)
        .chain(0x1fb70..=0x1fb9f)
        .chain(0x1fb00..=0x1fb3b)
        .chain(0x2800..=0x28ff)
        .chain([0x25a0])
    {
        if let Some(g) = char::from_u32(cp).and_then(glyph) {
            out.push(g);
        }
    }
    out
}
pub fn trait_glyphs() -> Vec<Glyph> {
    CHARS
        .iter()
        .enumerate()
        .filter_map(|(i, c)| matches!(*c as u32, 0x2500..=0x257f).then_some(i as Glyph))
        .collect()
}
fn shade(x: u32, y: u32) -> bool {
    BITMAP[177 * 16 + y as usize] & (0x80 >> (x / 2)) != 0
}
fn pattern(c: char, x: u32, y: u32) -> bool {
    let cp = c as u32;
    match cp {
        0x2580 => y < 8,
        0x2581..=0x2588 => y >= 16 - 2 * (cp - 0x2580),
        0x2589..=0x258f => x < 2 * (0x2590 - cp),
        0x2590 => x >= 8,
        0x2591 => x.is_multiple_of(2) && y.is_multiple_of(2),
        0x2592 => shade(x, y),
        0x2593 => !(x.is_multiple_of(2) && y.is_multiple_of(2)),
        0x2594 => y < 2,
        0x2595 => x >= 14,
        0x2596..=0x259f => {
            let masks = [4, 8, 1, 13, 9, 7, 11, 2, 6, 14];
            let quadrant = (y / 8) * 2 + x / 8;
            masks[(cp - 0x2596) as usize] & (1 << quadrant) != 0
        }
        0x1fb70..=0x1fb75 => x / 2 == cp - 0x1fb70 + 1,
        0x1fb76..=0x1fb7b => y / 2 == cp - 0x1fb76 + 1,
        0x1fb7c => x < 2 || y >= 14,
        0x1fb7d => x < 2 || y < 2,
        0x1fb7e => x >= 14 || y < 2,
        0x1fb7f => x >= 14 || y >= 14,
        0x1fb80 => !(2..14).contains(&y),
        0x1fb81 => [0, 2, 4, 7].contains(&(y / 2)),
        0x1fb82..=0x1fb86 => y < 2 * [2, 3, 5, 6, 7][(cp - 0x1fb82) as usize],
        0x1fb87..=0x1fb8b => x >= 16 - 2 * [2, 3, 5, 6, 7][(cp - 0x1fb87) as usize],
        0x1fb8c => x < 8 && shade(x, y),
        0x1fb8d => x >= 8 && shade(x, y),
        0x1fb8e => y < 8 && shade(x, y),
        0x1fb8f => y >= 8 && shade(x, y),
        0x1fb90 => !shade(x, y),
        0x1fb91 => y < 8 || !shade(x, y),
        0x1fb92 => y >= 8 || !shade(x, y),
        0x1fb94 => x >= 8 || !shade(x, y),
        0x1fb95 => (x / 4 + y / 4).is_multiple_of(2),
        0x1fb96 => !(x / 4 + y / 4).is_multiple_of(2),
        0x1fb97 => y % 4 < 2,
        0x1fb98 => (x + 16 - y).is_multiple_of(4),
        0x1fb99 => (x + y).is_multiple_of(4),
        0x1fb9a => (x as i32 * 2 - 15).abs() <= (y as i32 * 2 - 15).abs(),
        0x1fb9b => (x as i32 * 2 - 15).abs() >= (y as i32 * 2 - 15).abs(),
        0x1fb9c => x + y < 16 && shade(x, y),
        0x1fb9d => y <= x && shade(x, y),
        0x1fb9e => x + y >= 15 && shade(x, y),
        0x1fb9f => y >= x && shade(x, y),
        0x1fb00..=0x1fb3b => {
            let mask = (1u8..63)
                .filter(|m| *m != 21 && *m != 42)
                .nth((cp - 0x1fb00) as usize)
                .unwrap();
            let sextant = (y * 3 / 16) * 2 + x / 8;
            mask & (1 << sextant) != 0
        }
        0x2800..=0x28ff => {
            let row = y / 4;
            let col = x / 8;
            let bit = if col == 0 {
                [0, 1, 2, 6][row as usize]
            } else {
                [3, 4, 5, 7][row as usize]
            };
            let dx = x as i32 - (3 + col * 8) as i32;
            let dy = y as i32 - (1 + row * 4) as i32;
            (cp - 0x2800) & (1 << bit) != 0 && dx * dx + dy * dy <= 1
        }
        0x2501 => (6..10).contains(&y),
        0x2503 => (6..10).contains(&x),
        0x2571 => x + y == 15,
        0x2572 => x == y,
        0x2573 => x == y || x + y == 15,
        0x256d..=0x2570 => {
            let (cx, cy) = match cp {
                0x256d => (12, 12),
                0x256e => (3, 12),
                0x256f => (3, 3),
                _ => (12, 3),
            };
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            let arc = (dx * dx + dy * dy - 25).abs() <= 5;
            match cp {
                0x256d => (x >= 12 && y == 7) || (y >= 12 && x == 7) || (arc && x <= 12 && y <= 12),
                0x256e => (x <= 3 && y == 7) || (y >= 12 && x == 8) || (arc && x >= 3 && y <= 12),
                0x256f => (x <= 3 && y == 8) || (y <= 3 && x == 8) || (arc && x >= 3 && y >= 3),
                _ => (x >= 12 && y == 8) || (y <= 3 && x == 7) || (arc && x <= 12 && y >= 3),
            }
        }
        0x2504..=0x250b | 0x254c..=0x254f => {
            let vertical = matches!(cp, 0x2506 | 0x2507 | 0x250a | 0x250b | 0x254e | 0x254f);
            let heavy = cp % 2 == 1;
            let (across, along) = if vertical { (x, y) } else { (y, x) };
            let on_line = if heavy {
                (6..10).contains(&across)
            } else {
                across == 7
            };
            on_line
                && if cp >= 0x254c {
                    along % 8 < 6
                } else if cp >= 0x2508 {
                    along % 4 < 3
                } else {
                    along % 6 < 4
                }
        }
        0x20ac => {
            (BITMAP[b'C' as usize * 16 + y as usize] & (0x80 >> (x / 2)) != 0)
                || ((y == 6 || y == 9) && (2..11).contains(&x))
        }
        _ => false,
    }
}
static MASKS: LazyLock<Vec<[u16; 16]>> = LazyLock::new(|| {
    CHARS
        .iter()
        .enumerate()
        .map(|(g, c)| {
            let mut rows = [0u16; 16];
            for y in 0..16 {
                for x in 0..16 {
                    let on = if g < 256 {
                        BITMAP[g * 16 + y] & (0x80 >> (x / 2)) != 0
                    } else {
                        pattern(*c, x as u32, y as u32)
                    };
                    if on {
                        rows[y] |= 1 << x;
                    }
                }
            }
            rows
        })
        .collect()
});
pub fn bit(glyph: Glyph, x: u32, y: u32) -> bool {
    if x >= 16 || y >= 16 {
        return false;
    }
    MASKS
        .get(glyph as usize)
        .is_some_and(|rows| rows[y as usize] & (1 << x) != 0)
}
pub fn atlas_pixels() -> Vec<u8> {
    let mut pixels = vec![255; (ATLAS_SIDE * ATLAS_SIDE * 4) as usize];
    for p in pixels.as_chunks_mut::<4>().0 {
        p[3] = 0;
    }
    for g in 0..CHARS.len() {
        let ox = g as u32 % ATLAS_COLUMNS * 16;
        let oy = g as u32 / ATLAS_COLUMNS * 16;
        for y in 0..16 {
            for x in 0..16 {
                pixels[(((oy + y) * ATLAS_SIDE + ox + x) * 4 + 3) as usize] =
                    if bit(g as Glyph, x, y) { 255 } else { 0 };
            }
        }
    }
    pixels[((ATLAS_SIDE * ATLAS_SIDE - 1) * 4 + 3) as usize] = 255;
    pixels
}
pub fn parse_text(text: &str) -> Result<Vec<Vec<Glyph>>, String> {
    let normalized: String = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .nfc()
        .collect();
    let mut rows = vec![Vec::new()];
    let mut invalid = Vec::new();
    for c in normalized.chars() {
        if c == '\n' {
            if rows.len() >= crate::core::MAX_SIDE as usize {
                return Err("Texte trop grand (maximum 512 × 512).".into());
            }
            rows.push(Vec::new());
            continue;
        }
        let row = rows.last_mut().unwrap();
        if c == '\t' {
            row.resize(row.len() + 4 - row.len() % 4, 32);
        } else if let Some(g) = glyph(c) {
            row.push(g);
        } else if !invalid.contains(&c) {
            invalid.push(c);
        }
        if row.len() > crate::core::MAX_SIDE as usize {
            return Err("Texte trop grand (maximum 512 × 512).".into());
        }
    }
    if invalid.is_empty() {
        Ok(rows)
    } else {
        Err(format!(
            "Caractères hors répertoire : {}. Aucune modification.",
            invalid.iter().take(12).collect::<String>()
        ))
    }
}
