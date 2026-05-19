# Help And About Modals

## Goal

Wire the shared Help menu to in-app documentation and About modals that explain
the current TreeMaker app surface, include generated screenshots, and
acknowledge the upstream projects and references behind the app.

## Approach

- Add shared help/about modal state and route Help menu command IDs through the
  existing command dispatcher.
- Update the web menu, toolbar affordances, and Tauri native menu so browser
  and desktop users reach the same modal content.
- Build a compact documentation modal that covers file workflows, design
  editing, inspector usage, conditions, optimization, crease pattern review,
  simulation/folded-base views, diagnostics, layout, and settings.
- Generate static help screenshots with Playwright from the local web app and
  store them as web public assets.
- Add focused frontend tests for command routing, menu coverage, and modal
  rendering.

## Affected Areas

- `apps/web/src/components`
- `apps/web/src/commands`
- `apps/web/src/menus`
- `apps/web/src/store`
- `apps/web/src/styles`
- `apps/web/public/help`
- `apps/tauri/src-tauri/src/menu.rs`

## Checklist

- [x] Add implementation plan and branch.
- [x] Add shared help/about modal command wiring.
- [x] Build modal content and styling.
- [x] Generate and commit Playwright screenshots.
- [x] Add focused frontend tests.
- [x] Run targeted validation and prepare PR handoff.
