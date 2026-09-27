//! Document-space, editor-only freehand guides. They never enter the art raster.
use crate::core::Color;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const MAX_STROKES: usize = 2048;
pub const MAX_POINTS: usize = 65_536;
pub const MAX_STROKE_POINTS: usize = 8192;
pub type Point = [f32; 2];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stroke {
    pub color: Color,
    /// Diameter in cell-width units; coordinates use column and row units.
    pub width: f32,
    pub points: Vec<Point>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub visible: bool,
    pub opacity: f32,
    pub strokes: Arc<Vec<Arc<Stroke>>>,
}
impl Default for Layer {
    fn default() -> Self {
        Self {
            visible: true,
            opacity: 0.65,
            strokes: Arc::new(Vec::new()),
        }
    }
}
impl Layer {
    pub fn point_count(&self) -> usize {
        self.strokes.iter().map(|s| s.points.len()).sum()
    }
    pub fn memory_cost(&self) -> usize {
        self.point_count() * std::mem::size_of::<Point>()
            + self.strokes.len() * std::mem::size_of::<Stroke>()
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.opacity.is_finite() && (0.05..=1.).contains(&self.opacity),
            "Opacité des guides invalide."
        );
        ensure!(
            self.strokes.len() <= MAX_STROKES && self.point_count() <= MAX_POINTS,
            "Calque de guides trop volumineux."
        );
        for s in self.strokes.iter() {
            ensure!(
                s.width.is_finite() && (0.1..=8.).contains(&s.width),
                "Épaisseur de guide invalide."
            );
            ensure!(
                !s.points.is_empty() && s.points.len() <= MAX_STROKE_POINTS,
                "Trait de guide invalide."
            );
            ensure!(
                s.points
                    .iter()
                    .flatten()
                    .all(|p| p.is_finite() && (0.0..=512.).contains(p)),
                "Coordonnée de guide invalide."
            );
        }
        Ok(())
    }
    pub fn start(&mut self, point: Point, color: Color, width: f32) -> bool {
        if self.strokes.len() >= MAX_STROKES || self.point_count() >= MAX_POINTS {
            return false;
        }
        Arc::make_mut(&mut self.strokes).push(Arc::new(Stroke {
            color,
            width,
            points: vec![point],
        }));
        true
    }
    pub fn append(&mut self, point: Point) -> bool {
        if self.point_count() >= MAX_POINTS {
            return false;
        }
        let Some(last) = Arc::make_mut(&mut self.strokes).last_mut() else {
            return false;
        };
        if last.points.len() >= MAX_STROKE_POINTS {
            return false;
        }
        // Sub-cell sampling is independent of zoom and avoids duplicate points.
        if distance_squared(*last.points.last().unwrap(), point) < 0.0025 {
            return true;
        }
        Arc::make_mut(last).points.push(point);
        true
    }
    /// Erase entire intersected strokes, with a swept circular brush.
    pub fn erase(&mut self, a: Point, b: Point, radius: f32) {
        Arc::make_mut(&mut self.strokes).retain(|s| {
            let r2 = (radius + s.width / 2.).powi(2);
            if s.points.len() == 1 {
                return point_segment(s.points[0], a, b) > r2;
            }
            !s.points
                .windows(2)
                .any(|p| segment_distance(a, b, p[0], p[1]) <= r2)
        });
    }
}
fn physical(p: Point) -> Point {
    [p[0], p[1] / crate::typeface::ASPECT]
}
fn distance_squared(a: Point, b: Point) -> f32 {
    let a = physical(a);
    let b = physical(b);
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)
}
fn point_segment(p: Point, a: Point, b: Point) -> f32 {
    let p = physical(p);
    let a = physical(a);
    let b = physical(b);
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy).max(f32::EPSILON))
        .clamp(0., 1.);
    (p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)
}
fn segment_distance(a: Point, b: Point, c: Point, d: Point) -> f32 {
    let cross = |p: Point, q: Point, r: Point| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    // Strict crossing; collinear/disjoint segments use endpoint distances.
    if cross(a, b, c) * cross(a, b, d) < 0. && cross(c, d, a) * cross(c, d, b) < 0. {
        return 0.;
    }
    point_segment(a, c, d)
        .min(point_segment(b, c, d))
        .min(point_segment(c, a, b))
        .min(point_segment(d, a, b))
}
