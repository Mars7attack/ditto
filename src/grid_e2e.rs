//! Check the actual UI raster at multiple physical display resolutions, using
//! the same GPU pipeline and clipping as the window (no monitor reconfiguration).
use crate::{
    app::State,
    render::{Rect, Renderer},
};
use anyhow::{Result, ensure};
use std::{collections::BTreeSet, path::Path};

pub fn verify(renderer: &Renderer, dir: &Path) -> Result<()> {
    let dir = dir.join("grid");
    std::fs::create_dir_all(&dir)?;
    let mut report = format!("GPU: {}\n", renderer.adapter_name);
    let mut count = 0;
    for dpi in [1., 1.25, 1.5, 2., 3.] {
        for zoom in [8.3, 16., 31.5] {
            for (pan_index, pan) in [(0., 0.), (-57.73, 19.41)].into_iter().enumerate() {
                let mut s = State::new(dir.join("unused-recovery.ditto"));
                s.modal = None;
                s.tool = ditto::core::Tool::Guide;
                s.grid = true;
                s.fit = false;
                s.cell_size = zoom;
                s.pan = pan;
                s.settings.theme.canvas = [0; 3];
                s.settings.theme.grid = [240; 3];
                let draw = s.frame();
                let (width, height) = ((s.width * dpi) as u32, (s.height * dpi) as u32);
                let path = dir.join(format!("dpi-{dpi}-zoom-{zoom}-pan-{pan_index}.png"));
                renderer.capture_draw(draw, width, height, &path)?;
                let image = image::open(&path)?.to_rgba8();
                let board = Rect::new(
                    s.origin.0,
                    s.origin.1,
                    s.editor.document.width as f32 * s.cell_width(),
                    s.editor.document.height as f32 * s.cell_size,
                );
                let clip = board.intersect(s.canvas);
                // Stay away from the enclosing board border and clip rounding.
                let margin = (2. * dpi).ceil() as u32 + 1;
                let x0 = (clip.x * dpi).ceil() as u32 + margin;
                let x1 = ((clip.x + clip.w) * dpi).floor() as u32 - margin;
                let y0 = (clip.y * dpi).ceil() as u32 + margin;
                let y1 = ((clip.y + clip.h) * dpi).floor() as u32 - margin;
                let xs: BTreeSet<_> = (0..=s.editor.document.width)
                    .map(|i| ((s.origin.0 + i as f32 * s.cell_width()) * dpi).round() as i32)
                    .filter(|x| *x >= x0 as i32 && *x < x1 as i32)
                    .map(|x| x as u32)
                    .collect();
                let ys: BTreeSet<_> = (0..=s.editor.document.height)
                    .map(|i| ((s.origin.1 + i as f32 * s.cell_size) * dpi).round() as i32)
                    .filter(|y| *y >= y0 as i32 && *y < y1 as i32)
                    .map(|y| y as u32)
                    .collect();
                ensure!(xs.len() > 10 && ys.len() > 10);
                let column = (x0..x1).find(|x| !xs.contains(x)).unwrap();
                let row = (y0..y1).find(|y| !ys.contains(y)).unwrap();
                let lit = |x, y| image.get_pixel(x, y).0[..3].iter().all(|c| *c > 80);
                for x in x0..x1 {
                    ensure!(
                        lit(x, row) == xs.contains(&x),
                        "missing/extra vertical grid pixel ({x},{row}), DPI {dpi}, zoom {zoom}, pan {pan_index}: {:?}",
                        image.get_pixel(x, row)
                    );
                }
                for y in y0..y1 {
                    ensure!(
                        lit(column, y) == ys.contains(&y),
                        "missing/extra horizontal grid pixel ({column},{y}), DPI {dpi}, zoom {zoom}, pan {pan_index}: {:?}",
                        image.get_pixel(column, y)
                    );
                }
                report.push_str(&format!("PASS dpi={dpi} zoom={zoom} pan={pan_index}: {} vertical and {} horizontal lines, all 1 physical pixel\n",xs.len(),ys.len()));
                // No lines may leak outside the board into the canvas.
                if board.x > s.canvas.x + 8. {
                    ensure!(
                        !lit(((board.x - 4.) * dpi) as u32, row),
                        "grid leaked outside document"
                    );
                }
                count += 1;
            }
        }
    }
    ensure!(count == 30);
    report.push_str("PASS: 30 GPU grid fixtures across 5 display scales, 3 zooms and 2 pans\n");
    std::fs::write(dir.join("result.txt"), report)?;
    Ok(())
}
