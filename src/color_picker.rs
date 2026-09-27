use crate::render::Rect;
use ditto::core::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Plane,
    Hue,
}

/// Modal draft only. Keep hue when saturation/value is zero so a drag through
/// white or black does not reset the chosen hue to red.
#[derive(Clone, Copy, Debug, Default)]
pub struct Picker {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
    pub original: Color,
}
impl Picker {
    pub fn new(color: Color) -> Self {
        let mut picker = Self {
            original: color,
            ..Self::default()
        };
        picker.sync(color);
        picker
    }
    pub fn sync(&mut self, color: Color) {
        let [r, g, b] = color.map(|v| v as f32 / 255.);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        self.value = max;
        if max > 0. {
            self.saturation = delta / max;
        }
        if delta > 0. {
            self.hue = (if max == r {
                (g - b) / delta
            } else if max == g {
                (b - r) / delta + 2.
            } else {
                (r - g) / delta + 4.
            })
            .rem_euclid(6.)
                / 6.;
        }
    }
    pub fn rgb(&self) -> Color {
        let h = self.hue.rem_euclid(1.) * 6.;
        let c = self.value * self.saturation;
        let x = c * (1. - (h.rem_euclid(2.) - 1.).abs());
        let rgb = match h as u32 {
            0 => [c, x, 0.],
            1 => [x, c, 0.],
            2 => [0., c, x],
            3 => [0., x, c],
            4 => [x, 0., c],
            _ => [c, 0., x],
        };
        rgb.map(|v| ((v + self.value - c) * 255.).round().clamp(0., 255.) as u8)
    }
    pub fn hex(&self) -> String {
        let [r, g, b] = self.rgb();
        format!("{r:02X}{g:02X}{b:02X}")
    }
    pub fn pointer(&mut self, control: Control, rect: Rect, p: (f32, f32)) {
        if !p.0.is_finite() || !p.1.is_finite() {
            return;
        }
        let x = ((p.0 - rect.x) / rect.w).clamp(0., 1.);
        match control {
            Control::Hue => self.hue = x,
            Control::Plane => {
                self.saturation = x;
                self.value = 1. - ((p.1 - rect.y) / rect.h).clamp(0., 1.);
            }
        }
    }
    pub fn adjust(&mut self, control: Control, x: i32, y: i32) {
        match control {
            Control::Hue => self.hue = (self.hue + (x - y) as f32 / 360.).rem_euclid(1.),
            Control::Plane => {
                self.saturation = (self.saturation + x as f32 / 100.).clamp(0., 1.);
                self.value = (self.value - y as f32 / 100.).clamp(0., 1.);
            }
        }
    }
}

pub fn rect(width: f32, control: Control) -> Rect {
    let x = ((width - 624.) / 16.).floor() * 8. + 32.;
    match control {
        Control::Plane => Rect::new(x, 238., 320., 196.),
        Control::Hue => Rect::new(x, 470., 320., 22.),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rgb_roundtrip_over_cube_and_all_grays_is_exact() {
        for r in (0..=255).step_by(17) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(17) {
                    assert_eq!(Picker::new([r, g, b]).rgb(), [r, g, b]);
                }
            }
        }
        for v in 0..=255 {
            assert_eq!(Picker::new([v; 3]).rgb(), [v; 3]);
        }
    }
    #[test]
    fn achromatic_drafts_retain_hue_and_can_recover_from_black() {
        let mut p = Picker::new([0, 255, 0]);
        p.sync([0; 3]);
        assert_eq!(p.rgb(), [0; 3]);
        p.adjust(Control::Plane, 0, -100);
        assert_eq!(p.rgb(), [0, 255, 0]);
        p.sync([255; 3]);
        p.adjust(Control::Plane, 100, 0);
        assert_eq!(p.rgb(), [0, 255, 0]);
    }
    #[test]
    fn primary_hues_edges_and_nonfinite_pointer() {
        let mut p = Picker {
            saturation: 1.,
            value: 1.,
            ..Picker::default()
        };
        for (i, rgb) in [
            [255, 0, 0],
            [255, 255, 0],
            [0, 255, 0],
            [0, 255, 255],
            [0, 0, 255],
            [255, 0, 255],
            [255, 0, 0],
        ]
        .into_iter()
        .enumerate()
        {
            p.hue = i as f32 / 6.;
            assert_eq!(p.rgb(), rgb);
        }
        let r = rect(1120., Control::Plane);
        p.pointer(Control::Plane, r, (-1000., -1000.));
        assert_eq!(p.rgb(), [255; 3]);
        p.pointer(Control::Plane, r, (10000., 10000.));
        assert_eq!(p.rgb(), [0; 3]);
        p.pointer(Control::Plane, r, (f32::NAN, 0.));
        assert_eq!(p.rgb(), [0; 3]);
    }
}
