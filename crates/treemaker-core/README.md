# treemaker-core

Pure Rust engine port of the model side of Robert J. Lang's TreeMaker 5.0.1.

This crate is the headless API: it parses TreeMaker v3/v4/v5 stream files,
writes canonical v5, exports v4, checks feasibility, runs the ALM optimizers,
and builds crease-pattern polygons, vertices, creases, facets, fold directions,
and CP status diagnostics.

```rust
use treemaker_core::Tree;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let text = std::fs::read_to_string("model.tmd5")?;
let mut tree = Tree::from_tmd_str(&text)?;

tree.optimize_scale()?;
tree.build_polys_and_crease_pattern()?;

std::fs::write("out.tmd5", tree.to_tmd5_string())?;
# Ok(())
# }
```

The parity baseline is TreeMaker 5.0.1 with the distributable ALM optimizer.
CFSQP/RFSQP are not included because the public TreeMaker 5.0.1 source does not
ship redistributable source for those backends.

License: `GPL-2.0-or-later`.
