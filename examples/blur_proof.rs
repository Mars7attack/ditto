//! Visual comparison of blur and selective contour blur through the export pipeline.
use ditto::{
    core::{Cell, Document},
    font, project,
    shaders::{Kind, Layer},
};
use image::{Rgba, RgbaImage};

fn main() -> anyhow::Result<()> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "validation/runtime/blur".into()),
    );
    std::fs::create_dir_all(&dir)?;
    let mut doc = Document::new(24, 10).map_err(anyhow::Error::msg)?;
    for (y, text) in [(1, "SF Mono / Aa 0123"), (3, "() @& !? éà gjQ")] {
        for (x, c) in text.chars().enumerate() {
            doc.set(
                x as i32 + 3,
                y,
                Cell {
                    glyph: font::glyph(c).unwrap(),
                    fg: [230, 205, 180],
                    bg: None,
                },
            );
        }
    }
    for y in 5..7 {
        for x in 3..21 {
            doc.set(
                x,
                y,
                Cell {
                    glyph: font::glyph('█').unwrap(),
                    fg: [120, 210, 200],
                    bg: None,
                },
            );
        }
    }
    for x in 3..21 {
        doc.set(
            x,
            8,
            Cell {
                glyph: font::glyph('▒').unwrap(),
                fg: [115, 135, 150],
                bg: Some([80, 95, 110]),
            },
        );
    }
    let mut comparison = RgbaImage::from_pixel(768 * 3, 704, Rgba([16, 16, 20, 255]));
    for (i, (name, kind)) in [
        ("ORIGINAL", None),
        ("BLUR", Some(Kind::Blur)),
        ("BLUR DES CONTOURS", Some(Kind::ContourBlur)),
    ]
    .into_iter()
    .enumerate()
    {
        doc.shaders.layers = kind.map(Layer::new).into_iter().collect();
        let rendered = project::render_png(&doc, 4, Some([16, 16, 20]))?;
        rendered.save(dir.join(format!("{i}.png")))?;
        project::save(&dir.join(format!("{i}.ditto")), &doc)?;
        anyhow::ensure!(project::load(&dir.join(format!("{i}.ditto")))? == doc);
        let mut label = Document::new(24, 1).map_err(anyhow::Error::msg)?;
        for (x, c) in name.chars().enumerate() {
            label.set(
                x as i32 + 2,
                0,
                Cell {
                    glyph: font::glyph(c).unwrap(),
                    fg: [230, 161, 109],
                    bg: None,
                },
            );
        }
        image::imageops::overlay(
            &mut comparison,
            &project::render_cells(&label, 4, None)?,
            i as i64 * 768,
            0,
        );
        image::imageops::overlay(&mut comparison, &rendered, i as i64 * 768, 64);
        println!("{name}: export x4 + project round-trip OK");
    }
    comparison.save(dir.join("comparison.png"))?;
    Ok(())
}
