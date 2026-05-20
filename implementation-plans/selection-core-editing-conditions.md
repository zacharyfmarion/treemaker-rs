# Selection, Core Editing, And Conditions Completion

## Goal

Finish the product blocks called out in `PRODUCT_ROADMAP.md` and
`WEB_ROADMAP.md` for selection/history completion, original TreeMaker core
tree editing tools, and condition editing/diagnostics.

## Approach

Deliver the work in staged commits so each behavioral slice has matching tests
and validation. Keep product commands in the shared web app and engine behavior
in `treemaker-core`/`treemaker-wasm`, with the Tauri shell limited to native
menu wiring.

## Affected Areas

- Roadmap docs and this implementation plan.
- `crates/treemaker-core` tree editing APIs and tests.
- `crates/treemaker-wasm` edit bridge and TypeScript bindings.
- `apps/web` commands, menus, stores, inspectors, and panels.
- `apps/tauri` native menu definitions for shared command IDs.

## Checklist

- [x] Refresh roadmap docs for confirmed engine confidence and active product
      gaps.
- [ ] Add select-by-index UI, movable-part selection, corridor-facet selection,
      and condition multi-selection polish.
- [ ] Add core tree editing operations: make root, split edge, set/scale
      lengths, renormalize, absorb nodes/edges, strain tools, perturb tools,
      stub tools, and triangulate tree.
- [ ] Add dedicated editors for every condition type, scoped condition removal,
      and richer feasibility diagnostics.
- [ ] Run staged tests and final validation.
- [ ] Push the branch, open a draft PR, and start the local app.
