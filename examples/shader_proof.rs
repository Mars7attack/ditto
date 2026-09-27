//! Reproducible visual proof of the live/export shader pipeline.
use ditto::{
    core::{Cell, Document},
    font, project, shaders,
};
fn main() -> anyhow::Result<()> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "validation/runtime/shaders".into()),
    );
    std::fs::create_dir_all(&dir)?;
    let mut doc = Document::new(64, 30).map_err(anyhow::Error::msg)?;
    let ramp: Vec<char> = "·:░▒▓█".chars().collect();
    for y in 3..27 {
        for x in 5..59 {
            let px = (x as f32 - 31.5) / 24.;
            let py = (y as f32 - 14.5) / 12.;
            let radius = px * px + py * py;
            if radius < 1. {
                let z = (1. - radius).sqrt();
                let brightness = (0.2 + (-px * 0.4 - py * 0.55 + z * 0.7).max(0.)).min(1.);
                let glyph = ramp[(brightness * (ramp.len() - 1) as f32) as usize];
                doc.set(
                    x,
                    y,
                    Cell {
                        glyph: font::glyph(glyph).unwrap(),
                        fg: [
                            (80. + brightness * 165.) as u8,
                            (55. + brightness * 125.) as u8,
                            (95. + brightness * 130.) as u8,
                        ],
                        bg: None,
                    },
                );
            } else if ((px * 0.85).powi(2) + ((py + px * 0.35) * 2.6).powi(2) - 1.).abs() < 0.12 {
                doc.set(
                    x,
                    y,
                    Cell {
                        glyph: font::glyph('▒').unwrap(),
                        fg: [125, 198, 208],
                        bg: None,
                    },
                );
            }
        }
    }
    for (y, text) in [
        (1, "D I T T O   /   S H A D E R S"),
        (28, "GLYPHS > LIGHT > COLOUR > PATTERN"),
    ] {
        for (x, ch) in text.chars().enumerate() {
            doc.set(
                x as i32 + 6,
                y,
                Cell {
                    glyph: font::glyph(ch).unwrap(),
                    fg: [212, 211, 225],
                    bg: None,
                },
            );
        }
    }
    project::export_png(&dir.join("original.png"), &doc, 2, Some([16, 16, 20]))?;
    for (i, name) in shaders::PRESETS.iter().enumerate() {
        doc.shaders = shaders::preset(i);
        project::export_png(
            &dir.join(format!("look-{i}.png")),
            &doc,
            2,
            Some([16, 16, 20]),
        )?;
        println!("{name}: OK");
    }
    doc.shaders = shaders::preset(0);
    project::save(&dir.join("shader-demo.ditto"), &doc)?;
    project::export_png(&dir.join("transparent.png"), &doc, 2, None)?;
    let reloaded = project::load(&dir.join("shader-demo.ditto"))?;
    anyhow::ensure!(reloaded == doc);
    println!("Round-trip V4 + shaders: OK / {}", dir.display());
    Ok(())
}
