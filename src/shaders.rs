//! Non-destructive, ordered post-processing recipes. Values use document pixels.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

pub const MAX_LAYERS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum Kind {
    Glow = 0,
    Shine = 1,
    Color = 2,
    Duotone = 3,
    Scanlines = 4,
    Pattern = 5,
    Chromatic = 6,
    Grain = 7,
    Vignette = 8,
    // 9/10 are the existing internal glow passes in WGSL.
    Blur = 11,
    ContourBlur = 12,
}
pub const KINDS: [Kind; 11] = [
    Kind::Glow,
    Kind::Shine,
    Kind::Color,
    Kind::Duotone,
    Kind::Scanlines,
    Kind::Pattern,
    Kind::Chromatic,
    Kind::Grain,
    Kind::Vignette,
    Kind::Blur,
    Kind::ContourBlur,
];

#[derive(Clone, Copy)]
pub struct Parameter {
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub initial: f32,
}
const fn p(name: &'static str, min: f32, max: f32, step: f32, initial: f32) -> Parameter {
    Parameter {
        name,
        min,
        max,
        step,
        initial,
    }
}
impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Glow => "Glow",
            Self::Shine => "Brillance",
            Self::Color => "Couleurs",
            Self::Duotone => "Duotone",
            Self::Scanlines => "Scanlines",
            Self::Pattern => "Motif",
            Self::Chromatic => "Chromatique",
            Self::Grain => "Grain",
            Self::Vignette => "Vignette",
            Self::Blur => "Blur",
            Self::ContourBlur => "Blur des contours",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Glow => "Un halo diffus autour des zones lumineuses.",
            Self::Shine => "Un reflet oblique, positionné sur le dessin.",
            Self::Color => "Teinte, saturation et exposition du rendu.",
            Self::Duotone => "Deux encres réparties selon la luminosité.",
            Self::Scanlines => "Les lignes d’un écran cathodique.",
            Self::Pattern => "Trame : 0 points / 1 lignes / 2 damier.",
            Self::Chromatic => "Décalage des canaux rouge et bleu.",
            Self::Grain => "Une texture fixe, reproductible à l’export.",
            Self::Vignette => "Assombrit progressivement les bords.",
            Self::Blur => "Flou global, horizontal et vertical réglables.",
            Self::ContourBlur => "Adoucit les transitions de couleur et d’alpha.",
        }
    }
    pub fn parameters(self) -> [Parameter; 3] {
        match self {
            Self::Glow => [
                p("Rayon / px", 1., 32., 1., 8.),
                p("Puissance", 0., 3., 0.1, 1.4),
                p("Seuil", 0., 1., 0.05, 0.25),
            ],
            Self::Shine => [
                p("Position", 0., 1., 0.05, 0.5),
                p("Largeur", 0.02, 0.8, 0.02, 0.18),
                p("Éclat", 0., 2., 0.1, 0.8),
            ],
            Self::Color => [
                p("Teinte / °", -180., 180., 10., 30.),
                p("Saturation", 0., 2., 0.1, 1.2),
                p("Exposition", -2., 2., 0.1, 0.),
            ],
            Self::Duotone => [
                p("Contraste", 0.2, 3., 0.1, 1.),
                p("Point noir", 0., 0.8, 0.05, 0.),
                p("Inversion", 0., 1., 1., 0.),
            ],
            Self::Scanlines => [
                p("Espacement", 2., 32., 1., 4.),
                p("Épaisseur", 0.1, 0.9, 0.05, 0.35),
                p("Obscurité", 0., 1., 0.05, 0.5),
            ],
            Self::Pattern => [
                p("Espacement", 2., 32., 1., 6.),
                p("Densité", 0.1, 0.9, 0.05, 0.45),
                p("Forme 0/1/2", 0., 2., 1., 0.),
            ],
            Self::Chromatic => [
                p("Décalage / px", 0., 16., 0.5, 2.),
                p("Angle / °", 0., 180., 15., 0.),
                p("Balance", 0., 1., 0.1, 0.5),
            ],
            Self::Grain => [
                p("Taille / px", 1., 8., 1., 1.),
                p("Intensité", 0., 1., 0.05, 0.25),
                p("Graine", 0., 100., 1., 17.),
            ],
            Self::Vignette => [
                p("Rayon", 0.1, 1., 0.05, 0.65),
                p("Douceur", 0.05, 1., 0.05, 0.4),
                p("Obscurité", 0., 1., 0.05, 0.65),
            ],
            Self::Blur => [
                p("Rayon / px", 0., 32., 0.5, 2.),
                p("Horizontal", 0., 1., 0.1, 1.),
                p("Vertical", 0., 1., 0.1, 1.),
            ],
            Self::ContourBlur => [
                p("Rayon / px", 0., 16., 0.5, 2.),
                p("Seuil", 0., 1., 0.01, 0.08),
                p("Douceur", 0.01, 1., 0.01, 0.15),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub kind: Kind,
    pub enabled: bool,
    pub mix: f32,
    pub params: [f32; 3],
    pub colors: [[u8; 3]; 2],
}
impl Layer {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            enabled: true,
            mix: 1.,
            params: kind.parameters().map(|p| p.initial),
            colors: [[27, 16, 50], [255, 183, 113]],
        }
    }
    pub fn parameter(&self, index: usize) -> Parameter {
        if index == 0 {
            p("Mélange", 0., 1., 0.05, 1.)
        } else {
            self.kind.parameters()[index - 1]
        }
    }
    pub fn value(&self, index: usize) -> f32 {
        if index == 0 {
            self.mix
        } else {
            self.params[index - 1]
        }
    }
    pub fn adjust(&mut self, index: usize, direction: i32) {
        if index > 3 {
            return;
        }
        let spec = self.parameter(index);
        let value = (self.value(index) + spec.step * direction as f32).clamp(spec.min, spec.max);
        if index == 0 {
            self.mix = value;
        } else {
            self.params[index - 1] = value;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    pub enabled: bool,
    pub layers: Vec<Layer>,
}
impl Default for Stack {
    fn default() -> Self {
        Self {
            enabled: true,
            layers: vec![],
        }
    }
}
impl Stack {
    pub fn active(&self) -> bool {
        self.enabled && self.layers.iter().any(|l| l.enabled && l.mix > 0.)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(self.layers.len() <= MAX_LAYERS, "Huit shaders maximum.");
        for layer in &self.layers {
            for i in 0..4 {
                let v = layer.value(i);
                let p = layer.parameter(i);
                ensure!(
                    v.is_finite() && (p.min..=p.max).contains(&v),
                    "Réglage shader invalide : {} / {}",
                    layer.kind.name(),
                    p.name
                );
            }
        }
        Ok(())
    }
}

pub const PRESETS: [&str; 4] = ["Néon", "CRT", "Riso", "Irisé"];
pub fn preset(index: usize) -> Stack {
    let mut layers: Vec<Layer> = match index {
        0 => vec![Layer::new(Kind::Duotone), Layer::new(Kind::Glow)],
        1 => vec![
            Layer::new(Kind::Chromatic),
            Layer::new(Kind::Scanlines),
            Layer::new(Kind::Glow),
            Layer::new(Kind::Vignette),
        ],
        2 => vec![
            Layer::new(Kind::Duotone),
            Layer::new(Kind::Pattern),
            Layer::new(Kind::Grain),
        ],
        _ => vec![
            Layer::new(Kind::Color),
            Layer::new(Kind::Shine),
            Layer::new(Kind::Chromatic),
        ],
    };
    match index {
        0 => {
            layers[0].colors = [[35, 12, 65], [111, 241, 226]];
            layers[1].params = [12., 2., 0.2];
        }
        1 => {
            layers[0].params[0] = 1.;
            layers[2].mix = 0.45;
        }
        2 => {
            layers[0].colors = [[46, 26, 75], [241, 139, 89]];
            layers[1].mix = 0.35;
            layers[2].mix = 0.3;
        }
        _ => {
            layers[0].params = [65., 1.4, 0.1];
            layers[2].mix = 0.4;
        }
    }
    Stack {
        enabled: true,
        layers,
    }
}

pub fn save_preset(path: &std::path::Path, stack: &Stack) -> Result<()> {
    stack.validate()?;
    crate::project::atomic_write(path, |f| {
        serde_json::to_writer_pretty(f, stack)?;
        Ok(())
    })
}
pub fn load_preset(path: &std::path::Path) -> Result<Stack> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(64 * 1024 + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 64 * 1024, "Préréglage trop volumineux.");
    let stack: Stack = serde_json::from_slice(&bytes)?;
    stack.validate()?;
    Ok(stack)
}
