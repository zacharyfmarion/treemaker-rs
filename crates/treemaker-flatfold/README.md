# treemaker-flatfold

`treemaker-flatfold` is a Rust port of Jason Ku's Flat-Folder algorithm for
computing flat-folded layer-order states from FOLD crease patterns.

The crate is intentionally independent from TreeMaker's GPL model engine. It
uses `treemaker-fold` for shared FOLD data structures and ports the Flat-Folder
pipeline stage by stage with JavaScript oracle parity tests.

Status: scaffolded port. Unported stages return typed `Unimplemented` errors
instead of degraded approximations.
