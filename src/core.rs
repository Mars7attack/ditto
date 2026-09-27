use crate::font;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, sync::Arc};

pub type Color = [u8; 3];
pub const MAX_SIDE: u32 = 512;
pub const HISTORY_LIMIT: usize = 64 * 1024 * 1024;
pub const DEFAULT_FG: Color = [191, 192, 205];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub glyph: font::Glyph,
    pub fg: Color,
    pub bg: Option<Color>,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            glyph: 32,
            fg: DEFAULT_FG,
            bg: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub asset: Arc<Asset>,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub pixel_aspect: f32,
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
}
impl Reference {
    pub fn fit(asset: Arc<Asset>, w: u32, h: u32, cover: bool) -> Self {
        let sx = w as f32 / asset.width as f32;
        let sy = h as f32 / (asset.height as f32 * crate::typeface::ASPECT);
        let scale = if cover { sx.max(sy) } else { sx.min(sy) };
        Self {
            x: (w as f32 - asset.width as f32 * scale) / 2.,
            y: (h as f32 - asset.height as f32 * scale * crate::typeface::ASPECT) / 2.,
            asset,
            scale,
            pixel_aspect: crate::typeface::ASPECT,
            opacity: 0.35,
            visible: true,
            locked: true,
        }
    }
    pub fn valid(&self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.x.abs() <= 1e6
            && self.y.abs() <= 1e6
            && self.scale.is_finite()
            && self.scale > 0.
            && self.scale <= 1e4
            && self.pixel_aspect.is_finite()
            && self.pixel_aspect > 0.
            && self.pixel_aspect <= 4.
            && self.opacity.is_finite()
            && (0.0..=1.0).contains(&self.opacity)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub width: u32,
    pub height: u32,
    pub cells: Arc<Vec<Cell>>,
    pub palette: Vec<Color>,
    pub reference: Option<Reference>,
    pub shaders: crate::shaders::Stack,
    pub guides: crate::guides::Layer,
}
impl Document {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
            return Err("Dimensions : de 1 à 512 cellules par côté.".into());
        }
        Ok(Self {
            width,
            height,
            cells: Arc::new(vec![Cell::default(); (width * height) as usize]),
            palette: vec![
                [16, 16, 20],
                [73, 69, 82],
                [120, 113, 127],
                DEFAULT_FG,
                [229, 223, 206],
                [132, 126, 174],
                [189, 121, 95],
                [216, 162, 98],
                [126, 168, 140],
                [91, 143, 157],
                [105, 132, 181],
                [175, 107, 136],
                [238, 192, 109],
                [235, 121, 103],
                [232, 230, 233],
                [255, 255, 255],
            ],
            reference: None,
            shaders: crate::shaders::Stack::default(),
            guides: crate::guides::Layer::default(),
        })
    }
    pub fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32)
            .then(|| (y as u32 * self.width + x as u32) as usize)
    }
    pub fn get(&self, x: i32, y: i32) -> Option<Cell> {
        self.index(x, y).map(|i| self.cells[i])
    }
    pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            Arc::make_mut(&mut self.cells)[i] = cell;
        }
    }
    pub fn bounds(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.width as i32,
            h: self.height as i32,
        }
    }
    pub fn text(&self, selection: Option<Rect>) -> String {
        let r = selection
            .unwrap_or(self.bounds())
            .intersection(self.bounds());
        (r.y..r.y + r.h)
            .map(|y| {
                (r.x..r.x + r.w)
                    .map(|x| font::character(self.get(x, y).unwrap().glyph))
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        let mut new = Self::new(width, height)?;
        for y in 0..height.min(self.height) {
            for x in 0..width.min(self.width) {
                new.set(x as i32, y as i32, self.get(x as i32, y as i32).unwrap());
            }
        }
        self.width = width;
        self.height = height;
        self.cells = new.cells;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl Rect {
    pub fn from_points(a: (i32, i32), b: (i32, i32)) -> Self {
        Self {
            x: a.0.min(b.0),
            y: a.1.min(b.1),
            w: (a.0 - b.0).abs() + 1,
            h: (a.1 - b.1).abs() + 1,
        }
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            w: (self.x + self.w)
                .min(other.x + other.w)
                .saturating_sub(x)
                .max(0),
            h: (self.y + self.h)
                .min(other.y + other.h)
                .saturating_sub(y)
                .max(0),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Brush {
    pub cell: Cell,
    pub glyph: bool,
    pub fg: bool,
    pub bg: bool,
}
impl Default for Brush {
    fn default() -> Self {
        Self {
            cell: Cell {
                glyph: 177,
                ..Cell::default()
            },
            glyph: true,
            fg: true,
            bg: false,
        }
    }
}
impl Brush {
    pub fn apply(&self, old: Cell) -> Cell {
        Cell {
            glyph: if self.glyph {
                self.cell.glyph
            } else {
                old.glyph
            },
            fg: if self.fg { self.cell.fg } else { old.fg },
            bg: if self.bg { self.cell.bg } else { old.bg },
        }
    }
    pub fn matches(&self, a: Cell, b: Cell) -> bool {
        (!self.glyph || a.glyph == b.glyph)
            && (!self.fg || a.fg == b.fg)
            && (!self.bg || a.bg == b.bg)
    }
    pub fn active(&self) -> bool {
        self.glyph || self.fg || self.bg
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Recolor,
    Guide,
    GuideErase,
    Eraser,
    Line,
    Rectangle,
    Fill,
    Text,
    Select,
    Pick,
}
impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Self::Pencil => "Crayon",
            Self::Recolor => "Recolorer",
            Self::Guide => "Guides",
            Self::GuideErase => "Gomme guides",
            Self::Eraser => "Gomme",
            Self::Line => "Ligne",
            Self::Rectangle => "Rectangle",
            Self::Fill => "Remplir",
            Self::Text => "Texte",
            Self::Select => "Sélection",
            Self::Pick => "Pipette",
        }
    }
}
pub fn line(a: (i32, i32), b: (i32, i32)) -> Vec<(i32, i32)> {
    let (mut x, mut y) = a;
    let dx = (b.0 - x).abs();
    let sx = if x < b.0 { 1 } else { -1 };
    let dy = -(b.1 - y).abs();
    let sy = if y < b.1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut out = Vec::new();
    loop {
        out.push((x, y));
        if (x, y) == b {
            break;
        }
        let e = 2 * err;
        if e >= dy {
            err += dy;
            x += sx;
        }
        if e <= dx {
            err += dx;
            y += sy;
        }
    }
    out
}
pub fn shape(tool: Tool, a: (i32, i32), b: (i32, i32), filled: bool) -> Vec<(i32, i32)> {
    if tool == Tool::Line {
        return line(a, b);
    }
    if tool == Tool::Rectangle {
        let r = Rect::from_points(a, b);
        let mut out = Vec::new();
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                if filled || x == r.x || y == r.y || x == r.x + r.w - 1 || y == r.y + r.h - 1 {
                    out.push((x, y));
                }
            }
        }
        out
    } else {
        vec![b]
    }
}
#[derive(Debug, Clone)]
pub struct Block {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Cell>,
}
impl Block {
    pub fn copy(doc: &Document, r: Rect) -> Self {
        let r = r.intersection(doc.bounds());
        let mut cells = Vec::new();
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                cells.push(doc.get(x, y).unwrap());
            }
        }
        Self {
            width: r.w as u32,
            height: r.h as u32,
            cells,
        }
    }
    pub fn from_text(text: &str, brush: Brush) -> Result<Self, String> {
        let rows = font::parse_text(text)?;
        let width = rows.iter().map(Vec::len).max().unwrap_or(0);
        if width == 0 {
            return Err("Le presse-papiers est vide.".into());
        }
        let height = rows.len();
        let mut cells = vec![
            Cell {
                glyph: 32,
                ..brush.cell
            };
            width * height
        ];
        for (y, row) in rows.iter().enumerate() {
            for (x, g) in row.iter().enumerate() {
                cells[y * width + x].glyph = *g;
            }
        }
        Ok(Self {
            width: width as u32,
            height: height as u32,
            cells,
        })
    }
    pub fn fits(&self, doc: &Document, p: (i32, i32)) -> bool {
        p.0 >= 0
            && p.1 >= 0
            && p.0 + self.width as i32 <= doc.width as i32
            && p.1 + self.height as i32 <= doc.height as i32
    }
    pub fn paste(&self, doc: &mut Document, p: (i32, i32)) {
        for y in 0..self.height {
            for x in 0..self.width {
                doc.set(
                    p.0 + x as i32,
                    p.1 + y as i32,
                    self.cells[(y * self.width + x) as usize],
                );
            }
        }
    }
}
#[derive(Debug, Clone)]
struct Delta {
    index: usize,
    before: Cell,
    after: Cell,
}
#[derive(Debug, Clone)]
enum Change {
    Cells(Vec<Delta>),
    Guides(crate::guides::Layer, crate::guides::Layer),
    Shaders(crate::shaders::Stack, crate::shaders::Stack),
    Document(Box<(Document, Document)>),
}
#[derive(Debug, Clone)]
struct Entry {
    change: Change,
    before: u64,
    after: u64,
    cost: usize,
}
#[derive(Debug)]
pub struct Editor {
    pub document: Document,
    pub selection: Option<Rect>,
    pub revision: u64,
    pub saved_revision: u64,
    next_revision: u64,
    undo: VecDeque<Entry>,
    redo: Vec<Entry>,
    pending: Option<Document>,
    memory: usize,
}
impl Editor {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            selection: None,
            revision: 0,
            saved_revision: 0,
            next_revision: 1,
            undo: VecDeque::new(),
            redo: Vec::new(),
            pending: None,
            memory: 0,
        }
    }
    pub fn dirty(&self) -> bool {
        self.revision != self.saved_revision
    }
    pub fn begin(&mut self) {
        if self.pending.is_none() {
            self.pending = Some(self.document.clone());
        }
    }
    pub fn cancel(&mut self) {
        if let Some(before) = self.pending.take() {
            self.document = before;
        }
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn commit(&mut self) -> bool {
        let Some(before) = self.pending.take() else {
            return false;
        };
        if before == self.document {
            return false;
        }
        let change = if before.width == self.document.width
            && before.height == self.document.height
            && before.reference == self.document.reference
            && before.palette == self.document.palette
            && Arc::ptr_eq(&before.cells, &self.document.cells)
        {
            if before.shaders == self.document.shaders {
                Change::Guides(before.guides, self.document.guides.clone())
            } else if before.guides == self.document.guides {
                Change::Shaders(before.shaders, self.document.shaders.clone())
            } else {
                Change::Document(Box::new((before, self.document.clone())))
            }
        } else if before.width == self.document.width
            && before.height == self.document.height
            && before.reference == self.document.reference
            && before.palette == self.document.palette
            && before.shaders == self.document.shaders
            && before.guides == self.document.guides
        {
            Change::Cells(
                before
                    .cells
                    .iter()
                    .zip(self.document.cells.iter())
                    .enumerate()
                    .filter_map(|(index, (a, b))| {
                        (*a != *b).then_some(Delta {
                            index,
                            before: *a,
                            after: *b,
                        })
                    })
                    .collect(),
            )
        } else {
            Change::Document(Box::new((before, self.document.clone())))
        };
        let cost = match &change {
            Change::Guides(a, b) => {
                a.memory_cost() + b.memory_cost() + std::mem::size_of::<Entry>()
            }
            Change::Shaders(a, b) => {
                (a.layers.len() + b.layers.len()) * std::mem::size_of::<crate::shaders::Layer>()
                    + std::mem::size_of::<Entry>()
            }
            Change::Cells(v) => v.len() * std::mem::size_of::<Delta>(),
            Change::Document(d) => {
                (d.0.cells.len() + d.1.cells.len()) * std::mem::size_of::<Cell>()
                    + d.0.guides.memory_cost()
                    + d.1.guides.memory_cost()
                    + d.0.reference.as_ref().map_or(0, |r| r.asset.rgba.len())
                    + d.1.reference.as_ref().map_or(0, |r| r.asset.rgba.len())
            }
        };
        self.memory = self
            .memory
            .saturating_sub(self.redo.iter().map(|e| e.cost).sum::<usize>());
        self.redo.clear();
        let after = self.next_revision;
        self.next_revision += 1;
        self.undo.push_back(Entry {
            change,
            before: self.revision,
            after,
            cost,
        });
        self.revision = after;
        self.memory += cost;
        while self.memory > HISTORY_LIMIT && self.undo.len() > 1 {
            if let Some(e) = self.undo.pop_front() {
                self.memory = self.memory.saturating_sub(e.cost);
            }
        }
        true
    }
    pub fn edit(&mut self, f: impl FnOnce(&mut Document)) {
        self.begin();
        f(&mut self.document);
        self.commit();
    }
    fn apply_entry(&mut self, e: &Entry, forward: bool) {
        match &e.change {
            Change::Guides(a, b) => {
                self.document.guides = if forward { b.clone() } else { a.clone() }
            }
            Change::Shaders(a, b) => {
                self.document.shaders = if forward { b.clone() } else { a.clone() }
            }
            Change::Cells(v) => {
                let cells = Arc::make_mut(&mut self.document.cells);
                for d in v {
                    cells[d.index] = if forward { d.after } else { d.before };
                }
            }
            Change::Document(d) => self.document = if forward { d.1.clone() } else { d.0.clone() },
        }
        self.revision = if forward { e.after } else { e.before };
        self.selection = self
            .selection
            .map(|r| r.intersection(self.document.bounds()))
            .filter(|r| r.w > 0 && r.h > 0);
    }
    pub fn undo(&mut self) -> bool {
        self.cancel();
        if let Some(e) = self.undo.pop_back() {
            self.apply_entry(&e, false);
            self.redo.push(e);
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        self.cancel();
        if let Some(e) = self.redo.pop() {
            self.apply_entry(&e, true);
            self.undo.push_back(e);
            true
        } else {
            false
        }
    }
    pub fn paint(&mut self, points: &[(i32, i32)], brush: Brush, erase: bool) {
        let bounds = self.selection.unwrap_or(self.document.bounds());
        for &(x, y) in points {
            if bounds.contains(x, y)
                && let Some(old) = self.document.get(x, y)
            {
                self.document.set(
                    x,
                    y,
                    if erase {
                        Cell::default()
                    } else {
                        brush.apply(old)
                    },
                );
            }
        }
    }
    /// Repaint only visible glyph foregrounds; preserve glyphs, backgrounds and empty cells.
    pub fn recolor(&mut self, centers: &[(i32, i32)], color: Color, radius: i32) {
        let radius = radius.clamp(0, 8);
        let bounds = self.selection.unwrap_or(self.document.bounds());
        for &(cx, cy) in centers {
            for y in cy - radius..=cy + radius {
                for x in cx - radius..=cx + radius {
                    if (x - cx).pow(2) + (y - cy).pow(2) > radius.pow(2) || !bounds.contains(x, y) {
                        continue;
                    }
                    if let Some(mut c) = self.document.get(x, y)
                        && !matches!(c.glyph, 0 | 32 | 255)
                        && font::character(c.glyph) != '\u{2800}'
                        && c.fg != color
                    {
                        c.fg = color;
                        self.document.set(x, y, c);
                    }
                }
            }
        }
    }
    pub fn flood(&mut self, p: (i32, i32), brush: Brush) {
        if !brush.active() {
            return;
        }
        let Some(target) = self.document.get(p.0, p.1) else {
            return;
        };
        let bounds = self.selection.unwrap_or(self.document.bounds());
        if !bounds.contains(p.0, p.1) {
            return;
        }
        let mut seen = vec![false; self.document.cells.len()];
        let mut queue = VecDeque::from([p]);
        while let Some((x, y)) = queue.pop_front() {
            if !bounds.contains(x, y) {
                continue;
            }
            let Some(i) = self.document.index(x, y) else {
                continue;
            };
            if seen[i] {
                continue;
            }
            seen[i] = true;
            let old = self.document.cells[i];
            if !brush.matches(old, target) {
                continue;
            }
            self.document.set(x, y, brush.apply(old));
            queue.extend([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]);
        }
    }
    pub fn clear_selection(&mut self) {
        let r = self.selection.unwrap_or(self.document.bounds());
        self.begin();
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                self.document.set(x, y, Cell::default());
            }
        }
        self.commit();
    }
    pub fn move_selection(&mut self, to: (i32, i32)) -> bool {
        let Some(r) = self.selection else {
            return false;
        };
        let block = Block::copy(&self.document, r);
        if !block.fits(&self.document, to) {
            return false;
        }
        self.begin();
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                self.document.set(x, y, Cell::default());
            }
        }
        block.paste(&mut self.document, to);
        self.commit();
        self.selection = Some(Rect {
            x: to.0,
            y: to.1,
            w: r.w,
            h: r.h,
        });
        true
    }
}
