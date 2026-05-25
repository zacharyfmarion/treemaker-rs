# Oriedita Sidebar Help And Edit Tools

## Goal
Make every CP sidebar tool show useful Oriedita-style help in the bottom-right context panel, and clarify whether the visible Edit rail entries should remain tools or move into the Crease Pattern menu.

## Approach
- Map remaining rail commands to Oriedita action ids where the command currently only points at a handler/task name.
- Expand the CP tool instruction registry with vendored `help.properties` text for sidebar tools beyond the initial drawing pass.
- Add a safe fallback that derives help from the command tooltip and step prompts so every sidebar entry has panel content.
- Keep Edit rail entries that map to Oriedita mouse modes in the sidebar; do not move those four tools into the menu.
- Add unit coverage for rail help completeness and the Edit-tool classification.

## Affected Areas
- `apps/web/src/lib/oristudioCpActions.ts`
- `apps/web/src/lib/oristudioCpToolInstructions.ts`
- CP action and instruction tests
- CP panel tests for sidebar help display

## Checklist
- [x] Map missing rail commands to Oriedita action ids.
- [x] Add help text for all current sidebar tools.
- [x] Preserve mouse-mode Edit tools in the rail and test that behavior.
- [x] Run focused tests and web validation.
