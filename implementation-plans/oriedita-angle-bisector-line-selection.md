# Oriedita Angle Bisector Line Selection

## Goal

Match Oriedita's angle-bisector line-selection workflow in the editable crease-pattern canvas.

## Approach

- Preserve the existing three-point angle-bisector flow.
- Add a line-selection path for Angle Bisector that highlights the first two selected lines.
- Commit the two-line construction from the intersection of the selected lines to the selected destination line.
- Update tool help text to use Oriedita's more informative step labels.
- Add focused web regression tests for the line-selection payload and help text.

## Affected Areas

- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpToolInstructions.ts`
- CP frontend tests

## Checklist

- [x] Read existing angle-bisector interaction and instruction code.
- [x] Implement the Oriedita two-line selection path.
- [x] Update Angle Bisector help and prompts.
- [x] Add focused regression tests.
- [x] Run targeted web validation.
