use crate::{
    color_picker::{self, Control as ColorControl, Picker},
    render::{Draw, Rect as ScreenRect},
};
use ditto::{
    charset,
    core::*,
    font, project,
    settings::{self, Settings, Theme, Token},
    shaders::{self, Kind, Layer, Stack},
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditMode {
    Mouse,
    Keyboard,
}
#[derive(Clone, Copy)]
struct KeyEdit {
    parent: u64,
    before: ((i32, i32), Option<Rect>),
    after: ((i32, i32), Option<Rect>),
}
#[derive(Clone, Copy)]
struct PreviewDrag {
    mouse: (f32, f32),
    pan: (f32, f32),
}
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    New,
    ToggleEditMode,
    Charsets,
    SetCharset(usize),
    CharsetBank(i32),
    MappingPage,
    MappedKey(usize),
    Open,
    Save,
    SaveAs,
    Export,
    CopyText,
    Undo,
    Redo,
    Tool(Tool),
    Glyph(font::Glyph),
    Category(u8),
    GlyphPage(i32),
    Palette(usize),
    EditPalette(usize),
    Foreground,
    Background,
    ClearBackground,
    Mask(u8),
    Filled,
    Grid,
    Fit,
    Zoom(bool),
    ImportReference,
    RefVisible,
    RefLock,
    RefOpacity(i32),
    RefFit(bool),
    RefTransform,
    RefScale(bool),
    RefNudge(i32, i32),
    RefRemove,
    Trace,
    Copy,
    Cut,
    Paste,
    Move,
    SelectAll,
    Deselect,
    EraseSelection,
    Resize,
    Help,
    Quit,
    Input(usize),
    Preset(u32, u32),
    Submit,
    Cancel,
    Discard,
    SaveContinue,
    ExportScale(u32),
    ExportBackground,
    Recover,
    ForgetRecovery,
    AddColor,
    ColorControl(ColorControl),
    CopyExport,
    Settings,
    ThemePreset(usize),
    ThemeColor(Token),
    Guides,
    GuideVisible,
    GuideAbove,
    GuideRecolorAll,
    GuideOpacity(i32),
    GuideWidth(i32),
    GuideColor,
    GuideClear,
    RecolorSize(i32),
    Shaders,
    ShaderAdd(Kind),
    ShaderSelect(usize),
    ShaderToggle(usize),
    ShaderMove(usize, i32),
    ShaderRemove(usize),
    ShaderAdjust(usize, i32),
    ShaderColor(usize),
    ShaderEnabled,
    ShaderBefore,
    ShaderPreviewZoom(bool),
    ShaderPreviewFit,
    ShaderPreviewActual,
    ShaderPreviewPan(i32, i32),
    ShaderPreset(usize),
    ShaderReset,
    ShaderSave,
    ShaderLoad,
}
#[derive(Clone, Debug)]
pub enum Modal {
    New { width: String, height: String },
    Resize { width: String, height: String },
    Color { value: String, target: ColorTarget },
    Loss { next: Action },
    Export { scale: u32, opaque: bool },
    Text { content: String },
    Help,
    Recovery,
    Charsets,
    Shaders,
    Guides,
    Settings,
}
#[derive(Clone, Copy, Debug)]
pub enum ColorTarget {
    Foreground,
    Background,
    Palette(usize),
    Shader(usize, usize),
    Guide,
    Theme(Token),
}
#[derive(Clone)]
pub struct Hit {
    pub rect: ScreenRect,
    pub action: Action,
    pub label: String,
}
#[derive(Clone)]
pub enum Gesture {
    Recolor {
        last: Option<(i32, i32)>,
    },
    Guide {
        last: Option<[f32; 2]>,
    },
    Stroke {
        last: (i32, i32),
    },
    Shape {
        origin: (i32, i32),
    },
    Select {
        origin: (i32, i32),
    },
    Move {
        rect: Rect,
        origin: (i32, i32),
    },
    Reference {
        mouse: (f32, f32),
        x: f32,
        y: f32,
    },
    Pan {
        mouse: (f32, f32),
        offset: (f32, f32),
    },
}
#[derive(Clone)]
pub struct Floating {
    pub block: Block,
    pub position: (i32, i32),
}
pub enum Job {
    Loaded(anyhow::Result<Document>, PathBuf, bool),
    Saved(anyhow::Result<()>, PathBuf, u64, bool),
    Reference(anyhow::Result<Arc<Asset>>),
    Exported(anyhow::Result<()>, PathBuf),
}
pub struct State {
    pub settings: Settings,
    pub settings_draft: Option<Settings>,
    pub settings_path: Option<PathBuf>,
    pub guide_color: Color,
    pub guide_width: f32,
    pub recolor_radius: i32,
    pub editor: Editor,
    pub brush: Brush,
    pub tool: Tool,
    pub edit_mode: EditMode,
    pub charset: usize,
    pub charset_bank: usize,
    pub mapping_page: usize,
    pub last_key: Option<usize>,
    key_history: BTreeMap<u64, KeyEdit>,
    pub filled: bool,
    pub grid: bool,
    pub trace: bool,
    pub ref_transform: bool,
    pub category: u8,
    pub glyph_page: usize,
    pub recent: Vec<font::Glyph>,
    pub cursor: (i32, i32),
    pub text_origin: i32,
    pub path: Option<PathBuf>,
    pub modal: Option<Modal>,
    pub color_picker: Picker,
    color_drag: Option<ColorControl>,
    pub input: usize,
    pub input_replace: bool,
    pub ime: String,
    pub status: String,
    pub gesture: Option<Gesture>,
    pub floating: Option<Floating>,
    pub hits: Vec<Hit>,
    pub focus: Option<usize>,
    pub mouse: (f32, f32),
    pub canvas: ScreenRect,
    pub origin: (f32, f32),
    pub cell_size: f32,
    pub pan: (f32, f32),
    pub fit: bool,
    pub keyboard_canvas: bool,
    pub width: f32,
    pub height: f32,
    pub busy: bool,
    pub loading: bool,
    owns_recovery: bool,
    pub quit: bool,
    pub space: bool,
    pub recovery: PathBuf,
    pub last_change: Instant,
    pub autosaved_revision: Option<u64>,
    pub internal_clip: Option<(Block, String)>,
    pub clipboard: Option<arboard::Clipboard>,
    jobs_tx: mpsc::Sender<Job>,
    jobs_rx: mpsc::Receiver<Job>,
    after_save: Option<Action>,
    pub last_frame: Instant,
    pub generation: u64,
    pub shader_selected: usize,
    pub shader_before: bool,
    pub shader_preview_zoom: Option<f32>,
    pub shader_preview_pan: (f32, f32),
    shader_preview_drag: Option<PreviewDrag>,
    pub shader_error: Option<String>,
}
impl State {
    pub fn new(recovery: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        let has_recovery = recovery.exists();
        Self {
            settings: Settings::default(),
            settings_draft: None,
            settings_path: None,
            guide_color: [102, 211, 207],
            guide_width: 0.4,
            recolor_radius: 0,
            editor: Editor::new(Document::new(80, 50).unwrap()),
            brush: Brush::default(),
            tool: Tool::Pencil,
            edit_mode: EditMode::Mouse,
            charset: 3,
            charset_bank: 0,
            mapping_page: 0,
            last_key: None,
            key_history: BTreeMap::new(),
            filled: false,
            grid: false,
            trace: false,
            ref_transform: false,
            category: 0,
            glyph_page: 0,
            recent: vec![177, 219, 196, 179, 32],
            cursor: (0, 0),
            text_origin: 0,
            path: None,
            modal: Some(if has_recovery {
                Modal::Recovery
            } else {
                Modal::New {
                    width: "80".into(),
                    height: "50".into(),
                }
            }),
            color_picker: Picker::default(),
            color_drag: None,
            input: 0,
            input_replace: true,
            ime: String::new(),
            status: "Choisir un format pour commencer.".into(),
            gesture: None,
            floating: None,
            hits: Vec::new(),
            focus: None,
            mouse: (0., 0.),
            canvas: ScreenRect::default(),
            origin: (0., 0.),
            cell_size: 16.,
            pan: (0., 0.),
            fit: true,
            keyboard_canvas: true,
            width: 1184.,
            height: 832.,
            busy: false,
            loading: false,
            owns_recovery: false,
            quit: false,
            space: false,
            recovery,
            last_change: Instant::now(),
            autosaved_revision: None,
            internal_clip: None,
            clipboard: None,
            jobs_tx: tx,
            jobs_rx: rx,
            after_save: None,
            last_frame: Instant::now(),
            generation: 0,
            shader_selected: 0,
            shader_before: false,
            shader_preview_zoom: None,
            shader_preview_pan: (0., 0.),
            shader_preview_drag: None,
            shader_error: None,
        }
    }
    pub fn load_settings(&mut self, path: PathBuf) {
        match settings::load(&path) {
            Ok(value) => self.settings = value,
            Err(e) => self.status = format!("Réglages illisibles, thème Ditto utilisé : {e}"),
        }
        self.settings_path = Some(path);
    }
    pub fn theme(&self) -> Theme {
        if matches!(
            self.modal,
            Some(
                Modal::Settings
                    | Modal::Color {
                        target: ColorTarget::Theme(_),
                        ..
                    }
            )
        ) {
            self.settings_draft.as_ref().unwrap_or(&self.settings).theme
        } else {
            self.settings.theme
        }
    }
    pub fn guide_point(&self, p: (f32, f32)) -> Option<[f32; 2]> {
        self.cell_at(p)?;
        Some([
            (p.0 - self.origin.0) / self.cell_width(),
            (p.1 - self.origin.1) / self.cell_size,
        ])
    }
    fn paint_cells(&mut self, points: &[(i32, i32)]) {
        if self.tool == Tool::Recolor {
            self.editor
                .recolor(points, self.brush.cell.fg, self.recolor_radius);
        } else {
            self.editor
                .paint(points, self.brush, self.tool == Tool::Eraser);
        }
    }
    fn changed(&mut self) {
        self.last_change = Instant::now();
    }
    pub fn title(&self) -> String {
        format!(
            "{}{} — Ditto",
            self.path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Sans titre".into()),
            if self.editor.dirty() { " *" } else { "" }
        )
    }
    pub fn cell_width(&self) -> f32 {
        self.cell_size * ditto::typeface::ASPECT
    }
    pub fn cell_at(&self, p: (f32, f32)) -> Option<(i32, i32)> {
        if !self.canvas.contains(p) {
            return None;
        }
        let x = ((p.0 - self.origin.0) / self.cell_width()).floor() as i32;
        let y = ((p.1 - self.origin.1) / self.cell_size).floor() as i32;
        self.editor.document.index(x, y).map(|_| (x, y))
    }
    pub fn clamped_cell(&self, p: (f32, f32)) -> (i32, i32) {
        (
            (((p.0 - self.origin.0) / self.cell_width()).floor() as i32)
                .clamp(0, self.editor.document.width as i32 - 1),
            (((p.1 - self.origin.1) / self.cell_size).floor() as i32)
                .clamp(0, self.editor.document.height as i32 - 1),
        )
    }
    pub fn layout(&mut self, w: f32, h: f32) {
        let compact_changed = (self.height < 800.) != (h < 800.);
        self.width = w;
        self.height = h;
        if compact_changed {
            let all = self.category_glyphs();
            self.glyph_page = all
                .iter()
                .position(|g| *g == self.brush.cell.glyph)
                .unwrap_or(0)
                / self.glyph_page_size();
            self.mapping_page = 0;
        }
        self.canvas = ScreenRect::new(304., 104., (w - 320.).max(16.), (h - 296.).max(16.));
        if self.fit {
            self.cell_size = ((self.canvas.w - 32.)
                / (self.editor.document.width as f32 * ditto::typeface::ASPECT))
                .min((self.canvas.h - 32.) / self.editor.document.height as f32)
                .floor()
                .max(1.);
            self.pan = (0., 0.);
        }
        self.origin = (
            self.canvas.x
                + (self.canvas.w - self.editor.document.width as f32 * self.cell_width()) / 2.
                + self.pan.0,
            self.canvas.y
                + (self.canvas.h - self.editor.document.height as f32 * self.cell_size) / 2.
                + self.pan.1,
        );
        if self.fit {
            self.origin = (self.origin.0.round(), self.origin.1.round());
        }
    }
    pub fn frame(&mut self) -> Draw {
        self.layout(self.width, self.height);
        let (draw, hits) = crate::ui::build(self);
        self.hits = hits;
        if self.focus.is_some_and(|i| i >= self.hits.len()) {
            self.focus = None;
        }
        self.last_frame = Instant::now();
        draw
    }
    pub fn choose_glyph(&mut self, g: font::Glyph) {
        let all = self.category_glyphs();
        self.glyph_page = all.iter().position(|v| *v == g).unwrap_or(0) / self.glyph_page_size();
        self.brush.cell.glyph = g;
        self.recent.retain(|v| *v != g);
        self.recent.insert(0, g);
        self.recent.truncate(16);
    }
    fn reset(&mut self, doc: Document, path: Option<PathBuf>, recovered: bool) {
        self.editor = Editor::new(doc);
        self.key_history.clear();
        if recovered {
            self.owns_recovery = true;
            self.editor.saved_revision = u64::MAX;
        }
        self.path = path;
        self.modal = None;
        self.gesture = None;
        self.floating = None;
        self.cursor = (0, 0);
        self.fit = true;
        self.focus = None;
        self.keyboard_canvas = true;
        self.ref_transform = false;
        self.generation += 1;
        self.shader_selected = 0;
        self.shader_before = false;
        self.shader_preview_zoom = None;
        self.shader_preview_pan = (0., 0.);
        self.shader_preview_drag = None;
        self.autosaved_revision = None;
        self.changed();
    }
    fn dismiss_gesture(&mut self) {
        self.editor.cancel();
        self.gesture = None;
        self.floating = None;
        self.ime.clear();
    }
    pub fn activate(&mut self, action: Action) {
        if self.loading {
            return;
        }
        if self.busy
            && matches!(
                action,
                Action::New
                    | Action::Open
                    | Action::Save
                    | Action::SaveAs
                    | Action::ImportReference
                    | Action::Submit
                    | Action::Quit
            )
        {
            self.status = "Opération en cours…".into();
            return;
        }
        if matches!(action, Action::New | Action::Open | Action::Quit) && self.editor.dirty() {
            self.dismiss_gesture();
            self.modal = Some(Modal::Loss { next: action });
            self.focus = None;
            return;
        }
        self.perform(action);
        if self.modal.is_none() && self.focus.is_none() {
            self.keyboard_canvas = true;
        }
    }
    fn perform(&mut self, action: Action) {
        self.shader_preview_drag = None;
        self.color_drag = None;
        match action {
            Action::ColorControl(control) => {
                self.focus = self
                    .hits
                    .iter()
                    .position(|h| h.action == Action::ColorControl(control));
                self.keyboard_canvas = false;
            }
            Action::Settings => {
                self.dismiss_gesture();
                self.settings_draft = Some(self.settings.clone());
                self.modal = Some(Modal::Settings);
                self.focus = None;
            }
            Action::ThemePreset(i) => {
                if let Some(draft) = &mut self.settings_draft {
                    draft.theme = Theme::preset(i);
                }
            }
            Action::ThemeColor(token) => self.color_modal(ColorTarget::Theme(token)),
            Action::Guides => {
                self.dismiss_gesture();
                self.modal = Some(Modal::Guides);
                self.focus = None;
            }
            Action::GuideAbove => {
                self.editor
                    .edit(|d| d.guides.above_characters = !d.guides.above_characters);
                self.changed();
            }
            Action::GuideRecolorAll => {
                let color = self.guide_color;
                self.editor.edit(|d| d.guides.recolor_all(color));
                self.changed();
                self.status =
                    "Couleur appliquée à tous les traits du guide · Cmd/Ctrl Z pour annuler."
                        .into();
            }
            Action::GuideVisible => {
                self.editor.edit(|d| d.guides.visible = !d.guides.visible);
                self.changed();
            }
            Action::GuideOpacity(delta) => {
                self.editor.edit(|d| {
                    d.guides.opacity = (d.guides.opacity + delta as f32 * 0.05).clamp(0.05, 1.)
                });
                self.changed();
            }
            Action::GuideWidth(delta) => {
                self.guide_width = ((self.guide_width + delta as f32 * 0.1) * 10.)
                    .round()
                    .clamp(1., 80.)
                    / 10.
            }
            Action::GuideColor => self.color_modal(ColorTarget::Guide),
            Action::GuideClear => {
                self.editor
                    .edit(|d| d.guides.strokes = Arc::new(Vec::new()));
                self.changed();
            }
            Action::RecolorSize(delta) => {
                self.recolor_radius = (self.recolor_radius + delta).clamp(0, 8)
            }
            Action::Shaders => {
                self.dismiss_gesture();
                self.modal = Some(Modal::Shaders);
                self.shader_before = false;
                self.focus = None;
                self.status =
                    "Shaders : aperçu en direct · changements annulables · inclus dans le PNG."
                        .into();
            }
            Action::ShaderAdd(kind) => {
                if self.editor.document.shaders.layers.len() >= shaders::MAX_LAYERS {
                    self.status =
                        "Huit shaders maximum. Retirer un effet pour en ajouter un autre.".into();
                } else {
                    self.shader_edit(|s| s.layers.push(Layer::new(kind)));
                    self.shader_selected = self.editor.document.shaders.layers.len() - 1;
                }
            }
            Action::ShaderSelect(i) => {
                self.shader_selected =
                    i.min(self.editor.document.shaders.layers.len().saturating_sub(1));
            }
            Action::ShaderToggle(i) => self.shader_edit(|s| {
                if let Some(l) = s.layers.get_mut(i) {
                    l.enabled = !l.enabled;
                }
            }),
            Action::ShaderMove(i, direction) => {
                let dest = i as i32 + direction;
                if i < self.editor.document.shaders.layers.len()
                    && dest >= 0
                    && (dest as usize) < self.editor.document.shaders.layers.len()
                {
                    self.shader_edit(|s| s.layers.swap(i, dest as usize));
                    self.shader_selected = dest as usize;
                    self.focus = None;
                }
            }
            Action::ShaderRemove(i) => {
                self.shader_edit(|s| {
                    if i < s.layers.len() {
                        s.layers.remove(i);
                    }
                });
                self.shader_selected =
                    i.min(self.editor.document.shaders.layers.len().saturating_sub(1));
                self.focus = None;
            }
            Action::ShaderAdjust(parameter, direction) => {
                let i = self
                    .shader_selected
                    .min(self.editor.document.shaders.layers.len().saturating_sub(1));
                self.shader_edit(|s| {
                    if let Some(l) = s.layers.get_mut(i) {
                        l.adjust(parameter, direction);
                    }
                });
            }
            Action::ShaderColor(color) => {
                let i = self
                    .shader_selected
                    .min(self.editor.document.shaders.layers.len().saturating_sub(1));
                if color < 2 && self.editor.document.shaders.layers.get(i).is_some() {
                    self.color_modal(ColorTarget::Shader(i, color));
                }
            }
            Action::ShaderEnabled => self.shader_edit(|s| s.enabled = !s.enabled),
            Action::ShaderBefore => self.shader_before = !self.shader_before,
            Action::ShaderPreviewZoom(up) => {
                let p = self.shader_preview_rect();
                self.zoom_shader_preview(
                    if up { 1.25 } else { 0.8 },
                    (p.x + p.w / 2., p.y + p.h / 2.),
                );
            }
            Action::ShaderPreviewFit => {
                self.shader_preview_zoom = None;
                self.shader_preview_pan = (0., 0.);
            }
            Action::ShaderPreviewActual => {
                self.shader_preview_zoom = Some(1.);
                self.shader_preview_pan = (0., 0.);
            }
            Action::ShaderPreviewPan(x, y) => {
                self.shader_preview_pan.0 += x as f32;
                self.shader_preview_pan.1 += y as f32;
            }
            Action::ShaderPreset(i) => {
                if i < shaders::PRESETS.len() {
                    self.shader_edit(|s| *s = shaders::preset(i));
                    self.shader_selected = 0;
                    self.focus = None;
                    self.status = format!(
                        "Préréglage {} appliqué. Annuler pour retrouver la pile précédente.",
                        shaders::PRESETS[i]
                    );
                }
            }
            Action::ShaderReset => {
                self.shader_edit(|s| *s = Stack::default());
                self.shader_selected = 0;
                self.focus = None;
            }
            Action::ShaderSave => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Shaders Ditto", &["ditto-shaders"])
                    .set_file_name("effets.ditto-shaders")
                    .save_file()
                {
                    self.status = match shaders::save_preset(
                        &with_extension(path, "ditto-shaders"),
                        &self.editor.document.shaders,
                    ) {
                        Ok(()) => "Préréglage shaders enregistré.".into(),
                        Err(e) => format!("Préréglage impossible : {e:#}"),
                    };
                }
            }
            Action::ShaderLoad => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Shaders Ditto", &["ditto-shaders"])
                    .pick_file()
                {
                    match shaders::load_preset(&path) {
                        Ok(stack) => {
                            self.shader_edit(|s| *s = stack);
                            self.shader_selected = 0;
                            self.focus = None;
                            self.status = "Préréglage shaders chargé.".into();
                        }
                        Err(e) => self.status = format!("Préréglage impossible : {e:#}"),
                    }
                }
            }
            Action::ToggleEditMode => {
                self.dismiss_gesture();
                self.edit_mode = if self.edit_mode == EditMode::Mouse {
                    EditMode::Keyboard
                } else {
                    EditMode::Mouse
                };
                self.focus = None;
                self.keyboard_canvas = true;
                self.ref_transform = false;
                self.space = false;
                self.clamp_cursor();
                if self.edit_mode == EditMode::Keyboard {
                    self.layout(self.width, self.height);
                    self.keep_cursor_visible();
                }
                self.status=if self.edit_mode==EditMode::Keyboard{"Mode clavier : une touche insère son glyphe et avance. Lettres et rangée des chiffres, sans combinaison."}else{"Mode souris : outils de peinture et répertoire de glyphes."}.into();
            }
            Action::Charsets => {
                self.dismiss_gesture();
                self.modal = Some(Modal::Charsets);
                self.focus = None;
            }
            Action::SetCharset(i) => {
                if i < charset::ALL.len() {
                    self.charset = i;
                    self.charset_bank = 0;
                    self.mapping_page = 0;
                    self.last_key = None;
                    self.modal = None;
                    self.focus = None;
                    self.keyboard_canvas = true;
                    self.status = format!(
                        "Charset : {}. Le dessin existant reste intact.",
                        charset::ALL[i].name
                    );
                }
            }
            Action::CharsetBank(delta) => {
                let banks = self.active_charset().banks();
                self.charset_bank =
                    (self.charset_bank as i32 + delta).rem_euclid(banks as i32) as usize;
                self.last_key = None;
                self.focus = None;
                self.keyboard_canvas = true;
                self.status = format!(
                    "{} · {}",
                    self.active_charset().name,
                    self.active_charset().bank_label(self.charset_bank)
                );
            }
            Action::MappingPage => {
                self.mapping_page = (self.mapping_page + 1)
                    % charset::KEYS.len().div_ceil(self.mapping_page_size());
                self.focus = None;
            }
            Action::MappedKey(slot) => self.insert_mapping(slot),
            Action::New => {
                self.dismiss_gesture();
                self.modal = Some(Modal::New {
                    width: "80".into(),
                    height: "50".into(),
                });
                self.input = 0;
                self.input_replace = true;
                self.focus = None;
            }
            Action::Open => {
                self.dismiss_gesture();
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("Projet Ditto", &["ditto"])
                    .pick_file()
                {
                    self.start_load(p, false);
                }
            }
            Action::Quit => {
                if self.owns_recovery && !self.editor.dirty() {
                    let _ = std::fs::remove_file(&self.recovery);
                    self.owns_recovery = false;
                }
                self.quit = true;
            }
            Action::Save => self.save(false),
            Action::SaveAs => self.save(true),
            Action::Undo => {
                self.dismiss_gesture();
                let from = self.editor.revision;
                if self.editor.undo()
                    && let Some(k) = self
                        .key_history
                        .get(&from)
                        .filter(|k| k.parent == self.editor.revision)
                {
                    (self.cursor, self.editor.selection) = k.before;
                }
                self.changed();
                self.clamp_cursor();
            }
            Action::Redo => {
                self.dismiss_gesture();
                let from = self.editor.revision;
                if self.editor.redo()
                    && let Some(k) = self
                        .key_history
                        .get(&self.editor.revision)
                        .filter(|k| k.parent == from)
                {
                    (self.cursor, self.editor.selection) = k.after;
                }
                self.changed();
                self.clamp_cursor();
            }
            Action::Tool(t) => {
                self.edit_mode = EditMode::Mouse;
                self.dismiss_gesture();
                self.tool = t;
                if matches!(t, Tool::Guide | Tool::GuideErase)
                    && matches!(self.modal, Some(Modal::Guides))
                {
                    self.modal = None;
                }
                self.text_origin = self.cursor.0;
                self.ref_transform = false;
                self.status = format!(
                    "{} — {}",
                    t.name(),
                    if t == Tool::Recolor {
                        "glisser pour repeindre les glyphes avec FG ; [ / ] : taille"
                    } else if matches!(t, Tool::Guide | Tool::GuideErase) {
                        "calque de placement ; F8 : couleur, épaisseur, visibilité"
                    } else if t == Tool::Text {
                        "saisir ; Échap termine"
                    } else {
                        "Entrée applique ; flèches déplacent"
                    }
                );
                self.focus = None;
                self.keyboard_canvas = true;
            }
            Action::Glyph(g) => {
                self.choose_glyph(g);
                self.keyboard_canvas = true;
                self.focus = None;
            }
            Action::GlyphPage(delta) => {
                let pages = self
                    .category_glyphs()
                    .len()
                    .div_ceil(self.glyph_page_size())
                    .max(1);
                self.glyph_page =
                    (self.glyph_page as i32 + delta).rem_euclid(pages as i32) as usize;
                self.focus = None;
            }
            Action::Category(c) => {
                self.category = c;
                self.glyph_page = 0;
                self.focus = None;
            }
            Action::Palette(i) => {
                if let Some(c) = self.editor.document.palette.get(i) {
                    self.brush.cell.fg = *c;
                }
            }
            Action::Foreground => self.color_modal(ColorTarget::Foreground),
            Action::Background => self.color_modal(ColorTarget::Background),
            Action::EditPalette(i) => self.color_modal(ColorTarget::Palette(i)),
            Action::ClearBackground => {
                self.brush.cell.bg = None;
                self.brush.bg = true;
            }
            Action::Mask(i) => match i {
                0 => self.brush.glyph = !self.brush.glyph,
                1 => self.brush.fg = !self.brush.fg,
                _ => self.brush.bg = !self.brush.bg,
            },
            Action::Filled => self.filled = !self.filled,
            Action::Grid => self.grid = !self.grid,
            Action::Trace => self.trace = !self.trace,
            Action::Fit => {
                self.fit = true;
                self.layout(self.width, self.height);
            }
            Action::Zoom(up) => self.zoom(up, self.mouse),
            Action::CopyText => {
                self.modal = Some(Modal::Text {
                    content: self.editor.document.text(self.editor.selection),
                });
                self.focus = None;
            }
            Action::CopyExport => {
                let text = if let Some(Modal::Text { content }) = &self.modal {
                    content.clone()
                } else {
                    return;
                };
                match self.clip_set(&text) {
                    Ok(()) => {
                        self.status = "Texte copié dans le presse-papiers.".into();
                        self.modal = None;
                    }
                    Err(e) => self.status = e,
                }
            }
            Action::Export => {
                self.dismiss_gesture();
                self.modal = Some(Modal::Export {
                    scale: 1,
                    opaque: false,
                });
                self.focus = None;
            }
            Action::ExportScale(s) => {
                if let Some(Modal::Export { scale, .. }) = &mut self.modal {
                    *scale = s;
                }
            }
            Action::ExportBackground => {
                if let Some(Modal::Export { opaque, .. }) = &mut self.modal {
                    *opaque = !*opaque;
                }
            }
            Action::ImportReference => {
                self.dismiss_gesture();
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("Image de référence", &["png", "jpg", "jpeg"])
                    .pick_file()
                {
                    let tx = self.jobs_tx.clone();
                    self.busy = true;
                    self.status = "Lecture de la référence…".into();
                    std::thread::spawn(move || {
                        let _ = tx.send(Job::Reference(project::import_reference(&p)));
                    });
                }
            }
            Action::RefVisible => self.reference_edit(|r| r.visible = !r.visible),
            Action::RefLock => self.reference_edit(|r| r.locked = !r.locked),
            Action::RefOpacity(d) => {
                self.reference_edit(|r| r.opacity = (r.opacity + d as f32 / 100.).clamp(0., 1.))
            }
            Action::RefFit(cover) => {
                let (w, h) = (self.editor.document.width, self.editor.document.height);
                self.reference_edit(|r| {
                    let old_opacity = r.opacity;
                    let old_lock = r.locked;
                    *r = Reference::fit(r.asset.clone(), w, h, cover);
                    r.opacity = old_opacity;
                    r.locked = old_lock;
                });
            }
            Action::RefTransform => {
                if self
                    .editor
                    .document
                    .reference
                    .as_ref()
                    .is_some_and(|r| !r.locked)
                {
                    self.ref_transform = !self.ref_transform;
                    self.focus = None;
                    self.keyboard_canvas = true;
                } else {
                    self.status = "Déverrouiller la référence avant de la déplacer.".into();
                }
            }
            Action::RefScale(up) => self.reference_edit(|r| {
                if !r.locked {
                    let f = if up { 1.1 } else { 1. / 1.1 };
                    let next = (r.scale * f).clamp(0.00001, 1e4);
                    r.x += (r.asset.width as f32 * (r.scale - next)) / 2.;
                    r.y += (r.asset.height as f32 * (r.scale - next) * r.pixel_aspect) / 2.;
                    r.scale = next;
                }
            }),
            Action::RefNudge(x, y) => self.reference_edit(|r| {
                if !r.locked {
                    r.x = (r.x + x as f32).clamp(-1e6, 1e6);
                    r.y = (r.y + y as f32).clamp(-1e6, 1e6);
                }
            }),
            Action::RefRemove => {
                self.editor.edit(|d| d.reference = None);
                self.ref_transform = false;
                self.changed();
            }
            Action::Copy => self.copy(false),
            Action::Cut => self.copy(true),
            Action::Paste => self.paste(),
            Action::Move => {
                if let Some(r) = self.editor.selection {
                    self.gesture = Some(Gesture::Move {
                        rect: r,
                        origin: (r.x, r.y),
                    });
                    self.cursor = (r.x, r.y);
                    self.focus = None;
                    self.keyboard_canvas = true;
                    self.status = "Déplacer avec les flèches puis Entrée ; Échap annule.".into();
                }
            }
            Action::SelectAll => {
                self.editor.selection = Some(self.editor.document.bounds());
            }
            Action::Deselect => self.editor.selection = None,
            Action::EraseSelection => {
                if self.edit_mode == EditMode::Keyboard {
                    self.keyboard_erase(false);
                    return;
                }
                if self.editor.selection.is_some() {
                    self.editor.clear_selection();
                } else {
                    self.editor.begin();
                    self.editor.paint(&[self.cursor], self.brush, true);
                    self.editor.commit();
                }
                self.changed();
            }
            Action::Resize => {
                self.dismiss_gesture();
                self.modal = Some(Modal::Resize {
                    width: self.editor.document.width.to_string(),
                    height: self.editor.document.height.to_string(),
                });
                self.input = 0;
                self.input_replace = true;
                self.focus = None;
            }
            Action::Help => {
                self.dismiss_gesture();
                self.modal = Some(Modal::Help);
                self.focus = None;
            }
            Action::Input(i) => {
                self.input = i;
                self.input_replace = true;
                self.focus = None;
            }
            Action::Preset(w, h) => {
                if let Some(Modal::New { width, height }) = &mut self.modal {
                    *width = w.to_string();
                    *height = h.to_string();
                }
            }
            Action::Submit => self.submit(),
            Action::Cancel => {
                match &self.modal {
                    Some(Modal::Color {
                        target: ColorTarget::Theme(_),
                        ..
                    }) => {
                        self.modal = Some(Modal::Settings);
                        self.focus = None;
                        return;
                    }
                    Some(Modal::Color {
                        target: ColorTarget::Guide,
                        ..
                    }) => {
                        self.modal = Some(Modal::Guides);
                        self.focus = None;
                        return;
                    }
                    Some(Modal::Settings) => {
                        self.settings_draft = None;
                    }
                    _ => {}
                }

                if matches!(
                    self.modal,
                    Some(Modal::Color {
                        target: ColorTarget::Shader(..),
                        ..
                    })
                ) {
                    self.modal = Some(Modal::Shaders);
                    self.focus = None;
                    return;
                }
                if matches!(self.modal, Some(Modal::Recovery)) {
                    self.quit = true;
                    return;
                }
                if self.modal.is_some() {
                    self.modal = None;
                    self.focus = None;
                    self.keyboard_canvas = true;
                } else {
                    self.dismiss_gesture();
                    if self.tool == Tool::Text {
                        self.tool = Tool::Pencil;
                    } else {
                        self.editor.selection = None;
                    }
                    self.ref_transform = false;
                }
            }
            Action::Discard => {
                if let Some(Modal::Loss { next }) = self.modal.take() {
                    let _ = std::fs::remove_file(&self.recovery);
                    self.perform(next);
                }
            }
            Action::SaveContinue => {
                if let Some(Modal::Loss { next }) = self.modal.take() {
                    self.after_save = Some(next);
                    self.save(false);
                }
            }
            Action::Recover => self.start_load(self.recovery.clone(), true),
            Action::ForgetRecovery => {
                let _ = std::fs::remove_file(&self.recovery);
                self.perform(Action::New);
            }
            Action::AddColor => {
                if self.editor.document.palette.len() < 32 {
                    let c = self.brush.cell.fg;
                    self.editor.edit(|d| d.palette.push(c));
                    self.changed();
                } else {
                    self.status = "Palette : 32 couleurs maximum dans l’éditeur.".into();
                }
            }
        }
    }
    fn reference_edit(&mut self, f: impl FnOnce(&mut Reference)) {
        self.editor.begin();
        if let Some(r) = &mut self.editor.document.reference {
            f(r);
        }
        self.editor.commit();
        self.changed();
    }
    fn shader_edit(&mut self, f: impl FnOnce(&mut Stack)) {
        self.editor.edit(|d| f(&mut d.shaders));
        self.shader_before = false;
        self.changed();
    }
    fn color_modal(&mut self, target: ColorTarget) {
        self.dismiss_gesture();
        let c = match target {
            ColorTarget::Theme(token) => token.color(&self.theme()),
            ColorTarget::Guide => self.guide_color,
            ColorTarget::Foreground => self.brush.cell.fg,
            ColorTarget::Background => self.brush.cell.bg.unwrap_or([16, 16, 20]),
            ColorTarget::Palette(i) => self
                .editor
                .document
                .palette
                .get(i)
                .copied()
                .unwrap_or(self.brush.cell.fg),
            ColorTarget::Shader(layer, color) => {
                self.editor.document.shaders.layers[layer].colors[color]
            }
        };
        self.color_picker = Picker::new(c);
        self.color_drag = None;
        self.modal = Some(Modal::Color {
            value: format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2]),
            target,
        });
        self.input_replace = true;
        self.focus = None;
    }
    fn submit(&mut self) {
        let Some(m) = self.modal.clone() else {
            return;
        };
        match m {
            Modal::Settings => {
                let Some(draft) = self.settings_draft.as_ref() else {
                    return;
                };
                if let Some(path) = &self.settings_path
                    && let Err(e) = settings::save(path, draft)
                {
                    self.status = format!("Enregistrement des réglages impossible : {e}");
                    return;
                }
                self.settings = self.settings_draft.take().unwrap();
                self.modal = None;
                self.focus = None;
                self.status = "Réglages de l’application enregistrés.".into();
            }
            Modal::New { width, height } | Modal::Resize { width, height } => {
                let parsed = width.parse::<u32>().ok().zip(height.parse::<u32>().ok());
                let Some((w, h)) = parsed else {
                    self.status = "Dimensions entières requises.".into();
                    return;
                };
                match Document::new(w, h) {
                    Ok(d) => {
                        if matches!(self.modal, Some(Modal::Resize { .. })) {
                            self.editor.begin();
                            if let Err(e) = self.editor.document.resize(w, h) {
                                self.editor.cancel();
                                self.status = e;
                                return;
                            }
                            self.editor.commit();
                            self.editor.selection = None;
                            self.clamp_cursor();
                            self.modal = None;
                            self.fit = true;
                            self.changed();
                        } else {
                            self.reset(d, None, false);
                        }
                        self.status = format!("Grille {w} × {h} prête.");
                    }
                    Err(e) => self.status = e,
                }
            }
            Modal::Color { value, target } => match project::color_hex(&value) {
                Ok(c) => {
                    match target {
                        ColorTarget::Theme(token) => {
                            if let Some(draft) = &mut self.settings_draft {
                                *token.slot(&mut draft.theme) = c;
                            }
                        }
                        ColorTarget::Guide => self.guide_color = c,
                        ColorTarget::Foreground => self.brush.cell.fg = c,
                        ColorTarget::Background => {
                            self.brush.cell.bg = Some(c);
                            self.brush.bg = true;
                        }
                        ColorTarget::Palette(i) => {
                            self.editor.edit(|d| {
                                if let Some(slot) = d.palette.get_mut(i) {
                                    *slot = c;
                                }
                            });
                            self.changed();
                        }
                        ColorTarget::Shader(layer, color) => self.shader_edit(|s| {
                            if let Some(l) = s.layers.get_mut(layer) {
                                l.colors[color] = c;
                            }
                        }),
                    }
                    self.modal = match target {
                        ColorTarget::Shader(..) => Some(Modal::Shaders),
                        ColorTarget::Theme(_) => Some(Modal::Settings),
                        ColorTarget::Guide => Some(Modal::Guides),
                        _ => None,
                    };
                    self.focus = None;
                }
                Err(e) => self.status = e.to_string(),
            },
            Modal::Export { scale, opaque } => {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .set_file_name("dessin.png")
                    .save_file()
                {
                    let p = with_extension(p, "png");
                    let tx = self.jobs_tx.clone();
                    let doc = self.editor.document.clone();
                    self.busy = true;
                    self.modal = None;
                    self.status = "Export PNG…".into();
                    std::thread::spawn(move || {
                        let r =
                            project::export_png(&p, &doc, scale, opaque.then_some([16, 16, 20]));
                        let _ = tx.send(Job::Exported(r, p));
                    });
                }
            }
            Modal::Text { .. } => self.perform(Action::CopyExport),
            Modal::Help | Modal::Charsets | Modal::Shaders | Modal::Guides => self.modal = None,
            Modal::Recovery => self.perform(Action::Recover),
            Modal::Loss { .. } => self.perform(Action::SaveContinue),
        }
        if self.modal.is_none() {
            self.focus = None;
            self.keyboard_canvas = true;
            self.color_drag = None;
        }
    }
    fn clip_set(&mut self, text: &str) -> Result<(), String> {
        if self.clipboard.is_none() {
            self.clipboard = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
        }
        self.clipboard
            .as_mut()
            .unwrap()
            .set_text(text)
            .map_err(|e| format!("Presse-papiers : {e}"))
    }
    fn clip_get(&mut self) -> Result<String, String> {
        if self.clipboard.is_none() {
            self.clipboard = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
        }
        self.clipboard
            .as_mut()
            .unwrap()
            .get_text()
            .map_err(|e| format!("Presse-papiers : {e}"))
    }
    fn copy(&mut self, cut: bool) {
        let r = self
            .editor
            .selection
            .unwrap_or(self.editor.document.bounds());
        let block = Block::copy(&self.editor.document, r);
        let text = self.editor.document.text(Some(r));
        if let Err(e) = self.clip_set(&text) {
            self.status = e;
            return;
        }
        self.internal_clip = Some((block, text));
        if cut {
            self.editor.clear_selection();
            self.changed();
        }
        self.status = "Cellules et texte copiés. Coller puis Entrée pour placer.".into();
    }
    fn paste(&mut self) {
        self.dismiss_gesture();
        let text = match self.clip_get() {
            Ok(t) => t,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let block = if let Some((block, old)) = &self.internal_clip {
            if *old == text {
                Ok(block.clone())
            } else {
                Block::from_text(&text, self.brush)
            }
        } else {
            Block::from_text(&text, self.brush)
        };
        match block {
            Ok(block) => {
                self.floating = Some(Floating {
                    block,
                    position: self.cursor,
                });
                self.focus = None;
                self.keyboard_canvas = true;
                self.status = "Collage : déplacer, Entrée confirme ; Échap annule.".into();
            }
            Err(e) => self.status = e,
        }
    }
    fn start_load(&mut self, p: PathBuf, recovered: bool) {
        if self.busy {
            self.status = "Une opération est déjà en cours.".into();
            return;
        }
        self.loading = true;
        self.dismiss_gesture();
        let tx = self.jobs_tx.clone();
        self.busy = true;
        self.status = "Ouverture…".into();
        std::thread::spawn(move || {
            let r = project::load(&p);
            let _ = tx.send(Job::Loaded(r, p, recovered));
        });
    }
    pub fn open_path(&mut self, p: PathBuf) {
        self.start_load(p, false);
    }
    pub fn import_path(&mut self, p: PathBuf) {
        let tx = self.jobs_tx.clone();
        self.busy = true;
        self.status = "Lecture de la référence…".into();
        std::thread::spawn(move || {
            let _ = tx.send(Job::Reference(project::import_reference(&p)));
        });
    }
    fn save(&mut self, save_as: bool) {
        self.dismiss_gesture();
        let path = if save_as || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("Projet Ditto", &["ditto"])
                .set_file_name("dessin.ditto")
                .save_file()
                .map(|p| with_extension(p, "ditto"))
        } else {
            self.path.clone()
        };
        if let Some(p) = path {
            let tx = self.jobs_tx.clone();
            let doc = self.editor.document.clone();
            let revision = self.editor.revision;
            self.busy = true;
            self.status = "Enregistrement…".into();
            std::thread::spawn(move || {
                let r = project::save(&p, &doc);
                let _ = tx.send(Job::Saved(r, p, revision, false));
            });
        } else {
            self.after_save = None;
        }
    }
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        while let Ok(job) = self.jobs_rx.try_recv() {
            changed = true;
            self.busy = false;
            match job {
                Job::Loaded(Ok(doc), p, recovered) => {
                    self.loading = false;
                    self.reset(doc, if recovered { None } else { Some(p) }, recovered);
                    self.status = if recovered {
                        "Brouillon restauré : enregistrer le projet."
                    } else {
                        "Projet ouvert."
                    }
                    .into();
                }
                Job::Loaded(Err(e), _, _) => {
                    self.loading = false;
                    self.status = format!("Ouverture impossible : {e:#}");
                }
                Job::Reference(Ok(asset)) => {
                    let r = Reference::fit(
                        asset,
                        self.editor.document.width,
                        self.editor.document.height,
                        false,
                    );
                    self.editor.edit(|d| d.reference = Some(r));
                    self.changed();
                    self.status = "Référence ajoutée. Déverrouiller pour déplacer.".into();
                }
                Job::Reference(Err(e)) => self.status = format!("Image : {e:#}"),
                Job::Saved(Ok(()), p, revision, auto) => {
                    if auto {
                        self.owns_recovery = true;
                        self.autosaved_revision = Some(revision);
                    } else {
                        self.path = Some(p);
                        self.editor.saved_revision = revision;
                        self.status = "Projet enregistré.".into();
                        if self.owns_recovery && !self.editor.dirty() {
                            let _ = std::fs::remove_file(&self.recovery);
                            self.owns_recovery = false;
                        }
                        if let Some(next) = self.after_save.take() {
                            self.activate(next);
                        }
                    }
                }
                Job::Saved(Err(e), _, _, auto) => {
                    self.status = format!(
                        "{} : {e:#}",
                        if auto {
                            "Récupération"
                        } else {
                            "Enregistrement"
                        }
                    );
                    self.after_save = None;
                    if auto {
                        self.owns_recovery = true;
                        self.autosaved_revision = Some(self.editor.revision);
                    }
                }
                Job::Exported(Ok(()), p) => {
                    self.status = format!("PNG enregistré : {}", p.display())
                }
                Job::Exported(Err(e), _) => self.status = format!("Export : {e:#}"),
            }
        }
        if !self.busy
            && self.editor.dirty()
            && !self.editor.pending()
            && self.autosaved_revision != Some(self.editor.revision)
            && self.last_change.elapsed() > Duration::from_secs(3)
        {
            let tx = self.jobs_tx.clone();
            let path = self.recovery.clone();
            let doc = self.editor.document.clone();
            let rev = self.editor.revision;
            self.busy = true;
            std::thread::spawn(move || {
                let r = (|| {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    project::save(&path, &doc)
                })();
                let _ = tx.send(Job::Saved(r, path, rev, true));
            });
        }
        changed
    }
    pub fn needs_timer(&self) -> bool {
        self.busy || (self.editor.dirty() && self.autosaved_revision != Some(self.editor.revision))
    }
    fn clamp_cursor(&mut self) {
        self.cursor.0 = self.cursor.0.clamp(
            0,
            self.editor.document.width as i32
                - if self.edit_mode == EditMode::Keyboard {
                    0
                } else {
                    1
                },
        );
        self.cursor.1 = self
            .cursor
            .1
            .clamp(0, self.editor.document.height as i32 - 1);
    }
    pub fn text_input(&mut self, text: &str) {
        if self.loading {
            return;
        }
        self.ime.clear();
        if text.is_empty() {
            return;
        }
        if self.modal.is_some() {
            self.input_text(text);
            return;
        }
        if self.edit_mode == EditMode::Keyboard
            && self.keyboard_canvas
            && self.focus.is_none()
            && !self.ref_transform
        {
            match font::parse_text(text) {
                Ok(rows) => {
                    self.insert_keyboard_rows(&rows);
                }
                Err(e) => self.status = e,
            }
            return;
        }
        if self.tool != Tool::Text || !self.keyboard_canvas {
            return;
        }
        let rows = match font::parse_text(text) {
            Ok(r) => r,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let width = rows.iter().map(Vec::len).max().unwrap_or(0);
        if self.cursor.0 + width as i32 > self.editor.document.width as i32
            || self.cursor.1 + rows.len() as i32 > self.editor.document.height as i32
        {
            self.status = "Texte hors grille. Repositionner le curseur.".into();
            return;
        }
        self.editor.begin();
        for (y, row) in rows.iter().enumerate() {
            for (x, g) in row.iter().enumerate() {
                let mut b = self.brush;
                b.cell.glyph = *g;
                b.glyph = true;
                self.editor.paint(
                    &[(self.cursor.0 + x as i32, self.cursor.1 + y as i32)],
                    b,
                    false,
                );
            }
        }
        self.editor.commit();
        self.cursor.1 += rows.len() as i32 - 1;
        self.cursor.0 += rows.last().map_or(0, Vec::len) as i32;
        self.changed();
    }
    pub fn paste_field(&mut self) {
        if let Ok(t) = self.clip_get() {
            self.input_text(&t);
        }
    }
    fn input_text(&mut self, text: &str) {
        let replace = self.input_replace;
        let Some(m) = &mut self.modal else {
            return;
        };
        let color = matches!(m, Modal::Color { .. });
        let value =
            match m {
                Modal::New { width, height } | Modal::Resize { width, height } => {
                    if self.input == 0 { width } else { height }
                }
                Modal::Color { value, .. } => value,
                _ => return,
            };
        let accepted = text
            .chars()
            .filter(|c| {
                if color {
                    c.is_ascii_hexdigit()
                } else {
                    c.is_ascii_digit()
                }
            })
            .collect::<String>();
        if accepted.is_empty() {
            return;
        }
        if replace {
            value.clear();
            self.input_replace = false;
        }
        value.push_str(&accepted);
        value.truncate(8);
        self.sync_color_picker();
    }
    pub fn backspace(&mut self) {
        if let Some(m) = &mut self.modal {
            let value = match m {
                Modal::New { width, height } | Modal::Resize { width, height } => {
                    if self.input == 0 { width } else { height }
                }
                Modal::Color { value, .. } => value,
                _ => return,
            };
            if self.input_replace {
                value.clear();
                self.input_replace = false;
            } else {
                value.pop();
            }
            self.sync_color_picker();
            return;
        }
        if self.edit_mode == EditMode::Keyboard {
            self.keyboard_erase(true);
            return;
        }
        if self.tool == Tool::Text {
            self.cursor.0 = (self.cursor.0 - 1).max(self.text_origin);
            self.editor.begin();
            self.editor.paint(&[self.cursor], self.brush, true);
            self.editor.commit();
            self.changed();
        }
    }
    fn sync_color_picker(&mut self) {
        if let Some(Modal::Color { value, .. }) = &self.modal
            && let Ok(c) = project::color_hex(value)
        {
            self.color_picker.sync(c);
        }
    }
    fn write_picker(&mut self) {
        if let Some(Modal::Color { value, .. }) = &mut self.modal {
            *value = self.color_picker.hex();
            self.input_replace = true;
        }
    }
    pub fn adjust_color(&mut self, control: ColorControl, x: i32, y: i32) {
        if matches!(self.modal, Some(Modal::Color { .. })) {
            self.color_picker.adjust(control, x, y);
            self.write_picker();
        }
    }
    pub fn move_cursor(&mut self, x: i32, y: i32, shift: bool) {
        if matches!(self.modal, Some(Modal::Color { .. }))
            && let Some(hit) = self.focus.and_then(|i| self.hits.get(i))
            && let Action::ColorControl(control) = hit.action
        {
            self.adjust_color(control, x, y);
            return;
        }
        if self.modal.is_some() {
            self.tab(x < 0 || y < 0);
            return;
        }
        if let Some(focus) = self.focus {
            let group: Vec<usize> = if self
                .hits
                .get(focus)
                .is_some_and(|h| matches!(h.action, Action::Glyph(_) | Action::MappedKey(_)))
            {
                self.hits
                    .iter()
                    .enumerate()
                    .filter_map(|(i, h)| {
                        matches!(h.action, Action::Glyph(_) | Action::MappedKey(_)).then_some(i)
                    })
                    .collect()
            } else if self
                .hits
                .get(focus)
                .is_some_and(|h| matches!(h.action, Action::Palette(_)))
            {
                self.hits
                    .iter()
                    .enumerate()
                    .filter_map(|(i, h)| matches!(h.action, Action::Palette(_)).then_some(i))
                    .collect()
            } else {
                Vec::new()
            };
            if !group.is_empty() {
                let index = group.iter().position(|i| *i == focus).unwrap_or(0) as i32;
                let stride = if self.edit_mode == EditMode::Keyboard {
                    8
                } else {
                    16
                };
                let step = if y != 0 { y * stride } else { x };
                self.focus = Some(group[(index + step).rem_euclid(group.len() as i32) as usize]);
            } else {
                self.tab(x < 0 || y < 0);
            }
            return;
        }
        if self.ref_transform {
            self.perform(Action::RefNudge(x, y));
            return;
        }
        let before = self.cursor;
        if self.edit_mode == EditMode::Keyboard && !shift {
            self.editor.selection = None;
            if matches!(self.gesture, Some(Gesture::Select { .. })) {
                self.gesture = None;
            }
        }
        self.cursor.0 += x;
        self.cursor.1 += y;
        self.clamp_cursor();
        if shift && (self.tool == Tool::Select || self.edit_mode == EditMode::Keyboard) {
            if self.gesture.is_none() {
                self.gesture = Some(Gesture::Select { origin: before });
            }
            if let Some(Gesture::Select { origin }) = self.gesture {
                let last = self.editor.document.width as i32 - 1;
                self.editor.selection = Some(
                    Rect::from_points(
                        (origin.0.min(last), origin.1),
                        (self.cursor.0.min(last), self.cursor.1),
                    )
                    .intersection(self.editor.document.bounds()),
                );
            }
        }
        if let Some(f) = &mut self.floating {
            f.position = self.cursor;
        }
        self.keep_cursor_visible();
    }
    fn keep_cursor_visible(&mut self) {
        let x = self.origin.0 + self.cursor.0 as f32 * self.cell_width();
        let y = self.origin.1 + self.cursor.1 as f32 * self.cell_size;
        let dx = if x < self.canvas.x {
            self.canvas.x - x
        } else if x + self.cell_width() > self.canvas.x + self.canvas.w {
            self.canvas.x + self.canvas.w - x - self.cell_width()
        } else {
            0.
        };
        let dy = if y < self.canvas.y {
            self.canvas.y - y
        } else if y + self.cell_size > self.canvas.y + self.canvas.h {
            self.canvas.y + self.canvas.h - y - self.cell_size
        } else {
            0.
        };
        if dx != 0. || dy != 0. {
            self.fit = false;
            self.pan.0 += dx;
            self.pan.1 += dy;
            self.layout(self.width, self.height);
        }
    }
    pub fn tab(&mut self, back: bool) {
        let glyph = self
            .hits
            .iter()
            .position(|h| matches!(h.action,Action::Glyph(g) if g==self.brush.cell.glyph))
            .or_else(|| {
                self.hits
                    .iter()
                    .position(|h| matches!(h.action, Action::Glyph(_) | Action::MappedKey(_)))
            });
        let palette=self.hits.iter().position(|h|matches!(h.action,Action::Palette(i) if self.editor.document.palette.get(i)==Some(&self.brush.cell.fg))).or_else(||self.hits.iter().position(|h|matches!(h.action,Action::Palette(_))));
        let stops: Vec<usize> = self
            .hits
            .iter()
            .enumerate()
            .filter_map(|(i, h)| match h.action {
                Action::Glyph(_) | Action::MappedKey(_) => (Some(i) == glyph).then_some(i),
                Action::Palette(_) => (Some(i) == palette).then_some(i),
                _ => Some(i),
            })
            .collect();
        if stops.is_empty() {
            return;
        }
        let current = self.focus.map(|i| match self.hits[i].action {
            Action::Glyph(_) | Action::MappedKey(_) => glyph.unwrap_or(i),
            Action::Palette(_) => palette.unwrap_or(i),
            _ => i,
        });
        let n = match current.and_then(|i| stops.iter().position(|v| *v == i)) {
            Some(i) => {
                if back {
                    (i + stops.len() - 1) % stops.len()
                } else {
                    (i + 1) % stops.len()
                }
            }
            None => {
                if back {
                    stops.len() - 1
                } else {
                    0
                }
            }
        };
        let i = stops[n];
        self.focus = Some(i);
        self.keyboard_canvas = false;
        if let Action::Input(n) = self.hits[i].action {
            self.input = n;
            self.input_replace = true;
        }
    }
    pub fn focus_zone(&mut self, zone: u8) {
        self.keyboard_canvas = false;
        self.focus = self.hits.iter().position(|h| match zone {
            0 => {
                if self.edit_mode == EditMode::Keyboard {
                    h.action == Action::Charsets
                } else {
                    matches!(h.action,Action::Glyph(g) if g==self.brush.cell.glyph)
                }
            }
            1 => matches!(h.action, Action::Palette(_)),
            _ => {
                if self.edit_mode == EditMode::Keyboard {
                    h.action == Action::ToggleEditMode
                } else {
                    matches!(h.action, Action::Tool(_))
                }
            }
        });
    }
    pub fn enter(&mut self) {
        if self.loading {
            return;
        }
        if let Some(i) = self.focus {
            if let Some(hit) = self.hits.get(i) {
                let a = if matches!(hit.action, Action::Input(_) | Action::ColorControl(_)) {
                    Action::Submit
                } else {
                    hit.action.clone()
                };
                self.activate(a);
            }
            return;
        }
        if self.modal.is_some() {
            self.activate(Action::Submit);
            return;
        }
        if let Some(f) = self.floating.clone() {
            if !f.block.fits(&self.editor.document, f.position) {
                self.status = "Collage hors grille : déplacer avant de valider.".into();
                return;
            }
            self.editor.edit(|d| f.block.paste(d, f.position));
            self.editor.selection = Some(Rect {
                x: f.position.0,
                y: f.position.1,
                w: f.block.width as i32,
                h: f.block.height as i32,
            });
            self.floating = None;
            self.changed();
            return;
        }
        if let Some(Gesture::Move { rect, origin }) = self.gesture.clone() {
            let to = (
                rect.x + self.cursor.0 - origin.0,
                rect.y + self.cursor.1 - origin.1,
            );
            if self.editor.move_selection(to) {
                self.gesture = None;
                self.changed();
            } else {
                self.status = "Sélection hors grille.".into();
            }
            return;
        }
        if let Some(Gesture::Shape { origin }) = self.gesture.clone() {
            self.editor.begin();
            self.editor.paint(
                &shape(self.tool, origin, self.cursor, self.filled),
                self.brush,
                false,
            );
            self.editor.commit();
            self.gesture = None;
            self.changed();
            return;
        }
        if let Some(Gesture::Select { origin }) = self.gesture {
            if self.edit_mode == EditMode::Mouse {
                self.editor.selection = Some(
                    Rect::from_points(origin, self.cursor)
                        .intersection(self.editor.document.bounds()),
                );
                self.gesture = None;
                return;
            }
            self.gesture = None;
        }
        if self.edit_mode == EditMode::Keyboard {
            self.editor.selection = None;
            if self.cursor.1 + 1 >= self.editor.document.height as i32 {
                self.status = "Fin du document : déplacer le curseur pour continuer.".into();
                return;
            }
            self.cursor.0 = 0;
            self.cursor.1 += 1;
            self.text_origin = 0;
            self.keep_cursor_visible();
            return;
        }
        match self.tool {
            Tool::Guide | Tool::GuideErase => self.activate(Action::Guides),
            Tool::Line | Tool::Rectangle => {
                self.gesture = Some(Gesture::Shape {
                    origin: self.cursor,
                })
            }
            Tool::Select => {
                self.gesture = Some(Gesture::Select {
                    origin: self.cursor,
                })
            }
            Tool::Pick => self.pick(self.cursor),
            Tool::Text => {
                self.cursor.0 = self.text_origin;
                if self.cursor.1 + 1 < self.editor.document.height as i32 {
                    self.cursor.1 += 1;
                }
            }
            Tool::Fill => {
                self.editor.begin();
                self.editor.flood(self.cursor, self.brush);
                self.editor.commit();
                self.changed();
            }
            Tool::Pencil | Tool::Eraser | Tool::Recolor => {
                self.editor.begin();
                self.paint_cells(&[self.cursor]);
                self.editor.commit();
                self.changed();
            }
        }
    }
    pub fn pick(&mut self, p: (i32, i32)) {
        if let Some(c) = self.editor.document.get(p.0, p.1) {
            self.brush.cell = c;
            self.choose_glyph(c.glyph);
        }
    }
    pub fn mouse_down(&mut self, right: bool) {
        if self.loading {
            return;
        }
        if self.modal.is_some() {
            if matches!(self.modal, Some(Modal::Color { .. })) && !right {
                for control in [ColorControl::Plane, ColorControl::Hue] {
                    let rect = color_picker::rect(self.width, control);
                    if rect.contains(self.mouse) {
                        self.activate(Action::ColorControl(control));
                        self.color_drag = Some(control);
                        self.color_picker.pointer(control, rect, self.mouse);
                        self.write_picker();
                        return;
                    }
                }
            }
            if matches!(self.modal, Some(Modal::Shaders))
                && !right
                && self.shader_preview_rect().contains(self.mouse)
            {
                self.focus = None;
                self.shader_preview_drag = Some(PreviewDrag {
                    mouse: self.mouse,
                    pan: self.shader_preview_pan,
                });
                return;
            }
            if !right
                && let Some(h) = self
                    .hits
                    .iter()
                    .rev()
                    .find(|h| h.rect.contains(self.mouse))
                    .cloned()
            {
                self.activate(h.action);
            }
            return;
        }
        if let Some(h) = self
            .hits
            .iter()
            .rev()
            .find(|h| h.rect.contains(self.mouse))
            .cloned()
        {
            if right {
                if let Action::Palette(i) = h.action {
                    self.brush.cell.bg = Some(self.editor.document.palette[i]);
                    self.brush.bg = true;
                }
            } else {
                self.activate(h.action);
            }
            return;
        }
        if self.space && self.canvas.contains(self.mouse) {
            self.fit = false;
            self.gesture = Some(Gesture::Pan {
                mouse: self.mouse,
                offset: self.pan,
            });
            return;
        }
        let Some(p) = self.cell_at(self.mouse) else {
            return;
        };
        self.cursor = p;
        self.focus = None;
        self.keyboard_canvas = true;
        if right {
            self.pick(p);
            return;
        }
        if self.ref_transform {
            if let Some(r) = &self.editor.document.reference
                && !r.locked
            {
                self.gesture = Some(Gesture::Reference {
                    mouse: self.mouse,
                    x: r.x,
                    y: r.y,
                });
                self.editor.begin();
            }
            return;
        }
        if let Some(f) = &mut self.floating {
            f.position = p;
            self.enter();
            return;
        }
        if self.edit_mode == EditMode::Keyboard {
            self.editor.selection = None;
            self.text_origin = 0;
            self.gesture = Some(Gesture::Select { origin: p });
            return;
        }
        match self.tool {
            Tool::Guide | Tool::GuideErase => {
                if !self.editor.document.guides.visible {
                    self.status =
                        "Guides masqués : les afficher dans Guides (F8) avant de tracer.".into();
                    return;
                }
                let point = self.guide_point(self.mouse).unwrap();
                self.editor.begin();
                if self.tool == Tool::Guide {
                    if !self
                        .editor
                        .document
                        .guides
                        .start(point, self.guide_color, self.guide_width)
                    {
                        self.editor.cancel();
                        self.status =
                            "Calque de guides plein : effacer des traits pour continuer.".into();
                        return;
                    }
                } else {
                    self.editor
                        .document
                        .guides
                        .erase(point, point, self.guide_width.max(1.));
                }
                self.gesture = Some(Gesture::Guide { last: Some(point) });
            }
            Tool::Pencil | Tool::Eraser | Tool::Recolor => {
                self.editor.begin();
                self.paint_cells(&[p]);
                self.gesture = Some(if self.tool == Tool::Recolor {
                    Gesture::Recolor { last: Some(p) }
                } else {
                    Gesture::Stroke { last: p }
                });
            }
            Tool::Line | Tool::Rectangle => self.gesture = Some(Gesture::Shape { origin: p }),
            Tool::Select => {
                if let Some(r) = self.editor.selection.filter(|r| r.contains(p.0, p.1)) {
                    self.gesture = Some(Gesture::Move { rect: r, origin: p });
                } else {
                    self.editor.selection = None;
                    self.gesture = Some(Gesture::Select { origin: p });
                }
            }
            Tool::Fill => self.enter(),
            Tool::Text => self.text_origin = p.0,
            Tool::Pick => self.pick(p),
        }
    }
    pub fn mouse_move(&mut self, p: (f32, f32)) {
        self.mouse = p;
        if self.modal.is_some() {
            if matches!(self.modal, Some(Modal::Color { .. }))
                && let Some(control) = self.color_drag
            {
                self.color_picker
                    .pointer(control, color_picker::rect(self.width, control), p);
                self.write_picker();
            }
            if matches!(self.modal, Some(Modal::Shaders))
                && let Some(drag) = self.shader_preview_drag
            {
                self.shader_preview_pan = (
                    drag.pan.0 + p.0 - drag.mouse.0,
                    drag.pan.1 + p.1 - drag.mouse.1,
                );
            }
            return;
        }
        let cell = self.clamped_cell(p);
        if let Some(g) = self.gesture.clone() {
            match g {
                Gesture::Recolor { last } => {
                    let point = self.cell_at(p);
                    if let Some(point) = point {
                        self.cursor = point;
                        self.paint_cells(&line(last.unwrap_or(point), point));
                    }
                    self.gesture = Some(Gesture::Recolor { last: point });
                }
                Gesture::Guide { last } => {
                    let mut next = self.guide_point(p);
                    if let Some(point) = next {
                        if self.tool == Tool::GuideErase {
                            self.editor.document.guides.erase(
                                last.unwrap_or(point),
                                point,
                                self.guide_width.max(1.),
                            );
                        } else {
                            let ok = if last.is_none() {
                                self.editor.document.guides.start(
                                    point,
                                    self.guide_color,
                                    self.guide_width,
                                )
                            } else {
                                self.editor.document.guides.append(point)
                            };
                            if !ok {
                                next = None;
                                self.status =
                                    "Limite de points atteinte : terminer le trait.".into();
                            }
                        }
                    }
                    // Leaving the board breaks the stroke, avoiding bridges on re-entry.
                    self.gesture = Some(Gesture::Guide { last: next });
                }
                Gesture::Stroke { last } => {
                    self.cursor = cell;
                    self.paint_cells(&line(last, cell));
                    self.gesture = Some(Gesture::Stroke { last: cell });
                }
                Gesture::Shape { .. } | Gesture::Move { .. } => self.cursor = cell,
                Gesture::Select { origin } => {
                    self.cursor = cell;
                    self.editor.selection = Some(Rect::from_points(origin, cell));
                }
                Gesture::Reference { mouse, x, y } => {
                    if let Some(r) = &mut self.editor.document.reference {
                        r.x = (x + (p.0 - mouse.0) / (self.cell_size * ditto::typeface::ASPECT))
                            .clamp(-1e6, 1e6);
                        r.y = (y + (p.1 - mouse.1) / self.cell_size).clamp(-1e6, 1e6);
                    }
                }
                Gesture::Pan { mouse, offset } => {
                    self.pan = (offset.0 + p.0 - mouse.0, offset.1 + p.1 - mouse.1);
                    self.layout(self.width, self.height);
                }
            }
        } else if let Some(cell) = self.cell_at(p) {
            if (self.edit_mode == EditMode::Mouse && self.tool != Tool::Text)
                || self.floating.is_some()
            {
                self.cursor = cell;
            }
            if let Some(f) = &mut self.floating {
                f.position = cell;
            }
        }
    }
    pub fn mouse_up(&mut self) {
        if self.color_drag.take().is_some() {
            return;
        }
        if self.shader_preview_drag.take().is_some() {
            return;
        }
        if let Some(g) = self.gesture.clone() {
            match g {
                Gesture::Recolor { .. }
                | Gesture::Guide { .. }
                | Gesture::Stroke { .. }
                | Gesture::Reference { .. } => {
                    self.editor.commit();
                    self.gesture = None;
                    self.changed();
                }
                Gesture::Select { origin } if self.edit_mode == EditMode::Keyboard => {
                    self.editor.selection = if origin == self.cursor {
                        None
                    } else {
                        Some(
                            Rect::from_points(origin, self.cursor)
                                .intersection(self.editor.document.bounds()),
                        )
                    };
                    self.gesture = None;
                }
                Gesture::Shape { .. } | Gesture::Move { .. } | Gesture::Select { .. } => {
                    self.enter()
                }
                Gesture::Pan { .. } => self.gesture = None,
            }
        }
    }
    pub fn lost_focus(&mut self) {
        self.color_drag = None;
        self.shader_preview_drag = None;
        self.editor.cancel();
        self.gesture = None;
        self.space = false;
        self.ime.clear();
    }
    pub fn zoom(&mut self, up: bool, p: (f32, f32)) {
        self.zoom_by(if up { 1.25 } else { 0.8 }, p);
    }
    pub fn shader_preview_rect(&self) -> ScreenRect {
        let extra_rows = shaders::KINDS.len().div_ceil(3).saturating_sub(3);
        ScreenRect::new(
            44.,
            240.,
            (self.width - 556.).max(1.),
            (self.height - 468. - extra_rows as f32 * 28.).max(1.),
        )
    }
    fn shader_preview_fit_scale(&self) -> f32 {
        let r = self.shader_preview_rect();
        ((r.w - 24.).max(1.) / (self.editor.document.width * ditto::typeface::CELL_WIDTH) as f32)
            .min(
                (r.h - 24.).max(1.)
                    / (self.editor.document.height * ditto::typeface::CELL_HEIGHT) as f32,
            )
    }
    pub fn shader_preview_scale(&self) -> f32 {
        self.shader_preview_zoom
            .unwrap_or_else(|| self.shader_preview_fit_scale())
    }
    pub fn shader_preview_board(&self) -> ScreenRect {
        let r = self.shader_preview_rect();
        let zoom = self.shader_preview_scale();
        let w = (self.editor.document.width * ditto::typeface::CELL_WIDTH) as f32 * zoom;
        let h = (self.editor.document.height * ditto::typeface::CELL_HEIGHT) as f32 * zoom;
        ScreenRect::new(
            r.x + (r.w - w) / 2. + self.shader_preview_pan.0,
            r.y + (r.h - h) / 2. + self.shader_preview_pan.1,
            w,
            h,
        )
    }
    pub fn zoom_shader_preview(&mut self, factor: f32, p: (f32, f32)) {
        if !matches!(self.modal, Some(Modal::Shaders))
            || !factor.is_finite()
            || factor <= 0.
            || factor == 1.
        {
            return;
        }
        let old = self.shader_preview_scale();
        let fit = self.shader_preview_fit_scale();
        let zoom = (old * factor).clamp(fit.min(0.05), fit.max(16.));
        if zoom == old {
            return;
        }
        let r = self.shader_preview_rect();
        let p = if r.contains(p) {
            p
        } else {
            (r.x + r.w / 2., r.y + r.h / 2.)
        };
        let board = self.shader_preview_board();
        let source = ((p.0 - board.x) / old, (p.1 - board.y) / old);
        self.shader_preview_zoom = Some(zoom);
        self.shader_preview_pan = (0., 0.);
        let centered = self.shader_preview_board();
        self.shader_preview_pan = (
            p.0 - source.0 * zoom - centered.x,
            p.1 - source.1 * zoom - centered.y,
        );
        self.shader_preview_drag = None;
    }
    pub fn zoom_by(&mut self, factor: f32, p: (f32, f32)) {
        if !factor.is_finite() || factor <= 0. || factor == 1. {
            return;
        }
        if self.editor.pending() {
            return;
        }
        let size = (self.cell_size * factor).clamp(1., 128.);
        if size == self.cell_size {
            return;
        }
        let point = if self.canvas.contains(p) {
            p
        } else {
            (
                self.canvas.x + self.canvas.w / 2.,
                self.canvas.y + self.canvas.h / 2.,
            )
        };
        let cell = (
            (point.0 - self.origin.0) / self.cell_width(),
            (point.1 - self.origin.1) / self.cell_size,
        );
        self.fit = false;
        self.cell_size = size;
        self.layout(self.width, self.height);
        self.pan.0 += point.0 - (self.origin.0 + cell.0 * self.cell_width());
        self.pan.1 += point.1 - (self.origin.1 + cell.1 * self.cell_size);
        self.layout(self.width, self.height);
    }
}
fn with_extension(mut p: PathBuf, ext: &str) -> PathBuf {
    if p.extension().is_none() {
        p.set_extension(ext);
    }
    p
}
pub fn recovery_path() -> PathBuf {
    if let Some(p) = std::env::var_os("DITTO_RECOVERY_PATH") {
        return PathBuf::from(p);
    }
    directories::ProjectDirs::from("org", "Ditto", "Ditto")
        .map(|d| d.data_local_dir().join("recovery.ditto"))
        .unwrap_or_else(|| Path::new(".").join(".ditto-recovery.ditto"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> State {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery-for-tests"));
        s.reset(Document::new(32, 32).unwrap(), None, false);
        s.layout(1184., 832.);
        s.frame();
        s
    }
    fn point(s: &State, x: i32, y: i32) -> (f32, f32) {
        (
            s.origin.0 + (x as f32 + 0.5) * s.cell_width(),
            s.origin.1 + (y as f32 + 0.5) * s.cell_size,
        )
    }
    #[test]
    fn actual_mouse_stroke_undo_and_focus_cancel() {
        let mut s = state();
        s.mouse_move(point(&s, 1, 1));
        s.mouse_down(false);
        s.mouse_move(point(&s, 25, 16));
        s.mouse_up();
        let doc = s.editor.document.clone();
        assert!(doc.cells.iter().filter(|c| c.glyph == 177).count() >= 25);
        s.activate(Action::Undo);
        assert!(!s.editor.dirty());
        s.activate(Action::Redo);
        assert_eq!(s.editor.document, doc);
        s.mouse_move(point(&s, 2, 20));
        s.mouse_down(false);
        s.mouse_move(point(&s, 18, 28));
        s.lost_focus();
        assert_eq!(s.editor.document, doc);
    }
    #[test]
    fn mouse_and_keyboard_rectangles_match() {
        let mut mouse = state();
        mouse.activate(Action::Tool(Tool::Rectangle));
        mouse.mouse_move(point(&mouse, 2, 3));
        mouse.mouse_down(false);
        mouse.mouse_move(point(&mouse, 18, 24));
        mouse.mouse_up();
        let mut keyboard = state();
        keyboard.activate(Action::Tool(Tool::Rectangle));
        keyboard.cursor = (2, 3);
        keyboard.enter();
        keyboard.move_cursor(16, 21, false);
        keyboard.enter();
        assert_eq!(mouse.editor.document, keyboard.editor.document);
    }
    #[test]
    fn text_at_last_cell_does_not_overwrite_on_next_key() {
        let mut s = state();
        s.activate(Action::Tool(Tool::Text));
        s.cursor = (31, 0);
        s.text_origin = 31;
        s.text_input("A");
        s.text_input("B");
        assert_eq!(
            s.editor.document.get(31, 0).unwrap().glyph,
            b'A' as font::Glyph
        );
        assert!(s.status.contains("hors grille"));
        s.backspace();
        assert_eq!(s.editor.document.get(31, 0).unwrap().glyph, 32);
    }
    #[test]
    fn modal_numeric_input_replaces_and_rejects_letters() {
        let mut s = state();
        s.perform(Action::New);
        s.text_input("abc12");
        assert!(matches!(&s.modal,Some(Modal::New{width,..}) if width=="12"));
        s.perform(Action::Input(1));
        s.text_input("24");
        s.enter();
        assert_eq!(
            (s.editor.document.width, s.editor.document.height),
            (12, 24)
        );
    }
    #[test]
    fn zoom_anchors_cell_and_does_not_modify_document() {
        let mut s = state();
        let p = point(&s, 12, 14);
        let old = s.cell_at(p);
        s.zoom(true, p);
        assert_eq!(s.cell_at(p), old);
        assert!(!s.editor.dirty());
    }
    #[test]
    fn loading_cannot_mutate_previous_document() {
        let mut s = state();
        s.loading = true;
        s.enter();
        s.text_input("A");
        s.activate(Action::EraseSelection);
        assert!(!s.editor.dirty());
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    #[test]
    fn unopened_recovery_is_never_deleted_on_close() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("recovery.ditto");
        std::fs::write(&file, b"precious").unwrap();
        let mut s = State::new(file.clone());
        s.activate(Action::Quit);
        assert_eq!(std::fs::read(&file).unwrap(), b"precious");
        let mut s = State::new(file.clone());
        s.activate(Action::Cancel);
        assert!(s.quit);
        assert!(file.exists());
    }
    #[test]
    fn compact_window_keeps_all_controls_reachable() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery"));
        s.modal = None;
        s.layout(1120., 720.);
        s.frame();
        assert_eq!(
            s.hits
                .iter()
                .filter(|h| matches!(h.action, Action::Glyph(_) | Action::MappedKey(_)))
                .count(),
            128
        );
        for hit in &s.hits {
            assert!(
                hit.rect.x >= 0.
                    && hit.rect.y >= 0.
                    && hit.rect.x + hit.rect.w <= s.width
                    && hit.rect.y + hit.rect.h <= s.height,
                "{} {:?}",
                hit.label,
                hit.rect
            );
        }
        s.activate(Action::GlyphPage(1));
        s.frame();
        assert!(
            s.hits
                .iter()
                .any(|h| matches!(h.action,Action::Glyph(g) if g>=256))
        );
        s.activate(Action::GlyphPage(-1));
        s.activate(Action::GlyphPage(-1));
        s.frame();
        assert!(s.hits.iter().any(|h| h.action == Action::Glyph(65)));
    }
    #[test]
    fn tab_leaves_glyph_grid_in_one_step() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery"));
        s.modal = None;
        s.layout(1184., 832.);
        s.frame();
        s.focus_zone(0);
        assert!(matches!(s.hits[s.focus.unwrap()].action, Action::Glyph(_)));
        s.tab(false);
        assert!(!matches!(s.hits[s.focus.unwrap()].action, Action::Glyph(_)));
    }
    #[test]
    fn text_caret_is_not_moved_by_mouse_hover() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery"));
        s.modal = None;
        s.layout(1184., 832.);
        s.tool = Tool::Text;
        s.cursor = (1, 1);
        s.mouse_move((
            s.origin.0 + 20. * s.cell_width(),
            s.origin.1 + 20. * s.cell_size,
        ));
        assert_eq!(s.cursor, (1, 1));
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;
    #[test]
    fn abandoning_navigation_does_not_forge_a_saved_revision() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery"));
        s.modal = None;
        s.enter();
        assert!(s.editor.dirty());
        s.activate(Action::New);
        s.activate(Action::Discard);
        assert!(matches!(s.modal, Some(Modal::New { .. })));
        s.activate(Action::Cancel);
        assert!(s.editor.dirty());
    }
    #[test]
    fn delete_without_selection_only_affects_current_cell() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery"));
        s.modal = None;
        s.enter();
        s.move_cursor(1, 0, false);
        s.enter();
        s.activate(Action::EraseSelection);
        assert_eq!(s.editor.document.get(0, 0).unwrap().glyph, 177);
        assert_eq!(s.editor.document.get(1, 0).unwrap().glyph, 32);
    }
}

impl State {
    pub fn active_charset(&self) -> &'static charset::Charset {
        &charset::ALL[self.charset]
    }
    pub fn glyph_page_size(&self) -> usize {
        if self.height < 800. { 128 } else { 256 }
    }
    pub fn mapping_page_size(&self) -> usize {
        charset::KEYS.len()
    }
    pub fn category_glyphs(&self) -> Vec<font::Glyph> {
        match self.category {
            1 => font::trait_glyphs(),
            2 => font::block_glyphs(),
            3 => self.recent.clone(),
            _ => (0..font::CHARS.len() as font::Glyph).collect(),
        }
    }
    pub fn keyboard_active(&self) -> bool {
        self.edit_mode == EditMode::Keyboard
            && self.modal.is_none()
            && self.keyboard_canvas
            && self.focus.is_none()
            && !self.loading
            && self.ime.is_empty()
            && !self.ref_transform
            && self.floating.is_none()
            && !matches!(
                self.gesture,
                Some(
                    Gesture::Move { .. }
                        | Gesture::Shape { .. }
                        | Gesture::Pan { .. }
                        | Gesture::Reference { .. }
                )
            )
    }
    pub fn keyboard_input(&mut self, text: &str) -> bool {
        if !self.keyboard_active() {
            return false;
        }
        if text == " " {
            self.insert_keyboard_rows(&[vec![32]]);
            return true;
        }
        if let Some(key) = charset::source(text) {
            let slot = charset::KEYS.iter().position(|k| *k == key).unwrap();
            self.insert_mapping(slot);
        } else {
            self.status="Touches du charset : A–Z et rangée 0–9. Changer de banque pour les autres glyphes.".into();
        }
        true
    }
    pub fn ime_commit(&mut self, text: &str) {
        self.ime.clear();
        // Composition is literal; only direct keyboard events use the charset.
        self.text_input(text);
    }

    fn insert_mapping(&mut self, slot: usize) {
        if self.edit_mode != EditMode::Keyboard
            || self.modal.is_some()
            || self.loading
            || self.ref_transform
        {
            return;
        }
        if let Some(g) = self.active_charset().at(self.charset_bank, slot) {
            self.last_key = Some(slot);
            self.choose_glyph(g);
            self.focus = None;
            self.keyboard_canvas = true;
            if self.insert_keyboard_rows(&[vec![g]]) {
                self.status = format!(
                    "{} → {} · {} · flèches pour déplacer",
                    charset::KEYS[slot],
                    font::character(g),
                    self.active_charset().name
                );
            }
        } else {
            self.status = "Cette touche est libre dans cette banque.".into();
        }
    }
    fn remember_key_edit(&mut self, parent: u64, before: ((i32, i32), Option<Rect>)) {
        if self.editor.revision != parent {
            self.key_history.insert(
                self.editor.revision,
                KeyEdit {
                    parent,
                    before,
                    after: (self.cursor, self.editor.selection),
                },
            );
            while self.key_history.len() > 8192 {
                self.key_history.pop_first();
            }
        }
    }
    fn insert_keyboard_rows(&mut self, rows: &[Vec<font::Glyph>]) -> bool {
        let origin = self.editor.selection.map_or(self.cursor, |r| (r.x, r.y));
        if origin.1 + rows.len() as i32 > self.editor.document.height as i32
            || rows.iter().enumerate().any(|(i, row)| {
                let x = if i == 0 { origin.0 } else { 0 };
                x + row.len() as i32 > self.editor.document.width as i32
            })
        {
            self.status = "Fin de ligne : Entrée pour continuer, ou déplacer le curseur.".into();
            return false;
        }
        let before = (self.cursor, self.editor.selection);
        let parent = self.editor.revision;
        self.editor.begin();
        if let Some(r) = self.editor.selection {
            for y in r.y..r.y + r.h {
                for x in r.x..r.x + r.w {
                    self.editor.document.set(x, y, Cell::default());
                }
            }
        }
        let mut b = self.brush;
        b.glyph = true;
        for (y, row) in rows.iter().enumerate() {
            let left = if y == 0 { origin.0 } else { 0 };
            for (x, g) in row.iter().enumerate() {
                if let Some(old) = self
                    .editor
                    .document
                    .get(left + x as i32, origin.1 + y as i32)
                {
                    b.cell.glyph = *g;
                    self.editor
                        .document
                        .set(left + x as i32, origin.1 + y as i32, b.apply(old));
                }
            }
        }
        self.editor.commit();
        self.editor.selection = None;
        self.gesture = None;
        let last_origin = if rows.len() == 1 { origin.0 } else { 0 };
        self.cursor = (
            last_origin + rows.last().map_or(0, Vec::len) as i32,
            origin.1 + rows.len().saturating_sub(1) as i32,
        );
        self.remember_key_edit(parent, before);
        self.changed();
        self.keep_cursor_visible();
        true
    }
    fn keyboard_erase(&mut self, backward: bool) {
        let before = (self.cursor, self.editor.selection);
        let parent = self.editor.revision;
        if let Some(r) = self.editor.selection {
            self.editor.clear_selection();
            self.cursor = (r.x, r.y);
            self.editor.selection = None;
            self.gesture = None;
        } else {
            if backward {
                if self.cursor.0 == 0 {
                    return;
                }
                self.cursor.0 -= 1;
            }
            self.editor.begin();
            self.editor.paint(&[self.cursor], self.brush, true);
            self.editor.commit();
        }
        self.remember_key_edit(parent, before);
        self.changed();
    }
}

#[cfg(test)]
mod charset_editing_tests {
    use super::*;
    fn state() -> State {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-charset-recovery"));
        s.modal = None;
        s.editor = Editor::new(Document::new(24, 12).unwrap());
        s.layout(1184., 832.);
        s.activate(Action::ToggleEditMode);
        s.frame();
        s
    }
    #[test]
    fn letters_and_digits_insert_and_advance() {
        let mut s = state();
        for key in ['a', '1', 'z', '9', 'e', '0', 'r'] {
            assert!(s.keyboard_input(&key.to_string()));
        }
        assert_eq!(s.cursor, (7, 0));
        for (i, k) in ['a', '1', 'z', '9', 'e', '0', 'r'].iter().enumerate() {
            assert_eq!(
                s.editor.document.get(i as i32, 0).unwrap().glyph,
                s.active_charset().resolve(0, *k).unwrap()
            );
        }
        assert_eq!(s.tool, Tool::Pencil);
    }
    #[test]
    fn mappings_click_and_key_are_identical() {
        let mut a = state();
        let mut b = state();
        let i = charset::KEYS.iter().position(|c| *c == '9').unwrap();
        a.keyboard_input("9");
        b.activate(Action::MappedKey(i));
        assert_eq!(a.editor.document, b.editor.document);
        assert_eq!(a.cursor, b.cursor);
    }
    #[test]
    fn toggle_and_charset_changes_never_rewrite_art() {
        let mut s = state();
        s.keyboard_input("a");
        let d = s.editor.document.clone();
        let rev = s.editor.revision;
        let cur = s.cursor;
        s.activate(Action::SetCharset(0));
        s.activate(Action::ToggleEditMode);
        assert_eq!(s.editor.document, d);
        assert_eq!(s.editor.revision, rev);
        assert_eq!(s.cursor, cur);
        assert!(!s.keyboard_input("b"));
        s.activate(Action::ToggleEditMode);
        assert_eq!(s.charset, 0);
    }
    #[test]
    fn keyboard_mode_positions_and_selects_with_mouse_without_painting() {
        let mut s = state();
        let p = (
            s.origin.0 + 3.5 * s.cell_width(),
            s.origin.1 + 2.5 * s.cell_size,
        );
        s.mouse_move(p);
        assert_eq!(s.cursor, (0, 0));
        s.mouse_down(false);
        s.mouse_up();
        assert_eq!(s.cursor, (3, 2));
        assert!(s.editor.selection.is_none());
        assert!(!s.editor.dirty());
        s.mouse_move(p);
        s.mouse_down(false);
        s.mouse_move((p.0 + 2. * s.cell_width(), p.1 + s.cell_size));
        s.mouse_up();
        assert_eq!(
            s.editor.selection,
            Some(Rect {
                x: 3,
                y: 2,
                w: 3,
                h: 2
            })
        );
        assert!(!s.editor.dirty());
    }
    #[test]
    fn replacement_undo_restores_cells_cursor_and_selection() {
        let mut s = state();
        s.editor.edit(|d| {
            for x in 3..7 {
                d.set(
                    x,
                    2,
                    Cell {
                        glyph: 219,
                        ..Cell::default()
                    },
                );
            }
        });
        s.cursor = (6, 2);
        s.editor.selection = Some(Rect {
            x: 3,
            y: 2,
            w: 4,
            h: 1,
        });
        let before = s.editor.document.clone();
        s.keyboard_input("o");
        assert_eq!(s.cursor, (4, 2));
        assert_eq!(
            s.editor.document.get(3, 2).unwrap().glyph,
            font::glyph('▖').unwrap()
        );
        assert_eq!(s.editor.document.get(4, 2).unwrap().glyph, 32);
        s.activate(Action::Undo);
        assert_eq!(s.editor.document, before);
        assert_eq!(s.cursor, (6, 2));
        assert!(s.editor.selection.is_some());
        s.activate(Action::Redo);
        assert_eq!(s.cursor, (4, 2));
        assert!(s.editor.selection.is_none());
    }
    #[test]
    fn end_of_line_requires_enter_and_backspace_moves_left() {
        let mut s = state();
        s.cursor = (23, 0);
        s.keyboard_input("a");
        let d = s.editor.document.clone();
        s.keyboard_input("1");
        assert_eq!(s.editor.document, d);
        assert!(s.status.contains("Fin de ligne"));
        s.backspace();
        assert_eq!(s.cursor, (23, 0));
        assert_eq!(s.editor.document.get(23, 0).unwrap().glyph, 32);
        s.enter();
        assert_eq!(s.cursor, (0, 1));
        s.keyboard_input("z");
        assert_eq!(s.cursor, (1, 1));
    }
    #[test]
    fn fields_composition_and_literal_paste_are_not_remapped() {
        let mut s = state();
        s.activate(Action::Resize);
        s.text_input("12");
        assert!(!s.editor.dirty());
        assert!(!s.keyboard_input("a"));
        s.activate(Action::Cancel);
        s.ime = "e".into();
        assert!(!s.keyboard_input("a"));
        s.ime_commit("é");
        assert_eq!(
            s.editor.document.get(0, 0).unwrap().glyph,
            font::glyph('é').unwrap()
        );
        let block = Block::from_text("a1+", s.brush).unwrap();
        assert_eq!(
            block
                .cells
                .iter()
                .map(|c| font::character(c.glyph))
                .collect::<String>(),
            "a1+"
        );
    }
    #[test]
    fn every_visible_mapping_and_control_fits_compact_window() {
        let mut s = state();
        s.layout(1120., 720.);
        s.frame();
        assert_eq!(
            s.hits
                .iter()
                .filter(|h| matches!(h.action, Action::MappedKey(_)))
                .count(),
            36
        );
        for h in &s.hits {
            assert!(
                h.rect.x + h.rect.w <= s.width && h.rect.y + h.rect.h <= s.height,
                "{}",
                h.label
            );
        }
        s.activate(Action::MappingPage);
        s.frame();
        assert_eq!(
            s.hits
                .iter()
                .filter(|h| matches!(h.action, Action::MappedKey(_)))
                .count(),
            36
        );
        assert!(s.hits.iter().all(|h| !matches!(h.action, Action::Glyph(_))));
    }
    #[test]
    fn braille_heights_show_only_assigned_keys_and_leave_free_keys_inert() {
        let mut s = state();
        s.activate(Action::SetCharset(5));
        s.activate(Action::CharsetBank(3));
        let g = s.active_charset().resolve(3, 'a').unwrap();
        s.keyboard_input("a");
        assert_eq!(s.editor.document.get(0, 0).unwrap().glyph, g);
        let revision = s.editor.revision;
        s.frame();
        assert_eq!(
            s.hits
                .iter()
                .filter(|h| matches!(h.action, Action::MappedKey(_)))
                .count(),
            19
        );
        s.keyboard_input("0");
        assert_eq!(s.editor.revision, revision);
        assert!(s.status.contains("libre"));
    }
    #[test]
    fn tabs_leave_the_mapping_grid_and_f3_reaches_charset_selector() {
        let mut s = state();
        s.focus = Some(
            s.hits
                .iter()
                .position(|h| matches!(h.action, Action::MappedKey(_)))
                .unwrap(),
        );
        s.tab(false);
        assert!(!matches!(
            s.hits[s.focus.unwrap()].action,
            Action::MappedKey(_)
        ));
        s.focus_zone(0);
        assert_eq!(s.hits[s.focus.unwrap()].action, Action::Charsets);
    }
}

#[cfg(test)]
mod composition_regressions {
    use super::*;
    #[test]
    fn empty_commit_cannot_erase_a_selection() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery"));
        s.modal = None;
        s.activate(Action::ToggleEditMode);
        s.keyboard_input("a");
        s.editor.selection = Some(Rect {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        });
        let d = s.editor.document.clone();
        s.ime_commit("");
        assert_eq!(s.editor.document, d);
        assert!(s.editor.selection.is_some());
    }
    #[test]
    fn selection_at_line_end_stays_inside_grid() {
        let mut s = State::new(PathBuf::from("/tmp/ditto-no-recovery"));
        s.modal = None;
        s.activate(Action::ToggleEditMode);
        s.cursor = (s.editor.document.width as i32, 0);
        s.move_cursor(1, 0, true);
        let r = s.editor.selection.unwrap();
        assert_eq!(r.w, 1);
        assert_eq!(r.x, s.editor.document.width as i32 - 1);
    }
}

impl State {
    pub fn keyboard_tab(&mut self, back: bool) {
        if !self.keyboard_active() {
            return;
        }
        if back {
            self.cursor.0 = ((self.cursor.0.saturating_sub(1)).max(0) / 4) * 4;
            self.editor.selection = None;
            self.gesture = None;
            return;
        }
        let x = self.editor.selection.map_or(self.cursor.0, |r| r.x);
        let count = (4 - x % 4).min(self.editor.document.width as i32 - x);
        if count > 0 {
            self.insert_keyboard_rows(&[vec![32; count as usize]]);
        }
    }
}

#[cfg(test)]
mod shader_window_tests {
    use super::*;
    fn state() -> State {
        let mut s = State::new(PathBuf::from("/tmp/ditto-shader-ui-test-unused"));
        s.modal = None;
        s.activate(Action::Shaders);
        s
    }
    #[test]
    fn shader_window_controls_fit_minimum_size_and_never_overlap() {
        let mut s = state();
        for kind in [Kind::Blur, Kind::ContourBlur]
            .into_iter()
            .chain(shaders::KINDS.into_iter().take(6))
        {
            s.activate(Action::ShaderAdd(kind));
        }
        for (width, height) in [(1120., 720.), (1184., 832.)] {
            s.layout(width, height);
            s.frame();
            for (i, a) in s.hits.iter().enumerate() {
                assert!(
                    a.rect.x >= 0.
                        && a.rect.y >= 0.
                        && a.rect.x + a.rect.w <= width
                        && a.rect.y + a.rect.h <= height,
                    "out of bounds: {}",
                    a.label
                );
                for b in &s.hits[i + 1..] {
                    let overlap = a.rect.intersect(b.rect);
                    assert!(
                        overlap.w == 0. || overlap.h == 0.,
                        "overlap: {} / {}",
                        a.label,
                        b.label
                    );
                }
            }
        }
        assert_eq!(
            s.hits
                .iter()
                .filter(|h| matches!(h.action, Action::ShaderAdd(_)))
                .count(),
            shaders::KINDS.len()
        );
        let original = s.editor.document.clone();
        s.activate(Action::ShaderAdd(Kind::Glow));
        assert_eq!(s.editor.document, original);
    }
    #[test]
    fn shader_edits_and_comparison_are_separate_from_drawing_history() {
        let mut s = state();
        s.activate(Action::ShaderPreset(0));
        let initial = s.editor.document.clone();
        let rev = s.editor.revision;
        s.activate(Action::ShaderBefore);
        assert!(s.shader_before);
        assert_eq!(s.editor.revision, rev);
        s.activate(Action::ShaderSelect(1));
        s.activate(Action::ShaderAdjust(1, 1));
        assert!(!s.shader_before);
        s.activate(Action::Undo);
        assert_eq!(s.editor.document, initial);
        s.activate(Action::Redo);
        s.activate(Action::ShaderMove(1, -1));
        assert_eq!(s.editor.document.shaders.layers[0].kind, Kind::Glow);
        s.activate(Action::ShaderRemove(0));
        s.activate(Action::Undo);
        assert_eq!(s.editor.document.shaders.layers.len(), 2);
        s.activate(Action::Cancel);
        assert!(s.modal.is_none());
        assert_eq!(s.editor.document.cells, initial.cells);
    }
    #[test]
    fn preview_zoom_anchors_pointer_and_survives_comparison_edits_and_reopening() {
        let mut s = state();
        let r = s.shader_preview_rect();
        let p = (r.x + r.w * 0.3, r.y + r.h * 0.4);
        let board = s.shader_preview_board();
        let before = (
            (p.0 - board.x) / s.shader_preview_scale(),
            (p.1 - board.y) / s.shader_preview_scale(),
        );
        let canvas = (s.cell_size, s.pan, s.fit, s.cursor);
        let doc = s.editor.document.clone();
        let revision = s.editor.revision;
        s.zoom_shader_preview(3., p);
        let board = s.shader_preview_board();
        assert!(((p.0 - board.x) / s.shader_preview_scale() - before.0).abs() < 0.001);
        assert!(((p.1 - board.y) / s.shader_preview_scale() - before.1).abs() < 0.001);
        s.activate(Action::ShaderPreviewPan(25, -15));
        let view = (s.shader_preview_zoom, s.shader_preview_pan);
        s.activate(Action::ShaderBefore);
        assert_eq!(s.editor.document, doc);
        assert_eq!(s.editor.revision, revision);
        s.activate(Action::ShaderPreset(0));
        s.activate(Action::ShaderAdjust(1, 1));
        s.activate(Action::Undo);
        assert_eq!((s.shader_preview_zoom, s.shader_preview_pan), view);
        s.activate(Action::Cancel);
        s.activate(Action::Shaders);
        assert_eq!((s.shader_preview_zoom, s.shader_preview_pan), view);
        assert_eq!((s.cell_size, s.pan, s.fit, s.cursor), canvas);
        s.activate(Action::ShaderPreviewActual);
        assert_eq!(s.shader_preview_scale(), 1.);
        s.activate(Action::ShaderPreviewFit);
        assert_eq!(
            (s.shader_preview_zoom, s.shader_preview_pan),
            (None, (0., 0.))
        );
        s.layout(1120., 720.);
        let r = s.shader_preview_rect();
        let board = s.shader_preview_board();
        assert!(
            board.x >= r.x
                && board.y >= r.y
                && board.x + board.w <= r.x + r.w
                && board.y + board.h <= r.y + r.h
        );
    }
    #[test]
    fn preview_drag_stops_on_release_focus_loss_and_close_without_painting() {
        let mut s = state();
        let r = s.shader_preview_rect();
        let p = (r.x + r.w / 2., r.y + r.h / 2.);
        let doc = s.editor.document.clone();
        let cursor = s.cursor;
        s.mouse_move(p);
        s.mouse_down(false);
        s.mouse_move((p.0 + 60., p.1 - 20.));
        assert_eq!(s.shader_preview_pan, (60., -20.));
        s.mouse_up();
        s.mouse_move(p);
        assert_eq!(s.shader_preview_pan, (60., -20.));
        s.mouse_down(false);
        s.lost_focus();
        s.mouse_move((p.0 + 10., p.1 + 10.));
        assert_eq!(s.shader_preview_pan, (60., -20.));
        s.mouse_down(false);
        s.activate(Action::Cancel);
        s.activate(Action::Shaders);
        s.mouse_move(p);
        assert_eq!(s.shader_preview_pan, (60., -20.));
        s.mouse_down(true);
        s.mouse_move((p.0 - 20., p.1));
        assert_eq!(s.shader_preview_pan, (60., -20.));
        assert_eq!(s.editor.document, doc);
        assert_eq!(s.cursor, cursor);
        assert!(!s.editor.dirty());
    }
    #[test]
    fn duotone_picker_returns_to_shaders_and_can_be_cancelled() {
        let mut s = state();
        s.activate(Action::ShaderAdd(Kind::Duotone));
        s.activate(Action::ShaderColor(1));
        s.text_input("123456");
        s.activate(Action::Submit);
        assert!(matches!(s.modal, Some(Modal::Shaders)));
        assert_eq!(
            s.editor.document.shaders.layers[0].colors[1],
            [0x12, 0x34, 0x56]
        );
        let doc = s.editor.document.clone();
        s.activate(Action::ShaderColor(0));
        s.text_input("AABBCC");
        s.activate(Action::Cancel);
        assert!(matches!(s.modal, Some(Modal::Shaders)));
        assert_eq!(s.editor.document, doc);
    }
}

#[cfg(test)]
mod guide_recolor_settings_tests {
    use super::*;
    fn state() -> State {
        let mut s = State::new(PathBuf::from("/tmp/ditto-tools-test-recovery"));
        s.reset(Document::new(32, 24).unwrap(), None, false);
        s.frame();
        s
    }
    fn point(s: &State, x: f32, y: f32) -> (f32, f32) {
        (
            s.origin.0 + x * s.cell_width(),
            s.origin.1 + y * s.cell_size,
        )
    }
    #[test]
    fn freehand_is_subcell_follows_zoom_breaks_at_edges_and_is_one_undo() {
        let mut s = state();
        let cells = s.editor.document.cells.clone();
        s.activate(Action::Tool(Tool::Guide));
        s.mouse_move(point(&s, 2.25, 3.75));
        s.mouse_down(false);
        s.mouse_move(point(&s, 5.3, 6.7));
        s.mouse_move((0., 0.));
        s.mouse_move(point(&s, 10.1, 6.6));
        s.mouse_up();
        assert!(Arc::ptr_eq(&cells, &s.editor.document.cells));
        let layer = s.editor.document.guides.clone();
        assert_eq!(layer.strokes.len(), 2);
        assert!((layer.strokes[0].points[0][0] - 2.25).abs() < 0.001);
        assert!(s.editor.dirty());
        s.activate(Action::Undo);
        assert!(s.editor.document.guides.strokes.is_empty());
        s.activate(Action::Redo);
        assert_eq!(s.editor.document.guides, layer);
        s.zoom_by(2., point(&s, 8., 8.));
        assert_eq!(s.editor.document.guides, layer);
        s.mouse_move(point(&s, 10., 7.));
        s.mouse_down(false);
        s.mouse_move(point(&s, 11., 8.));
        s.lost_focus();
        assert_eq!(s.editor.document.guides, layer);
        s.activate(Action::GuideVisible);
        let rev = s.editor.revision;
        s.mouse_down(false);
        s.mouse_up();
        assert_eq!(s.editor.revision, rev);
        assert!(!s.editor.pending());
    }
    #[test]
    fn recolor_drag_ignores_masks_and_blank_space_and_does_not_bridge_outside() {
        let mut s = state();
        for x in 0..32 {
            s.editor.document.set(
                x,
                5,
                Cell {
                    glyph: 65,
                    fg: [10, 20, 30],
                    bg: Some([1, 2, 3]),
                },
            );
        }
        s.brush.glyph = true;
        s.brush.bg = true;
        s.brush.fg = false;
        s.brush.cell.fg = [220, 30, 50];
        s.brush.cell.glyph = 219;
        s.activate(Action::Tool(Tool::Recolor));
        let before = s.editor.document.clone();
        s.mouse_move(point(&s, 2.5, 5.5));
        s.mouse_down(false);
        s.mouse_move(point(&s, 5.5, 5.5));
        s.mouse_move((0., 0.));
        s.mouse_move(point(&s, 12.5, 5.5));
        s.mouse_up();
        for x in 0..32 {
            let c = s.editor.document.get(x, 5).unwrap();
            assert_eq!(c.glyph, 65);
            assert_eq!(c.bg, Some([1, 2, 3]));
            assert_eq!(
                c.fg,
                if (2..=5).contains(&x) || x == 12 {
                    [220, 30, 50]
                } else {
                    [10, 20, 30]
                }
            );
        }
        s.activate(Action::Undo);
        assert_eq!(s.editor.document, before);
        s.activate(Action::RecolorSize(2));
        assert_eq!(s.recolor_radius, 2);
        s.mouse_down(false);
        s.mouse_move(point(&s, 15.5, 5.5));
        s.activate(Action::Cancel);
        assert_eq!(s.editor.document, before);
    }
    #[test]
    fn settings_preview_save_cancel_and_picker_are_independent_of_document() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("settings.json");
        let mut s = state();
        s.load_settings(p.clone());
        let d = s.editor.document.clone();
        let rev = s.editor.revision;
        s.activate(Action::Settings);
        s.activate(Action::ThemePreset(2));
        assert_eq!(s.theme(), Theme::preset(2));
        assert!(!p.exists());
        s.activate(Action::ThemeColor(Token::Accent));
        s.text_input("247AAC");
        s.activate(Action::Submit);
        assert!(matches!(s.modal, Some(Modal::Settings)));
        assert_eq!(s.theme().accent, [36, 122, 172]);
        s.activate(Action::Cancel);
        assert_eq!(s.theme(), Theme::default());
        assert!(!p.exists());
        s.activate(Action::Settings);
        s.activate(Action::ThemePreset(1));
        s.activate(Action::Submit);
        assert_eq!(settings::load(&p).unwrap().theme, Theme::preset(1));
        assert_eq!(s.editor.document, d);
        assert_eq!(s.editor.revision, rev);
        let mut restarted = state();
        restarted.load_settings(p);
        assert_eq!(restarted.theme(), Theme::preset(1));
        s.activate(Action::Guides);
        s.activate(Action::GuideColor);
        s.text_input("FFCC33");
        s.activate(Action::Submit);
        assert_eq!(s.guide_color, [255, 204, 51]);
        assert!(matches!(s.modal, Some(Modal::Guides)));
    }
    #[test]
    fn guide_order_and_recolor_all_are_independent_undoable_document_edits() {
        let mut s = state();
        assert!(!s.editor.document.guides.above_characters);
        s.editor.document.guides.start([1., 1.], [10, 20, 30], 0.2);
        s.editor.document.guides.append([5., 4.]);
        s.editor.document.guides.start([2., 1.], [40, 50, 60], 0.8);
        s.editor.document.guides.visible = false;
        let original = s.editor.document.clone();
        s.activate(Action::Guides);
        s.activate(Action::GuideColor);
        s.text_input("AABBCC");
        s.activate(Action::Submit);
        assert_eq!(
            s.editor.document, original,
            "ink picker affects future strokes only"
        );
        s.activate(Action::GuideRecolorAll);
        let colored = s.editor.document.clone();
        assert!(Arc::ptr_eq(&original.cells, &colored.cells));
        for (before, after) in original
            .guides
            .strokes
            .iter()
            .zip(colored.guides.strokes.iter())
        {
            assert_eq!(after.color, [170, 187, 204]);
            assert_eq!(before.points, after.points);
            assert_eq!(before.width, after.width);
        }
        assert!(!colored.guides.visible);
        let revision = s.editor.revision;
        s.activate(Action::GuideRecolorAll);
        assert_eq!(s.editor.revision, revision, "same ink is a no-op");
        s.activate(Action::GuideAbove);
        assert!(s.editor.document.guides.above_characters);
        s.activate(Action::Undo);
        assert_eq!(s.editor.document, colored);
        s.activate(Action::Undo);
        assert_eq!(s.editor.document, original);
        s.activate(Action::Redo);
        s.activate(Action::Redo);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("guides.ditto");
        project::save(&path, &s.editor.document).unwrap();
        assert_eq!(project::load(&path).unwrap(), s.editor.document);
    }
    #[test]
    fn failed_settings_save_keeps_draft_and_previous_preferences() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("blocked");
        std::fs::write(&p, "file").unwrap();
        let mut s = state();
        s.settings_path = Some(p.join("settings.json"));
        s.activate(Action::Settings);
        s.activate(Action::ThemePreset(2));
        s.activate(Action::Submit);
        assert!(matches!(s.modal, Some(Modal::Settings)));
        assert_eq!(s.settings, Settings::default());
        s.activate(Action::Cancel);
        assert_eq!(s.theme(), Theme::default());
    }
    #[test]
    fn new_controls_fit_and_do_not_overlap_at_minimum_size() {
        let mut s = state();
        s.layout(1120., 720.);
        s.tool = Tool::Recolor;
        for modal in [None, Some(Modal::Guides), Some(Modal::Settings)] {
            s.modal = modal;
            s.frame();
            for (i, h) in s.hits.iter().enumerate() {
                assert!(
                    h.rect.x >= 0.
                        && h.rect.y >= 0.
                        && h.rect.x + h.rect.w <= s.width
                        && h.rect.y + h.rect.h <= s.height,
                    "offscreen {:?}",
                    h.action
                );
                for other in s.hits.iter().skip(i + 1) {
                    let overlap = h.rect.intersect(other.rect);
                    assert!(
                        overlap.w <= 0. || overlap.h <= 0.,
                        "overlap {:?} {:?}",
                        h.action,
                        other.action
                    );
                }
            }
        }
    }
}
