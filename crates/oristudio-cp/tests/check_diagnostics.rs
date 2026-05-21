use oristudio_cp::checks::{
    FlatFoldabilityColor, FlatFoldabilityRule, check_camv_task, check1, check2, check3, check4,
};
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

#[test]
fn check3_reports_invalid_boundary_vertex_line_counts() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(segment(0.0, 0.0, 10.0, 0.0, LineColor::Black0));
    model.add_line_segment(segment(0.0, 0.0, 0.0, 10.0, LineColor::Red1));

    let diagnostics = check3(&model);

    assert!(diagnostics.contains(&LineSegment::new(Point::origin(), Point::origin())));
}

#[test]
fn check3_reports_maekawa_and_fushimi_failures_without_deduplicating_markers() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1));
    model.add_line_segment(segment(0.0, 0.0, -10.0, 0.0, LineColor::Red1));
    model.add_line_segment(segment(0.0, 0.0, 0.0, 10.0, LineColor::Blue2));
    model.add_line_segment(segment(0.0, 0.0, 0.0, -10.0, LineColor::Blue2));

    let origin_markers = check3(&model)
        .into_iter()
        .filter(|segment| segment.a == Point::origin() && segment.b == Point::origin())
        .count();

    assert!(origin_markers >= 2);
}

#[test]
fn check4_reports_structured_maekawa_violation() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1));
    model.add_line_segment(segment(0.0, 0.0, -10.0, 0.0, LineColor::Blue2));

    let violations = check4(&model);

    let origin = violations
        .iter()
        .find(|violation| violation.point == Point::origin())
        .expect("shared vertex should have a structured violation");
    assert_eq!(origin.rule, FlatFoldabilityRule::Maekawa);
    assert_eq!(origin.color, FlatFoldabilityColor::Equal);
    assert!(origin.little_big_little.is_empty());
}

#[test]
fn check4_reports_little_big_little_payloads() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1));
    model.add_line_segment(segment(0.0, 0.0, 8.660254037844386, 5.0, LineColor::Red1));
    model.add_line_segment(segment(0.0, 0.0, 0.0, 10.0, LineColor::Blue2));
    model.add_line_segment(segment(0.0, 0.0, -10.0, 0.0, LineColor::Blue2));
    model.add_line_segment(segment(0.0, 0.0, -8.660254037844386, -5.0, LineColor::Red1));
    model.add_line_segment(segment(0.0, 0.0, 0.0, -10.0, LineColor::Red1));

    let violations = check4(&model);

    assert!(violations.iter().any(|violation| {
        violation.rule == FlatFoldabilityRule::LittleBigLittle
            && violation
                .little_big_little
                .iter()
                .any(|line| line.violating)
    }));
}

#[test]
fn check_camv_task_recomputes_check4_and_marks_dirty() {
    let mut model = CreasePatternModel::default();
    model.add_line_segment(segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1));
    model.add_line_segment(segment(0.0, 0.0, -10.0, 0.0, LineColor::Blue2));

    let result = check_camv_task(&model);

    assert!(result.dirty);
    assert_eq!(result.violations, check4(&model));
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}
