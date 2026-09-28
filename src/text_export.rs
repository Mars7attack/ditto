//! Text export with explicit, reversible braille spacing compatibility.
//! The document/source snapshot stays intact; only exported blank cells change.
use crate::{
    core::{Document, Rect},
    project,
};
use std::{io::Write, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Plain,
    Markdown,
    Discord,
}
impl Format {
    pub const ALL: [Self; 3] = [Self::Plain, Self::Markdown, Self::Discord];
    pub fn label(self) -> &'static str {
        match self {
            Self::Plain => "Texte / apps",
            Self::Markdown => "Markdown",
            Self::Discord => "Discord",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Export {
    pub content: String,
    pub width: usize,
    pub height: usize,
    pub selection: bool,
    pub format: Format,
    pub part: usize,
    pub parts: Result<Vec<String>, String>,
    pub braille_compatible: bool,
    pub align_braille: bool,
    output: String,
}
impl Export {
    pub fn new(doc: &Document, selection: Option<Rect>) -> Self {
        let r = selection.unwrap_or(doc.bounds()).intersection(doc.bounds());
        let content = doc.text(Some(r));
        let braille_compatible = content
            .chars()
            .all(|c| matches!(c, ' ' | '\n' | '\u{a0}' | '\u{2800}'..='\u{28ff}'))
            && content
                .chars()
                .any(|c| matches!(c, '\u{2801}'..='\u{28ff}'));
        let mut export = Self {
            width: r.w as usize,
            height: r.h as usize,
            selection: selection.is_some(),
            parts: Ok(Vec::new()),
            content,
            format: Format::Plain,
            part: 0,
            braille_compatible,
            align_braille: braille_compatible,
            output: String::new(),
        };
        export.refresh();
        export
    }
    pub fn set_format(&mut self, format: Format) {
        self.format = format;
        self.refresh();
    }
    pub fn set_braille_alignment(&mut self, enabled: bool) {
        self.align_braille = enabled && self.braille_compatible;
        self.refresh();
    }
    pub fn output(&self) -> &str {
        &self.output
    }
    fn refresh(&mut self) {
        self.part = 0;
        self.output = if self.align_braille {
            self.content
                .chars()
                .map(|c| {
                    if matches!(c, ' ' | '\u{a0}') {
                        '\u{2800}'
                    } else {
                        c
                    }
                })
                .collect()
        } else {
            self.content.clone()
        };
        self.parts = match self.format {
            Format::Plain => Ok(vec![self.output.clone()]),
            Format::Markdown => {
                let mut longest = 0;
                let mut run = 0;
                for c in self.output.chars() {
                    run = if c == '`' { run + 1 } else { 0 };
                    longest = longest.max(run);
                }
                let fence = "`".repeat((longest + 1).max(3));
                Ok(vec![format!("{fence}\n{}\n{fence}", self.output)])
            }
            Format::Discord => discord_parts(&self.output),
        };
    }
    pub fn payload(&self) -> Result<&str, &str> {
        match &self.parts {
            Ok(parts) => parts
                .get(self.part)
                .map(String::as_str)
                .ok_or("Aucun bloc à copier."),
            Err(e) => Err(e),
        }
    }
    pub fn preview(&self) -> &str {
        if self.format == Format::Discord {
            self.payload()
                .ok()
                .and_then(|s| s.strip_prefix("```\n"))
                .and_then(|s| s.strip_suffix("\n```"))
                .unwrap_or(&self.output)
        } else {
            &self.output
        }
    }
    pub fn preview_height(&self) -> usize {
        self.preview().split('\n').count()
    }
    pub fn change_part(&mut self, delta: i32) {
        if let Ok(parts) = &self.parts {
            self.part =
                (self.part as i32 + delta).clamp(0, parts.len().saturating_sub(1) as i32) as usize;
        }
    }
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        project::atomic_write(path, |file| {
            file.write_all(self.output.as_bytes())?;
            Ok(())
        })
    }
    pub fn html(&self) -> String {
        let escaped = self
            .output
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        format!(
            "<pre style=\"font-family: 'SFMono-Regular', Consolas, 'DejaVu Sans Mono', monospace; font-size: 14px; line-height: 1.2; white-space: pre; margin: 0;\">{escaped}</pre>"
        )
    }
}

fn discord_parts(content: &str) -> Result<Vec<String>, String> {
    // Discord documents triple-backtick blocks; do not silently alter glyphs
    // with zero-width characters or rely on unsupported longer fences.
    if content.contains("```") {
        return Err(
            "Le dessin contient trois accents graves consécutifs. Utiliser le fichier .txt.".into(),
        );
    }
    let mut parts = Vec::new();
    let mut lines = Vec::new();
    let mut length = 0;
    // 8 UTF-16 units for the opening/closing fence and their newlines.
    for line in content.split('\n') {
        let size = line.encode_utf16().count();
        if size + 8 > 2000 {
            return Err("Une ligne dépasse la limite Discord. Utiliser le fichier .txt.".into());
        }
        let separator = usize::from(!lines.is_empty());
        if length + separator + size + 8 > 2000 {
            parts.push(format!("```\n{}\n```", lines.join("\n")));
            lines.clear();
            length = 0;
        }
        length += usize::from(!lines.is_empty()) + size;
        lines.push(line);
    }
    parts.push(format!("```\n{}\n```", lines.join("\n")));
    Ok(parts)
}
