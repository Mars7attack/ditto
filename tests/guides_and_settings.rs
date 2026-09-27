use ditto::{core::*, font, guides, project, settings};
use std::sync::Arc;

#[test]
fn recolor_preserves_glyphs_backgrounds_and_empty_cells_and_respects_selection() {
    let mut d = Document::new(12, 6).unwrap();
    for x in 1..10 {
        d.set(
            x,
            2,
            Cell {
                glyph: 65 + x as u16,
                fg: [10, 20, 30],
                bg: Some([4, 5, 6]),
            },
        );
    }
    d.set(
        6,
        2,
        Cell {
            bg: Some([9, 8, 7]),
            ..Cell::default()
        },
    );
    d.set(
        7,
        2,
        Cell {
            glyph: font::glyph('\u{2800}').unwrap(),
            ..Cell::default()
        },
    );
    let original = d.clone();
    let mut e = Editor::new(d);
    e.selection = Some(Rect {
        x: 3,
        y: 1,
        w: 5,
        h: 3,
    });
    e.begin();
    e.recolor(&line((0, 2), (11, 2)), [225, 60, 90], 2);
    e.commit();
    for y in 0..6 {
        for x in 0..12 {
            let before = original.get(x, y).unwrap();
            let after = e.document.get(x, y).unwrap();
            assert_eq!((before.glyph, before.bg), (after.glyph, after.bg));
            if y == 2 && (3..6).contains(&x) {
                assert_eq!(after.fg, [225, 60, 90]);
            } else {
                assert_eq!(after, before);
            }
        }
    }
    let painted = e.document.clone();
    assert!(e.undo());
    assert_eq!(e.document, original);
    assert!(e.redo());
    assert_eq!(e.document, painted);
}
#[test]
fn guide_history_shares_art_and_restores_mixed_edits() {
    let d = Document::new(512, 512).unwrap();
    let cells = d.cells.clone();
    let mut e = Editor::new(d);
    e.begin();
    e.document.guides.start([1.25, 2.75], [30, 220, 200], 0.4);
    e.document.guides.append([14.2, 10.9]);
    e.commit();
    assert!(Arc::ptr_eq(&cells, &e.document.cells));
    let stroke = e.document.guides.strokes[0].clone();
    e.edit(|d| d.shaders = ditto::shaders::preset(0));
    e.edit(|d| d.guides.visible = false);
    e.edit(|d| {
        d.resize(4, 4).unwrap();
    });
    assert_eq!(e.document.guides.strokes[0].points[1], [14.2, 10.9]);
    for _ in 0..3 {
        assert!(e.undo());
    }
    assert!(e.document.guides.visible);
    assert!(e.document.shaders.layers.is_empty());
    assert!(Arc::ptr_eq(&stroke, &e.document.guides.strokes[0]));
    assert!(Arc::ptr_eq(&cells, &e.document.cells));
    assert!(e.undo());
    assert!(e.document.guides.strokes.is_empty());
    assert!(e.redo());
    assert_eq!(e.document.guides.strokes.len(), 1);
    e.begin();
    e.document.guides.erase([8., 0.], [8., 20.], 0.3);
    assert!(e.document.guides.strokes.is_empty());
    e.cancel();
    assert_eq!(e.document.guides.strokes.len(), 1);
}
#[test]
fn guides_roundtrip_and_never_enter_text_or_raster() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("with-guides.ditto");
    let mut d = Document::new(8, 8).unwrap();
    d.set(
        3,
        3,
        Cell {
            glyph: 65,
            ..Cell::default()
        },
    );
    let before = project::render_png(&d, 2, None).unwrap();
    let text = d.text(None);
    d.guides.start([0.2, 0.3], [255, 0, 0], 2.);
    d.guides.append([7.8, 7.4]);
    d.guides.opacity = 0.4;
    project::save(&p, &d).unwrap();
    assert_eq!(project::load(&p).unwrap(), d);
    assert_eq!(project::render_png(&d, 2, None).unwrap(), before);
    assert_eq!(d.text(None), text);
    d.guides.visible = false;
    project::save(&p, &d).unwrap();
    assert_eq!(project::load(&p).unwrap(), d);
    assert_eq!(project::render_png(&d, 2, None).unwrap(), before);
}
#[test]
fn guide_geometry_validation_limits_and_swept_eraser() {
    let mut l = guides::Layer::default();
    l.start([2., 2.], [0, 0, 0], 0.1);
    l.append([8., 2.]);
    l.erase([0., 2.], [1., 2.], 0.2);
    assert_eq!(l.strokes.len(), 1, "collinear but disjoint");
    l.erase([5., 0.], [5., 4.], 0.1);
    assert!(
        l.strokes.is_empty(),
        "fast crossing hits even without point samples"
    );
    l.start([3., 3.], [0, 0, 0], 1.);
    l.erase([3., 3.], [3., 3.], 0.1);
    assert!(l.strokes.is_empty());
    l.start([1., 1.], [0, 0, 0], 0.4);
    l.opacity = f32::NAN;
    assert!(l.validate().is_err());
    l.opacity = 0.65;
    Arc::make_mut(&mut Arc::make_mut(&mut l.strokes)[0]).points[0][0] = 513.;
    assert!(l.validate().is_err());
    Arc::make_mut(&mut Arc::make_mut(&mut l.strokes)[0]).points =
        vec![[1., 1.]; guides::MAX_STROKE_POINTS];
    assert!(l.validate().is_ok());
    assert!(!l.append([2., 2.]));
    Arc::make_mut(&mut l.strokes).resize(
        guides::MAX_STROKES + 1,
        Arc::new(guides::Stroke {
            points: vec![[0., 0.]],
            width: 1.,
            color: [0, 0, 0],
        }),
    );
    assert!(l.validate().is_err());
    assert!(!l.start([0., 0.], [0, 0, 0], 1.));
}
#[test]
fn preferences_roundtrip_and_invalid_files_do_not_get_overwritten_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("config/settings.json");
    assert_eq!(settings::load(&p).unwrap(), settings::Settings::default());
    let mut s = settings::Settings {
        theme: settings::Theme::preset(2),
        ..Default::default()
    };
    *settings::Token::Accent.slot(&mut s.theme) = [34, 91, 122];
    settings::save(&p, &s).unwrap();
    assert_eq!(settings::load(&p).unwrap(), s);
    let invalid = b"{\"version\":99}";
    std::fs::write(&p, invalid).unwrap();
    assert!(settings::load(&p).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), invalid);
    for i in 0..3 {
        assert!(!settings::Theme::preset(i).low_contrast());
    }
    let mut low = settings::Theme::default();
    low.text = low.panel;
    assert!(low.low_contrast());
}

#[test]
fn project_rejects_corrupt_guides_and_unversioned_layer_data() {
    use std::io::{Read, Write};
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.ditto");
    let mut d = Document::new(4, 4).unwrap();
    d.guides.start([1., 1.], [1, 2, 3], 0.3);
    project::save(&source, &d).unwrap();
    let mut z = zip::ZipArchive::new(std::fs::File::open(&source).unwrap()).unwrap();
    let mut manifest = String::new();
    z.by_name("manifest.json")
        .unwrap()
        .read_to_string(&mut manifest)
        .unwrap();
    let drawing = serde_json::to_vec(&d.cells).unwrap();
    for (i,bad) in [
        serde_json::json!({"visible":true,"opacity":0.5,"strokes":[{"color":[1,2,3],"width":0.5,"points":[]}]}),
        serde_json::json!({"visible":true,"opacity":1.5,"strokes":[]}),
        serde_json::json!({"visible":true,"opacity":0.5,"strokes":[],"unexpected":true}),
        serde_json::to_value(&d.guides).unwrap(),
    ].into_iter().enumerate() {
        let p=dir.path().join(format!("bad-{i}.ditto"));
        let mut w=zip::ZipWriter::new(std::fs::File::create(&p).unwrap());let options=zip::write::SimpleFileOptions::default();
        let mut m:serde_json::Value=serde_json::from_str(&manifest).unwrap();if i==3 {m["version"]=4.into();}
        w.start_file("manifest.json",options).unwrap();serde_json::to_writer(&mut w,&m).unwrap();
        w.start_file("drawing.json",options).unwrap();w.write_all(&drawing).unwrap();
        w.start_file("guides.json",options).unwrap();serde_json::to_writer(&mut w,&bad).unwrap();w.finish().unwrap();
        assert!(project::load(&p).is_err());
    }
    assert_eq!(project::load(&source).unwrap(), d);
}

#[test]
fn v5_guides_without_order_migrate_below_characters_and_save_in_v6() {
    use std::io::{Read, Write};
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("legacy-v5.ditto");
    let mut w = zip::ZipWriter::new(std::fs::File::create(&old).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    w.start_file("manifest.json", options).unwrap();
    serde_json::to_writer(&mut w,&serde_json::json!({"format":"ditto","version":5,"profile":ditto::typeface::PROFILE,"width":1,"height":1,"palette":[[0,0,0]],"reference":null})).unwrap();
    w.start_file("drawing.json", options).unwrap();
    serde_json::to_writer(&mut w, &vec![Cell::default()]).unwrap();
    w.start_file("guides.json", options).unwrap();
    write!(w,r#"{{"visible":true,"opacity":0.65,"strokes":[{{"color":[10,20,30],"width":0.4,"points":[[0.5,0.5]]}}]}}"#).unwrap();
    w.finish().unwrap();
    let mut d = project::load(&old).unwrap();
    assert!(!d.guides.above_characters);
    assert_eq!(d.guides.strokes[0].color, [10, 20, 30]);
    d.guides.above_characters = true;
    d.guides.recolor_all([11, 22, 33]);
    let new = dir.path().join("new.ditto");
    project::save(&new, &d).unwrap();
    assert_eq!(project::load(&new).unwrap(), d);
    let mut z = zip::ZipArchive::new(std::fs::File::open(new).unwrap()).unwrap();
    let mut m = String::new();
    z.by_name("manifest.json")
        .unwrap()
        .read_to_string(&mut m)
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&m).unwrap()["version"],
        6
    );
}
