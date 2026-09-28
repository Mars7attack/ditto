use ditto::{
    core::{Cell, Document, Rect},
    font,
    text_export::{Export, Format},
};

#[test]
fn pure_braille_exports_use_one_font_for_blank_and_occupied_cells_in_every_format() {
    let mut doc = Document::new(20, 18).unwrap();
    // Every braille pattern, surrounded by ordinary blank cells.
    for bits in 0..256 {
        doc.set(
            bits % 16 + 2,
            bits / 16 + 1,
            Cell {
                glyph: font::glyph(char::from_u32(0x2800 + bits as u32).unwrap()).unwrap(),
                ..Cell::default()
            },
        );
    }
    let source = doc.text(None);
    let expected = source.replace(' ', "\u{2800}");
    let mut export = Export::new(&doc, None);
    assert!(export.braille_compatible && export.align_braille);
    assert_eq!(export.content, source);
    assert_eq!(export.output(), expected);
    for format in Format::ALL {
        export.set_format(format);
        let restored = match format {
            Format::Plain => export.payload().unwrap().to_owned(),
            Format::Markdown => export
                .payload()
                .unwrap()
                .strip_prefix("```\n")
                .unwrap()
                .strip_suffix("\n```")
                .unwrap()
                .to_owned(),
            Format::Discord => export
                .parts
                .as_ref()
                .unwrap()
                .iter()
                .map(|s| {
                    s.strip_prefix("```\n")
                        .unwrap()
                        .strip_suffix("\n```")
                        .unwrap()
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };
        assert_eq!(restored, expected);
        assert_eq!(
            restored.encode_utf16().count(),
            source.encode_utf16().count()
        );
        assert_eq!(
            restored
                .chars()
                .filter(|c| matches!(c, '\u{2801}'..='\u{28ff}'))
                .collect::<String>(),
            source
                .chars()
                .filter(|c| matches!(c, '\u{2801}'..='\u{28ff}'))
                .collect::<String>()
        );
    }
    assert!(export.html().contains(&expected));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("braille.txt");
    export.save(&path).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);
    export.set_braille_alignment(false);
    export.save(&path).unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    assert_eq!(export.content, source);
    assert_eq!(doc.text(None), source);
}

#[test]
fn braille_spacing_is_not_enabled_for_empty_or_mixed_text_and_handles_nbsp() {
    let mut doc = Document::new(4, 2).unwrap();
    assert!(!Export::new(&doc, None).braille_compatible);
    doc.set(
        1,
        0,
        Cell {
            glyph: font::glyph('⣿').unwrap(),
            ..Cell::default()
        },
    );
    doc.set(
        2,
        0,
        Cell {
            glyph: font::glyph('\u{a0}').unwrap(),
            ..Cell::default()
        },
    );
    let export = Export::new(&doc, None);
    assert_eq!(export.output(), "⠀⣿⠀⠀\n⠀⠀⠀⠀");
    doc.set(
        0,
        1,
        Cell {
            glyph: font::glyph('A').unwrap(),
            ..Cell::default()
        },
    );
    let mut export = Export::new(&doc, None);
    assert!(!export.braille_compatible);
    export.set_braille_alignment(true);
    assert!(!export.align_braille);
    assert_eq!(export.output(), doc.text(None));
}

#[test]
fn aligned_braille_budget_counts_characters_not_tripled_utf8_bytes() {
    let mut doc = Document::new(48, 32).unwrap();
    doc.set(
        0,
        0,
        Cell {
            glyph: font::glyph('⣿').unwrap(),
            ..Cell::default()
        },
    );
    let mut export = Export::new(&doc, None);
    export.set_format(Format::Discord);
    assert_eq!(export.parts.as_ref().unwrap().len(), 1);
    assert_eq!(export.payload().unwrap().encode_utf16().count(), 1575);
    assert!(export.payload().unwrap().len() > 4000);
}

#[cfg(target_os = "macos")]
#[test]
fn actual_apple_font_fallback_keeps_every_exported_cell_on_the_same_grid() {
    let mut doc = Document::new(16, 8).unwrap();
    for y in 0..8 {
        for x in y..16 - y {
            doc.set(
                x,
                y,
                Cell {
                    glyph: font::glyph('⣿').unwrap(),
                    ..Cell::default()
                },
            );
        }
    }
    let export = Export::new(&doc, None);
    let temp = tempfile::tempdir().unwrap();
    let dir = std::env::var_os("DITTO_TEXT_LAYOUT_EVIDENCE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| temp.path().to_owned());
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("apple-braille.txt");
    export.save(&path).unwrap();
    let prefix = dir.join("apple-braille-layout");
    let result = std::process::Command::new("swift")
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("scripts/probe-text-layout.swift"),
        )
        .arg(&path)
        .arg(&prefix)
        .args(["Menlo", "22"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(prefix.with_extension("json")).unwrap()).unwrap();
    let glyphs = report["glyphs"].as_array().unwrap();
    assert_eq!(glyphs.len(), 16 * 8);
    let advance = glyphs[0]["advance"].as_f64().unwrap();
    for g in glyphs {
        assert!(
            (g["advance"].as_f64().unwrap() - advance).abs() < 0.0001,
            "Mixed font advances deform the drawing: {g}"
        );
        assert!(
            (g["x"].as_f64().unwrap() - g["utf16_index"].as_f64().unwrap() * advance).abs() < 0.001
        );
    }
}

#[test]
fn utf8_file_roundtrip_preserves_every_cell_space_blank_line_and_selection() {
    let mut doc = Document::new(8, 4).unwrap();
    for (x, ch) in " ⣿⠁<&`".chars().enumerate() {
        doc.set(
            x as i32,
            1,
            Cell {
                glyph: font::glyph(ch).unwrap(),
                ..Cell::default()
            },
        );
    }
    let export = Export::new(&doc, None);
    let expected = "        \n ⣿⠁<&`  \n        \n        ";
    assert_eq!(export.content, expected);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("drawing.txt");
    export.save(&path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), expected.as_bytes());
    let selected = Export::new(
        &doc,
        Some(Rect {
            x: 1,
            y: 1,
            w: 4,
            h: 2,
        }),
    );
    assert_eq!(selected.content, "⣿⠁<&\n    ");
    assert!(export.save(dir.path()).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), expected);
}

#[test]
fn markdown_fences_and_rich_text_protect_symbols_without_rewriting_the_drawing() {
    let doc = Document::new(1, 1).unwrap();
    let mut export = Export::new(&doc, None);
    export.content = "  <img>&\n``` ~~**@everyone**~~\n    ".into();
    export.set_format(Format::Markdown);
    let payload = export.payload().unwrap();
    assert_eq!(
        payload
            .strip_prefix("````\n")
            .unwrap()
            .strip_suffix("\n````")
            .unwrap(),
        export.content
    );
    let html = export.html();
    assert!(html.contains("white-space: pre") && html.contains("monospace"));
    assert!(html.contains("&lt;img&gt;&amp;"));
    assert!(!html.contains("<img>"));
    export.set_format(Format::Discord);
    assert!(export.payload().is_err());
    export.set_format(Format::Plain);
    assert_eq!(export.payload().unwrap(), export.content);
}

#[test]
fn discord_chunks_preserve_rows_with_a_conservative_utf16_budget() {
    let doc = Document::new(1, 1).unwrap();
    let mut export = Export::new(&doc, None);
    export.content = [
        "⣿".repeat(512),
        " ".repeat(512),
        "😀".repeat(512),
        " ".repeat(512),
        "".into(),
    ]
    .join("\n");
    let original = export.content.clone();
    export.set_format(Format::Discord);
    let parts = export.parts.as_ref().unwrap();
    assert!(parts.len() > 1);
    let bodies: Vec<_> = parts
        .iter()
        .map(|p| {
            assert!(p.encode_utf16().count() <= 2000);
            p.strip_prefix("```\n")
                .unwrap()
                .strip_suffix("\n```")
                .unwrap()
        })
        .collect();
    assert_eq!(bodies.join("\n"), original);
    export.change_part(1000);
    assert_eq!(export.part, export.parts.as_ref().unwrap().len() - 1);
    assert_eq!(
        export.preview(),
        export
            .payload()
            .unwrap()
            .strip_prefix("```\n")
            .unwrap()
            .strip_suffix("\n```")
            .unwrap()
    );
    export.change_part(-1000);
    assert_eq!(export.part, 0);
    export.content = "A".repeat(1992);
    export.set_format(Format::Discord);
    assert_eq!(export.payload().unwrap().len(), 2000);
    export.content.push('A');
    export.set_format(Format::Discord);
    assert!(export.payload().is_err());
}
