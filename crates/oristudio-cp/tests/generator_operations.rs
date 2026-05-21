use oristudio_cp::geometry::{LineColor, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::generators::{
    DefaultMolecule, default_molecule, regular_polygon_no_corners,
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
