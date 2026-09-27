//! Render the actual system typeface through Ditto's PNG pipeline.
use ditto::{
    core::{Cell, Document},
    font, project, typeface,
};
fn main() -> anyhow::Result<()> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "validation/runtime/sfmono".into()),
    );
    std::fs::create_dir_all(&dir)?;
    let mut d = Document::new(58, 13).map_err(anyhow::Error::msg)?;
    let lines = [
        "DITTO / SF MONO",
        "Interface, dessin et PNG",
        "abcdefghijklmnopqrstuvwxyz",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "0123456789  !@#$%&*()[]{}/\\",
        "café déjà vu / élève / Noël",
        "┌──────────┬──────────┐",
        "│  ░▒▓█    │  ▖▗▘▝▚▞  │",
        "└──────────┴──────────┘",
        "Braille : ⠁⠃⠇⡇⣿",
        "Mosaïques : 🬀🬁🬂🬃🬄🬅",
        "Anticrénelage et transparence",
    ];
    for (y, line) in lines.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if let Some(g) = font::glyph(c) {
                d.set(
                    x as i32 + 1,
                    y as i32,
                    Cell {
                        glyph: g,
                        fg: if y == 0 {
                            [230, 161, 109]
                        } else {
                            [210, 207, 214]
                        },
                        bg: None,
                    },
                );
            }
        }
    }
    project::save(&dir.join("sfmono.ditto"), &d)?;
    project::export_png(&dir.join("sfmono.png"), &d, 2, Some([16, 16, 20]))?;
    std::fs::write(dir.join("font.txt"), typeface::description())?;
    println!("{}", typeface::description());
    Ok(())
}
