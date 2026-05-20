# Release Checklist

Ori Studio has three release surfaces:

- Cloudflare Pages web deployment at `https://oristudio.pages.dev/`.
- Signed and notarized Apple Silicon DMGs on GitHub Releases.
- Rust crates for the reusable TreeMaker engine, CLI, FOLD helpers, and WASM
  bridge.

## Web App

The `Deploy Web App` workflow deploys `apps/web/dist` to Cloudflare Pages on
pushes to `main` and on manual dispatch.

Required GitHub Actions secrets:

- `CLOUDFLARE_ACCOUNT_ID`
- `CLOUDFLARE_API_TOKEN`

The Cloudflare Pages project name is `oristudio`; the production URL is
`https://oristudio.pages.dev/`. Pull request previews are deployed from
non-fork PRs to `https://pr-<number>.oristudio.pages.dev/`.

## Desktop App

Desktop release builds are local-only. GitHub Actions validates release tags but
does not build DMGs or store Apple signing credentials.

Prepare the release PR:

```sh
./scripts/release.sh prepare 0.2.0
```

After the release PR is merged to `main`, publish from a clean local Mac:

```sh
./scripts/release.sh publish 0.2.0
```

The publish step builds, signs, notarizes, staples, verifies, tags, and uploads:

- `OriStudio_0.2.0_aarch64.dmg`
- `OriStudio_latest_aarch64.dmg`
- `sha256-aarch64.txt`

Local release secrets should live in `.env.release.local`, copied from
`.env.release.example`. That file is ignored and must not be committed.

Required local values:

- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

Optional values, only needed when the Developer ID certificate is not already
installed in the local login keychain:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`

## Crates

This repository publishes multiple crates from one workspace. Publish
`treemaker-fold`, `treemaker-flatfold`, and `treemaker-core` before dependent
crates that reference their exact version.

Preflight:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
tools/oracle/build_oracle.sh
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test cpp_oracle
TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test generated_families
TREEMAKER_CORPUS_DIR=tests/fixtures TREEMAKER_CPP_ORACLE=tools/oracle/build/treemaker-oracle cargo test -p oracle-tests --test corpus -- --nocapture
cargo publish -p treemaker-fold --dry-run
cargo publish -p treemaker-flatfold --dry-run
cargo publish -p treemaker-core --dry-run
```

For optional WASM release confidence:

```sh
wasm-pack build crates/treemaker-wasm --target bundler
wasm-pack test --node crates/treemaker-wasm
```

Publish order:

```sh
cargo publish -p treemaker-fold
cargo publish -p treemaker-flatfold
cargo publish -p treemaker-core
```

Wait for crates.io indexing, then:

```sh
cargo publish -p treemaker-cli --dry-run
cargo publish -p treemaker-cli
cargo publish -p treemaker-wasm --dry-run
cargo publish -p treemaker-wasm
```

`oracle-tests` is an internal parity-test crate and has `publish = false`.

## Manual Setup

1. In Cloudflare, create or confirm a Pages project named `oristudio` with
   production branch `main`.
2. Locally authenticate Wrangler if needed:
   ```sh
   npx wrangler@4 login
   ```
3. If the project does not exist yet, create it:
   ```sh
   npx wrangler@4 pages project create oristudio --production-branch main
   ```
4. In Cloudflare, create an API token that can edit Cloudflare Pages for the
   account.
5. Copy the Cloudflare account ID from the dashboard.
6. In GitHub repo settings, add Actions secrets `CLOUDFLARE_ACCOUNT_ID` and
   `CLOUDFLARE_API_TOKEN`.
7. Trigger `Deploy Web App` manually and verify
   `https://oristudio.pages.dev/`.
8. Open a test PR that changes web code and verify the preview comment points to
   `https://pr-<number>.oristudio.pages.dev/`.
9. On your Mac, install and authenticate release tooling:
   ```sh
   brew install gh jq
   gh auth login
   ```
10. Ensure Rust, Node, npm, Xcode, and Xcode command-line tools are available.
11. In Apple Developer, ensure you have a Developer ID Application certificate,
    an app-specific password for notarization, and your Team ID.
12. Confirm the certificate is installed locally:
    ```sh
    security find-identity -v -p codesigning
    ```
13. Copy `.env.release.example` to `.env.release.local` and fill the required
    Apple values.
14. For each release, run `./scripts/release.sh prepare X.Y.Z`, merge the
    release PR, then run `./scripts/release.sh publish X.Y.Z` from a clean local
    Mac checkout.
15. After publish, verify the GitHub Release contains the versioned and latest
    Apple Silicon DMGs, download the DMG, mount it, and launch `Ori Studio.app`.
