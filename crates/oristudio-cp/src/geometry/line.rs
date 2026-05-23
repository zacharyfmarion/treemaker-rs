use super::line_color::LineColor;
use serde::{Deserialize, Serialize};

/// PointSet line edge, referring to begin/end point indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Line {
    pub begin: i32,
    pub end: i32,
    pub color: LineColor,
}

impl Line {
    pub const fn new(begin: i32, end: i32, color: LineColor) -> Self {
        Self { begin, end, color }
    }

    pub const fn reset(self) -> Self {
        let _ = self;
        Self::new(0, 0, LineColor::Black0)
    }

    pub const fn with_begin(self, begin: i32) -> Self {
        Self { begin, ..self }
    }

    pub const fn with_end(self, end: i32) -> Self {
        Self { end, ..self }
    }

    pub const fn with_color(self, color: LineColor) -> Self {
        Self { color, ..self }
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::new(0, 0, LineColor::Black0)
    }
}
