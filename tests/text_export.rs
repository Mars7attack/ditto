use ditto::{
    core::{Cell, Document, Rect},
    font,
    text_export::{Export, Format},
};

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
