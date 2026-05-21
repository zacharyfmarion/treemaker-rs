//! Arrangement and cleanup helpers ported from Oriedita `FoldLineSet` workers.

use crate::geometry::{
    Epsilon, Intersection, LineColor, LineSegment, Point, StraightLine, StraightLineIntersection,
    determine_line_segment_intersection, determine_line_segment_intersection_sweet,
    determine_line_segment_intersection_with_precision, find_intersection_segments,
    find_intersection_straight_lines, find_projection, is_line_segment_overlapping,
    line_segment_to_straight_line, line_segment_x_kousa_decide,
};
use crate::model::CreasePatternModel;
use std::collections::BTreeSet;

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

/// Divide newly added lines against the existing crease-pattern lines.
///
/// `original_end` and `added_end` are counts in the same role as Oriedita's
/// one-based inclusive parameters: lines before `original_end` are existing
/// lines, lines in `original_end..added_end` are newly inserted lines.
pub fn divide_line_segment_with_new_lines(
    model: &mut CreasePatternModel,
    original_end: usize,
    added_end: usize,
) {
    let original_end = original_end.min(model.line_segments.len());
    let added_end = added_end.min(model.line_segments.len());
    let mut flags = vec![0u8; model.line_segments.len() + 101];

    for flag in flags.iter_mut().take(original_end) {
        *flag = 1;
    }
    for flag in flags.iter_mut().take(added_end).skip(original_end) {
        *flag = 2;
    }

    let mut to_delete = BTreeSet::new();
    let mut i = original_end;
    while i < model.line_segments.len() {
        if flag_at(&flags, i) == 2 {
            let scan_len = model.line_segments.len();
            for j in 0..scan_len {
                if flag_at(&flags, j) != 1 || i == j {
                    continue;
                }

                let result = divide_intersections_fast(model, i, j, &mut to_delete);
                ensure_flags_len(&mut flags, model.line_segments.len());
                let total = model.line_segments.len();

                match result {
                    Intersection::Intersects1 => {
                        flags[total - 2] = 2;
                        flags[total - 1] = 1;
                    }
                    Intersection::IntersectsAux2
                    | Intersection::IntersectTA211
                    | Intersection::IntersectTB221 => {
                        flags[total - 1] = 2;
                    }
                    Intersection::IntersectsAux3
                    | Intersection::IntersectTA121
                    | Intersection::IntersectTB122
                    | Intersection::ParallelS2IncludesS1_363
                    | Intersection::ParallelS2IncludesS1_364 => {
                        flags[total - 1] = 1;
                    }
                    Intersection::ParallelS1IncludesS2_361
                    | Intersection::ParallelS1IncludesS2_362 => {
                        flags[j] = 0;
                        flags[total - 1] = 2;
                    }
                    Intersection::ParallelS1EndOverlapsS2Start371
                    | Intersection::ParallelS1StartOverlapsS2End373
                    | Intersection::ParallelS1EndOverlapsS2End372
                    | Intersection::ParallelS1StartOverlapsS2Start374 => {
                        flags[total - 1] = 0;
                    }
                    _ => {}
                }
            }
        }

        i += 1;
    }

    for index in to_delete.into_iter().rev() {
        if index < model.line_segments.len() {
            model.line_segments.remove(index);
        }
    }
}

/// Oriedita `FoldLineSet.divideIntersectionsFast` for one new/existing pair.
pub fn divide_intersections_fast(
    model: &mut CreasePatternModel,
    i: usize,
    j: usize,
    indices_to_delete: &mut BTreeSet<usize>,
) -> Intersection {
    if i == j || i >= model.line_segments.len() || j >= model.line_segments.len() {
        return Intersection::NoIntersection0;
    }

    let si = model.line_segments[i].clone();
    let sj = model.line_segments[j].clone();

    if si.determine_max_x() < sj.determine_min_x()
        || sj.determine_max_x() < si.determine_min_x()
        || si.determine_max_y() < sj.determine_min_y()
        || sj.determine_max_y() < si.determine_min_y()
    {
        return Intersection::NoIntersection0;
    }

    let straight_line0 = StraightLine::from_points(si.a, si.b);
    let intersect_flag0 = straight_line0.line_segment_intersect_reverse_detail(&sj);
    if intersect_flag0 == StraightLineIntersection::None0 {
        return Intersection::NoIntersection0;
    }

    let straight_line1 = StraightLine::from_points(sj.a, sj.b);
    let intersect_flag1 = straight_line1.line_segment_intersect_reverse_detail(&si);
    if intersect_flag1 == StraightLineIntersection::None0 {
        return Intersection::NoIntersection0;
    }

    if intersect_flag0 == StraightLineIntersection::IntersectX1
        && intersect_flag1 == StraightLineIntersection::IntersectX1
    {
        let intersection_point = find_intersection_straight_lines(straight_line0, straight_line1);
        if same_aux_class(si.color, sj.color) {
            add_line_like(model, intersection_point, si.b, &si);
            set_segment(model, i, si.with_b(intersection_point));
            add_line_like(model, intersection_point, sj.b, &sj);
            set_segment(model, j, sj.with_b(intersection_point));
            return Intersection::Intersects1;
        }
        if si.color == LineColor::Cyan3 && sj.color != LineColor::Cyan3 {
            add_line_like(model, intersection_point, si.b, &si);
            set_segment(model, i, si.with_b(intersection_point));
            return Intersection::IntersectsAux2;
        }
        if si.color != LineColor::Cyan3 && sj.color == LineColor::Cyan3 {
            add_line_like(model, intersection_point, sj.b, &sj);
            set_segment(model, j, sj.with_b(intersection_point));
            return Intersection::IntersectsAux3;
        }
    }

    if intersect_flag0 == StraightLineIntersection::IntersectX1
        && intersect_flag1 == StraightLineIntersection::IntersectTA21
    {
        let intersection_point = find_projection(line_segment_to_straight_line(&sj), si.a);
        if same_aux_class(si.color, sj.color)
            || (si.color != LineColor::Cyan3 && sj.color == LineColor::Cyan3)
        {
            add_line_like(model, intersection_point, sj.b, &sj);
            set_segment(model, j, sj.with_b(intersection_point));
            return Intersection::IntersectTA121;
        }
        return Intersection::NoIntersection0;
    }

    if intersect_flag0 == StraightLineIntersection::IntersectX1
        && intersect_flag1 == StraightLineIntersection::IntersectTB22
    {
        let intersection_point = find_projection(line_segment_to_straight_line(&sj), si.b);
        if same_aux_class(si.color, sj.color)
            || (si.color != LineColor::Cyan3 && sj.color == LineColor::Cyan3)
        {
            add_line_like(model, intersection_point, sj.b, &sj);
            set_segment(model, j, sj.with_b(intersection_point));
            return Intersection::IntersectTB122;
        }
        return Intersection::NoIntersection0;
    }

    if intersect_flag0 == StraightLineIntersection::IntersectTA21
        && intersect_flag1 == StraightLineIntersection::IntersectX1
    {
        let intersection_point = find_projection(line_segment_to_straight_line(&si), sj.a);
        if same_aux_class(si.color, sj.color)
            || (si.color == LineColor::Cyan3 && sj.color != LineColor::Cyan3)
        {
            add_line_like(model, intersection_point, si.b, &si);
            set_segment(model, i, si.with_b(intersection_point));
            return Intersection::IntersectTA211;
        }
        return Intersection::NoIntersection0;
    }

    if intersect_flag0 == StraightLineIntersection::IntersectTB22
        && intersect_flag1 == StraightLineIntersection::IntersectX1
    {
        let intersection_point = find_projection(line_segment_to_straight_line(&si), sj.b);
        if same_aux_class(si.color, sj.color)
            || (si.color == LineColor::Cyan3 && sj.color != LineColor::Cyan3)
        {
            add_line_like(model, intersection_point, si.b, &si);
            set_segment(model, i, si.with_b(intersection_point));
            return Intersection::IntersectTB221;
        }
        return Intersection::NoIntersection0;
    }

    if intersect_flag0 == StraightLineIntersection::Included3 {
        let p1 = si.a;
        let p2 = si.b;
        let p3 = sj.a;
        let p4 = sj.b;
        let intersection =
            determine_line_segment_intersection_with_precision(&si, &sj, Epsilon::UNKNOWN_1EN6);

        match intersection {
            Intersection::ParallelEqual31 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                indices_to_delete.insert(j);
                return Intersection::ParallelEqual31;
            }
            Intersection::ParallelStartOfS1ContainsStartOfS2_321 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_color(model, j, si.color);
                set_segment(model, i, si.with_a(sj.b));
                return intersection;
            }
            Intersection::ParallelStartOfS2ContainsStartOfS1_322 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_segment(model, j, sj.with_a(si.b));
                return intersection;
            }
            Intersection::ParallelStartOfS1ContainsEndOfS2_331 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_color(model, j, si.color);
                set_segment(model, i, si.with_a(sj.a));
                return intersection;
            }
            Intersection::ParallelEndOfS2ContainsStartOfS1_332 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_segment(model, j, sj.with_b(si.b));
                return intersection;
            }
            Intersection::ParallelEndOfS1ContainsStartOfS2_341 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_color(model, j, si.color);
                set_segment(model, i, si.with_b(sj.b));
                return intersection;
            }
            Intersection::ParallelStartOfS2ContainsEndOfS1_342 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_segment(model, j, sj.with_a(si.a));
                return intersection;
            }
            Intersection::ParallelEndOfS1ContainsEndOfS2_351 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_color(model, j, si.color);
                set_segment(model, i, si.with_b(sj.a));
                return intersection;
            }
            Intersection::ParallelEndOfS2ContainsEndOfS1_352 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_segment(model, j, sj.with_b(si.a));
                return intersection;
            }
            Intersection::ParallelS1IncludesS2_361 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_color(model, j, si.color);
                add_line_like(model, sj.b, si.b, &si);
                set_segment(model, i, si.with_b(sj.a));
                return intersection;
            }
            Intersection::ParallelS1IncludesS2_362 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                set_color(model, j, si.color);
                add_line_like(model, sj.a, si.b, &si);
                set_segment(model, i, si.with_b(sj.b));
                return intersection;
            }
            Intersection::ParallelS2IncludesS1_363 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                add_line_like(model, si.b, sj.b, &sj);
                set_segment(model, j, sj.with_b(si.a));
                return intersection;
            }
            Intersection::ParallelS2IncludesS1_364 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                add_line_like(model, si.a, sj.b, &sj);
                set_segment(model, j, sj.with_b(si.b));
                return intersection;
            }
            Intersection::ParallelS1EndOverlapsS2Start371 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                add_line_like(model, p3, p2, &si);
                set_segment(model, i, si.with_b(p3));
                set_segment(model, j, sj.with_a(p2));
                return intersection;
            }
            Intersection::ParallelS1EndOverlapsS2End372 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                add_line_like(model, p4, p2, &si);
                set_segment(model, i, si.with_b(p4));
                set_segment(model, j, sj.with_b(p2));
                return intersection;
            }
            Intersection::ParallelS1StartOverlapsS2End373 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                add_line_like(model, p1, p4, &si);
                set_segment(model, i, si.with_a(p4));
                set_segment(model, j, sj.with_b(p1));
                return intersection;
            }
            Intersection::ParallelS1StartOverlapsS2Start374 => {
                if has_cyan_mismatch(si.color, sj.color) {
                    return Intersection::NoIntersection0;
                }
                add_line_like(model, p1, p3, &si);
                set_segment(model, i, si.with_a(p3));
                set_segment(model, j, sj.with_a(p1));
                return intersection;
            }
            _ => {}
        }
    }

    Intersection::NoIntersection0
}

/// Delete lines that overlap the supplied drag/selection segment.
pub fn delete_overlapping_lines_along(
    model: &mut CreasePatternModel,
    selection: &LineSegment,
) -> bool {
    delete_inside_line(model, selection, false)
}

/// Delete lines that overlap or X-intersect the supplied selection segment.
pub fn delete_intersecting_or_overlapping_lines_along(
    model: &mut CreasePatternModel,
    selection: &LineSegment,
) -> bool {
    delete_inside_line(model, selection, true)
}

/// Remove dangling branch lines and merge straight same-color vertices.
pub fn branch_trim(model: &mut CreasePatternModel) {
    let radius = Epsilon::UNKNOWN_1EN6;
    let mut i_one_based = 1;

    while i_one_based <= model.line_segments.len() {
        let index = i_one_based - 1;
        let segment = model.line_segments[index].clone();
        let mut has_a_connection = false;
        let mut has_b_connection = false;

        for (other_index, other) in model.line_segments.iter().enumerate() {
            if index == other_index {
                continue;
            }

            if segment.a.distance(other.a) < radius || segment.a.distance(other.b) < radius {
                has_a_connection = true;
            }
            if segment.b.distance(other.a) < radius || segment.b.distance(other.b) < radius {
                has_b_connection = true;
            }
        }

        if !has_a_connection || !has_b_connection {
            delete_line_segment_vertex(model, index);
            i_one_based = 2;
        } else {
            i_one_based += 1;
        }
    }
}

/// Delete a line and apply Oriedita's same-color straight vertex cleanup.
pub fn delete_line_segment_vertex(model: &mut CreasePatternModel, index: usize) {
    if index >= model.line_segments.len() {
        return;
    }

    let segment = model.line_segments[index].clone();
    model.line_segments.remove(index);

    let _ = del_v_at_point(
        model,
        segment.a,
        Epsilon::UNKNOWN_1EN6,
        Epsilon::UNKNOWN_1EN6,
    );
    let _ = del_v_at_point(
        model,
        segment.b,
        Epsilon::UNKNOWN_1EN6,
        Epsilon::UNKNOWN_1EN6,
    );
}

/// Oriedita `FoldLineSet.del_V(Point, hikiyose, r)`.
///
/// The Java method returns `false` even after a successful merge; this function
/// preserves that observable return value.
pub fn del_v_at_point(
    model: &mut CreasePatternModel,
    point: Point,
    snap_radius: f64,
    vertex_radius: f64,
) -> bool {
    let q = closest_endpoint(model, point);
    if q.distance_squared(point) > snap_radius * snap_radius {
        return false;
    }

    let adjacent = vertex_surrounding_line_indices(model, q, vertex_radius);
    if adjacent.len() != 2 {
        return false;
    }

    let ix = adjacent[0];
    let iy = adjacent[1];
    let lix = model.line_segments[ix].clone();
    let liy = model.line_segments[iy].clone();
    let intersection =
        determine_line_segment_intersection_with_precision(&lix, &liy, Epsilon::UNKNOWN_1EN6);

    let Some((a, b)) = del_v_merge_endpoints(&lix, &liy, intersection) else {
        return false;
    };

    if lix.color != liy.color {
        return false;
    }

    let (first_delete, second_delete) = if ix > iy { (ix, iy) } else { (iy, ix) };
    model.line_segments.remove(first_delete);
    model.line_segments.remove(second_delete);
    model.add_line_segment(LineSegment::with_color(a, b, lix.color));

    false
}

fn delete_inside_line(
    model: &mut CreasePatternModel,
    selection: &LineSegment,
    include_crossing: bool,
) -> bool {
    let original_len = model.line_segments.len();
    model.line_segments.retain(|segment| {
        let should_delete = is_line_segment_overlapping(segment, selection)
            || (include_crossing && line_segment_x_kousa_decide(segment, selection));
        !should_delete
    });
    model.line_segments.len() != original_len
}

fn closest_endpoint(model: &CreasePatternModel, point: Point) -> Point {
    let mut closest = Point::new(100000.0, 100000.0);
    for segment in &model.line_segments {
        if point.distance_squared(segment.a) < point.distance_squared(closest) {
            closest = segment.a;
        }
        if point.distance_squared(segment.b) < point.distance_squared(closest) {
            closest = segment.b;
        }
    }
    closest
}

fn vertex_surrounding_line_indices(
    model: &CreasePatternModel,
    point: Point,
    radius: f64,
) -> Vec<usize> {
    let q = closest_endpoint(model, point);
    let mut indices = Vec::new();
    for (index, segment) in model.line_segments.iter().enumerate() {
        let closest = if q.distance_squared(segment.b) < q.distance_squared(segment.a) {
            segment.b
        } else {
            segment.a
        };
        if q.distance_squared(closest) < radius * radius {
            indices.push(index);
        }
    }
    indices
}

fn del_v_merge_endpoints(
    first: &LineSegment,
    second: &LineSegment,
    intersection: Intersection,
) -> Option<(Point, Point)> {
    match intersection {
        Intersection::ParallelStartOfS1IntersectsStartOfS2_323 => Some((first.b, second.b)),
        Intersection::ParallelStartOfS1IntersectsEndOfS2_333 => Some((first.b, second.a)),
        Intersection::ParallelEndOfS1IntersectsStartOfS2_343 => Some((first.a, second.b)),
        Intersection::ParallelEndOfS1IntersectsEndOfS2_353 => Some((first.a, second.a)),
        _ => None,
    }
}

fn flag_at(flags: &[u8], index: usize) -> u8 {
    flags.get(index).copied().unwrap_or_default()
}

fn ensure_flags_len(flags: &mut Vec<u8>, len: usize) {
    if flags.len() < len + 101 {
        flags.resize(len + 101, 0);
    }
}

fn same_aux_class(first: LineColor, second: LineColor) -> bool {
    (first == LineColor::Cyan3 && second == LineColor::Cyan3)
        || (first != LineColor::Cyan3 && second != LineColor::Cyan3)
}

fn has_cyan_mismatch(first: LineColor, second: LineColor) -> bool {
    (first == LineColor::Cyan3 && second != LineColor::Cyan3)
        || (first != LineColor::Cyan3 && second == LineColor::Cyan3)
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

fn add_line_like(model: &mut CreasePatternModel, a: Point, b: Point, template: &LineSegment) {
    model.add_line_segment(template.with_coordinates(a, b));
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
