use ditto::{
    core::{Cell, Document, Editor},
    project, shader_gpu,
    shaders::{self, Kind, Layer, Stack},
};
use image::{Rgba, RgbaImage};

fn stack(kind: Kind) -> Stack {
    Stack {
        enabled: true,
        layers: vec![Layer::new(kind)],
    }
}

#[test]
#[ignore = "Requires a real wgpu adapter; run explicitly during native shader validation"]
fn chromatic_preserves_dark_primaries_without_black_ghosts_and_moves_subpixels() {
    let mut recipe = stack(Kind::Chromatic);
    recipe.layers[0].params = [4., 0., 0.5];
    for (channel, color, shift) in [(0, [48, 0, 0], -4), (1, [0, 48, 0], 0), (2, [0, 0, 48], 4)] {
        let mut source = RgbaImage::from_pixel(64, 16, Rgba([255, 80, 10, 0]));
        for y in 0..16 {
            source.put_pixel(32, y, Rgba([color[0], color[1], color[2], 192]));
        }
        let out = shader_gpu::render(&source, &recipe, 1.).unwrap();
        for x in 0..64 {
            let p = out.get_pixel(x, 8).0;
            if x as i32 == 32 + shift {
                assert!(
                    p[channel].abs_diff(48) <= 1 && p[3].abs_diff(192) <= 1,
                    "{channel}: {p:?}"
                );
            } else {
                assert_eq!(
                    p[3], 0,
                    "absent channel produced an opaque ghost at {x}: {p:?}"
                );
            }
        }
    }
    let mut source = RgbaImage::new(64, 16);
    for y in 0..16 {
        source.put_pixel(32, y, Rgba([0, 0, 48, 255]));
    }
    recipe.layers[0].params[0] = 0.25;
    let out = shader_gpu::render(&source, &recipe, 1.).unwrap();
    assert!(out.get_pixel(32, 8)[3].abs_diff(191) <= 1);
    assert!(out.get_pixel(33, 8)[3].abs_diff(64) <= 1);
    assert_eq!(out.get_pixel(33, 8)[2], 48);
    recipe.layers[0].params[0] = 0.;
    assert_eq!(shader_gpu::render(&source, &recipe, 1.).unwrap(), source);
}

#[test]
#[ignore = "Requires a real wgpu adapter; run explicitly during native shader validation"]
fn chromatic_uses_linear_light_and_stable_document_space_offsets() {
    let mut recipe = stack(Kind::Chromatic);
    recipe.layers[0].params = [0.5, 0., 0.5];
    let mut source = RgbaImage::new(64, 16);
    for y in 0..16 {
        source.put_pixel(32, y, Rgba([64, 0, 0, 255]));
        source.put_pixel(33, y, Rgba([192, 0, 0, 255]));
    }
    let out = shader_gpu::render(&source, &recipe, 1.).unwrap();
    // The linear-light average encodes to sRGB 146, not the encoded average 128.
    assert!(out.get_pixel(32, 8)[0].abs_diff(146) <= 1);
    for color in [[0, 0, 48], [8, 16, 64], [90; 3], [255; 3], [0; 3]] {
        let solid = RgbaImage::from_pixel(32, 32, Rgba([color[0], color[1], color[2], 128]));
        let out = shader_gpu::render(&solid, &recipe, 1.).unwrap();
        for (a, b) in out
            .get_pixel(16, 16)
            .0
            .into_iter()
            .zip([color[0], color[1], color[2], 128])
        {
            assert!(a.abs_diff(b) <= 1);
        }
    }
    for scale in [1, 2, 4] {
        let mut source = RgbaImage::new(64 * scale, 16 * scale);
        for y in 0..16 * scale {
            for x in 32 * scale..33 * scale {
                source.put_pixel(x, y, Rgba([0, 0, 48, 255]));
            }
        }
        for (angle, dx, dy) in [
            (0., 0.25, 0.),
            (90., 0., 0.25),
            (45., 0.25 / 2_f64.sqrt(), 0.25 / 2_f64.sqrt()),
        ] {
            recipe.layers[0].params = [0.25, angle, 0.5];
            // Vertical edges deliberately span the full image; sample the centre row for x shifts.
            let out = shader_gpu::render(&source, &recipe, scale as f32).unwrap();
            let row = 8 * scale;
            let mass: f64 = (0..out.width())
                .map(|x| out.get_pixel(x, row)[3] as f64)
                .sum();
            let center: f64 = (0..out.width())
                .map(|x| (x as f64 + 0.5) * out.get_pixel(x, row)[3] as f64)
                .sum::<f64>()
                / mass;
            assert!(
                (center / scale as f64 - 32.5 - dx).abs() < 0.01,
                "{scale} {angle} {center} {dy}"
            );
            assert!((mass / (255. * scale as f64) - 1.).abs() < 0.01);
        }
    }
}

#[test]
fn shader_edits_survive_undo_redo_save_reopen_and_resize() {
    let mut editor = Editor::new(Document::new(12, 8).unwrap());
    editor.document.set(
        3,
        3,
        Cell {
            glyph: 219,
            fg: [255, 190, 120],
            bg: None,
        },
    );
    let cells = editor.document.cells.clone();
    editor.edit(|d| d.shaders = shaders::preset(0));
    let recipe = editor.document.shaders.clone();
    assert!(editor.dirty());
    assert!(editor.undo());
    assert!(!editor.document.shaders.active());
    assert!(editor.redo());
    assert_eq!(editor.document.shaders, recipe);
    assert_eq!(editor.document.cells, cells);
    editor.edit(|d| d.resize(14, 10).unwrap());
    assert_eq!(editor.document.shaders, recipe);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shaders.ditto");
    project::save(&path, &editor.document).unwrap();
    assert_eq!(project::load(&path).unwrap(), editor.document);
    let path = dir.path().join("look.ditto-shaders");
    shaders::save_preset(&path, &recipe).unwrap();
    assert_eq!(shaders::load_preset(&path).unwrap(), recipe);
}

#[test]
fn large_drawing_keeps_shader_history_without_copying_cells() {
    let mut editor = Editor::new(Document::new(512, 512).unwrap());
    let cells = editor.document.cells.clone();
    editor.edit(|d| d.shaders = stack(Kind::Color));
    for _ in 0..30 {
        editor.edit(|d| d.shaders.layers[0].adjust(1, 1));
    }
    assert!(std::sync::Arc::ptr_eq(&cells, &editor.document.cells));
    while editor.undo() {}
    assert_eq!(editor.document.shaders, Stack::default());
}

#[test]
fn invalid_and_oversized_presets_are_rejected_without_mutating_project() {
    for kind in shaders::KINDS {
        stack(kind).validate().unwrap();
    }
    for i in 0..shaders::PRESETS.len() {
        shaders::preset(i).validate().unwrap();
    }
    let mut bad = stack(Kind::Glow);
    bad.layers[0].params[0] = f32::NAN;
    assert!(bad.validate().is_err());
    bad.layers[0].params[0] = 1e9;
    assert!(bad.validate().is_err());
    bad = stack(Kind::Glow);
    bad.layers = vec![Layer::new(Kind::Glow); 9];
    assert!(bad.validate().is_err());
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.ditto-shaders");
    std::fs::write(&file, br#"{"enabled":true,"layers":[],"unknown":true}"#).unwrap();
    assert!(shaders::load_preset(&file).is_err());
    std::fs::write(&file, vec![b' '; 65_537]).unwrap();
    assert!(shaders::load_preset(&file).is_err());
}

#[test]
fn disabled_stack_and_zero_mix_are_pixel_identical_without_a_gpu() {
    let mut doc = Document::new(3, 3).unwrap();
    doc.set(
        1,
        1,
        Cell {
            glyph: 219,
            fg: [220, 140, 50],
            bg: None,
        },
    );
    let clean = project::render_png(&doc, 1, None).unwrap();
    doc.shaders = stack(Kind::Glow);
    doc.shaders.enabled = false;
    assert_eq!(project::render_png(&doc, 1, None).unwrap(), clean);
    doc.shaders.enabled = true;
    doc.shaders.layers[0].mix = 0.;
    assert_eq!(project::render_png(&doc, 1, None).unwrap(), clean);
}

#[test]
fn blur_recipes_roundtrip_through_projects_presets_and_undo() {
    let dir = tempfile::tempdir().unwrap();
    let mut editor = Editor::new(Document::new(8, 4).unwrap());
    let cells = editor.document.cells.clone();
    for kind in [Kind::Blur, Kind::ContourBlur] {
        editor.edit(|doc| doc.shaders.layers.push(Layer::new(kind)));
    }
    let recipe = editor.document.shaders.clone();
    editor.undo();
    assert_eq!(editor.document.shaders.layers.len(), 1);
    editor.redo();
    assert_eq!(editor.document.shaders, recipe);
    assert!(std::sync::Arc::ptr_eq(&cells, &editor.document.cells));
    let project_path = dir.path().join("blur.ditto");
    project::save(&project_path, &editor.document).unwrap();
    assert_eq!(project::load(&project_path).unwrap(), editor.document);
    let preset_path = dir.path().join("blur.ditto-shaders");
    shaders::save_preset(&preset_path, &recipe).unwrap();
    assert_eq!(shaders::load_preset(&preset_path).unwrap(), recipe);
    for kind in [Kind::Blur, Kind::ContourBlur] {
        let mut invalid = stack(kind);
        invalid.layers[0].params[0] = -0.5;
        assert!(invalid.validate().is_err());
    }
}

#[test]
fn preview_retains_atlas_resolution_with_bounded_large_document_fallbacks() {
    assert_eq!(shader_gpu::preview_scale(80, 50, 16384).unwrap(), 4);
    assert_eq!(shader_gpu::preview_scale(16, 6, 16384).unwrap(), 4);
    assert_eq!(shader_gpu::preview_scale(256, 128, 16384).unwrap(), 2);
    assert_eq!(shader_gpu::preview_scale(512, 512, 16384).unwrap(), 1);
    assert_eq!(shader_gpu::preview_scale(80, 50, 2048).unwrap(), 2);
    assert!(shader_gpu::preview_scale(512, 512, 4096).is_err());
    assert!(shader_gpu::preview_scale(0, 10, 16384).is_err());
    let mut doc = Document::new(1, 1).unwrap();
    for glyph in ['S', '@', 'g', 'é', '⣿'] {
        let g = ditto::font::glyph(glyph).unwrap();
        doc.set(
            0,
            0,
            Cell {
                glyph: g,
                fg: [255; 3],
                bg: None,
            },
        );
        let image =
            project::render_cells(&doc, shader_gpu::preview_scale(1, 1, 16384).unwrap(), None)
                .unwrap();
        let alpha: Vec<_> = image.pixels().map(|p| p[3]).collect();
        assert_eq!(
            alpha,
            ditto::typeface::mask(g),
            "{glyph} lost atlas coverage before effects"
        );
    }
}

#[test]
#[ignore = "Requires a real wgpu adapter; run explicitly during native shader validation"]
fn gpu_effects_alpha_order_scale_and_export() {
    let mut source = RgbaImage::new(64, 64);
    for y in 8..56 {
        for x in 8..56 {
            source.put_pixel(x, y, Rgba([(x * 4) as u8, (y * 4) as u8, 160, 255]));
        }
    }
    for kind in shaders::KINDS {
        let recipe = stack(kind);
        let output = shader_gpu::render(&source, &recipe, 1.).unwrap();
        assert_ne!(output, source, "{} did not change any pixels", kind.name());
        assert_eq!(output.dimensions(), source.dimensions());
        if !matches!(
            kind,
            Kind::Glow | Kind::Chromatic | Kind::Blur | Kind::ContourBlur
        ) {
            assert!(
                output
                    .pixels()
                    .zip(source.pixels())
                    .all(|(a, b)| a[3] == b[3]),
                "{} changed alpha",
                kind.name()
            );
        }
        let empty = RgbaImage::new(16, 16);
        assert_eq!(
            shader_gpu::render(&empty, &recipe, 1.).unwrap(),
            empty,
            "{} polluted empty canvas",
            kind.name()
        );
    }
    let glow = shader_gpu::render(&source, &stack(Kind::Glow), 1.).unwrap();
    assert!(
        glow.pixels()
            .zip(source.pixels())
            .any(|(a, b)| a[3] > 0 && b[3] == 0),
        "glow must extend into transparent pixels"
    );
    let mut thin_stroke = RgbaImage::new(128, 128);
    for y in 0..128 {
        thin_stroke.put_pixel(64, y, Rgba([255; 4]));
    }
    let smooth = shader_gpu::render(&thin_stroke, &stack(Kind::Glow), 4.).unwrap();
    for x in 54..75 {
        assert!(
            smooth.get_pixel(x, 64)[3] > 0,
            "high-resolution glow left a hole at {x}"
        );
    }
    let mut recipe = Stack {
        enabled: true,
        layers: vec![Layer::new(Kind::Glow), Layer::new(Kind::Pattern)],
    };
    let before = shader_gpu::render(&source, &recipe, 1.).unwrap();
    recipe.layers.reverse();
    assert_ne!(
        before,
        shader_gpu::render(&source, &recipe, 1.).unwrap(),
        "order must affect output"
    );
    let grain = stack(Kind::Grain);
    let first = shader_gpu::render(&source, &grain, 1.).unwrap();
    assert_eq!(
        first,
        shader_gpu::render(&source, &grain, 1.).unwrap(),
        "grain must be deterministic"
    );
    let big = image::imageops::resize(&source, 128, 128, image::imageops::FilterType::Nearest);
    let big_output = shader_gpu::render(&big, &grain, 2.).unwrap();
    for y in 0..64 {
        for x in 0..64 {
            assert_eq!(first.get_pixel(x, y), big_output.get_pixel(x * 2, y * 2));
        }
    }
    // Repeated layers exercise all ping-pong slots, including reuse of the initial source texture.
    recipe.layers = shaders::KINDS[..8].iter().map(|k| Layer::new(*k)).collect();
    let combined = shader_gpu::render(&source, &recipe, 1.).unwrap();
    assert_ne!(combined, source);
    let mut doc = Document::new(8, 4).unwrap();
    doc.set(
        3,
        1,
        Cell {
            glyph: 219,
            fg: [240, 170, 90],
            bg: None,
        },
    );
    doc.shaders = stack(Kind::Glow);
    let raw = project::render_cells(&doc, 1, None).unwrap();
    let exported = project::render_png(&doc, 1, None).unwrap();
    assert_eq!(
        exported,
        shader_gpu::render(&raw, &doc.shaders, 1.).unwrap()
    );
    let bg = [16, 16, 20];
    let opaque = project::render_png(&doc, 1, Some(bg)).unwrap();
    for (p, o) in exported.pixels().zip(opaque.pixels()) {
        assert_eq!(o[3], 255);
        for k in 0..3 {
            assert_eq!(
                o[k],
                (p[k] as f32 * (p[3] as f32 / 255.) + bg[k] as f32 * (1. - p[3] as f32 / 255.))
                    .round() as u8
            );
        }
    }
    for scale in [2, 4] {
        assert_eq!(
            project::render_png(&doc, scale, None).unwrap().dimensions(),
            (64 * scale, 64 * scale)
        );
    }
}

#[test]
#[ignore = "Requires a real wgpu adapter; run explicitly during native shader validation"]
fn gpu_blur_is_directional_and_contour_blur_preserves_low_contrast_interiors() {
    let mut source = RgbaImage::new(128, 128);
    for y in 32..96 {
        for x in 32..96 {
            source.put_pixel(x, y, Rgba([200, 80, 40, 255]));
        }
    }
    let mut blur = stack(Kind::Blur);
    blur.layers[0].params[0] = 6.;
    let full = shader_gpu::render(&source, &blur, 1.).unwrap();
    assert!(full.get_pixel(31, 64)[3] > 0);
    assert!(full.get_pixel(32, 64)[3] < 255);
    assert_eq!(full.get_pixel(64, 64), source.get_pixel(64, 64));
    for (k, want) in [200u8, 80, 40].into_iter().enumerate() {
        assert!(
            full.get_pixel(31, 64)[k].abs_diff(want) <= 3,
            "transparent edges darkened the colour"
        );
    }
    let alpha_sum = |image: &RgbaImage| image.pixels().map(|p| u64::from(p[3])).sum::<u64>();
    assert!(alpha_sum(&full).abs_diff(alpha_sum(&source)) < alpha_sum(&source) / 100);
    blur.layers[0].params[2] = 0.;
    let horizontal = shader_gpu::render(&source, &blur, 1.).unwrap();
    assert!(horizontal.get_pixel(31, 64)[3] > 0);
    assert_eq!(horizontal.get_pixel(64, 31)[3], 0);
    blur.layers[0].params[1] = 0.;
    blur.layers[0].params[2] = 1.;
    let vertical = shader_gpu::render(&source, &blur, 1.).unwrap();
    assert_eq!(vertical.get_pixel(31, 64)[3], 0);
    assert!(vertical.get_pixel(64, 31)[3] > 0);

    // The same edge behaviour in document coordinates at Retina/export scales.
    blur.layers[0].params = [6., 1., 1.];
    for scale in [2, 4] {
        let high = image::imageops::resize(
            &source,
            128 * scale,
            128 * scale,
            image::imageops::FilterType::Nearest,
        );
        let output = shader_gpu::render(&high, &blur, scale as f32).unwrap();
        assert!(output.get_pixel(31 * scale, 64 * scale)[3] > 0);
        assert_eq!(output.get_pixel(24 * scale, 64 * scale)[3], 0);
        assert_eq!(
            output.get_pixel(64 * scale, 64 * scale),
            high.get_pixel(64 * scale, 64 * scale)
        );
    }
    for kind in [Kind::Blur, Kind::ContourBlur] {
        let mut neutral = stack(kind);
        neutral.layers[0].params[0] = 0.;
        assert_eq!(shader_gpu::render(&source, &neutral, 1.).unwrap(), source);
        neutral.layers[0].params[0] = 3.;
        neutral.layers[0].mix = 0.;
        assert_eq!(shader_gpu::render(&source, &neutral, 1.).unwrap(), source);
    }
    let mut textured = RgbaImage::from_pixel(128, 128, Rgba([96, 96, 96, 255]));
    for y in 16..48 {
        for x in 16..48 {
            let v = if (x + y) % 2 == 0 { 104 } else { 96 };
            textured.put_pixel(x, y, Rgba([v, v, v, 255]));
        }
    }
    for y in 64..96 {
        for x in 64..96 {
            textured.put_pixel(x, y, Rgba([240, 240, 240, 255]));
        }
    }
    let full = shader_gpu::render(&textured, &blur, 1.).unwrap();
    let mut contours = stack(Kind::ContourBlur);
    contours.layers[0].params[0] = 6.;
    let selective = shader_gpu::render(&textured, &contours, 1.).unwrap();
    assert_ne!(full.get_pixel(32, 32), textured.get_pixel(32, 32));
    assert_eq!(selective.get_pixel(32, 32), textured.get_pixel(32, 32));
    assert_ne!(
        selective.get_pixel(63, 80),
        textured.get_pixel(63, 80),
        "opaque colour contours should soften too"
    );
    assert_eq!(selective.get_pixel(80, 80), textured.get_pixel(80, 80));
    contours.layers[0].params[1] = 1.;
    assert_eq!(
        shader_gpu::render(&textured, &contours, 1.).unwrap(),
        textured
    );

    let mut doc = Document::new(8, 4).unwrap();
    doc.set(
        3,
        1,
        Cell {
            glyph: 219,
            fg: [200, 80, 40],
            bg: None,
        },
    );
    doc.shaders.layers = vec![
        Layer::new(Kind::Blur),
        Layer::new(Kind::ContourBlur),
        Layer::new(Kind::Glow),
        Layer::new(Kind::Blur),
    ];
    for scale in [1, 2, 4] {
        let raw = project::render_cells(&doc, scale, None).unwrap();
        assert_eq!(
            project::render_png(&doc, scale, None).unwrap(),
            shader_gpu::render(&raw, &doc.shaders, scale as f32).unwrap()
        );
    }
}
