use ditto::{
    core::{Cell, Document},
    font, project,
};
fn main() -> anyhow::Result<()> {
    let mut d = Document::new(32, 12).map_err(anyhow::Error::msg)?;
    for y in 0..12 {
        for x in 0..32 {
            let c = ['█', '▓', '▒', '░'][(x / 8) as usize];
            d.set(
                x,
                y,
                Cell {
                    glyph: font::glyph(c).unwrap(),
                    fg: [210, 207, 214],
                    bg: Some([16, 16, 20]),
                },
            );
        }
    }
    let dir = std::path::Path::new("validation/runtime/line-spacing");
    std::fs::create_dir_all(dir)?;
    project::export_png(&dir.join("jointive.png"), &d, 4, None)?;
    Ok(())
}
