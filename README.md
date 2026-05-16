# tree-maker-rust

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
checked-in fixtures.

The parity baseline is TreeMaker 5.0.1's distributable ALM backend. The
proprietary CFSQP/RFSQP optimizer backends are intentionally out of scope
unless redistributable source and license terms are provided.

## Useful commands

```sh
cargo test --workspace
cargo run -p treemaker-cli -- inspect tests/fixtures/tmModelTester_1.tmd5
cargo run -p treemaker-cli -- check tests/fixtures/tmModelTester_1.tmd5
cargo run -p treemaker-cli -- optimize tests/fixtures/tmModelTester_1.tmd5 --kind scale --out /tmp/out.tmd5
cargo run -p treemaker-cli -- build-cp tests/fixtures/tmModelTester_1.tmd5 --out /tmp/cp.tmd5
cargo run -p treemaker-cli -- export-v4 tests/fixtures/tmModelTester_1.tmd5 --out /tmp/out.tmd4
tools/oracle/build_oracle.sh
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test cpp_oracle
wasm-pack build crates/treemaker-wasm --target bundler
wasm-pack test --node crates/treemaker-wasm
```
