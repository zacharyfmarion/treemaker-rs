# Generated CP Editing And Symmetry

## Goal

Make crease patterns generated from the Design pane editable with the existing
crease-pattern tools without making the tree/CP relationship ambiguous. Add
crease-pattern symmetry authoring so point, line, and selection-based CP edits
can be reflected live across a user-defined axis. Preserve existing projects and
make the saved-file contract explicit.

## Approach

### Recommendation

Treat a generated crease pattern as a linked editable companion document, not as
mutable TreeMaker engine output. The tree remains the canonical source for
optimization and rebuilds. The editable CP is a snapshot created from the latest
generated FOLD output and carries lineage back to the tree revision that
produced it.

This gives the user a clear mental model:

- Build CP creates or refreshes an editable CP companion and opens the Crease
  Pattern pane with CP tools available.
- Before any CP edit, the companion is "Generated from design" and in sync with
  the tree revision that produced it.
- The first mutating CP tool command marks it "Customized". The CP is now a
  manual derivative; future tree edits never silently overwrite it.
- Editing the tree after a CP was generated marks the companion "Design changed"
  or "Stale". The stale CP remains viewable and editable as a snapshot.
- Rebuild from the tree replaces a non-customized stale CP without confirmation.
  Replacing a customized CP requires confirmation and should offer a duplicate
  or save-before-replace path.
- Export TreeMaker 4/5 continues to export the tree. Export CP/FOLD exports the
  active editable CP when one exists. Save as `.osf` preserves both the tree and
  the editable CP relationship.

Avoid trying to store CP edits as a patch set over generated TreeMaker output in
the first version. Replaying arbitrary Oriedita operations after a tree rebuild
would be fragile because line IDs, topology, and generated helper creases can
change.

### Generated CP companion

- Add explicit workspace state for a tree document plus optional
  `oristudioCpDocument` companion. Do not use `documentMode` alone to decide
  whether CP tools are available.
- Add a CP lineage state:
  - `kind`: `generated-from-tree`, `imported`, `blank`, or `detached`.
  - `treeDocumentId` for generated companions.
  - `sourceTreeDigest` from the TMD5 text or another deterministic tree
    snapshot digest.
  - `generatedAt`, `manualEditCount`, and optional `sourceGeneratedFold`.
- On `buildCreasePattern`, export the generated tree CP as FOLD, import that
  FOLD into the Oriedita CP runtime, store the companion lineage, and activate
  the CP pane.
- Preserve the TreeMaker snapshot metadata (`tm:creaseSourceIds`,
  `tm:creaseKinds`, facet order/color extras) in lineage or document metadata
  so generated crease/facet inspection can still explain where a line came from
  until manual edits invalidate exact parity.
- Add a visible CP source badge:
  - `Generated from design`
  - `Customized CP`
  - `Design changed`
  - `Detached CP`
- Add rebuild/reset commands:
  - `Rebuild from Design`
  - `Detach CP from Design`
  - `Replace Edited CP...` confirmation when manual edits exist.
- Route fold artifacts by source. In-sync generated CPs can use TreeMaker
  artifacts; customized CPs should use the editable CP FOLD export and flat-fold
  pipeline.

### CP symmetry controls

- Add CP-specific symmetry state, separate from tree conditions:
  - `enabled`
  - `axis: { loc: Point; angle: number }` in Oriedita model coordinates
  - `preset: none | book | diagonal | custom`
  - `showAxis`
  - optional `pairSelection` for selection-scoped commands.
- Default new/imported CP documents to symmetry disabled with an axis through
  the Oriedita paper center. When a generated CP comes from a symmetric tree,
  seed the CP axis from the transformed tree symmetry line, but do not reflect
  commands until the user turns CP symmetry on.
- Add compact controls to the existing CP viewport toolbar/settings strip as a
  single symmetry menu button, so symmetry is colocated with grid, snap, and
  active line type without consuming the full width:
  - toolbar chip showing `Sym`, `Book`, `Diag`, `Custom`, or axis-pick progress.
  - popover toggle switches for symmetry enable and axis visibility.
  - preset actions for `Book` and `Diagonal`, plus flip preset orientation.
  - `Set custom axis` canvas action: pick two snapped points to define a custom
    axis.
- Keep the toolbar narrow by moving secondary symmetry choices into the popover
  rather than adding a persistent row of buttons.
- Render the axis, snap lane, and reflected live preview. Preview should show
  both the original candidate geometry and the mirrored candidate before commit.
- Preserve mountain/valley assignment when reflecting. Add a future advanced
  option for M/V swapping only if real workflows need it.

### Symmetric command execution

- Add shared CP symmetry geometry helpers for reflecting points, segments,
  circles, text positions, and selection boxes.
- Add an explicit symmetry behavior registry for every ready mutating CP
  operation:
  - point-input creation: execute the original payload and a reflected payload.
  - drag-line/drag-box/drag-path tools: reflect the final points/path and show a
    reflected live preview while dragging.
  - selection-scoped changes: expand selected line/circle/text IDs with their
    geometric mirror counterparts before executing.
  - move/copy tools: execute against the original selection and mirrored
    selection with reflected movement points when mirror counterparts exist.
  - diagnostics and global repair operations: run once and display as global,
    not mirrored.
- If a ready mutating operation cannot be mirrored safely yet, the registry must
  say so explicitly. While symmetry is enabled, the UI should either disable
  that action with a clear reason or run it only after the user turns symmetry
  off.
- De-duplicate reflected geometry when the original lies on the symmetry axis.
- After a mirrored command, one undo step should undo the original and reflected
  edits together.

### Saved file format and migration

- Bump the native `.osf` schema to v2.
- Use the existing `workspace.documents[]` shape to save both a tree document
  and its editable CP companion in the same project.
- Add v2 crease-pattern lineage and CP symmetry authoring state. Store CP
  symmetry in the native project because it affects future authoring behavior;
  keep CP/FOLD export focused on geometry unless a later explicit Oriedita/FOLD
  extension is needed.
- Keep `.tmd`, `.tmd4`, `.tmd5`, `.cp`, and `.fold` as import/export formats.
  Do not change TreeMaker's saved file format to carry Ori Studio CP authoring
  state.
- Migrate v1 `.osf` on read:
  - tree-only v1 files become v2 tree projects with no CP companion.
  - CP-only v1 files become standalone editable CP projects with lineage
    `imported` or `blank` and CP symmetry disabled.
  - missing CP symmetry fields default to disabled.
- No one-time rewrite is required. Saving an opened v1 project writes v2.

## Affected Areas

- `apps/web/src/store/workspaceStore`
- `apps/web/src/lib/nativeProjectFile.ts`
- `apps/web/src/lib/creasePatternViewport.ts`
- `apps/web/src/lib/symmetryAuthoring.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CpToolRail.tsx`
- `apps/web/src/components/panels/ConditionsPanel.tsx`
- `apps/web/src/lib/oristudioCpActions.ts`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/commands/menuActions.ts`
- `apps/web/src/menus/menuDefinition.ts`
- `apps/tauri/src-tauri/src/menu.rs`
- `crates/oristudio-cp`
- `crates/oristudio-cp-wasm`
- `crates/treemaker-wasm`

## Checklist

- [x] Split document kind from active editing surface so tree projects can own
      editable CP companions.
- [x] Add generated CP lineage state, stale/customized status derivation, and
      source badges.
- [x] Convert `buildCreasePattern` to create or refresh an Oriedita editable CP
      companion from the generated FOLD output.
- [x] Add rebuild replacement confirmation for customized generated CPs.
- [ ] Add explicit detach/duplicate UX for customized generated CPs.
- [x] Route undo/redo, menu capabilities, exports, and fold artifacts through
      the active editing surface and CP lineage.
- [x] Add CP symmetry state, helpers, and tests.
- [x] Add compact CP toolbar symmetry controls, axis rendering, and the
      two-point `Set custom axis` interaction.
- [x] Add reflected live preview for point, drag-line, drag-box, and drag-path
      CP tools.
- [x] Add generic symmetry payload routing for ready mutating CP commands,
      including point payloads and mirrored line/circle selection IDs.
- [x] Harden CP symmetry routing with explicit per-command policies so
      fixed-arity tools keep their operand contracts under mirroring.
- [x] Execute mirrored command batches as one history entry and de-duplicate
      on-axis geometry.
- [x] Bump `.osf` to schema v2 and add v1-to-v2 migration tests.
- [x] Update save/open/autosave/recents for multi-document tree plus CP
      projects.
- [x] Add focused store, native-file, command-wrapper, and CP panel tests.
- [x] Run `npm run lint:web`, `npm run typecheck:web`, and `npm run test:web`.
