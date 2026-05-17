# treemaker-rs

[![CI](https://github.com/zacharyfmarion/treemaker-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/zacharyfmarion/treemaker-rs/actions/workflows/ci.yml)
[![treemaker-core on crates.io](https://img.shields.io/crates/v/treemaker-core.svg)](https://crates.io/crates/treemaker-core)
[![treemaker-core docs](https://docs.rs/treemaker-core/badge.svg)](https://docs.rs/treemaker-core)

`treemaker-rs` is a Rust and WebAssembly port of the engine inside Robert J.
Lang's TreeMaker 5.0.1.

TreeMaker turns a tree structure, such as a stick figure of limbs and branches,
into an origami crease pattern. This project focuses on the engine only: file
I/O, optimization, feasibility checks, and crease-pattern generation. It does
not include the original GUI.

## Install

Use the Rust API:

```sh
cargo add treemaker-core
```

Install the command-line tool:

```sh
cargo install treemaker-cli
```

Use the wasm bindings:

```toml
treemaker-wasm = "0.1"
```

## Quick Start

### Rust

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

### CLI

```sh
treemaker inspect model.tmd5 --format json
treemaker check model.tmd5 --details
treemaker optimize model.tmd5 --kind scale --out optimized.tmd5
treemaker build-cp optimized.tmd5 --out crease-pattern.tmd5
treemaker export-v4 crease-pattern.tmd5 --out legacy.tmd4
```

## Crates

- [`treemaker-core`](https://docs.rs/treemaker-core): native Rust engine API.
- [`treemaker-cli`](https://crates.io/crates/treemaker-cli): headless command-line tool.
- [`treemaker-wasm`](https://docs.rs/treemaker-wasm): `wasm-bindgen` wrapper for browser or Node workflows.

The main API entry point is [`Tree`](https://docs.rs/treemaker-core/latest/treemaker_core/struct.Tree.html).

## Web App

A browser-first GUI is being built in this repository so the app can evolve
alongside the Rust/WASM engine APIs it needs. The current plan and phase status
live in [`WEB_ROADMAP.md`](WEB_ROADMAP.md).

## What Works Today

Version `0.1.0` supports the TreeMaker 5.0.1 model engine surface:

- Read TreeMaker v3, v4, and v5 files.
- Write canonical v5 files and export v4 files.
- Inspect summaries and crease-pattern status.
- Run the ALM scale, edge-strain, and strain optimizers.
- Build polygons, vertices, creases, facets, fold directions, and facet order.
- Use the engine from native Rust, a CLI, or wasm.

The parity baseline is the public TreeMaker 5.0.1 source with its distributable
ALM optimizer. CFSQP and RFSQP are not included because the public TreeMaker
5.0.1 source does not include redistributable source for those optimizer
backends.

## Confidence

This port is tested against a C++ oracle built from the vendored TreeMaker
5.0.1 source. CI currently checks:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/oracle/build_oracle.sh
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test cpp_oracle
```

There are also generated family tests, wasm tests, and an external corpus
harness for private/user `.tmd`, `.tmd4`, and `.tmd5` collections.

```sh
treemaker corpus /path/to/private/corpus --oracle tools/oracle/build/treemaker-oracle
```

No real-user corpus files are committed to this repository.

## Development

Useful local checks:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
wasm-pack build crates/treemaker-wasm --target bundler
wasm-pack test --node crates/treemaker-wasm
```

Release steps live in [`RELEASE.md`](RELEASE.md). Porting notes live in
[`PORTING.md`](PORTING.md).

## License

This project is `GPL-2.0-or-later` because it is a direct Rust port of
TreeMaker's GPL model code. See [`LICENSING.md`](LICENSING.md) for the full
licensing guide, including optimizer backend notes and dependency inventory.
