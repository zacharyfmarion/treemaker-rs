use oristudio_cp::geometry::{LineColor, Point};
use oristudio_cp::model::CreasePatternModel;
use oristudio_cp::operations::generators::regular_polygon_no_corners;

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
