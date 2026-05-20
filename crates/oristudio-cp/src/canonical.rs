//! Canonical crease-pattern comparison helpers for oracle tests.

use crate::CreasePatternDocument;
use crate::geometry::{Circle, LineSegment, Point, RgbColor};
use crate::model::{CreasePatternModel, GridMetadata, TextElement};
use serde::{Deserialize, Serialize};

/// Canonical semantic view of a crease-pattern document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalCreasePattern {
    pub title: Option<String>,
    pub lines: Vec<CanonicalLine>,
    pub aux_lines: Vec<CanonicalLine>,
    pub circles: Vec<CanonicalCircle>,
    pub texts: Vec<CanonicalText>,
    pub points: Vec<CanonicalPoint>,
    pub grid: CanonicalGrid,
}

impl CanonicalCreasePattern {
    pub fn from_document(document: &CreasePatternDocument, tolerance: f64) -> Self {
        Self::from_model(document.title.clone(), &document.crease_pattern, tolerance)
    }

    pub fn from_model(title: Option<String>, model: &CreasePatternModel, tolerance: f64) -> Self {
        let mut lines = model
            .line_segments
            .iter()
            .map(|segment| CanonicalLine::from_segment(segment, tolerance))
            .collect::<Vec<_>>();
        lines.sort();

        let mut aux_lines = model
            .aux_line_segments
            .iter()
            .map(|segment| CanonicalLine::from_segment(segment, tolerance))
            .collect::<Vec<_>>();
        aux_lines.sort();

        let mut circles = model
            .circles
            .iter()
            .map(|circle| CanonicalCircle::from_circle(*circle, tolerance))
            .collect::<Vec<_>>();
        circles.sort();

        let mut texts = model
            .texts
            .iter()
            .map(|text| CanonicalText::from_text(text, tolerance))
            .collect::<Vec<_>>();
        texts.sort();

        let mut points = model
            .points
            .iter()
            .map(|point| CanonicalPoint::from_point(*point, tolerance))
            .collect::<Vec<_>>();
        points.sort();

        Self {
            title,
            lines,
            aux_lines,
            circles,
            texts,
            points,
            grid: CanonicalGrid::from_grid(model.grid, tolerance),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalPoint {
    pub x: i64,
    pub y: i64,
}

impl CanonicalPoint {
    pub fn from_point(point: Point, tolerance: f64) -> Self {
        Self {
            x: quantize(point.x, tolerance),
            y: quantize(point.y, tolerance),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalLine {
    pub a: CanonicalPoint,
    pub b: CanonicalPoint,
    pub color: i32,
    pub active: u8,
    pub selected: i32,
    pub customized: i32,
    pub customized_color: Option<CanonicalColor>,
}

impl CanonicalLine {
    pub fn from_segment(segment: &LineSegment, tolerance: f64) -> Self {
        let mut a = CanonicalPoint::from_point(segment.a, tolerance);
        let mut b = CanonicalPoint::from_point(segment.b, tolerance);
        if b < a {
            std::mem::swap(&mut a, &mut b);
        }

        Self {
            a,
            b,
            color: segment.color.number(),
            active: active_state_code(segment.active),
            selected: segment.selected,
            customized: segment.customized,
            customized_color: (segment.customized == 1)
                .then_some(CanonicalColor::from_rgb(segment.customized_color)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalCircle {
    pub center: CanonicalPoint,
    pub radius: i64,
    pub color: i32,
    pub customized: i32,
    pub customized_color: Option<CanonicalColor>,
}

impl CanonicalCircle {
    pub fn from_circle(circle: Circle, tolerance: f64) -> Self {
        Self {
            center: CanonicalPoint::from_point(circle.determine_center(), tolerance),
            radius: quantize(circle.r, tolerance),
            color: circle.color.number(),
            customized: circle.customized,
            customized_color: (circle.customized == 1)
                .then_some(CanonicalColor::from_rgb(circle.customized_color)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalText {
    pub position: CanonicalPoint,
    pub text: String,
}

impl CanonicalText {
    pub fn from_text(text: &TextElement, tolerance: f64) -> Self {
        Self {
            position: CanonicalPoint::from_point(text.position(), tolerance),
            text: text.text.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalGrid {
    pub interval_grid_size: i32,
    pub grid_size: i32,
    pub grid_xa: i64,
    pub grid_xb: i64,
    pub grid_xc: i64,
    pub grid_ya: i64,
    pub grid_yb: i64,
    pub grid_yc: i64,
    pub grid_angle: i64,
    pub base_state: i32,
    pub vertical_scale_position: i32,
    pub horizontal_scale_position: i32,
    pub draw_diagonal_gridlines: bool,
}

impl CanonicalGrid {
    pub fn from_grid(grid: GridMetadata, tolerance: f64) -> Self {
        Self {
            interval_grid_size: grid.interval_grid_size,
            grid_size: grid.grid_size,
            grid_xa: quantize(grid.grid_xa, tolerance),
            grid_xb: quantize(grid.grid_xb, tolerance),
            grid_xc: quantize(grid.grid_xc, tolerance),
            grid_ya: quantize(grid.grid_ya, tolerance),
            grid_yb: quantize(grid.grid_yb, tolerance),
            grid_yc: quantize(grid.grid_yc, tolerance),
            grid_angle: quantize(grid.grid_angle, tolerance),
            base_state: grid.base_state.state(),
            vertical_scale_position: grid.vertical_scale_position,
            horizontal_scale_position: grid.horizontal_scale_position,
            draw_diagonal_gridlines: grid.draw_diagonal_gridlines,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CanonicalColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl CanonicalColor {
    pub const fn from_rgb(color: RgbColor) -> Self {
        Self {
            red: color.red,
            green: color.green,
            blue: color.blue,
        }
    }
}

fn active_state_code(active: crate::geometry::ActiveState) -> u8 {
    match active {
        crate::geometry::ActiveState::Inactive0 => 0,
        crate::geometry::ActiveState::ActiveA1 => 1,
        crate::geometry::ActiveState::ActiveB2 => 2,
        crate::geometry::ActiveState::ActiveBoth3 => 3,
    }
}

fn quantize(value: f64, tolerance: f64) -> i64 {
    let tolerance = if tolerance.is_finite() && tolerance > 0.0 {
        tolerance
    } else {
        1.0e-9
    };
    (value / tolerance).round() as i64
}
