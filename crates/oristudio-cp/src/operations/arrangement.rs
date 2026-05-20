//! Arrangement and cleanup helpers ported from Oriedita `FoldLineSet` workers.

use crate::geometry::{
    Intersection, LineColor, LineSegment, Point, determine_line_segment_intersection,
    determine_line_segment_intersection_sweet, determine_line_segment_intersection_with_precision,
    find_intersection_segments,
};
use crate::model::CreasePatternModel;

/// Oriedita sentinel used by `FoldLineSet.removeOverlappingLines()`.
const DEFAULT_PRECISION_SENTINEL: f64 = -9999.9;
const INTERSECT_DIVIDE_BOUNDS_EPSILON: f64 = 0.05;

/// Divide line segments at intersections and overlapping spans.
///
/// This ports the standalone `IntersectDivide` worker semantics. The upstream
/// implementation uses a quadtree to find possible collisions; this Rust port
/// preserves pair mutation behavior with a direct dynamic scan.
pub fn divide_intersections(model: &mut CreasePatternModel) {
    let mut i = 0;
    while i < model.line_segments.len() {
        let scan_len = model.line_segments.len();
        for j in 0..scan_len {
            let _ = intersect_divide_pair(model, i, j);
        }
        i += 1;
    }
}

/// Divide a single pair of line segments.
///
/// Returns the number of lines added, or `-1` when Oriedita would do nothing.
pub fn intersect_divide_pair(model: &mut CreasePatternModel, i: usize, j: usize) -> i32 {
    if i == j || i >= model.line_segments.len() || j >= model.line_segments.len() {
        return -1;
    }

    let si = model.line_segments[i].clone();
    let sj = model.line_segments[j].clone();

    if si.determine_ax().max(si.determine_bx()) + INTERSECT_DIVIDE_BOUNDS_EPSILON
        < sj.determine_ax().min(sj.determine_bx())
        || sj.determine_ax().max(sj.determine_bx()) + INTERSECT_DIVIDE_BOUNDS_EPSILON
            < si.determine_ax().min(si.determine_bx())
        || si.determine_ay().max(si.determine_by()) + INTERSECT_DIVIDE_BOUNDS_EPSILON
            < sj.determine_ay().min(sj.determine_by())
        || sj.determine_ay().max(sj.determine_by()) + INTERSECT_DIVIDE_BOUNDS_EPSILON
            < si.determine_ay().min(si.determine_by())
    {
        return -1;
    }

    let intersection = determine_line_segment_intersection_sweet(&si, &sj);
    let p1 = si.a;
    let p2 = si.b;
    let p3 = sj.a;
    let p4 = sj.b;

    match intersection {
        Intersection::Intersects1 => {
            let pk = find_intersection_segments(&si, &sj);
            set_segment(model, i, si.with_coordinates(p1, pk));
            set_segment(model, j, sj.with_coordinates(p3, pk));
            add_line(model, p2, pk, si.color);
            add_line(model, p4, pk, sj.color);
            2
        }
        Intersection::IntersectsTShapeS1VerticalBar25
        | Intersection::IntersectsTShapeS1VerticalBar26 => {
            let pk = find_intersection_segments(&si, &sj);
            set_segment(model, j, sj.with_coordinates(p3, pk));
            add_line(model, p4, pk, sj.color);
            1
        }
        Intersection::IntersectsTShapeS2VerticalBar27
        | Intersection::IntersectsTShapeS2VerticalBar28 => {
            let pk = find_intersection_segments(&si, &sj);
            set_segment(model, i, si.with_coordinates(p1, pk));
            add_line(model, p2, pk, si.color);
            1
        }
        Intersection::ParallelEqual31 => -1,
        Intersection::ParallelStartOfS1ContainsStartOfS2_321 => {
            set_segment(model, i, si.with_coordinates(p2, p4));
            set_color(model, j, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelStartOfS2ContainsStartOfS1_322 => {
            set_segment(model, j, sj.with_coordinates(p2, p4));
            set_color(model, i, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelStartOfS1ContainsEndOfS2_331 => {
            set_segment(model, i, si.with_coordinates(p2, p3));
            set_color(model, j, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelEndOfS2ContainsStartOfS1_332 => {
            set_segment(model, j, sj.with_coordinates(p2, p3));
            set_color(model, i, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelEndOfS1ContainsStartOfS2_341 => {
            set_segment(model, i, si.with_coordinates(p1, p4));
            set_color(model, j, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelStartOfS2ContainsEndOfS1_342 => {
            set_segment(model, j, sj.with_coordinates(p1, p4));
            set_color(model, i, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelEndOfS1ContainsEndOfS2_351 => {
            set_segment(model, i, si.with_coordinates(p1, p3));
            set_color(model, j, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelEndOfS2ContainsEndOfS1_352 => {
            set_segment(model, j, sj.with_coordinates(p1, p3));
            set_color(model, i, overlapping_color(i, j, si.color, sj.color));
            0
        }
        Intersection::ParallelS1IncludesS2_361 => {
            set_segment(model, i, si.with_coordinates(p1, p3));
            add_line(model, p2, p4, si.color);
            set_color(model, j, overlapping_color(i, j, si.color, sj.color));
            1
        }
        Intersection::ParallelS1IncludesS2_362 => {
            set_segment(model, i, si.with_coordinates(p1, p4));
            add_line(model, p2, p3, si.color);
            set_color(model, j, overlapping_color(i, j, si.color, sj.color));
            1
        }
        Intersection::ParallelS2IncludesS1_363 => {
            set_segment(model, j, sj.with_coordinates(p1, p3));
            add_line(model, p2, p4, sj.color);
            set_color(model, i, overlapping_color(i, j, si.color, sj.color));
            1
        }
        Intersection::ParallelS2IncludesS1_364 => {
            set_segment(model, j, sj.with_coordinates(p1, p4));
            add_line(model, p2, p3, sj.color);
            set_color(model, i, overlapping_color(i, j, si.color, sj.color));
            1
        }
        Intersection::ParallelS1EndOverlapsS2Start371 => {
            set_segment(model, i, si.with_coordinates(p1, p3));
            set_segment(model, j, sj.with_coordinates(p2, p4));
            add_line(model, p2, p3, overlapping_color(i, j, si.color, sj.color));
            1
        }
        Intersection::ParallelS1EndOverlapsS2End372 => {
            set_segment(model, i, si.with_coordinates(p1, p4));
            set_segment(model, j, sj.with_coordinates(p3, p2));
            add_line(model, p2, p4, overlapping_color(i, j, si.color, sj.color));
            1
        }
        Intersection::ParallelS1StartOverlapsS2End373 => {
            set_segment(model, i, si.with_coordinates(p2, p4));
            set_segment(model, j, sj.with_coordinates(p1, p3));
            add_line(model, p1, p4, overlapping_color(i, j, si.color, sj.color));
            1
        }
        Intersection::ParallelS1StartOverlapsS2Start374 => {
            set_segment(model, i, si.with_coordinates(p3, p2));
            set_segment(model, j, sj.with_coordinates(p1, p4));
            add_line(model, p1, p3, overlapping_color(i, j, si.color, sj.color));
            1
        }
        _ => -1,
    }
}

fn set_segment(model: &mut CreasePatternModel, index: usize, segment: LineSegment) {
    model.line_segments[index] = segment;
}

fn set_color(model: &mut CreasePatternModel, index: usize, color: LineColor) {
    model.line_segments[index] = model.line_segments[index].with_line_color(color);
}

fn add_line(model: &mut CreasePatternModel, a: Point, b: Point, color: LineColor) {
    model.add_line_segment(LineSegment::with_color(a, b, color));
}

fn overlapping_color(i: usize, j: usize, si_color: LineColor, sj_color: LineColor) -> LineColor {
    if i < j { sj_color } else { si_color }
}

/// Remove duplicate overlapping line segments in Oriedita's order.
///
/// This ports the observable behavior of `FoldLineSet.removeOverlappingLines`:
/// when two line segments classify as `PARALLEL_EQUAL_31`, the later line is
/// removed and the earlier line survives. The upstream implementation uses
/// spatial acceleration; this first Rust port intentionally keeps the same
/// pair-order semantics with a direct scan so correctness is visible.
pub fn remove_overlapping_lines(model: &mut CreasePatternModel) {
    remove_overlapping_lines_with_precision(model, DEFAULT_PRECISION_SENTINEL);
}

/// Remove duplicate overlapping line segments with Oriedita's optional radius.
pub fn remove_overlapping_lines_with_precision(model: &mut CreasePatternModel, radius: f64) {
    let mut remove = vec![false; model.line_segments.len()];

    let len = model.line_segments.len();
    for i in 0..len.saturating_sub(1) {
        for (j, remove_j) in remove.iter_mut().enumerate().take(len).skip(i + 1) {
            let intersection = if radius <= DEFAULT_PRECISION_SENTINEL {
                determine_line_segment_intersection(
                    &model.line_segments[i],
                    &model.line_segments[j],
                )
            } else {
                determine_line_segment_intersection_with_precision(
                    &model.line_segments[i],
                    &model.line_segments[j],
                    radius,
                )
            };

            if intersection == Intersection::ParallelEqual31 {
                *remove_j = true;
            }
        }
    }

    model.line_segments = model
        .line_segments
        .iter()
        .enumerate()
        .filter_map(|(index, segment)| (!remove[index]).then_some(segment.clone()))
        .collect();
}
