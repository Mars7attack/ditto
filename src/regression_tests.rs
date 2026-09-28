use super::*;
use app::{ColorTarget, Modal};
use color_picker::Control;
use ditto::{
    core::*,
    project,
    settings::Token,
    shaders::{Kind, Layer},
};

fn state() -> State {
    let mut s = State::new(PathBuf::new());
    s.modal = None;
    s.editor = Editor::new(Document::new(24, 16).unwrap());
    s.cursor = (7, 5);
    s.frame();
    s
}

#[test]
fn foreground_picker_edits_the_selected_swatch_and_history_keeps_brush_in_sync() {
    let mut s = state();
    s.editor.document.palette[4] = s.editor.document.palette[3];
    click(&mut s, Action::Palette(4));
    let original = s.editor.document.clone();
    click(&mut s, Action::Foreground);
    assert!(matches!(
        s.modal,
        Some(Modal::Color {
            target: ColorTarget::Palette(4),
            ..
        })
    ));
    s.text_input("003080");
    click(&mut s, Action::Cancel);
    assert_eq!(s.editor.document, original);
    click(&mut s, Action::Foreground);
    s.text_input("003080");
    click(&mut s, Action::Submit);
    assert_eq!(s.brush.cell.fg, [0, 48, 128]);
    assert_eq!(s.editor.document.palette[4], s.brush.cell.fg);
    assert_eq!(s.editor.document.palette[3], original.palette[3]);
    assert_eq!(s.editor.document.cells, original.cells);
    s.activate(Action::Undo);
    assert_eq!(s.editor.document, original);
    assert_eq!(s.brush.cell.fg, original.palette[4]);
    s.activate(Action::Redo);
    assert_eq!(s.brush.cell.fg, [0, 48, 128]);
    assert_eq!(s.selected_palette(), Some(4));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("palette.ditto");
    project::save(&path, &s.editor.document).unwrap();
    assert_eq!(project::load(&path).unwrap().palette[4], [0, 48, 128]);
}

#[test]
fn palette_editing_does_not_overwrite_other_swatches_or_sampled_colours() {
    let mut s = state();
    click(&mut s, Action::Palette(3));
    let fg = s.brush.cell.fg;
    s.activate(Action::EditPalette(7));
    s.text_input("112233");
    s.activate(Action::Submit);
    assert_eq!(s.brush.cell.fg, fg);
    s.pick((0, 0));
    assert!(s.selected_palette().is_none());
    let palette = s.editor.document.palette.clone();
    s.activate(Action::Foreground);
    s.text_input("442255");
    s.activate(Action::Submit);
    assert_eq!(s.editor.document.palette, palette);
    assert_eq!(s.brush.cell.fg, [68, 34, 85]);
    s.activate(Action::Undo);
    assert_eq!(s.brush.cell.fg, [68, 34, 85]);
    s.activate(Action::AddColor);
    assert_eq!(
        s.selected_palette(),
        Some(s.editor.document.palette.len() - 1)
    );
}

#[test]
fn every_shader_slider_previews_live_clamps_and_commits_one_undo() {
    for kind in ditto::shaders::KINDS {
        for parameter in 0..4 {
            let mut s = state();
            s.editor.document.shaders.layers.push(Layer::new(kind));
            s.activate(Action::Shaders);
            s.frame();
            let original = s.editor.document.clone();
            let spec = original.shaders.layers[0].parameter(parameter);
            let rect = s
                .hits
                .iter()
                .find(|h| h.action == Action::ShaderSlider(parameter))
                .unwrap()
                .rect;
            s.mouse_move((rect.x + rect.w * 0.37, rect.y + 11.));
            s.mouse_down(false);
            assert!(s.editor.pending());
            assert_eq!(s.editor.revision, 0);
            let mid = s.editor.document.shaders.layers[0].value(parameter);
            assert!((spec.min..=spec.max).contains(&mid));
            if spec.discrete {
                assert_eq!(mid.fract(), 0.);
            }
            s.mouse_move((rect.x - 200., rect.y - 200.));
            assert_eq!(
                s.editor.document.shaders.layers[0].value(parameter),
                spec.min
            );
            s.mouse_move((rect.x + rect.w + 200., rect.y + 200.));
            assert_eq!(
                s.editor.document.shaders.layers[0].value(parameter),
                spec.max
            );
            s.mouse_up();
            assert!(!s.editor.pending());
            if original.shaders.layers[0].value(parameter) != spec.max {
                assert_eq!(s.editor.revision, 1);
                s.activate(Action::Undo);
                assert_eq!(s.editor.document, original);
                assert!(!s.editor.undo());
                s.activate(Action::Redo);
                assert_eq!(
                    s.editor.document.shaders.layers[0].value(parameter),
                    spec.max
                );
            }
        }
    }
}

#[test]
fn shader_slider_cancellation_keyboard_and_layer_changes_do_not_leak_transactions() {
    let mut s = state();
    s.activate(Action::ShaderAdd(Kind::Chromatic));
    s.activate(Action::Shaders);
    s.frame();
    let original = s.editor.document.shaders.clone();
    let r = s.shader_slider_rect(1);
    for lose_focus in [false, true] {
        s.mouse_move((r.x + 85., r.y + 11.));
        s.mouse_down(false);
        assert_ne!(s.editor.document.shaders, original);
        if lose_focus {
            s.lost_focus();
        } else {
            key(&mut s, NamedKey::Escape);
        }
        assert_eq!(s.editor.document.shaders, original);
        assert!(matches!(s.modal, Some(Modal::Shaders)));
        s.mouse_move((r.x, r.y));
        s.mouse_up();
        assert_eq!(s.editor.document.shaders, original);
    }
    s.activate(Action::ShaderSlider(1));
    key(&mut s, NamedKey::ArrowRight);
    assert_eq!(s.editor.document.shaders.layers[0].params[0], 2.5);
    let before = s.editor.document.clone();
    s.set_shader_value(1, f32::NAN);
    assert_eq!(s.editor.document, before);
    s.mouse_move((r.x + 50., r.y + 11.));
    s.mouse_down(false);
    s.activate(Action::ShaderAdd(Kind::Glow));
    assert!(!s.editor.pending());
    let first = s.editor.document.shaders.layers[0].clone();
    s.mouse_move((r.x + 90., r.y));
    s.mouse_up();
    assert_eq!(s.editor.document.shaders.layers[0], first);
    s.activate(Action::Undo);
    assert_eq!(s.editor.document.shaders.layers.len(), 1);
    s.activate(Action::Undo);
    assert_eq!(s.editor.document, before);
}

#[test]
fn text_export_formats_preview_and_file_leave_the_document_unchanged() {
    let mut s = state();
    s.activate(Action::ToggleEditMode);
    s.editor.selection = Some(Rect {
        x: 2,
        y: 3,
        w: 12,
        h: 4,
    });
    let before = (s.editor.document.clone(), s.cursor, s.pan, s.cell_size);
    s.activate(Action::CopyText);
    s.frame();
    assert!(s.hits.iter().any(|h| h.action == Action::SaveText));
    let r = s.text_preview_rect();
    s.mouse_move((r.x + r.w / 2., r.y + r.h / 2.));
    route_scroll(&mut s, MouseScrollDelta::LineDelta(0., 2.), 1.);
    s.mouse_down(false);
    s.mouse_move((r.x + 50., r.y + 40.));
    s.mouse_up();
    assert_ne!(s.text_preview_pan, (0., 0.));
    click(
        &mut s,
        Action::TextFormat(ditto::text_export::Format::Discord),
    );
    if let Some(Modal::Text { export }) = &s.modal {
        assert_eq!((export.width, export.height), (12, 4));
        assert!(export.payload().unwrap().starts_with("```\n"));
    } else {
        panic!();
    }
    s.activate(Action::TextFit);
    assert_eq!(s.text_preview_pan, (0., 0.));
    assert_eq!(
        (s.editor.document.clone(), s.cursor, s.pan, s.cell_size),
        before
    );
    key(&mut s, NamedKey::Escape);
    assert!(s.modal.is_none());
    assert!(s.keyboard_active() && caret_is_drawn(&mut s));
    s.editor = Editor::new(Document::new(512, 512).unwrap());
    s.layout(1120., 720.);
    s.activate(Action::CopyText);
    let area = s.text_preview_rect();
    let board = s.text_preview_board();
    assert!(board.w <= area.w - 24. && board.h <= area.h - 24.);
    assert!(area.contains((board.x, board.y)));
    assert!(area.contains((board.x + board.w, board.y + board.h)));
}

#[test]
fn shader_sliders_expose_accessible_values_and_new_panels_fit_minimum_size() {
    use accesskit::{Action as A, ActionData, ActionRequest, NodeId, Role, TreeId};
    let mut s = state();
    s.layout(1120., 720.);
    s.activate(Action::ShaderAdd(Kind::Chromatic));
    s.activate(Action::Shaders);
    s.frame();
    let index = s
        .hits
        .iter()
        .position(|h| h.action == Action::ShaderSlider(1))
        .unwrap();
    let target = NodeId(index as u64 + 10);
    let tree = accessibility::tree(&s, 1.);
    let node = &tree.nodes.iter().find(|(id, _)| *id == target).unwrap().1;
    assert_eq!(node.role(), Role::Slider);
    assert_eq!(node.numeric_value(), Some(2.));
    accessibility::action(
        &mut s,
        ActionRequest {
            action: A::SetValue,
            target_tree: TreeId::ROOT,
            target_node: target,
            data: Some(ActionData::NumericValue(3.125)),
        },
    );
    assert_eq!(s.editor.document.shaders.layers[0].params[0], 3.125);
    s.activate(Action::Undo);
    assert_eq!(s.editor.document.shaders.layers[0].params[0], 2.);
    for text in [false, true] {
        if text {
            s.activate(Action::CopyText);
        }
        s.frame();
        for (i, a) in s.hits.iter().enumerate() {
            assert!(a.rect.x >= 0. && a.rect.x + a.rect.w <= s.width);
            assert!(a.rect.y >= 0. && a.rect.y + a.rect.h <= s.height);
            for b in &s.hits[i + 1..] {
                let r = a.rect.intersect(b.rect);
                assert!(
                    r.w <= 0. || r.h <= 0.,
                    "overlap: {:?} {:?}",
                    a.action,
                    b.action
                );
            }
        }
    }
}

#[test]
fn braille_alignment_toggle_is_explicit_shared_by_file_and_copy_and_preserves_document() {
    let mut s = state();
    s.editor.document.set(
        3,
        3,
        Cell {
            glyph: ditto::font::glyph('⡖').unwrap(),
            ..Cell::default()
        },
    );
    let before = s.editor.document.clone();
    s.layout(1120., 720.);
    s.activate(Action::CopyText);
    s.frame();
    assert!(
        s.hits
            .iter()
            .any(|h| h.action == Action::TextBrailleAlignment)
    );
    click(
        &mut s,
        Action::TextFormat(ditto::text_export::Format::Discord),
    );
    if let Some(Modal::Text { export }) = &s.modal {
        assert!(export.align_braille);
        assert!(export.output().contains('\u{2800}'));
        assert!(!export.output().contains(' '));
    } else {
        panic!();
    }
    click(&mut s, Action::TextBrailleAlignment);
    if let Some(Modal::Text { export }) = &s.modal {
        assert!(!export.align_braille);
        assert_eq!(export.output(), before.text(None));
        assert_eq!(export.preview(), before.text(None));
    } else {
        panic!();
    }
    click(&mut s, Action::TextBrailleAlignment);
    for (i, a) in s.hits.iter().enumerate() {
        for b in &s.hits[i + 1..] {
            let r = a.rect.intersect(b.rect);
            assert!(
                r.w <= 0. || r.h <= 0.,
                "overlap: {:?} {:?}",
                a.action,
                b.action
            );
        }
    }
    assert_eq!(s.editor.document, before);
    s.activate(Action::Cancel);
    assert_eq!(s.editor.document, before);
}
fn key(s: &mut State, key: NamedKey) {
    route_key(s, ModifiersState::empty(), Key::Named(key), None);
    s.frame();
}
fn click(s: &mut State, action: Action) {
    s.frame();
    let r = s.hits.iter().find(|h| h.action == action).unwrap().rect;
    s.mouse_move((r.x + r.w / 2., r.y + r.h / 2.));
    s.mouse_down(false);
    s.mouse_up();
    s.frame();
}
fn caret_is_drawn(s: &mut State) -> bool {
    let draw = s.frame();
    let x = (s.cursor.0 as f32 * s.cell_width())
        .min(s.editor.document.width as f32 * s.cell_width() - 2.);
    draw.has_solid_at(
        (
            s.origin.0 + x + 0.5,
            s.origin.1 + s.cursor.1 as f32 * s.cell_size + s.cell_size / 2.,
        ),
        s.theme().accent,
    )
}

const MOUSE_TOOLS: [Tool; 11] = [
    Tool::Pencil,
    Tool::Eraser,
    Tool::Recolor,
    Tool::Guide,
    Tool::GuideErase,
    Tool::Line,
    Tool::Rectangle,
    Tool::Fill,
    Tool::Text,
    Tool::Select,
    Tool::Pick,
];

#[test]
fn keyboard_escape_clears_selection_without_changing_the_remembered_mouse_tool() {
    for tool in MOUSE_TOOLS {
        let mut s = state();
        s.activate(Action::Tool(tool));
        click(&mut s, Action::ToggleEditMode);
        s.move_cursor(2, 0, true);
        assert!(s.editor.selection.is_some());
        key(&mut s, NamedKey::Escape);
        assert!(s.editor.selection.is_none(), "selection kept by {tool:?}");
        assert!(s.keyboard_active() && caret_is_drawn(&mut s));
        click(&mut s, Action::ToggleEditMode);
        assert_eq!(
            s.active_mouse_tool(),
            Some(tool),
            "keyboard Escape changed mouse tool"
        );
    }
}

#[test]
fn keyboard_ime_with_ui_focus_never_falls_through_to_the_mouse_text_tool() {
    for tool in MOUSE_TOOLS {
        let mut s = state();
        s.activate(Action::Tool(tool));
        s.activate(Action::ToggleEditMode);
        // A focused colour swatch can keep keyboard_canvas true; IME commits
        // must respect focus just like ordinary key events.
        s.focus = s.hits.iter().position(|h| h.action == Action::Foreground);
        assert!(s.focus.is_some());
        let before = (s.editor.document.clone(), s.cursor, s.editor.revision);
        s.ime_commit("A");
        assert_eq!(
            (s.editor.document.clone(), s.cursor, s.editor.revision),
            before,
            "{tool:?}"
        );
    }
}

#[test]
fn keyboard_history_keeps_restored_caret_in_view_after_navigation() {
    for tool in MOUSE_TOOLS {
        let mut s = state();
        s.activate(Action::Tool(tool));
        s.activate(Action::ToggleEditMode);
        s.fit = false;
        s.cell_size = 100.;
        s.frame();
        s.move_cursor(-7, -5, false);
        s.keyboard_input("a");
        s.move_cursor(22, 15, false);
        assert!(caret_is_drawn(&mut s));
        let command = if cfg!(target_os = "macos") {
            ModifiersState::SUPER
        } else {
            ModifiersState::CONTROL
        };
        route_key(&mut s, command, Key::Character("z".into()), None);
        assert_eq!(s.cursor, (0, 0));
        assert!(caret_is_drawn(&mut s), "undo hid caret after {tool:?}");
        s.move_cursor(23, 15, false);
        route_key(
            &mut s,
            command | ModifiersState::SHIFT,
            Key::Character("z".into()),
            None,
        );
        assert_eq!(s.cursor, (1, 0));
        assert!(caret_is_drawn(&mut s), "redo hid caret after {tool:?}");
    }
}

#[test]
fn keyboard_inputs_and_canvas_selection_are_identical_for_every_mouse_tool() {
    let mut expected = None;
    for tool in MOUSE_TOOLS {
        let mut s = state();
        s.editor
            .document
            .guides
            .start([1., 5.5], [80, 220, 160], 1.);
        s.editor.document.guides.append([20., 5.5]);
        s.activate(Action::Tool(tool));
        click(&mut s, Action::ToggleEditMode);
        let mut snapshots = Vec::new();
        let mut record = |s: &mut State| {
            assert!(s.active_mouse_tool().is_none());
            assert!(s.keyboard_active() && caret_is_drawn(s), "{tool:?}");
            snapshots.push((s.editor.document.clone(), s.cursor, s.editor.selection));
        };
        let start = s.cursor;
        s.mouse_move(point(&s, 15, 8));
        assert_eq!(s.cursor, start, "hover moved keyboard caret after {tool:?}");
        record(&mut s);
        stroke(&mut s, (3, 4), (6, 4)); // Always selects; never draws with the remembered tool.
        record(&mut s);
        key(&mut s, NamedKey::Escape);
        record(&mut s);
        for input in ["a", "b", "d", "t"] {
            route_key(
                &mut s,
                ModifiersState::empty(),
                Key::Character(input.into()),
                Some(input),
            );
            record(&mut s);
        }
        for input in [
            NamedKey::ArrowLeft,
            NamedKey::Backspace,
            NamedKey::Tab,
            NamedKey::Space,
            NamedKey::Enter,
            NamedKey::Delete,
        ] {
            key(&mut s, input);
            record(&mut s);
        }
        s.move_cursor(3, 0, true);
        record(&mut s);
        s.ime_commit("XY");
        record(&mut s);
        for action in [Action::Undo, Action::Undo, Action::Redo, Action::Redo] {
            s.activate(action);
            record(&mut s);
        }
        if let Some(expected) = &expected {
            assert!(
                &snapshots == expected,
                "keyboard behavior depends on {tool:?}"
            );
        } else {
            expected = Some(snapshots);
        }
        click(&mut s, Action::ToggleEditMode);
        assert_eq!(s.active_mouse_tool(), Some(tool));
    }
}

#[test]
fn keyboard_caret_is_drawn_after_every_mouse_tool_and_repeated_mode_switches() {
    for tool in [
        Tool::Pencil,
        Tool::Eraser,
        Tool::Recolor,
        Tool::Guide,
        Tool::GuideErase,
        Tool::Line,
        Tool::Rectangle,
        Tool::Fill,
        Tool::Text,
        Tool::Select,
        Tool::Pick,
    ] {
        let mut s = state();
        s.activate(Action::Tool(tool));
        for _ in 0..8 {
            s.activate(Action::ToggleEditMode);
            assert!(
                caret_is_drawn(&mut s),
                "keyboard caret hidden after {tool:?}"
            );
            assert!(s.keyboard_active());
            s.activate(Action::ToggleEditMode);
            assert_eq!(
                caret_is_drawn(&mut s),
                !matches!(tool, Tool::Guide | Tool::GuideErase)
            );
        }
    }
}

#[test]
fn guide_color_submit_cancel_and_parent_close_restore_keyboard_caret() {
    for accept in [false, true] {
        for keyboard_first in [false, true] {
            let mut s = state();
            s.activate(Action::Tool(Tool::Guide));
            if keyboard_first {
                s.activate(Action::ToggleEditMode);
            }
            key(&mut s, NamedKey::F8);
            click(&mut s, Action::GuideColor);
            s.text_input("42a5f5");
            click(
                &mut s,
                if accept {
                    Action::Submit
                } else {
                    Action::Cancel
                },
            );
            assert!(matches!(s.modal, Some(Modal::Guides)));
            key(&mut s, NamedKey::Escape);
            if !keyboard_first {
                s.activate(Action::ToggleEditMode);
            }
            assert!(caret_is_drawn(&mut s));
            key(&mut s, NamedKey::ArrowRight);
            assert_eq!(s.cursor, (8, 5));
            assert!(caret_is_drawn(&mut s));
        }
    }
}

#[test]
fn picker_is_shared_by_all_targets_and_draft_cancel_is_non_destructive() {
    for action in [
        Action::Foreground,
        Action::Background,
        Action::EditPalette(2),
        Action::GuideColor,
        Action::ShaderColor(0),
        Action::ShaderColor(1),
    ]
    .into_iter()
    .chain(Token::ALL.map(Action::ThemeColor))
    {
        let mut s = state();
        s.editor
            .document
            .shaders
            .layers
            .push(Layer::new(Kind::Duotone));
        s.activate(Action::Settings);
        let original = (
            s.editor.document.clone(),
            s.brush.cell,
            s.guide_color,
            s.settings_draft.clone(),
        );
        s.activate(action.clone());
        s.frame();
        assert!(
            s.hits
                .iter()
                .any(|h| h.action == Action::ColorControl(Control::Plane)),
            "{action:?}"
        );
        let r = color_picker::rect(s.width, Control::Plane);
        s.mouse_move((r.x + r.w * 0.7, r.y + r.h * 0.3));
        s.mouse_down(false);
        s.mouse_move((r.x + r.w, r.y));
        s.mouse_up();
        assert_eq!(s.editor.document, original.0);
        assert_eq!(s.brush.cell, original.1);
        assert_eq!(s.guide_color, original.2);
        click(&mut s, Action::Cancel);
        assert_eq!(s.settings_draft, original.3);
        s.activate(action);
        s.text_input("12ABEF");
        assert_eq!(s.color_picker.rgb(), [18, 171, 239]);
        let target = match s.modal {
            Some(Modal::Color { target, .. }) => target,
            _ => panic!(),
        };
        click(&mut s, Action::Submit);
        let actual = match target {
            ColorTarget::Foreground => s.brush.cell.fg,
            ColorTarget::Background => {
                assert!(s.brush.bg);
                s.brush.cell.bg.unwrap()
            }
            ColorTarget::Palette(i) => s.editor.document.palette[i],
            ColorTarget::Guide => s.guide_color,
            ColorTarget::Theme(t) => t.color(&s.settings_draft.as_ref().unwrap().theme),
            ColorTarget::Shader(i, c) => s.editor.document.shaders.layers[i].colors[c],
        };
        assert_eq!(actual, [18, 171, 239]);
        if matches!(target, ColorTarget::Palette(_) | ColorTarget::Shader(..)) {
            s.activate(Action::Undo);
            assert_eq!(s.editor.document, original.0);
            s.activate(Action::Redo);
        } else {
            assert_eq!(s.editor.document, original.0);
        }
    }
}

#[test]
fn picker_drag_clamps_and_stops_on_release_focus_loss_or_cancel() {
    for end in 0..3 {
        let mut s = state();
        s.activate(Action::Foreground);
        let r = color_picker::rect(s.width, Control::Plane);
        s.mouse_move((r.x + 100., r.y + 100.));
        s.mouse_down(false);
        s.mouse_move((r.x - 100., r.y - 100.));
        assert_eq!(s.color_picker.rgb(), [255; 3]);
        match end {
            0 => s.mouse_up(),
            1 => s.lost_focus(),
            _ => s.activate(Action::Cancel),
        }
        s.mouse_move((r.x + 200., r.y + 180.));
        assert_eq!(s.color_picker.rgb(), [255; 3]);
        assert!(!s.editor.dirty());
        assert!(s.gesture.is_none());
    }
}

#[test]
fn picker_hex_keyboard_navigation_validation_and_return_to_canvas() {
    let mut s = state();
    s.activate(Action::ToggleEditMode);
    s.activate(Action::Foreground);
    s.frame();
    s.text_input("00FF00");
    key(&mut s, NamedKey::Tab); // plane
    key(&mut s, NamedKey::ArrowDown);
    assert_eq!(s.color_picker.rgb(), [0, 252, 0]);
    key(&mut s, NamedKey::Tab); // hue
    key(&mut s, NamedKey::ArrowRight);
    assert!(matches!(&s.modal, Some(Modal::Color {value,..}) if value == &s.color_picker.hex()));
    key(&mut s, NamedKey::Tab); // hex
    s.text_input("ABC");
    let before = s.brush.cell;
    key(&mut s, NamedKey::Enter);
    assert!(matches!(s.modal, Some(Modal::Color { .. })));
    assert_eq!(s.brush.cell, before);
    s.input_replace = true;
    s.text_input("#112233");
    key(&mut s, NamedKey::Enter);
    assert_eq!(s.brush.cell.fg, [17, 34, 51]);
    assert!(s.keyboard_active());
    assert!(caret_is_drawn(&mut s));
}

#[test]
fn color_controls_fit_without_overlap_at_supported_sizes() {
    for (w, h) in [(1120., 720.), (1184., 832.), (1800., 1000.)] {
        let mut s = state();
        s.layout(w, h);
        s.activate(Action::Foreground);
        s.frame();
        for (i, a) in s.hits.iter().enumerate() {
            assert!(
                a.rect.x >= 0.
                    && a.rect.y >= 0.
                    && a.rect.x + a.rect.w <= w
                    && a.rect.y + a.rect.h <= h
            );
            for b in s.hits.iter().skip(i + 1) {
                let r = a.rect.intersect(b.rect);
                assert!(
                    r.w == 0. || r.h == 0.,
                    "{:?} overlaps {:?}",
                    a.action,
                    b.action
                );
            }
        }
    }
}

#[test]
fn caret_returns_from_all_panels_with_mouse_and_keyboard_confirmation() {
    for action in [
        Action::Help,
        Action::Guides,
        Action::Shaders,
        Action::Charsets,
        Action::Settings,
        Action::Resize,
        Action::Foreground,
        Action::Background,
        Action::EditPalette(0),
    ] {
        for accept in [false, true] {
            let mut s = state();
            s.activate(Action::Tool(Tool::GuideErase));
            s.activate(Action::ToggleEditMode);
            let before = s.editor.document.clone();
            s.activate(action.clone());
            s.frame();
            assert!(!caret_is_drawn(&mut s), "modal {action:?}");
            if accept {
                // Exercise focus set by keyboard, then activate the actual confirmation.
                let confirm = if action == Action::Charsets {
                    Action::SetCharset(3)
                } else {
                    Action::Submit
                };
                s.focus = s.hits.iter().position(|h| h.action == confirm);
                s.keyboard_canvas = false;
                if s.focus.is_some() {
                    key(&mut s, NamedKey::Enter);
                } else {
                    s.activate(confirm);
                }
            } else {
                key(&mut s, NamedKey::Tab);
                key(&mut s, NamedKey::Escape);
            }
            assert!(s.modal.is_none(), "{action:?}");
            assert!(s.keyboard_active(), "keyboard blocked after {action:?}");
            assert!(caret_is_drawn(&mut s), "cursor hidden after {action:?}");
            assert_eq!(s.editor.document, before);
        }
    }
}

#[test]
fn caret_survives_guides_shader_stack_themes_selection_ime_and_focus_loss() {
    for theme in 0..3 {
        for above in [false, true] {
            let mut s = state();
            s.settings.theme = ditto::settings::Theme::preset(theme);
            s.editor
                .document
                .guides
                .start([0., 5.5], s.theme().accent, 4.);
            s.editor.document.guides.append([24., 5.5]);
            s.editor.document.guides.above_characters = above;
            s.editor
                .document
                .shaders
                .layers
                .push(Layer::new(Kind::Glow));
            s.activate(Action::Tool(Tool::Guide));
            s.activate(Action::ToggleEditMode);
            s.activate(Action::SelectAll);
            assert!(caret_is_drawn(&mut s));
            s.ime = "é".into();
            assert!(caret_is_drawn(&mut s));
            s.lost_focus();
            assert!(caret_is_drawn(&mut s));
            key(&mut s, NamedKey::F4);
            assert!(caret_is_drawn(&mut s));
            key(&mut s, NamedKey::F6);
            assert!(s.keyboard_active());
            s.mouse_move((s.canvas.x - 20., s.canvas.y));
            assert!(caret_is_drawn(&mut s));
        }
    }
}

#[test]
fn caret_is_visible_at_edges_after_zoom_pan_resize_and_end_of_line() {
    let mut s = state();
    s.activate(Action::Tool(Tool::Guide));
    s.fit = false;
    s.cell_size = 60.;
    s.pan = (3000., -4000.);
    s.frame();
    s.activate(Action::ToggleEditMode);
    assert!(caret_is_drawn(&mut s));
    for (dx, dy) in [(100, 100), (-100, 0), (0, -100), (100, 0)] {
        s.move_cursor(dx, dy, false);
        assert!(caret_is_drawn(&mut s), "{:?}", s.cursor);
    }
    assert_eq!(s.cursor, (24, 0));
    s.activate(Action::Resize);
    s.input = 0;
    s.text_input("1");
    s.input = 1;
    s.input_replace = true;
    s.text_input("1");
    s.activate(Action::Submit);
    assert_eq!(s.cursor, (1, 0));
    assert!(caret_is_drawn(&mut s));
    s.activate(Action::Undo);
    s.activate(Action::Fit);
    assert!(caret_is_drawn(&mut s));
}

fn point(s: &State, x: i32, y: i32) -> (f32, f32) {
    (
        s.origin.0 + (x as f32 + 0.5) * s.cell_width(),
        s.origin.1 + (y as f32 + 0.5) * s.cell_size,
    )
}
fn stroke(s: &mut State, a: (i32, i32), b: (i32, i32)) {
    s.mouse_move(point(s, a.0, a.1));
    s.mouse_down(false);
    s.mouse_move(point(s, b.0, b.1));
    s.mouse_up();
}
#[test]
fn drawing_tools_use_single_undo_and_can_cancel_incomplete_gestures() {
    for tool in [
        Tool::Pencil,
        Tool::Line,
        Tool::Rectangle,
        Tool::Guide,
        Tool::Recolor,
        Tool::Eraser,
    ] {
        for cancel in [false, true] {
            let mut s = state();
            s.editor.document.set(
                2,
                2,
                Cell {
                    glyph: 65,
                    fg: [1, 2, 3],
                    ..Cell::default()
                },
            );
            let before = s.editor.document.clone();
            s.activate(Action::Tool(tool));
            s.mouse_move(point(&s, 2, 2));
            s.mouse_down(false);
            s.mouse_move(point(&s, 10, 6));
            if cancel {
                s.lost_focus();
                s.mouse_up();
                assert_eq!(s.editor.document, before);
                assert_eq!(s.editor.revision, 0);
            } else {
                s.mouse_up();
                let after = s.editor.document.clone();
                assert_ne!(after, before, "{tool:?}");
                s.activate(Action::Undo);
                assert_eq!(s.editor.document, before);
                assert!(!s.editor.undo());
                s.activate(Action::Redo);
                assert_eq!(s.editor.document, after);
            }
        }
    }
}

#[test]
fn fill_pick_text_selection_move_and_floating_paste_through_input_pipeline() {
    let mut s = state();
    s.activate(Action::Tool(Tool::Rectangle));
    s.filled = true;
    stroke(&mut s, (2, 2), (4, 4));
    assert_eq!(
        s.editor
            .document
            .cells
            .iter()
            .filter(|c| c.glyph != 32)
            .count(),
        9
    );
    s.activate(Action::Tool(Tool::Fill));
    s.brush.cell.glyph = 219;
    stroke(&mut s, (3, 3), (3, 3));
    assert_eq!(s.editor.document.get(2, 2).unwrap().glyph, 219);
    s.activate(Action::Tool(Tool::Pick));
    stroke(&mut s, (3, 3), (3, 3));
    assert_eq!(s.brush.cell.glyph, 219);
    s.activate(Action::Tool(Tool::Text));
    stroke(&mut s, (10, 1), (10, 1));
    s.text_input("AB");
    assert_eq!(s.editor.document.get(11, 1).unwrap().glyph, 66);
    s.backspace();
    assert_eq!(s.editor.document.get(11, 1).unwrap().glyph, 32);
    s.activate(Action::Tool(Tool::Select));
    stroke(&mut s, (2, 2), (4, 4));
    assert_eq!(
        s.editor.selection,
        Some(Rect {
            x: 2,
            y: 2,
            w: 3,
            h: 3
        })
    );
    stroke(&mut s, (3, 3), (8, 8));
    assert_eq!(
        s.editor.selection,
        Some(Rect {
            x: 7,
            y: 7,
            w: 3,
            h: 3
        })
    );
    assert_eq!(s.editor.document.get(2, 2).unwrap().glyph, 32);
    assert_eq!(s.editor.document.get(7, 7).unwrap().glyph, 219);
    let block = Block::copy(&s.editor.document, s.editor.selection.unwrap());
    s.floating = Some(app::Floating {
        block,
        position: (23, 15),
    });
    let before = s.editor.document.clone();
    s.enter();
    assert_eq!(s.editor.document, before);
    assert!(s.floating.is_some());
    s.floating.as_mut().unwrap().position = (0, 0);
    s.enter();
    assert!(s.floating.is_none());
    assert_eq!(s.editor.document.get(0, 0).unwrap().glyph, 219);
    s.activate(Action::Undo);
    assert_eq!(s.editor.document, before);
}

#[test]
fn guide_visibility_limits_clear_and_palette_cap_are_undoable() {
    let mut s = state();
    s.activate(Action::Tool(Tool::Guide));
    s.activate(Action::GuideVisible);
    stroke(&mut s, (2, 2), (4, 4));
    assert!(s.editor.document.guides.strokes.is_empty());
    s.activate(Action::GuideVisible);
    stroke(&mut s, (2, 2), (4, 4));
    for _ in 0..100 {
        s.activate(Action::GuideWidth(1));
        s.activate(Action::GuideOpacity(-1));
        s.activate(Action::RecolorSize(1));
    }
    assert_eq!(s.guide_width, 8.);
    assert_eq!(s.editor.document.guides.opacity, 0.05);
    assert_eq!(s.recolor_radius, 8);
    s.activate(Action::GuideClear);
    assert!(s.editor.document.guides.strokes.is_empty());
    s.activate(Action::Undo);
    assert_eq!(s.editor.document.guides.strokes.len(), 1);
    for _ in 0..40 {
        s.activate(Action::AddColor);
    }
    assert_eq!(s.editor.document.palette.len(), 32);
    let rev = s.editor.revision;
    s.activate(Action::AddColor);
    assert_eq!(s.editor.revision, rev);
    s.activate(Action::Undo);
    assert_eq!(s.editor.document.palette.len(), 31);
}

#[test]
fn shader_stack_actions_handle_limit_reorder_remove_toggle_reset_and_history() {
    let mut s = state();
    for kind in ditto::shaders::KINDS {
        s.activate(Action::ShaderAdd(kind));
    }
    assert_eq!(
        s.editor.document.shaders.layers.len(),
        ditto::shaders::MAX_LAYERS
    );
    let before = s.editor.document.clone();
    for action in [
        Action::ShaderMove(0, -1),
        Action::ShaderMove(7, 1),
        Action::ShaderRemove(99),
        Action::ShaderToggle(99),
    ] {
        s.activate(action);
        assert_eq!(s.editor.document, before);
    }
    s.activate(Action::ShaderMove(0, 1));
    assert_eq!(
        s.editor.document.shaders.layers[1],
        before.shaders.layers[0]
    );
    s.activate(Action::ShaderToggle(1));
    assert!(!s.editor.document.shaders.layers[1].enabled);
    s.activate(Action::ShaderRemove(1));
    assert_eq!(s.editor.document.shaders.layers.len(), 7);
    s.activate(Action::ShaderReset);
    assert!(s.editor.document.shaders.layers.is_empty());
    for _ in 0..4 {
        s.activate(Action::Undo);
    }
    assert_eq!(s.editor.document, before);
    for i in 0..ditto::shaders::PRESETS.len() {
        s.activate(Action::ShaderPreset(i));
        s.editor.document.shaders.validate().unwrap();
    }
}

fn finish_jobs(s: &mut State) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while s.busy || s.loading {
        assert!(
            Instant::now() < deadline,
            "asynchronous job timed out: {}",
            s.status
        );
        s.poll();
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn asynchronous_save_keeps_new_edits_dirty_and_recovery_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state();
    s.recovery = dir.path().join("recovery.ditto");
    s.path = Some(dir.path().join("saved.ditto"));
    stroke(&mut s, (2, 2), (4, 4));
    let saved = s.editor.document.clone();
    s.activate(Action::Save);
    s.editor.edit(|d| {
        d.set(
            0,
            0,
            Cell {
                glyph: 65,
                ..Cell::default()
            },
        )
    });
    finish_jobs(&mut s);
    assert!(s.editor.dirty());
    assert_eq!(
        ditto::project::load(s.path.as_ref().unwrap()).unwrap(),
        saved
    );
    s.last_change = Instant::now() - Duration::from_secs(4);
    s.poll();
    finish_jobs(&mut s);
    assert_eq!(
        ditto::project::load(&s.recovery).unwrap(),
        s.editor.document
    );
    let mut recovered = State::new(s.recovery.clone());
    assert!(matches!(recovered.modal, Some(Modal::Recovery)));
    recovered.activate(Action::Recover);
    finish_jobs(&mut recovered);
    assert_eq!(recovered.editor.document, s.editor.document);
    assert!(recovered.editor.dirty());
    assert!(recovered.path.is_none());
}

#[test]
fn corrupt_open_and_failed_save_preserve_document_and_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.ditto");
    std::fs::write(&bad, "broken").unwrap();
    let mut s = state();
    stroke(&mut s, (1, 1), (3, 2));
    let before = s.editor.document.clone();
    s.open_path(bad.clone());
    finish_jobs(&mut s);
    assert_eq!(s.editor.document, before);
    assert!(!s.loading);
    s.path = Some(bad.join("impossible.ditto"));
    s.activate(Action::Save);
    finish_jobs(&mut s);
    assert!(s.editor.dirty());
    assert_eq!(s.editor.document, before);
    assert_eq!(std::fs::read(&bad).unwrap(), b"broken");
}

#[test]
fn reference_import_transform_lock_opacity_fit_remove_and_history() {
    let dir = tempfile::tempdir().unwrap();
    let image = dir.path().join("reference.png");
    image::RgbaImage::from_pixel(8, 4, image::Rgba([10, 20, 30, 255]))
        .save(&image)
        .unwrap();
    let mut s = state();
    s.import_path(image);
    finish_jobs(&mut s);
    let original = s.editor.document.reference.clone().unwrap();
    assert!(original.locked);
    s.activate(Action::RefTransform);
    stroke(&mut s, (2, 2), (4, 4));
    assert_eq!(s.editor.document.reference.as_ref().unwrap(), &original);
    s.activate(Action::RefLock);
    s.activate(Action::RefTransform);
    s.activate(Action::RefNudge(2, 3));
    let moved = s.editor.document.reference.as_ref().unwrap();
    assert_eq!((moved.x, moved.y), (original.x + 2., original.y + 3.));
    s.activate(Action::RefOpacity(-1000));
    assert_eq!(s.editor.document.reference.as_ref().unwrap().opacity, 0.);
    s.activate(Action::RefOpacity(1000));
    assert_eq!(s.editor.document.reference.as_ref().unwrap().opacity, 1.);
    s.activate(Action::RefFit(true));
    assert!(!s.editor.document.reference.as_ref().unwrap().locked);
    s.activate(Action::RefRemove);
    assert!(s.editor.document.reference.is_none());
    s.activate(Action::Undo);
    assert!(s.editor.document.reference.is_some());
}

#[test]
fn accessibility_picker_has_focus_hex_value_and_adjustable_hue() {
    use accesskit::{Action as A, ActionData, ActionRequest, NodeId, TreeId};
    let mut s = state();
    s.activate(Action::Foreground);
    s.frame();
    let hue = s
        .hits
        .iter()
        .position(|h| h.action == Action::ColorControl(Control::Hue))
        .unwrap();
    let request = |action, target, data| ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: NodeId(target as u64 + 10),
        data,
    };
    accessibility::action(&mut s, request(A::Focus, hue, None));
    assert_eq!(s.focus, Some(hue));
    let before = s.color_picker.hue;
    accessibility::action(&mut s, request(A::Increment, hue, None));
    assert_ne!(s.color_picker.hue, before);
    let field = s
        .hits
        .iter()
        .position(|h| h.action == Action::Input(0))
        .unwrap();
    accessibility::action(
        &mut s,
        request(A::SetValue, field, Some(ActionData::Value("112233".into()))),
    );
    assert_eq!(s.color_picker.rgb(), [17, 34, 51]);
    let tree = accessibility::tree(&s, 2.);
    assert!(
        tree.nodes
            .iter()
            .any(|(_, n)| n.role() == accesskit::Role::Slider)
    );
}
