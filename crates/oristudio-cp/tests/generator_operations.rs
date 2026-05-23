use oristudio_cp::geometry::{LineColor, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::generators::{
    DefaultMolecule, VoronoiState, default_molecule, regular_polygon_no_corners, voronoi_apply,
    voronoi_press,
};

#[test]
fn regular_polygon_no_corners_adds_rotated_edges() {
    let mut model = CreasePatternModel::default();

    let added = regular_polygon_no_corners(
        &mut model,
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        4,
        LineColor::Red1,
    );

    assert_eq!(added, 4);
    assert_eq!(model.line_segments.len(), 4);
    assert!(
        model
            .line_segments
            .iter()
            .all(|segment| segment.color == LineColor::Red1)
    );
}

#[test]
fn default_molecule_adds_template_edges_with_selected_color() {
    let mut model = CreasePatternModel::default();

    let added = default_molecule(
        &mut model,
        DefaultMolecule::Blintz,
        Point::new(-199.99999999999997, -200.0),
        Point::new(200.0, 200.0),
        LineColor::Blue2,
    )
    .expect("bundled default molecule should import");

    assert_eq!(added, 4);
    assert_eq!(model.line_segments.len(), 4);
    assert!(
        model
            .line_segments
            .iter()
            .all(|segment| segment.color == LineColor::Blue2)
    );
}

#[test]
fn default_molecule_ignores_degenerate_anchor_points() {
    let mut model = CreasePatternModel::default();

    let added = default_molecule(
        &mut model,
        DefaultMolecule::Blintz,
        Point::new(1.0, 1.0),
        Point::new(1.0, 1.0),
        LineColor::Red1,
    )
    .expect("bundled default molecule should import");

    assert_eq!(added, 0);
    assert!(model.line_segments.is_empty());
}

#[test]
fn voronoi_press_adds_and_removes_seed_points_with_preview_lines() {
    let model = CreasePatternModel::default();
    let mut state = VoronoiState::default();

    voronoi_press(&model, &mut state, Point::new(0.0, 0.0), 0.25);
    assert_eq!(state.seed_points, vec![Point::new(0.0, 0.0)]);
    assert!(state.line_segments.is_empty());

    voronoi_press(&model, &mut state, Point::new(2.0, 0.0), 0.25);
    assert_eq!(state.seed_points.len(), 2);
    assert_eq!(state.line_segments.len(), 1);
    assert_eq!(state.line_segments[0].voronoi_a, 0);
    assert_eq!(state.line_segments[0].voronoi_b, 1);
    assert!((state.line_segments[0].line_segment.a.x - 1.0).abs() < 1e-12);
    assert!((state.line_segments[0].line_segment.b.x - 1.0).abs() < 1e-12);

    voronoi_press(&model, &mut state, Point::new(2.1, 0.0), 0.25);
    assert_eq!(state.seed_points, vec![Point::new(0.0, 0.0)]);
    assert!(state.line_segments.is_empty());
}

#[test]
fn voronoi_apply_commits_preview_lines_and_seed_circles_then_resets() {
    let model = CreasePatternModel::default();
    let mut state = VoronoiState::default();
    voronoi_press(&model, &mut state, Point::new(0.0, 0.0), 0.25);
    voronoi_press(&model, &mut state, Point::new(2.0, 0.0), 0.25);
    voronoi_press(&model, &mut state, Point::new(0.0, 2.0), 0.25);

    let mut model = CreasePatternModel::default();
    let result = voronoi_apply(&mut model, &mut state, LineColor::Blue2);

    assert_eq!(result.lines_added, 3);
    assert_eq!(result.circles_added, 3);
    assert_eq!(model.circles.len(), 3);
    assert!(model.circles.iter().all(|circle| circle.r == 5.0));
    assert!(
        model
            .line_segments
            .iter()
            .all(|segment| segment.color == LineColor::Blue2)
    );
    assert!(state.seed_points.is_empty());
    assert!(state.line_segments.is_empty());
}
