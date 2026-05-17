# Licensing

This file summarizes the licensing posture of this repository. It is a
developer-facing guide, not legal advice.

## Project License

The Rust workspace is declared as `GPL-2.0-or-later` in `Cargo.toml`.

This is a direct Rust/WASM port of the model side of Robert J. Lang's
TreeMaker 5.0.1, so the Rust implementation should be treated as a derivative
work of TreeMaker's GPL model code. Do not relicense this port as MIT, Apache,
BSD, proprietary, or another non-GPL license unless the relevant copyright
holders explicitly grant that permission.

The root `LICENSE.txt` contains the GPL v2 license text shipped with
TreeMaker 5.0.1. The Free Software Directory also records TreeMaker as
`GPLv2orlater`.

If you distribute binaries, wasm packages, npm packages, or other object-code
forms built from this repository, the GPL requires that recipients receive or
can obtain the corresponding source code under the same GPL terms.

## What Is Covered

| Path or artifact | License / status | Notes |
| --- | --- | --- |
| Rust workspace crates | `GPL-2.0-or-later` | `treemaker-core`, `treemaker-cli`, `treemaker-wasm`, and `oracle-tests`. |
| `LICENSE.txt` | GPL v2 text from TreeMaker 5.0.1 | Keep this file in source distributions. |
| `third_party/treemaker-5.0.1` | TreeMaker GPL source distribution | Vendored as the behavioral baseline and C++ oracle source. Preserve notices. |
| `third_party/treemaker-5.0.1/Source/tmModel/wnlib` | Unrestricted per TreeMaker's bundled license notice | The TreeMaker license file says the `wnlib` directory may be distributed with no restrictions. |
| `tests/fixtures` | GPL-compatible TreeMaker fixture data | Fixtures are copied or generated from the TreeMaker parity workflow; keep them with the GPL source distribution. |
| `crates/treemaker-wasm/LICENSE.txt` | GPL v2 text | Included so the generated wasm/npm package carries the license text. |
| `crates/treemaker-wasm/pkg` | Generated GPL package output | Ignored by git; if published, publish with license/source availability. |
| `target/` and other build outputs | Generated from GPL source | Ignored by git; distribution triggers GPL source obligations. |

## Optimizer Backends

TreeMaker 5.0.1 abstracts nonlinear constrained optimization behind `tmNLCO`.
The public source supports several possible adapters, but only ALM is enabled by
default in `Source/tmModel/tmNLCO/tmNLCO.h`.

| Backend | Port status | License / redistribution status | Practical effect |
| --- | --- | --- | --- |
| `ALM` | Ported | Distributable TreeMaker code | This is the parity baseline used by Rust and the C++ oracle. |
| `CFSQP` | Not ported | External/proprietary optimizer; not redistributed with TreeMaker 5.0.1 source | Would only affect numerical optimization performance/results, not file I/O or CP construction. |
| `RFSQP` | Not ported | External/evaluation FSQP-family optimizer; TreeMaker source comments note it is not redistributable | Would only affect numerical optimization performance/results. |
| `wnlib` | Not ported as an optimizer backend | Bundled `wnlib` code is unrestricted, but not enabled by default in TreeMaker 5.0.1 | Lang's bundled notes say it was faster than ALM but less reliable on some convergence tests. |

CFSQP/RFSQP are intentionally excluded unless redistributable source and
compatible license terms are provided. They are not required for TreeMaker
5.0.1 ALM parity.

## Publishing Checklist

Before publishing a repository, CLI binary, wasm package, or npm package:

1. Keep `LICENSE.txt`, `LICENSING.md`, and the TreeMaker notices in
   `third_party/treemaker-5.0.1`.
2. Publish the corresponding source for any binary or wasm artifact.
3. Do not include CFSQP/RFSQP source or binaries unless you have a separate
   redistribution license that is compatible with the GPL.
4. Make package metadata say `GPL-2.0-or-later`.
5. Keep generated package outputs from hiding the source dependency: link back
   to this repository or otherwise provide the exact source used to build them.
6. If you add new dependencies, check their licenses before release.

## Rust Dependency License Inventory

The current crates.io dependency graph is GPL-compatible. This list was
generated from `cargo metadata` against the checked-in `Cargo.lock`.

| Crate | Version | License |
| --- | --- | --- |
| `anstream` | `1.0.0` | `MIT OR Apache-2.0` |
| `anstyle` | `1.0.14` | `MIT OR Apache-2.0` |
| `anstyle-parse` | `1.0.0` | `MIT OR Apache-2.0` |
| `anstyle-query` | `1.1.5` | `MIT OR Apache-2.0` |
| `anstyle-wincon` | `3.0.11` | `MIT OR Apache-2.0` |
| `anyhow` | `1.0.102` | `MIT OR Apache-2.0` |
| `async-trait` | `0.1.89` | `MIT OR Apache-2.0` |
| `autocfg` | `1.5.0` | `Apache-2.0 OR MIT` |
| `bumpalo` | `3.20.2` | `MIT OR Apache-2.0` |
| `cast` | `0.3.0` | `MIT OR Apache-2.0` |
| `cc` | `1.2.62` | `MIT OR Apache-2.0` |
| `cfg-if` | `1.0.4` | `MIT OR Apache-2.0` |
| `clap` | `4.6.1` | `MIT OR Apache-2.0` |
| `clap_builder` | `4.6.0` | `MIT OR Apache-2.0` |
| `clap_derive` | `4.6.1` | `MIT OR Apache-2.0` |
| `clap_lex` | `1.1.0` | `MIT OR Apache-2.0` |
| `colorchoice` | `1.0.5` | `MIT OR Apache-2.0` |
| `find-msvc-tools` | `0.1.9` | `MIT OR Apache-2.0` |
| `futures-core` | `0.3.32` | `MIT OR Apache-2.0` |
| `futures-task` | `0.3.32` | `MIT OR Apache-2.0` |
| `futures-util` | `0.3.32` | `MIT OR Apache-2.0` |
| `heck` | `0.5.0` | `MIT OR Apache-2.0` |
| `is_terminal_polyfill` | `1.70.2` | `MIT OR Apache-2.0` |
| `itoa` | `1.0.18` | `MIT OR Apache-2.0` |
| `js-sys` | `0.3.98` | `MIT OR Apache-2.0` |
| `libm` | `0.2.16` | `MIT` |
| `memchr` | `2.8.0` | `Unlicense OR MIT` |
| `minicov` | `0.3.8` | `Apache-2.0/MIT` |
| `nu-ansi-term` | `0.50.3` | `MIT` |
| `num-traits` | `0.2.19` | `MIT OR Apache-2.0` |
| `once_cell` | `1.21.4` | `MIT OR Apache-2.0` |
| `once_cell_polyfill` | `1.70.2` | `MIT OR Apache-2.0` |
| `oorandom` | `11.1.5` | `MIT` |
| `pin-project-lite` | `0.2.17` | `Apache-2.0 OR MIT` |
| `proc-macro2` | `1.0.106` | `MIT OR Apache-2.0` |
| `quote` | `1.0.45` | `MIT OR Apache-2.0` |
| `rustversion` | `1.0.22` | `MIT OR Apache-2.0` |
| `same-file` | `1.0.6` | `Unlicense/MIT` |
| `serde` | `1.0.228` | `MIT OR Apache-2.0` |
| `serde-wasm-bindgen` | `0.6.5` | `MIT` |
| `serde_core` | `1.0.228` | `MIT OR Apache-2.0` |
| `serde_derive` | `1.0.228` | `MIT OR Apache-2.0` |
| `serde_json` | `1.0.149` | `MIT OR Apache-2.0` |
| `shlex` | `1.3.0` | `MIT OR Apache-2.0` |
| `slab` | `0.4.12` | `MIT` |
| `strsim` | `0.11.1` | `MIT` |
| `syn` | `2.0.117` | `MIT OR Apache-2.0` |
| `thiserror` | `2.0.18` | `MIT OR Apache-2.0` |
| `thiserror-impl` | `2.0.18` | `MIT OR Apache-2.0` |
| `unicode-ident` | `1.0.24` | `(MIT OR Apache-2.0) AND Unicode-3.0` |
| `utf8parse` | `0.2.2` | `Apache-2.0 OR MIT` |
| `walkdir` | `2.5.0` | `Unlicense/MIT` |
| `wasm-bindgen` | `0.2.121` | `MIT OR Apache-2.0` |
| `wasm-bindgen-futures` | `0.4.71` | `MIT OR Apache-2.0` |
| `wasm-bindgen-macro` | `0.2.121` | `MIT OR Apache-2.0` |
| `wasm-bindgen-macro-support` | `0.2.121` | `MIT OR Apache-2.0` |
| `wasm-bindgen-shared` | `0.2.121` | `MIT OR Apache-2.0` |
| `wasm-bindgen-test` | `0.3.71` | `MIT OR Apache-2.0` |
| `wasm-bindgen-test-macro` | `0.3.71` | `MIT OR Apache-2.0` |
| `wasm-bindgen-test-shared` | `0.2.121` | `MIT OR Apache-2.0` |
| `winapi-util` | `0.1.11` | `Unlicense OR MIT` |
| `windows-link` | `0.2.1` | `MIT OR Apache-2.0` |
| `windows-sys` | `0.61.2` | `MIT OR Apache-2.0` |
| `zmij` | `1.0.21` | `MIT` |

## Source References

- TreeMaker 5.0.1 bundled license: `third_party/treemaker-5.0.1/LICENSE.txt`
- TreeMaker optimizer notes: `third_party/treemaker-5.0.1/Source/tmModel/tmNLCO/README.txt`
- Enabled optimizer flags: `third_party/treemaker-5.0.1/Source/tmModel/tmNLCO/tmNLCO.h`
- FSF GPL v2 text: <https://www.gnu.org/licenses/old-licenses/gpl-2.0.html>
- FSF Directory TreeMaker entry: <https://directory.fsf.org/wiki/TreeMaker>
