//! Locally loaded vector typeface shared by UI, canvas and PNG export.
//! Font files are never embedded in the application or document archives.
use crate::font::{self, Glyph};
use fontdue::{Font, FontSettings};
use std::{path::PathBuf, sync::LazyLock};

pub const CELL_WIDTH: u32 = 8;
pub const PROFILE: &str = "ditto-sfmono-v3";
pub const ASPECT: f32 = CELL_WIDTH as f32 / CELL_HEIGHT as f32;
pub const CELL_HEIGHT: u32 = 16;
pub const OVERSAMPLE: u32 = 4;
pub const NATIVE_WIDTH: u32 = 8;
pub const GLYPH_WIDTH: u32 = NATIVE_WIDTH * OVERSAMPLE;
pub const GLYPH_HEIGHT: u32 = CELL_HEIGHT * OVERSAMPLE;
pub const PAD: u32 = 2;
pub const COLUMNS: u32 = 32;
pub const TILE_WIDTH: u32 = GLYPH_WIDTH + PAD * 2;
pub const TILE_HEIGHT: u32 = GLYPH_HEIGHT + PAD * 2;
pub const ATLAS_WIDTH: u32 = COLUMNS * TILE_WIDTH;
pub static CHARACTERS: LazyLock<Vec<char>> = LazyLock::new(|| {
    let mut c = font::CHARS.clone();
    for x in "—–‘’“”…\u{202f}œŒ↦✓✕ÀÈ×".chars() {
        if !c.contains(&x) {
            c.push(x);
        }
    }
    c
});
struct Face {
    font: Font,
    name: String,
    path: PathBuf,
}
fn load_face() -> Option<Face> {
    let mut paths = Vec::new();
    if let Some(p) = std::env::var_os("DITTO_FONT_PATH") {
        paths.push(PathBuf::from(p));
    }
    paths.extend([
        "/Library/Fonts/SF-Mono-Regular.otf",
        "/System/Applications/Utilities/Terminal.app/Contents/Resources/Fonts/SF-Mono-Regular.otf",
        "/System/Library/Fonts/SFNSMono.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/liberation2/LiberationMono-Regular.ttf",
        "/System/Library/Fonts/Menlo.ttc",
    ].into_iter().map(PathBuf::from));
    if let Some(home) = directories::BaseDirs::new() {
        for file in [
            ".local/share/fonts/SF-Mono-Regular.otf",
            ".local/share/fonts/SFMono-Regular.ttf",
            "Library/Fonts/SF-Mono-Regular.otf",
        ] {
            paths.insert(
                usize::from(std::env::var_os("DITTO_FONT_PATH").is_some()),
                home.home_dir().join(file),
            );
        }
    }
    #[cfg(target_os = "linux")]
    if let Ok(o) = std::process::Command::new("fc-match")
        .args(["-f", "%{file}", "monospace"])
        .output()
        && o.status.success()
    {
        paths.push(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));
    }
    for path in paths {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        if let Ok(font) = Font::from_bytes(data, FontSettings::default()) {
            let name = font.name().unwrap_or("Monospace").to_string();
            return Some(Face { font, name, path });
        }
    }
    None
}
static FACE: LazyLock<Option<Face>> = LazyLock::new(load_face);
pub fn description() -> String {
    FACE.as_ref()
        .map(|f| format!("{} ({})", f.name, f.path.display()))
        .unwrap_or_else(|| "monospace bitmap de secours".into())
}
pub fn is_vector() -> bool {
    FACE.is_some()
}
pub fn supported(c: char) -> bool {
    FACE.as_ref()
        .is_some_and(|f| f.font.lookup_glyph_index(c) != 0)
}
pub fn atlas_height() -> u32 {
    (CHARACTERS.len() as u32).div_ceil(COLUMNS) * TILE_HEIGHT
}
pub fn ui_glyph(c: char) -> Glyph {
    CHARACTERS
        .iter()
        .position(|v| *v == c)
        .unwrap_or(b'?' as usize) as Glyph
}
/// Terminal graphics are tiles, not text with typographic leading.
/// Their geometry and edge padding must cover their full cell to join adjacent rows.
pub fn is_tile(c: char) -> bool {
    matches!(c as u32,0x2500..=0x259f|0x1fb00..=0x1fb3b|0x1fb70..=0x1fb9f|0x2800..=0x28ff)
}
fn make_mask(c: char) -> Vec<u8> {
    let mut mask = vec![0; (GLYPH_WIDTH * GLYPH_HEIGHT) as usize];
    if let Some(face) = FACE
        .as_ref()
        .filter(|f| !is_tile(c) && f.font.lookup_glyph_index(c) != 0)
    {
        let px = NATIVE_WIDTH as f32 / face.font.metrics('M', 1.).advance_width;
        let line = face.font.horizontal_line_metrics(px).unwrap();
        let baseline = (CELL_HEIGHT as f32 - (line.ascent - line.descent)) / 2. + line.ascent;
        let (m, bitmap) = face.font.rasterize(c, px * OVERSAMPLE as f32);
        let top = (baseline * OVERSAMPLE as f32).round() as i32 - m.ymin - m.height as i32;
        for y in 0..m.height {
            for x in 0..m.width {
                let dx = m.xmin + x as i32;
                let dy = top + y as i32;
                if dx >= 0 && dy >= 0 && dx < GLYPH_WIDTH as i32 && dy < GLYPH_HEIGHT as i32 {
                    mask[(dy as u32 * GLYPH_WIDTH + dx as u32) as usize] = bitmap[y * m.width + x];
                }
            }
        }
    } else if let Some(g) = font::glyph(c) {
        for y in 0..GLYPH_HEIGHT {
            for x in 0..GLYPH_WIDTH {
                if font::bit(g, x * 16 / GLYPH_WIDTH, y * 16 / GLYPH_HEIGHT) {
                    mask[(y * GLYPH_WIDTH + x) as usize] = 255;
                }
            }
        }
    }
    mask
}
static MASKS: LazyLock<Vec<Vec<u8>>> =
    LazyLock::new(|| CHARACTERS.iter().map(|c| make_mask(*c)).collect());
pub fn mask(g: Glyph) -> &'static [u8] {
    MASKS.get(g as usize).map(Vec::as_slice).unwrap_or(&[])
}
pub fn coverage(g: Glyph, x: u32, y: u32, scale: u32) -> u8 {
    let mask = mask(g);
    if mask.is_empty() {
        return 0;
    }
    // Match bilinear texture sampling at output pixel centres, including transparent padding.
    let sx = (x as f32 + 0.5) * GLYPH_WIDTH as f32 / (CELL_WIDTH * scale) as f32 - 0.5;
    let sy = (y as f32 + 0.5) * OVERSAMPLE as f32 / scale as f32 - 0.5;
    let x0 = sx.floor() as i32;
    let y0 = sy.floor() as i32;
    let fx = sx - x0 as f32;
    let fy = sy - y0 as f32;
    let tile = CHARACTERS.get(g as usize).is_some_and(|c| is_tile(*c));
    let at = |mut x: i32, mut y: i32| -> f32 {
        if tile {
            x = x.clamp(0, GLYPH_WIDTH as i32 - 1);
            y = y.clamp(0, GLYPH_HEIGHT as i32 - 1);
        }
        if x >= 0 && y >= 0 && x < GLYPH_WIDTH as i32 && y < GLYPH_HEIGHT as i32 {
            mask[(y as u32 * GLYPH_WIDTH + x as u32) as usize] as f32
        } else {
            0.
        }
    };
    ((at(x0, y0) * (1. - fx) + at(x0 + 1, y0) * fx) * (1. - fy)
        + (at(x0, y0 + 1) * (1. - fx) + at(x0 + 1, y0 + 1) * fx) * fy)
        .round() as u8
}
pub fn uv(g: Glyph) -> [f32; 4] {
    let x = g as u32 % COLUMNS * TILE_WIDTH + PAD;
    let y = g as u32 / COLUMNS * TILE_HEIGHT + PAD;
    [
        x as f32 / ATLAS_WIDTH as f32,
        y as f32 / atlas_height() as f32,
        (x + GLYPH_WIDTH) as f32 / ATLAS_WIDTH as f32,
        (y + GLYPH_HEIGHT) as f32 / atlas_height() as f32,
    ]
}
pub fn atlas_pixels() -> Vec<u8> {
    let mut pixels = vec![255; (ATLAS_WIDTH * atlas_height() * 4) as usize];
    for p in pixels.as_chunks_mut::<4>().0 {
        p[3] = 0;
    }
    for (g, m) in MASKS.iter().enumerate() {
        let ox = g as u32 % COLUMNS * TILE_WIDTH + PAD;
        let oy = g as u32 / COLUMNS * TILE_HEIGHT + PAD;
        let tile = is_tile(CHARACTERS[g]);
        for y in -(PAD as i32)..(GLYPH_HEIGHT + PAD) as i32 {
            for x in -(PAD as i32)..(GLYPH_WIDTH + PAD) as i32 {
                if tile || (x >= 0 && y >= 0 && x < GLYPH_WIDTH as i32 && y < GLYPH_HEIGHT as i32) {
                    let sx = x.clamp(0, GLYPH_WIDTH as i32 - 1) as u32;
                    let sy = y.clamp(0, GLYPH_HEIGHT as i32 - 1) as u32;
                    let dst = (((oy as i32 + y) as u32 * ATLAS_WIDTH + (ox as i32 + x) as u32) * 4
                        + 3) as usize;
                    pixels[dst] = m[(sy * GLYPH_WIDTH + sx) as usize];
                }
            }
        }
    }
    pixels[3] = 255;
    pixels
}
