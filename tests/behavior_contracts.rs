//! Cross-feature invariants, with deterministic input sweeps and a snapshot
//! oracle independent of the editor's delta-based undo representation.
use ditto::{
    core::*,
    font, project,
    shaders::{self, Kind, Layer},
};
use std::{collections::HashSet, sync::Arc};

#[test]
fn every_brush_mask_preserves_inactive_channels_and_has_correct_flood_boundary() {
    let old = Cell {
        glyph: 65,
        fg: [1, 2, 3],
        bg: Some([4, 5, 6]),
    };
    let new = Cell {
        glyph: 66,
        fg: [7, 8, 9],
        bg: None,
    };
    for mask in 0..8 {
        let brush = Brush {
            cell: new,
            glyph: mask & 1 != 0,
            fg: mask & 2 != 0,
            bg: mask & 4 != 0,
        };
        let c = brush.apply(old);
        assert_eq!(c.glyph, if brush.glyph { new.glyph } else { old.glyph });
        assert_eq!(c.fg, if brush.fg { new.fg } else { old.fg });
        assert_eq!(c.bg, if brush.bg { new.bg } else { old.bg });
        let mut e = Editor::new(Document::new(5, 3).unwrap());
        for y in 0..3 {
            for x in 0..5 {
                e.document.set(x, y, old);
            }
        }
        e.selection = Some(Rect {
            x: 1,
            y: 1,
            w: 3,
            h: 1,
        });
        e.begin();
        e.flood((2, 1), brush);
        e.commit();
        for y in 0..3 {
            for x in 0..5 {
                assert_eq!(
                    e.document.get(x, y).unwrap(),
                    if y == 1 && (1..=3).contains(&x) {
                        c
                    } else {
                        old
                    }
                );
            }
        }
        assert_eq!(e.dirty(), mask != 0);
    }
}

#[test]
fn line_all_octants_are_connected_bounded_and_include_both_endpoints() {
    for x in -16..=16 {
        for y in -16..=16 {
            let pts = line((0, 0), (x, y));
            assert_eq!(pts.first(), Some(&(0, 0)));
            assert_eq!(pts.last(), Some(&(x, y)));
            assert_eq!(pts.len(), x.abs().max(y.abs()) as usize + 1);
            assert_eq!(pts.iter().collect::<HashSet<_>>().len(), pts.len());
            assert!(
                pts.windows(2)
                    .all(|p| (p[0].0 - p[1].0).abs() <= 1 && (p[0].1 - p[1].1).abs() <= 1)
            );
            assert!(pts.iter().all(|&(px, py)| px >= x.min(0)
                && px <= x.max(0)
                && py >= y.min(0)
                && py <= y.max(0)));
        }
    }
}

#[test]
fn rectangles_reversed_single_cell_and_degenerate_edges_have_exact_coverage() {
    for w in 1..=12 {
        for h in 1..=12 {
            for filled in [false, true] {
                let expected: HashSet<_> = (0..h)
                    .flat_map(|y| {
                        (0..w).filter_map(move |x| {
                            (filled || x == 0 || y == 0 || x == w - 1 || y == h - 1)
                                .then_some((x, y))
                        })
                    })
                    .collect();
                for (a, b) in [
                    ((0, 0), (w - 1, h - 1)),
                    ((w - 1, h - 1), (0, 0)),
                    ((0, h - 1), (w - 1, 0)),
                ] {
                    let actual = shape(Tool::Rectangle, a, b, filled);
                    assert_eq!(actual.len(), expected.len());
                    assert_eq!(actual.into_iter().collect::<HashSet<_>>(), expected);
                }
            }
        }
    }
}

#[test]
fn mixed_history_matches_full_snapshots_through_undo_redo_and_branching() {
    let mut e = Editor::new(Document::new(12, 9).unwrap());
    let mut states = vec![e.document.clone()];
    let mut seed = 0x243f6a88u32;
    for step in 0..160 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        e.edit(|d| match step % 6 {
            0 => d.set(
                (seed % d.width) as i32,
                ((seed >> 8) % d.height) as i32,
                Cell {
                    glyph: 65 + (step % 26) as u16,
                    fg: [step as u8, 7, 201],
                    bg: None,
                },
            ),
            1 => d.palette[0] = [step as u8, 3, 99],
            2 => {
                d.guides.start([1., 1.], [step as u8, 2, 3], 0.4);
                d.guides.append([3., 4.]);
            }
            3 => {
                d.shaders = shaders::preset(step % shaders::PRESETS.len());
            }
            4 => {
                d.resize(10 + step as u32 % 3, 8 + step as u32 % 2).unwrap();
            }
            _ => d.guides.above_characters = !d.guides.above_characters,
        });
        if states.last() != Some(&e.document) {
            states.push(e.document.clone());
        }
    }
    for expected in states.iter().rev().skip(1) {
        assert!(e.undo());
        assert_eq!(&e.document, expected);
    }
    assert!(!e.undo());
    assert!(!e.dirty());
    for expected in states.iter().skip(1) {
        assert!(e.redo());
        assert_eq!(&e.document, expected);
    }
    assert!(!e.redo());
    e.undo();
    e.edit(|d| d.palette[0] = [255; 3]);
    assert!(!e.redo());
}

#[test]
fn empty_and_noop_edits_do_not_erase_redo_history() {
    let mut e = Editor::new(Document::new(1, 1).unwrap());
    e.edit(|d| {
        d.set(
            0,
            0,
            Cell {
                glyph: 65,
                ..Cell::default()
            },
        )
    });
    e.undo();
    e.edit(|_| {});
    e.begin();
    e.paint(&[(-1, 0), (1, 0), (0, -1)], Brush::default(), false);
    e.commit();
    assert_eq!(e.revision, 0);
    assert!(e.redo());
    assert_eq!(e.document.get(0, 0).unwrap().glyph, 65);
}

#[test]
fn block_clipping_transparent_background_and_exact_fit() {
    let mut d = Document::new(3, 2).unwrap();
    d.set(
        2,
        1,
        Cell {
            glyph: 219,
            fg: [255, 0, 0],
            bg: None,
        },
    );
    let b = Block::copy(
        &d,
        Rect {
            x: -10,
            y: -10,
            w: 30,
            h: 30,
        },
    );
    assert_eq!((b.width, b.height), (3, 2));
    assert!(b.fits(&d, (0, 0)));
    for p in [(-1, 0), (0, -1), (1, 0), (0, 1)] {
        assert!(!b.fits(&d, p));
    }
    let c = Block::copy(
        &d,
        Rect {
            x: 2,
            y: 1,
            w: 1,
            h: 1,
        },
    );
    c.paste(&mut d, (0, 0));
    assert_eq!(d.get(0, 0), d.get(2, 1));
    assert_eq!(
        Block::copy(
            &d,
            Rect {
                x: 9,
                y: 9,
                w: 1,
                h: 1
            }
        )
        .cells
        .len(),
        0
    );
}

#[test]
fn all_shader_parameter_extremes_and_presets_validate_and_roundtrip() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("preset.ditto-shaders");
    for kind in shaders::KINDS {
        for direction in [-1, 1] {
            let mut layer = Layer::new(kind);
            for _ in 0..500 {
                for i in 0..4 {
                    layer.adjust(i, direction);
                }
            }
            let stack = shaders::Stack {
                layers: vec![layer],
                ..Default::default()
            };
            stack.validate().unwrap();
            shaders::save_preset(&p, &stack).unwrap();
            assert_eq!(shaders::load_preset(&p).unwrap(), stack);
        }
    }
    for i in 0..shaders::PRESETS.len() {
        let stack = shaders::preset(i);
        stack.validate().unwrap();
    }
}

#[test]
fn malformed_document_fields_never_replace_a_valid_project() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("saved.ditto");
    let d = Document::new(4, 3).unwrap();
    project::save(&path, &d).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    for case in 0..8 {
        let mut bad = d.clone();
        match case {
            0 => bad.width = 0,
            1 => bad.height = 513,
            2 => bad.cells = Arc::new(vec![]),
            3 => Arc::make_mut(&mut bad.cells)[0].glyph = u16::MAX,
            4 => bad.palette.clear(),
            5 => bad.palette = vec![[0; 3]; 257],
            6 => bad.guides.opacity = f32::NAN,
            _ => {
                let mut l = Layer::new(Kind::Glow);
                l.mix = f32::NAN;
                bad.shaders.layers.push(l);
            }
        }
        assert!(project::save(&path, &bad).is_err(), "case {case}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(project::load(&path).unwrap(), d);
    }
}

#[test]
fn corrupt_archives_missing_entries_and_truncations_are_rejected() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("project.ditto");
    project::save(&path, &Document::new(2, 2).unwrap()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    for len in [0, 1, 4, bytes.len() / 2, bytes.len() - 1] {
        std::fs::write(&path, &bytes[..len]).unwrap();
        assert!(project::load(&path).is_err());
    }
    let file = std::fs::File::create(&path).unwrap();
    zip::ZipWriter::new(file).finish().unwrap();
    assert!(project::load(&path).is_err());
}

#[test]
fn hex_color_acceptance_and_invalid_inputs() {
    for (s, c) in [
        ("000000", [0; 3]),
        ("#FFFFFF", [255; 3]),
        (" abc123 ", [171, 193, 35]),
    ] {
        assert_eq!(project::color_hex(s).unwrap(), c);
    }
    for s in [
        "",
        "123",
        "1234567",
        "GG0000",
        "🙂0000",
        "#12345",
        "#12345678",
    ] {
        assert!(project::color_hex(s).is_err(), "{s}");
    }
}

#[test]
fn unicode_text_normalization_is_atomic_and_trailing_whitespace_is_preserved() {
    assert!(Block::from_text("A\n🙂", Brush::default()).is_err());
    let b = Block::from_text("é \r\nA  ", Brush::default()).unwrap();
    let mut d = Document::new(b.width, b.height).unwrap();
    b.paste(&mut d, (0, 0));
    assert_eq!(d.text(None), "é  \nA  ");
    assert_eq!(
        font::parse_text("e\u{301}").unwrap(),
        font::parse_text("é").unwrap()
    );
}
