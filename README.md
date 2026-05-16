# tree-maker-rust

Rust/WASM port of the model side of Robert J. Lang's TreeMaker 5.0.1.

This repository vendors the canonical GPL TreeMaker 5.0.1 source under
`third_party/treemaker-5.0.1` and starts the engine-first port as a Rust
workspace:

- `treemaker-core`: parser, serializer, summaries, checks, and headless engine API.
- `treemaker-cli`: useful command-line wrapper around the engine.
- `treemaker-wasm`: `wasm-bindgen`/`wasm-pack` package.
- `oracle-tests`: fixture/parity tests against the vendored TreeMaker data.

Current status: TreeMaker v4/v5 document parsing for the model-only fixture
surface, v5 serialization, v4 export, typed condition parsing/feasibility, ALM
scale/edge/strain optimization, summary/check APIs, fixture tests, CLI, and
wasm bindings are implemented. Full crease-pattern generation is represented in
the public API but returns an explicit unsupported error until those C++
algorithms are ported directly.

## Useful commands

```sh
cargo test --workspace
cargo run -p treemaker-cli -- inspect tests/fixtures/tmModelTester_1.tmd5
cargo run -p treemaker-cli -- check tests/fixtures/tmModelTester_1.tmd5
cargo run -p treemaker-cli -- optimize tests/fixtures/tmModelTester_1.tmd5 --kind scale --out /tmp/out.tmd5
tools/oracle/build_oracle.sh
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test cpp_oracle
wasm-pack build crates/treemaker-wasm --target bundler
```
