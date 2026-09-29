use ditto::{
    charset,
    core::{Cell, Document},
    font, project,
};
use std::collections::HashSet;
use std::io::{Read, Write};
fn mask(c: char) -> Vec<bool> {
    let g = font::glyph(c).unwrap();
    (0..16)
        .flat_map(|y| (0..16).map(move |x| font::bit(g, x, y)))
        .collect()
}
#[test]
fn asciitor_letters_are_preserved_and_sources_are_only_letters_and_digits() {
    let expected = [
        r#"/\|_-+.,'`()[]{}<>:;^v=~!*"#,
        r#".:;,'`il!|/\_-+=*ox%cO08#@"#,
        "┌┬┐─╔╦╗═╱╲├┼┤│╠╬╣║━┃└┴┘╚╩╝",
        "░▒▓█▀▄▌▐▖▗▘▝▙▛▜▟▚▞▁▂▃▅▆▇▏▎",
    ];
    assert_eq!(charset::KEYS.len(), 36);
    assert_eq!(charset::KEYS.iter().collect::<HashSet<_>>().len(), 36);
    for (set, expected) in charset::ALL.iter().zip(expected) {
        assert_eq!(expected.chars().count(), 26);
        for (key, want) in charset::LETTERS.chars().zip(expected.chars()) {
            assert_eq!(
                font::character(set.resolve(0, key).unwrap()),
                want,
                "{} {key}",
                set.name
            );
            assert_eq!(
                set.resolve(0, key),
                set.resolve(0, key.to_ascii_uppercase())
            );
        }
        for key in "0123456789".chars() {
            assert!(set.resolve(0, key).is_some(), "{} {key}", set.name);
        }
    }
    assert_eq!(charset::ALL[3].targets.len(), 79);
    assert!(charset::KEYS.iter().all(char::is_ascii_alphanumeric));
    for c in "+-=*/@!&éèçàù€".chars() {
        assert_eq!(charset::source(&c.to_string()), None);
    }
}
#[test]
fn classic_extensions_preserve_ascii_keys_and_keep_families_on_separate_pages() {
    for (set, old_start, extra_counts) in [
        (&charset::ALL[0], r#"/\|_-+.,'`()[]{}<>:;^v=~!*"#, [22, 25]),
        (&charset::ALL[1], r#".:;,'`il!|/\_-+=*ox%cO08#@"#, [14, 31]),
    ] {
        let mut original: Vec<char> = old_start.chars().collect();
        for c in '!'..='~' {
            if !original.contains(&c) {
                original.push(c);
            }
        }
        assert_eq!(original.len(), 94);
        // Includes the final partial bank: additions must not fill its free keys.
        for (i, c) in original.into_iter().enumerate() {
            let bank = i / charset::KEYS.len();
            let key = charset::KEYS[i % charset::KEYS.len()];
            assert_eq!(set.resolve(bank, key).map(font::character), Some(c));
        }
        assert_eq!(set.bank_len(2), 22);
        assert_eq!(set.resolve(2, 'c'), None);
        assert_eq!(set.banks(), 5);
        for (bank, count) in extra_counts.into_iter().enumerate() {
            assert_eq!(set.bank_len(bank + 3), count);
            for slot in 0..count {
                let glyph = set.at(bank + 3, slot).unwrap();
                assert!(!font::character(glyph).is_ascii());
                assert!(ditto::typeface::mask(glyph).iter().any(|alpha| *alpha > 0));
            }
        }
    }
    for (set, bank, key, glyph) in [
        (0, 3, 'p', '╭'),
        (0, 4, 'h', '←'),
        (1, 3, 'e', '•'),
        (1, 4, 'a', '░'),
        (1, 4, 'e', '▓'),
    ] {
        assert_eq!(
            charset::ALL[set].resolve(bank, key).map(font::character),
            Some(glyph)
        );
    }
}
#[test]
fn every_charset_target_is_reachable_in_its_banks() {
    for set in charset::ALL.iter() {
        let reachable: Vec<_> = (0..set.banks())
            .flat_map(|b| (0..charset::KEYS.len()).filter_map(move |k| set.at(b, k)))
            .collect();
        assert_eq!(reachable, set.targets);
        assert_eq!(
            reachable.iter().collect::<HashSet<_>>().len(),
            reachable.len()
        );
        assert_eq!(set.at(set.banks(), 0), None);
        for b in 0..set.banks() {
            assert_eq!(set.at(b, set.bank_len(b)), None);
            assert_eq!(set.at(b, charset::KEYS.len()), None);
        }
        assert!(reachable.iter().all(|g| font::valid(*g)));
    }
    assert_eq!(charset::ALL[5].banks(), 6);
}
#[test]
fn curated_braille_covers_four_heights_widths_and_mirrored_directions() {
    let set = &charset::ALL[5];
    assert_eq!(set.targets.len(), 156);
    let rows = |g| {
        let bits = font::character(g) as u32 - 0x2800;
        [0, 1, 2, 6]
            .into_iter()
            .zip([3, 4, 5, 7])
            .map(|(l, r)| ((bits >> l) & 1) as u8 | (((bits >> r) & 1) << 1) as u8)
            .collect::<Vec<_>>()
    };
    for (bank, count) in [12, 25, 30, 19].into_iter().enumerate() {
        let height = bank + 1;
        assert_eq!(set.bank_len(bank), count);
        let patterns: HashSet<_> = (0..count).map(|i| rows(set.at(bank, i).unwrap())).collect();
        for pattern in &patterns {
            let first = pattern.iter().position(|r| *r != 0).unwrap();
            let last = pattern.iter().rposition(|r| *r != 0).unwrap();
            assert_eq!(last - first + 1, height);
            assert!(
                pattern[first..=last].iter().all(|r| *r != 0),
                "spaced motifs belong on their own page"
            );
            let mirror = pattern
                .iter()
                .map(|r| ((r & 1) << 1) | ((r & 2) >> 1))
                .collect::<Vec<_>>();
            assert!(
                patterns.contains(&mirror),
                "missing horizontal mirror: {pattern:?}"
            );
            let mut vertical = pattern.clone();
            vertical[first..=last].reverse();
            assert!(
                patterns.contains(&vertical),
                "missing vertical mirror: {pattern:?}"
            );
        }
        for (key, width) in [('a', 1), ('z', 2), ('e', 3)] {
            assert_eq!(
                rows(set.resolve(bank, key).unwrap()),
                [vec![0; 4 - height], vec![width; height]].concat()
            );
        }
    }
    for (key, character) in [('e', '⣤'), ('k', '⡤'), ('l', '⢤'), ('m', '⣄'), ('w', '⣠')] {
        let g = set.resolve(1, key).unwrap();
        assert_eq!(font::character(g), character);
        assert_eq!(
            &rows(g)[..2],
            &[0, 0],
            "{key} must sit on the bottom two dot rows"
        );
    }
    // Corner-only L shapes: full three-dot column plus its end-row neighbor.
    for (key, character, pattern) in [
        ('c', '⡖', [0, 3, 1, 1]),
        ('v', '⢲', [0, 3, 2, 2]),
        ('b', '⣆', [0, 1, 1, 3]),
        ('n', '⣰', [0, 2, 2, 3]),
        ('0', '⠏', [3, 1, 1, 0]),
        ('1', '⠹', [3, 2, 2, 0]),
        ('2', '⠧', [1, 1, 3, 0]),
        ('3', '⠼', [2, 2, 3, 0]),
    ] {
        let glyph = set.resolve(2, key).unwrap();
        assert_eq!(font::character(glyph), character);
        assert_eq!(rows(glyph), pattern);
        assert_eq!(pattern.iter().map(|r| r.count_ones()).sum::<u32>(), 4);
    }
    for (key, character) in [('h', '⣷'), ('j', '⣾'), ('k', '⡿'), ('l', '⢿')] {
        let g = set.resolve(3, key).unwrap();
        assert_eq!(font::character(g), character);
        assert_eq!(rows(g).iter().map(|r| r.count_ones()).sum::<u32>(), 7);
    }
}
#[test]
fn spaced_braille_groups_both_heights_and_all_upper_lower_blocks() {
    let set = &charset::ALL[5];
    let rows = |g| {
        let bits = font::character(g) as u32 - 0x2800;
        [0, 1, 2, 6]
            .into_iter()
            .zip([3, 4, 5, 7])
            .map(|(l, r)| ((bits >> l) & 1) as u8 | (((bits >> r) & 1) << 1) as u8)
            .collect::<Vec<_>>()
    };
    let lines: HashSet<_> = (0..set.bank_len(4))
        .map(|i| rows(set.at(4, i).unwrap()))
        .collect();
    let blocks: HashSet<_> = (0..set.bank_len(5))
        .map(|i| rows(set.at(5, i).unwrap()))
        .collect();
    assert_eq!(lines.len(), 34);
    assert_eq!(blocks.len(), 36);
    assert!(lines.is_disjoint(&blocks));
    for height in [3, 4] {
        for top in 1..=3 {
            for bottom in 1..=3 {
                let mut pattern = vec![0; 4];
                pattern[4 - height] = top;
                pattern[3] = bottom;
                assert!(
                    lines.contains(&pattern),
                    "missing spaced lines: {pattern:?}"
                );
            }
        }
    }
    let stepped = [
        vec![0, 1, 2, 3],
        vec![0, 2, 1, 3],
        vec![0, 3, 2, 1],
        vec![0, 3, 1, 2],
        vec![1, 2, 3, 0],
        vec![2, 1, 3, 0],
        vec![3, 2, 1, 0],
        vec![3, 1, 2, 0],
        vec![0, 1, 3, 1],
        vec![0, 2, 3, 2],
        vec![1, 3, 1, 0],
        vec![2, 3, 2, 0],
    ];
    for (key, want) in [
        ('l', '⣢'),
        ('m', '⣔'),
        ('w', '⡲'),
        ('x', '⢖'),
        ('c', '⠵'),
        ('v', '⠮'),
        ('b', '⠝'),
        ('n', '⠫'),
    ] {
        assert_eq!(font::character(set.resolve(4, key).unwrap()), want);
    }
    let mut positions = Vec::new();
    for (row, mask) in rows(set.resolve(4, 'l').unwrap()).iter().enumerate() {
        for column in 0..2 {
            if *mask & (1 << column) != 0 {
                positions.push(row * 2 + column + 1);
            }
        }
    }
    assert_eq!(
        positions,
        [3, 6, 7, 8],
        "use the user's row-major dot numbering"
    );
    for pattern in &stepped {
        assert!(lines.contains(pattern));
        assert_eq!(pattern.iter().map(|r| r.count_ones()).sum::<u32>(), 4);
    }
    for (key, want, pattern) in [
        ('0', '⡦', vec![0, 1, 3, 1]),
        ('1', '⢴', vec![0, 2, 3, 2]),
        ('2', '⠗', vec![1, 3, 1, 0]),
        ('3', '⠺', vec![2, 3, 2, 0]),
    ] {
        let glyph = set.resolve(4, key).unwrap();
        assert_eq!(font::character(glyph), want);
        assert_eq!(rows(glyph), pattern);
        assert!(lines.contains(&pattern));
    }
    let base = rows(set.resolve(4, '0').unwrap());
    let positions: Vec<_> = (1..=8)
        .filter(|position| base[(position - 1) / 2] & (1 << ((position - 1) % 2)) != 0)
        .collect();
    assert_eq!(positions, [3, 5, 6, 7]);
    let alternating = [
        vec![0, 1, 2, 1],
        vec![0, 2, 1, 2],
        vec![1, 2, 1, 0],
        vec![2, 1, 2, 0],
    ];
    for ((key, want), pattern) in [('4', '⡢'), ('5', '⢔'), ('6', '⠕'), ('7', '⠪')]
        .into_iter()
        .zip(&alternating)
    {
        let glyph = set.resolve(4, key).unwrap();
        assert_eq!(font::character(glyph), want);
        assert_eq!(&rows(glyph), pattern);
        assert!(lines.contains(pattern));
        assert_eq!(pattern.iter().map(|r| r.count_ones()).sum::<u32>(), 3);
    }
    let base = rows(set.resolve(4, '4').unwrap());
    let positions: Vec<_> = (1..=8)
        .filter(|position| base[(position - 1) / 2] & (1 << ((position - 1) % 2)) != 0)
        .collect();
    assert_eq!(positions, [3, 6, 7]);
    for top in 1u8..=3 {
        for bottom in 1u8..=3 {
            if top.count_ones() + bottom.count_ones() < 3 {
                continue;
            }
            for line in 1..=3 {
                let mut pattern = vec![top, bottom, 0, line];
                assert!(
                    blocks.contains(&pattern),
                    "missing upper block: {pattern:?}"
                );
                pattern.reverse();
                assert!(
                    blocks.contains(&pattern),
                    "missing lower block: {pattern:?}"
                );
            }
        }
    }
    let corners = [vec![3, 1, 2, 3], vec![3, 2, 1, 3]];
    for (key, want) in [('4', '⣫'), ('5', '⣝')] {
        assert_eq!(font::character(set.resolve(5, key).unwrap()), want);
    }
    for corner in &corners {
        assert!(blocks.contains(corner));
        assert_eq!(corner[..2].iter().map(|r| r.count_ones()).sum::<u32>(), 3);
        assert_eq!(corner[2..].iter().map(|r| r.count_ones()).sum::<u32>(), 3);
    }
    let notched = [
        vec![3, 1, 2, 2],
        vec![3, 2, 1, 1],
        vec![2, 2, 1, 3],
        vec![1, 1, 2, 3],
    ];
    for (key, want) in [('6', '⢫'), ('7', '⡝'), ('8', '⣜'), ('9', '⣣')] {
        assert_eq!(font::character(set.resolve(5, key).unwrap()), want);
    }
    let original_bits = font::character(set.resolve(5, '4').unwrap()) as u32 - 0x2800;
    let notched_bits = font::character(set.resolve(5, '6').unwrap()) as u32 - 0x2800;
    assert_eq!(
        original_bits ^ notched_bits,
        1 << 6,
        "only the bottom-left point may change"
    );
    for pattern in &notched {
        assert!(blocks.contains(pattern));
        assert_eq!(pattern.iter().map(|r| r.count_ones()).sum::<u32>(), 5);
    }
    for (bank, patterns) in [(4, lines), (5, blocks)] {
        assert!(set.bank_label(bank).starts_with("Espacés"));
        assert!(set.bank_len(bank) <= charset::KEYS.len());
        for pattern in &patterns {
            let first = pattern.iter().position(|r| *r != 0).unwrap();
            let last = pattern.iter().rposition(|r| *r != 0).unwrap();
            assert!(
                pattern[first..=last].contains(&0)
                    || (bank == 4 && (stepped.contains(pattern) || alternating.contains(pattern)))
                    || (bank == 5 && (corners.contains(pattern) || notched.contains(pattern)))
            );
            let mirror: Vec<_> = pattern
                .iter()
                .map(|r| ((r & 1) << 1) | ((r & 2) >> 1))
                .collect();
            assert!(patterns.contains(&mirror));
            let mut vertical = pattern.clone();
            vertical[first..=last].reverse();
            assert!(patterns.contains(&vertical));
        }
    }
}
#[test]
fn old_cp437_bitmaps_are_exactly_preserved() {
    for g in 0..256 {
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(
                    font::bit(g, x, y),
                    font::BITMAP[g as usize * 16 + y as usize] & (0x80 >> (x / 2)) != 0
                );
            }
        }
    }
}
#[test]
fn catalogue_has_complete_blocks_sextants_and_braille() {
    let blocks = font::block_glyphs();
    for cp in (0x2580..=0x259f)
        .chain(0x1fb00..=0x1fb3b)
        .chain(0x2800..=0x28ff)
    {
        let g = font::glyph(char::from_u32(cp).unwrap()).unwrap();
        assert!(blocks.contains(&g));
    }
    assert_eq!(blocks.len(), 397);
    assert_eq!(font::CHARS.len(), 665);
}
#[test]
fn fractional_blocks_quadrants_and_sextants_have_correct_geometry() {
    for (c, count) in [
        ('▁', 32),
        ('▂', 64),
        ('▃', 96),
        ('▅', 160),
        ('▆', 192),
        ('▇', 224),
        ('▏', 32),
        ('▎', 64),
        ('▉', 224),
        ('▘', 64),
        ('▚', 128),
        ('▟', 192),
    ] {
        assert_eq!(mask(c).iter().filter(|x| **x).count(), count, "{c}");
    }
    let ul = mask('▘');
    assert!(ul[0]);
    assert!(!ul[8]);
    assert!(!ul[8 * 16]);
    let lr = mask('▗');
    assert!(!lr[0]);
    assert!(lr[15 * 16 + 15]);
    let first = mask('\u{1fb00}');
    assert!(first[0]);
    assert!(!first[8]);
    assert!(!first[6 * 16]);
    let sextants: HashSet<_> = (0x1fb00..=0x1fb3b)
        .map(|c| mask(char::from_u32(c).unwrap()))
        .collect();
    assert_eq!(sextants.len(), 60);
}
#[test]
fn braille_bits_and_shading_are_distinct() {
    let blank = mask('\u{2800}');
    assert!(!blank.iter().any(|b| *b));
    let dot1 = mask('\u{2801}');
    assert!(dot1[16 + 3]);
    assert!(!dot1[16 + 11]);
    let dot4 = mask('\u{2808}');
    assert!(dot4[16 + 11]);
    assert!(!dot4[16 + 3]);
    let dot7 = mask('\u{2840}');
    assert!(dot7[13 * 16 + 3]);
    let patterns: HashSet<_> = (0x2800..=0x28ff)
        .map(|c| mask(char::from_u32(c).unwrap()))
        .collect();
    assert_eq!(patterns.len(), 256);
    assert_ne!(mask('\u{1fb95}'), mask('\u{1fb96}'));
    assert!(
        mask('▒')
            .iter()
            .zip(mask('\u{1fb90}'))
            .all(|(a, b)| *a != b)
    );
}
#[test]
fn legacy_geometry_atlas_still_matches_its_masks() {
    let atlas = font::atlas_pixels();
    for g in 0..font::CHARS.len() {
        let ox = g as u32 % font::ATLAS_COLUMNS * 16;
        let oy = g as u32 / font::ATLAS_COLUMNS * 16;
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(
                    atlas[(((oy + y) * font::ATLAS_SIDE + ox + x) * 4 + 3) as usize],
                    if font::bit(g as font::Glyph, x, y) {
                        255
                    } else {
                        0
                    }
                );
            }
        }
    }
}
fn read_manifest(path: &std::path::Path) -> serde_json::Value {
    let mut z = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut s = String::new();
    z.by_name("manifest.json")
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    serde_json::from_str(&s).unwrap()
}
#[test]
fn old_and_extended_cells_roundtrip_in_sfmono_projects() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("test.ditto");
    let mut doc = Document::new(4, 1).unwrap();
    doc.set(
        0,
        0,
        Cell {
            glyph: 219,
            ..Cell::default()
        },
    );
    project::save(&p, &doc).unwrap();
    let m = read_manifest(&p);
    assert_eq!(m["version"], 6);
    assert_eq!(m["profile"], ditto::typeface::PROFILE);
    assert_eq!(doc, project::load(&p).unwrap());
    for (i, c) in ['▘', '▇', '\u{1fb02}', '\u{28ff}'].into_iter().enumerate() {
        doc.set(
            i as i32,
            0,
            Cell {
                glyph: font::glyph(c).unwrap(),
                fg: [30, 100, 200],
                bg: None,
            },
        );
    }
    project::save(&p, &doc).unwrap();
    let m = read_manifest(&p);
    assert_eq!(m["version"], 6);
    assert_eq!(m["profile"], ditto::typeface::PROFILE);
    assert_eq!(doc, project::load(&p).unwrap());
    assert_eq!(doc.text(None), "▘▇\u{1fb02}\u{28ff}");
    let image = project::render_png(&doc, 1, None).unwrap();
    assert_eq!(image.dimensions(), (32, 16));
    assert!(image.pixels().any(|p| p[3] > 0));
    assert!(image.pixels().any(|p| p[3] == 0));
}
#[test]
fn unknown_ids_and_extended_glyphs_in_v1_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("bad.ditto");
    let mut doc = Document::new(1, 1).unwrap();
    doc.set(
        0,
        0,
        Cell {
            glyph: u16::MAX,
            ..Cell::default()
        },
    );
    assert!(project::save(&p, &doc).is_err());
    let f = std::fs::File::create(&p).unwrap();
    let mut z = zip::ZipWriter::new(f);
    let o = zip::write::SimpleFileOptions::default();
    z.start_file("manifest.json", o).unwrap();
    write!(z,"{{\"format\":\"ditto\",\"version\":1,\"profile\":\"{}\",\"width\":1,\"height\":1,\"palette\":[[0,0,0]],\"reference\":null}}",font::LEGACY_PROFILE).unwrap();
    z.start_file("drawing.json", o).unwrap();
    z.write_all(b"[{\"glyph\":256,\"fg\":[0,0,0],\"bg\":null}]")
        .unwrap();
    z.finish().unwrap();
    assert!(project::load(&p).is_err());
}
