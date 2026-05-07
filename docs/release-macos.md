# macOS Release Packaging

SkillNotebook currently ships macOS packages through GitHub Actions. The default
mode is ad-hoc signing because the project does not yet have a paid Apple
Developer ID certificate.

## What the Workflow Does

The workflow is defined in `.github/workflows/release-macos.yml`.

- Every push to `main` builds and uploads macOS artifacts to the workflow run.
- Every pushed `v*` tag builds, verifies, creates or updates a GitHub Release,
  and uploads the generated DMG files plus SHA-256 checksum files.
- Manual runs can build artifacts only, or publish to a provided release tag.
- Both Apple Silicon and Intel targets are built:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`

The CI validation path runs:

```bash
python3 -m venv "$RUNNER_TEMP/skillnotebook-python"
"$RUNNER_TEMP/skillnotebook-python/bin/python" -m pip install PyYAML
echo "$RUNNER_TEMP/skillnotebook-python/bin" >> "$GITHUB_PATH"
npm run lint
npm run build
npm run test:e2e
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri:build
codesign --verify --deep --strict
hdiutil verify
```

Before invoking `tauri build`, the packaging job unsets empty Apple signing and
notarization environment variables. This matters because an unset Developer ID
secret should mean "use ad-hoc signing", while an empty `APPLE_CERTIFICATE`
variable makes Tauri try to import an invalid certificate.

## Current Distribution Status

The current package is ad-hoc signed. It is useful for internal testing and
repeatable release packaging, but it does not remove macOS Gatekeeper's
"Apple cannot verify" warning for downloaded apps.

For internal testing, users can open the app with right-click > Open, or approve
it from System Settings > Privacy & Security after macOS blocks the first launch.
Do not describe the ad-hoc package as fully trusted or notarized.

## Semantic Versioning

SkillNotebook uses Semantic Versioning 2.0.0. The app version must be identical
in all release metadata files:

- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

Release tags must be `v`-prefixed SemVer tags and must match the app version
exactly. For example, app version `0.4.2` must be released with tag `v0.4.2`.

During the `0.x` phase:

- Use `PATCH` for compatible fixes, CI/CD changes, copy polish, and packaging
  improvements.
- Use `MINOR` for new product capabilities or meaningful workflow changes.
- Reserve `MAJOR` for the future `1.0.0` compatibility baseline.

Useful commands:

```bash
npm run version:check
npm run version:set -- 0.4.2
npm run version:check -- --tag v0.4.2
```

CI runs the version check automatically. Tag-triggered and manual release runs
fail if the release tag is not valid SemVer or does not match the app version.

## Release a New Package

1. Choose the SemVer bump.
2. Bump all version files:

```bash
npm run version:set -- 0.4.2
npm run version:check -- --tag v0.4.2
```

3. Commit and push the change to `main`.
4. Create and push the matching tag:

```bash
git tag v0.4.2
git push origin main v0.4.2
```

The workflow will create or update the GitHub Release for that tag and upload
the macOS DMG assets.

To build without publishing a release, run the `macOS Package` workflow manually
with an empty `release_tag`.

To publish manually, run the workflow with a `release_tag` such as `v0.4.2`.
Manual releases are drafts by default.

## Upgrade Later to Developer ID Signing

When a paid Apple Developer Program account is available, keep the workflow and
add repository secrets instead of rewriting the release process.

Required signing secrets:

```bash
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
APPLE_SIGNING_IDENTITY
```

`APPLE_CERTIFICATE` is the base64-encoded `.p12` exported from Keychain Access.
`APPLE_SIGNING_IDENTITY` should look like:

```text
Developer ID Application: Your Name or Company (TEAMID)
```

Recommended notarization secrets using App Store Connect API:

```bash
APPLE_API_KEY
APPLE_API_ISSUER
APPLE_API_PRIVATE_KEY
```

Alternative notarization secrets using an Apple ID:

```bash
APPLE_ID
APPLE_PASSWORD
APPLE_TEAM_ID
```

`APPLE_PASSWORD` must be an app-specific password, not the normal Apple ID
login password.

After these secrets are configured, the workflow will use
`APPLE_SIGNING_IDENTITY` instead of the ad-hoc `-` identity. For non-ad-hoc
builds, CI also runs:

```bash
xcrun stapler validate <generated-dmg>
```

That makes a paid release fail fast if notarization or stapling did not complete.

## Useful Local Checks

For local release triage:

```bash
security find-identity -v -p codesigning
codesign --verify --deep --strict --verbose=2 "src-tauri/target/release/bundle/macos/Skill Notebook.app"
hdiutil verify "src-tauri/target/release/bundle/dmg/Skill Notebook_0.4.2_aarch64.dmg"
xcrun stapler validate "src-tauri/target/release/bundle/dmg/Skill Notebook_0.4.2_aarch64.dmg"
```

The `stapler validate` command is expected to fail for ad-hoc packages.

## References

- Apple Developer ID: https://developer.apple.com/support/developer-id/
- Apple macOS code signing security: https://support.apple.com/en-mide/guide/security/sec3ad8e6e53/web
- Tauri macOS code signing: https://v2.tauri.app/ko/distribute/sign/macos/
- Tauri environment variables: https://tauri.app/ja/reference/environment-variables/
- Tauri GitHub Action: https://github.com/tauri-apps/tauri-action
