# treemaker-rs

Rust/WASM port of the model side of Robert J. Lang's TreeMaker 5.0.1.

This repository vendors the canonical GPL TreeMaker 5.0.1 source under
`third_party/treemaker-5.0.1` and starts the engine-first port as a Rust
workspace:

- `treemaker-core`: parser, serializer, summaries, checks, and headless engine API.
- `treemaker-cli`: useful command-line wrapper around the engine.
- `treemaker-wasm`: `wasm-bindgen`/`wasm-pack` package.
- `oracle-tests`: fixture/parity tests against the vendored TreeMaker data.

Current status: TreeMaker v3/v4/v5 document parsing for the model-only fixture
surface, v5 serialization, v4 export, typed condition parsing/feasibility, ALM
scale/edge/strain optimization, polygon construction, crease-pattern
generation, `CPStatus`, summary/check APIs, fixture tests, CLI, and wasm
bindings are implemented and checked against the vendored C++ oracle on the
checked-in fixtures and deterministic generated families. External corpus and
bounded stress harnesses are included for broader local confidence.

The parity baseline is TreeMaker 5.0.1's distributable ALM backend. The
proprietary CFSQP/RFSQP optimizer backends are intentionally out of scope
unless redistributable source and license terms are provided.

## Licensing and Optimizer Backends

This port is intended to remain GPL-compatible and is currently declared as
`GPL-2.0-or-later` in the Cargo workspace. The vendored TreeMaker 5.0.1 source
is distributed with the GNU GPL v2 license text, and the Free Software
Directory records TreeMaker as `GPLv2orlater`.

The Rust crates are direct ports/translations of the TreeMaker model code, so
publishing this repository should preserve the GPL notices and provide the
corresponding source for any distributed binaries or wasm packages.

TreeMaker's nonlinear optimizer abstraction had adapters for multiple backends:

- `ALM`: Augmented Lagrangian Multiplier code written for TreeMaker and enabled
  by default in the public TreeMaker 5.0.1 source. This is the Rust parity
  baseline.
- `CFSQP`: a faster external FSQP optimizer used by TreeMaker 4, but not
  redistributable with the public TreeMaker 5 source.
- `RFSQP`: an evaluation/proprietary FSQP-family backend noted in TreeMaker's
  source comments as not redistributable.
- `wnlib`: freely distributable but not enabled by default in TreeMaker 5.0.1,
  and historically less reliable for complete convergence in Lang's notes.

CFSQP/RFSQP would only affect the numerical optimization step. They do not
change the file format, tree data structures, polygon construction, or crease
pattern construction algorithms. If redistributable source and compatible
license terms are provided later, they can be added as optional optimizer
backends, but they are not required for 5.0.1 ALM parity.

See `LICENSING.md` for the full repository licensing guide, including vendored
source notices, optimizer backend status, publishing obligations, and the Rust
dependency license inventory.

## Useful commands

```sh
cargo test --workspace
cargo run -p treemaker-cli -- inspect tests/fixtures/tmModelTester_1.tmd5
cargo run -p treemaker-cli -- check tests/fixtures/tmModelTester_1.tmd5
cargo run -p treemaker-cli -- check tests/fixtures/tmModelTester_1.tmd5 --details --format json
cargo run -p treemaker-cli -- optimize tests/fixtures/tmModelTester_1.tmd5 --kind scale --out /tmp/out.tmd5
cargo run -p treemaker-cli -- build-cp tests/fixtures/tmModelTester_1.tmd5 --out /tmp/cp.tmd5
cargo run -p treemaker-cli -- export-v4 tests/fixtures/tmModelTester_1.tmd5 --out /tmp/out.tmd4
cargo run -p treemaker-cli -- corpus tests/fixtures --format json
cargo run -p treemaker-cli -- corpus /path/to/private/corpus --oracle tools/oracle/build/treemaker-oracle
tools/oracle/build_oracle.sh
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test cpp_oracle
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test generated_families
TREEMAKER_CORPUS_DIR=/path/to/private/corpus TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test corpus -- --nocapture
wasm-pack build crates/treemaker-wasm --target bundler
wasm-pack test --node crates/treemaker-wasm
```

The default GitHub Actions workflow is intentionally native-only on Ubuntu:
format, clippy, workspace tests, C++ oracle build, and Linux oracle parity.
WASM builds remain an explicit local/release check rather than default CI.

## Rust API

Add the engine crate to a Rust project:

```sh
cargo add treemaker-core
```

```rust
use treemaker_core::Tree;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let text = std::fs::read_to_string("model.tmd5")?;
let mut tree = Tree::from_tmd_str(&text)?;

println!("{:#?}", tree.summary());

tree.optimize_scale()?;
tree.build_polys_and_crease_pattern()?;

std::fs::write("out.tmd5", tree.to_tmd5_string())?;
# Ok(())
# }
```

The public API is intentionally engine-shaped and parity-first. Model records
such as nodes, edges, paths, polygons, vertices, creases, facets, and conditions
are exposed as typed Rust structs with TreeMaker's 1-based external indices
preserved for file compatibility.

## Publishing

The intended crates.io order is:

```sh
cargo publish -p treemaker-core --dry-run
cargo publish -p treemaker-core
cargo publish -p treemaker-cli --dry-run
cargo publish -p treemaker-cli
cargo publish -p treemaker-wasm --dry-run
cargo publish -p treemaker-wasm
```

`oracle-tests` is an internal workspace crate and is not published. For npm,
publish the generated `treemaker-wasm` package separately through `wasm-pack`
after the Rust release is tagged.

See `RELEASE.md` for the full release checklist and intervention points.
