//! Lossless text export. Formatting is a transport envelope, never a glyph rewrite.
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
}
impl Export {
    pub fn new(doc: &Document, selection: Option<Rect>) -> Self {
        let r = selection.unwrap_or(doc.bounds()).intersection(doc.bounds());
        let content = doc.text(Some(r));
        Self {
            width: r.w as usize,
            height: r.h as usize,
            selection: selection.is_some(),
            parts: Ok(vec![content.clone()]),
            content,
            format: Format::Plain,
            part: 0,
        }
    }
    pub fn set_format(&mut self, format: Format) {
        self.format = format;
        self.part = 0;
        self.parts = match format {
            Format::Plain => Ok(vec![self.content.clone()]),
            Format::Markdown => {
                let mut longest = 0;
                let mut run = 0;
                for c in self.content.chars() {
                    run = if c == '`' { run + 1 } else { 0 };
                    longest = longest.max(run);
                }
                let fence = "`".repeat((longest + 1).max(3));
                Ok(vec![format!("{fence}\n{}\n{fence}", self.content)])
            }
            Format::Discord => discord_parts(&self.content),
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
                .unwrap_or(&self.content)
        } else {
            &self.content
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
            file.write_all(self.content.as_bytes())?;
            Ok(())
        })
    }
    pub fn html(&self) -> String {
        let escaped = self
            .content
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
