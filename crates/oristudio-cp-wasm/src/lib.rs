//! `wasm-bindgen` wrapper around `oristudio-cp`.
//!
//! The wasm API stores editable crease-pattern documents behind integer
//! handles so the web worker can keep command calls cheap and explicit.

use oristudio_cp::{
    CommandError, CreasePatternCommand, CreasePatternDocument, OperationCategory,
    OperationDescriptor, OperationId, OperationStatus, execute_command, io, operation_descriptors,
};
use serde::Serialize;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static DOCUMENTS: RefCell<Vec<Option<CreasePatternDocument>>> = const { RefCell::new(Vec::new()) };
}

#[derive(Serialize)]
struct JsErrorEnvelope {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct JsOperationDescriptor {
    id: OperationId,
    upstream: &'static str,
    target: &'static str,
    category: OperationCategory,
    stage: u8,
    status: OperationStatus,
}

impl From<&'static OperationDescriptor> for JsOperationDescriptor {
    fn from(descriptor: &'static OperationDescriptor) -> Self {
        Self {
            id: descriptor.id,
            upstream: descriptor.upstream,
            target: descriptor.target,
            category: descriptor.category,
            stage: descriptor.stage,
            status: descriptor.status,
        }
    }
}

#[derive(Serialize)]
struct DocumentSummary {
    title: Option<String>,
    line_segments: usize,
    circles: usize,
    points: usize,
    aux_line_segments: usize,
    texts: usize,
    can_save_as_cp: bool,
    is_empty: bool,
}

#[wasm_bindgen]
pub fn cp_operation_descriptors() -> Result<JsValue, JsValue> {
    let descriptors = operation_descriptors()
        .iter()
        .map(JsOperationDescriptor::from)
        .collect::<Vec<_>>();
    to_js_value(&descriptors)
}

#[wasm_bindgen]
pub fn load_cp(text: &str, title: &str) -> Result<u32, JsValue> {
    let model = io::cp::import_cp_str(text).map_err(to_js_io_error)?;
    store_document(CreasePatternDocument {
        title: title_from_js(title),
        crease_pattern: model,
        metadata: Default::default(),
    })
}

#[wasm_bindgen]
pub fn load_fold(text: &str, title: &str) -> Result<u32, JsValue> {
    let model = io::fold::import_fold_json(text).map_err(to_js_io_error)?;
    store_document(CreasePatternDocument {
        title: title_from_js(title),
        crease_pattern: model,
        metadata: Default::default(),
    })
}

#[wasm_bindgen]
pub fn load_document(document: JsValue) -> Result<u32, JsValue> {
    let document: CreasePatternDocument =
        serde_wasm_bindgen::from_value(document).map_err(to_js_value_error)?;
    store_document(document)
}

#[wasm_bindgen]
pub fn document_snapshot(handle: u32) -> Result<JsValue, JsValue> {
    with_document(handle, |document| to_js_value(document))
}

#[wasm_bindgen]
pub fn document_summary(handle: u32) -> Result<JsValue, JsValue> {
    with_document(handle, |document| {
        let model = &document.crease_pattern;
        to_js_value(&DocumentSummary {
            title: document.title.clone(),
            line_segments: model.line_segments.len(),
            circles: model.circles.len(),
            points: model.points.len(),
            aux_line_segments: model.aux_line_segments.len(),
            texts: model.texts.len(),
            can_save_as_cp: model.can_save_as_cp(),
            is_empty: model.is_empty(),
        })
    })
}

#[wasm_bindgen]
pub fn execute_cp_command(handle: u32, operation: JsValue) -> Result<JsValue, JsValue> {
    let operation: OperationId =
        serde_wasm_bindgen::from_value(operation).map_err(to_js_value_error)?;
    with_document_mut(handle, |document| {
        let result = execute_command(document, CreasePatternCommand::new(operation))
            .map_err(to_js_command_error)?;
        to_js_value(&result)
    })
}

#[wasm_bindgen]
pub fn export_cp(handle: u32) -> Result<String, JsValue> {
    with_document(handle, |document| {
        Ok(io::cp::export_cp_string(&document.crease_pattern))
    })
}

#[wasm_bindgen]
pub fn export_fold(handle: u32) -> Result<String, JsValue> {
    with_document(handle, |document| {
        io::fold::export_fold_json(&document.crease_pattern, document.title.clone())
            .map_err(to_js_io_error)
    })
}

#[wasm_bindgen]
pub fn free_document(handle: u32) -> Result<(), JsValue> {
    DOCUMENTS.with(|documents| {
        let mut documents = documents.borrow_mut();
        let slot = documents
            .get_mut(handle as usize)
            .ok_or_else(|| js_error("invalid_handle", "invalid CreasePatternDocument handle"))?;
        *slot = None;
        Ok(())
    })
}

fn store_document(document: CreasePatternDocument) -> Result<u32, JsValue> {
    DOCUMENTS.with(|documents| {
        let mut documents = documents.borrow_mut();
        if let Some((index, slot)) = documents
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(document);
            Ok(index as u32)
        } else {
            documents.push(Some(document));
            Ok((documents.len() - 1) as u32)
        }
    })
}

fn with_document<T>(
    handle: u32,
    f: impl FnOnce(&CreasePatternDocument) -> Result<T, JsValue>,
) -> Result<T, JsValue> {
    DOCUMENTS.with(|documents| {
        let documents = documents.borrow();
        let document = documents
            .get(handle as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| js_error("invalid_handle", "invalid CreasePatternDocument handle"))?;
        f(document)
    })
}

fn with_document_mut<T>(
    handle: u32,
    f: impl FnOnce(&mut CreasePatternDocument) -> Result<T, JsValue>,
) -> Result<T, JsValue> {
    DOCUMENTS.with(|documents| {
        let mut documents = documents.borrow_mut();
        let document = documents
            .get_mut(handle as usize)
            .and_then(Option::as_mut)
            .ok_or_else(|| js_error("invalid_handle", "invalid CreasePatternDocument handle"))?;
        f(document)
    })
}

fn title_from_js(title: &str) -> Option<String> {
    let title = title.trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_owned())
    }
}

fn to_js_command_error(error: CommandError) -> JsValue {
    let code = match error {
        CommandError::UnsupportedOperation { .. } => "unsupported_operation",
        CommandError::NotImplemented { .. } => "not_implemented",
        CommandError::InvalidInput { .. } => "invalid_input",
    };
    js_error(code, error.to_string())
}

fn to_js_io_error(error: io::IoError) -> JsValue {
    js_error("invalid_input", error.to_string())
}

fn to_js_value_error(error: impl std::fmt::Display) -> JsValue {
    js_error("js_value", error.to_string())
}

fn to_js_value(value: &impl Serialize) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value.serialize(&serializer).map_err(to_js_value_error)
}

fn js_error(code: &'static str, message: impl Into<String>) -> JsValue {
    serde_wasm_bindgen::to_value(&JsErrorEnvelope {
        code,
        message: message.into(),
    })
    .unwrap_or_else(|_| JsValue::from_str("failed to serialize wasm error"))
}
