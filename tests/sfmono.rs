use ditto::{
    core::{Asset, Cell, Document, Reference},
    font, project, typeface,
};
use std::{io::Write, sync::Arc};
#[test]
fn sf_mono_is_loaded_locally_on_the_target_mac() {
    if cfg!(target_os = "macos")
        && std::env::var_os("DITTO_FONT_PATH").is_none()
        && std::path::Path::new("/Library/Fonts/SF-Mono-Regular.otf").exists()
    {
        assert!(typeface::description().contains("SF Mono Regular"));
        assert!(typeface::is_vector());
    }
}
#[test]
fn vector_letters_are_antialiased_and_shared_by_gpu_and_png() {
    let g = font::glyph('A').unwrap();
    let mut d = Document::new(1, 1).unwrap();
    d.set(
        0,
        0,
        Cell {
            glyph: g,
            fg: [200, 80, 30],
            bg: None,
        },
    );
    let im = project::render_png(&d, 4, None).unwrap();
    assert_eq!(im.dimensions(), (32, 64));
    let atlas = typeface::atlas_pixels();
    let ox = g as u32 % typeface::COLUMNS * typeface::TILE_WIDTH + typeface::PAD;
    let oy = g as u32 / typeface::COLUMNS * typeface::TILE_HEIGHT + typeface::PAD;
    for (x, y, p) in im.enumerate_pixels() {
        assert_eq!(
            p[3],
            atlas[(((oy + y) * typeface::ATLAS_WIDTH + ox + x) * 4 + 3) as usize]
        );
        if p[3] > 0 {
            assert_eq!(&p.0[..3], &[200, 80, 30]);
        }
    }
    if typeface::is_vector() {
        assert!(im.pixels().any(|p| p[3] > 0 && p[3] < 255));
    }
}
#[test]
fn every_extended_glyph_has_a_visible_fallback_except_intentional_spaces() {
    for (g, c) in font::CHARS.iter().enumerate() {
        if [' ', '\u{a0}', '\u{2800}'].contains(c) {
            continue;
        }
        assert!(
            typeface::mask(g as font::Glyph).iter().any(|a| *a > 0),
            "Missing glyph U+{:04X}",
            *c as u32
        );
    }
}
#[test]
fn glyph_coverage_does_not_bleed_into_neighbours() {
    let atlas = typeface::atlas_pixels();
    for g in 1..font::CHARS.len() {
        if typeface::is_tile(font::character(g as font::Glyph)) {
            continue;
        }
        let x = g as u32 % typeface::COLUMNS * typeface::TILE_WIDTH;
        let y = g as u32 / typeface::COLUMNS * typeface::TILE_HEIGHT;
        for xx in 0..typeface::TILE_WIDTH {
            assert_eq!(
                atlas[((y * typeface::ATLAS_WIDTH + x + xx) * 4 + 3) as usize],
                0
            );
        }
    }
}
#[test]
fn natural_cell_proportions_keep_new_references_undistorted() {
    let asset = Arc::new(Asset {
        width: 20,
        height: 10,
        rgba: vec![0; 800],
    });
    let r = Reference::fit(asset, 40, 40, false);
    let w = r.asset.width as f32 * r.scale * typeface::CELL_WIDTH as f32;
    let h = r.asset.height as f32 * r.scale * r.pixel_aspect * typeface::CELL_HEIGHT as f32;
    assert!((w / h - 2.).abs() < 0.0001);
    assert_eq!(typeface::ASPECT, 0.5);
}
#[test]
fn old_project_formats_still_load_without_changing_cells() {
    for (version, profile, g) in [
        (1, font::LEGACY_PROFILE, 65),
        (2, font::PROFILE, font::glyph('▘').unwrap()),
        (4, typeface::PROFILE, 65),
        (3, typeface::PROFILE, font::glyph('▘').unwrap()),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("old.ditto");
        let f = std::fs::File::create(&p).unwrap();
        let mut z = zip::ZipWriter::new(f);
        let o = zip::write::SimpleFileOptions::default();
        z.start_file("manifest.json", o).unwrap();
        write!(z,"{{\"format\":\"ditto\",\"version\":{version},\"profile\":\"{profile}\",\"width\":1,\"height\":1,\"palette\":[[0,0,0]],\"reference\":null}}").unwrap();
        z.start_file("drawing.json", o).unwrap();
        write!(z, "[{{\"glyph\":{g},\"fg\":[20,50,90],\"bg\":null}}]").unwrap();
        z.finish().unwrap();
        let d = project::load(&p).unwrap();
        assert_eq!(d.get(0, 0).unwrap().glyph, g);
        assert_eq!(d.get(0, 0).unwrap().fg, [20, 50, 90]);
        assert_eq!(d.shaders, ditto::shaders::Stack::default());
    }
}

#[test]
fn full_blocks_have_no_empty_rows_or_columns_at_cell_boundaries() {
    let mut d = Document::new(3, 3).unwrap();
    for y in 0..3 {
        for x in 0..3 {
            d.set(
                x,
                y,
                Cell {
                    glyph: font::glyph('█').unwrap(),
                    fg: [210, 100, 40],
                    bg: None,
                },
            );
        }
    }
    for scale in [1, 2, 4] {
        let im = project::render_png(&d, scale, None).unwrap();
        assert!(
            im.pixels().all(|p| p.0 == [210, 100, 40, 255]),
            "Visible seam at scale {scale}"
        );
    }
}
#[test]
fn tile_padding_prevents_bilinear_seams_when_zoomed_in() {
    let atlas = typeface::atlas_pixels();
    let g = font::glyph('█').unwrap();
    let x = g as u32 % typeface::COLUMNS * typeface::TILE_WIDTH;
    let y = g as u32 / typeface::COLUMNS * typeface::TILE_HEIGHT;
    for yy in 0..typeface::TILE_HEIGHT {
        for xx in 0..typeface::TILE_WIDTH {
            assert_eq!(
                atlas[(((y + yy) * typeface::ATLAS_WIDTH + x + xx) * 4 + 3) as usize],
                255
            );
        }
    }
    for scale in [8, 16] {
        assert_eq!(typeface::coverage(g, 0, 0, scale), 255);
        assert_eq!(
            typeface::coverage(
                g,
                typeface::CELL_WIDTH * scale - 1,
                typeface::CELL_HEIGHT * scale - 1,
                scale
            ),
            255
        );
    }
}
#[test]
fn shade_and_rule_masks_join_without_typographic_leading() {
    for c in ['░', '▒', '▓', '│', '║'] {
        let g = font::glyph(c).unwrap();
        for y in 0..typeface::CELL_HEIGHT {
            assert!(
                (0..typeface::CELL_WIDTH).any(|x| typeface::coverage(g, x, y, 1) > 0),
                "Empty row in {c} at {y}"
            );
        }
    }
    for c in ['─', '═'] {
        let g = font::glyph(c).unwrap();
        for x in 0..typeface::CELL_WIDTH {
            assert!(
                (0..typeface::CELL_HEIGHT).any(|y| typeface::coverage(g, x, y, 1) > 0),
                "Empty column in {c} at {x}"
            );
        }
    }
}
