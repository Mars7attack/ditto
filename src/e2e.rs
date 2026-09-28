//! Native application scenarios: real input routing, hit testing, GPU frames and
//! persisted documents. No platform file dialogs or user's clipboard/settings.
use crate::{
    app::{Action, EditMode, Modal, State},
    color_picker::{self, Control},
    route_key,
};
use anyhow::{Result, ensure};
use ditto::{
    core::{Document, Editor, Tool},
    project,
    settings::Settings,
    shaders::Kind,
};
use std::path::Path;
use winit::keyboard::{Key, ModifiersState, NamedKey};

pub const END: u32 = 42;
pub fn name(frame: u32) -> Option<&'static str> {
    Some(match frame {
        17 => "cursor-after-guide",
        18 => "picker-guide-original",
        19 => "picker-guide-blue",
        20 => "cursor-after-guide-color",
        21 => "cursor-after-mode-cycles",
        22 => "cursor-after-focus-loss",
        23 => "cursor-with-shaders",
        24 => "cursor-zoom-end-of-line",
        25 => "picker-foreground",
        26 => "cursor-after-picker-keyboard",
        27 => "cursor-project-reopened",
        28 => "cursor-guide-stroke-keyboard",
        29 => "cursor-guide-front-keyboard",
        30 => "cursor-undo-after-navigation",
        31 => "cursor-redo-after-navigation",
        32 => "cursor-guide-eraser-keyboard",
        33 => "cursor-text-tool-keyboard-escape",
        34 => "palette-picker-edited",
        35 => "palette-swatch-synchronized",
        36 => "chromatic-original",
        37 => "chromatic-slider-live",
        38 => "chromatic-slider-committed",
        39 => "text-export-monospaced",
        40 => "text-export-discord",
        41 => "text-export-zoomed",
        _ => return None,
    })
}
fn key(s: &mut State, key: NamedKey) {
    route_key(s, ModifiersState::empty(), Key::Named(key), None);
    s.frame();
}
fn click(s: &mut State, action: Action) -> Result<()> {
    s.frame();
    let r = s
        .hits
        .iter()
        .find(|h| h.action == action)
        .ok_or_else(|| anyhow::anyhow!("Missing control: {action:?}"))?
        .rect;
    s.mouse_move((r.x + r.w / 2., r.y + r.h / 2.));
    s.mouse_down(false);
    s.mouse_up();
    s.frame();
    Ok(())
}
fn drag(s: &mut State, control: Control, to: (f32, f32)) {
    let r = color_picker::rect(s.width, control);
    s.mouse_move((r.x + 2., r.y + 2.));
    s.mouse_down(false);
    s.mouse_move((r.x + to.0 * r.w, r.y + to.1 * r.h));
    s.mouse_up();
}
fn history_key(s: &mut State, redo: bool) {
    let command = if cfg!(target_os = "macos") {
        ModifiersState::SUPER
    } else {
        ModifiersState::CONTROL
    };
    route_key(
        s,
        command
            | if redo {
                ModifiersState::SHIFT
            } else {
                ModifiersState::empty()
            },
        Key::Character("z".into()),
        None,
    );
    s.frame();
}
pub fn prepare(s: &mut State, frame: u32, dir: &Path) -> Result<()> {
    match frame {
        17 => {
            s.modal = None;
            s.settings = Settings::default();
            s.editor = Editor::new(Document::new(24, 16).unwrap());
            s.editor
                .document
                .guides
                .start([1., 5.5], [80, 220, 160], 1.);
            s.editor.document.guides.append([20., 5.5]);
            s.editor.document.guides.above_characters = true;
            s.cursor = (6, 5);
            s.fit = false;
            s.cell_size = 24.;
            s.pan = (0., 0.);
            s.frame();
            s.activate(Action::Tool(Tool::Guide));
            click(s, Action::ToggleEditMode)?;
            ensure!(s.keyboard_active() && s.active_mouse_tool().is_none());
        }
        18 => {
            key(s, NamedKey::F8);
            click(s, Action::GuideColor)?;
        }
        19 => {
            drag(s, Control::Hue, (2. / 3., 0.5));
            drag(s, Control::Plane, (0.75, 0.2));
            ensure!(s.color_picker.rgb() == [51, 51, 204]);
            ensure!(s.editor.document.guides.strokes[0].color == [80, 220, 160]);
        }
        20 => {
            click(s, Action::Submit)?;
            ensure!(s.guide_color == [51, 51, 204]);
            click(s, Action::GuideRecolorAll)?;
            key(s, NamedKey::Escape);
            ensure!(s.keyboard_active());
        }
        21 => {
            for _ in 0..6 {
                click(s, Action::ToggleEditMode)?;
            }
            ensure!(s.keyboard_active());
            route_key(
                s,
                ModifiersState::empty(),
                Key::Character("a".into()),
                Some("a"),
            );
            ensure!(s.cursor == (7, 5));
            s.activate(Action::Undo);
            ensure!(s.cursor == (6, 5));
            s.activate(Action::Redo);
            ensure!(s.cursor == (7, 5));
        }
        22 => {
            s.lost_focus();
            key(s, NamedKey::F4);
            key(s, NamedKey::F6);
            ensure!(s.keyboard_active());
        }
        23 => {
            s.activate(Action::ShaderAdd(Kind::Duotone));
            ensure!(s.editor.document.shaders.active());
        }
        24 => {
            s.zoom_by(3., s.mouse);
            s.move_cursor(24, 10, false);
            ensure!(s.cursor.0 == 24 && s.cursor.1 == 15);
            s.frame();
        }
        25 => {
            click(s, Action::Foreground)?;
            drag(s, Control::Hue, (1. / 3., 0.5));
            drag(s, Control::Plane, (1., 0.));
            ensure!(s.color_picker.rgb() == [0, 255, 0]);
        }
        26 => {
            key(s, NamedKey::Tab); // hue, after plane drag
            key(s, NamedKey::Tab); // hex
            route_key(
                s,
                ModifiersState::empty(),
                Key::Character("8".into()),
                Some("88CC44"),
            );
            key(s, NamedKey::Enter);
            ensure!(s.brush.cell.fg == [136, 204, 68] && s.keyboard_active());
        }
        27 => {
            let path = dir.join("e2e-roundtrip.ditto");
            project::save(&path, &s.editor.document)?;
            let loaded = project::load(&path)?;
            ensure!(loaded == s.editor.document);
            project::export_png(&dir.join("e2e-export.png"), &loaded, 1, None)?;
            let mut art_only = loaded.clone();
            art_only.guides = Default::default();
            art_only.reference = None;
            ensure!(
                project::render_png(&loaded, 1, None)? == project::render_png(&art_only, 1, None)?
            );
            s.editor = Editor::new(loaded);
            s.cursor = (6, 5);
            s.fit = true;
            ensure!(s.edit_mode == EditMode::Keyboard);
        }
        28 => {
            s.editor = Editor::new(Document::new(24, 16).unwrap());
            s.fit = true;
            key(s, NamedKey::F8);
            click(s, Action::Tool(Tool::Guide))?;
            let point = |x: f32| {
                (
                    s.origin.0 + x * s.cell_width(),
                    s.origin.1 + 5.5 * s.cell_size,
                )
            };
            let (a, b) = (point(2.), point(20.));
            s.mouse_move(a);
            s.mouse_down(false);
            s.mouse_move(b);
            s.mouse_up();
            ensure!(s.editor.document.guides.strokes.len() == 1);
            click(s, Action::ToggleEditMode)?;
            let guides = s.editor.document.guides.clone();
            s.mouse_move((
                s.origin.0 + 6.5 * s.cell_width(),
                s.origin.1 + 5.5 * s.cell_size,
            ));
            s.mouse_down(false);
            s.mouse_up();
            ensure!(s.cursor == (6, 5) && s.editor.document.guides == guides);
        }
        29 => {
            key(s, NamedKey::F8);
            click(s, Action::GuideAbove)?;
            click(s, Action::GuideColor)?;
            s.text_input("42a5f5");
            click(s, Action::Submit)?;
            click(s, Action::GuideRecolorAll)?;
            key(s, NamedKey::Escape);
            s.keyboard_input("a");
            ensure!(s.cursor == (7, 5));
        }
        30 => {
            s.fit = false;
            s.cell_size = 100.;
            s.frame();
            s.move_cursor(16, 10, false);
            history_key(s, false); // Keyboard edit, followed by guide recolour.
            ensure!(s.cursor == (6, 5));
            history_key(s, false);
            ensure!(s.editor.document.guides.strokes[0].color != [66, 165, 245]);
        }
        31 => {
            s.move_cursor(17, 10, false);
            history_key(s, true);
            ensure!(s.editor.document.guides.strokes[0].color == [66, 165, 245]);
            history_key(s, true);
            ensure!(s.cursor == (7, 5));
        }
        32 => {
            key(s, NamedKey::F8);
            click(s, Action::Tool(Tool::GuideErase))?;
            click(s, Action::ToggleEditMode)?;
            s.move_cursor(1, 0, true);
            key(s, NamedKey::Escape);
            ensure!(s.editor.selection.is_none());
            s.keyboard_input("b");
        }
        33 => {
            click(s, Action::ToggleEditMode)?;
            click(s, Action::Tool(Tool::Text))?;
            click(s, Action::ToggleEditMode)?;
            s.move_cursor(1, 0, true);
            key(s, NamedKey::Escape);
            ensure!(s.editor.selection.is_none());
            click(s, Action::ToggleEditMode)?;
            ensure!(s.active_mouse_tool() == Some(Tool::Text));
            click(s, Action::ToggleEditMode)?;
        }
        34 => {
            click(s, Action::Palette(3))?;
            click(s, Action::Foreground)?;
            drag(s, Control::Hue, (2. / 3., 0.5));
            drag(s, Control::Plane, (1., 0.8));
            ensure!(s.color_picker.rgb() == [0, 0, 51]);
        }
        35 => {
            click(s, Action::Submit)?;
            ensure!(s.editor.document.palette[3] == [0, 0, 51] && s.brush.cell.fg == [0, 0, 51]);
            history_key(s, false);
            ensure!(
                s.brush.cell.fg == s.editor.document.palette[3] && s.brush.cell.fg != [0, 0, 51]
            );
            history_key(s, true);
            ensure!(s.brush.cell.fg == [0, 0, 51]);
        }
        36 => {
            let mut doc = Document::new(32, 12).unwrap();
            for (row, (label, color)) in [
                ("PURE BLUE", [0, 0, 255]),
                ("DARK BLUE", [0, 0, 48]),
                ("NAVY BLUE", [8, 16, 64]),
                ("WHITE", [255; 3]),
                ("CYAN", [0, 180, 220]),
                ("RED", [200, 0, 0]),
            ]
            .into_iter()
            .enumerate()
            {
                for (x, ch) in format!("{label:10}  ⣿⣿  ███  ▓▒░").chars().enumerate()
                {
                    doc.set(
                        x as i32 + 1,
                        row as i32 * 2 + 1,
                        ditto::core::Cell {
                            glyph: ditto::font::glyph(ch).unwrap(),
                            fg: color,
                            bg: None,
                        },
                    );
                }
            }
            s.editor = Editor::new(doc);
            s.activate(Action::Shaders);
            click(s, Action::ShaderAdd(Kind::Chromatic))?;
            click(s, Action::ShaderPreviewFit)?;
            s.set_shader_value(1, 0.);
            project::export_png(
                &dir.join("chromatic-source.png"),
                &s.editor.document,
                4,
                None,
            )?;
        }
        37 => {
            let r = s.shader_slider_rect(1);
            s.mouse_move((r.x + 5., r.y + 11.));
            s.mouse_down(false);
            s.mouse_move((r.x + 17.5, r.y + 11.));
            ensure!(s.editor.pending());
            ensure!((s.editor.document.shaders.layers[0].params[0] - 2.8).abs() < 0.001);
        }
        38 => {
            s.mouse_up();
            ensure!(!s.editor.pending());
            history_key(s, false);
            ensure!(s.editor.document.shaders.layers[0].params[0] == 0.);
            history_key(s, true);
            ensure!((s.editor.document.shaders.layers[0].params[0] - 2.8).abs() < 0.001);
            project::export_png(
                &dir.join("chromatic-result.png"),
                &s.editor.document,
                4,
                None,
            )?;
            let path = dir.join("palette-sliders.ditto");
            project::save(&path, &s.editor.document)?;
            ensure!(project::load(&path)? == s.editor.document);
        }
        39 => {
            key(s, NamedKey::Escape);
            click(s, Action::CopyText)?;
            if let Some(Modal::Text { export }) = &s.modal {
                let path = dir.join("drawing.txt");
                export.save(&path)?;
                ensure!(std::fs::read_to_string(path)? == s.editor.document.text(None));
                std::fs::write(dir.join("copy-monospaced.html"), export.html())?;
            } else {
                anyhow::bail!("Missing text export");
            }
        }
        40 => {
            key(s, NamedKey::Escape);
            s.editor.edit(|doc| {
                doc.resize(80, 50).unwrap();
                for y in 14..50 {
                    for (x, ch) in format!("ROW {y:02}   ⣿⡖⢲⣆⣰   /\\__()[]{{}}_   ♥ ♦ ♣ ♠")
                        .chars()
                        .enumerate()
                    {
                        doc.set(
                            x as i32,
                            y,
                            ditto::core::Cell {
                                glyph: ditto::font::glyph(ch).unwrap(),
                                ..Default::default()
                            },
                        );
                    }
                }
            });
            click(s, Action::CopyText)?;
            click(s, Action::TextFormat(ditto::text_export::Format::Discord))?;
            click(s, Action::TextPart(1))?;
            if let Some(Modal::Text { export }) = &s.modal {
                let parts = export.parts.as_ref().map_err(|e| anyhow::anyhow!("{e}"))?;
                ensure!(parts.len() > 1 && export.part == 1);
                let mut bodies = Vec::new();
                for (i, part) in parts.iter().enumerate() {
                    ensure!(part.encode_utf16().count() <= 2000);
                    std::fs::write(dir.join(format!("discord-part-{}.txt", i + 1)), part)?;
                    bodies.push(
                        part.strip_prefix("```\n")
                            .unwrap()
                            .strip_suffix("\n```")
                            .unwrap(),
                    );
                }
                ensure!(bodies.join("\n") == s.editor.document.text(None));
                export.save(&dir.join("drawing-large.txt"))?;
            } else {
                anyhow::bail!("Missing text export");
            }
        }
        41 => {
            click(s, Action::TextFormat(ditto::text_export::Format::Markdown))?;
            click(s, Action::TextZoom(true))?;
            let r = s.text_preview_rect();
            s.mouse_move((r.x + r.w / 2., r.y + r.h / 2.));
            s.mouse_down(false);
            s.mouse_move((s.mouse.0 + 80., s.mouse.1 + 60.));
            s.mouse_up();
            ensure!(s.text_preview_pan != (0., 0.));
        }
        _ => return Ok(()),
    }
    s.frame();
    let name = name(frame).unwrap();
    let mut probes = Vec::new();
    if matches!(s.modal, Some(Modal::Color { .. })) {
        let r = color_picker::rect(s.width, Control::Plane);
        // Before/after swatch and hue strip are actual GPU output, checked below.
        probes.push((r.x + 475., 358., s.color_picker.rgb()));
        let h = color_picker::rect(s.width, Control::Hue);
        probes.push((h.x + h.w / 6., h.y + 11., [255, 255, 0]));
    } else if matches!(s.modal, Some(Modal::Shaders)) {
        let rect = s.shader_slider_rect(1);
        probes.push((rect.x + 50., rect.y + 10., s.theme().border));
        if frame >= 37 {
            probes.push((rect.x + 2., rect.y + 10., s.theme().accent));
        }
    } else if matches!(s.modal, Some(Modal::Text { .. })) {
        let r = s.text_preview_rect();
        probes.push((r.x + r.w - 5., r.y + 5., s.theme().canvas));
    } else {
        ensure!(s.keyboard_active(), "Keyboard blocked at {name}");
        ensure!(
            s.active_mouse_tool().is_none(),
            "Mouse tool active in keyboard mode at {name}"
        );
        let x = (s.cursor.0 as f32 * s.cell_width())
            .min(s.editor.document.width as f32 * s.cell_width() - 2.);
        let p = (
            s.origin.0 + x + 0.5,
            s.origin.1 + s.cursor.1 as f32 * s.cell_size + s.cell_size / 2.,
        );
        ensure!(s.canvas.contains(p), "Caret off canvas at {name}");
        probes.push((p.0, p.1, s.theme().accent));
        if frame == 35 {
            let r = s
                .hits
                .iter()
                .find(|h| h.action == Action::Palette(3))
                .unwrap()
                .rect;
            probes.push((r.x + 7., r.y + 7., [0, 0, 51]));
        }
        if (28..34).contains(&frame) {
            let left = s.origin.0 + x;
            let top = s.origin.1 + s.cursor.1 as f32 * s.cell_size;
            let width = if s.cursor.0 == s.editor.document.width as i32 {
                2.
            } else {
                s.cell_width()
            };
            // Check all four edges, not merely that some caret geometry exists.
            for (x, y) in [
                (left + width - 0.5, top + s.cell_size / 2.),
                (left + width / 2., top + 0.5),
                (left + width / 2., top + s.cell_size - 0.5),
            ] {
                ensure!(s.canvas.contains((x, y)), "Caret edge off canvas at {name}");
                probes.push((x, y, s.theme().accent));
            }
        }
    }
    std::fs::write(
        dir.join(format!("{name}.json")),
        serde_json::to_vec(&(s.width, s.height, probes))?,
    )?;
    Ok(())
}
pub fn verify(dir: &Path) -> Result<()> {
    for frame in 17..END {
        let name = name(frame).unwrap();
        type Probes = (f32, f32, Vec<(f32, f32, [u8; 3])>);
        let (w, h, probes): Probes =
            serde_json::from_slice(&std::fs::read(dir.join(format!("{name}.json")))?)?;
        let image = image::open(dir.join(format!("{name}.png")))?.to_rgba8();
        for (edge, (x, y, expected)) in probes.into_iter().enumerate() {
            let sx = x * image.width() as f32 / w;
            let sy = y * image.height() as f32 / h;
            let px = sx.floor() as u32;
            let py = sy.floor() as u32;
            let actual = image.get_pixel(px, py).0;
            let matches = |x, y| {
                image.get_pixel(x, y).0[..3]
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| a.abs_diff(b) <= 6)
            };
            // At 1x, a half-cell can put the centre of a one-pixel stroke
            // exactly between two physical pixel centres. Accept either tied
            // nearest pixel, only across the tested edge (never along it).
            // Away from an exact tie, ceil - 1 == floor: no search tolerance.
            let alternate = if (28..34).contains(&frame) && edge < 2 {
                ((sx.ceil() as u32).saturating_sub(1), py)
            } else if (28..34).contains(&frame) {
                (px, (sy.ceil() as u32).saturating_sub(1))
            } else {
                (px, py)
            };
            ensure!(
                matches(px, py) || matches(alternate.0, alternate.1),
                "{name}: pixel ({px},{py}) = {actual:?}, expected {expected:?}"
            );
        }
    }
    std::fs::write(
        dir.join("e2e-result.txt"),
        "PASS: picker hit testing, drag, keyboard, validation, mode cycles, focus, guides, shaders, zoom, cursor GPU pixels, project roundtrip and clean export; mouse-tool isolation, mixed guide/keyboard undo/redo, all four caret edges at viewport boundary\n",
    )?;
    Ok(())
}
