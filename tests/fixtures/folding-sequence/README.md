# Folding Sequence Fixtures

These fixtures are the Phase 0 visual and correctness baseline for the folding
sequence research track.

Each fixture has three pieces:

- `fold/<id>.fold`: a small FOLD crease pattern.
- `expected/<id>.sequence.json`: the planner artifact shape expected before any
  planner implementation exists.
- `manifest.json`: review metadata, expected future outcome, and visual checks.

## Not Implemented Policy

Until a planner phase is genuinely implemented, expected artifacts must use
`"status": "not_implemented"` with a diagnostic code of `"not_implemented"`.
Do not add a temporary fake plan, approximate rewrite, or guessed instruction
sequence just to make a fixture look complete. A precise `not_implemented`
artifact is better than a misleading partial implementation.

Future phases should update each fixture artifact only when the corresponding
planner behavior is validated by tests and visual review.

## Visual Review

To inspect the fixture crease patterns, serve this directory and open the visual
review page:

```bash
cd tests/fixtures/folding-sequence
python3 -m http.server 8787
```

Then open:

```text
http://localhost:8787/visual-review.html
```

The reviewer draws each FOLD fixture with mountain, valley, boundary, flat, and
unassigned line styles. It is intentionally simple so visual differences are easy
to spot during research.

## Fixture Intent

- `simple-valley`: one horizontal valley fold across a square. Expected to be a
  complete V1 simple-fold plan.
- `accordion-book-fold`: two parallel folds forming three strips. Expected to be
  a complete or near-complete V1 simple-fold/multi-layer plan.
- `kite-rabbit-ear-local`: a four-crease local kite pattern around one interior
  vertex. Expected to need early complex-move recognition.
- `squash-local`: an eight-sector local pattern that should become a squash-like
  complex-move fixture.
- `treemaker-triad-base`: a small TreeMaker-style three-flap base proxy for
  visual and target-state testing. It is hand-authored for Phase 0 and should be
  replaced or supplemented with an exported TreeMaker FOLD fixture once sequence
  APIs can consume generated TreeMaker artifacts directly.
- `simultaneous-collapse-unsupported`: a valid flat-fold target that stands in
  for a simultaneous collapse V1 should label as unsupported until the matching
  rule exists.
