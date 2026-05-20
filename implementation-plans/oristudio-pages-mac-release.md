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

## Cloudflare Pages Setup Guide

The Cloudflare project name is `oristudio`, which gives the production Pages
URL `https://oristudio.pages.dev/`. The GitHub workflows use Wrangler direct
uploads instead of Cloudflare's connected-repository build pipeline, so GitHub
Actions owns the build and Cloudflare Pages only receives the built
`apps/web/dist` assets.

Use a Direct Upload Pages project for this flow. Cloudflare documents Direct
Upload projects as separate from Git-integrated Pages projects, and a Direct
Upload project cannot later be switched into Git integration. If `oristudio`
was accidentally created as a connected Git project, create a new Direct Upload
Pages project before wiring the GitHub Actions secrets.

### One-Time Cloudflare Setup

1. Authenticate Wrangler locally:

   ```sh
   npx wrangler@4 login
   ```

2. Create the Pages project if it does not already exist:

   ```sh
   npx wrangler@4 pages project create oristudio --production-branch main
   ```

3. Confirm the project exists:

   ```sh
   npx wrangler@4 pages project list
   ```

4. Confirm the production URL in the Cloudflare dashboard:

   ```text
   https://oristudio.pages.dev/
   ```

5. Create a Cloudflare API token:
   - Go to Cloudflare dashboard -> My Profile -> API Tokens.
   - Create a custom token.
   - Add permission: Account -> Cloudflare Pages -> Edit.
   - Scope the token to the account that owns `oristudio`.
   - Copy the token once and store it in the password manager.

6. Find the Cloudflare account ID from the account dashboard/sidebar.

7. Add these GitHub Actions repository secrets:

   ```text
   CLOUDFLARE_ACCOUNT_ID
   CLOUDFLARE_API_TOKEN
   ```

   Exact GitHub path:

   ```text
   Repository -> Settings -> Secrets and variables -> Actions ->
   Repository secrets -> New repository secret
   ```

8. After the workflow lands on `main`, run the GitHub Actions workflow
   `Deploy Web App` manually once:

   ```text
   Repository -> Actions -> Deploy Web App -> Run workflow -> main
   ```

9. Verify the production app:

   ```text
   https://oristudio.pages.dev/
   ```

10. Open a test PR that changes a web-relevant file and verify the sticky preview
   comment points to:

   ```text
   https://pr-<number>.oristudio.pages.dev/
   ```

### Cloudflare Validation Expectations

- `Deploy Web App` should run on pushes to `main` and manual dispatch.
- `Deploy PR Preview` should run for non-fork PRs against `main`.
- Docs-only and desktop-only PRs should receive a skipped preview comment.
- Fork PRs should skip preview deployment because GitHub does not expose the
  required Cloudflare secrets to forked workflows.

### Cloudflare Source References

- Cloudflare Pages direct upload CI:
  https://developers.cloudflare.com/pages/how-to/use-direct-upload-with-continuous-integration/
- Wrangler Pages commands:
  https://developers.cloudflare.com/workers/wrangler/commands/pages/

## Apple Signing And Notarization Guide

The first macOS release target is Apple Silicon only:

```text
aarch64-apple-darwin
```

The public app name is `Ori Studio`, the Rust/Tauri package name is
`ori-studio`, and the uploaded DMG artifacts use shell-safe names:

```text
OriStudio_X.Y.Z_aarch64.dmg
OriStudio_latest_aarch64.dmg
sha256-aarch64.txt
```

Apple signing credentials stay local. Do not add Apple signing or notarization
secrets to GitHub Actions for this workflow.

### Apple Account Prerequisites

1. Use a paid Apple Developer Program account. A free Apple developer account is
   not sufficient for public Developer ID notarization.

2. Make sure the person creating the Developer ID Application certificate has
   the required Apple Developer role. Apple documents Developer ID certificate
   creation as requiring the Account Holder role unless cloud-managed
   certificate access is configured.

3. Install Xcode and command-line tools on the release Mac:

   ```sh
   xcode-select -p
   xcodebuild -version
   xcrun notarytool --version
   ```

4. If command-line tools are missing:

   ```sh
   xcode-select --install
   ```

5. If `xcrun notarytool --version` fails because the active developer
   directory is not a full Xcode install, point `xcode-select` at Xcode:

   ```sh
   sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
   ```

6. If Xcode prompts for the license, open Xcode once and accept it, or run:

   ```sh
   sudo xcodebuild -license accept
   ```

7. Install local helper tools:

   ```sh
   brew install gh jq
   gh auth login
   ```

8. Confirm repo/runtime tools:

   ```sh
   node --version
   npm --version
   rustc --version
   cargo --version
   ```

### Find The Apple Team ID

1. Go to:

   ```text
   https://developer.apple.com/account
   ```

2. Open Membership details.

3. Copy the 10-character Team ID.

4. This value becomes:

   ```sh
   APPLE_TEAM_ID="TEAMID"
   ```

### Create The Certificate Signing Request

Create the CSR on the Mac that will hold the private key.

1. Open Keychain Access:

   ```text
   /Applications/Utilities/Keychain Access.app
   ```

2. In the menu bar choose:

   ```text
   Keychain Access -> Certificate Assistant -> Request a Certificate from a Certificate Authority...
   ```

3. Fill the Certificate Assistant form:

   ```text
   User Email Address: <Apple Developer account email>
   Common Name: Ori Studio Developer ID Application
   CA Email Address: leave blank
   Request is: Saved to disk
   ```

4. Save the CSR somewhere obvious, for example:

   ```text
   ~/Desktop/OriStudioDeveloperID.certSigningRequest
   ```

### Create The Developer ID Application Certificate

1. Go to:

   ```text
   https://developer.apple.com/account/resources/certificates/list
   ```

2. Choose:

   ```text
   Certificates -> + -> Software -> Developer ID -> Continue
   ```

3. Select:

   ```text
   Developer ID Application
   ```

4. Do not choose `Developer ID Installer`; that is for signed `.pkg` installers,
   while this release flow distributes a signed `.dmg`.

5. Upload the CSR from the previous section.

6. Download the generated `.cer` file.

7. Double-click the `.cer` file to install it into the login keychain.

### Verify The Certificate And Private Key

Run:

```sh
security find-identity -v -p codesigning
```

Expected output includes a quoted identity like:

```text
"Developer ID Application: Your Name (TEAMID)"
```

Copy the exact quoted identity without the quotes. The script also accepts the
identity hash shown by `security find-identity`, but the full Developer ID
Application name is easier to audit. It becomes:

```sh
APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
```

If the identity does not appear:

1. Open Keychain Access.
2. Select the `login` keychain.
3. Select `My Certificates`.
4. Find the Developer ID Application certificate.
5. Expand the certificate row.
6. Confirm a private key is nested underneath it.

If there is no private key, the certificate was not created from a CSR generated
on this Mac, or the key lives on another Mac. Either create a new CSR and
certificate from this Mac, or export/import a password-protected `.p12` from the
Mac that owns the private key.

Sanity check the app entitlements before the first release:

```sh
grep -n "get-task-allow" apps/tauri/src-tauri/entitlements.plist
```

This command should print nothing. Apple rejects notarization when a production
Developer ID submission includes `com.apple.security.get-task-allow=true`.

### Optional Certificate Backup As `.p12`

This local release workflow does not require `APPLE_CERTIFICATE` if the
Developer ID Application identity is already installed in the login keychain.
Still, exporting a password-protected backup is useful for recovery.

1. Open Keychain Access -> login -> My Certificates.
2. Expand the Developer ID Application certificate.
3. Right-click the private key under the certificate.
4. Choose Export.
5. Save a `.p12` file and protect it with a strong password.
6. Store the `.p12` and password in the password manager.

If a future release Mac does not have the cert installed, encode the `.p12`:

```sh
openssl base64 -A -in /path/to/certificate.p12 -out certificate-base64.txt
```

Then add these optional values to `.env.release.local`:

```sh
APPLE_CERTIFICATE="<contents of certificate-base64.txt>"
APPLE_CERTIFICATE_PASSWORD="<p12 export password>"
```

The script imports the certificate into a temporary keychain and removes that
temporary keychain during cleanup.

### Create The App-Specific Password

1. Go to:

   ```text
   https://account.apple.com
   ```

2. Open:

   ```text
   Sign-In and Security -> App-Specific Passwords
   ```

3. Generate a password named:

   ```text
   Ori Studio Notarization
   ```

4. Copy the generated password immediately. It usually has this shape:

   ```text
   xxxx-xxxx-xxxx-xxxx
   ```

5. This value becomes:

   ```sh
   APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"
   ```

Apple requires two-factor authentication for app-specific passwords. If the
primary Apple Account password is changed or reset, Apple revokes app-specific
passwords, so generate a fresh one before the next release.

### Create The Local Release Env File

From the repo root:

```sh
cp .env.release.example .env.release.local
```

Fill:

```sh
RELEASE_GITHUB_REPO=zacharyfmarion/ori-studio
RELEASE_REMOTE=origin

APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
APPLE_ID="you@example.com"
APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"
APPLE_TEAM_ID="TEAMID"
```

Do not commit `.env.release.local`.

### Validate The Local Signing Setup

Confirm the identity exists:

```sh
security find-identity -v -p codesigning
```

Load the env file in the current shell:

```sh
set -a
source .env.release.local
set +a
```

Confirm non-secret values:

```sh
echo "$APPLE_ID"
echo "$APPLE_TEAM_ID"
echo "$APPLE_SIGNING_IDENTITY"
```

Do not echo `APPLE_PASSWORD`.

Confirm Apple notarization credentials:

```sh
xcrun notarytool history \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID"
```

If credentials are valid, this should return notarization history, possibly
empty. Fix authentication here before trying a real release.

Confirm the release script can see the signing identity:

```sh
security find-identity -v -p codesigning | grep -F "$APPLE_SIGNING_IDENTITY"
```

### Release Flow

Start from a clean checkout:

```sh
git switch main
git pull origin main
git status
```

Prepare a release PR:

```sh
./scripts/release.sh prepare X.Y.Z
```

Merge that release PR.

Publish from the local Mac:

```sh
git switch main
git pull origin main
git status
./scripts/release.sh publish X.Y.Z
```

Expected local artifacts:

```text
target/release-artifacts/vX.Y.Z/OriStudio_X.Y.Z_aarch64.dmg
target/release-artifacts/vX.Y.Z/OriStudio_latest_aarch64.dmg
target/release-artifacts/vX.Y.Z/sha256-aarch64.txt
```

Expected GitHub Release assets:

```text
OriStudio_X.Y.Z_aarch64.dmg
OriStudio_latest_aarch64.dmg
```

### Post-Release Verification

Download the GitHub Release `OriStudio_latest_aarch64.dmg`, then:

```sh
open OriStudio_latest_aarch64.dmg
```

Launch `Ori Studio.app` from the mounted volume.

Optional command-line checks:

```sh
spctl -a -vv -t open --context context:primary-signature /path/to/OriStudio_latest_aarch64.dmg
codesign --verify --deep --strict --verbose=2 "/Volumes/Ori Studio/Ori Studio.app"
```

### Common Apple Signing Failure Modes

- `security find-identity` does not show the Developer ID Application identity:
  the certificate is not installed in the login keychain, lacks a private key,
  or was created from a CSR generated on another Mac.
- Notarization rejects with an authentication error: regenerate the
  app-specific password, confirm `APPLE_TEAM_ID`, and rerun `notarytool history`.
- Notarization rejects because of signing problems: inspect the notary log saved
  under `target/release-artifacts/vX.Y.Z/notarytool-log-aarch64.json`.
- macOS still blocks the app: confirm the DMG was notarized, stapled, and tested
  after downloading through a browser or equivalent Gatekeeper path.

### Apple/Tauri Source References

- Apple Developer ID certificates:
  https://developer.apple.com/help/account/certificates/create-developer-id-certificates
- Apple certificate signing request:
  https://developer.apple.com/help/account/certificates/create-a-certificate-signing-request
- Apple Developer ID/Gatekeeper overview:
  https://developer.apple.com/developer-id/
- Apple app-specific passwords:
  https://support.apple.com/en-us/102654
- Apple Team ID:
  https://developer.apple.com/help/glossary/team-id/
- Tauri macOS signing and notarization:
  https://v2.tauri.app/distribute/sign/macos/

## Checklist

- [x] Confirm current Ori Studio branding and repo state.
- [x] Add Cloudflare Pages production and preview workflows.
- [x] Add release tag validation workflow.
- [x] Add local macOS release scripts and env example.
- [x] Enable Tauri bundle metadata for macOS release builds.
- [x] Update release and user-facing docs with manual setup steps.
- [x] Run focused validation.
