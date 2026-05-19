# Ori Studio Branding Rename

## Goal

Rename the user-facing web and Tauri app from TreeMaker to Ori Studio while preserving internal `treemaker-*` package, crate, storage, and compatibility identifiers.

## Approach

- Branch from latest `origin/main` and keep the change scoped to app branding.
- Update web document titles, toolbar copy, modal/menu labels, user-facing status and error text, and native file dialog labels.
- Update Tauri product name, initial window title, bundle identifier, menu labels, capability description, and dev executable name.
- Keep TreeMaker wording where it names legacy/upstream compatibility, including TreeMaker 4/5 file formats and Robert J. Lang's TreeMaker 5.0.1.

## Affected Areas

- Shared React app strings, help/about copy, window title formatting, capability reasons, and related tests.
- Tauri app metadata, native menu labels, and dev binary naming.
- Validation for web lint/typecheck/tests/build, desktop check, and whitespace diff checks.

## Checklist

- [x] Create branch from latest `origin/main`.
- [x] Rename web app-facing branding strings.
- [x] Rename Tauri/mac-facing branding and identifier.
- [x] Update affected tests.
- [x] Audit remaining TreeMaker/treemaker occurrences in app files.
- [x] Run web, desktop, and diff validation.
- [x] Open draft PR against `main`.
