use crate::geometry::{Epsilon, LineColor, LineSegment, Point, line_segment_rotate};
use crate::io::Result;
use crate::io::fold::import_fold_json;
use crate::model::CreasePatternModel;
use crate::operations::arrangement::add_line_segment_like_worker;
use crate::operations::transform::transform_segments_by_points;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultMolecule {
    Blintz,
    FishBase,
    DoveBase,
    BirdBase,
    FrogBase,
}

/// Oriedita `POLYGON_SET_NO_CORNERS_29` after both polygon points are resolved.
pub fn regular_polygon_no_corners(
    model: &mut CreasePatternModel,
    p1: Point,
    p2: Point,
    corners: usize,
    color: LineColor,
) -> usize {
    let mut added = 0;
    let mut seed = LineSegment::with_color(p1, p2, color);
    add_line_segment_like_worker(model, &seed);
    added += 1;

    if corners < 2 {
        return added;
    }

    let rotation = (corners as f64 - 2.0) * 180.0 / corners as f64;
    for _ in 2..=corners {
        let rotated = line_segment_rotate(&seed, rotation);
        seed = LineSegment::with_color(rotated.b, rotated.a, color);
        add_line_segment_like_worker(model, &seed);
        added += 1;
    }

    added
}

pub fn regular_polygon(
    model: &mut CreasePatternModel,
    p1: Point,
    p2: Point,
    corners: usize,
    color: LineColor,
) -> usize {
    regular_polygon_no_corners(model, p1, p2, corners, color)
}

pub fn default_molecule(
    model: &mut CreasePatternModel,
    molecule: DefaultMolecule,
    p1: Point,
    p2: Point,
    color: LineColor,
) -> Result<usize> {
    if distance_too_small(p1, p2) {
        return Ok(0);
    }

    let mut template = import_fold_json(molecule.fold_json())?;
    let starting_circles: Vec<_> = template
        .circles
        .iter()
        .copied()
        .filter(|circle| circle.r > Epsilon::UNKNOWN_1EN6)
        .collect();
    if starting_circles.len() < 2 {
        return Ok(0);
    }

    transform_segments_by_points(
        &mut template.line_segments,
        starting_circles[0].determine_center(),
        starting_circles[1].determine_center(),
        p1,
        p2,
    );

    let mut added = 0;
    for segment in template
        .line_segments
        .iter()
        .filter(|segment| segment.determine_length() > Epsilon::UNKNOWN_1EN6)
    {
        add_line_segment_like_worker(model, &segment.with_line_color(color));
        added += 1;
    }

    Ok(added)
}

fn distance_too_small(p1: Point, p2: Point) -> bool {
    p1.distance(p2) < Epsilon::UNKNOWN_1EN6
}

impl DefaultMolecule {
    fn fold_json(self) -> &'static str {
        match self {
            Self::Blintz => include_str!("../../resources/default-molecules/blintz.fold"),
            Self::FishBase => include_str!("../../resources/default-molecules/fish_base.fold"),
            Self::DoveBase => include_str!("../../resources/default-molecules/dove_base.fold"),
            Self::BirdBase => include_str!("../../resources/default-molecules/bird_base.fold"),
            Self::FrogBase => include_str!("../../resources/default-molecules/frog_base.fold"),
        }
    }
}
