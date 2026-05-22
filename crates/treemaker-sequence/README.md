# treemaker-sequence

`treemaker-sequence` contains the folding-sequence research surface that sits on
top of `treemaker-fold` and `treemaker-flatfold`.

Phase 1 only resolves a target folded state. It does not synthesize user-facing
folding instructions yet. Missing planner behavior must remain explicit through
typed `NotImplemented`/diagnostic results rather than approximate placeholder
steps.
