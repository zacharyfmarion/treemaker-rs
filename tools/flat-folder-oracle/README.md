# Flat-Folder Oracle

`oracle.mjs` wraps the vendored Flat-Folder JavaScript implementation and emits
stable JSON for Rust parity tests.

Commands:

```sh
node tools/flat-folder-oracle/oracle.mjs normalize tests/fixtures/flat-folder/kabuto.fold
node tools/flat-folder-oracle/oracle.mjs project tests/fixtures/flat-folder/kabuto.fold
node tools/flat-folder-oracle/oracle.mjs overlap tests/fixtures/flat-folder/kabuto.fold
node tools/flat-folder-oracle/oracle.mjs constraints tests/fixtures/flat-folder/kabuto.fold
node tools/flat-folder-oracle/oracle.mjs solve tests/fixtures/flat-folder/kabuto.fold --limit 10
node tools/flat-folder-oracle/oracle.mjs run-corpus tests/fixtures/flat-folder --limit 1
```

`--limit` accepts a positive integer or `all`. Set
`FLATFOLDER_ORACLE_INCLUDE_DATA=1` for debugging first-solution face-order data
in `solve` output.
