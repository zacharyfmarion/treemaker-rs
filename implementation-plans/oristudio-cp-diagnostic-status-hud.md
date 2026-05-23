# Oriedita CP Diagnostic Status HUD

## Goal

Fix point-only diagnostic rendering crashes and add an in-canvas status readout
for Oriedita CP checks such as CAMV.

## Approach

- Treat `diagnostic_entries[].segments` as optional on the web side because the
  Rust serializer omits empty arrays.
- Render diagnostic segments with `segments ?? []` while keeping point markers.
- Add a compact top-right HUD in the CP viewport that summarizes the latest
  check result by operation, count, and severity.
- Add regression coverage for CAMV entries with points but no segments.

## Affected Areas

- `apps/web/src/engine/oristudioCpTypes.ts`
- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/styles/theme.css`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Make diagnostic segment lists optional in TypeScript.
- [x] Guard CP marker rendering against missing segment arrays.
- [x] Add top-right CP diagnostic status HUD.
- [x] Limit the status HUD to diagnostic/check commands instead of ordinary
      edit command success messages.
- [x] Render point-only theorem violations above normal vertex markers with a
      readable target marker.
- [x] Add focused web regression tests.
- [x] Run non-browser validation and commit.
