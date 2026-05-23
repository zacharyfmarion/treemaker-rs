# OriStudio CP Stage 8 Organize Circles

## Goal

Enable Oriedita's circle organization command, which prunes zero-radius circles using the oracle-tested circle cleanup worker.

## Approach

- Dispatch `OrganizeCircles` through `operations::circle::organize`.
- Mark the web command ready as an immediate annotation cleanup command.
- Add focused Rust and web tests to cover command dispatch and UI invocation.

## Affected Areas

- `crates/oristudio-cp/src/lib.rs`
- `apps/web/src/lib/oristudioCpCommands.ts`
- `apps/web/src/lib/oristudioCpCommands.test.ts`
- `apps/web/src/components/panels/CreasePatternPanel.test.tsx`
- `implementation-plans/oristudio-cp-ui-roadmap.md`

## Checklist

- [x] Add Rust command dispatch.
- [x] Mark the command ready in the web registry.
- [x] Add Rust and web command tests.
- [x] Run focused validation.
- [x] Commit the completed Stage 8 circle organization slice.
