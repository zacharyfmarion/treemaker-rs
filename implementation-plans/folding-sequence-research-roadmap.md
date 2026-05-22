# Folding Sequence Research Roadmap

## Goal

Build a verified folding-sequence research track that can turn a TreeMaker or
generic FOLD crease pattern into a human-facing collapse assistant for reaching a
folded base.

The V1 target is not a universal origami diagram generator. It is a verified,
partial, hierarchical planner that can say what it knows, show intermediate
states, and stop with useful diagnostics when the crease pattern needs a
technique outside the current move library.

Reference/precrease construction is out of scope for V1. A future V2 can port
ReferenceFinder-style point and line construction into Rust.

## Approach

Separate the work into four independently testable layers:

1. Target-state resolution: use `treemaker-flatfold` as the Rust reference for
   normalization, flat-folded projection, overlap cells, constraints, solutions,
   and `face_orders`.
2. Sequence state model: represent a planning state as FOLD-compatible topology
   plus active/inactive crease groups, face/layer order, folded positions,
   unresolved regions, and step provenance.
3. Reverse simplification planner: search from the folded target state toward
   the unfolded sheet using graph rewrite rules for known origami techniques,
   then reverse the path for user-facing instructions.
4. Product visualization: expose every accepted step as before/after FOLD frames
   so the web app can render the CP, folded-base preview, and simulator preview
   during research validation.

V1 should be written as new Rust code, probably in a `treemaker-sequence` crate
that depends on `treemaker-fold` and `treemaker-flatfold`. Keep it independent
from `treemaker-core` unless a TreeMaker-specific adapter is needed. This
preserves the current split where reusable FOLD/flat-folding crates can remain
permissively licensed even though the full app is GPL.

The active Oriedita core port in another worktree may become useful for
cross-checking folded-state or CP-editing behavior, but the folding-sequence
roadmap should not depend on it. Treat it as an optional oracle and fixture
source once the APIs settle.

Correctness comes before apparent progress. Any planner stage, move recognizer,
state transform, reference construction, or UI action that is not finished must
return an explicit `not_implemented`, `unsupported`, or typed error diagnostic
instead of producing a lossy placeholder result. Tests should prefer failing
loudly over accepting a bad temporary implementation, and expected artifacts
should mark missing behavior as `status: "not_implemented"` until the behavior
has a validated implementation.

## Affected Areas

- `crates/treemaker-sequence` for the new planner, state model, move library,
  and validation pipeline.
- `crates/treemaker-fold` for shared FOLD extensions if the planner needs
  reusable frame, order, or topology helpers.
- `crates/treemaker-flatfold` for target-state APIs and regression fixtures.
- `crates/treemaker-wasm` for browser-callable sequence generation APIs.
- `apps/web` for research UI, visual previews, diagnostics, and step timeline.
- `tests/fixtures/folding-sequence` for small canonical crease patterns and
  expected planner outputs.
- `crates/oracle-tests` for optional cross-checks against external tools or
  fixture corpora.
- `implementation-plans` and product roadmaps when the research surface becomes
  a committed product feature.

## Checklist

- [x] Phase 0: Define fixtures, correctness contracts, and visual review format.
- [x] Phase 1: Add the sequence crate and target-state adapter over
      `treemaker-flatfold`.
- [ ] Phase 2: Implement sequence-state snapshots and deterministic validators.
- [ ] Phase 3: Build the first reverse rewrite rules for simple folds.
- [ ] Phase 4: Add hierarchical complex moves for common base-collapse
      techniques.
- [ ] Phase 5: Add search, ranking, partial-plan handling, and diagnostics.
- [ ] Phase 6: Expose planner artifacts through WASM and add a research UI.
- [ ] Phase 7: Expand fixture/corpus validation and optional Oriedita cross-checks.
- [ ] Phase 8: Decide whether ML/ranking data collection is warranted.
- [ ] Phase 9: V2 reference/precrease planner integration.

## Phase 0: Fixtures And Contracts

Goal: make the research measurable before implementing planner logic.

Deliverables:

- Add `tests/fixtures/folding-sequence/README.md` describing fixture categories,
  expected outcomes, and how visual snapshots are reviewed.
- Add tiny canonical FOLD fixtures:
  - one simple valley/mountain fold across a square;
  - book fold with multiple faces/layers;
  - kite/rabbit-ear-like local pattern;
  - squash-fold-like local pattern;
  - a small TreeMaker-generated base from existing generated fixtures;
  - an intentionally unsupported simultaneous-collapse case.
- Define serialized planner output:
  - `status`: `complete`, `partial`, `unsupported`, or `invalid_input`;
  - `steps`: ordered hierarchical instruction steps;
  - `states`: before/after FOLD frames or references to frame IDs;
  - `diagnostics`: machine-readable validation and unsupported-rule messages;
  - `unresolved_regions`: face/crease groups remaining after partial planning.
- For Phase 0 only, every expected planner output uses
  `status: "not_implemented"` and a `not_implemented` diagnostic so fixture
  review can proceed without implying the planner exists yet.

Validation:

- Unit tests parse every fixture through `treemaker-fold`.
- Fixture tests solve target states with `treemaker-flatfold`.
- Golden JSON schema tests ensure planner artifacts are stable before the real
  planner exists.
- Visual validation starts with a small script or web-only debug page that renders
  fixture CP, folded target, and empty planned timeline.

Exit criteria:

- Every fixture has a documented expected result: complete, partial, or
  unsupported.
- The expected result includes a human-readable reason, not just a snapshot hash.

## Phase 1: Target-State Adapter

Goal: wrap `treemaker-flatfold` in the exact state shape the planner needs.

Deliverables:

- Add `treemaker-sequence::TargetState`.
- Convert a `FoldDocument` into:
  - normalized FOLD document;
  - folded vertex coordinates;
  - face flip flags;
  - overlap graph;
  - selected `face_orders`;
  - layer-order ambiguity diagnostics.
- Support `solution_limit` and deterministic target selection.
- Preserve all relevant FOLD IDs or provide an explicit ID map back to input IDs.

Validation:

- Unit tests against the Phase 0 fixtures.
- Regression tests against existing `tests/fixtures/flat-folder` cases.
- Property tests for stable ID-map round trips where feasible.
- Oracle parity remains covered by existing `flat_folder_*` tests; do not re-port
  Flat-Folder logic into the sequence crate.

Visual validation:

- Render the normalized CP and selected folded target side by side.
- Show ambiguous layer-order components as overlays or diagnostics.

Exit criteria:

- The planner can reject malformed, unsolved, or ambiguous target states with
  specific diagnostics.
- No sequence logic is needed to inspect the target state visually.

Phase 1 implementation notes:

- Added `crates/treemaker-sequence` as a reusable crate over
  `treemaker-fold` and `treemaker-flatfold`.
- `TargetState` now exposes normalized FOLD data, folded coordinates, face flip
  flags, overlap graph, deterministic first-solution `face_orders`, solution
  counts, constraints, diagnostics, and a best-effort source ID map.
- Ambiguous layer orders produce an `ambiguous_layer_order` diagnostic by
  default and can be rejected with `require_unique_layer_order`.
- The sequence planner itself remains explicitly not implemented through
  `SequenceError::NotImplemented` rather than a placeholder plan.

## Phase 2: Sequence State And Validators

Goal: create a deterministic state machine before adding search.

Deliverables:

- Add `SequenceState` with FOLD-compatible topology, active crease groups,
  face/layer order, folded positions, and provenance.
- Add `InstructionStep` variants:
  - `precrease_region` placeholder;
  - `simple_fold`;
  - `reverse_fold`;
  - `squash_fold`;
  - `rabbit_ear`;
  - `molecule_collapse`;
  - `simultaneous_collapse`;
  - `manual_choice`;
  - `unsupported_region`.
- Add validators:
  - topology references are in bounds;
  - face cycles remain valid;
  - MV assignments remain consistent;
  - layer-order constraints are preserved or intentionally relaxed with a
    diagnostic;
  - generated before/after FOLD frames can be parsed and rendered.

Validation:

- Unit tests for each validator with both passing and failing synthetic states.
- Snapshot tests for serialized `SequenceState` and `InstructionStep` output.
- Fuzz or proptest coverage for invalid indices and malformed face/edge
  references.

Visual validation:

- Render a single static `SequenceState` and expose debug toggles for active
  groups, unresolved regions, and face-order overlays.

Exit criteria:

- Invalid states fail loudly before any search result can be returned.
- Accepted states can always produce a visual artifact.

## Phase 3: Simple-Fold Reverse Rules

Goal: implement the smallest useful reverse simplification path.

Deliverables:

- Model reflection paths in the sequence crate.
- Detect complete reflection paths that correspond to undoing a simple fold.
- Apply a reverse simple-fold rewrite to remove or deactivate the corresponding
  crease group.
- Produce a forward `simple_fold` instruction when the reverse path is inverted.

Validation:

- Unit tests for reflection-path detection.
- Fixture tests for one-layer and multi-layer simple folds.
- Golden output tests assert step count, affected creases, affected faces, and
  before/after frame references.
- Re-run `treemaker-flatfold` on generated intermediate states when the state is
  still intended to be flat-foldable.

Visual validation:

- Step timeline shows the CP before and after each simplification.
- Folded-base preview updates for each accepted step.

Exit criteria:

- At least the simple-fold fixtures produce complete plans.
- Unsupported fixtures stop cleanly with unresolved regions.

## Phase 4: Hierarchical Complex Moves

Goal: support the first practical TreeMaker-base collapse moves without
pretending every action is one crease at a time.

Deliverables:

- Add graph patterns for:
  - reverse fold;
  - squash fold;
  - rabbit ear;
  - small molecule collapse;
  - explicit simultaneous-collapse fallback.
- Keep each move reversible at the artifact level: reverse simplification for
  planning, forward instruction for users.
- Add move metadata:
  - difficulty;
  - whether the move is single-layer, multi-layer, or simultaneous;
  - affected crease and face groups;
  - confidence and validation notes.

Validation:

- One fixture per move type with expected step metadata.
- Negative tests where near-matching topology must not be accepted.
- Validator tests proving a complex move never silently drops unmatched creases.
- Intermediate-state parse/render tests for every accepted move.

Visual validation:

- Research UI highlights the local region for each complex move.
- Each step has a compact before/after thumbnail and an expanded FOLD/folded-base
  view.

Exit criteria:

- The planner can complete at least one small TreeMaker-style base using a mix of
  simple and complex moves.
- Simultaneous moves are labeled honestly rather than decomposed into invalid
  simple folds.

## Phase 5: Search, Ranking, And Partial Plans

Goal: move from hand-picked rule application to useful automated planning.

Deliverables:

- Add bounded beam search or A* over reverse rewrites.
- Add deterministic scoring:
  - fewer unresolved creases;
  - smaller unresolved regions;
  - simpler move types;
  - lower layer-order ambiguity;
  - fewer simultaneous moves;
  - TreeMaker corridor/facet hints when available through an adapter.
- Return partial plans as first-class successful research artifacts.
- Include search statistics in diagnostics: states explored, branches pruned,
  timeout, repeated-state count, and best unresolved score.

Validation:

- Determinism tests: same input/options produces identical plan output.
- Timeout/budget tests: planner returns best partial result rather than hanging.
- Regression tests for branch ordering and tie-breakers.
- Corpus smoke test over a small checked-in fixture set.

Visual validation:

- Debug view can show the chosen path and, optionally, rejected candidate moves
  for the current step.

Exit criteria:

- Search never blocks indefinitely.
- Complete, partial, and unsupported outcomes are stable enough for CI snapshots.

## Phase 6: WASM And Research UI

Goal: make progress visually inspectable inside the app.

Deliverables:

- Add wasm bindings:
  - `sequence_analyze_fold(fold_json, options)`;
  - `sequence_plan_fold(fold_json, options)`;
  - optional `sequence_plan_tree(handle, options)` once TreeMaker adapters are
    useful.
- Add worker methods and Zustand state for sequence artifacts.
- Add a research panel or debug tab with:
  - target-state summary;
  - plan status;
  - step timeline;
  - CP preview;
  - folded-base preview;
  - unresolved-region diagnostics.
- Gate the UI as experimental if product polish is not ready.

Validation:

- wasm-pack Node tests for analyze/plan APIs.
- Web unit tests for worker/store behavior.
- Component tests for complete, partial, unsupported, and invalid-input states.
- `npm run test:web` and targeted wasm tests before merging UI changes.

Visual validation:

- Use the in-app browser to inspect the research panel on at least one complete
  and one partial fixture.
- Capture screenshots during review when a phase changes visual output.

Exit criteria:

- A user can load or generate a CP, run the planner, and inspect every returned
  step without opening developer tools.

## Phase 7: Corpus And Optional Oriedita Cross-Checks

Goal: broaden confidence without making external corpora mandatory for normal CI.

Deliverables:

- Add a checked-in micro-corpus of authorized FOLD fixtures.
- Add optional external corpus harness variables, following the pattern used by
  `flat_folder_corpus.rs`.
- If the Oriedita core port exposes stable folded-state or CP-analysis APIs, add
  optional cross-check tests for:
  - foldability status;
  - face/layer order where comparable;
  - CP topology normalization differences;
  - cases where Oriedita succeeds but the sequence planner only returns partial.

Validation:

- Normal CI runs checked-in micro-corpus only.
- Optional local runs can scan larger corpora and report status buckets.
- Corpus reports distinguish planner limits from target-state solver failures.

Visual validation:

- Add a local-only corpus gallery route or generated HTML report with thumbnails
  for complete, partial, unsupported, and failed cases.

Exit criteria:

- Corpus validation produces actionable buckets rather than a single pass/fail
  number.

## Phase 8: ML Readiness Decision

Goal: decide from data, not vibes, whether ML should enter the architecture.

Deliverables:

- Log search traces in a stable JSONL format:
  - state features;
  - candidate moves;
  - validation results;
  - chosen path;
  - unresolved score changes.
- Define the first ML target as candidate ranking, not geometry validation.
- Draft a small offline experiment plan for a graph-based ranker only if the
  symbolic planner creates enough successful traces.

Validation:

- Trace schema tests.
- Replay tests: recorded traces can be replayed against current validators.
- No production behavior depends on ML output in this phase.

Visual validation:

- Optional debug overlay compares symbolic score versus learned/ranked score once
  an experiment exists.

Exit criteria:

- Either keep ML out of the product, or justify a narrow ranker with real planner
  trace data.

## Phase 9: V2 Reference And Precrease Planning

Goal: connect collapse instructions to physical construction of reference points
and lines.

Deliverables:

- Port ReferenceFinder-style Huzita-Justin construction search into a separate
  module or crate.
- Keep licensing explicit. ReferenceFinder is GPL-2.0, so do not mix ported code
  into permissive crates.
- Add `reference_fold` and `precrease_region` steps that can explain how a user
  locates important lines before collapse.
- Link reference steps to sequence planner requirements: target crease lines,
  landmarks, tolerance, and approximate/exact construction status.

Validation:

- Unit tests for Huzita-Justin operations.
- Golden tests for classic references like thirds, fifths, and simple diagonal
  landmarks.
- Numeric tolerance tests that reflect practical folding precision.
- Visual tests that draw reference construction steps before the collapse
  timeline.

Exit criteria:

- The app can distinguish "we know how to collapse this base" from "we also know
  how a person can precrease the required landmarks."

## Validation Command Guide

Use the smallest validation set that covers the changed phase.

- Sequence crate logic:
  - `cargo test -p treemaker-sequence`
- Shared FOLD or flat-folding changes:
  - `cargo test -p treemaker-fold -p treemaker-flatfold`
- Flat-Folder parity-sensitive changes:
  - `FLATFOLDER_ORACLE=tools/flat-folder-oracle/... cargo test -p oracle-tests --test flat_folder_oracle`
- Optional external corpus:
  - `FLATFOLDER_ORACLE=... FLATFOLDER_CORPUS_DIR=... cargo test -p oracle-tests --test flat_folder_corpus`
- WASM API changes:
  - `wasm-pack build crates/treemaker-wasm --target bundler`
  - `wasm-pack test --node crates/treemaker-wasm`
- Web research UI changes:
  - `npm run lint:web`
  - `npm run typecheck:web`
  - `npm run test:web`
  - `npm run build:web` when bundling or generated wasm output changes.

## Research References

- Akitaya, Mitani, Kanamori, and Fukui: graph rewriting for generating folding
  sequences from flat-foldable crease patterns.
- Jason Ku's Flat-Folder: target-state and layer-order solving; use the Rust
  `treemaker-flatfold` port as the implementation reference.
- FOLD specification: frame, assignment, fold-angle, face, and layer-order data
  model.
- Ramseyer's verifiable origami folding work: motivation and validation model
  for future reference/precrease planning.
- ReferenceFinder: V2 reference construction, not a V1 dependency.
