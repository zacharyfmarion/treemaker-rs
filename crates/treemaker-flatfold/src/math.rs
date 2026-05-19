use crate::{FlatFoldError, Result};

pub(crate) const FLOAT_EPS: f64 = 1.0e-16;

pub(crate) type Point = [f64; 2];

pub(crate) fn near_zero(value: f64) -> bool {
    value.abs() < FLOAT_EPS
}

pub(crate) fn add(a: Point, b: Point) -> Point {
    [a[0] + b[0], a[1] + b[1]]
}

pub(crate) fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}

pub(crate) fn mul(a: Point, scale: f64) -> Point {
    [a[0] * scale, a[1] * scale]
}

pub(crate) fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

pub(crate) fn magsq(a: Point) -> f64 {
    dot(a, a)
}

pub(crate) fn mag(a: Point) -> f64 {
    magsq(a).sqrt()
}

pub(crate) fn unit(a: Point) -> Result<Point> {
    let length = mag(a);
    if near_zero(length) {
        return Err(FlatFoldError::PrecisionFailure(
            "cannot normalize a zero-length vector".to_string(),
        ));
    }
    Ok(mul(a, 1.0 / length))
}

pub(crate) fn perp(a: Point) -> Point {
    [a[1], -a[0]]
}

pub(crate) fn distsq(a: Point, b: Point) -> f64 {
    magsq(sub(b, a))
}

#[allow(dead_code)]
pub(crate) fn close(a: Point, b: Point, eps: f64) -> bool {
    (a[0] - b[0]).abs() < eps && (a[1] - b[1]).abs() < eps
}

pub(crate) fn angle(a: Point) -> f64 {
    let angle = a[1].atan2(a[0]);
    if angle < 0.0 {
        angle + 2.0 * std::f64::consts::PI
    } else {
        angle
    }
}

pub(crate) fn min_line_length(lines: &[[Point; 2]]) -> f64 {
    lines
        .iter()
        .map(|[a, b]| distsq(*a, *b))
        .fold(f64::INFINITY, f64::min)
        .sqrt()
}

pub(crate) fn bounding_box(points: &[Point]) -> Result<[Point; 2]> {
    if points.is_empty() {
        return Err(FlatFoldError::InvalidInput(
            "cannot compute bounding box for empty point set".to_string(),
        ));
    }
    let mut min = [f64::INFINITY, f64::INFINITY];
    let mut max = [f64::NEG_INFINITY, f64::NEG_INFINITY];
    for [x, y] in points {
        if *x < min[0] {
            min[0] = *x;
        }
        if *x > max[0] {
            max[0] = *x;
        }
        if *y < min[1] {
            min[1] = *y;
        }
        if *y > max[1] {
            max[1] = *y;
        }
    }
    Ok([min, max])
}

pub(crate) fn polygon_area2(points: &[Point]) -> f64 {
    if points.is_empty() {
        return 0.0;
    }
    let mut area = 0.0;
    let mut p1 = points[points.len() - 1];
    for p2 in points {
        area += (p1[0] + p2[0]) * (p2[1] - p1[1]);
        p1 = *p2;
    }
    area
}

pub(crate) fn encode(values: &[usize]) -> Vec<u16> {
    let mut out = Vec::with_capacity(values.len());
    for value in values {
        if *value >= 0x8000 {
            out.push(0x8000 + ((value >> 16) as u16));
        }
        out.push(*value as u16);
    }
    out
}

pub(crate) fn encode_order_pair(a: usize, b: usize) -> Vec<u16> {
    if a < b {
        encode(&[a, b])
    } else {
        encode(&[b, a])
    }
}

pub(crate) fn decode(key: &[u16]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < key.len() {
        let mut value = key[i] as usize;
        if value >= 0x8000 {
            i += 1;
            value = ((value - 0x8000) << 16) + key[i] as usize;
        }
        out.push(value);
        i += 1;
    }
    out
}
