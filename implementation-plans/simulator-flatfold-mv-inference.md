# Simulator Flat-Folder M/V Inference

## Goal

Drive imported crease-pattern simulation from Flat-Folder-inferred mountain and
valley assignments when source creases are unassigned, without changing the
canonical imported FOLD document shown by the UI or exported by the app.

## Approach

- Add a reusable `treemaker-flatfold` helper that derives edge assignments from
  solved `face_orders` using Flat-Folder's GUI convention.
- Use the helper only for the simulator-prepared FOLD clone returned inside
  `simulation_model`.
- Keep `foldArtifacts.fold`, folded-base data, crease-pattern display, and
  imported FOLD export faithful to the original normalized assignments.

## Affected Areas

- `crates/treemaker-flatfold`
- `crates/treemaker-wasm`
- `crates/treemaker-wasm/tests`

## Checklist

- [x] Add inferred-assignment helper and focused Rust unit tests.
- [x] Use inferred assignments only for imported simulation artifacts.
- [x] Extend wasm tests for canonical-versus-simulator assignment behavior.
- [x] Run targeted validation.
