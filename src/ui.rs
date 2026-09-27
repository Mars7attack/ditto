use crate::{
    app::{Action, ColorTarget, EditMode, Gesture, Hit, Modal, State},
    color_picker::{self, Control, Picker},
    render::{Draw, Rect},
};
use ditto::{
    charset,
    core::{self, Color, Tool},
    font, shaders,
};
struct Ui {
    d: Draw,
    hits: Vec<Hit>,
    focus: Option<usize>,
    theme: ditto::settings::Theme,
}
impl Ui {
    fn text(&mut self, x: f32, y: f32, s: &str, c: Color) {
        self.d.text(x, y, s, c, 1.);
    }
    fn button(&mut self, x: f32, y: f32, s: &str, a: Action, active: bool) {
        let r = Rect::new(x, y, s.chars().count() as f32 * 8. + 8., 22.);
        let focused = self.focus == Some(self.hits.len());
        self.d.rect(
            r,
            if active {
                self.theme.accent
            } else if focused {
                self.theme.focus
            } else {
                self.theme.panel
            },
            1.,
        );
        self.text(
            x + 4.,
            y + 3.,
            s,
            if active {
                self.theme.active_text
            } else {
                self.theme.text
            },
        );
        if focused {
            self.d.border(r, self.theme.accent);
        }
        self.hits.push(Hit {
            rect: r,
            action: a,
            label: s.into(),
        });
    }
    fn frame(&mut self, r: Rect, label: &str) {
        let cols = (r.w / 8.).floor() as usize;
        let rows = (r.h / 16.).floor() as usize;
        if cols < 2 || rows < 2 {
            return;
        }
        let top = format!("┌{}┐", "─".repeat(cols - 2));
        let bottom = format!("└{}┘", "─".repeat(cols - 2));
        self.text(r.x, r.y, &top, self.theme.border);
        self.text(
            r.x,
            r.y + (rows - 1) as f32 * 16.,
            &bottom,
            self.theme.border,
        );
        for y in 1..rows - 1 {
            self.text(r.x, r.y + y as f32 * 16., "│", self.theme.border);
            self.text(
                r.x + (cols - 1) as f32 * 8.,
                r.y + y as f32 * 16.,
                "│",
                self.theme.border,
            );
        }
        if !label.is_empty() {
            let t = format!(" {label} ");
            self.d.rect(
                Rect::new(r.x + 16., r.y, t.chars().count() as f32 * 8., 16.),
                self.theme.background,
                1.,
            );
            self.text(r.x + 16., r.y, &t, self.theme.muted);
        }
    }
    fn field(&mut self, x: f32, y: f32, label: &str, value: &str, index: usize, selected: bool) {
        self.text(x, y, label, self.theme.muted);
        let r = Rect::new(x, y + 22., 128., 26.);
        self.d.rect(
            r,
            if selected {
                self.theme.focus
            } else {
                self.theme.panel
            },
            1.,
        );
        self.d.border(
            r,
            if selected {
                self.theme.accent
            } else {
                self.theme.border
            },
        );
        self.text(x + 8., y + 27., &format!("{value}_"), self.theme.text);
        self.hits.push(Hit {
            rect: r,
            action: Action::Input(index),
            label: label.into(),
        });
    }
}
pub fn build(s: &State) -> (Draw, Vec<Hit>) {
    let mut u = Ui {
        d: Draw::new(s.width, s.height),
        hits: Vec::new(),
        focus: s.focus,
        theme: s.theme(),
    };
    u.d.rect(Rect::new(0., 0., s.width, s.height), u.theme.background, 1.);
    u.text(24., 16., "d i t t o _", u.theme.accent);
    let name = s.title();
    u.text(
        200.,
        16.,
        &truncate(&name, ((s.width - 520.) / 8.) as usize),
        u.theme.text,
    );
    u.button(s.width - 296., 12., "[Guides F8]", Action::Guides, false);
    u.button(s.width - 160., 12., "[Réglages]", Action::Settings, false);
    let menu = [
        ("[Nouveau]", Action::New),
        ("[Ouvrir]", Action::Open),
        ("[Enregistrer]", Action::Save),
        ("[Texte]", Action::CopyText),
        ("[PNG]", Action::Export),
        ("[Annuler]", Action::Undo),
        ("[Rétablir]", Action::Redo),
        ("[Aide F1]", Action::Help),
    ];
    let mut x = 20.;
    for (label, a) in menu {
        u.button(x, 48., label, a, false);
        x += label.chars().count() as f32 * 8. + 16.;
    }
    u.button(
        x,
        48.,
        if s.edit_mode == EditMode::Keyboard {
            "[Mode : clavier]"
        } else {
            "[Mode : souris]"
        },
        Action::ToggleEditMode,
        s.edit_mode == EditMode::Keyboard,
    );
    u.button(
        x + 168.,
        48.,
        "[Shaders F7]",
        Action::Shaders,
        s.editor.document.shaders.active(),
    );
    let compact = s.height < 800.;
    if s.edit_mode == EditMode::Mouse {
        let tools = [
            (Tool::Pencil, "B Crayon"),
            (Tool::Eraser, "G Gomme"),
            (Tool::Line, "L Ligne"),
            (Tool::Rectangle, "R Rectangle"),
            (Tool::Fill, "F Remplir"),
            (Tool::Pick, "I Pipette"),
            (Tool::Text, "T Texte"),
            (Tool::Select, "M Sélection"),
            (Tool::Recolor, "C Recolorer"),
            (Tool::Guide, "D Guides"),
        ];
        u.frame(Rect::new(16., 80., 280., 144.), "OUTILS");
        for (i, (t, label)) in tools.iter().enumerate() {
            u.button(
                28. + (i % 2) as f32 * 132.,
                98. + (i / 2) as f32 * 22.,
                label,
                Action::Tool(*t),
                s.tool == *t || (*t == Tool::Guide && s.tool == Tool::GuideErase),
            );
        }
        if s.tool == Tool::Recolor {
            u.text(
                24.,
                223.,
                &format!("Brosse {}", s.recolor_radius * 2 + 1),
                u.theme.muted,
            );
            u.button(104., 220., "[-]", Action::RecolorSize(-1), false);
            u.button(144., 220., "[+]", Action::RecolorSize(1), false);
        } else {
            u.button(
                24.,
                220.,
                if s.filled { "[x] Plein" } else { "[ ] Plein" },
                Action::Filled,
                s.filled,
            );
        }
        glyph_library(&mut u, s, compact);
    } else {
        u.frame(Rect::new(16., 88., 280., 128.), "ÉDITION CLAVIER");
        for (i, line) in [
            "Touche : insérer et avancer",
            "Flèches : déplacer le curseur",
            "Entrée : nouvelle ligne",
            "Maj+flèches : sélection",
        ]
        .iter()
        .enumerate()
        {
            u.text(
                28.,
                112. + i as f32 * 23.,
                line,
                if i == 0 {
                    u.theme.accent
                } else {
                    u.theme.muted
                },
            );
        }
        u.text(24., 224., "Espace : vide", u.theme.muted);
        keyboard_library(&mut u, s, compact);
    }
    u.button(208., 220., "[Taille]", Action::Resize, false);
    let py = if compact { 518. } else { 646. };
    u.frame(Rect::new(16., py, 280., 144.), "PINCEAU");
    u.button(
        28.,
        py + 22.,
        &format!("FG {}", hex(s.brush.cell.fg)),
        Action::Foreground,
        false,
    );
    let recoloring = s.edit_mode == EditMode::Mouse && s.tool == Tool::Recolor;
    if !recoloring {
        u.button(
            160.,
            py + 22.,
            &format!(
                "BG {}",
                s.brush.cell.bg.map(hex).unwrap_or_else(|| "vide".into())
            ),
            Action::Background,
            false,
        );
    } else {
        u.text(160., py + 25., "Recoloration", u.theme.muted);
    }
    for (i, c) in s.editor.document.palette.iter().take(32).enumerate() {
        let r = Rect::new(
            28. + (i % 16) as f32 * 16.,
            py + 52. + (i / 16) as f32 * 18.,
            14.,
            14.,
        );
        u.d.rect(r, *c, 1.);
        if *c == s.brush.cell.fg || s.focus == Some(u.hits.len()) {
            u.d.border(r, u.theme.text);
        }
        u.hits.push(Hit {
            rect: r,
            action: Action::Palette(i),
            label: format!("Couleur {}", hex(*c)),
        });
    }
    let yy = py + 91.;
    if recoloring {
        u.text(28., yy + 3., "Glyphes et fonds conservés", u.theme.muted);
        u.button(
            28.,
            py + 115.,
            "[Ajouter à la palette]",
            Action::AddColor,
            false,
        );
    } else {
        if s.edit_mode == EditMode::Keyboard {
            u.text(28., yy + 3., "Glyphe", u.theme.muted);
        } else {
            u.button(
                28.,
                yy,
                if s.brush.glyph { "[x]G" } else { "[ ]G" },
                Action::Mask(0),
                false,
            );
        }
        u.button(
            88.,
            yy,
            if s.brush.fg { "[×]FG" } else { "[ ]FG" },
            Action::Mask(1),
            false,
        );
        u.button(
            160.,
            yy,
            if s.brush.bg { "[×]BG" } else { "[ ]BG" },
            Action::Mask(2),
            false,
        );
        u.button(228., yy, "[+]", Action::AddColor, false);
        u.button(
            28.,
            py + 115.,
            "[Fond vide]",
            Action::ClearBackground,
            false,
        );
    }
    // Drawing area has its own coordinate system and clip.
    u.frame(
        Rect::new(296., 88., s.width - 304., s.height - 264.),
        "DESSIN / 01",
    );
    u.d.clip = s.canvas;
    u.d.rect(s.canvas, u.theme.canvas, 1.);
    let doc = &s.editor.document;
    let board = Rect::new(
        s.origin.0,
        s.origin.1,
        doc.width as f32 * s.cell_width(),
        doc.height as f32 * s.cell_size,
    );
    u.d.clip = s.canvas.intersect(board);
    if let Some(r) = &doc.reference
        && r.visible
    {
        u.d.image(
            Rect::new(
                s.origin.0 + r.x * s.cell_width(),
                s.origin.1 + r.y * s.cell_size,
                r.asset.width as f32 * r.scale * s.cell_width(),
                r.asset.height as f32 * r.scale * r.pixel_aspect * s.cell_size,
            ),
            r.opacity,
        );
    }
    if !doc.guides.above_characters {
        draw_guides(&mut u.d, s);
    }
    let alpha = if s.trace { 0.4 } else { 1. };
    let x0 = (((s.canvas.x - s.origin.0) / s.cell_width()).floor() as i32).max(0);
    let y0 = (((s.canvas.y - s.origin.1) / s.cell_size).floor() as i32).max(0);
    let x1 = (((s.canvas.x + s.canvas.w - s.origin.0) / s.cell_width()).ceil() as i32)
        .min(doc.width as i32);
    let y1 = (((s.canvas.y + s.canvas.h - s.origin.1) / s.cell_size).ceil() as i32)
        .min(doc.height as i32);
    if doc.shaders.active() && s.shader_error.is_none() {
        u.d.artwork(board, alpha, false);
    } else {
        for y in y0..y1 {
            for x in x0..x1 {
                let c = doc.get(x, y).unwrap();
                cell(&mut u.d, s, x, y, c, alpha);
            }
        }
    }
    if s.grid && s.cell_size >= 4. {
        for x in x0..=x1 {
            u.d.rect(
                Rect::new(
                    s.origin.0 + x as f32 * s.cell_width(),
                    s.canvas.y,
                    0.5,
                    s.canvas.h,
                ),
                u.theme.grid,
                0.6,
            );
        }
        for y in y0..=y1 {
            u.d.rect(
                Rect::new(
                    s.canvas.x,
                    s.origin.1 + y as f32 * s.cell_size,
                    s.canvas.w,
                    0.5,
                ),
                u.theme.grid,
                0.6,
            );
        }
    }
    if let Some(f) = &s.floating {
        for y in 0..f.block.height {
            for x in 0..f.block.width {
                cell(
                    &mut u.d,
                    s,
                    f.position.0 + x as i32,
                    f.position.1 + y as i32,
                    f.block.cells[(y * f.block.width + x) as usize],
                    0.85,
                );
            }
        }
    }
    if let Some(Gesture::Shape { origin }) = s.gesture {
        for (x, y) in core::shape(s.tool, origin, s.cursor, s.filled) {
            if s.editor.selection.is_none_or(|r| r.contains(x, y))
                && let Some(c) = doc.get(x, y)
            {
                cell(&mut u.d, s, x, y, s.brush.apply(c), 0.85);
            }
        }
    }
    if let Some(Gesture::Move { rect, origin }) = s.gesture {
        let block = core::Block::copy(doc, rect);
        let p = (
            rect.x + s.cursor.0 - origin.0,
            rect.y + s.cursor.1 - origin.1,
        );
        for y in 0..block.height {
            for x in 0..block.width {
                cell(
                    &mut u.d,
                    s,
                    p.0 + x as i32,
                    p.1 + y as i32,
                    block.cells[(y * block.width + x) as usize],
                    0.8,
                );
            }
        }
    }
    if doc.guides.above_characters {
        draw_guides(&mut u.d, s);
    }
    if s.modal.is_none() && s.edit_mode == EditMode::Mouse && s.cell_at(s.mouse).is_some() {
        if s.tool == Tool::Recolor {
            let r = s.recolor_radius;
            for y in s.cursor.1 - r..=s.cursor.1 + r {
                for x in s.cursor.0 - r..=s.cursor.0 + r {
                    if (x - s.cursor.0).pow(2) + (y - s.cursor.1).pow(2) <= r * r {
                        u.d.border(
                            Rect::new(
                                s.origin.0 + x as f32 * s.cell_width(),
                                s.origin.1 + y as f32 * s.cell_size,
                                s.cell_width(),
                                s.cell_size,
                            ),
                            u.theme.accent,
                        );
                    }
                }
            }
        } else if matches!(s.tool, Tool::Guide | Tool::GuideErase) {
            let radius = if s.tool == Tool::GuideErase {
                s.guide_width.max(1.)
            } else {
                s.guide_width / 2.
            };
            let points: Vec<_> = (0..=32)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 32.;
                    [
                        s.mouse.0 + a.cos() * radius * s.cell_width(),
                        s.mouse.1 + a.sin() * radius * s.cell_width(),
                    ]
                })
                .collect();
            u.d.polyline(&points, 1., u.theme.accent, 1.);
        }
    }
    if let Some(r) = s.editor.selection {
        u.d.border(
            Rect::new(
                s.origin.0 + r.x as f32 * s.cell_width(),
                s.origin.1 + r.y as f32 * s.cell_size,
                r.w as f32 * s.cell_width(),
                r.h as f32 * s.cell_size,
            ),
            u.theme.selection,
        );
    }
    if s.modal.is_none()
        && (s.edit_mode == EditMode::Keyboard || !matches!(s.tool, Tool::Guide | Tool::GuideErase))
    {
        u.d.border(
            Rect::new(
                s.origin.0
                    + (s.cursor.0 as f32 * s.cell_width())
                        .min(doc.width as f32 * s.cell_width() - 2.),
                s.origin.1 + s.cursor.1 as f32 * s.cell_size,
                if s.cursor.0 == doc.width as i32 {
                    2.
                } else {
                    s.cell_width()
                },
                s.cell_size,
            ),
            u.theme.accent,
        );
    }
    u.d.clip = Rect::new(0., 0., s.width, s.height);
    u.d.border(board.intersect(s.canvas), u.theme.border);
    let cy = s.height - 174.;
    u.text(
        312.,
        cy,
        &format!(
            "{:03}:{:03}  {}×{}  {:.0}×{:.0}px",
            s.cursor.0,
            s.cursor.1,
            doc.width,
            doc.height,
            s.cell_width(),
            s.cell_size
        ),
        u.theme.muted,
    );
    u.button(s.width - 288., cy - 3., "[-]", Action::Zoom(false), false);
    u.button(s.width - 248., cy - 3., "[+]", Action::Zoom(true), false);
    u.button(s.width - 208., cy - 3., "[Ajuster]", Action::Fit, false);
    u.button(
        s.width - 112.,
        cy - 3.,
        if s.grid { "[×]Grille" } else { "[ ]Grille" },
        Action::Grid,
        false,
    );
    // Reference settings are text commands, not graphical sliders.
    let ry = s.height - 142.;
    u.frame(Rect::new(296., ry, s.width - 304., 96.), "RÉFÉRENCE");
    u.button(
        312.,
        ry + 22.,
        if doc.reference.is_some() {
            "[Remplacer]"
        } else {
            "[Importer image]"
        },
        Action::ImportReference,
        false,
    );
    if let Some(r) = &doc.reference {
        u.button(
            440.,
            ry + 22.,
            if r.visible { "[×]Voir" } else { "[ ]Voir" },
            Action::RefVisible,
            false,
        );
        u.button(
            528.,
            ry + 22.,
            if r.locked { "[×]Verrou" } else { "[ ]Verrou" },
            Action::RefLock,
            false,
        );
        u.text(
            640.,
            ry + 25.,
            &format!("Opacité {:3.0}%", r.opacity * 100.),
            u.theme.muted,
        );
        u.button(768., ry + 22., "[-]", Action::RefOpacity(-5), false);
        u.button(808., ry + 22., "[+]", Action::RefOpacity(5), false);
        u.button(
            864.,
            ry + 22.,
            if s.trace { "[×]Tracé" } else { "[ ]Tracé" },
            Action::Trace,
            false,
        );
        u.button(976., ry + 22., "[Retirer]", Action::RefRemove, false);
        u.button(312., ry + 52., "[Contenir]", Action::RefFit(false), false);
        u.button(416., ry + 52., "[Remplir]", Action::RefFit(true), false);
        u.button(
            520.,
            ry + 52.,
            "[Déplacer]",
            Action::RefTransform,
            s.ref_transform,
        );
        u.button(
            632.,
            ry + 52.,
            "[Échelle -]",
            Action::RefScale(false),
            false,
        );
        u.button(752., ry + 52., "[Échelle +]", Action::RefScale(true), false);
        u.text(
            880.,
            ry + 56.,
            &format!("x{:.1} y{:.1}", r.x, r.y),
            u.theme.muted,
        );
    } else {
        u.text(
            480.,
            ry + 26.,
            "PNG / JPEG · incorporée au projet",
            u.theme.muted,
        );
        u.text(
            312.,
            ry + 56.,
            "La référence ne sera pas incluse dans les exports.",
            u.theme.muted,
        );
    }
    if s.editor.selection.is_some() {
        let yy = 72.;
        let mut x = 312.;
        for (label, a) in [
            ("[Copier]", Action::Copy),
            ("[Couper]", Action::Cut),
            ("[Coller]", Action::Paste),
            ("[Déplacer]", Action::Move),
            ("[Effacer]", Action::EraseSelection),
            ("[Désélect.]", Action::Deselect),
        ] {
            u.button(x, yy, label, a, false);
            x += label.len() as f32 * 8. + 12.;
        }
    }
    u.d.clip = Rect::new(0., 0., s.width, s.height);
    u.d.rect(
        Rect::new(16., s.height - 34., s.width - 32., 1.),
        u.theme.border,
        1.,
    );
    u.text(
        24.,
        s.height - 25.,
        &truncate(&s.status, ((s.width - 48.) / 8.) as usize),
        if s.status.contains("impossible") || s.status.contains("hors") {
            u.theme.accent
        } else {
            u.theme.muted
        },
    );
    if !s.ime.is_empty() {
        u.d.rect(
            Rect::new(320., 110., s.ime.chars().count() as f32 * 8. + 16., 24.),
            u.theme.panel,
            1.,
        );
        u.text(328., 114., &s.ime, u.theme.accent);
    }
    if let Some(modal) = &s.modal {
        if matches!(modal, Modal::Shaders) {
            shader_window(&mut u, s);
            return (u.d, u.hits);
        }
        u.hits.clear();
        u.d.rect(
            Rect::new(0., 76., s.width, s.height - 116.),
            u.theme.background,
            0.96,
        );
        let r = Rect::new(((s.width - 624.) / 16.).floor() * 8., 144., 624., 512.);
        u.d.rect(r, u.theme.panel, 1.);
        u.frame(r, "DITTO");
        let x = r.x + 32.;
        let y = r.y + 40.;
        match modal {
            Modal::Settings => settings_window(&mut u, s, x, y),
            Modal::Guides => guides_window(&mut u, s, x, y),
            Modal::Shaders => unreachable!(),
            Modal::New { width, height } | Modal::Resize { width, height } => {
                let new = matches!(modal, Modal::New { .. });
                u.text(
                    x,
                    y,
                    if new {
                        "NOUVEAU DOCUMENT"
                    } else {
                        "REDIMENSIONNER LE DESSIN"
                    },
                    u.theme.accent,
                );
                u.text(
                    x,
                    y + 32.,
                    if new {
                        "Choisir une grille, puis dessiner."
                    } else {
                        "Aperçu du recadrage : origine en haut à gauche."
                    },
                    u.theme.muted,
                );
                if new {
                    u.button(
                        x,
                        y + 80.,
                        "[1] Sprite       32 × 32",
                        Action::Preset(32, 32),
                        width == "32" && height == "32",
                    );
                    u.button(
                        x,
                        y + 112.,
                        "[2] Illustration 80 × 50",
                        Action::Preset(80, 50),
                        width == "80" && height == "50",
                    );
                    u.button(
                        x,
                        y + 144.,
                        "[3] Carte       100 × 60",
                        Action::Preset(100, 60),
                        width == "100" && height == "60",
                    );
                } else {
                    let w = width.parse::<u32>().unwrap_or(1).max(1);
                    let h = height.parse::<u32>().unwrap_or(1).max(1);
                    let k = (480. / (doc.width.max(w) as f32 * ditto::typeface::ASPECT))
                        .min(100. / doc.height.max(h) as f32);
                    u.d.rect(
                        Rect::new(
                            x,
                            y + 78.,
                            doc.width as f32 * k * ditto::typeface::ASPECT,
                            doc.height as f32 * k,
                        ),
                        u.theme.focus,
                        1.,
                    );
                    u.d.border(
                        Rect::new(
                            x,
                            y + 78.,
                            w as f32 * k * ditto::typeface::ASPECT,
                            h as f32 * k,
                        ),
                        u.theme.accent,
                    );
                }
                u.field(x, y + 206., "COLONNES", width, 0, s.input == 0);
                u.field(x + 200., y + 206., "LIGNES", height, 1, s.input == 1);
                u.text(
                    x,
                    y + 278.,
                    "1 à 512 par côté · SF Mono + blocs étendus",
                    u.theme.muted,
                );
                u.text(
                    x,
                    y + 306.,
                    "Entrée : confirmer / Tab : naviguer / Échap : retour",
                    u.theme.muted,
                );
                u.button(x, y + 372., "[Retour]", Action::Cancel, false);
                u.button(x + 136., y + 372., "[Ouvrir projet]", Action::Open, false);
                u.button(
                    x + 368.,
                    y + 372.,
                    if new { "[Créer →]" } else { "[Appliquer]" },
                    Action::Submit,
                    true,
                );
            }
            Modal::Color { value, target } => {
                u.text(
                    x,
                    y,
                    match target {
                        ColorTarget::Theme(_) => "COULEUR DE L’INTERFACE",
                        ColorTarget::Guide => "COULEUR DES PROCHAINS GUIDES",
                        ColorTarget::Foreground => "COULEUR DU GLYPHE",
                        ColorTarget::Background => "COULEUR DU FOND",
                        ColorTarget::Palette(_) => "MODIFIER LA PALETTE",
                        ColorTarget::Shader(..) => "ENCRE DU SHADER DUOTONE",
                    },
                    u.theme.accent,
                );
                color_window(&mut u, s, value, x, y);
            }
            Modal::Loss { .. } => {
                u.text(x, y, "MODIFICATIONS NON ENREGISTRÉES", u.theme.accent);
                u.text(
                    x,
                    y + 64.,
                    "Enregistrer le document avant de continuer ?",
                    u.theme.text,
                );
                u.button(x, y + 240., "[Annuler]", Action::Cancel, false);
                u.button(x + 128., y + 240., "[Abandonner]", Action::Discard, false);
                u.button(
                    x + 320.,
                    y + 240.,
                    "[Enregistrer]",
                    Action::SaveContinue,
                    true,
                );
            }
            Modal::Export { scale, opaque } => {
                u.text(x, y, "EXPORT PNG", u.theme.accent);
                u.text(x, y + 48., "Échelle entière", u.theme.muted);
                for (i, k) in [1, 2, 4].iter().enumerate() {
                    u.button(
                        x + i as f32 * 96.,
                        y + 80.,
                        &format!("[×{k}]"),
                        Action::ExportScale(*k),
                        scale == k,
                    );
                }
                u.button(
                    x,
                    y + 136.,
                    if *opaque {
                        "[×] Fond opaque"
                    } else {
                        "[ ] Fond opaque"
                    },
                    Action::ExportBackground,
                    false,
                );
                u.text(
                    x,
                    y + 192.,
                    &format!(
                        "{} × {} pixels",
                        doc.width * ditto::typeface::CELL_WIDTH * scale,
                        doc.height * ditto::typeface::CELL_HEIGHT * scale
                    ),
                    u.theme.text,
                );
                u.text(
                    x,
                    y + 224.,
                    "Dessin seul. Sans référence ni aides d’édition.",
                    u.theme.muted,
                );
                u.text(
                    x,
                    y + 256.,
                    if doc.shaders.active() {
                        "Shaders actifs inclus · transparence conservée."
                    } else {
                        "Shaders désactivés ou pile vide : rendu original."
                    },
                    u.theme.accent,
                );
                u.button(x, y + 332., "[Annuler]", Action::Cancel, false);
                u.button(x + 328., y + 332., "[Exporter]", Action::Submit, true);
            }
            Modal::Text { content } => {
                u.text(x, y, "TEXTE À COPIER / UTF-8", u.theme.accent);
                u.text(
                    x,
                    y + 32.,
                    if s.editor.selection.is_some() {
                        "Portée : sélection · sans couleurs"
                    } else {
                        "Portée : grille entière · sans couleurs"
                    },
                    u.theme.muted,
                );
                let preview = Rect::new(x, y + 64., 544., 240.);
                u.d.rect(preview, u.theme.canvas, 1.);
                u.d.clip = preview;
                for (i, line) in content.lines().take(15).enumerate() {
                    u.text(
                        x + 8.,
                        y + 68. + i as f32 * 16.,
                        &truncate(line, 65),
                        u.theme.text,
                    );
                }
                u.d.clip = Rect::new(0., 0., s.width, s.height);
                u.text(
                    x,
                    y + 320.,
                    "L’aperçu est limité ; la copie contient toute la portée.",
                    u.theme.muted,
                );
                u.button(x, y + 368., "[Retour]", Action::Cancel, false);
                u.button(
                    x + 280.,
                    y + 368.,
                    "[Copier le texte]",
                    Action::CopyExport,
                    true,
                );
            }
            Modal::Help => {
                u.text(
                    x,
                    y,
                    &format!("DITTO {} ({}) / AIDE", ditto::VERSION, ditto::BUILD_ID),
                    u.theme.accent,
                );
                let lines = [
                    "Cmd/Ctrl Shift M : basculer souris / clavier",
                    "Clavier : touches mappées ; Entrée : ligne suivante",
                    "Souris : Entrée applique une cellule ou une forme",
                    "Espace glisser / Alt flèches : vue · molette : zoom",
                    "Cmd/Ctrl S : sauver · O ouvrir · N nouveau",
                    "Cmd/Ctrl Z : annuler · Shift Z : rétablir",
                    "Cmd/Ctrl C/X/V : cellules · Shift C : texte",
                    "Cmd/Ctrl A : tout sélectionner · Suppr : effacer",
                    "F3 glyphes/charset · F4 couleurs · F6 canevas",
                    "Palette : clic droit = fond · Shift clic = modifier",
                    "Référence : déverrouiller puis déplacer / flèches",
                    "Cmd/Ctrl ↑/↓ : banque du charset · Échap : annuler",
                    "+ / - : zoom · 0 : ajuster · F2 : taille",
                    "C recolorer · D guides · F8 calque · Cmd/Ctrl , réglages",
                ];
                for (i, l) in lines.iter().enumerate() {
                    u.text(x, y + 40. + i as f32 * 24., l, u.theme.text);
                }
                u.button(x, y + 408., "[Fermer]", Action::Cancel, true);
            }
            Modal::Charsets => {
                u.text(x, y, "CHOISIR UN CHARSET", u.theme.accent);
                u.text(
                    x,
                    y + 28.,
                    "Lettres + rangée 0–9 / sans Maj ni Option",
                    u.theme.muted,
                );
                for (i, set) in charset::ALL.iter().enumerate() {
                    u.button(
                        x,
                        y + 72. + i as f32 * 44.,
                        &format!(
                            "[{}] {:26} {:3} glyphes",
                            i + 1,
                            set.name,
                            set.targets.len()
                        ),
                        Action::SetCharset(i),
                        s.charset == i,
                    );
                }
                u.text(
                    x,
                    y + 348.,
                    "Braille : 4 hauteurs + motifs espacés regroupés.",
                    u.theme.muted,
                );
                u.button(x, y + 400., "[Retour]", Action::Cancel, false);
            }
            Modal::Recovery => {
                u.text(x, y, "BROUILLON DE RÉCUPÉRATION", u.theme.accent);
                u.text(
                    x,
                    y + 64.,
                    "Un document non enregistré a été retrouvé.",
                    u.theme.text,
                );
                u.text(
                    x,
                    y + 96.,
                    "Le restaurer ouvre une copie à enregistrer.",
                    u.theme.muted,
                );
                u.button(
                    x,
                    y + 256.,
                    "[Ignorer et supprimer]",
                    Action::ForgetRecovery,
                    false,
                );
                u.button(x + 320., y + 256., "[Restaurer]", Action::Recover, true);
            }
        }
    }
    (u.d, u.hits)
}
fn settings_window(u: &mut Ui, s: &State, x: f32, y: f32) {
    u.text(x, y, "RÉGLAGES / APPARENCE", u.theme.accent);
    u.text(
        x,
        y + 28.,
        "Préférences locales · aperçu immédiat",
        u.theme.muted,
    );
    for (i, label) in ditto::settings::Theme::PRESETS.iter().enumerate() {
        u.button(
            x + i as f32 * 120.,
            y + 62.,
            &format!("[{label}]"),
            Action::ThemePreset(i),
            s.theme() == ditto::settings::Theme::preset(i),
        );
    }
    for (i, token) in ditto::settings::Token::ALL.iter().enumerate() {
        let bx = x + (i / 6) as f32 * 280.;
        let by = y + 106. + (i % 6) as f32 * 38.;
        u.d.rect(
            Rect::new(bx, by + 3., 18., 16.),
            token.color(&s.theme()),
            1.,
        );
        u.d.border(Rect::new(bx, by + 3., 18., 16.), u.theme.border);
        u.button(
            bx + 26.,
            by,
            token.label(),
            Action::ThemeColor(*token),
            false,
        );
    }
    u.text(
        x,
        y + 348.,
        if s.theme().low_contrast() {
            "Contraste faible : ajuster texte, fond ou accent."
        } else {
            "Les couleurs du dessin et des exports restent inchangées."
        },
        u.theme.muted,
    );
    u.button(x, y + 400., "[Annuler]", Action::Cancel, false);
    u.button(
        x + 152.,
        y + 400.,
        "[Défaut]",
        Action::ThemePreset(0),
        false,
    );
    u.button(x + 360., y + 400., "[Enregistrer]", Action::Submit, true);
}
fn draw_guides(d: &mut Draw, s: &State) {
    let doc = &s.editor.document;
    if doc.guides.visible {
        for stroke in doc.guides.strokes.iter() {
            let points: Vec<_> = stroke
                .points
                .iter()
                .map(|p| {
                    [
                        s.origin.0 + p[0] * s.cell_width(),
                        s.origin.1 + p[1] * s.cell_size,
                    ]
                })
                .collect();
            d.polyline(
                &points,
                stroke.width * s.cell_width(),
                stroke.color,
                doc.guides.opacity,
            );
        }
    }
}
fn guides_window(u: &mut Ui, s: &State, x: f32, y: f32) {
    let layer = &s.editor.document.guides;
    u.text(x, y, "GUIDES / CALQUE DE PLACEMENT", u.theme.accent);
    u.text(
        x,
        y + 32.,
        "Référence > guides > caractères, ou guides au premier plan.",
        u.theme.muted,
    );
    u.text(
        x,
        y + 56.,
        "Enregistrés avec le projet ; exclus des PNG et du texte.",
        u.theme.muted,
    );
    u.button(
        x,
        y + 100.,
        if layer.visible {
            "[x] Visible"
        } else {
            "[ ] Visible"
        },
        Action::GuideVisible,
        layer.visible,
    );
    u.text(
        x + 168.,
        y + 103.,
        &format!("{} traits", layer.strokes.len()),
        u.theme.muted,
    );
    u.button(
        x + 312.,
        y + 100.,
        if layer.above_characters {
            "[x] Devant les caractères"
        } else {
            "[ ] Devant les caractères"
        },
        Action::GuideAbove,
        layer.above_characters,
    );
    u.text(
        x,
        y + 148.,
        &format!("Opacité {:3.0}%", layer.opacity * 100.),
        u.theme.text,
    );
    u.button(x + 216., y + 145., "[-]", Action::GuideOpacity(-1), false);
    u.button(x + 260., y + 145., "[+]", Action::GuideOpacity(1), false);
    u.text(
        x,
        y + 194.,
        &format!("Épaisseur {:.1} cellule", s.guide_width),
        u.theme.text,
    );
    u.button(x + 216., y + 191., "[-]", Action::GuideWidth(-1), false);
    u.button(x + 260., y + 191., "[+]", Action::GuideWidth(1), false);
    u.button(
        x,
        y + 239.,
        &format!("Couleur {}", hex(s.guide_color)),
        Action::GuideColor,
        false,
    );
    u.d.rect(Rect::new(x + 200., y + 242., 32., 16.), s.guide_color, 1.);
    u.button(
        x + 256.,
        y + 239.,
        "[Recolorer tous les traits]",
        Action::GuideRecolorAll,
        false,
    );
    u.button(
        x,
        y + 291.,
        "[Tracer]",
        Action::Tool(Tool::Guide),
        s.tool == Tool::Guide,
    );
    u.button(
        x + 120.,
        y + 291.,
        "[Gommer des traits]",
        Action::Tool(Tool::GuideErase),
        s.tool == Tool::GuideErase,
    );
    u.text(
        x,
        y + 332.,
        "Gomme : retire le trait touché · Cmd/Ctrl Z : annuler",
        u.theme.muted,
    );
    u.button(x, y + 400., "[Tout effacer]", Action::GuideClear, false);
    u.button(x + 376., y + 400., "[Fermer]", Action::Cancel, true);
}
fn shader_window(u: &mut Ui, s: &State) {
    u.hits.clear();
    u.d.rect(
        Rect::new(0., 76., s.width, s.height - 116.),
        u.theme.background,
        0.98,
    );
    let r = Rect::new(24., 80., s.width - 48., s.height - 128.);
    u.d.rect(r, u.theme.panel, 1.);
    u.frame(r, "SHADERS / ATELIER DE RENDU");
    let x = r.x + 20.;
    let right = r.x + r.w - 460.;
    let left_width = right - x - 28.;
    let stack = &s.editor.document.shaders;
    u.text(x, 108., "DESSIN > EFFETS > RENDU", u.theme.accent);
    u.button(
        right,
        104.,
        if stack.enabled {
            "[x] Effets"
        } else {
            "[ ] Effets"
        },
        Action::ShaderEnabled,
        stack.enabled,
    );
    u.button(right + 144., 104., "[Exporter PNG]", Action::Export, false);
    u.button(right + 304., 104., "[Fermer]", Action::Cancel, false);
    u.text(x, 145., "Looks", u.theme.muted);
    let mut bx = x + 56.;
    for (i, label) in shaders::PRESETS.iter().enumerate() {
        u.button(
            bx,
            141.,
            &format!("[{label}]"),
            Action::ShaderPreset(i),
            false,
        );
        bx += (label.chars().count() as f32 + 2.) * 8. + 16.;
    }
    u.button(right, 141., "[Charger look]", Action::ShaderLoad, false);
    u.button(
        right + 144.,
        141.,
        "[Sauver look]",
        Action::ShaderSave,
        false,
    );
    u.text(
        x,
        180.,
        if s.shader_before {
            "APERÇU / AVANT"
        } else {
            "APERÇU / APRÈS"
        },
        u.theme.muted,
    );
    u.button(
        x + left_width - 184.,
        176.,
        "[Comparer avant/après]",
        Action::ShaderBefore,
        s.shader_before,
    );
    u.button(x, 208., "[-]", Action::ShaderPreviewZoom(false), false);
    u.button(x + 40., 208., "[+]", Action::ShaderPreviewZoom(true), false);
    u.text(
        x + 88.,
        212.,
        &format!("{:.0}%", s.shader_preview_scale() * 100.),
        u.theme.muted,
    );
    u.button(
        x + 164.,
        208.,
        "[Ajuster]",
        Action::ShaderPreviewFit,
        s.shader_preview_zoom.is_none() && s.shader_preview_pan == (0., 0.),
    );
    u.button(x + 252., 208., "[100%]", Action::ShaderPreviewActual, false);
    u.text(x + 332., 212., "Molette : zoom · glisser", u.theme.muted);
    let preview = s.shader_preview_rect();
    // Checkerboard shows the exported alpha; the reference and all editing guides stay outside.
    u.d.clip = preview;
    for row in 0..(preview.h / 16.).ceil() as i32 {
        for col in 0..(preview.w / 16.).ceil() as i32 {
            u.d.rect(
                Rect::new(
                    preview.x + col as f32 * 16.,
                    preview.y + row as f32 * 16.,
                    16.,
                    16.,
                ),
                if (row + col) % 2 == 0 {
                    u.theme.canvas
                } else {
                    u.theme.checker
                },
                1.,
            );
        }
    }
    let doc = &s.editor.document;
    if s.shader_error.is_none() {
        u.d.artwork(s.shader_preview_board(), 1., s.shader_before);
    }
    u.d.clip = Rect::new(0., 0., s.width, s.height);
    u.d.border(preview, u.theme.border);
    if s.shader_error.is_some() {
        u.text(
            x + 16.,
            preview.y + preview.h / 2.,
            "Aperçu indisponible : voir le message en bas.",
            u.theme.accent,
        );
    }
    if doc.cells.iter().all(|c| c.glyph == 32 && c.bg.is_none()) {
        u.text(
            x + 16.,
            preview.y + preview.h / 2.,
            "Dessinez quelques glyphes pour voir les effets.",
            u.theme.muted,
        );
    }
    let catalog_y = preview.y + preview.h + 16.;
    u.text(x, catalog_y, "AJOUTER UN EFFET / À LA SUITE", u.theme.muted);
    for (i, kind) in shaders::KINDS.iter().enumerate() {
        u.button(
            x + (i % 3) as f32 * (left_width / 3.),
            catalog_y + 26. + (i / 3) as f32 * 28.,
            &format!("[+ {}]", kind.name()),
            Action::ShaderAdd(*kind),
            false,
        );
    }
    u.text(
        right,
        180.,
        &format!("PILE / {} sur 8 · de haut en bas", stack.layers.len()),
        u.theme.muted,
    );
    if stack.layers.is_empty() {
        u.text(right, 220., "Aucun effet pour le moment.", u.theme.text);
        u.text(
            right,
            248.,
            "Choisissez un look ou ajoutez un shader.",
            u.theme.muted,
        );
        u.text(
            right,
            288.,
            "Chaque effet reste réglable et annulable.",
            u.theme.muted,
        );
        u.text(
            right,
            316.,
            "Le dessin original est conservé.",
            u.theme.muted,
        );
    }
    let selected = s.shader_selected.min(stack.layers.len().saturating_sub(1));
    for (i, layer) in stack.layers.iter().enumerate() {
        let y = 204. + i as f32 * 26.;
        u.button(
            right,
            y,
            if layer.enabled { "[x]" } else { "[ ]" },
            Action::ShaderToggle(i),
            false,
        );
        if let Some(hit) = u.hits.last_mut() {
            hit.label = format!(
                "{} {}",
                if layer.enabled {
                    "Désactiver"
                } else {
                    "Activer"
                },
                layer.kind.name()
            );
        }
        u.button(
            right + 40.,
            y,
            &format!("{:02} {:17}", i + 1, layer.kind.name()),
            Action::ShaderSelect(i),
            i == selected,
        );
        if i > 0 {
            u.button(right + 232., y, "[↑]", Action::ShaderMove(i, -1), false);
            u.hits.last_mut().unwrap().label = format!("Monter {}", layer.kind.name());
        }
        if i + 1 < stack.layers.len() {
            u.button(right + 272., y, "[↓]", Action::ShaderMove(i, 1), false);
            u.hits.last_mut().unwrap().label = format!("Descendre {}", layer.kind.name());
        }
        u.button(right + 320., y, "[Retirer]", Action::ShaderRemove(i), false);
        u.hits.last_mut().unwrap().label = format!("Retirer {}", layer.kind.name());
    }
    if let Some(layer) = stack.layers.get(selected) {
        u.text(
            right,
            424.,
            &format!("RÉGLAGES / {}", layer.kind.name().to_uppercase()),
            u.theme.accent,
        );
        for i in 0..4 {
            let spec = layer.parameter(i);
            let value = layer.value(i);
            let y = 452. + i as f32 * 32.;
            u.text(right, y + 3., spec.name, u.theme.muted);
            u.text(
                right + 128.,
                y + 3.,
                &format!("{:6.2}", value),
                u.theme.text,
            );
            u.button(right + 192., y, "[-]", Action::ShaderAdjust(i, -1), false);
            u.hits.last_mut().unwrap().label = format!("Diminuer {} : {:.2}", spec.name, value);
            let bar = Rect::new(right + 232., y + 8., 100., 6.);
            u.d.rect(bar, u.theme.border, 1.);
            u.d.rect(
                Rect::new(
                    bar.x,
                    bar.y,
                    bar.w * ((value - spec.min) / (spec.max - spec.min)),
                    bar.h,
                ),
                u.theme.accent,
                1.,
            );
            u.button(right + 344., y, "[+]", Action::ShaderAdjust(i, 1), false);
            u.hits.last_mut().unwrap().label = format!("Augmenter {} : {:.2}", spec.name, value);
        }
        if layer.kind == shaders::Kind::Duotone {
            for i in 0..2 {
                let xx = right + i as f32 * 200.;
                u.d.rect(Rect::new(xx, 588., 16., 16.), layer.colors[i], 1.);
                u.button(
                    xx + 24.,
                    584.,
                    &format!(
                        "[{} #{}]",
                        if i == 0 { "A" } else { "B" },
                        hex(layer.colors[i])
                    ),
                    Action::ShaderColor(i),
                    false,
                );
            }
        } else {
            u.text(
                right,
                588.,
                &truncate(layer.kind.description(), 49),
                u.theme.muted,
            );
        }
    }
    let footer = r.y + r.h - 38.;
    u.button(x, footer, "[Annuler]", Action::Undo, false);
    u.button(x + 96., footer, "[Rétablir]", Action::Redo, false);
    u.text(
        x + 208.,
        footer + 3.,
        "Échap : retour au dessin",
        u.theme.muted,
    );
    u.button(right, footer, "[Vider la pile]", Action::ShaderReset, false);
    u.text(
        right + 160.,
        footer + 3.,
        "Inclus dans le projet",
        u.theme.muted,
    );
}

fn cell(d: &mut Draw, s: &State, x: i32, y: i32, c: core::Cell, alpha: f32) {
    let r = Rect::new(
        s.origin.0 + x as f32 * s.cell_width(),
        s.origin.1 + y as f32 * s.cell_size,
        s.cell_width(),
        s.cell_size,
    );
    if let Some(bg) = c.bg {
        d.rect(r, bg, alpha);
    }
    d.glyph(c.glyph, r, c.fg, alpha);
}
fn hex(c: Color) -> String {
    format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}
fn truncate(s: &str, len: usize) -> String {
    if s.chars().count() > len {
        let mut out = s.chars().take(len.saturating_sub(2)).collect::<String>();
        out.push_str("..");
        out
    } else {
        s.into()
    }
}

fn glyph_library(u: &mut Ui, s: &State, compact: bool) {
    u.frame(
        Rect::new(16., 252., 280., if compact { 256. } else { 384. }),
        "GLYPHES / ÉTENDUS",
    );
    for (i, (label, id)) in [("Tout", 0), ("Traits", 1), ("Blocs", 2), ("Récents", 3)]
        .iter()
        .enumerate()
    {
        u.button(
            24. + i as f32 * 66.,
            274.,
            label,
            Action::Category(*id),
            s.category == *id,
        );
    }
    let all = s.category_glyphs();
    let size = s.glyph_page_size();
    let pages = all.len().div_ceil(size).max(1);
    let page = s.glyph_page.min(pages - 1);
    for (i, g) in all.iter().skip(page * size).take(size).enumerate() {
        let x = 28. + (i % 16) as f32 * 16.;
        let y = 306. + (i / 16) as f32 * 18.;
        let r = Rect::new(x, y, 16., 18.);
        let selected = *g == s.brush.cell.glyph;
        let focused = s.focus == Some(u.hits.len());
        if selected {
            u.d.rect(r, u.theme.accent, 1.);
        } else if focused {
            u.d.rect(r, u.theme.border, 1.);
        }
        u.d.glyph(
            *g,
            Rect::new(x + 4., y + 1., 8., 16.),
            if selected {
                u.theme.background
            } else {
                u.theme.text
            },
            1.,
        );
        u.hits.push(Hit {
            rect: r,
            action: Action::Glyph(*g),
            label: format!(
                "Glyphe {} U+{:04X}",
                font::character(*g),
                font::character(*g) as u32
            ),
        });
    }
    let y = if compact { 474. } else { 606. };
    u.text(
        28.,
        y,
        &format!(
            "#{:03} {}",
            s.brush.cell.glyph,
            font::character(s.brush.cell.glyph)
        ),
        u.theme.accent,
    );
    if pages > 1 {
        u.button(132., y - 4., "[<]", Action::GlyphPage(-1), false);
        u.text(172., y, &format!("{}/{}", page + 1, pages), u.theme.muted);
        u.button(236., y - 4., "[>]", Action::GlyphPage(1), false);
    }
}
fn keyboard_library(u: &mut Ui, s: &State, compact: bool) {
    u.frame(
        Rect::new(16., 252., 280., if compact { 256. } else { 384. }),
        "CHARSET / TOUCHES",
    );
    let set = s.active_charset();
    u.button(
        28.,
        274.,
        &format!("[{} v]", set.name),
        Action::Charsets,
        false,
    );
    if set.banks() > 1 {
        u.button(28., 300., "[<]", Action::CharsetBank(-1), false);
        u.text(76., 304., &set.bank_label(s.charset_bank), u.theme.muted);
        u.button(236., 300., "[>]", Action::CharsetBank(1), false);
    } else {
        u.text(
            28.,
            304.,
            &format!(
                "{} touches / {} glyphes",
                charset::KEYS.len(),
                set.targets.len()
            ),
            u.theme.muted,
        );
    }
    let per = s.mapping_page_size();
    let pages = charset::KEYS.len().div_ceil(per);
    let page = s.mapping_page.min(pages - 1);
    for (i, (slot, key)) in charset::KEYS
        .iter()
        .enumerate()
        .skip(page * per)
        .take(per.min(set.bank_len(s.charset_bank).saturating_sub(page * per)))
        .enumerate()
    {
        let x = 28. + (i % 8) as f32 * 32.;
        let y = 332. + (i / 8) as f32 * 24.;
        let r = Rect::new(x, y, 30., 22.);
        let target = set.at(s.charset_bank, slot);
        let active = s.last_key == Some(slot);
        let focus = s.focus == Some(u.hits.len());
        u.d.rect(r, if active { u.theme.focus } else { u.theme.panel }, 1.);
        if focus {
            u.d.border(r, u.theme.accent);
        }
        u.text(x + 1., y + 3., &key.to_string(), u.theme.accent);
        if let Some(g) = target {
            u.d.glyph(g, Rect::new(x + 18., y + 3., 8., 16.), u.theme.text, 1.);
        } else {
            u.text(x + 15., y + 3., "·", u.theme.border);
        }
        u.hits.push(Hit {
            rect: r,
            action: Action::MappedKey(slot),
            label: match target {
                Some(g) => format!(
                    "Touche {} insère {} U+{:04X}",
                    key,
                    font::character(g),
                    font::character(g) as u32
                ),
                None => format!("Touche {key} non affectée"),
            },
        });
    }
    let y = if compact { 474. } else { 606. };
    if pages > 1 {
        u.text(
            28.,
            y,
            &format!("Touches {}/{}", page + 1, pages),
            u.theme.muted,
        );
        u.button(180., y - 4., "[Suite]", Action::MappingPage, false);
    } else {
        u.text(28., y - 18., "A–Z + rangée 0–9 sans Maj", u.theme.muted);
        u.text(
            28.,
            y,
            if cfg!(target_os = "macos") {
                "Cmd ↑/↓ : changer de banque"
            } else {
                "Ctrl ↑/↓ : changer de banque"
            },
            u.theme.muted,
        );
    }
}

fn color_window(u: &mut Ui, s: &State, value: &str, x: f32, y: f32) {
    let p = &s.color_picker;
    let plane = color_picker::rect(s.width, Control::Plane);
    let hue = color_picker::rect(s.width, Control::Hue);
    let pure = Picker {
        saturation: 1.,
        value: 1.,
        ..*p
    }
    .rgb();
    let rgba = |c: Color| {
        [
            c[0] as f32 / 255.,
            c[1] as f32 / 255.,
            c[2] as f32 / 255.,
            1.,
        ]
    };
    u.text(x, y + 30., "SATURATION / LUMINOSITÉ", u.theme.muted);
    u.d.gradient(
        plane,
        [rgba([255; 3]), rgba(pure), rgba([255; 3]), rgba(pure)],
    );
    u.d.gradient(
        plane,
        [
            [0., 0., 0., 0.],
            [0., 0., 0., 0.],
            [0., 0., 0., 1.],
            [0., 0., 0., 1.],
        ],
    );
    let marker = (
        plane.x + p.saturation * plane.w,
        plane.y + (1. - p.value) * plane.h,
    );
    u.d.border(Rect::new(marker.0 - 4., marker.1 - 4., 9., 9.), [0; 3]);
    u.d.border(Rect::new(marker.0 - 3., marker.1 - 3., 7., 7.), [255; 3]);
    picker_hit(
        u,
        plane,
        Control::Plane,
        "Saturation et luminosité : flèches gauche/droite et haut/bas",
    );
    u.text(
        x,
        hue.y - 24.,
        &format!("TEINTE {:03.0}°", p.hue * 360.),
        u.theme.muted,
    );
    let hues = [
        [255, 0, 0],
        [255, 255, 0],
        [0, 255, 0],
        [0, 255, 255],
        [0, 0, 255],
        [255, 0, 255],
        [255, 0, 0],
    ];
    for (i, pair) in hues.windows(2).enumerate() {
        u.d.gradient(
            Rect::new(hue.x + i as f32 * hue.w / 6., hue.y, hue.w / 6., hue.h),
            [rgba(pair[0]), rgba(pair[1]), rgba(pair[0]), rgba(pair[1])],
        );
    }
    let hx = hue.x + p.hue * (hue.w - 1.);
    u.d.border(Rect::new(hx - 2., hue.y - 2., 5., hue.h + 4.), [0; 3]);
    u.d.rect(Rect::new(hx, hue.y - 1., 1., hue.h + 2.), [255; 3], 1.);
    picker_hit(
        u,
        hue,
        Control::Hue,
        "Teinte : flèches pour ajuster d’un degré",
    );
    u.field(
        x + 352.,
        y + 54.,
        "HEX / RGB",
        value,
        0,
        s.focus.is_none()
            || s.focus
                .is_some_and(|i| s.hits.get(i).is_some_and(|h| h.action == Action::Input(0))),
    );
    u.text(x + 352., y + 128., "AVANT / APRÈS", u.theme.muted);
    u.d.rect(Rect::new(x + 352., y + 150., 88., 62.), p.original, 1.);
    u.d.rect(Rect::new(x + 440., y + 150., 88., 62.), p.rgb(), 1.);
    u.text(
        x + 352.,
        y + 230.,
        &format!("S {:3.0}% · L {:3.0}%", p.saturation * 100., p.value * 100.),
        u.theme.muted,
    );
    if ditto::project::color_hex(value).is_err() {
        u.text(x + 352., y + 262., "6 chiffres requis", u.theme.accent);
    }
    u.text(
        x,
        y + 336.,
        "Glisser pour choisir · Tab puis flèches au clavier",
        u.theme.muted,
    );
    u.text(
        x,
        y + 364.,
        "Entrée : appliquer · Échap : annuler",
        u.theme.muted,
    );
    u.button(x, y + 404., "[Annuler]", Action::Cancel, false);
    u.button(x + 432., y + 404., "[Appliquer]", Action::Submit, true);
}
fn picker_hit(u: &mut Ui, r: Rect, control: Control, label: &str) {
    if u.focus == Some(u.hits.len()) {
        u.d.border(
            Rect::new(r.x - 6., r.y - 6., r.w + 12., r.h + 12.),
            u.theme.accent,
        );
    }
    u.hits.push(Hit {
        rect: r,
        action: Action::ColorControl(control),
        label: label.into(),
    });
}
