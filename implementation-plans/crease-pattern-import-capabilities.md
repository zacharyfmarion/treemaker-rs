# Crease Pattern Import Capabilities

## Goal

Add standalone `.fold` and `.cp` import for crease-pattern-only documents while
centralizing command gating around document mode.

## Approach

- Add document-mode state and imported crease-pattern metadata to the shared
  workspace store.
- Add a workspace capability helper that is the policy source for menus,
  toolbar actions, empty states, and store command guards.
- Parse `.fold` and ORIPA-style `.cp` files into normalized FOLD/display
  geometry, with topology inference when faces are missing.
- Route imported crease patterns through the Rust Flat-Folder WASM API so
  folded-base layer-order output is available for CP-only documents.
- Update panes and exports so CP-only documents can be viewed, simulated, and
  exported without exposing tree-only actions.

## Affected Areas

- `apps/web/src/lib`
- `apps/web/src/store/workspaceStore`
- `apps/web/src/components`
- `apps/web/src/commands`
- `apps/web/src/menus`
- `crates/treemaker-wasm`

## Checklist

- [x] Add capability and imported-pattern model helpers.
- [x] Wire document mode through store load/open/export/action guards.
- [x] Update toolbar, menus, panels, and empty states to use capabilities.
- [x] Add `.fold`/`.cp` parser and topology tests.
- [x] Add capability, command, store, and panel regression tests.
- [x] Expose Rust Flat-Folder artifacts through WASM and the web worker.
- [x] Render imported CP folded-base output instead of the previous placeholder.
- [x] Add imported folded-base regression coverage.
- [x] Run targeted Rust/WASM/web validation.
