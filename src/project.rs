use crate::{
    core::{Asset, Cell, Color, Document, MAX_SIDE, Reference},
    font, typeface,
};
use anyhow::{Context, Result, bail, ensure};
use image::{ImageDecoder, ImageReader, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Cursor, Read, Write},
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};
pub const MAX_IMAGE_SIDE: u32 = 2048;
pub const MAX_FILE: u64 = 64 * 1024 * 1024;
const MAX_DRAWING: u64 = 32 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    version: u32,
    profile: String,
    width: u32,
    height: u32,
    palette: Vec<Color>,
    reference: Option<ReferenceMeta>,
    #[serde(default)]
    shaders: crate::shaders::Stack,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceMeta {
    asset: String,
    x: f32,
    y: f32,
    scale: f32,
    #[serde(default = "legacy_aspect")]
    pixel_aspect: f32,
    opacity: f32,
    visible: bool,
    locked: bool,
}
fn legacy_aspect() -> f32 {
    1.0
}
fn validate(doc: &Document) -> Result<()> {
    doc.shaders.validate()?;
    doc.guides.validate()?;
    ensure!(
        doc.width > 0 && doc.height > 0 && doc.width <= MAX_SIDE && doc.height <= MAX_SIDE,
        "Dimensions invalides."
    );
    ensure!(
        doc.cells.len() == (doc.width * doc.height) as usize,
        "Nombre de cellules incohérent."
    );
    ensure!(
        !doc.palette.is_empty() && doc.palette.len() <= 256,
        "Palette invalide."
    );
    ensure!(
        doc.cells.iter().all(|c| font::valid(c.glyph)),
        "Indice de glyphe hors répertoire."
    );
    if let Some(r) = &doc.reference {
        ensure!(r.valid(), "Transformation de référence invalide.");
        ensure!(
            r.asset.width > 0
                && r.asset.height > 0
                && r.asset.width <= MAX_IMAGE_SIDE
                && r.asset.height <= MAX_IMAGE_SIDE,
            "Référence trop grande (2048 × 2048 maximum)."
        );
        ensure!(
            r.asset.rgba.len() == (r.asset.width * r.asset.height * 4) as usize,
            "Image incohérente."
        );
    }
    Ok(())
}
pub fn atomic_write(path: &Path, write: impl FnOnce(&mut File) -> Result<()>) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let id = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temp = parent.join(format!(".ditto-{}-{id}.tmp", std::process::id()));
    let result = (|| {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .context("Création du fichier temporaire impossible")?;
        write(&mut f)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&temp, path).context("Remplacement du fichier impossible")?;
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}
pub fn save(path: &Path, doc: &Document) -> Result<()> {
    validate(doc)?;
    let manifest = Manifest {
        format: "ditto".into(),
        version: 5,
        profile: typeface::PROFILE.into(),
        width: doc.width,
        height: doc.height,
        palette: doc.palette.clone(),
        shaders: doc.shaders.clone(),
        reference: doc.reference.as_ref().map(|r| ReferenceMeta {
            asset: "assets/reference.png".into(),
            x: r.x,
            y: r.y,
            scale: r.scale,
            pixel_aspect: r.pixel_aspect,
            opacity: r.opacity,
            visible: r.visible,
            locked: r.locked,
        }),
    };
    atomic_write(path, |f| {
        let mut z = ZipWriter::new(f);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        z.start_file("manifest.json", options)?;
        serde_json::to_writer(&mut z, &manifest)?;
        z.start_file("drawing.json", options)?;
        serde_json::to_writer(&mut z, &doc.cells)?;
        z.start_file("guides.json", options)?;
        serde_json::to_writer(&mut z, &doc.guides)?;
        if let Some(r) = &doc.reference {
            z.start_file(
                "assets/reference.png",
                options.compression_method(zip::CompressionMethod::Stored),
            )?;
            let im = RgbaImage::from_raw(r.asset.width, r.asset.height, r.asset.rgba.clone())
                .context("Image invalide")?;
            let mut png = Cursor::new(Vec::new());
            im.write_to(&mut png, image::ImageFormat::Png)?;
            z.write_all(&png.into_inner())?;
        }
        z.finish()?;
        Ok(())
    })
}
fn read_entry(z: &mut ZipArchive<File>, name: &str, max: u64) -> Result<Vec<u8>> {
    let entry = z
        .by_name(name)
        .with_context(|| format!("Entrée manquante : {name}"))?;
    ensure!(entry.size() <= max, "Entrée trop grande : {name}");
    let mut data = Vec::new();
    entry.take(max + 1).read_to_end(&mut data)?;
    ensure!(data.len() as u64 <= max, "Entrée trop grande");
    Ok(data)
}
pub fn load(path: &Path) -> Result<Document> {
    ensure!(
        fs::metadata(path)?.len() <= MAX_FILE,
        "Projet trop volumineux."
    );
    let mut z = ZipArchive::new(File::open(path)?)?;
    ensure!((2..=4).contains(&z.len()), "Conteneur inattendu.");
    let names = z.file_names().map(str::to_owned).collect::<Vec<_>>();
    let mut unique = std::collections::HashSet::new();
    for n in &names {
        ensure!(
            [
                "manifest.json",
                "drawing.json",
                "guides.json",
                "assets/reference.png"
            ]
            .contains(&n.as_str())
                && unique.insert(n),
            "Entrée inattendue ou dupliquée."
        );
    }
    let m: Manifest = serde_json::from_slice(&read_entry(&mut z, "manifest.json", 64 * 1024)?)?;
    ensure!(
        m.format == "ditto" && [1, 2, 3, 4, 5].contains(&m.version),
        "Version de projet non prise en charge."
    );
    ensure!(
        (m.version == 1 && m.profile == font::LEGACY_PROFILE)
            || (m.version == 2 && m.profile == font::PROFILE)
            || ([3, 4, 5].contains(&m.version) && m.profile == typeface::PROFILE),
        "Profil de glyphes non pris en charge."
    );
    let mut doc = Document::new(m.width, m.height).map_err(anyhow::Error::msg)?;
    let cells: Vec<Cell> =
        serde_json::from_slice(&read_entry(&mut z, "drawing.json", MAX_DRAWING)?)?;
    ensure!(
        m.version != 1 || cells.iter().all(|c| c.glyph < 256),
        "Glyphe étendu invalide dans un projet V1."
    );
    doc.cells = Arc::new(cells);
    doc.palette = m.palette;
    ensure!(
        m.version >= 4 || m.shaders == crate::shaders::Stack::default(),
        "Shaders incompatibles avec cette version de projet."
    );
    doc.shaders = m.shaders;
    if m.version >= 5 {
        doc.guides = serde_json::from_slice(&read_entry(&mut z, "guides.json", 8 * 1024 * 1024)?)?;
    } else {
        ensure!(
            !names.iter().any(|n| n == "guides.json"),
            "Guides incompatibles avec cette version de projet."
        );
    }
    let expected_entries = 2 + usize::from(m.version >= 5) + usize::from(m.reference.is_some());
    ensure!(z.len() == expected_entries, "Conteneur inattendu.");
    if let Some(r) = m.reference {
        ensure!(r.asset == "assets/reference.png", "Ressource inattendue.");
        let asset = decode_reference(&read_entry(&mut z, &r.asset, 24 * 1024 * 1024)?)?;
        doc.reference = Some(Reference {
            asset,
            x: r.x,
            y: r.y,
            scale: r.scale,
            pixel_aspect: r.pixel_aspect,
            opacity: r.opacity,
            visible: r.visible,
            locked: r.locked,
        });
    }
    validate(&doc)?;
    Ok(doc)
}
pub fn decode_reference(bytes: &[u8]) -> Result<Arc<Asset>> {
    ensure!(
        bytes.len() <= 32 * 1024 * 1024,
        "Image trop volumineuse (32 Mio maximum)."
    );
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_SIDE);
    limits.max_image_height = Some(MAX_IMAGE_SIDE);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut im = image::DynamicImage::from_decoder(decoder)?;
    im.apply_orientation(orientation);
    let im = im.into_rgba8();
    let (width, height) = im.dimensions();
    ensure!(width > 0 && height > 0, "Image vide.");
    Ok(Arc::new(Asset {
        width,
        height,
        rgba: im.into_raw(),
    }))
}
pub fn import_reference(path: &Path) -> Result<Arc<Asset>> {
    ensure!(
        fs::metadata(path)?.len() <= 32 * 1024 * 1024,
        "Image trop volumineuse."
    );
    decode_reference(&fs::read(path)?)
}
pub fn render_png(doc: &Document, scale: u32, background: Option<Color>) -> Result<RgbaImage> {
    if !doc.shaders.active() {
        return render_cells(doc, scale, background);
    }
    let raw = render_cells(doc, scale, None)?;
    let mut out = crate::shader_gpu::render(&raw, &doc.shaders, scale as f32)?;
    if let Some(bg) = background {
        for pixel in out.pixels_mut() {
            let a = pixel[3] as f32 / 255.;
            for k in 0..3 {
                pixel[k] = (pixel[k] as f32 * a + bg[k] as f32 * (1. - a)).round() as u8;
            }
            pixel[3] = 255;
        }
    }
    Ok(out)
}
/// The clean drawing, without shaders, reference image or editor overlays.
pub fn render_cells(doc: &Document, scale: u32, background: Option<Color>) -> Result<RgbaImage> {
    validate(doc)?;
    ensure!([1, 2, 4].contains(&scale), "Échelle invalide.");
    let cw = typeface::CELL_WIDTH * scale;
    let ch = typeface::CELL_HEIGHT * scale;
    let w = doc.width * cw;
    let h = doc.height * ch;
    ensure!(
        u64::from(w) * u64::from(h) <= 64 * 1024 * 1024,
        "Export trop grand (64 millions de pixels maximum)."
    );
    let base = background.map_or([0, 0, 0, 0], |c| [c[0], c[1], c[2], 255]);
    let mut out = RgbaImage::from_pixel(w, h, Rgba(base));
    for cy in 0..doc.height {
        for cx in 0..doc.width {
            let cell = doc.cells[(cy * doc.width + cx) as usize];
            if cell.glyph == 32 && cell.bg.is_none() {
                continue;
            }
            let bg = cell.bg.map_or(base, |c| [c[0], c[1], c[2], 255]);
            for y in 0..ch {
                for x in 0..cw {
                    let a = typeface::coverage(cell.glyph, x, y, scale) as f32 / 255.;
                    let ba = bg[3] as f32 / 255.;
                    let alpha = a + ba * (1. - a);
                    let mut rgba = [0, 0, 0, (alpha * 255.).round() as u8];
                    if alpha > 0. {
                        for k in 0..3 {
                            rgba[k] = ((cell.fg[k] as f32 * a + bg[k] as f32 * ba * (1. - a))
                                / alpha)
                                .round() as u8;
                        }
                    }
                    out.put_pixel(cx * cw + x, cy * ch + y, Rgba(rgba));
                }
            }
        }
    }
    Ok(out)
}
pub fn export_png(
    path: &Path,
    doc: &Document,
    scale: u32,
    background: Option<Color>,
) -> Result<()> {
    let im = render_png(doc, scale, background)?;
    atomic_write(path, |f| {
        im.write_to(f, image::ImageFormat::Png)?;
        Ok(())
    })
}
pub fn color_hex(text: &str) -> Result<Color> {
    let s = text.trim().trim_start_matches('#');
    if s.len() != 6 || !s.is_ascii() {
        bail!("Couleur : six chiffres hexadécimaux.");
    }
    Ok([
        u8::from_str_radix(&s[0..2], 16)?,
        u8::from_str_radix(&s[2..4], 16)?,
        u8::from_str_radix(&s[4..6], 16)?,
    ])
}
