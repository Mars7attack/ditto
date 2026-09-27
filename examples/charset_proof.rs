//! Reproducible visual/export fixture for the extended repertoire.
use ditto::{
    charset,
    core::{Cell, Document},
    font, project,
};
fn main() -> anyhow::Result<()> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "validation/runtime/charsets".into()),
    );
    std::fs::create_dir_all(&dir)?;
    let set = &charset::ALL[3];
    let mut d =
        Document::new(16, set.targets.len().div_ceil(16) as u32).map_err(anyhow::Error::msg)?;
    for (i, g) in set.targets.iter().enumerate() {
        d.set(
            i as i32 % 16,
            i as i32 / 16,
            Cell {
                glyph: *g,
                fg: [220, 215, 224],
                bg: Some(if i % 2 == 0 {
                    [26, 22, 32]
                } else {
                    [37, 31, 44]
                }),
            },
        );
    }
    project::save(&dir.join("blocs.ditto"), &d)?;
    project::export_png(&dir.join("blocs.png"), &d, 4, None)?;
    anyhow::ensure!(project::load(&dir.join("blocs.ditto"))? == d);
    std::fs::write(dir.join("blocs.txt"), d.text(None))?;
    let braille = &charset::ALL[5];
    let columns = (0..braille.banks())
        .map(|b| braille.bank_len(b) * 2)
        .max()
        .unwrap()
        .max(48);
    let mut sheet =
        Document::new(columns as u32, braille.banks() as u32 * 4).map_err(anyhow::Error::msg)?;
    for cell in std::sync::Arc::make_mut(&mut sheet.cells) {
        cell.bg = Some([20, 19, 24]);
    }
    for bank in 0..braille.banks() {
        let y = bank as i32 * 4;
        let label = format!(
            "{} / {} formes",
            braille.bank_label(bank),
            braille.bank_len(bank)
        );
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
        for (slot, key) in charset::KEYS
            .iter()
            .enumerate()
            .take(braille.bank_len(bank))
        {
            sheet.set(
                slot as i32 * 2,
                y + 1,
                Cell {
                    glyph: font::glyph(*key).unwrap(),
                    fg: [230, 161, 109],
                    bg: Some([20, 19, 24]),
                },
            );
            sheet.set(
                slot as i32 * 2,
                y + 2,
                Cell {
                    glyph: braille.at(bank, slot).unwrap(),
                    fg: [210, 207, 214],
                    bg: Some([20, 19, 24]),
                },
            );
        }
    }
    project::save(&dir.join("braille.ditto"), &sheet)?;
    project::export_png(&dir.join("braille.png"), &sheet, 2, None)?;
    anyhow::ensure!(project::load(&dir.join("braille.ditto"))? == sheet);
    let mut report = format!(
        "Glyphs: {}\nBlock category: {}\nMapping keys: {}\n",
        font::CHARS.len(),
        font::block_glyphs().len(),
        charset::KEYS.len()
    );
    for set in charset::ALL.iter() {
        report.push_str(&format!(
            "{}: {} glyphs, {} banks\n",
            set.name,
            set.targets.len(),
            set.banks()
        ));
    }
    std::fs::write(dir.join("catalogue.txt"), &report)?;
    println!("{report}");
    Ok(())
}
