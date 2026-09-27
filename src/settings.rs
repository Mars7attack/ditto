//! Local application preferences, independent of document history and exports.
use crate::core::Color;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub background: Color,
    pub panel: Color,
    pub text: Color,
    pub muted: Color,
    pub border: Color,
    pub accent: Color,
    pub active_text: Color,
    pub focus: Color,
    pub canvas: Color,
    pub grid: Color,
    pub selection: Color,
    pub checker: Color,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Background,
    Panel,
    Text,
    Muted,
    Border,
    Accent,
    ActiveText,
    Focus,
    Canvas,
    Grid,
    Selection,
    Checker,
}
impl Token {
    pub const ALL: [Self; 12] = [
        Self::Background,
        Self::Panel,
        Self::Text,
        Self::Muted,
        Self::Border,
        Self::Accent,
        Self::ActiveText,
        Self::Focus,
        Self::Canvas,
        Self::Grid,
        Self::Selection,
        Self::Checker,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Background => "Fond application",
            Self::Panel => "Panneaux / boutons",
            Self::Text => "Texte principal",
            Self::Muted => "Texte secondaire",
            Self::Border => "Bordures",
            Self::Accent => "Accent / curseur",
            Self::ActiveText => "Texte actif",
            Self::Focus => "Fond focus",
            Self::Canvas => "Fond canevas",
            Self::Grid => "Grille",
            Self::Selection => "Sélection",
            Self::Checker => "Damier transparence",
        }
    }
    pub fn color(self, theme: &Theme) -> Color {
        let mut copy = *theme;
        *self.slot(&mut copy)
    }
    pub fn slot(self, t: &mut Theme) -> &mut Color {
        match self {
            Self::Background => &mut t.background,
            Self::Panel => &mut t.panel,
            Self::Text => &mut t.text,
            Self::Muted => &mut t.muted,
            Self::Border => &mut t.border,
            Self::Accent => &mut t.accent,
            Self::ActiveText => &mut t.active_text,
            Self::Focus => &mut t.focus,
            Self::Canvas => &mut t.canvas,
            Self::Grid => &mut t.grid,
            Self::Selection => &mut t.selection,
            Self::Checker => &mut t.checker,
        }
    }
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            background: [16, 16, 20],
            panel: [20, 19, 24],
            text: [210, 207, 214],
            muted: [165, 157, 169],
            border: [94, 88, 103],
            accent: [230, 161, 109],
            active_text: [16, 16, 20],
            focus: [67, 57, 72],
            canvas: [12, 12, 16],
            grid: [49, 43, 56],
            selection: [161, 151, 195],
            checker: [33, 31, 38],
        }
    }
}
impl Theme {
    pub const PRESETS: [&'static str; 3] = ["Ditto", "Minuit", "Papier"];
    pub fn preset(i: usize) -> Self {
        match i {
            1 => Self {
                background: [13, 22, 33],
                panel: [20, 32, 46],
                text: [224, 236, 245],
                muted: [154, 180, 201],
                border: [84, 112, 138],
                accent: [107, 210, 205],
                active_text: [8, 24, 32],
                focus: [42, 66, 88],
                canvas: [9, 17, 27],
                grid: [43, 67, 87],
                selection: [229, 181, 107],
                checker: [30, 46, 61],
            },
            2 => Self {
                background: [241, 237, 228],
                panel: [252, 249, 243],
                text: [39, 41, 45],
                muted: [90, 86, 82],
                border: [147, 138, 123],
                accent: [107, 63, 126],
                active_text: [255, 255, 255],
                focus: [224, 212, 234],
                canvas: [235, 230, 218],
                grid: [184, 174, 153],
                selection: [39, 117, 122],
                checker: [211, 204, 188],
            },
            _ => Self::default(),
        }
    }
    pub fn low_contrast(&self) -> bool {
        let luminance = |c: Color| {
            c.into_iter()
                .zip([0.2126, 0.7152, 0.0722])
                .map(|(v, w)| {
                    let v = v as f32 / 255.;
                    w * if v <= 0.04045 {
                        v / 12.92
                    } else {
                        ((v + 0.055) / 1.055).powf(2.4)
                    }
                })
                .sum::<f32>()
        };
        let contrast = |a, b| {
            let a = luminance(a);
            let b = luminance(b);
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        };
        contrast(self.text, self.panel) < 4.5
            || contrast(self.active_text, self.accent) < 4.5
            || contrast(self.muted, self.background) < 3.
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub version: u32,
    pub theme: Theme,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            theme: Theme::default(),
        }
    }
}
pub fn path() -> PathBuf {
    directories::ProjectDirs::from("org", "Ditto", "Ditto")
        .map(|d| d.config_dir().join("settings.json"))
        .unwrap_or_else(|| PathBuf::from(".ditto-settings.json"))
}
pub fn load(path: &Path) -> Result<Settings> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    ensure!(
        std::fs::metadata(path)?.len() <= 16 * 1024,
        "Réglages trop volumineux."
    );
    let s: Settings = serde_json::from_slice(&std::fs::read(path)?)?;
    ensure!(s.version == 1, "Version de réglages non prise en charge.");
    Ok(s)
}
pub fn save(path: &Path, settings: &Settings) -> Result<()> {
    ensure!(
        settings.version == 1,
        "Version de réglages non prise en charge."
    );
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    crate::project::atomic_write(path, |f| {
        serde_json::to_writer_pretty(f, settings)?;
        Ok(())
    })
}
