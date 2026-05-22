# Reference And Precrease V2 Boundary

## Goal

Define where reference/precrease planning belongs without starting a partial
ReferenceFinder port.

## Approach

The V1 folding-sequence planner works from crease patterns that already exist.
It may emit collapse steps and unsupported-region diagnostics, but it must not
invent construction sequences for locating reference points or crease lines.

V2 should port a ReferenceFinder-style construction search separately. Because
ReferenceFinder is GPL-2.0, any direct port should live in a GPL-compatible
crate or module and must not be mixed into permissively licensed FOLD or
flat-folding crates.

Until that port exists, `treemaker-sequence` exposes typed reference/precrease
plan types and returns `SequenceError::NotImplemented` for reference planning.

## Affected Areas

- `crates/treemaker-sequence` owns the public reference/precrease planning
  boundary types.
- A future GPL-compatible crate or module should own the actual construction
  search.
- WASM and UI surfaces should show a `not_implemented` diagnostic instead of a
  placeholder construction sequence.

## Checklist

- [x] Add typed reference/precrease plan request and artifact shapes.
- [x] Validate basic FOLD input before returning the V2 not-implemented boundary.
- [x] Keep missing construction search explicit as `not_implemented`.
- [ ] Port ReferenceFinder-style construction search in V2.
- [ ] Add visual construction-step review once the V2 port exists.
