use ditto::{core::*, font, project};
use std::sync::Arc;
fn painted(g: font::Glyph) -> Cell {
    Cell {
        glyph: g,
        fg: [7, 23, 251],
        bg: Some([40, 3, 90]),
    }
}
#[test]
fn interpolated_stroke_is_one_reversible_gesture() {
    let mut e = Editor::new(Document::new(40, 20).unwrap());
    let b = Brush {
        cell: painted(219),
        ..Brush::default()
    };
    e.begin();
    for (a, z) in [((0, 0), (9, 4)), ((9, 4), (30, 17))] {
        e.paint(&line(a, z), b, false);
    }
    e.commit();
    let after = e.document.clone();
    assert!(after.cells.iter().filter(|c| c.glyph == 219).count() >= 31);
    assert!(e.undo());
    assert!(e.document.cells.iter().all(|c| *c == Cell::default()));
    assert!(e.redo());
    assert_eq!(e.document, after);
    assert!(!e.redo());
}
#[test]
fn brush_preserves_unselected_channels() {
    let mut e = Editor::new(Document::new(2, 2).unwrap());
    e.document.set(0, 0, painted(65));
    e.begin();
    e.paint(
        &[(0, 0)],
        Brush {
            cell: painted(90),
            glyph: true,
            fg: false,
            bg: false,
        },
        false,
    );
    e.commit();
    assert_eq!(
        e.document.get(0, 0).unwrap(),
        Cell {
            glyph: 90,
            ..painted(65)
        }
    );
}
#[test]
fn flood_respects_selection_and_active_attributes() {
    let mut e = Editor::new(Document::new(6, 4).unwrap());
    e.document.set(
        2,
        1,
        Cell {
            glyph: 88,
            ..Cell::default()
        },
    );
    e.selection = Some(Rect {
        x: 1,
        y: 1,
        w: 3,
        h: 2,
    });
    let b = Brush {
        cell: painted(219),
        glyph: false,
        fg: true,
        bg: false,
    };
    e.begin();
    e.flood((1, 1), b);
    e.commit();
    assert_eq!(e.document.get(2, 1).unwrap().glyph, 88);
    assert_eq!(
        e.document
            .cells
            .iter()
            .filter(|c| c.fg == b.cell.fg)
            .count(),
        6
    );
    assert_eq!(e.document.get(0, 0).unwrap(), Cell::default());
}
#[test]
fn overlapping_move_reads_snapshot() {
    let mut e = Editor::new(Document::new(6, 1).unwrap());
    for (i, g) in b"ABCDE".iter().enumerate() {
        e.document.set(i as i32, 0, painted(*g as font::Glyph));
    }
    let before = e.document.clone();
    e.selection = Some(Rect {
        x: 0,
        y: 0,
        w: 4,
        h: 1,
    });
    assert!(e.move_selection((1, 0)));
    assert_eq!(e.document.text(None), " ABCD ");
    e.undo();
    assert_eq!(e.document, before);
}
#[test]
fn cancelled_gesture_and_branch_revision() {
    let mut e = Editor::new(Document::new(3, 1).unwrap());
    e.begin();
    e.paint(&[(0, 0)], Brush::default(), false);
    e.cancel();
    assert!(!e.dirty());
    e.edit(|d| d.set(0, 0, painted(65)));
    e.saved_revision = e.revision;
    e.edit(|d| d.set(1, 0, painted(66)));
    assert!(e.dirty());
    e.undo();
    assert!(!e.dirty());
    e.edit(|d| d.set(2, 0, painted(67)));
    assert!(!e.redo());
    assert!(e.dirty());
}
#[test]
fn resize_undo_recovers_cropped_cells() {
    let mut e = Editor::new(Document::new(20, 20).unwrap());
    e.document.set(19, 19, painted(254));
    let before = e.document.clone();
    e.edit(|d| d.resize(2, 3).unwrap());
    assert_eq!(e.document.cells.len(), 6);
    e.undo();
    assert_eq!(e.document, before);
    e.redo();
    assert_eq!(e.document.cells.len(), 6);
}
#[test]
fn text_cp437_normalizes_accents_and_rejects_without_partial_paste() {
    assert_eq!(font::parse_text("e\u{301}").unwrap(), vec![vec![130]]);
    let b = Block::from_text("A\tB\r\nC", Brush::default()).unwrap();
    assert_eq!((b.width, b.height), (5, 2));
    assert_eq!(b.cells[4].glyph, b'B' as font::Glyph);
    assert!(Block::from_text("ab🙂", Brush::default()).is_err());
    for i in 0..=255 {
        assert!(!font::character(i).is_control());
    }
}
#[test]
fn copy_preserves_shape_and_trailing_spaces() {
    let mut d = Document::new(3, 2).unwrap();
    d.set(0, 0, painted(218));
    d.set(1, 0, painted(196));
    assert_eq!(d.text(None), "┌─ \n   ");
    assert_eq!(
        d.text(Some(Rect {
            x: 1,
            y: 0,
            w: 2,
            h: 1
        })),
        "─ "
    );
}
fn referenced_doc() -> Document {
    let mut d = Document::new(3, 2).unwrap();
    d.set(0, 0, painted(219));
    d.set(
        1,
        0,
        Cell {
            glyph: 32,
            fg: [0, 0, 0],
            bg: Some([255, 0, 255]),
        },
    );
    let asset = Arc::new(Asset {
        width: 2,
        height: 2,
        rgba: vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 128, 0, 0, 0, 0],
    });
    let mut r = Reference::fit(asset, 3, 2, false);
    r.x = 0.25;
    r.opacity = 0.61;
    r.locked = false;
    d.reference = Some(r);
    d
}
#[test]
fn portable_project_roundtrip_and_png_excludes_reference() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.ditto");
    let b = tmp.path().join("moved.ditto");
    let d = referenced_doc();
    project::save(&a, &d).unwrap();
    std::fs::rename(&a, &b).unwrap();
    let loaded = project::load(&b).unwrap();
    assert_eq!(loaded, d);
    let im = project::render_png(&loaded, 1, None).unwrap();
    assert_eq!(im.dimensions(), (24, 32));
    assert_eq!(im.get_pixel(20, 24).0, [0, 0, 0, 0]);
    assert_eq!(im.get_pixel(10, 4).0, [255, 0, 255, 255]);
    assert_eq!(im.get_pixel(4, 4).0, [7, 23, 251, 255]);
    let mut no_ref = loaded.clone();
    no_ref.reference = None;
    assert_eq!(im, project::render_png(&no_ref, 1, None).unwrap());
}
#[test]
fn png_integer_scaling_and_solid_background() {
    let d = referenced_doc();
    let a = project::render_png(&d, 1, Some([3, 5, 7])).unwrap();
    let b = project::render_png(&d, 2, Some([3, 5, 7])).unwrap();
    assert_eq!(b.dimensions(), (a.width() * 2, a.height() * 2));
    assert_eq!(a.get_pixel(21, 31).0, [3, 5, 7, 255]);
    assert_eq!(b.get_pixel(42, 62).0, [3, 5, 7, 255]);
    assert_eq!(a.get_pixel(10, 4).0, [255, 0, 255, 255]);
    assert_eq!(b.get_pixel(20, 8).0, [255, 0, 255, 255]);
    assert!(project::render_png(&d, 3, None).is_err());
}
#[test]
fn failed_write_preserves_previous_file() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("project.ditto");
    std::fs::write(&path, b"original").unwrap();
    let r = project::atomic_write(&path, |f| {
        use std::io::Write;
        f.write_all(b"partial")?;
        anyhow::bail!("simulated disk failure")
    });
    assert!(r.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"original");
    assert_eq!(std::fs::read_dir(t.path()).unwrap().count(), 1);
}
#[test]
fn malformed_project_and_dimensions_are_rejected() {
    assert!(Document::new(0, 3).is_err());
    assert!(Document::new(513, 3).is_err());
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("bad.ditto");
    std::fs::write(&p, b"bad").unwrap();
    assert!(project::load(&p).is_err());
    let mut d = referenced_doc();
    d.reference.as_mut().unwrap().scale = f32::NAN;
    assert!(project::save(&p, &d).is_err());
}
#[test]
fn glyph_bitmap_has_connected_box_edges() {
    for y in 0..16 {
        assert_eq!(font::bit(196, 0, y), font::bit(196, 15, y));
    }
    for x in 0..16 {
        assert_eq!(font::bit(179, x, 0), font::bit(179, x, 15));
    }
    assert!(font::bit(219, 0, 0));
    assert!((0..16).all(|y| (0..16).all(|x| !font::bit(32, x, y))));
}
