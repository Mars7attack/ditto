//! Inspect every mapping in the classic ASCII+ presets through the PNG pipeline.
use ditto::{
    charset,
    core::{Cell, Document},
    font, project,
};

fn main() -> anyhow::Result<()> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "validation/runtime/classic-charsets".into()),
    );
    std::fs::create_dir_all(&dir)?;
    for (index, stem) in [(0, "contours"), (1, "textures")] {
        let set = &charset::ALL[index];
        let mut sheet = Document::new(72, set.banks() as u32 * 4).map_err(anyhow::Error::msg)?;
        for cell in std::sync::Arc::make_mut(&mut sheet.cells) {
            cell.bg = Some([20, 19, 24]);
        }
        let mut mapping = format!("{} : {} formes\n", set.name, set.targets.len());
        for bank in 0..set.banks() {
            let y = bank as i32 * 4;
            let label = format!("{} / {} formes", set.bank_label(bank), set.bank_len(bank));
            mapping.push_str(&format!("\n{label}\n"));
            for (x, c) in label.chars().enumerate() {
                sheet.set(
                    x as i32,
                    y,
                    Cell {
                        glyph: font::glyph(c).unwrap(),
                        fg: [230, 161, 109],
                        bg: Some([20, 19, 24]),
                    },
                );
            }
            for (slot, key) in charset::KEYS.iter().enumerate().take(set.bank_len(bank)) {
                let glyph = set.at(bank, slot).unwrap();
                mapping.push_str(&format!("{key} -> {}\n", font::character(glyph)));
                for (row, glyph, color) in [
                    (1, font::glyph(*key).unwrap(), [230, 161, 109]),
                    (2, glyph, [220, 217, 226]),
                ] {
                    sheet.set(
                        slot as i32 * 2,
                        y + row,
                        Cell {
                            glyph,
                            fg: color,
                            bg: Some([20, 19, 24]),
                        },
                    );
                }
            }
        }
        let path = dir.join(format!("{stem}.ditto"));
        project::save(&path, &sheet)?;
        anyhow::ensure!(project::load(&path)? == sheet);
        project::export_png(&dir.join(format!("{stem}.png")), &sheet, 2, None)?;
        std::fs::write(dir.join(format!("{stem}-mapping.txt")), mapping)?;
        println!(
            "{}: {} formes, {} pages",
            set.name,
            set.targets.len(),
            set.banks()
        );
    }
    Ok(())
}
