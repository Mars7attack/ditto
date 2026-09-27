//! Direct letter/digit mappings. The event router also accepts the bare number row.
use crate::font::{self, Glyph};
use std::sync::LazyLock;
pub const LETTERS: &str = "azertyuiopqsdfghjklmwxcvbn";
pub const SOURCES: &str = "azertyuiopqsdfghjklmwxcvbn0123456789";
pub static KEYS: LazyLock<Vec<char>> = LazyLock::new(|| SOURCES.chars().collect());
#[derive(Debug)]
pub struct Charset {
    pub id: &'static str,
    pub name: &'static str,
    pub targets: Vec<Glyph>,
    banks: Vec<(std::ops::Range<usize>, Option<&'static str>)>,
}
impl Charset {
    pub fn banks(&self) -> usize {
        self.banks.len()
    }
    pub fn at(&self, bank: usize, slot: usize) -> Option<Glyph> {
        let (range, _) = self.banks.get(bank)?;
        (slot < range.len() && slot < KEYS.len()).then(|| self.targets[range.start + slot])
    }
    pub fn bank_len(&self, bank: usize) -> usize {
        self.banks.get(bank).map_or(0, |(r, _)| r.len())
    }
    pub fn bank_label(&self, bank: usize) -> String {
        match self.banks.get(bank).and_then(|(_, name)| *name) {
            Some(name) => name.into(),
            None => format!("Banque {}/{}", bank + 1, self.banks()),
        }
    }
    pub fn resolve(&self, bank: usize, key: char) -> Option<Glyph> {
        let key = key.to_ascii_lowercase();
        KEYS.iter()
            .position(|k| *k == key)
            .and_then(|i| self.at(bank, i))
    }
}
fn make(
    id: &'static str,
    name: &'static str,
    first: &str,
    extra: impl Iterator<Item = char>,
) -> Charset {
    let mut targets = Vec::new();
    for c in first.chars().chain(extra) {
        if let Some(g) = font::glyph(c)
            && !targets.contains(&g)
        {
            targets.push(g);
        }
    }
    let banks = (0..targets.len())
        .step_by(KEYS.len())
        .map(|start| (start..(start + KEYS.len()).min(targets.len()), None))
        .collect();
    Charset {
        id,
        name,
        targets,
        banks,
    }
}

/// One/two columns per occupied row; braille dot numbering is not row-major.
fn braille_rows(start: usize, rows: &[u8]) -> Glyph {
    let left = [0, 1, 2, 6];
    let right = [3, 4, 5, 7];
    let mut bits = 0;
    for (i, row) in rows.iter().enumerate() {
        if row & 1 != 0 {
            bits |= 1 << left[start + i];
        }
        if row & 2 != 0 {
            bits |= 1 << right[start + i];
        }
    }
    font::glyph(char::from_u32(0x2800 + bits).unwrap()).unwrap()
}

fn spaced_braille_lines() -> Vec<Glyph> {
    let mut targets = Vec::new();
    for height in [3, 4] {
        for (top, bottom) in [
            (3, 3),
            (1, 1),
            (2, 2),
            (1, 2),
            (2, 1),
            (1, 3),
            (2, 3),
            (3, 1),
            (3, 2),
        ] {
            let mut rows = [0; 4];
            rows[4 - height] = top;
            rows[3] = bottom;
            targets.push(braille_rows(0, &rows));
        }
    }
    // Row-major positions 3/6/7/8, its mirrors, and their top-aligned variants.
    for start in [1, 0] {
        for rows in [[1, 2, 3], [2, 1, 3], [3, 2, 1], [3, 1, 2]] {
            targets.push(braille_rows(start, &rows));
        }
    }
    // Row-major 3/5/6/7 and its horizontal mirror, bottom then top aligned.
    // Vertical reflection gives the same motif within each three-row span.
    for start in [1, 0] {
        for rows in [[1, 3, 1], [2, 3, 2]] {
            targets.push(braille_rows(start, &rows));
        }
    }
    // Row-major 3/6/7: alternating columns, bottom then top aligned.
    for start in [1, 0] {
        for rows in [[1, 2, 1], [2, 1, 2]] {
            targets.push(braille_rows(start, &rows));
        }
    }
    targets
}

fn spaced_braille_blocks() -> Vec<Glyph> {
    let mut targets = Vec::new();
    // A 2x2 block or any three-dot corner, an empty row, then 1/2 dots.
    // Each upper group is immediately followed by its lower-group counterpart.
    for group in [[3, 3], [3, 1], [3, 2], [1, 3], [2, 3]] {
        for line in [3, 1, 2] {
            let mut rows = [group[0], group[1], 0, line];
            targets.push(braille_rows(0, &rows));
            rows.reverse();
            targets.push(braille_rows(0, &rows));
        }
    }
    // Opposing three-dot corners: the two middle holes are diagonal rather
    // than occupying a whole row. Keep them alongside the other holed blocks.
    for rows in [[3, 1, 2, 3], [3, 2, 1, 3]] {
        targets.push(braille_rows(0, &rows));
    }
    // Key 4 with the bottom-left point removed, then H/V/180-degree mirrors.
    for rows in [[3, 1, 2, 2], [3, 2, 1, 1], [2, 2, 1, 3], [1, 1, 2, 3]] {
        targets.push(braille_rows(0, &rows));
    }
    targets
}

fn braille() -> Charset {
    let mut set = Charset {
        id: "braille",
        name: "Points / braille",
        targets: Vec::new(),
        banks: Vec::new(),
    };
    for (height, name) in ["1 point", "2 points", "3 points", "4 points"]
        .into_iter()
        .enumerate()
    {
        let height = height + 1;
        let start = set.targets.len();
        // A/Z/E always give left/right/double columns, anchored at the bottom.
        let positions: &[usize] = match height {
            1 => &[3, 2, 1, 0],
            2 => &[2, 0, 1],
            3 => &[1, 0],
            _ => &[0],
        };
        for &top in positions {
            for width in 1..=3 {
                set.targets.push(braille_rows(top, &vec![width; height]));
            }
        }
        match height {
            2 => {
                for top in [2, 0] {
                    for rows in [[1, 2], [2, 1]] {
                        set.targets.push(braille_rows(top, &rows));
                    }
                }
                for rows in [[3, 1], [3, 2], [1, 3], [2, 3]] {
                    set.targets.push(braille_rows(1, &rows));
                }
                // Append the missing bottom corners without moving existing keys.
                // Their top mirrors keep both orientations available in this bank.
                for top in [2, 0] {
                    for rows in [[3, 1], [3, 2], [1, 3], [2, 3]] {
                        set.targets.push(braille_rows(top, &rows));
                    }
                }
            }
            3 => {
                for top in [1, 0] {
                    for rows in [
                        [1, 3, 3],
                        [2, 3, 3],
                        [3, 3, 1],
                        [3, 3, 2],
                        [1, 1, 2],
                        [1, 2, 2],
                        [2, 2, 1],
                        [2, 1, 1],
                    ] {
                        set.targets.push(braille_rows(top, &rows));
                    }
                }
            }
            4 => {
                for rows in [
                    [1, 1, 2, 2],
                    [2, 2, 1, 1],
                    [1, 3, 3, 2],
                    [2, 3, 3, 1],
                    [1, 1, 3, 3],
                    [2, 2, 3, 3],
                    [3, 3, 1, 1],
                    [3, 3, 2, 2],
                    [3, 1, 1, 1],
                    [3, 2, 2, 2],
                    [1, 1, 1, 3],
                    [2, 2, 2, 3],
                    // Seven dots: a full column beside three contiguous dots.
                    [1, 3, 3, 3],
                    [2, 3, 3, 3],
                    [3, 3, 3, 1],
                    [3, 3, 3, 2],
                ] {
                    set.targets.push(braille_rows(0, &rows));
                }
            }
            _ => {}
        }
        set.banks.push((start..set.targets.len(), Some(name)));
    }
    for (name, targets) in [
        ("Espacés / lignes", spaced_braille_lines()),
        ("Espacés / blocs", spaced_braille_blocks()),
    ] {
        let start = set.targets.len();
        set.targets.extend(targets);
        set.banks.push((start..set.targets.len(), Some(name)));
    }
    set
}
pub static ALL: LazyLock<Vec<Charset>> = LazyLock::new(|| {
    let blocks = || {
        (0x2580..=0x259f)
            .chain(0x1fb70..=0x1fb9f)
            .filter_map(char::from_u32)
    };
    vec![
        make(
            "contours-ascii",
            "Contours ASCII",
            r#"/\|_-+.,'`()[]{}<>:;^v=~!*"#,
            (0x21..=0x7e).filter_map(char::from_u32),
        ),
        make(
            "texture-ascii",
            "Texture ASCII",
            r#".:;,'`il!|/\_-+=*ox%cO08#@"#,
            (0x21..=0x7e).filter_map(char::from_u32),
        ),
        make(
            "traits-unicode",
            "Traits Unicode",
            "┌┬┐─╔╦╗═╱╲├┼┤│╠╬╣║━┃└┴┘╚╩╝",
            (0x2500..=0x257f)
                .filter_map(char::from_u32)
                .chain(r#"/\|_-=+<>[]{}()^v.,:;*#"#.chars()),
        ),
        make(
            "blocks",
            "Blocs et ombrages",
            "░▒▓█▀▄▌▐▖▗▘▝▙▛▜▟▚▞▁▂▃▅▆▇▏▎",
            blocks(),
        ),
        make(
            "sextants",
            "Mosaïques / sextants",
            "",
            (0x1fb00..=0x1fb3b)
                .filter_map(char::from_u32)
                .chain("█▌▐▀▄▖▗▘▝▙▛▜▟▚▞░▒▓■".chars()),
        ),
        braille(),
    ]
});
pub fn source(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    KEYS.contains(&c.to_ascii_lowercase())
        .then_some(c.to_ascii_lowercase())
}
