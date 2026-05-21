use oristudio_cp::checks::{check1, check2};
use oristudio_cp::geometry::{LineColor, LineSegment, Point};
use oristudio_cp::model::CreasePatternModel;

#[test]
fn check1_reports_overlapping_non_auxiliary_pairs_in_oriedita_order() {
    let mut model = CreasePatternModel::default();
    let first = segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1);
    let second = segment(5.0, 0.0, 15.0, 0.0, LineColor::Blue2);
    let aux_overlap = segment(6.0, 0.0, 8.0, 0.0, LineColor::Cyan3);
    model.add_line_segment(first.clone());
    model.add_line_segment(second.clone());
    model.add_line_segment(aux_overlap);

    assert_eq!(check1(&model), vec![second, first]);
}

#[test]
fn check2_reports_t_shape_non_auxiliary_pairs_in_oriedita_order() {
    let mut model = CreasePatternModel::default();
    let bar = segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1);
    let stem = segment(5.0, 0.0, 5.0, 5.0, LineColor::Blue2);
    let crossing = segment(2.0, -1.0, 2.0, 1.0, LineColor::Black0);
    model.add_line_segment(bar.clone());
    model.add_line_segment(stem.clone());
    model.add_line_segment(crossing);

    assert_eq!(check2(&model), vec![stem, bar]);
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}
