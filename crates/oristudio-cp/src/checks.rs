//! Oriedita-compatible crease-pattern diagnostic helpers.

use crate::geometry::{
    Epsilon, Intersection, LineColor, LineSegment, Point, angle, angle_between_0_360,
    angle_between_0_kmax, determine_line_segment_intersection_sweet_with_tolerances,
    determine_line_segment_intersection_with_precision,
    determine_line_segment_intersection_with_tolerances, equal_with_radius,
    find_intersection_segments, find_line_symmetry_line_segment,
};
use crate::model::CreasePatternModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatFoldableBoundaryCheck {
    pub color: LineColor,
    pub suitable_intersections: bool,
    pub crossing_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatFoldabilityRule {
    NumberOfFolds,
    Angles,
    Maekawa,
    LittleBigLittle,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlatFoldabilityColor {
    NotEnoughMountain,
    NotEnoughValley,
    Equal,
    Correct,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LittleBigLittleSegment {
    pub segment: LineSegment,
    pub violating: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatFoldabilityViolation {
    pub point: Point,
    pub rule: FlatFoldabilityRule,
    pub color: FlatFoldabilityColor,
    pub little_big_little: Vec<LittleBigLittleSegment>,
}

impl FlatFoldabilityViolation {
    pub fn new(point: Point, rule: FlatFoldabilityRule, color: FlatFoldabilityColor) -> Self {
        Self {
            point,
            rule,
            color,
            little_big_little: Vec::new(),
        }
    }

    pub fn little_big_little(point: Point, little_big_little: Vec<LittleBigLittleSegment>) -> Self {
        Self {
            point,
            rule: FlatFoldabilityRule::LittleBigLittle,
            color: FlatFoldabilityColor::Correct,
            little_big_little,
        }
    }
}

/// Oriedita `Check1.apply`: collect non-auxiliary overlapping/contained line pairs.
pub fn check1(model: &CreasePatternModel) -> Vec<LineSegment> {
    let mut diagnostics = Vec::new();

    for i in 0..model.line_segments.len() {
        let si = &model.line_segments[i];
        if si.color == LineColor::Cyan3 {
            continue;
        }

        for sj in &model.line_segments[..i] {
            if sj.color == LineColor::Cyan3 {
                continue;
            }

            let intersection = determine_line_segment_intersection_with_tolerances(
                si,
                sj,
                Epsilon::UNKNOWN_0001,
                Epsilon::PARALLEL_FOR_FIX,
            );
            if check1_intersection_matches(intersection) || intersection.is_contained_inside() {
                diagnostics.push(si.clone());
                diagnostics.push(sj.clone());
            }
        }
    }

    diagnostics
}

/// Oriedita `Check2.apply`: collect non-auxiliary near-T intersection pairs.
pub fn check2(model: &CreasePatternModel) -> Vec<LineSegment> {
    let mut diagnostics = Vec::new();

    for i in 0..model.line_segments.len() {
        let si = &model.line_segments[i];
        if si.color == LineColor::Cyan3 {
            continue;
        }

        for sj in &model.line_segments[..i] {
            if sj.color == LineColor::Cyan3 {
                continue;
            }

            let intersection = determine_line_segment_intersection_sweet_with_tolerances(
                si,
                sj,
                Epsilon::UNKNOWN_0001,
                Epsilon::PARALLEL_FOR_FIX,
            );
            if matches!(
                intersection,
                Intersection::IntersectsTShapeS1VerticalBar25
                    | Intersection::IntersectsTShapeS1VerticalBar26
                    | Intersection::IntersectsTShapeS2VerticalBar27
                    | Intersection::IntersectsTShapeS2VerticalBar28
            ) {
                diagnostics.push(si.clone());
                diagnostics.push(sj.clone());
            }
        }
    }

    diagnostics
}

/// Oriedita `Check3.apply`: collect legacy vertex flat-foldability markers.
pub fn check3(model: &CreasePatternModel) -> Vec<LineSegment> {
    let mut diagnostics = Vec::new();
    for segment in &model.line_segments {
        if segment.color == LineColor::Cyan3 {
            continue;
        }

        check3_point(model, segment.a, &mut diagnostics);
        check3_point(model, segment.b, &mut diagnostics);
    }

    diagnostics
}

/// Oriedita `Check4.apply`: collect CAMV and little-big-little violations.
pub fn check4(model: &CreasePatternModel) -> Vec<FlatFoldabilityViolation> {
    point_line_map(model)
        .into_iter()
        .filter_map(|(point, lines)| find_flat_foldability_violation(point, &lines))
        .collect()
}

/// Oriedita `Check4.findFlatfoldabilityViolation` for one point and its incident lines.
pub fn find_flat_foldability_violation(
    point: Point,
    lines: &[LineSegment],
) -> Option<FlatFoldabilityViolation> {
    let mut red = 0usize;
    let mut blue = 0usize;
    let mut black = 0usize;
    let mut nbox = SortingBox::default();

    for segment in lines {
        match segment.color {
            LineColor::Red1 => red += 1,
            LineColor::Blue2 => blue += 1,
            LineColor::Black0 => black += 1,
            _ => {}
        }

        if segment.color.is_folding_line() {
            if point.distance(segment.a) < Epsilon::FLAT {
                nbox.add_by_weight(segment.clone(), angle((segment.a, segment.b)));
            } else if point.distance(segment.b) < Epsilon::FLAT {
                nbox.add_by_weight(segment.clone(), angle((segment.b, segment.a)));
            }
        }
    }

    if black != 0 && black != 2 {
        return Some(FlatFoldabilityViolation::new(
            point,
            FlatFoldabilityRule::NumberOfFolds,
            FlatFoldabilityColor::Unknown,
        ));
    }

    if black == 0 {
        let angle_or_lbl = find_flat_foldability_violation_inside(point, nbox);
        let mut rule = angle_or_lbl
            .as_ref()
            .map(|violation| violation.rule)
            .unwrap_or(FlatFoldabilityRule::None);

        if red.abs_diff(blue) != 2 {
            if matches!(
                rule,
                FlatFoldabilityRule::LittleBigLittle | FlatFoldabilityRule::None
            ) {
                rule = FlatFoldabilityRule::Maekawa;
            }
            return Some(FlatFoldabilityViolation::new(
                point,
                rule,
                maekawa_color(red, blue),
            ));
        }

        if !matches!(
            rule,
            FlatFoldabilityRule::Maekawa | FlatFoldabilityRule::None
        ) {
            if blue == red {
                return Some(FlatFoldabilityViolation::new(
                    point,
                    rule,
                    FlatFoldabilityColor::Equal,
                ));
            }
            if rule == FlatFoldabilityRule::LittleBigLittle {
                return angle_or_lbl;
            }
            return Some(FlatFoldabilityViolation::new(
                point,
                rule,
                FlatFoldabilityColor::Correct,
            ));
        }

        return None;
    }

    find_little_big_little_violation_on_sides(point, nbox)
}

/// Oriedita `FLAT_FOLDABLE_CHECK_63` result coloring once the boundary loop is resolved.
pub fn flat_foldable_boundary_check(
    model: &CreasePatternModel,
    boundary: &mut [LineSegment],
) -> FlatFoldableBoundaryCheck {
    let mut suitable_intersections = true;
    let mut ordered_crossings = Vec::new();

    for boundary_segment in boundary.iter() {
        let mut segment_crossings = Vec::<(f64, LineSegment)>::new();
        for segment in &model.line_segments {
            let intersection = determine_line_segment_intersection_with_precision(
                segment,
                boundary_segment,
                Epsilon::UNKNOWN_1EN4,
            );
            if intersection != Intersection::NoIntersection0
                && intersection != Intersection::Intersects1
            {
                suitable_intersections = false;
            }

            if intersection == Intersection::Intersects1 && segment.color.number() < 3 {
                segment_crossings.push((
                    boundary_segment
                        .a
                        .distance(find_intersection_segments(segment, boundary_segment)),
                    segment.clone(),
                ));
            }
        }

        segment_crossings.sort_by(|left, right| left.0.total_cmp(&right.0));
        ordered_crossings.extend(segment_crossings.into_iter().map(|(_, segment)| segment));
    }

    let color = if suitable_intersections {
        flat_foldable_boundary_color(&ordered_crossings)
    } else {
        LineColor::Yellow7
    };

    if suitable_intersections {
        for segment in boundary {
            *segment = segment.with_line_color(color);
        }
    }

    FlatFoldableBoundaryCheck {
        color,
        suitable_intersections,
        crossing_count: ordered_crossings.len(),
    }
}

fn check3_point(model: &CreasePatternModel, point: Point, diagnostics: &mut Vec<LineSegment>) {
    let tss = vertex_surrounding_line_count(model, point, Epsilon::UNKNOWN_1EN4, |_| true);
    let tss_red = vertex_surrounding_line_count(model, point, Epsilon::UNKNOWN_1EN4, |segment| {
        segment.color == LineColor::Red1
    });
    let tss_blue = vertex_surrounding_line_count(model, point, Epsilon::UNKNOWN_1EN4, |segment| {
        segment.color == LineColor::Blue2
    });
    let tss_black = vertex_surrounding_line_count(model, point, Epsilon::UNKNOWN_1EN4, |segment| {
        segment.color == LineColor::Black0
    });
    let tss_aux_live =
        vertex_surrounding_line_count(model, point, Epsilon::UNKNOWN_1EN4, |segment| {
            !segment.color.is_folding_line()
        });

    if tss_black != 0 && tss_black != 2 {
        diagnostics.push(LineSegment::new(point, point));
    }

    if tss_black == 0 {
        if tss - tss_aux_live == tss_red + tss_blue && tss_red.abs_diff(tss_blue) != 2 {
            diagnostics.push(LineSegment::new(point, point));
        }
        if !extended_fushimi_decide_inside_model(model, point) {
            diagnostics.push(LineSegment::new(point, point));
        }
    }

    if tss_black == 2 && !extended_fushimi_decide_sides_model(model, point) {
        diagnostics.push(LineSegment::new(point, point));
    }
}

fn maekawa_color(red: usize, blue: usize) -> FlatFoldabilityColor {
    if blue == red {
        FlatFoldabilityColor::Equal
    } else if red.abs_diff(blue) > 2 {
        if blue > red {
            FlatFoldabilityColor::NotEnoughMountain
        } else {
            FlatFoldabilityColor::NotEnoughValley
        }
    } else if blue > red {
        FlatFoldabilityColor::NotEnoughValley
    } else {
        FlatFoldabilityColor::NotEnoughMountain
    }
}

fn find_flat_foldability_violation_inside(
    point: Point,
    mut nbox: SortingBox,
) -> Option<FlatFoldabilityViolation> {
    if nbox.total() % 2 == 1 {
        return Some(FlatFoldabilityViolation::new(
            point,
            FlatFoldabilityRule::NumberOfFolds,
            FlatFoldabilityColor::Unknown,
        ));
    }

    if nbox.total() == 2 {
        return match determine_line_segment_intersection_with_precision(
            nbox.value(1),
            nbox.value(2),
            Epsilon::FLAT,
        ) {
            Intersection::ParallelStartOfS1IntersectsStartOfS2_323
            | Intersection::ParallelStartOfS1IntersectsEndOfS2_333
            | Intersection::ParallelEndOfS1IntersectsEndOfS2_353
            | Intersection::ParallelEndOfS1IntersectsStartOfS2_343 => {
                if nbox.value(1).color != nbox.value(2).color {
                    Some(FlatFoldabilityViolation::new(
                        point,
                        FlatFoldabilityRule::Maekawa,
                        FlatFoldabilityColor::Unknown,
                    ))
                } else {
                    Some(FlatFoldabilityViolation::new(
                        point,
                        FlatFoldabilityRule::None,
                        FlatFoldabilityColor::Unknown,
                    ))
                }
            }
            _ => Some(FlatFoldabilityViolation::new(
                point,
                FlatFoldabilityRule::Angles,
                FlatFoldabilityColor::Unknown,
            )),
        };
    }

    if nbox.total() < 2 {
        return Some(FlatFoldabilityViolation::new(
            point,
            FlatFoldabilityRule::Angles,
            FlatFoldabilityColor::Unknown,
        ));
    }

    if !angularly_flatfoldable(&nbox) {
        return Some(FlatFoldabilityViolation::new(
            point,
            FlatFoldabilityRule::Angles,
            FlatFoldabilityColor::Unknown,
        ));
    }

    let mut max_angle = 360.0;
    let mut little_big_little = initial_little_big_little_segments(point, &nbox);

    while nbox.total() > 2 {
        let mut result = None;
        let mut min_angle = 10000.0;

        for k in 1..=nbox.total() {
            let next = if k + 1 > nbox.total() { 1 } else { k + 1 };
            let temp_angle = angle_between_0_kmax(
                angle_between_0_kmax(nbox.weight(next), max_angle)
                    - angle_between_0_kmax(nbox.weight(k), max_angle),
                max_angle,
            );
            if temp_angle < min_angle {
                min_angle = temp_angle;
            }
        }

        for _ in 1..=nbox.total() {
            let temp_angle = angle_between_0_kmax(nbox.weight(2) - nbox.weight(1), max_angle);
            if (temp_angle - min_angle).abs() < Epsilon::FLAT {
                if nbox.value(1).color != nbox.value(2).color {
                    let next_angle = nbox.weight(3);
                    let mut temp = SortingBox::default();
                    for weighted in nbox.iter() {
                        temp.add(WeightedLine {
                            weight: angle_between_0_kmax(weighted.weight - next_angle, max_angle),
                            segment: weighted.segment.clone(),
                        });
                    }

                    let mut reduced = SortingBox::default();
                    for weighted in temp.iter().skip(2) {
                        reduced.add(weighted.clone());
                    }

                    max_angle -= 2.0 * min_angle;
                    result = Some(reduced);
                    break;
                }

                mark_little_big_little(point, nbox.value(1), &mut little_big_little);
            }
            nbox.shift();
        }

        let next = result.unwrap_or_else(|| nbox.clone());
        if next.total() == nbox.total() {
            return Some(FlatFoldabilityViolation::little_big_little(
                point,
                little_big_little,
            ));
        }
        nbox = next;
    }

    let temp_angle = angle_between_0_kmax(
        angle_between_0_kmax(nbox.weight(1), max_angle)
            - angle_between_0_kmax(nbox.weight(2), max_angle),
        max_angle,
    );
    if (max_angle - temp_angle * 2.0).abs() < Epsilon::FLAT {
        None
    } else {
        Some(FlatFoldabilityViolation::new(
            point,
            FlatFoldabilityRule::Angles,
            FlatFoldabilityColor::Unknown,
        ))
    }
}

fn find_little_big_little_violation_on_sides(
    point: Point,
    mut nbox: SortingBox,
) -> Option<FlatFoldabilityViolation> {
    if nbox.total() < 2 {
        return Some(FlatFoldabilityViolation::new(
            point,
            FlatFoldabilityRule::Maekawa,
            FlatFoldabilityColor::Unknown,
        ));
    }

    if nbox.total() == 2 {
        if nbox.value(1).color != LineColor::Black0 || nbox.value(2).color != LineColor::Black0 {
            return Some(FlatFoldabilityViolation::new(
                point,
                FlatFoldabilityRule::Maekawa,
                FlatFoldabilityColor::Unknown,
            ));
        }
        return None;
    }

    let mut first = None;
    for i in 1..nbox.total() {
        if nbox.value(i).color == LineColor::Black0 && nbox.value(i + 1).color == LineColor::Black0
        {
            first = Some(i + 1);
        }
    }
    if nbox.value(nbox.total()).color == LineColor::Black0
        && nbox.value(1).color == LineColor::Black0
    {
        first = Some(1);
    }

    let Some(first) = first else {
        return Some(FlatFoldabilityViolation::new(
            point,
            FlatFoldabilityRule::Maekawa,
            FlatFoldabilityColor::Unknown,
        ));
    };

    for _ in 1..first {
        nbox.shift();
    }

    let base_angle = nbox.weight(1);
    let mut normalized = SortingBox::default();
    for weighted in nbox.iter() {
        normalized.add(WeightedLine {
            weight: angle_between_0_360(weighted.weight - base_angle),
            segment: weighted.segment.clone(),
        });
    }
    nbox = normalized;

    let mut little_big_little = initial_little_big_little_segments(point, &nbox);
    while nbox.total() > 2 {
        let next = little_big_little_single_step(&nbox, &mut little_big_little, point);
        if next.total() == nbox.total() {
            return Some(FlatFoldabilityViolation::little_big_little(
                point,
                little_big_little,
            ));
        }
        nbox = next;
    }

    None
}

fn little_big_little_single_step(
    nbox: &SortingBox,
    little_big_little: &mut Vec<LittleBigLittleSegment>,
    point: Point,
) -> SortingBox {
    let mut min_angle = 10000.0;
    for k in 1..nbox.total() {
        let temp_angle = nbox.weight(k + 1) - nbox.weight(k);
        if temp_angle < min_angle {
            min_angle = temp_angle;
        }
    }

    let temp_angle = nbox.weight(2) - nbox.weight(1);
    if (temp_angle - min_angle).abs() < Epsilon::FLAT {
        let mut reduced = SortingBox::default();
        for weighted in nbox.iter().skip(1) {
            reduced.add(weighted.clone());
        }
        return reduced;
    }

    let temp_angle = nbox.weight(nbox.total()) - nbox.weight(nbox.total() - 1);
    if (temp_angle - min_angle).abs() < Epsilon::FLAT {
        let mut reduced = SortingBox::default();
        for weighted in nbox.iter().take(nbox.total() - 1) {
            reduced.add(weighted.clone());
        }
        return reduced;
    }

    for k in 2..=nbox.total().saturating_sub(2) {
        let temp_angle = nbox.weight(k + 1) - nbox.weight(k);
        if (temp_angle - min_angle).abs() < Epsilon::FLAT {
            if nbox.value(k).color != nbox.value(k + 1).color {
                let mut reduced = SortingBox::default();
                for weighted in nbox.iter().take(k - 1) {
                    reduced.add(weighted.clone());
                }
                for weighted in nbox.iter().skip(k + 1) {
                    reduced.add(WeightedLine {
                        weight: weighted.weight - 2.0 * min_angle,
                        segment: weighted.segment.clone(),
                    });
                }
                return reduced;
            }

            mark_little_big_little(point, nbox.value(k), little_big_little);
        }
    }

    nbox.clone()
}

fn angularly_flatfoldable(lines: &SortingBox) -> bool {
    let mut even = 0.0;
    let mut odd = 0.0;
    for k in 1..=lines.total() {
        if k % 2 == 0 {
            even += lines.weight(k) - lines.weight(k - 1);
        } else if k == 1 {
            odd += lines.weight(k) - (lines.weight(lines.total()) - 360.0);
        } else {
            odd += lines.weight(k) - lines.weight(k - 1);
        }
    }

    (even.abs() - odd.abs()).abs() < Epsilon::FLAT
}

fn initial_little_big_little_segments(
    point: Point,
    nbox: &SortingBox,
) -> Vec<LittleBigLittleSegment> {
    nbox.iter()
        .map(|weighted| LittleBigLittleSegment {
            segment: orient_little_big_little_segment(point, &weighted.segment),
            violating: false,
        })
        .collect()
}

fn mark_little_big_little(
    point: Point,
    segment: &LineSegment,
    little_big_little: &mut Vec<LittleBigLittleSegment>,
) {
    let segment = orient_little_big_little_segment(point, segment);
    if let Some(entry) = little_big_little
        .iter_mut()
        .find(|entry| entry.segment == segment)
    {
        entry.violating = true;
    } else {
        little_big_little.push(LittleBigLittleSegment {
            segment,
            violating: true,
        });
    }
}

fn orient_little_big_little_segment(point: Point, segment: &LineSegment) -> LineSegment {
    if segment.a.distance(point) > Epsilon::UNKNOWN_1EN6 {
        segment.with_swapped_coordinates()
    } else {
        segment.clone()
    }
}

fn point_line_map(model: &CreasePatternModel) -> Vec<(Point, Vec<LineSegment>)> {
    let mut map = Vec::<(Point, Vec<LineSegment>)>::new();
    let eps_squared = Epsilon::UNKNOWN_1EN4 * Epsilon::UNKNOWN_1EN4;

    for segment in &model.line_segments {
        if segment.color != LineColor::Cyan3 {
            point_line_map_process(&mut map, segment.a, segment, eps_squared);
            point_line_map_process(&mut map, segment.b, segment, eps_squared);
        }
    }

    map
}

fn point_line_map_process(
    map: &mut Vec<(Point, Vec<LineSegment>)>,
    point: Point,
    segment: &LineSegment,
    eps_squared: f64,
) {
    if let Some((_, lines)) = map
        .iter_mut()
        .find(|(candidate, _)| candidate.distance_squared(point) < eps_squared)
    {
        lines.push(segment.clone());
    } else {
        map.push((point, vec![segment.clone()]));
    }
}

fn check1_intersection_matches(intersection: Intersection) -> bool {
    matches!(
        intersection,
        Intersection::ParallelEqual31
            | Intersection::ParallelStartOfS1ContainsStartOfS2_321
            | Intersection::ParallelStartOfS2ContainsStartOfS1_322
            | Intersection::ParallelStartOfS1ContainsEndOfS2_331
            | Intersection::ParallelEndOfS2ContainsStartOfS1_332
            | Intersection::ParallelEndOfS1ContainsStartOfS2_341
            | Intersection::ParallelStartOfS2ContainsEndOfS1_342
            | Intersection::ParallelEndOfS1ContainsEndOfS2_351
            | Intersection::ParallelEndOfS2ContainsEndOfS1_352
    )
}

fn vertex_surrounding_line_count(
    model: &CreasePatternModel,
    point: Point,
    radius: f64,
    predicate: impl Fn(&LineSegment) -> bool,
) -> usize {
    let q = closest_point(model, point);
    let radius_squared = radius * radius;

    model
        .line_segments
        .iter()
        .filter(|segment| {
            let endpoint = if q.distance_squared(segment.b) < q.distance_squared(segment.a) {
                segment.b
            } else {
                segment.a
            };
            q.distance_squared(endpoint) < radius_squared && predicate(segment)
        })
        .count()
}

fn extended_fushimi_decide_inside_model(model: &CreasePatternModel, point: Point) -> bool {
    let vertex = closest_point_of_fold_line(model, point);
    let nbox = vertex_sorting_box(model, vertex);
    extended_fushimi_decide_inside(nbox)
}

fn extended_fushimi_decide_inside(mut nbox: SortingBox) -> bool {
    if nbox.total() % 2 == 1 {
        return false;
    }
    if nbox.total() < 2 {
        return false;
    }

    if nbox.total() == 2 {
        if nbox.value(1).color != nbox.value(2).color {
            return false;
        }

        return matches!(
            determine_line_segment_intersection_with_precision(
                nbox.value(1),
                nbox.value(2),
                Epsilon::FLAT
            ),
            Intersection::ParallelStartOfS1IntersectsStartOfS2_323
                | Intersection::ParallelStartOfS1IntersectsEndOfS2_333
                | Intersection::ParallelEndOfS1IntersectsEndOfS2_353
                | Intersection::ParallelEndOfS1IntersectsStartOfS2_343
        );
    }

    let mut max_angle = 360.0;
    while nbox.total() > 2 {
        let mut result = None;
        let mut min_angle = 10000.0;

        for k in 1..=nbox.total() {
            let current = k;
            let next = if k + 1 > nbox.total() { 1 } else { k + 1 };
            let temp_angle = angle_between_0_kmax(
                angle_between_0_kmax(nbox.weight(next), max_angle)
                    - angle_between_0_kmax(nbox.weight(current), max_angle),
                max_angle,
            );
            if temp_angle < min_angle {
                min_angle = temp_angle;
            }
        }

        for _ in 1..=nbox.total() {
            let temp_angle = angle_between_0_kmax(nbox.weight(2) - nbox.weight(1), max_angle);
            if (temp_angle - min_angle).abs() < Epsilon::FLAT
                && nbox.value(1).color != nbox.value(2).color
            {
                let base_angle = nbox.weight(3);
                let mut temp = SortingBox::default();
                for weighted in nbox.iter() {
                    temp.add(WeightedLine {
                        weight: angle_between_0_kmax(weighted.weight - base_angle, max_angle),
                        segment: weighted.segment.clone(),
                    });
                }

                let mut reduced = SortingBox::default();
                for weighted in temp.iter().skip(2) {
                    reduced.add(weighted.clone());
                }

                max_angle -= 2.0 * min_angle;
                result = Some(reduced);
                break;
            }
            nbox.shift();
        }

        let next = result.unwrap_or_else(|| nbox.clone());
        if next.total() == nbox.total() {
            return false;
        }
        nbox = next;
    }

    let temp_angle = angle_between_0_kmax(
        angle_between_0_kmax(nbox.weight(1), max_angle)
            - angle_between_0_kmax(nbox.weight(2), max_angle),
        max_angle,
    );
    (max_angle - temp_angle * 2.0).abs() < Epsilon::FLAT
}

fn extended_fushimi_decide_sides_model(model: &CreasePatternModel, point: Point) -> bool {
    let vertex = closest_point_of_fold_line(model, point);
    let nbox = vertex_sorting_box(model, vertex);
    extended_fushimi_decide_sides(nbox)
}

fn extended_fushimi_decide_sides(mut nbox: SortingBox) -> bool {
    if nbox.total() < 2 {
        return false;
    }

    if nbox.total() == 2 {
        if nbox.value(1).color != LineColor::Black0 {
            return false;
        }
        return nbox.value(2).color == LineColor::Black0;
    }

    let mut first = None;
    for i in 1..nbox.total() {
        if nbox.value(i).color == LineColor::Black0 && nbox.value(i + 1).color == LineColor::Black0
        {
            first = Some(i + 1);
        }
    }
    if nbox.total() > 0
        && nbox.value(nbox.total()).color == LineColor::Black0
        && nbox.value(1).color == LineColor::Black0
    {
        first = Some(1);
    }

    let Some(first) = first else {
        return false;
    };

    for _ in 1..first {
        nbox.shift();
    }

    let base_angle = nbox.weight(1);
    let mut normalized = SortingBox::default();
    for weighted in nbox.iter() {
        normalized.add(WeightedLine {
            weight: angle_between_0_360(weighted.weight - base_angle),
            segment: weighted.segment.clone(),
        });
    }
    nbox = normalized;

    while nbox.total() > 2 {
        let next = extended_fushimi_determine_sides_theorem(&nbox);
        if next.total() == nbox.total() {
            return false;
        }
        nbox = next;
    }

    true
}

fn extended_fushimi_determine_sides_theorem(nbox: &SortingBox) -> SortingBox {
    let mut min_angle = 10000.0;
    for k in 1..nbox.total() {
        let temp_angle = nbox.weight(k + 1) - nbox.weight(k);
        if temp_angle < min_angle {
            min_angle = temp_angle;
        }
    }

    let temp_angle = nbox.weight(2) - nbox.weight(1);
    if (temp_angle - min_angle).abs() < Epsilon::FLAT {
        let mut reduced = SortingBox::default();
        for weighted in nbox.iter().skip(1) {
            reduced.add(weighted.clone());
        }
        return reduced;
    }

    let temp_angle = nbox.weight(nbox.total()) - nbox.weight(nbox.total() - 1);
    if (temp_angle - min_angle).abs() < Epsilon::FLAT {
        let mut reduced = SortingBox::default();
        for weighted in nbox.iter().take(nbox.total() - 1) {
            reduced.add(weighted.clone());
        }
        return reduced;
    }

    for k in 2..=nbox.total().saturating_sub(2) {
        let temp_angle = nbox.weight(k + 1) - nbox.weight(k);
        if (temp_angle - min_angle).abs() < Epsilon::FLAT
            && nbox.value(k).color != nbox.value(k + 1).color
        {
            let mut reduced = SortingBox::default();
            for weighted in nbox.iter().take(k - 1) {
                reduced.add(weighted.clone());
            }
            for weighted in nbox.iter().skip(k + 1) {
                reduced.add(WeightedLine {
                    weight: weighted.weight - 2.0 * min_angle,
                    segment: weighted.segment.clone(),
                });
            }
            return reduced;
        }
    }

    nbox.clone()
}

fn vertex_sorting_box(model: &CreasePatternModel, vertex: Point) -> SortingBox {
    let mut nbox = SortingBox::default();
    for segment in &model.line_segments {
        if segment.color.is_folding_line() {
            if vertex.distance(segment.a) < Epsilon::FLAT {
                nbox.add_by_weight(segment.clone(), angle((segment.a, segment.b)));
            } else if vertex.distance(segment.b) < Epsilon::FLAT {
                nbox.add_by_weight(segment.clone(), angle((segment.b, segment.a)));
            }
        }
    }
    nbox
}

fn closest_point(model: &CreasePatternModel, point: Point) -> Point {
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

fn closest_point_of_fold_line(model: &CreasePatternModel, point: Point) -> Point {
    let mut closest = Point::new(100000.0, 100000.0);
    for segment in &model.line_segments {
        if segment.color.is_folding_line() {
            if point.distance_squared(segment.a) < point.distance_squared(closest) {
                closest = segment.a;
            }
            if point.distance_squared(segment.b) < point.distance_squared(closest) {
                closest = segment.b;
            }
        }
    }
    closest
}

#[derive(Debug, Clone)]
struct WeightedLine {
    segment: LineSegment,
    weight: f64,
}

#[derive(Debug, Clone, Default)]
struct SortingBox {
    values: Vec<WeightedLine>,
}

impl SortingBox {
    fn total(&self) -> usize {
        self.values.len()
    }

    fn add(&mut self, value: WeightedLine) {
        self.values.push(value);
    }

    fn add_by_weight(&mut self, segment: LineSegment, weight: f64) {
        let value = WeightedLine { segment, weight };
        for index in 0..self.values.len() {
            if value.weight < self.values[index].weight {
                self.values.insert(index, value);
                return;
            }
        }
        self.values.push(value);
    }

    fn shift(&mut self) {
        if !self.values.is_empty() {
            self.values.rotate_left(1);
        }
    }

    fn iter(&self) -> impl Iterator<Item = &WeightedLine> {
        self.values.iter()
    }

    fn value(&self, one_based: usize) -> &LineSegment {
        &self.values[one_based - 1].segment
    }

    fn weight(&self, one_based: usize) -> f64 {
        self.values[one_based - 1].weight
    }
}

fn flat_foldable_boundary_color(crossings: &[LineSegment]) -> LineColor {
    if !crossings.len().is_multiple_of(2) {
        return LineColor::Magenta5;
    }
    if crossings.is_empty() {
        return LineColor::Cyan3;
    }

    let mut moved = crossings[0].clone();
    for crossing in crossings.iter().skip(1) {
        moved = find_line_symmetry_line_segment(&moved, crossing);
    }

    if equal_with_radius(crossings[0].a, moved.a, Epsilon::UNKNOWN_1EN4)
        && equal_with_radius(crossings[0].b, moved.b, Epsilon::UNKNOWN_1EN4)
    {
        LineColor::Cyan3
    } else {
        LineColor::Magenta5
    }
}
