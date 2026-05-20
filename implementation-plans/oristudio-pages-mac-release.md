# Ori Studio Pages And Mac Release

## Goal

Set up Ori Studio for Cloudflare Pages deployment at `oristudio.pages.dev` and
for local Apple Silicon macOS releases that are signed, notarized, stapled, and
published to GitHub Releases.

## Approach

- Add production and pull request preview Cloudflare Pages workflows that build
  the existing Vite app and deploy `apps/web/dist` to the `oristudio` Pages
  project.
- Add a tag validation workflow for release tags while keeping Apple signing and
  notarization local-only.
- Add local release scripts based on the Cascade flow: prepare a release PR,
  publish from the merged PR, build a signed DMG locally, push the tag, and
  upload release artifacts.
- Enable the Tauri bundle metadata needed for macOS packaging.
- Document the required GitHub Actions secrets and local Apple release env.

## Affected Areas

- GitHub Actions workflows under `.github/workflows/`.
- Tauri bundle metadata in `apps/tauri/src-tauri/`.
- Local release scripts and release documentation.
- README links for the hosted app and Mac release artifacts.

## Checklist

- [x] Confirm current Ori Studio branding and repo state.
- [x] Add Cloudflare Pages production and preview workflows.
- [x] Add release tag validation workflow.
- [x] Add local macOS release scripts and env example.
- [x] Enable Tauri bundle metadata for macOS release builds.
- [x] Update release and user-facing docs with manual setup steps.
- [x] Run focused validation.
