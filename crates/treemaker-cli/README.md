# treemaker-cli

Headless command-line interface for the TreeMaker 5.0.1 Rust port.

```sh
cargo install treemaker-cli
treemaker inspect model.tmd5 --format json
treemaker check model.tmd5 --details
treemaker optimize model.tmd5 --kind scale --out optimized.tmd5
treemaker build-cp optimized.tmd5 --out cp.tmd5
treemaker export-v4 cp.tmd5 --out legacy.tmd4
```

The `corpus` subcommand recursively scans `.tmd`, `.tmd4`, and `.tmd5` files,
deduplicates by SHA-256, round-trips through canonical v5, and can optionally
compare summaries against the C++ oracle from the repository.

License: `GPL-2.0-or-later`.
