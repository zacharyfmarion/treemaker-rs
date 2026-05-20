# Mac Download Header

## Goal

Add a browser-only header button that mirrors Cascade's macOS download flow and
points users at the latest signed Apple Silicon Ori Studio DMG.

## Approach

- Add release constants for the GitHub repository, latest release base URL, and
  `OriStudio_latest_aarch64.dmg` artifact name.
- Add a tiny `useMacDownloadUrl` hook matching the Cascade pattern.
- Add a `macDownloadCta` platform feature flag that is visible on web and hidden
  in the Tauri desktop app.
- Add a download icon button to the existing toolbar near Help/Settings.
- Cover the feature flag behavior with the existing platform feature tests.

## Affected Areas

- `apps/web/src/App.tsx`
- `apps/web/src/constants/release.ts`
- `apps/web/src/hooks/useMacDownloadUrl.ts`
- `apps/web/src/platform/features.ts`
- `apps/web/src/platform/features.test.ts`

## Checklist

- [x] Add release URL constants and hook.
- [x] Add the web-only platform feature flag.
- [x] Add the toolbar download button.
- [x] Run focused web validation.
- [x] Open a draft PR against `main`.
