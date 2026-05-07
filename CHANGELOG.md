# Changelog

This project uses Semantic Versioning.

## [0.4.2] - 2026-05-07

### Added

- GitHub Actions macOS packaging for Apple Silicon and Intel DMG artifacts.
- Optional GitHub Release publishing from matching SemVer tags.
- SemVer helper scripts for checking and updating all app version files.

### Changed

- CI now validates that release tags are `v`-prefixed SemVer and match the app
  version exactly.
- macOS packaging defaults to ad-hoc signing when Apple Developer ID secrets are
  not configured, while preserving the notarized path for future paid releases.

### Validation

- `npm run version:check -- --tag v0.4.2`
- GitHub Actions validates `lint`, frontend build, browser/CLI E2E, Rust tests,
  Tauri build, code signing verification, and DMG verification.

## [0.1.0] - 2026-04-29

Initial V1 acceptance release.

### Added

- Tauri desktop bundle for Skill Notebook.
- Project-root management with recent roots.
- Workbench UI aligned to the runnable design baseline.
- Preview-before-save creation from text, local sources, and URL.
- Package file editing, metadata editing, eval, smoke tests, formal version save, diff, and restore.
- Export/use modal with native sanitized zip export.
- Release CLI commands for find, create, eval, test, and version flows.

### Acceptance

- Release bundle CLI passed `find`, `eval`, `test`, `create`, `version save`, `version diff`, and `version restore` against a temporary real project root copied from `examples/project-root`.
- Script-backed smoke test executed `scripts/run.sh` successfully from the release CLI.
- Formal version restore removed an intentional `SKILL.md` drift introduced during acceptance.
- DMG checksum verification passed with `hdiutil verify`.
- Mounted DMG contained `Skill Notebook.app` and `/Applications` symlink.
- `.app` bundle from both build output and mounted DMG passed `codesign --verify --deep --strict`.
- Release `.app` launched as a macOS process and exited cleanly.

### Distribution Notes

- Version files are aligned at `0.1.0` in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- The build is ad-hoc signed for local validation. Developer ID signing and Apple notarization are still required for external distribution.
