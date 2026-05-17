# External Corpus Testing

Real-user TreeMaker documents are intentionally not committed here unless their
authors explicitly permit redistribution. Keep large/private corpora outside the
repository and point the harness at those paths.

Useful commands:

```sh
cargo run -p treemaker-cli -- corpus /path/to/private/corpus --format json
cargo run -p treemaker-cli -- corpus /path/to/private/corpus --oracle tools/oracle/build/treemaker-oracle
TREEMAKER_CORPUS_DIR=/path/to/private/corpus TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test corpus -- --nocapture
```

The corpus command recursively scans `.tmd`, `.tmd4`, and `.tmd5` files,
deduplicates them by SHA-256, parses with the Rust engine, round-trips through
canonical v5 output, and optionally compares C++ oracle summaries.
