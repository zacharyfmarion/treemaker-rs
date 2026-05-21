use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn loads_cp_and_exports_document() {
    let handle = oristudio_cp_wasm::load_cp("1 0 0 1 0\n3 0 0 0 1\n", "sample")
        .expect("cp import should succeed");
    let summary = oristudio_cp_wasm::document_summary(handle).expect("summary should serialize");
    let summary: serde_json::Value =
        serde_wasm_bindgen::from_value(summary).expect("summary should deserialize");

    assert_eq!(summary["title"], "sample");
    assert_eq!(summary["line_segments"], 2);
    assert_eq!(summary["can_save_as_cp"], true);

    let exported = oristudio_cp_wasm::export_cp(handle).expect("cp export should succeed");
    assert!(exported.contains("1 0.0 0.0 1.0 0.0"));
    oristudio_cp_wasm::free_document(handle).expect("document handle should free");
}

#[wasm_bindgen_test]
fn command_dispatch_returns_typed_not_implemented_error() {
    let handle =
        oristudio_cp_wasm::load_cp("1 0 0 1 0\n", "sample").expect("cp import should succeed");
    let error = oristudio_cp_wasm::execute_cp_command(
        handle,
        serde_wasm_bindgen::to_value("DrawCreaseFree").expect("operation id should serialize"),
    )
    .expect_err("registered commands should stay disabled until UI wiring exists");
    let error: serde_json::Value =
        serde_wasm_bindgen::from_value(error).expect("error should deserialize");

    assert_eq!(error["code"], "not_implemented");
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|message| message.contains("DrawCreaseFree"))
    );
    oristudio_cp_wasm::free_document(handle).expect("document handle should free");
}
