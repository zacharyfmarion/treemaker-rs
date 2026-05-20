use super::point::Point;
use super::polygon::Polygon;
use serde::{Deserialize, Serialize};

/// Oriedita rectangle is a four-point polygon in caller-provided order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rectangle {
    polygon: Polygon,
}

impl Rectangle {
    pub fn new(p1: Point, p2: Point, p3: Point, p4: Point) -> Self {
        Self {
            polygon: Polygon::new(vec![p1, p2, p3, p4]),
        }
    }

    pub fn p1(&self) -> Option<Point> {
        self.polygon.vertices.first().copied()
    }

    pub fn p2(&self) -> Option<Point> {
        self.polygon.vertices.get(1).copied()
    }

    pub fn p3(&self) -> Option<Point> {
        self.polygon.vertices.get(2).copied()
    }

    pub fn p4(&self) -> Option<Point> {
        self.polygon.vertices.get(3).copied()
    }

    pub fn as_polygon(&self) -> &Polygon {
        &self.polygon
    }

    pub fn into_polygon(self) -> Polygon {
        self.polygon
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self::new(
            Point::origin(),
            Point::origin(),
            Point::origin(),
            Point::origin(),
        )
    }
}
