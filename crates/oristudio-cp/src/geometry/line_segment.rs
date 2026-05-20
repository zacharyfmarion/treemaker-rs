use super::line_color::LineColor;
use super::point::Point;
use serde::{Deserialize, Serialize};

const DEFAULT_CUSTOMIZED_COLOR: RgbColor = RgbColor::new(100, 200, 200);

/// Simple RGB color used for Oriedita custom line/circle colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// Oriedita line segment active state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ActiveState {
    #[default]
    Inactive0,
    ActiveA1,
    ActiveB2,
    ActiveBoth3,
}

/// Immutable Oriedita line segment carrier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineSegment {
    pub a: Point,
    pub b: Point,
    pub active: ActiveState,
    pub color: LineColor,
    pub selected: i32,
    pub customized: i32,
    pub customized_color: RgbColor,
}

impl LineSegment {
    pub const fn new(a: Point, b: Point) -> Self {
        Self::with_color(a, b, LineColor::Black0)
    }

    pub const fn with_color(a: Point, b: Point, color: LineColor) -> Self {
        Self::with_color_and_active(a, b, color, ActiveState::Inactive0)
    }

    pub const fn with_color_and_active(
        a: Point,
        b: Point,
        color: LineColor,
        active: ActiveState,
    ) -> Self {
        Self {
            a,
            b,
            active,
            color,
            selected: 0,
            customized: 0,
            customized_color: DEFAULT_CUSTOMIZED_COLOR,
        }
    }

    pub const fn from_coordinates(ax: f64, ay: f64, bx: f64, by: f64) -> Self {
        Self::new(Point::new(ax, ay), Point::new(bx, by))
    }

    pub fn with_coordinates(&self, a: Point, b: Point) -> Self {
        Self { a, b, ..*self }
    }

    pub fn with_swapped_coordinates(&self) -> Self {
        self.with_coordinates(self.b, self.a)
    }

    pub fn with_a(&self, a: Point) -> Self {
        Self { a, ..*self }
    }

    pub fn with_b(&self, b: Point) -> Self {
        Self { b, ..*self }
    }

    pub fn with_line_color(&self, color: LineColor) -> Self {
        Self { color, ..*self }
    }

    pub fn with_active(&self, active: ActiveState) -> Self {
        Self { active, ..*self }
    }

    pub fn with_selected(&self, selected: i32) -> Self {
        Self { selected, ..*self }
    }

    pub fn with_customized_color(&self, customized_color: RgbColor) -> Self {
        Self {
            customized: 1,
            customized_color,
            ..*self
        }
    }

    pub fn determine_max_x(&self) -> i32 {
        (self.a.x.ceil() as i32).max(self.b.x.ceil() as i32)
    }

    pub fn determine_min_x(&self) -> i32 {
        (self.a.x.floor() as i32).min(self.b.x.floor() as i32)
    }

    pub fn determine_max_y(&self) -> i32 {
        (self.a.y.ceil() as i32).max(self.b.y.ceil() as i32)
    }

    pub fn determine_min_y(&self) -> i32 {
        (self.a.y.floor() as i32).min(self.b.y.floor() as i32)
    }

    pub fn determine_closest_endpoint(&self, p: Point) -> Point {
        if p.distance_squared(self.a) <= p.distance_squared(self.b) {
            self.a
        } else {
            self.b
        }
    }

    pub fn determine_furthest_endpoint(&self, p: Point) -> Point {
        if p.distance_squared(self.a) >= p.distance_squared(self.b) {
            self.a
        } else {
            self.b
        }
    }

    pub fn determine_length(&self) -> f64 {
        self.a.distance(self.b)
    }

    pub const fn determine_ax(&self) -> f64 {
        self.a.x
    }

    pub const fn determine_ay(&self) -> f64 {
        self.a.y
    }

    pub const fn determine_bx(&self) -> f64 {
        self.b.x
    }

    pub const fn determine_by(&self) -> f64 {
        self.b.y
    }

    pub fn determine_delta_x(&self) -> f64 {
        self.b.x - self.a.x
    }

    pub fn determine_delta_y(&self) -> f64 {
        self.b.y - self.a.y
    }
}

impl Default for LineSegment {
    fn default() -> Self {
        Self::new(Point::origin(), Point::origin())
    }
}

/// Oriedita segment intersection classification codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum Intersection {
    Error = -1,
    NoIntersection0 = 0,
    Intersects1 = 1,
    IntersectsAux2 = 2,
    IntersectsAux3 = 3,
    IntersectAtPoint4 = 4,
    IntersectAtPointS1_5 = 5,
    IntersectAtPointS2_6 = 6,
    IntersectsLShapeS1StartS2Start21 = 21,
    IntersectsLShapeS1StartS2End22 = 22,
    IntersectsLShapeS1EndS2Start23 = 23,
    IntersectsLShapeS1EndS2End24 = 24,
    IntersectsTShapeS1VerticalBar25 = 25,
    IntersectsTShapeS1VerticalBar26 = 26,
    IntersectsTShapeS2VerticalBar27 = 27,
    IntersectsTShapeS2VerticalBar28 = 28,
    ParallelEqual31 = 31,
    IntersectTA121 = 121,
    IntersectTB122 = 122,
    IntersectTA211 = 211,
    IntersectTB221 = 221,
    ParallelStartOfS1ContainsStartOfS2_321 = 321,
    ParallelStartOfS2ContainsStartOfS1_322 = 322,
    ParallelStartOfS1IntersectsStartOfS2_323 = 323,
    ParallelStartOfS1ContainsEndOfS2_331 = 331,
    ParallelEndOfS2ContainsStartOfS1_332 = 332,
    ParallelStartOfS1IntersectsEndOfS2_333 = 333,
    ParallelEndOfS1ContainsStartOfS2_341 = 341,
    ParallelStartOfS2ContainsEndOfS1_342 = 342,
    ParallelEndOfS1IntersectsStartOfS2_343 = 343,
    ParallelEndOfS1ContainsEndOfS2_351 = 351,
    ParallelEndOfS2ContainsEndOfS1_352 = 352,
    ParallelEndOfS1IntersectsEndOfS2_353 = 353,
    ParallelS1IncludesS2_361 = 361,
    ParallelS1IncludesS2_362 = 362,
    ParallelS2IncludesS1_363 = 363,
    ParallelS2IncludesS1_364 = 364,
    ParallelS1EndOverlapsS2Start371 = 371,
    ParallelS1EndOverlapsS2End372 = 372,
    ParallelS1StartOverlapsS2End373 = 373,
    ParallelS1StartOverlapsS2Start374 = 374,
}

impl Intersection {
    pub const fn state(self) -> i32 {
        self as i32
    }

    pub const fn is_endpoint_intersection(self) -> bool {
        self.state() >= 21 && self.state() <= 28
    }

    pub const fn is_intersection(self) -> bool {
        self.state() >= 1
    }

    pub const fn is_contained_inside(self) -> bool {
        self.state() >= 360
    }

    pub const fn is_overlapping(self) -> bool {
        self.state() >= 30
    }

    pub const fn is_segment_overlapping(self) -> bool {
        matches!(
            self,
            Self::ParallelEqual31
                | Self::ParallelStartOfS1ContainsStartOfS2_321
                | Self::ParallelStartOfS2ContainsStartOfS1_322
                | Self::ParallelStartOfS1ContainsEndOfS2_331
                | Self::ParallelEndOfS2ContainsStartOfS1_332
                | Self::ParallelEndOfS1ContainsStartOfS2_341
                | Self::ParallelStartOfS2ContainsEndOfS1_342
                | Self::ParallelEndOfS1ContainsEndOfS2_351
                | Self::ParallelEndOfS2ContainsEndOfS1_352
                | Self::ParallelS1IncludesS2_361
                | Self::ParallelS1IncludesS2_362
                | Self::ParallelS2IncludesS1_363
                | Self::ParallelS2IncludesS1_364
                | Self::ParallelS1EndOverlapsS2Start371
                | Self::ParallelS1EndOverlapsS2End372
                | Self::ParallelS1StartOverlapsS2End373
                | Self::ParallelS1StartOverlapsS2Start374
        )
    }
}
