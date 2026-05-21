# Oriedita Oracle

This directory contains the staged headless oracle harness for the Oriedita
port. It currently exposes geometry/model primitive commands, Stage 4
import/export commands for Oriedita's ORH, OBJ, and DXF paths, and Stage 5
arrangement commands for `IntersectDivide`, `FoldLineSet` insertion splitting,
delete-inside-line modes, and `BranchTrim`/`del_V` cleanup. Later stages should
extend the same pattern for documents, operations, checks, and folding
snapshots. The vertex cleanup commands include the same-color, color-changing,
single-pair, and all-vertex variants. The arrangement repair commands include
the legacy `Fix1`/`Fix2` workers. The first color commands cover
`FoldLineSet.setColor(Collection, LineColor)` and mountain/valley toggling.
The first selection commands cover select/unselect all, index selection, box
selection via `lineSegmentsInside`, polygon selection via `select_Takakukei`,
line-overlap/intersection selection via `select_lX`, and
`selectProbablyConnected`.

The oracle intentionally compiles against a pinned Oriedita source checkout
instead of reimplementing the behavior in Rust.

```bash
ORIEDITA_SOURCE=/private/tmp/oriedita-research tools/oriedita-oracle/build_geometry_oracle.sh
ORIEDITA_GEOMETRY_ORACLE=tools/oriedita-oracle/build/oriedita-geometry-oracle \
  cargo test -p oristudio-cp \
    --test oriedita_geometry_oracle \
    --test oriedita_model_oracle \
    --test oriedita_io_oracle \
    --test oriedita_operations_oracle
```

`ORIEDITA_IO_ORACLE` and `ORIEDITA_OPERATIONS_ORACLE` can also point at the
same built wrapper for scoped parity runs. If the oracle environment variables
are unset, the Rust oracle tests exit early so normal `cargo test -p
oristudio-cp` does not require Java.
