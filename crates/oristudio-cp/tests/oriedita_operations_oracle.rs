use oristudio_cp::geometry::{Intersection, LineColor, LineSegment, Point};
use oristudio_cp::model::{CreasePatternModel, CustomLineType};
use oristudio_cp::operations::arrangement::{
    branch_trim, del_v_all, del_v_all_color_change, del_v_at_point, del_v_at_point_color_change,
    del_v_pair, delete_intersecting_or_overlapping_lines_along,
    delete_line_segment_vertex_for_index, delete_line_segments_for_indices,
    delete_overlapping_lines_along, divide_intersections, divide_intersections_fast,
    divide_line_segment_with_new_lines, fix1, fix2, intersect_divide_pair,
};
use oristudio_cp::operations::color::{
    advance_line_type, alternate_mountain_valley_along, alternate_mountain_valley_crossing,
    change_crease_type, delete_line_type_for_indices, make_aux, make_edge, make_mountain,
    make_valley, replace_line_type_for_indices, set_line_color_for_indices, toggle_mountain_valley,
};
use oristudio_cp::operations::measure::{angle_between_three_points, length_between_points};
use oristudio_cp::operations::point::{
    divide_segment_by_count, divide_segment_by_ratio, draw_point_on_segment,
};
use oristudio_cp::operations::selection::{
    delete_selected_lines, select_all, select_box, select_connected_from_point, select_indices,
    select_intersecting_line, select_polygon, unselect_all, unselect_indices,
    unselect_intersecting_line, unselect_polygon,
};
use oristudio_cp::operations::transform::{
    copy_selected_lines, copy_selected_lines_by_points, extend_to_intersection_point_2,
    move_selected_lines, move_selected_lines_by_points, translate_model,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn intersect_divide_pair_matches_oriedita_operations_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for (i, j, segments) in [
        (
            0,
            1,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
            ],
        ),
        (
            0,
            1,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, 0.0, 5.0, 5.0, LineColor::Blue2),
            ],
        ),
        (
            0,
            1,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, 0.0, 15.0, 0.0, LineColor::Blue2),
            ],
        ),
        (
            1,
            0,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, 0.0, 15.0, 0.0, LineColor::Blue2),
            ],
        ),
    ] {
        let mut model = model_from_segments(&segments);
        let added = intersect_divide_pair(&mut model, i, j);

        let mut args = vec![
            "intersect-divide-pair".to_string(),
            i.to_string(),
            j.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);

        let rust_summary = format!("added|{added}\n{}", line_segment_set_summary(&model));
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }
}

#[test]
fn divide_intersections_matches_oriedita_operations_oracle_for_crossing_fixture() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
    ];
    let mut model = model_from_segments(&segments);
    divide_intersections(&mut model);

    let mut args = vec!["intersect-divide".to_string(), segments.len().to_string()];
    push_segment_args(&mut args, &segments);

    assert_eq!(line_segment_set_summary(&model), run_oracle(&oracle, &args));
}

#[test]
fn divide_intersections_fast_matches_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for (i, j, segments) in [
        (
            1,
            0,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
            ],
        ),
        (
            1,
            0,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, 0.0, 5.0, 5.0, LineColor::Cyan3),
            ],
        ),
        (
            1,
            0,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Blue2),
                segment(5.0, 0.0, 15.0, 0.0, LineColor::Red1),
            ],
        ),
        (
            1,
            0,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(10.0, 0.0, 0.0, 0.0, LineColor::Blue2),
            ],
        ),
    ] {
        let mut model = model_from_segments(&segments);
        let mut to_delete = BTreeSet::new();
        let intersection = divide_intersections_fast(&mut model, i, j, &mut to_delete);

        let mut args = vec![
            "foldline-divide-fast".to_string(),
            i.to_string(),
            j.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);

        let rust_summary = format!(
            "intersection|{}\n{}{}",
            intersection_state(intersection),
            delete_summary(&to_delete),
            line_segment_set_summary(&model)
        );
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }
}

#[test]
fn divide_line_segment_with_new_lines_matches_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for (original_end, added_end, segments) in [
        (
            1,
            2,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
            ],
        ),
        (
            1,
            2,
            vec![
                segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
                segment(10.0, 0.0, 0.0, 0.0, LineColor::Blue2),
            ],
        ),
    ] {
        let mut model = model_from_segments(&segments);
        divide_line_segment_with_new_lines(&mut model, original_end, added_end);

        let mut args = vec![
            "foldline-divide-new-lines".to_string(),
            original_end.to_string(),
            added_end.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);

        assert_eq!(line_segment_set_summary(&model), run_oracle(&oracle, &args));
    }
}

#[test]
fn delete_inside_line_matches_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let selection = segment(2.0, 0.0, 8.0, 0.0, LineColor::Black0);
    let segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
        segment(0.0, 1.0, 10.0, 1.0, LineColor::Cyan3),
    ];

    for mode in ["l", "lX"] {
        let mut model = model_from_segments(&segments);
        let deleted = match mode {
            "l" => delete_overlapping_lines_along(&mut model, &selection),
            "lX" => delete_intersecting_or_overlapping_lines_along(&mut model, &selection),
            _ => unreachable!(),
        };

        let mut args = vec!["foldline-delete-inside".to_string(), mode.to_string()];
        push_one_segment_args(&mut args, &selection);
        args.push(segments.len().to_string());
        push_segment_args(&mut args, &segments);

        let rust_summary = format!("deleted|{deleted}\n{}", line_segment_set_summary(&model));
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }
}

#[test]
fn line_delete_commands_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let vertex_segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(10.0, 0.0, 20.0, 0.0, LineColor::Red1),
        segment(10.0, 0.0, 10.0, 5.0, LineColor::Blue2),
    ];
    let mut model = model_from_segments(&vertex_segments);
    let deleted = delete_line_segment_vertex_for_index(&mut model, 2);
    let mut args = vec![
        "foldline-delete-line-vertex".to_string(),
        "2".to_string(),
        vertex_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &vertex_segments);
    let rust_summary = format!("deleted|{deleted}\n{}", line_segment_set_summary(&model));
    assert_eq!(rust_summary, run_oracle(&oracle, &args));

    let box_segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(0.0, 1.0, 1.0, 1.0, LineColor::Blue2),
        segment(0.0, 2.0, 1.0, 2.0, LineColor::Black0),
    ];
    let mut model = model_from_segments(&box_segments);
    let deleted = delete_line_segments_for_indices(&mut model, &[0, 2]);
    let mut args = vec![
        "foldline-delete-lines".to_string(),
        "0,2".to_string(),
        box_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &box_segments);
    let rust_summary = format!("deleted|{deleted}\n{}", line_segment_set_summary(&model));
    assert_eq!(rust_summary, run_oracle(&oracle, &args));
}

#[test]
fn del_v_at_point_matches_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for segments in [
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(10.0, 0.0, 20.0, 0.0, LineColor::Red1),
        ],
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(10.0, 0.0, 20.0, 0.0, LineColor::Blue2),
        ],
    ] {
        let mut model = model_from_segments(&segments);
        let result = del_v_at_point(&mut model, Point::new(10.0, 0.0), 0.000001, 0.000001);

        let mut args = vec![
            "foldline-del-v".to_string(),
            "10".to_string(),
            "0".to_string(),
            "0.000001".to_string(),
            "0.000001".to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);

        let rust_summary = format!("result|{result}\n{}", line_segment_set_summary(&model));
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }
}

#[test]
fn del_v_at_point_color_change_matches_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for segments in [
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Black0),
            segment(10.0, 0.0, 20.0, 0.0, LineColor::Red1),
        ],
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(10.0, 0.0, 20.0, 0.0, LineColor::Blue2),
        ],
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Cyan3),
            segment(10.0, 0.0, 20.0, 0.0, LineColor::Cyan3),
        ],
    ] {
        let mut model = model_from_segments(&segments);
        let result =
            del_v_at_point_color_change(&mut model, Point::new(10.0, 0.0), 0.000001, 0.000001);

        let mut args = vec![
            "foldline-del-v-cc".to_string(),
            "10".to_string(),
            "0".to_string(),
            "0.000001".to_string(),
            "0.000001".to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);

        let rust_summary = format!("result|{result}\n{}", line_segment_set_summary(&model));
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }
}

#[test]
fn del_v_pair_and_all_variants_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let mixed_segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(10.0, 0.0, 20.0, 0.0, LineColor::Blue2),
    ];
    let mut pair_model = model_from_segments(&mixed_segments);
    let pair_result = del_v_pair(&mut pair_model, &mixed_segments[0], &mixed_segments[1]);
    let mut pair_args = vec![
        "foldline-del-v-pair".to_string(),
        "0".to_string(),
        "1".to_string(),
        mixed_segments.len().to_string(),
    ];
    push_segment_args(&mut pair_args, &mixed_segments);
    let pair_summary = format!(
        "{}{}",
        optional_segment_result_summary(pair_result.as_ref()),
        line_segment_set_summary(&pair_model)
    );
    assert_eq!(pair_summary, run_oracle(&oracle, &pair_args));

    let same_color_segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(10.0, 0.0, 20.0, 0.0, LineColor::Red1),
    ];
    let mut all_model = model_from_segments(&same_color_segments);
    del_v_all(&mut all_model);
    let mut all_args = vec![
        "foldline-del-v-all".to_string(),
        same_color_segments.len().to_string(),
    ];
    push_segment_args(&mut all_args, &same_color_segments);
    assert_eq!(
        line_segment_set_summary(&all_model),
        run_oracle(&oracle, &all_args)
    );

    let mut all_cc_model = model_from_segments(&mixed_segments);
    del_v_all_color_change(&mut all_cc_model);
    let mut all_cc_args = vec![
        "foldline-del-v-all-cc".to_string(),
        mixed_segments.len().to_string(),
    ];
    push_segment_args(&mut all_cc_args, &mixed_segments);
    assert_eq!(
        line_segment_set_summary(&all_cc_model),
        run_oracle(&oracle, &all_cc_args)
    );
}

#[test]
fn branch_trim_matches_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(10.0, 0.0, 20.0, 0.0, LineColor::Red1),
        segment(20.0, 0.0, 30.0, 0.0, LineColor::Red1),
    ];
    let mut model = model_from_segments(&segments);
    branch_trim(&mut model);

    let mut args = vec![
        "foldline-branch-trim".to_string(),
        segments.len().to_string(),
    ];
    push_segment_args(&mut args, &segments);

    assert_eq!(line_segment_set_summary(&model), run_oracle(&oracle, &args));
}

#[test]
fn fix_workers_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for segments in [
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(10.0, 0.0, 0.0, 0.0, LineColor::Blue2),
        ],
        vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
            segment(5.0, 0.0, 15.0, 0.0, LineColor::Blue2),
        ],
    ] {
        let mut model = model_from_segments(&segments);
        let result = fix1(&mut model);
        let mut args = vec!["foldline-fix1".to_string(), segments.len().to_string()];
        push_segment_args(&mut args, &segments);
        let rust_summary = format!(
            "result|{result}\n{}",
            line_segment_set_with_selection_summary(&model)
        );
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }

    let fix2_segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(5.0, 0.0, 5.0, 5.0, LineColor::Blue2),
    ];
    let mut model = model_from_segments(&fix2_segments);
    fix2(&mut model);
    let mut args = vec!["foldline-fix2".to_string(), fix2_segments.len().to_string()];
    push_segment_args(&mut args, &fix2_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );
}

#[test]
fn color_operations_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let aux_segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(5.0, -5.0, 5.0, 5.0, LineColor::Cyan3),
    ];
    let mut model = model_from_segments(&aux_segments);
    let changed = set_line_color_for_indices(&mut model, &[1], LineColor::Red1);
    let mut args = vec![
        "foldline-set-color".to_string(),
        LineColor::Red1.number().to_string(),
        "1".to_string(),
        aux_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &aux_segments);
    let rust_summary = format!("changed|{changed}\n{}", line_segment_set_summary(&model));
    assert_eq!(rust_summary, run_oracle(&oracle, &args));

    let mv_segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(0.0, 1.0, 1.0, 1.0, LineColor::Blue2),
        segment(0.0, 2.0, 1.0, 2.0, LineColor::Black0),
    ];
    let mut model = model_from_segments(&mv_segments);
    toggle_mountain_valley(&mut model, &[0, 1, 2]);
    let mut args = vec![
        "foldline-change-mv".to_string(),
        "0,1,2".to_string(),
        mv_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &mv_segments);
    assert_eq!(line_segment_set_summary(&model), run_oracle(&oracle, &args));

    let change_type_segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Black0),
        segment(0.0, 1.0, 1.0, 1.0, LineColor::Cyan3),
    ];
    let mut model = model_from_segments(&change_type_segments);
    let changed = change_crease_type(&mut model, 0);
    let mut args = vec![
        "foldline-change-type".to_string(),
        "0".to_string(),
        change_type_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &change_type_segments);
    let rust_summary = format!("changed|{changed}\n{}", line_segment_set_summary(&model));
    assert_eq!(rust_summary, run_oracle(&oracle, &args));

    for (color, apply) in [
        (
            LineColor::Red1,
            make_mountain as fn(&mut CreasePatternModel, &[usize]) -> usize,
        ),
        (LineColor::Blue2, make_valley),
        (LineColor::Black0, make_edge),
    ] {
        let make_segments = vec![
            segment(0.0, 0.0, 10.0, 0.0, LineColor::Cyan3),
            segment(5.0, 0.0, 5.0, 5.0, LineColor::Black0),
        ];
        let mut model = model_from_segments(&make_segments);
        let changed = apply(&mut model, &[0, 1]);
        let mut args = vec![
            "foldline-make-color".to_string(),
            color.number().to_string(),
            "0,1".to_string(),
            make_segments.len().to_string(),
        ];
        push_segment_args(&mut args, &make_segments);
        let rust_summary = format!("changed|{changed}\n{}", line_segment_set_summary(&model));
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }

    let make_aux_segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(0.0, 1.0, 1.0, 1.0, LineColor::Blue2),
        segment(0.0, 2.0, 1.0, 2.0, LineColor::Cyan3),
    ];
    let mut model = model_from_segments(&make_aux_segments);
    let changed = make_aux(&mut model, &[0, 1, 2]);
    let mut args = vec![
        "foldline-make-aux".to_string(),
        "0,1,2".to_string(),
        make_aux_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &make_aux_segments);
    let rust_summary = format!("changed|{changed}\n{}", line_segment_set_summary(&model));
    assert_eq!(rust_summary, run_oracle(&oracle, &args));

    let advance_segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Black0),
        segment(0.0, 1.0, 1.0, 1.0, LineColor::Blue2),
    ];
    let mut model = model_from_segments(&advance_segments);
    let result = advance_line_type(&mut model, 0);
    let mut args = vec![
        "foldline-advance-type".to_string(),
        "0".to_string(),
        advance_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &advance_segments);
    let rust_summary = format!(
        "result|{result}\n{}",
        line_segment_set_with_selection_summary(&model)
    );
    assert_eq!(rust_summary, run_oracle(&oracle, &args));

    let alternate_segments = vec![
        segment(10.0, 0.0, 20.0, 0.0, LineColor::Black0),
        segment(0.0, 0.0, 5.0, 0.0, LineColor::Black0),
    ];
    let guide = segment(0.0, 0.0, 20.0, 0.0, LineColor::Red1);
    let mut model = model_from_segments(&alternate_segments);
    let changed = alternate_mountain_valley_along(&mut model, &guide, LineColor::Red1);
    let mut args = vec![
        "foldline-alternate-mv".to_string(),
        LineColor::Red1.number().to_string(),
    ];
    push_one_segment_args(&mut args, &guide);
    args.push(alternate_segments.len().to_string());
    push_segment_args(&mut args, &alternate_segments);
    let rust_summary = format!("changed|{changed}\n{}", line_segment_set_summary(&model));
    assert_eq!(rust_summary, run_oracle(&oracle, &args));

    let crossing_segments = vec![
        segment(5.0, -1.0, 5.0, 1.0, LineColor::Black0),
        segment(15.0, -1.0, 15.0, 1.0, LineColor::Black0),
    ];
    let guide = segment(0.0, 0.0, 20.0, 0.0, LineColor::Blue2);
    let mut model = model_from_segments(&crossing_segments);
    let changed = alternate_mountain_valley_crossing(&mut model, &guide, LineColor::Red1);
    let mut args = vec![
        "foldline-alternate-mv-crossing".to_string(),
        LineColor::Red1.number().to_string(),
    ];
    push_one_segment_args(&mut args, &guide);
    args.push(crossing_segments.len().to_string());
    push_segment_args(&mut args, &crossing_segments);
    let rust_summary = format!("changed|{changed}\n{}", line_segment_set_summary(&model));
    assert_eq!(rust_summary, run_oracle(&oracle, &args));
}

#[test]
fn selected_type_commands_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let duplicate_segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(0.0, 1.0, 10.0, 1.0, LineColor::Blue2),
    ];
    let mut model = model_from_segments(&duplicate_segments);
    let changed = replace_line_type_for_indices(
        &mut model,
        &[1],
        CustomLineType::Mountain,
        CustomLineType::Valley,
    );
    let mut args = vec![
        "foldline-replace-type".to_string(),
        CustomLineType::Mountain.number().to_string(),
        CustomLineType::Valley.number().to_string(),
        "1".to_string(),
        duplicate_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &duplicate_segments);
    let rust_summary = format!(
        "changed|{changed}\n{}",
        line_segment_set_with_selection_summary(&model)
    );
    assert_eq!(rust_summary, run_oracle(&oracle, &args));

    let delete_segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(0.0, 1.0, 1.0, 1.0, LineColor::Blue2),
        segment(0.0, 2.0, 1.0, 2.0, LineColor::Black0),
    ];
    let mut model = model_from_segments(&delete_segments);
    delete_line_type_for_indices(&mut model, &[0, 1, 2], CustomLineType::MountainAndValley);
    let mut args = vec![
        "foldline-delete-type".to_string(),
        CustomLineType::MountainAndValley.number().to_string(),
        "0,1,2".to_string(),
        delete_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &delete_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let mut model = model_from_segments(&delete_segments);
    model.line_segments[0] = model.line_segments[0].with_selected(2);
    model.line_segments[2] = model.line_segments[2].with_selected(2);
    delete_selected_lines(&mut model);
    let mut args = vec![
        "foldline-delete-selected".to_string(),
        "0,2".to_string(),
        delete_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &delete_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );
}

#[test]
fn selection_primitives_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let segments = vec![
        segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1),
        segment(5.0, -5.0, 5.0, 5.0, LineColor::Blue2),
        segment(0.0, 1.0, 10.0, 1.0, LineColor::Black0),
    ];

    let mut model = model_from_segments(&segments);
    select_all(&mut model);
    let mut args = vec![
        "foldline-select-all".to_string(),
        "select".to_string(),
        "-".to_string(),
        segments.len().to_string(),
    ];
    push_segment_args(&mut args, &segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let mut model = model_from_segments(&segments);
    for index in 0..segments.len() {
        model.line_segments[index] = model.line_segments[index].with_selected(2);
    }
    unselect_all(&mut model);
    let mut args = vec![
        "foldline-select-all".to_string(),
        "unselect".to_string(),
        "0,1,2".to_string(),
        segments.len().to_string(),
    ];
    push_segment_args(&mut args, &segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let mut model = model_from_segments(&segments);
    select_indices(&mut model, &[0, 2]);
    let mut args = vec![
        "foldline-select-indices".to_string(),
        "select".to_string(),
        "0,2".to_string(),
        "-".to_string(),
        segments.len().to_string(),
    ];
    push_segment_args(&mut args, &segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let mut model = model_from_segments(&segments);
    model.line_segments[0] = model.line_segments[0].with_selected(2);
    model.line_segments[2] = model.line_segments[2].with_selected(2);
    unselect_indices(&mut model, &[2]);
    let mut args = vec![
        "foldline-select-indices".to_string(),
        "unselect".to_string(),
        "2".to_string(),
        "0,2".to_string(),
        segments.len().to_string(),
    ];
    push_segment_args(&mut args, &segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );
}

#[test]
fn polygon_and_intersection_selection_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let polygon_segments = vec![
        segment(0.25, 0.25, 0.75, 0.75, LineColor::Red1),
        segment(-1.0, 0.0, 0.0, 0.0, LineColor::Blue2),
        segment(-1.0, 0.5, 2.0, 0.5, LineColor::Black0),
    ];
    let polygon = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(0.0, 1.0),
    ];

    let mut model = model_from_segments(&polygon_segments);
    select_polygon(
        &mut model,
        &oristudio_cp::geometry::Polygon::new(polygon.clone()),
    );
    let mut args = vec![
        "foldline-select-polygon".to_string(),
        "select".to_string(),
        "-".to_string(),
        polygon.len().to_string(),
    ];
    push_points_args(&mut args, &polygon);
    args.push(polygon_segments.len().to_string());
    push_segment_args(&mut args, &polygon_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    model.line_segments[2] = model.line_segments[2].with_selected(2);
    unselect_polygon(
        &mut model,
        &oristudio_cp::geometry::Polygon::new(polygon.clone()),
    );
    let mut args = vec![
        "foldline-select-polygon".to_string(),
        "unselect".to_string(),
        "0,2".to_string(),
        polygon.len().to_string(),
    ];
    push_points_args(&mut args, &polygon);
    args.push(polygon_segments.len().to_string());
    push_segment_args(&mut args, &polygon_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let mut box_model = model_from_segments(&polygon_segments);
    select_box(
        &mut box_model,
        &oristudio_cp::geometry::Polygon::new(polygon.clone()),
    );
    let mut args = vec![
        "foldline-select-box".to_string(),
        "select".to_string(),
        "-".to_string(),
        polygon.len().to_string(),
    ];
    push_points_args(&mut args, &polygon);
    args.push(polygon_segments.len().to_string());
    push_segment_args(&mut args, &polygon_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&box_model),
        run_oracle(&oracle, &args)
    );

    let mut lx_model = model_from_segments(&polygon_segments);
    let selection = segment(-0.5, 0.5, 1.5, 0.5, LineColor::Magenta5);
    select_intersecting_line(&mut lx_model, &selection);
    let mut args = vec![
        "foldline-select-lx".to_string(),
        "select".to_string(),
        "-".to_string(),
    ];
    push_one_segment_args(&mut args, &selection);
    args.push(polygon_segments.len().to_string());
    push_segment_args(&mut args, &polygon_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&lx_model),
        run_oracle(&oracle, &args)
    );

    unselect_intersecting_line(&mut lx_model, &selection);
    let mut args = vec![
        "foldline-select-lx".to_string(),
        "unselect".to_string(),
        "0,2".to_string(),
    ];
    push_one_segment_args(&mut args, &selection);
    args.push(polygon_segments.len().to_string());
    push_segment_args(&mut args, &polygon_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&lx_model),
        run_oracle(&oracle, &args)
    );
}

#[test]
fn connected_selection_matches_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(1.0, 0.0, 2.0, 0.0, LineColor::Blue2),
        segment(2.0, 0.0, 3.0, 0.0, LineColor::Black0),
        segment(10.0, 0.0, 11.0, 0.0, LineColor::Cyan3),
    ];
    let mut model = model_from_segments(&segments);
    select_connected_from_point(&mut model, Point::new(1.0, 0.0));

    let mut args = vec![
        "foldline-select-connected".to_string(),
        "1".to_string(),
        "0".to_string(),
        "-".to_string(),
        segments.len().to_string(),
    ];
    push_segment_args(&mut args, &segments);

    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );
}

#[test]
fn transform_commands_match_oriedita_foldlineset_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let translate_segments = vec![
        segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
        segment(0.0, 2.0, 1.0, 2.0, LineColor::Blue2),
    ];
    let mut model = model_from_segments(&translate_segments);
    translate_model(&mut model, 5.0, -2.0);
    let mut args = vec![
        "foldline-translate".to_string(),
        "5".to_string(),
        "-2".to_string(),
        translate_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &translate_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let selected_segments = vec![
        segment(1.0, -1.0, 1.0, 2.0, LineColor::Black0),
        segment(0.0, 0.0, 2.0, 0.0, LineColor::Red1),
    ];
    let mut model = model_from_segments(&selected_segments);
    model.line_segments[1] = model.line_segments[1].with_selected(2);
    move_selected_lines(&mut model, Point::new(0.0, 1.0));
    let mut args = vec![
        "foldline-transform-selected".to_string(),
        "move".to_string(),
        "0".to_string(),
        "1".to_string(),
        "1".to_string(),
        selected_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &selected_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let mut model = model_from_segments(&selected_segments);
    model.line_segments[1] = model.line_segments[1].with_selected(2);
    copy_selected_lines(&mut model, Point::new(0.0, 1.0));
    let mut args = vec![
        "foldline-transform-selected".to_string(),
        "copy".to_string(),
        "0".to_string(),
        "1".to_string(),
        "1".to_string(),
        selected_segments.len().to_string(),
    ];
    push_segment_args(&mut args, &selected_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let four_point_segments = vec![segment(1.0, 0.0, 1.0, 1.0, LineColor::Red1)];
    let original_a = Point::new(0.0, 0.0);
    let original_b = Point::new(1.0, 0.0);
    let target_a = Point::new(2.0, 3.0);
    let target_b = Point::new(4.0, 3.0);

    let mut model = model_from_segments(&four_point_segments);
    model.line_segments[0] = model.line_segments[0].with_selected(2);
    move_selected_lines_by_points(&mut model, original_a, original_b, target_a, target_b);
    let mut args = vec![
        "foldline-transform-selected-4p".to_string(),
        "move".to_string(),
    ];
    push_points_args(&mut args, &[original_a, original_b, target_a, target_b]);
    args.push("0".to_string());
    args.push(four_point_segments.len().to_string());
    push_segment_args(&mut args, &four_point_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );

    let mut model = model_from_segments(&four_point_segments);
    model.line_segments[0] = model.line_segments[0].with_selected(2);
    copy_selected_lines_by_points(&mut model, original_a, original_b, target_a, target_b);
    let mut args = vec![
        "foldline-transform-selected-4p".to_string(),
        "copy".to_string(),
    ];
    push_points_args(&mut args, &[original_a, original_b, target_a, target_b]);
    args.push("0".to_string());
    args.push(four_point_segments.len().to_string());
    push_segment_args(&mut args, &four_point_segments);
    assert_eq!(
        line_segment_set_with_selection_summary(&model),
        run_oracle(&oracle, &args)
    );
}

#[test]
fn extend_to_intersection_matches_oriedita_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    for (source, segments) in [
        (
            segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
            vec![
                segment(5.0, -1.0, 5.0, 1.0, LineColor::Blue2),
                segment(10.0, -1.0, 10.0, 1.0, LineColor::Black0),
            ],
        ),
        (
            segment(0.0, 0.0, 1.0, 0.0, LineColor::Red1),
            vec![segment(5.0, 0.0, 7.0, 0.0, LineColor::Blue2)],
        ),
    ] {
        let model = model_from_segments(&segments);
        let result = extend_to_intersection_point_2(&model, &source);
        let mut args = vec!["foldline-extend-to-intersection".to_string()];
        push_one_segment_args(&mut args, &source);
        args.push(segments.len().to_string());
        push_segment_args(&mut args, &segments);
        assert_eq!(
            optional_segment_result_summary(Some(&result)),
            run_oracle(&oracle, &args)
        );
    }
}

#[test]
fn point_line_division_matches_oriedita_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let count_segment = segment(0.0, 0.0, 2.0, 0.0, LineColor::Red1);
    let count_segments = vec![segment(1.0, -1.0, 1.0, 1.0, LineColor::Black0)];
    let mut count_model = model_from_segments(&count_segments);
    divide_segment_by_count(&mut count_model, &count_segment, 2);

    let mut count_args = vec!["foldline-divide-count".to_string(), "2".to_string()];
    push_one_segment_args(&mut count_args, &count_segment);
    count_args.push(count_segments.len().to_string());
    push_segment_args(&mut count_args, &count_segments);
    assert_eq!(
        line_segment_set_summary(&count_model),
        run_oracle(&oracle, &count_args)
    );

    let ratio_segment = segment(0.0, 0.0, 10.0, 0.0, LineColor::Blue2);
    let mut ratio_model = CreasePatternModel::default();
    divide_segment_by_ratio(&mut ratio_model, &ratio_segment, 1.0, 3.0);

    let mut ratio_args = vec![
        "foldline-divide-ratio".to_string(),
        "1.0".to_string(),
        "3.0".to_string(),
    ];
    push_one_segment_args(&mut ratio_args, &ratio_segment);
    ratio_args.push("0".to_string());
    assert_eq!(
        line_segment_set_summary(&ratio_model),
        run_oracle(&oracle, &ratio_args)
    );

    let zero_ratio_segment = segment(-1.0, 2.0, 3.0, 2.0, LineColor::Cyan3);
    let mut zero_ratio_model = CreasePatternModel::default();
    divide_segment_by_ratio(&mut zero_ratio_model, &zero_ratio_segment, 0.0, 4.0);

    let mut zero_ratio_args = vec![
        "foldline-divide-ratio".to_string(),
        "0.0".to_string(),
        "4.0".to_string(),
    ];
    push_one_segment_args(&mut zero_ratio_args, &zero_ratio_segment);
    zero_ratio_args.push("0".to_string());
    assert_eq!(
        line_segment_set_summary(&zero_ratio_model),
        run_oracle(&oracle, &zero_ratio_args)
    );
}

#[test]
fn draw_point_matches_oriedita_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let segments = vec![segment(0.0, 0.0, 10.0, 0.0, LineColor::Red1)];
    for (target, selection_distance) in [(Point::new(4.0, 0.2), 1.0), (Point::new(4.0, 2.0), 1.0)] {
        let mut model = model_from_segments(&segments);
        let changed = draw_point_on_segment(&mut model, 0, target, selection_distance);
        let mut args = vec![
            "foldline-draw-point".to_string(),
            "0".to_string(),
            target.x.to_string(),
            target.y.to_string(),
            selection_distance.to_string(),
            segments.len().to_string(),
        ];
        push_segment_args(&mut args, &segments);
        let rust_summary = format!("changed|{changed}\n{}", line_segment_set_summary(&model));
        assert_eq!(rust_summary, run_oracle(&oracle, &args));
    }
}

#[test]
fn measure_commands_match_oriedita_oracle() {
    let Some(oracle) = operations_oracle() else {
        eprintln!(
            "skipping Oriedita operations oracle test: ORIEDITA_OPERATIONS_ORACLE is not set"
        );
        return;
    };

    let length_a = Point::new(0.0, 0.0);
    let length_b = Point::new(3.0, 4.0);
    let length = length_between_points(length_a, length_b);
    let mut args = vec!["measure-length".to_string()];
    push_points_args(&mut args, &[length_a, length_b]);
    assert_eq!(
        format!("length|{}\n", java_double_string(length)),
        run_oracle(&oracle, &args)
    );

    let angle_a = Point::new(0.0, 1.0);
    let angle_center = Point::new(0.0, 0.0);
    let angle_b = Point::new(1.0, 0.0);
    let angle = angle_between_three_points(angle_a, angle_center, angle_b);
    let mut args = vec!["measure-angle".to_string()];
    push_points_args(&mut args, &[angle_a, angle_center, angle_b]);
    assert_eq!(
        format!("angle|{}\n", java_double_string(angle)),
        run_oracle(&oracle, &args)
    );
}

fn operations_oracle() -> Option<PathBuf> {
    std::env::var("ORIEDITA_OPERATIONS_ORACLE")
        .or_else(|_| std::env::var("ORIEDITA_GEOMETRY_ORACLE"))
        .ok()
        .map(|oracle| resolve_oracle_path(&oracle))
}

fn resolve_oracle_path(oracle: &str) -> PathBuf {
    let path = PathBuf::from(oracle);
    if path.is_absolute() || path.exists() {
        return path;
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run_oracle(oracle: &Path, args: &[String]) -> String {
    let output = Command::new(oracle)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run Oriedita operations oracle {oracle:?}: {err}"));

    assert!(
        output.status.success(),
        "Oriedita operations oracle failed with status {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("oracle stdout should be valid UTF-8")
}

fn segment(ax: f64, ay: f64, bx: f64, by: f64, color: LineColor) -> LineSegment {
    LineSegment::with_color(Point::new(ax, ay), Point::new(bx, by), color)
}

fn model_from_segments(segments: &[LineSegment]) -> CreasePatternModel {
    let mut model = CreasePatternModel::default();
    for segment in segments {
        model.add_line_segment(segment.clone());
    }
    model
}

fn push_segment_args(args: &mut Vec<String>, segments: &[LineSegment]) {
    for segment in segments {
        push_one_segment_args(args, segment);
    }
}

fn push_one_segment_args(args: &mut Vec<String>, segment: &LineSegment) {
    args.push(segment.a.x.to_string());
    args.push(segment.a.y.to_string());
    args.push(segment.b.x.to_string());
    args.push(segment.b.y.to_string());
    args.push(segment.color.number().to_string());
}

fn push_points_args(args: &mut Vec<String>, points: &[Point]) {
    for point in points {
        args.push(point.x.to_string());
        args.push(point.y.to_string());
    }
}

fn line_segment_set_summary(model: &CreasePatternModel) -> String {
    let mut output = String::new();
    output.push_str(&format!("lines|{}\n", model.line_segments.len()));
    for segment in &model.line_segments {
        output.push_str(&format!(
            "line|{}|{}|{}|{}|{}\n",
            java_double_string(segment.a.x),
            java_double_string(segment.a.y),
            java_double_string(segment.b.x),
            java_double_string(segment.b.y),
            segment.color.number()
        ));
    }
    output
}

fn line_segment_set_with_selection_summary(model: &CreasePatternModel) -> String {
    let mut output = String::new();
    output.push_str(&format!("lines|{}\n", model.line_segments.len()));
    for segment in &model.line_segments {
        output.push_str(&format!(
            "line|{}|{}|{}|{}|{}|{}\n",
            java_double_string(segment.a.x),
            java_double_string(segment.a.y),
            java_double_string(segment.b.x),
            java_double_string(segment.b.y),
            segment.color.number(),
            segment.selected
        ));
    }
    output
}

fn optional_segment_result_summary(segment: Option<&LineSegment>) -> String {
    match segment {
        Some(segment) => format!(
            "result|{}|{}|{}|{}|{}\n",
            java_double_string(segment.a.x),
            java_double_string(segment.a.y),
            java_double_string(segment.b.x),
            java_double_string(segment.b.y),
            segment.color.number()
        ),
        None => "result|null\n".to_string(),
    }
}

fn delete_summary(to_delete: &BTreeSet<usize>) -> String {
    let mut output = String::from("delete");
    for index in to_delete {
        output.push_str(&format!("|{index}"));
    }
    output.push('\n');
    output
}

fn intersection_state(intersection: Intersection) -> i32 {
    intersection.state()
}

fn java_double_string(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
