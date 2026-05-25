# Oriedita CP Tool Instructions

## Goal

Show Oriedita-style tool instructions in the editable crease-pattern context
panel so migrated users can see how each selected drawing tool is used.

## Approach

- Add a typed, sanitized instruction registry keyed by Oriedita action id.
- Resolve instructions from the active CP action so alias actions keep their
  original Oriedita help text.
- Render instructions above existing tool settings in the bottom-right context
  panel.
- Keep the viewport status readout as the short current-step prompt.

## Affected Areas

- `apps/web/src/lib/oristudioCpToolInstructions.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/styles/theme.css`
- CP frontend tests

## Checklist

- [x] Add Oriedita instruction registry and resolver tests.
- [x] Render instructions in the CP context panel.
- [x] Update panel styling.
- [x] Update interaction/rendering tests.
- [x] Run web validation.
