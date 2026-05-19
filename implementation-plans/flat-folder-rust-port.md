# Flat-Folder Rust Port

## Goal

Port Jason Ku's Flat-Folder algorithm into a standalone Rust crate and validate
each ported stage against the vendored JavaScript implementation as an oracle.

## Approach

Add a generic `treemaker-flatfold` crate that depends on `treemaker-fold`, vendor
a pinned Flat-Folder source snapshot for oracle runs, and implement the port in
small parity-tested stages. Any not-yet-ported stage returns a typed
`Unimplemented` error rather than an approximate result.

## Affected Areas

- `crates/treemaker-flatfold` for the Rust port.
- `third_party/flat-folder` and `tools/flat-folder-oracle` for oracle parity.
- `crates/oracle-tests` for gated fixture and corpus comparisons.
- `crates/treemaker-cli` for the final command-line entrypoint.

## Checklist

- [x] Stage 0: Scaffold crate, vendor oracle, and add fixture smoke coverage.
- [ ] Stage 1: Port FOLD normalization and topology inference.
- [ ] Stage 2: Port geometry and planar arrangement construction.
- [ ] Stage 3: Port flat folded projection.
- [ ] Stage 4: Port overlap cell graph construction.
- [ ] Stage 5: Port layer-order constraint generation.
- [ ] Stage 6: Port solver and face-order output.
- [ ] Stage 7: Add external corpus oracle harness.
- [ ] Stage 8: Add CLI/docs and run final validation.
