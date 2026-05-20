# Ori Studio

[![CI](https://github.com/zacharyfmarion/ori-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/zacharyfmarion/ori-studio/actions/workflows/ci.yml)
[![treemaker-core on crates.io](https://img.shields.io/crates/v/treemaker-core.svg)](https://crates.io/crates/treemaker-core)
[![treemaker-core docs](https://docs.rs/treemaker-core/badge.svg)](https://docs.rs/treemaker-core)

Ori Studio is an origami design environment for turning a tree structure into a
crease pattern. It combines a modern web and desktop interface with a Rust and
WebAssembly port of the model engine from Robert J. Lang's TreeMaker 5.0.1.

The app is built for interactive design work: create or open TreeMaker projects,
inspect the underlying tree, run optimization passes, build crease patterns,
export compatible files, and preview folded results. The original TreeMaker
behavior remains the parity reference for the engine, while the shared frontend
is evolving into a pane-based design workspace for browser and desktop use.

## Applications

- `apps/web`: the shared React and Vite frontend used by the browser app and the
  Tauri shell.
- `apps/tauri`: the Tauri v2 desktop wrapper for native menus, dialogs, window
  metadata, and packaging.

## Exposed Packages and Crates

### Rust crates

- [`treemaker-core`](https://docs.rs/treemaker-core): the native Rust engine API
  for TreeMaker model files, optimization, feasibility checks, crease-pattern
  generation, FOLD conversion, and simulation preparation.
- [`treemaker-cli`](https://crates.io/crates/treemaker-cli): the `treemaker`
  command-line tool for inspecting, checking, optimizing, and exporting models.
- [`treemaker-wasm`](https://docs.rs/treemaker-wasm): `wasm-bindgen` bindings
  that expose the engine to browser and Node workflows.
- [`treemaker-fold`](https://docs.rs/treemaker-fold): generic FOLD data
  structures and geometry helpers for origami applications.
- [`treemaker-flatfold`](https://docs.rs/treemaker-flatfold): flat-foldability
  and layer-order solving for FOLD crease patterns.

The main TreeMaker engine entry point is
[`Tree`](https://docs.rs/treemaker-core/latest/treemaker_core/struct.Tree.html).

### npm workspace packages

- `@treemaker/web`: the private workspace package for the shared Ori Studio web
  app.
- `@treemaker/tauri`: the private workspace package for the desktop shell.
- `@treemaker/origami-simulator`: the private workspace package that adapts
  Origami Simulator-style folding utilities for FOLD inputs.

## Engine Capability

Version `0.1.0` supports the TreeMaker 5.0.1 model engine surface:

- Read TreeMaker v3, v4, and v5 files.
- Write canonical v5 files and export v4 files.
- Inspect summaries and crease-pattern status.
- Run the ALM scale, edge-strain, and strain optimizers.
- Build polygons, vertices, creases, facets, fold directions, and facet order.
- Use the engine from native Rust, a CLI, or WebAssembly.

The parity baseline is the public TreeMaker 5.0.1 source with its distributable
ALM optimizer. CFSQP and RFSQP are not included because the public TreeMaker
5.0.1 source does not include redistributable source for those optimizer
backends.

## Getting Started

Install dependencies:

```sh
npm ci
```

Run the web app:

```sh
npm run dev:web
```

Run the desktop app:

```sh
npm run dev:desktop
```

Use the Rust API:

```sh
cargo add treemaker-core
```

Install the command-line tool:

```sh
cargo install treemaker-cli
```

Use the WebAssembly bindings:

```toml
treemaker-wasm = "0.1"
```

## Confidence

The engine is tested against a C++ oracle built from the vendored TreeMaker
5.0.1 source. CI checks the Rust workspace, web client, generated WebAssembly
bindings used by the web client, and oracle parity suite. An external corpus
harness is available for private/user `.tmd`, `.tmd4`, and `.tmd5` collections,
but no real-user corpus files are committed to this repository.

## Development

Useful local checks:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run lint:web
npm run typecheck:web
npm run test:web
wasm-pack build crates/treemaker-wasm --target bundler
wasm-pack test --node crates/treemaker-wasm
```

Roadmaps live in [`WEB_ROADMAP.md`](WEB_ROADMAP.md) and
[`PRODUCT_ROADMAP.md`](PRODUCT_ROADMAP.md). Release steps live in
[`RELEASE.md`](RELEASE.md). Porting notes live in [`PORTING.md`](PORTING.md).

## License

This project is `GPL-2.0-or-later` because it includes a direct Rust port of
TreeMaker's GPL model code. See [`LICENSING.md`](LICENSING.md) for the full
licensing guide, including optimizer backend notes and dependency inventory.
