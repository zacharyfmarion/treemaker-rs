# Oriedita Extend Line Parity

## Goal

Make the editable CP Extend Line tool behave like Oriedita: click the line to
extend, then click the target line it should extend to.

## Approach

- Add a line-id command path for `LengthenCrease` and
  `LengthenCreaseSameColor` in the CP kernel.
- Update the CP pane line-click handler so Extend Line stores the first clicked
  line and executes on the second clicked line.
- Keep the existing point-based lengthen payload path for drag/advanced
  selection parity.
- Update tests for both kernel dispatch and UI payload behavior.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- CP frontend and kernel tests

## Checklist

- [x] Add kernel line-id dispatch.
- [x] Add UI line-click flow.
- [x] Update tests.
- [x] Run focused and web validation.
