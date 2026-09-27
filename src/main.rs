mod accessibility;
mod app;
mod render;
mod ui;
use app::{Action, EditMode, State};
use ditto::core::Tool;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
    window::{Window, WindowId},
};
struct Ditto {
    window: Option<Arc<Window>>,
    renderer: Option<render::Renderer>,
    state: State,
    modifiers: ModifiersState,
    access: Option<accesskit_winit::Adapter>,
    proxy: winit::event_loop::EventLoopProxy<accesskit_winit::Event>,
    path: Option<PathBuf>,
    smoke: Option<PathBuf>,
    smoke_frame: u32,
    shader_quality: bool,
    failure: Option<String>,
}
impl ApplicationHandler<accesskit_winit::Event> for Ditto {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Ditto")
            .with_visible(false)
            .with_inner_size(winit::dpi::LogicalSize::new(1184., 832.))
            .with_min_inner_size(winit::dpi::LogicalSize::new(1120., 720.));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Création de la fenêtre"),
        );
        window.set_ime_allowed(true);
        self.access = Some(accesskit_winit::Adapter::with_event_loop_proxy(
            event_loop,
            &window,
            self.proxy.clone(),
        ));
        window.set_visible(true);
        let renderer = match pollster::block_on(render::Renderer::new(window.clone())) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Impossible d’initialiser le rendu : {e:#}");
                self.failure = Some(format!("Initialisation GPU : {e:#}"));
                event_loop.exit();
                return;
            }
        };
        eprintln!("Ditto {} ({})", ditto::VERSION, ditto::BUILD_ID);
        eprintln!("Ditto GPU: {}", renderer.adapter_name);
        eprintln!("Ditto font: {}", ditto::typeface::description());
        self.renderer = Some(renderer);
        self.window = Some(window.clone());
        if let Some(p) = self.path.take() {
            self.state.open_path(p);
        }
        window.request_redraw();
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone() else {
            return;
        };
        if let Some(a) = &mut self.access {
            a.process_event(&window, &event);
        }
        let mut redraw = true;
        match event {
            WindowEvent::CloseRequested => self.state.activate(Action::Quit),
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size.width, size.height);
                }
                let logical = size.to_logical::<f32>(window.scale_factor());
                self.state.layout(logical.width, logical.height);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = window.inner_size();
                if let Some(r) = &mut self.renderer {
                    r.resize(size.width, size.height);
                }
                let logical = size.to_logical::<f32>(window.scale_factor());
                self.state.layout(logical.width, logical.height);
            }
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            WindowEvent::Focused(false) => self.state.lost_focus(),
            WindowEvent::CursorMoved { position, .. } => {
                let p = position.to_logical::<f32>(window.scale_factor());
                self.state.mouse_move((p.x, p.y));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left || button == MouseButton::Right {
                    if state == ElementState::Pressed {
                        if self.modifiers.shift_key() {
                            if let Some(h) = self
                                .state
                                .hits
                                .iter()
                                .find(|h| h.rect.contains(self.state.mouse))
                                .cloned()
                            {
                                if let Action::Palette(i) = h.action {
                                    self.state.activate(Action::EditPalette(i));
                                } else {
                                    self.state.mouse_down(button == MouseButton::Right);
                                }
                            } else {
                                self.state.mouse_down(button == MouseButton::Right);
                            }
                        } else {
                            self.state.mouse_down(button == MouseButton::Right);
                        }
                    } else if button == MouseButton::Left {
                        self.state.mouse_up();
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                route_scroll(&mut self.state, delta, window.scale_factor());
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == Key::Named(NamedKey::Space) {
                    self.state.space = event.state == ElementState::Pressed
                        && self.state.tool != Tool::Text
                        && self.state.edit_mode == EditMode::Mouse
                        && self.state.modal.is_none();
                }
                if event.state == ElementState::Pressed {
                    self.key(event.physical_key, event.logical_key, event.text.as_deref());
                }
            }
            WindowEvent::Ime(Ime::Enabled | Ime::Disabled) => {}
            WindowEvent::Ime(Ime::Preedit(t, _)) => self.state.ime = t,
            WindowEvent::Ime(Ime::Commit(t)) => self.state.ime_commit(&t),
            WindowEvent::DroppedFile(p) => {
                if p.extension().is_some_and(|e| e == "ditto") {
                    if !self.state.editor.dirty() {
                        self.state.open_path(p);
                    } else {
                        self.state.status = "Enregistrer avant d’ouvrir un projet déposé.".into();
                    }
                } else if !self.state.busy {
                    self.state.import_path(p);
                }
            }
            WindowEvent::RedrawRequested => {
                self.state.poll();
                let size = window.inner_size().to_logical::<f32>(window.scale_factor());
                self.state.layout(size.width, size.height);
                let r = self.renderer.as_mut().unwrap();
                if let Some(dir) = &self.smoke {
                    let name = match (self.shader_quality, self.smoke_frame) {
                        (true, 2) => Some("quality-clean.png"),
                        (true, 3) => Some("quality-neutral.png"),
                        (true, 4) => Some("quality-glow.png"),
                        (false, 2) => Some("shaders-window.png"),
                        (false, 3) => Some("shaders-zoomed-window.png"),
                        (false, 4) => Some("braille-window.png"),
                        (false, 5) => Some("braille-spaced-blocks-window.png"),
                        (false, 6) => Some("workspace-guides.png"),
                        (false, 7) => Some("workspace-no-guides.png"),
                        (false, 8) => Some("guides-panel.png"),
                        (false, 9) => Some("settings-midnight.png"),
                        (false, 10) => Some("settings-paper.png"),
                        (false, 11) => Some("workspace-paper.png"),
                        (false, 12) => Some("guide-below-native.png"),
                        (false, 13) => Some("guide-above-native.png"),
                        (false, 14) => Some("guide-below-shader.png"),
                        (false, 15) => Some("guide-above-shader.png"),
                        (false, 16) => Some("guide-recolored.png"),
                        _ => None,
                    };
                    r.capture_next = name.map(|name| dir.join(name));
                }
                let needs_artwork = self.state.editor.document.shaders.active()
                    || matches!(self.state.modal, Some(app::Modal::Shaders));
                self.state.shader_error = None;
                if let Err(e) = r.set_artwork(&self.state.editor.document, needs_artwork) {
                    self.state.status = format!("Aperçu shaders impossible : {e:#}");
                    self.state.shader_error = Some(self.state.status.clone());
                    if self.smoke.is_some() {
                        self.failure = Some(self.state.status.clone());
                    }
                }
                let draw = self.state.frame();
                r.set_reference(
                    self.state
                        .editor
                        .document
                        .reference
                        .as_ref()
                        .map(|r| &r.asset),
                );
                match r.draw(draw, window.scale_factor() as f32) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        self.state.status = "Mémoire GPU insuffisante.".into();
                        self.failure = Some(self.state.status.clone());
                        event_loop.exit();
                    }
                    Err(e) => eprintln!("Rendu : {e}"),
                };
                window.set_title(&self.state.title());
                if let Some(a) = &mut self.access {
                    a.update_if_active(|| accessibility::tree(&self.state, window.scale_factor()));
                }
                redraw = false;
                if self.smoke.is_some() {
                    self.smoke_frame += 1;
                    if self.smoke_frame == 2 {
                        if self.shader_quality {
                            self.prepare_quality_smoke();
                        } else {
                            self.run_smoke();
                        }
                        window.request_redraw();
                    }
                    if self.smoke_frame == 3 {
                        if self.shader_quality {
                            let mut neutral =
                                ditto::shaders::Layer::new(ditto::shaders::Kind::Shine);
                            neutral.params[2] = 0.;
                            self.state.editor.document.shaders.layers = vec![neutral];
                        } else {
                            if let Err(e) = self.navigate_shader_preview_smoke() {
                                self.failure = Some(format!("Shader preview navigation: {e:#}"));
                            }
                        }
                        window.request_redraw();
                    }
                    if self.smoke_frame == 4 {
                        if self.shader_quality {
                            self.state.editor.document.shaders = ditto::shaders::preset(0);
                        } else {
                            self.state.modal = None;
                            self.state.activate(Action::SetCharset(5));
                            self.state.activate(Action::CharsetBank(4));
                        }
                        window.request_redraw();
                    }
                    if !self.shader_quality && self.smoke_frame == 5 {
                        self.state.activate(Action::CharsetBank(1));
                        window.request_redraw();
                    }
                    if !self.shader_quality && (6..=11).contains(&self.smoke_frame) {
                        let result = match self.smoke_frame {
                            6 => self.workspace_smoke(),
                            7 => {
                                self.state.activate(Action::GuideVisible);
                                Ok(())
                            }
                            8 => {
                                self.state.activate(Action::Undo);
                                self.state.activate(Action::Guides);
                                Ok(())
                            }
                            9 => {
                                self.state.activate(Action::Cancel);
                                self.state.activate(Action::Settings);
                                self.state.activate(Action::ThemePreset(1));
                                Ok(())
                            }
                            10 => {
                                self.state.activate(Action::ThemePreset(2));
                                Ok(())
                            }
                            11 => {
                                self.state.activate(Action::Submit);
                                let path = self.state.settings_path.as_ref().unwrap();
                                ditto::settings::load(path).and_then(|loaded| {
                                    anyhow::ensure!(
                                        loaded == self.state.settings,
                                        "settings roundtrip failed"
                                    );
                                    Ok(())
                                })
                            }
                            _ => unreachable!(),
                        };
                        if let Err(e) = result {
                            self.failure = Some(format!("Workspace smoke: {e:#}"));
                        }
                        window.request_redraw();
                    }
                    if !self.shader_quality && (12..=16).contains(&self.smoke_frame) {
                        match self.smoke_frame {
                            12 => self.prepare_guide_order_smoke(),
                            13 | 15 => self.state.activate(Action::GuideAbove),
                            14 => {
                                self.state.activate(Action::GuideAbove);
                                let mut neutral =
                                    ditto::shaders::Layer::new(ditto::shaders::Kind::Shine);
                                neutral.params[2] = 0.;
                                self.state.editor.document.shaders.layers = vec![neutral];
                            }
                            16 => {
                                self.state.guide_color = [248, 200, 48];
                                self.state.activate(Action::GuideRecolorAll);
                            }
                            _ => unreachable!(),
                        }
                        window.request_redraw();
                    }
                    if !self.shader_quality
                        && self.smoke_frame == 17
                        && let Err(e) = self.verify_guide_order_smoke()
                    {
                        self.failure = Some(format!("Guide compositing: {e:#}"));
                    }
                    if self.smoke_frame >= if self.shader_quality { 5 } else { 17 } {
                        if self.shader_quality
                            && self.smoke_frame == 5
                            && let Err(e) = self.verify_quality_smoke()
                        {
                            self.failure = Some(format!("Shader quality: {e:#}"));
                        }
                        if let Some(error) = &self.renderer.as_ref().unwrap().capture_error {
                            self.failure = Some(error.clone());
                        }
                        event_loop.exit();
                    }
                }
            }
            _ => redraw = false,
        }
        if self.state.quit {
            event_loop.exit();
            return;
        }
        if redraw {
            window.request_redraw();
        }
    }
    fn user_event(&mut self, _: &ActiveEventLoop, event: accesskit_winit::Event) {
        match event.window_event {
            accesskit_winit::WindowEvent::ActionRequested(r) => {
                accessibility::action(&mut self.state, r)
            }
            accesskit_winit::WindowEvent::InitialTreeRequested => {}
            accesskit_winit::WindowEvent::AccessibilityDeactivated => return,
        }
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.poll()
            && let Some(w) = &self.window
        {
            w.request_redraw();
        }
        if self.state.quit {
            event_loop.exit();
            return;
        }
        if self.state.needs_timer() || self.smoke.is_some() {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(50),
            ));
            if self.smoke.is_some()
                && let Some(w) = &self.window
            {
                w.request_redraw();
            }
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
impl Ditto {
    fn prepare_guide_order_smoke(&mut self) {
        use ditto::core::{Asset, Cell, Document, Editor, Reference};
        let mut doc = Document::new(8, 4).unwrap();
        let asset = Arc::new(Asset {
            width: 16,
            height: 16,
            rgba: [24, 72, 148, 255].repeat(256),
        });
        let mut reference = Reference::fit(asset, 8, 4, true);
        reference.opacity = 1.;
        doc.reference = Some(reference);
        doc.set(
            3,
            1,
            Cell {
                glyph: 219,
                fg: [220, 40, 80],
                bg: None,
            },
        );
        doc.guides.opacity = 1.;
        doc.guides.start([0.5, 1.5], [40, 220, 120], 0.5);
        doc.guides.append([7.5, 1.5]);
        let s = &mut self.state;
        s.modal = None;
        s.editor = Editor::new(doc);
        s.fit = false;
        s.cell_size = 80.;
        s.pan = (0., 0.);
        s.trace = false;
        s.grid = false;
        s.tool = Tool::Pencil;
        s.cursor = (0, 0);
        s.mouse = (0., 0.);
        s.layout(s.width, s.height);
        s.status = "Référence bleue · caractère rouge · guide vert".into();
    }
    fn verify_guide_order_smoke(&self) -> anyhow::Result<()> {
        let s = &self.state;
        let dir = self.smoke.as_ref().unwrap();
        let dpi = self.window.as_ref().unwrap().scale_factor() as f32;
        let sample = |im: &image::RgbaImage, x: f32, y: f32| -> [u8; 3] {
            let x = ((s.origin.0 + x * s.cell_width()) * dpi).floor() as u32;
            let y = ((s.origin.1 + y * s.cell_size) * dpi).floor() as u32;
            let p = im.get_pixel(x, y);
            [p[0], p[1], p[2]]
        };
        let mut report = String::new();
        for (file, ink, over) in [
            ("guide-below-native.png", [40, 220, 120], false),
            ("guide-above-native.png", [40, 220, 120], true),
            ("guide-below-shader.png", [40, 220, 120], false),
            ("guide-above-shader.png", [40, 220, 120], true),
            ("guide-recolored.png", [248, 200, 48], true),
        ] {
            let im = image::open(dir.join(file))?.into_rgba8();
            let reference = sample(&im, 1.5, 2.5);
            let on_reference = sample(&im, 1.5, 1.5);
            let on_character = sample(&im, 3.5, 1.5);
            let expect = if over { ink } else { [220, 40, 80] };
            let near = |actual: [u8; 3], expected: [u8; 3]| {
                actual
                    .into_iter()
                    .zip(expected)
                    .all(|(a, b)| a.abs_diff(b) <= 2)
            };
            anyhow::ensure!(
                near(reference, [24, 72, 148]),
                "{file}: reference pixel {reference:?}"
            );
            anyhow::ensure!(
                near(on_reference, ink),
                "{file}: guide is not above reference: {on_reference:?}"
            );
            anyhow::ensure!(
                near(on_character, expect),
                "{file}: wrong guide/character order: {on_character:?}"
            );
            report += &format!(
                "{file}: reference={reference:?}, guide/reference={on_reference:?}, guide/character={on_character:?} OK\n"
            );
        }
        let mut art = s.editor.document.clone();
        art.reference = None;
        art.guides = ditto::guides::Layer::default();
        anyhow::ensure!(
            ditto::project::render_png(&art, 2, None)?
                == ditto::project::render_png(&s.editor.document, 2, None)?,
            "guide order/ink leaked into export"
        );
        ditto::project::save(&dir.join("guide-order.ditto"), &s.editor.document)?;
        anyhow::ensure!(ditto::project::load(&dir.join("guide-order.ditto"))? == s.editor.document);
        std::fs::write(dir.join("guide-compositing.txt"), report)?;
        eprintln!(
            "GUIDE COMPOSITING OK: reference < guide < character, toggle, shaders, global recolor, clean export"
        );
        Ok(())
    }
    fn workspace_smoke(&mut self) -> anyhow::Result<()> {
        use ditto::core::{Cell, Document, Editor};
        let s = &mut self.state;
        let mut doc = Document::new(40, 28).unwrap();
        for y in 0..28 {
            for x in 0..40 {
                let r = ((x as f32 - 19.5) / 14.).powi(2) + ((y as f32 - 13.5) / 9.).powi(2);
                if r < 1. {
                    doc.set(
                        x,
                        y,
                        Cell {
                            glyph: if r > 0.75 {
                                178
                            } else if (x + y) % 3 == 0 {
                                177
                            } else {
                                176
                            },
                            fg: [183, 155, 206],
                            bg: None,
                        },
                    );
                }
            }
        }
        for (i, c) in "d i t t o".chars().enumerate() {
            doc.set(
                15 + i as i32,
                13,
                Cell {
                    glyph: ditto::font::glyph(c).unwrap(),
                    fg: [242, 227, 201],
                    bg: Some([28, 25, 37]),
                },
            );
        }
        s.modal = None;
        s.editor = Editor::new(doc);
        s.fit = true;
        s.grid = false;
        s.pan = (0., 0.);
        s.layout(s.width, s.height);
        s.activate(Action::Tool(Tool::Recolor));
        s.recolor_radius = 2;
        s.brush.cell.fg = [112, 215, 188];
        let point = |s: &State, x: f32, y: f32| {
            (
                s.origin.0 + x * s.cell_width(),
                s.origin.1 + y * s.cell_size,
            )
        };
        let original = s.editor.document.clone();
        s.mouse_move(point(s, 13.5, 5.5));
        s.mouse_down(false);
        s.mouse_move(point(s, 13.5, 22.5));
        s.mouse_up();
        let repainted = s.editor.document.clone();
        anyhow::ensure!(original.cells != repainted.cells);
        anyhow::ensure!(
            original
                .cells
                .iter()
                .zip(repainted.cells.iter())
                .all(|(a, b)| a.glyph == b.glyph && a.bg == b.bg)
        );
        s.activate(Action::Undo);
        anyhow::ensure!(s.editor.document == original);
        s.activate(Action::Redo);
        anyhow::ensure!(s.editor.document == repainted);
        let clean = ditto::project::render_png(&s.editor.document, 2, None)?;
        let cells = s.editor.document.cells.clone();
        s.activate(Action::Tool(Tool::Guide));
        s.guide_width = 0.2;
        s.guide_color = [236, 167, 102];
        for i in 0..=96 {
            let theta = i as f32 * std::f32::consts::TAU / 96.;
            s.mouse_move(point(s, 20. + theta.cos() * 16., 14. + theta.sin() * 10.5));
            if i == 0 {
                s.mouse_down(false);
            }
        }
        s.mouse_up();
        s.guide_color = [116, 198, 223];
        s.mouse_move(point(s, 20., 1.));
        s.mouse_down(false);
        s.mouse_move(point(s, 20., 27.));
        s.mouse_up();
        anyhow::ensure!(Arc::ptr_eq(&cells, &s.editor.document.cells));
        anyhow::ensure!(
            ditto::project::render_png(&s.editor.document, 2, None)? == clean,
            "guides leaked into export"
        );
        let layer = s.editor.document.guides.clone();
        s.activate(Action::Tool(Tool::GuideErase));
        s.mouse_move(point(s, 18., 2.));
        s.mouse_down(false);
        s.mouse_move(point(s, 22., 2.));
        s.mouse_up();
        anyhow::ensure!(s.editor.document.guides.strokes.len() < layer.strokes.len());
        s.activate(Action::Undo);
        anyhow::ensure!(s.editor.document.guides == layer);
        let dir = self.smoke.as_ref().unwrap();
        ditto::project::save(&dir.join("workspace.ditto"), &s.editor.document)?;
        anyhow::ensure!(ditto::project::load(&dir.join("workspace.ditto"))? == s.editor.document);
        clean.save(dir.join("workspace-export.png"))?;
        s.activate(Action::Tool(Tool::Recolor));
        s.mouse_move(point(s, 13.5, 8.5));
        s.status = "C : recolorer · D : guides · F8 : calque · Cmd/Ctrl , : réglages".into();
        eprintln!(
            "WORKSPACE OK: recolor preserves glyph/background, guides undo/erase/save/load, clean export"
        );
        Ok(())
    }
    fn navigate_shader_preview_smoke(&mut self) -> anyhow::Result<()> {
        let s = &mut self.state;
        let canvas = (s.cell_size, s.pan, s.cursor);
        let doc = s.editor.document.clone();
        s.activate(Action::ShaderPreviewActual);
        let r = s.shader_preview_rect();
        let center = (r.x + r.w / 2., r.y + r.h / 2.);
        s.mouse_move(center);
        route_scroll(s, MouseScrollDelta::LineDelta(0., 4.), 1.);
        let zoom = s.shader_preview_scale();
        anyhow::ensure!(zoom > 1.2 && zoom < 1.23);
        s.mouse_down(false);
        s.mouse_move((center.0 + 224. * zoom, center.1 + 208. * zoom));
        s.mouse_up();
        let view = (s.shader_preview_zoom, s.shader_preview_pan);
        s.activate(Action::ShaderBefore);
        s.activate(Action::ShaderBefore);
        s.activate(Action::ShaderAdjust(0, -1));
        s.activate(Action::Undo);
        anyhow::ensure!((s.shader_preview_zoom, s.shader_preview_pan) == view);
        anyhow::ensure!((s.cell_size, s.pan, s.cursor) == canvas && s.editor.document == doc);
        anyhow::ensure!(matches!(s.modal, Some(app::Modal::Shaders)));
        eprintln!(
            "SHADER PREVIEW NAVIGATION OK: zoom {:.1}%, pan {:?}, canvas/document unchanged",
            zoom * 100.,
            s.shader_preview_pan
        );
        Ok(())
    }
    fn prepare_quality_smoke(&mut self) {
        let mut doc = ditto::core::Document::new(16, 7).unwrap();
        for (row, text) in ["SF Mono @&!?", "Aa Bb 012345", "()/\\ %éàgjQ", "░▒▓█▀▄▌▐⣿⡇"]
            .into_iter()
            .enumerate()
        {
            for (column, c) in text.chars().enumerate() {
                doc.set(
                    column as i32 + 2,
                    if row == 3 { 5 } else { row as i32 + 1 },
                    ditto::core::Cell {
                        glyph: ditto::font::glyph(c).unwrap(),
                        fg: [230, 201, 172],
                        bg: None,
                    },
                );
            }
        }
        self.state.modal = None;
        self.state.editor = ditto::core::Editor::new(doc);
        self.state.grid = false;
        self.state.fit = false;
        self.state.cell_size = 64.;
        self.state.pan = (0., 0.);
        self.state.layout(self.state.width, self.state.height);
        std::fs::create_dir_all(self.smoke.as_ref().unwrap()).unwrap();
    }
    fn verify_quality_smoke(&self) -> anyhow::Result<()> {
        let dir = self.smoke.as_ref().unwrap();
        let clean = image::open(dir.join("quality-clean.png"))?.into_rgba8();
        let processed = image::open(dir.join("quality-neutral.png"))?.into_rgba8();
        let dpi = self.window.as_ref().unwrap().scale_factor() as f32;
        let s = &self.state;
        let x0 = ((s.origin.0 + s.cell_width()) * dpi).ceil() as u32;
        let y0 = ((s.origin.1 + s.cell_size) * dpi).ceil() as u32;
        let x1 = ((s.origin.0 + s.cell_width() * 15.) * dpi).floor() as u32;
        let y1 = ((s.origin.1 + s.cell_size * 6.) * dpi).floor() as u32;
        anyhow::ensure!(
            clean.dimensions() == processed.dimensions()
                && x1 <= clean.width()
                && y1 <= clean.height(),
            "invalid quality capture dimensions"
        );
        let mut worst = 0;
        let mut row_worst = [0; 5];
        let mut interior_worst = [0; 5];
        let mut total = 0u64;
        for y in y0..y1 {
            for x in x0..x1 {
                for k in 0..3 {
                    let difference =
                        clean.get_pixel(x, y)[k].abs_diff(processed.get_pixel(x, y)[k]);
                    worst = worst.max(difference);
                    let row = ((y - y0) as f32 / (s.cell_size * dpi)) as usize;
                    row_worst[row.min(4)] = row_worst[row.min(4)].max(difference);
                    let cx = (x as f32 + 0.5 - s.origin.0 * dpi).rem_euclid(s.cell_width() * dpi);
                    let cy = (y as f32 + 0.5 - s.origin.1 * dpi).rem_euclid(s.cell_size * dpi);
                    // The postprocessed canvas filters across cell boundaries;
                    // direct glyph quads clip there. Exclude that one-texel rim.
                    let rim = s.cell_size * dpi / ditto::typeface::GLYPH_HEIGHT as f32;
                    if cx >= rim
                        && cx <= s.cell_width() * dpi - rim
                        && cy >= rim
                        && cy <= s.cell_size * dpi - rim
                    {
                        interior_worst[row.min(4)] = interior_worst[row.min(4)].max(difference);
                    }
                    total += u64::from(difference);
                }
            }
        }
        let mean = total as f64 / f64::from((x1 - x0) * (y1 - y0) * 3);
        let report = format!(
            "GPU: {}\nDPI: {dpi}\nCell logical height: {}\nFull fixture: max {worst}/255, mean {mean:.6}/255\nRows (text, text, text, gap, tiles): {row_worst:?}\nGlyph interiors (excluding cell-boundary filtering): {interior_worst:?}\n",
            self.renderer.as_ref().unwrap().adapter_name,
            s.cell_size
        );
        std::fs::write(dir.join("quality.txt"), &report)?;
        eprintln!("{report}");
        anyhow::ensure!(
            interior_worst.iter().all(|v| *v <= 3),
            "neutral shader degraded font contours"
        );
        ditto::project::save(&dir.join("quality.ditto"), &s.editor.document)?;
        ditto::project::export_png(&dir.join("quality.png"), &s.editor.document, 4, None)?;
        eprintln!("SHADER QUALITY OK");
        Ok(())
    }
    fn key(&mut self, physical: PhysicalKey, key: Key, text: Option<&str>) {
        route_keyboard_event(&mut self.state, self.modifiers, physical, key, text);
    }
    fn run_smoke(&mut self) {
        let Some(dir) = self.smoke.clone() else {
            return;
        };
        let result = (|| -> anyhow::Result<()> {
            std::fs::create_dir_all(&dir)?;
            self.state.modal = None;
            self.state.activate(Action::Tool(Tool::Rectangle));
            self.state.cursor = (3, 3);
            self.state.enter();
            self.state.cursor = (28, 24);
            self.state.enter();
            self.state.activate(Action::Tool(Tool::Text));
            self.state.cursor = (8, 10);
            self.state.text_origin = 8;
            self.state.text_input("DITTO");
            self.state.activate(Action::ToggleEditMode);
            self.state.cursor = (8, 12);
            for key in ["a", "1", "z", "9"] {
                let (physical, printed) = match key {
                    "1" => (KeyCode::Digit1, "&"),
                    "9" => (KeyCode::Digit9, "ç"),
                    "a" => (KeyCode::KeyQ, "a"),
                    _ => (KeyCode::KeyW, "z"),
                };
                route_keyboard_event(
                    &mut self.state,
                    ModifiersState::empty(),
                    PhysicalKey::Code(physical),
                    Key::Character(printed.into()),
                    Some(printed),
                );
            }
            anyhow::ensure!(
                self.state.cursor == (12, 12),
                "keyboard caret did not advance"
            );
            self.state.activate(Action::SetCharset(5));
            let command = if cfg!(target_os = "macos") {
                ModifiersState::SUPER
            } else {
                ModifiersState::CONTROL
            };
            route_key(
                &mut self.state,
                command,
                Key::Named(NamedKey::ArrowUp),
                None,
            );
            anyhow::ensure!(
                self.state.charset_bank == self.state.active_charset().banks() - 1,
                "charset shortcut did not wrap"
            );
            let zoom_before = self.state.cell_size;
            route_scroll(&mut self.state, MouseScrollDelta::LineDelta(0., 1.), 1.);
            anyhow::ensure!(
                self.state.cell_size > zoom_before && self.state.cell_size < zoom_before * 1.06,
                "scroll zoom is too coarse"
            );
            self.state.cursor = (8, 13);
            self.state.keyboard_input("a");
            anyhow::ensure!(
                self.state
                    .editor
                    .document
                    .cells
                    .iter()
                    .any(|c| c.glyph >= 256),
                "extended glyphs were not inserted"
            );
            let doc = &self.state.editor.document;
            ditto::project::save(&dir.join("smoke.ditto"), doc)?;
            let reload = ditto::project::load(&dir.join("smoke.ditto"))?;
            anyhow::ensure!(*doc == reload);
            ditto::project::export_png(&dir.join("smoke.png"), doc, 1, None)?;
            std::fs::write(dir.join("smoke.txt"), doc.text(None))?;
            self.state.activate(Action::Shaders);
            self.state.activate(Action::ShaderPreset(0));
            self.state
                .activate(Action::ShaderAdd(ditto::shaders::Kind::Scanlines));
            self.state.activate(Action::ShaderAdjust(0, -1));
            self.state.activate(Action::ShaderMove(2, -1));
            self.state.activate(Action::Undo);
            self.state.activate(Action::Redo);
            self.state
                .activate(Action::ShaderAdd(ditto::shaders::Kind::Blur));
            for _ in 0..3 {
                self.state.activate(Action::ShaderAdjust(1, -1));
            }
            self.state
                .activate(Action::ShaderAdd(ditto::shaders::Kind::ContourBlur));
            let doc = &self.state.editor.document;
            ditto::project::save(&dir.join("shaders.ditto"), doc)?;
            anyhow::ensure!(ditto::project::load(&dir.join("shaders.ditto"))? == *doc);
            ditto::project::export_png(&dir.join("shaders.png"), doc, 2, None)?;
            std::fs::write(
                dir.join("runtime.txt"),
                format!(
                    "GPU: {}\nProject round-trip: OK\nShader stack / undo / redo / export: OK\nCells: {}\n",
                    self.renderer.as_ref().unwrap().adapter_name,
                    doc.cells.len()
                ),
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => eprintln!("SMOKE OK: {}", dir.display()),
            Err(e) => {
                eprintln!("SMOKE FAILED: {e:#}");
                self.failure = Some(format!("Smoke failed: {e:#}"));
            }
        }
    }
}
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mut path = None;
    let mut smoke = None;
    let mut shader_quality = false;
    let mut recovery_override = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--smoke-dir" => smoke = args.next().map(PathBuf::from),
            "--shader-quality-smoke-dir" => {
                smoke = args.next().map(PathBuf::from);
                shader_quality = true;
            }
            "--recovery-path" => recovery_override = args.next().map(PathBuf::from),
            "--version" | "-V" => {
                println!("Ditto {} ({})", ditto::VERSION, ditto::BUILD_ID);
                return Ok(());
            }
            "--help" => {
                println!(
                    "Ditto [projet.ditto] [--smoke-dir dossier]\nÉditeur natif de caractères. F1 : aide."
                );
                return Ok(());
            }
            _ => path = Some(PathBuf::from(a)),
        }
    }
    let recovery = if let Some(p) = recovery_override {
        p
    } else if let Some(dir) = &smoke {
        dir.join("recovery.ditto")
    } else {
        app::recovery_path()
    };
    let event_loop = EventLoop::<accesskit_winit::Event>::with_user_event().build()?;
    let mut state = State::new(recovery);
    state.load_settings(
        smoke
            .as_ref()
            .map(|p| p.join("settings.json"))
            .unwrap_or_else(ditto::settings::path),
    );
    let mut app = Ditto {
        window: None,
        renderer: None,
        state,
        modifiers: ModifiersState::empty(),
        access: None,
        proxy: event_loop.create_proxy(),
        path,
        smoke,
        smoke_frame: 0,
        shader_quality,
        failure: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(failure) = app.failure {
        anyhow::bail!(failure);
    }
    Ok(())
}

// Letters follow the active layout; number-row positions work without Shift on AZERTY.
// Only the drawing canvas uses this shortcut: fields and IME keep their original input.
fn route_keyboard_event(
    state: &mut State,
    modifiers: ModifiersState,
    physical: PhysicalKey,
    key: Key,
    text: Option<&str>,
) {
    if state.keyboard_active()
        && !modifiers
            .intersects(ModifiersState::SUPER | ModifiersState::CONTROL | ModifiersState::ALT)
    {
        let digit = match physical {
            PhysicalKey::Code(KeyCode::Digit0) => Some("0"),
            PhysicalKey::Code(KeyCode::Digit1) => Some("1"),
            PhysicalKey::Code(KeyCode::Digit2) => Some("2"),
            PhysicalKey::Code(KeyCode::Digit3) => Some("3"),
            PhysicalKey::Code(KeyCode::Digit4) => Some("4"),
            PhysicalKey::Code(KeyCode::Digit5) => Some("5"),
            PhysicalKey::Code(KeyCode::Digit6) => Some("6"),
            PhysicalKey::Code(KeyCode::Digit7) => Some("7"),
            PhysicalKey::Code(KeyCode::Digit8) => Some("8"),
            PhysicalKey::Code(KeyCode::Digit9) => Some("9"),
            _ => None,
        };
        if let Some(digit) = digit {
            state.keyboard_input(digit);
            return;
        }
    }
    route_key(state, modifiers, key, text);
}

fn route_scroll(state: &mut State, delta: MouseScrollDelta, scale_factor: f64) {
    if state.loading {
        return;
    }
    // Equal travel gives equal zoom, regardless of event rate or Retina scaling.
    let amount = match delta {
        MouseScrollDelta::LineDelta(_, y) => y as f64 * 0.05,
        MouseScrollDelta::PixelDelta(p) => p.to_logical::<f64>(scale_factor).y * 0.001,
    };
    if amount.is_finite() {
        let factor = amount.clamp(-10., 10.).exp() as f32;
        match state.modal {
            Some(app::Modal::Shaders) if state.shader_preview_rect().contains(state.mouse) => {
                state.zoom_shader_preview(factor, state.mouse)
            }
            None => state.zoom_by(factor, state.mouse),
            _ => {}
        }
    }
}

fn route_key(state: &mut State, modifiers: ModifiersState, key: Key, text: Option<&str>) {
    let command = if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    };
    if state.loading {
        return;
    }
    if !state.ime.is_empty() {
        return;
    }
    if matches!(state.modal, Some(app::Modal::Shaders)) {
        if !modifiers.alt_key()
            && (command || (!modifiers.control_key() && !modifiers.super_key()))
            && let Key::Character(c) = &key
        {
            let action = match c.as_str() {
                "+" | "=" => Some(Action::ShaderPreviewZoom(true)),
                "-" => Some(Action::ShaderPreviewZoom(false)),
                "0" => Some(Action::ShaderPreviewFit),
                "1" => Some(Action::ShaderPreviewActual),
                _ => None,
            };
            if let Some(action) = action {
                state.activate(action);
                return;
            }
        }
        if modifiers.alt_key() && !command && !modifiers.control_key() && !modifiers.super_key() {
            let delta = match key {
                Key::Named(NamedKey::ArrowLeft) => Some((32, 0)),
                Key::Named(NamedKey::ArrowRight) => Some((-32, 0)),
                Key::Named(NamedKey::ArrowUp) => Some((0, 32)),
                Key::Named(NamedKey::ArrowDown) => Some((0, -32)),
                _ => None,
            };
            if let Some((x, y)) = delta {
                state.activate(Action::ShaderPreviewPan(x, y));
                return;
            }
        }
    }
    if command
        && matches!(state.modal, Some(app::Modal::Shaders | app::Modal::Guides))
        && let Key::Character(c) = &key
    {
        match c.to_lowercase().as_str() {
            "z" => state.activate(if modifiers.shift_key() {
                Action::Redo
            } else {
                Action::Undo
            }),
            "y" => state.activate(Action::Redo),
            "s" => state.activate(Action::Save),
            "e" => state.activate(Action::Export),
            _ => {}
        }
        return;
    }
    if command && state.modal.is_some() {
        if let Key::Character(c) = &key {
            match c.to_lowercase().as_str() {
                "a" => state.input_replace = true,
                "v" => state.paste_field(),
                "c" => state.activate(Action::CopyExport),
                _ => {}
            }
        }
        return;
    }
    if command
        && !modifiers.alt_key()
        && !modifiers.shift_key()
        && state.edit_mode == EditMode::Keyboard
        && !state.ref_transform
    {
        let bank = match key {
            Key::Named(NamedKey::ArrowUp) => Some(-1),
            Key::Named(NamedKey::ArrowDown) => Some(1),
            _ => None,
        };
        if let Some(delta) = bank {
            state.activate(Action::CharsetBank(delta));
            return;
        }
    }
    if modifiers.alt_key() && state.modal.is_none() && !state.ref_transform {
        let delta = match key {
            Key::Named(NamedKey::ArrowLeft) => Some((32., 0.)),
            Key::Named(NamedKey::ArrowRight) => Some((-32., 0.)),
            Key::Named(NamedKey::ArrowUp) => Some((0., 32.)),
            Key::Named(NamedKey::ArrowDown) => Some((0., -32.)),
            _ => None,
        };
        if let Some((x, y)) = delta {
            state.fit = false;
            state.pan.0 += x;
            state.pan.1 += y;
            return;
        }
    }
    if command && let Key::Character(c) = &key {
        let a = match c.to_lowercase().as_str() {
            "," => Some(Action::Settings),
            "m" if modifiers.shift_key() => Some(Action::ToggleEditMode),
            "n" => Some(Action::New),
            "o" => Some(Action::Open),
            "s" => Some(if modifiers.shift_key() {
                Action::SaveAs
            } else {
                Action::Save
            }),
            "z" => Some(if modifiers.shift_key() {
                Action::Redo
            } else {
                Action::Undo
            }),
            "y" => Some(Action::Redo),
            "c" => Some(if modifiers.shift_key() {
                Action::CopyText
            } else {
                Action::Copy
            }),
            "x" => Some(Action::Cut),
            "v" => Some(Action::Paste),
            "a" => Some(Action::SelectAll),
            "e" => Some(Action::Export),
            "q" => Some(Action::Quit),
            _ => None,
        };
        if let Some(a) = a {
            state.activate(a);
            return;
        }
    }
    if modifiers.super_key()
        || modifiers.control_key()
        || (modifiers.alt_key() && state.edit_mode == EditMode::Keyboard && state.modal.is_none())
    {
        return;
    }
    match &key {
        Key::Named(NamedKey::Space)
            if state.edit_mode == EditMode::Mouse
                && state.tool == Tool::Text
                && state.modal.is_none() =>
        {
            state.text_input(" ")
        }
        Key::Named(NamedKey::Space) if state.edit_mode == EditMode::Keyboard => {
            state.keyboard_input(" ");
        }
        Key::Named(NamedKey::PageUp)
            if state.edit_mode == EditMode::Keyboard && state.modal.is_none() =>
        {
            state.activate(Action::CharsetBank(-1))
        }
        Key::Named(NamedKey::PageDown)
            if state.edit_mode == EditMode::Keyboard && state.modal.is_none() =>
        {
            state.activate(Action::CharsetBank(1))
        }
        Key::Named(NamedKey::Escape) => state.activate(Action::Cancel),
        Key::Named(NamedKey::Tab) => {
            if state.keyboard_active() {
                state.keyboard_tab(modifiers.shift_key());
            } else {
                state.tab(modifiers.shift_key());
            }
        }
        Key::Named(NamedKey::Enter) => {
            if modifiers.shift_key()
                && state.modal.is_none()
                && let Some(i) = state.focus
                && let Some(app::Hit {
                    action: Action::Palette(p),
                    ..
                }) = state.hits.get(i)
            {
                state.activate(Action::EditPalette(*p));
                return;
            }
            state.enter();
        }
        Key::Named(NamedKey::Backspace) => state.backspace(),
        Key::Named(NamedKey::Delete) => {
            if state.modal.is_some() {
                state.backspace();
            } else {
                state.activate(Action::EraseSelection);
            }
        }
        Key::Named(NamedKey::F1) => state.activate(Action::Help),
        Key::Named(NamedKey::F2) => state.activate(Action::Resize),
        Key::Named(NamedKey::F3) => state.focus_zone(0),
        Key::Named(NamedKey::F4) => state.focus_zone(1),
        Key::Named(NamedKey::F5) => state.focus_zone(2),
        Key::Named(NamedKey::F6) => {
            state.focus = None;
            state.keyboard_canvas = true;
        }
        Key::Named(NamedKey::F7) => state.activate(Action::Shaders),
        Key::Named(NamedKey::F8) => state.activate(Action::Guides),
        Key::Named(NamedKey::ArrowLeft) => state.move_cursor(-1, 0, modifiers.shift_key()),
        Key::Named(NamedKey::ArrowRight) => state.move_cursor(1, 0, modifiers.shift_key()),
        Key::Named(NamedKey::ArrowUp) => state.move_cursor(0, -1, modifiers.shift_key()),
        Key::Named(NamedKey::ArrowDown) => state.move_cursor(0, 1, modifiers.shift_key()),
        Key::Character(c) => {
            if matches!(state.modal, Some(app::Modal::Charsets))
                && let Ok(n) = c.parse::<usize>()
                && (1..=ditto::charset::ALL.len()).contains(&n)
            {
                state.activate(Action::SetCharset(n - 1));
                return;
            }
            if state.modal.is_none() && state.edit_mode == EditMode::Keyboard {
                state.keyboard_input(text.unwrap_or(c));
                return;
            }
            if state.modal.is_some() || state.tool == Tool::Text {
                if state.ime.is_empty() {
                    state.text_input(text.unwrap_or(c));
                }
                return;
            }
            let a = match c.to_lowercase().as_str() {
                "c" => Some(Action::Tool(Tool::Recolor)),
                "d" => Some(Action::Tool(Tool::Guide)),
                "[" if state.tool == Tool::Recolor => Some(Action::RecolorSize(-1)),
                "]" if state.tool == Tool::Recolor => Some(Action::RecolorSize(1)),
                "b" => Some(Action::Tool(Tool::Pencil)),
                "g" => Some(Action::Tool(Tool::Eraser)),
                "l" => Some(Action::Tool(Tool::Line)),
                "r" => Some(Action::Tool(Tool::Rectangle)),
                "f" => Some(Action::Tool(Tool::Fill)),
                "i" => Some(Action::Tool(Tool::Pick)),
                "t" => Some(Action::Tool(Tool::Text)),
                "m" => Some(Action::Tool(Tool::Select)),
                "+" | "=" => Some(Action::Zoom(true)),
                "-" => Some(Action::Zoom(false)),
                "0" => Some(Action::Fit),
                _ => None,
            };
            if let Some(a) = a {
                state.activate(a);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod keyboard_routing_tests {
    use super::*;
    fn state() -> State {
        let mut s = State::new(PathBuf::from("/tmp/ditto-key-routing-test"));
        s.modal = None;
        s.editor = ditto::core::Editor::new(ditto::core::Document::new(24, 12).unwrap());
        s.activate(Action::ToggleEditMode);
        s.layout(1184., 832.);
        s.frame();
        s
    }
    fn key(s: &mut State, source: &str, text: &str, m: ModifiersState) {
        route_key(s, m, Key::Character(source.into()), Some(text));
    }
    #[test]
    fn raw_key_routing_uses_printed_character_and_never_tool_or_zoom_shortcuts() {
        let mut s = state();
        let tool = s.tool;
        let zoom = s.cell_size;
        for k in ["b", "t", "0", "+", "-"] {
            key(&mut s, k, k, ModifiersState::empty());
        }
        assert_eq!(s.cursor, (3, 0));
        assert_eq!(s.tool, tool);
        assert_eq!(s.cell_size, zoom);
        key(&mut s, "l", "@", ModifiersState::ALT);
        assert_eq!(s.cursor, (3, 0));
    }
    #[test]
    fn command_shortcuts_and_unknown_modified_keys_do_not_draw() {
        let mut s = state();
        let command = if cfg!(target_os = "macos") {
            ModifiersState::SUPER
        } else {
            ModifiersState::CONTROL
        };
        key(&mut s, "a", "a", command);
        assert_eq!(s.editor.selection, Some(s.editor.document.bounds()));
        assert!(!s.editor.dirty());
        key(&mut s, "1", "1", command);
        assert!(!s.editor.dirty());
        key(&mut s, "m", "M", command | ModifiersState::SHIFT);
        assert_eq!(s.edit_mode, EditMode::Mouse);
        assert!(!s.editor.dirty());
    }
    #[test]
    fn macbook_arrows_switch_banks_without_editing_or_moving_the_caret() {
        let mut s = state();
        let command = if cfg!(target_os = "macos") {
            ModifiersState::SUPER
        } else {
            ModifiersState::CONTROL
        };
        for set in 0..ditto::charset::ALL.len() {
            s.activate(Action::SetCharset(set));
            let cursor = s.cursor;
            let doc = s.editor.document.clone();
            route_key(&mut s, command, Key::Named(NamedKey::ArrowUp), None);
            assert_eq!(s.charset_bank, s.active_charset().banks() - 1);
            route_key(&mut s, command, Key::Named(NamedKey::ArrowDown), None);
            assert_eq!(s.charset_bank, 0);
            route_key(&mut s, command, Key::Named(NamedKey::ArrowDown), None);
            assert_eq!(s.charset_bank, 1);
            assert_eq!(s.cursor, cursor);
            assert_eq!(s.editor.document, doc);
            assert!(!s.editor.dirty());
            route_key(
                &mut s,
                ModifiersState::empty(),
                Key::Named(NamedKey::PageUp),
                None,
            );
            assert_eq!(s.charset_bank, 0);
            route_key(
                &mut s,
                ModifiersState::empty(),
                Key::Named(NamedKey::ArrowDown),
                None,
            );
            assert_eq!(s.charset_bank, 0);
            assert_eq!(s.cursor, (cursor.0, cursor.1 + 1));
        }
        s.activate(Action::Resize);
        route_key(&mut s, command, Key::Named(NamedKey::ArrowDown), None);
        assert_eq!(s.charset_bank, 0);
        s.modal = None;
        s.ime = "^".into();
        route_key(&mut s, command, Key::Named(NamedKey::ArrowDown), None);
        assert_eq!(s.charset_bank, 0);
        s.ime.clear();
        s.activate(Action::ToggleEditMode);
        route_key(&mut s, command, Key::Named(NamedKey::ArrowDown), None);
        assert_eq!(s.charset_bank, 0);
    }
    #[test]
    fn trackpad_zoom_depends_on_travel_not_event_count_or_retina_scale() {
        let mut single = state();
        let mut split = state();
        let mut retina = state();
        let p = (
            single.origin.0 + single.cell_width() * 6.5,
            single.origin.1 + single.cell_size * 5.5,
        );
        let before = single.cell_size;
        for s in [&mut single, &mut split, &mut retina] {
            s.mouse = p;
        }
        let pixel = |y| MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0., y));
        route_scroll(&mut single, pixel(100.), 1.);
        for _ in 0..200 {
            route_scroll(&mut split, pixel(0.5), 1.);
        }
        route_scroll(&mut retina, pixel(200.), 2.);
        assert!((single.cell_size - split.cell_size).abs() < 0.002);
        assert!((single.cell_size - retina.cell_size).abs() < 0.002);
        assert!(single.cell_size > before && single.cell_size < before * 1.11);
        for s in [&single, &split, &retina] {
            assert!((p.0 - s.origin.0 - s.cell_width() * 6.5).abs() < 0.01);
            assert!((p.1 - s.origin.1 - s.cell_size * 5.5).abs() < 0.01);
            assert!(!s.editor.dirty());
        }
        route_scroll(&mut single, pixel(-100.), 1.);
        assert!((single.cell_size - before).abs() < 0.002);
    }
    #[test]
    fn wheel_zoom_is_fine_bounded_and_ignores_horizontal_or_modal_scroll() {
        let mut s = state();
        let before = s.cell_size;
        route_scroll(&mut s, MouseScrollDelta::LineDelta(3., 0.), 1.);
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., f32::NAN), 1.);
        assert_eq!(s.cell_size, before);
        assert!(s.fit);
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., 1.), 1.);
        assert!(s.cell_size > before * 1.04 && s.cell_size < before * 1.06);
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., -1.), 1.);
        assert!((s.cell_size - before).abs() < 0.002);
        s.activate(Action::Resize);
        let modal_size = s.cell_size;
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., 5.), 1.);
        assert_eq!(s.cell_size, modal_size);
        s.modal = None;
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., 1000.), 1.);
        assert_eq!(s.cell_size, 128.);
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., -1000.), 1.);
        assert_eq!(s.cell_size, 1.);
        assert!(!s.editor.dirty());
        s.editor = ditto::core::Editor::new(ditto::core::Document::new(1, 1).unwrap());
        s.fit = true;
        s.layout(s.width, s.height);
        let fitted = s.cell_size;
        assert!(fitted > 128.);
        route_scroll(&mut s, MouseScrollDelta::LineDelta(5., 0.), 1.);
        assert_eq!(s.cell_size, fitted);
        assert!(s.fit);
    }
    #[test]
    fn option_and_altgr_never_insert_mapped_characters() {
        let mut s = state();
        key(
            &mut s,
            "q",
            "@",
            ModifiersState::CONTROL | ModifiersState::ALT,
        );
        assert_eq!(s.cursor, (0, 0));
        key(
            &mut s,
            "a",
            "a",
            ModifiersState::CONTROL | ModifiersState::ALT,
        );
        assert_eq!(s.cursor, (0, 0));
    }
    #[test]
    fn bare_azerty_number_row_maps_digits_but_fields_and_modifiers_do_not() {
        let mut s = state();
        for (i, (code, printed, digit)) in [
            (KeyCode::Digit1, "&", '1'),
            (KeyCode::Digit2, "é", '2'),
            (KeyCode::Digit3, "\"", '3'),
            (KeyCode::Digit4, "'", '4'),
            (KeyCode::Digit5, "(", '5'),
            (KeyCode::Digit6, "-", '6'),
            (KeyCode::Digit7, "è", '7'),
            (KeyCode::Digit8, "_", '8'),
            (KeyCode::Digit9, "ç", '9'),
            (KeyCode::Digit0, "à", '0'),
        ]
        .into_iter()
        .enumerate()
        {
            route_keyboard_event(
                &mut s,
                ModifiersState::empty(),
                PhysicalKey::Code(code),
                Key::Character(printed.into()),
                Some(printed),
            );
            assert_eq!(
                s.editor.document.get(i as i32, 0).unwrap().glyph,
                s.active_charset().resolve(0, digit).unwrap()
            );
        }
        assert_eq!(s.cursor, (10, 0));
        for modifier in [
            ModifiersState::ALT,
            ModifiersState::SUPER,
            ModifiersState::CONTROL,
        ] {
            route_keyboard_event(
                &mut s,
                modifier,
                PhysicalKey::Code(KeyCode::Digit1),
                Key::Character("&".into()),
                Some("&"),
            );
            assert_eq!(s.cursor, (10, 0));
        }
        s.ime = "^".into();
        route_keyboard_event(
            &mut s,
            ModifiersState::empty(),
            PhysicalKey::Code(KeyCode::Digit1),
            Key::Character("&".into()),
            Some("&"),
        );
        assert_eq!(s.cursor, (10, 0));
        s.ime.clear();
        s.activate(Action::Resize);
        route_keyboard_event(
            &mut s,
            ModifiersState::empty(),
            PhysicalKey::Code(KeyCode::Digit1),
            Key::Character("&".into()),
            Some("&"),
        );
        assert!(matches!(s.modal,Some(app::Modal::Resize{ref width,..})if width != "1"));
        route_keyboard_event(
            &mut s,
            ModifiersState::SHIFT,
            PhysicalKey::Code(KeyCode::Digit1),
            Key::Character("1".into()),
            Some("1"),
        );
        assert!(matches!(s.modal,Some(app::Modal::Resize{ref width,..})if width == "1"));
    }
    #[test]
    fn marked_text_and_numeric_fields_are_not_remapped() {
        let mut s = state();
        s.ime = "^".into();
        key(&mut s, "a", "a", ModifiersState::empty());
        assert!(!s.editor.dirty());
        s.ime_commit("ê");
        assert_eq!(
            s.editor.document.get(0, 0).unwrap().glyph,
            ditto::font::glyph('ê').unwrap()
        );
        s.activate(Action::Resize);
        key(&mut s, "1", "1", ModifiersState::empty());
        assert!(matches!(s.modal,Some(app::Modal::Resize{ref width,..})if width=="1"));
    }
    #[test]
    fn space_and_tab_follow_asciitor_caret_rules() {
        let mut s = state();
        key(&mut s, "a", "a", ModifiersState::empty());
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Named(NamedKey::Space),
            Some(" "),
        );
        assert_eq!(s.cursor, (2, 0));
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Named(NamedKey::Tab),
            None,
        );
        assert_eq!(s.cursor, (4, 0));
        route_key(
            &mut s,
            ModifiersState::SHIFT,
            Key::Named(NamedKey::Tab),
            None,
        );
        assert_eq!(s.cursor, (0, 0));
    }
    #[test]
    fn end_of_document_does_not_return_to_start_of_last_line() {
        let mut s = state();
        s.cursor = (24, 11);
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Named(NamedKey::Enter),
            None,
        );
        assert_eq!(s.cursor, (24, 11));
        assert!(s.status.contains("Fin du document"));
    }
    #[test]
    fn charset_number_selects_preset_without_inserting_a_glyph() {
        let mut s = state();
        s.activate(Action::Charsets);
        key(&mut s, "6", "6", ModifiersState::empty());
        assert_eq!(s.charset, 5);
        assert!(s.modal.is_none());
        assert!(!s.editor.dirty());
    }
    #[test]
    fn shader_shortcuts_work_in_keyboard_mode_and_do_not_insert_text() {
        let mut s = state();
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Named(NamedKey::F7),
            None,
        );
        assert!(matches!(s.modal, Some(app::Modal::Shaders)));
        s.activate(Action::ShaderPreset(0));
        let command = if cfg!(target_os = "macos") {
            ModifiersState::SUPER
        } else {
            ModifiersState::CONTROL
        };
        key(&mut s, "z", "z", command);
        assert!(s.editor.document.shaders.layers.is_empty());
        key(&mut s, "z", "z", command | ModifiersState::SHIFT);
        assert_eq!(s.editor.document.shaders.layers.len(), 2);
        key(&mut s, "b", "b", ModifiersState::empty());
        assert!(
            s.editor
                .document
                .cells
                .iter()
                .all(|c| *c == ditto::core::Cell::default())
        );
    }
    #[test]
    fn shader_preview_scroll_and_shortcuts_stay_inside_the_panel() {
        let mut s = state();
        let doc = s.editor.document.clone();
        let canvas = (s.cell_size, s.pan, s.cursor);
        s.activate(Action::Shaders);
        let initial = s.shader_preview_scale();
        s.mouse = (20., 20.);
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., 5.), 1.);
        assert_eq!(s.shader_preview_scale(), initial);
        let r = s.shader_preview_rect();
        s.mouse = (r.x + 30., r.y + 30.);
        route_scroll(&mut s, MouseScrollDelta::LineDelta(0., 1.), 1.);
        assert!(s.shader_preview_scale() > initial && s.shader_preview_scale() < initial * 1.06);
        key(&mut s, "+", "+", ModifiersState::empty());
        assert!(s.shader_preview_scale() > initial * 1.3);
        key(&mut s, "1", "1", ModifiersState::empty());
        assert_eq!(s.shader_preview_scale(), 1.);
        route_key(
            &mut s,
            ModifiersState::ALT,
            Key::Named(NamedKey::ArrowLeft),
            None,
        );
        assert_eq!(s.shader_preview_pan, (32., 0.));
        key(&mut s, "0", "0", ModifiersState::empty());
        assert_eq!(s.shader_preview_scale(), initial);
        assert_eq!(s.shader_preview_pan, (0., 0.));
        s.zoom_shader_preview(f32::NAN, s.mouse);
        assert_eq!(s.shader_preview_scale(), initial);
        s.zoom_shader_preview(10000., s.mouse);
        assert_eq!(s.shader_preview_scale(), 16.);
        assert_eq!((s.cell_size, s.pan, s.cursor), canvas);
        assert_eq!(s.editor.document, doc);
        assert!(!s.editor.dirty());
    }
}

#[cfg(test)]
mod workspace_shortcut_tests {
    use super::*;
    #[test]
    fn workspace_shortcuts_keep_text_mapping_and_modal_edits_separate() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-workspace-keys"));
        s.modal = None;
        let command = if cfg!(target_os = "macos") {
            ModifiersState::SUPER
        } else {
            ModifiersState::CONTROL
        };
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Character("c".into()),
            None,
        );
        assert_eq!(s.tool, Tool::Recolor);
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Character("]".into()),
            None,
        );
        assert_eq!(s.recolor_radius, 1);
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Character("d".into()),
            None,
        );
        assert_eq!(s.tool, Tool::Guide);
        s.activate(Action::ToggleEditMode);
        let original = s.editor.document.clone();
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Named(NamedKey::F8),
            None,
        );
        assert!(matches!(s.modal, Some(app::Modal::Guides)));
        s.activate(Action::GuideVisible);
        assert!(!s.editor.document.guides.visible);
        route_key(&mut s, command, Key::Character("z".into()), None);
        assert_eq!(s.editor.document, original);
        s.activate(Action::Cancel);
        route_key(&mut s, command, Key::Character(",".into()), None);
        assert!(matches!(s.modal, Some(app::Modal::Settings)));
        s.activate(Action::ThemePreset(1));
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Named(NamedKey::Escape),
            None,
        );
        assert_eq!(s.settings, ditto::settings::Settings::default());
        assert_eq!(s.editor.document, original);
        route_key(
            &mut s,
            ModifiersState::empty(),
            Key::Character("c".into()),
            None,
        );
        assert_eq!(s.cursor, (1, 0));
        assert_eq!(s.edit_mode, EditMode::Keyboard);
    }
}
