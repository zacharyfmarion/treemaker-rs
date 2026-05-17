//! `wasm-bindgen` wrapper around `treemaker-core`.
//!
//! The wasm API stores loaded trees behind integer handles so JavaScript can
//! call into the engine without copying the whole model for every operation.
//! Error values are serialized objects with the same stable `code` values as
//! native [`treemaker_core::TreeError`].

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use treemaker_core::{Tree, TreeDesign, TreeEdit, TreeError};
use wasm_bindgen::prelude::*;

thread_local! {
    static TREES: RefCell<Vec<Option<Tree>>> = const { RefCell::new(Vec::new()) };
}

#[wasm_bindgen]
pub fn load_tmd(text: &str) -> std::result::Result<u32, JsValue> {
    let tree = Tree::from_tmd_str(text).map_err(to_js_error)?;
    store_tree(tree)
}

#[derive(Deserialize)]
struct NewDesignConfig {
    paper_width: Option<f64>,
    paper_height: Option<f64>,
}

#[wasm_bindgen]
pub fn new_design(config: JsValue) -> std::result::Result<u32, JsValue> {
    let config = if config.is_null() || config.is_undefined() {
        NewDesignConfig {
            paper_width: None,
            paper_height: None,
        }
    } else {
        serde_wasm_bindgen::from_value(config).map_err(to_js_value)?
    };
    let tree = Tree::new_design(
        config.paper_width.unwrap_or(1.0),
        config.paper_height.unwrap_or(1.0),
    )
    .map_err(to_js_error)?;
    store_tree(tree)
}

#[wasm_bindgen]
pub fn load_design(design: JsValue) -> std::result::Result<u32, JsValue> {
    let design: TreeDesign = serde_wasm_bindgen::from_value(design).map_err(to_js_value)?;
    let tree = Tree::from_design(design).map_err(to_js_error)?;
    store_tree(tree)
}

fn store_tree(tree: Tree) -> std::result::Result<u32, JsValue> {
    TREES.with(|trees| {
        let mut trees = trees.borrow_mut();
        if let Some((idx, slot)) = trees
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(tree);
            Ok(idx as u32)
        } else {
            trees.push(Some(tree));
            Ok((trees.len() - 1) as u32)
        }
    })
}

#[wasm_bindgen]
pub fn tree_summary(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree(handle, |tree| {
        serde_wasm_bindgen::to_value(&tree.summary()).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn tree_snapshot(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree(handle, |tree| {
        serde_wasm_bindgen::to_value(&tree.snapshot()).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn tree_design(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree(handle, |tree| {
        serde_wasm_bindgen::to_value(&tree.to_design()).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn apply_edit(handle: u32, edit: JsValue) -> std::result::Result<JsValue, JsValue> {
    let edit: TreeEdit = serde_wasm_bindgen::from_value(edit).map_err(to_js_value)?;
    with_tree_mut(handle, |tree| {
        let report = tree.apply_edit(edit).map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&report).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn check(handle: u32) -> std::result::Result<JsValue, JsValue> {
    tree_summary(handle)
}

#[wasm_bindgen]
pub fn cp_status_report(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree(handle, |tree| {
        serde_wasm_bindgen::to_value(&tree.cp_status_report()).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn optimize_scale(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree_mut(handle, |tree| {
        let report = tree.optimize_scale().map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&report).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn optimize_edges(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree_mut(handle, |tree| {
        let report = tree.optimize_edges().map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&report).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn optimize_strain(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree_mut(handle, |tree| {
        let report = tree.optimize_strain().map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&report).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn build_crease_pattern(handle: u32) -> std::result::Result<JsValue, JsValue> {
    with_tree_mut(handle, |tree| {
        tree.build_polys_and_crease_pattern().map_err(to_js_error)?;
        serde_wasm_bindgen::to_value(&tree.summary()).map_err(to_js_value)
    })
}

#[wasm_bindgen]
pub fn save_tmd5(handle: u32) -> std::result::Result<String, JsValue> {
    with_tree(handle, |tree| Ok(tree.to_tmd5_string()))
}

#[wasm_bindgen]
pub fn free_tree(handle: u32) -> std::result::Result<(), JsValue> {
    TREES.with(|trees| {
        let mut trees = trees.borrow_mut();
        let slot = trees
            .get_mut(handle as usize)
            .ok_or_else(|| js_error("invalid_handle", "invalid TreeHandle"))?;
        *slot = None;
        Ok(())
    })
}

fn with_tree<T>(
    handle: u32,
    f: impl FnOnce(&Tree) -> std::result::Result<T, JsValue>,
) -> std::result::Result<T, JsValue> {
    TREES.with(|trees| {
        let trees = trees.borrow();
        let tree = trees
            .get(handle as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| js_error("invalid_handle", "invalid TreeHandle"))?;
        f(tree)
    })
}

fn with_tree_mut<T>(
    handle: u32,
    f: impl FnOnce(&mut Tree) -> std::result::Result<T, JsValue>,
) -> std::result::Result<T, JsValue> {
    TREES.with(|trees| {
        let mut trees = trees.borrow_mut();
        let tree = trees
            .get_mut(handle as usize)
            .and_then(Option::as_mut)
            .ok_or_else(|| js_error("invalid_handle", "invalid TreeHandle"))?;
        f(tree)
    })
}

#[derive(Serialize)]
struct JsErrorEnvelope {
    code: &'static str,
    message: String,
}

fn to_js_error(error: TreeError) -> JsValue {
    js_error(error.code(), error.to_string())
}

fn to_js_value(error: impl std::fmt::Display) -> JsValue {
    js_error("js_value", error.to_string())
}

fn js_error(code: &'static str, message: impl Into<String>) -> JsValue {
    serde_wasm_bindgen::to_value(&JsErrorEnvelope {
        code,
        message: message.into(),
    })
    .unwrap_or_else(|_| JsValue::from_str(code))
}
