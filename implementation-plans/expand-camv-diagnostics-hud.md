# Expand CAMV Diagnostics HUD

## Goal

Make the in-canvas CP diagnostic HUD expandable so users can inspect and focus
individual CAMV/Oriedita diagnostic errors without leaving the crease-pattern
pane, while showing exact issue locations consistently in the Diagnostics pane.

## Approach

- Share CP diagnostic labels and location formatting between the Diagnostics
  panel and the crease-pattern HUD.
- Convert the HUD summary into a collapsible button that preserves the compact
  default readout.
- Render the same diagnostic rows in the expanded HUD, with row selection wired
  to the existing diagnostic focus behavior.
- Keep the UI in shared React code and reuse the existing CAMV diagnostic state
  instead of adding new runtime or Tauri behavior.
- Add focused frontend tests for expanded HUD rows, exact point text, and row
  selection.

## Affected Areas

- `apps/web/src/components/panels/CreasePatternPanel.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.tsx`
- `apps/web/src/lib/oristudioCpDiagnostics.ts`
- `apps/web/src/styles/theme.css`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `apps/web/src/components/panels/DiagnosticsPanel.test.tsx`

## Checklist

- [x] Inspect existing always-on CAMV and diagnostic HUD behavior.
- [x] Add shared diagnostic label and point-location formatting.
- [x] Make the CP HUD expandable/collapsible.
- [x] Render individual HUD diagnostic rows with existing focus behavior.
- [x] Update Diagnostics panel rows to include exact locations.
- [x] Add focused frontend regression tests.
- [x] Run focused web tests, typecheck, lint, and diff hygiene checks.
- [x] Commit, push, and open a draft PR.
- [x] Start the local web app for browser testing after PR creation.
