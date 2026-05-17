use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use treemaker_wasm::{
    build_crease_pattern, cp_status_report, free_tree, load_tmd, optimize_scale, save_tmd5,
    tree_summary,
};

const FIXTURE_1: &str = include_str!("../testdata/tmModelTester_1.tmd5");

#[wasm_bindgen_test]
fn load_optimize_build_save_and_free() {
    let handle = load_tmd(FIXTURE_1).expect("load fixture");

    let summary = json(tree_summary(handle).expect("summary"));
    assert_eq!(summary["nodes"], 4);
    assert_eq!(summary["is_feasible"], true);

    let report = json(optimize_scale(handle).expect("optimize scale"));
    assert_eq!(report["kind"], "scale");
    assert_eq!(report["converged"], true);
    free_tree(handle).expect("free optimized handle");

    let build_handle = load_tmd(FIXTURE_1).expect("reload fixture");
    let built = json(build_crease_pattern(build_handle).expect("build cp"));
    assert_eq!(built["vertices"], 4);
    assert_eq!(built["creases"], 6);
    assert_eq!(built["facets"], 3);
    let report = json(cp_status_report(build_handle).expect("cp status report"));
    assert_eq!(report["status"], "polys_multiple_ibps");
    assert_eq!(report["bad_polys"][0], 1);

    let saved = save_tmd5(build_handle).expect("save tmd5");
    assert!(saved.starts_with("tree\n5.0\n"));

    free_tree(build_handle).expect("free handle");
    let err = tree_summary(build_handle).expect_err("freed handle should error");
    assert_js_error(&err, "invalid_handle", "invalid TreeHandle");
}

#[wasm_bindgen_test]
fn parse_errors_are_structured() {
    let err = load_tmd("not-a-tree").expect_err("invalid input should fail");
    assert_js_error(&err, "parse", "expected tag");
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("json value")
}

fn assert_js_error(value: &JsValue, code: &str, message_fragment: &str) {
    let error = json(value.clone());
    assert_eq!(error["code"], code);
    assert!(
        error["message"]
            .as_str()
            .expect("error message")
            .contains(message_fragment),
        "{error:?}"
    );
}
