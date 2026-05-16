# TreeMaker C++ Oracle

This directory contains the phase-one oracle harness for the Rust port. It
builds a small command-line executable directly from the vendored TreeMaker
5.0.1 model source and emits stable JSON lines for fixture summaries and ALM
optimizer results.

The oracle intentionally excludes wxWidgets, GUI sources, WNLIB, CFSQP, and
RFSQP. TreeMaker 5.0.1 enables ALM by default in `tmNLCO.h`, and that is the
only optimizer backend used for v1 parity.

Build:

```sh
tools/oracle/build_oracle.sh
```

Run against the checked-in fixtures:

```sh
tools/oracle/build/treemaker-oracle run-fixtures --fixture-dir tests/fixtures
```

Run the gated Rust parity test:

```sh
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test cpp_oracle
```

Known local-compiler drift: on Apple Clang 17, the C++ ALM optimizer converges
to different feasible scale optima for `tmModelTester_2.tmd5` and
`tmModelTester_3.tmd5` than the shipped `tmModelTester.out.txt`. The gated Rust
test therefore enforces parse parity for all fixtures and optimizer parity for
the stable scale/edge/strain cases, while keeping those two C++ records visible
as oracle output.

The binary is a local build artifact and is ignored by git.
