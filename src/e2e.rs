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

pub const END: u32 = 28;
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
            ensure!(s.keyboard_active() && s.tool == Tool::Guide);
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
    } else {
        ensure!(s.keyboard_active(), "Keyboard blocked at {name}");
        let x = (s.cursor.0 as f32 * s.cell_width())
            .min(s.editor.document.width as f32 * s.cell_width() - 2.);
        let p = (
            s.origin.0 + x + 0.5,
            s.origin.1 + s.cursor.1 as f32 * s.cell_size + s.cell_size / 2.,
        );
        ensure!(s.canvas.contains(p), "Caret off canvas at {name}");
        probes.push((p.0, p.1, s.theme().accent));
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
        for (x, y, expected) in probes {
            let px = (x * image.width() as f32 / w).floor() as u32;
            let py = (y * image.height() as f32 / h).floor() as u32;
            let actual = image.get_pixel(px, py).0;
            ensure!(
                actual[..3]
                    .iter()
                    .zip(expected)
                    .all(|(a, b)| a.abs_diff(b) <= 6),
                "{name}: pixel ({px},{py}) = {actual:?}, expected {expected:?}"
            );
        }
    }
    std::fs::write(
        dir.join("e2e-result.txt"),
        "PASS: picker hit testing, drag, keyboard, validation, mode cycles, focus, guides, shaders, zoom, cursor GPU pixels, project roundtrip and clean export\n",
    )?;
    Ok(())
}
