# treemaker-wasm

`wasm-bindgen` package for the TreeMaker 5.0.1 Rust engine port.

The API loads trees into integer handles and exposes summary, check, optimizer,
crease-pattern build, save, and free operations. JavaScript errors are returned
as structured values containing a stable `code` plus a human-readable message.

```sh
wasm-pack build crates/treemaker-wasm --target bundler
wasm-pack test --node crates/treemaker-wasm
```

For native Rust use, prefer `treemaker-core`. For command-line use, prefer
`treemaker-cli`.

License: `GPL-2.0-or-later`.
