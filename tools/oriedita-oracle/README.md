# Oriedita Oracle

This directory contains the staged headless oracle harness for the Oriedita
port. Stage 2 only exposes a geometry oracle for primitive line-segment
intersection parity. Later stages should extend the same pattern for documents,
operations, import/export, checks, and folding snapshots.

The oracle intentionally compiles against a pinned Oriedita source checkout
instead of reimplementing the behavior in Rust.

```bash
ORIEDITA_SOURCE=/private/tmp/oriedita-research tools/oriedita-oracle/build_geometry_oracle.sh
ORIEDITA_GEOMETRY_ORACLE=tools/oriedita-oracle/build/oriedita-geometry-oracle \
  cargo test -p oristudio-cp --test oriedita_geometry_oracle
```

If `ORIEDITA_GEOMETRY_ORACLE` is unset, the Rust oracle test exits early so
normal `cargo test -p oristudio-cp` does not require Java.
