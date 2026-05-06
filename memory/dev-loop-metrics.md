# Dev Loop Metrics

## 2026-04-14

### Alignment Snapshot

| Metric | Value | Notes |
| --- | --- | --- |
| Core loop operational coverage | 4/4 | `Find`, `Create`, `Evaluate`, and `Version` (save) are now usable end-to-end. |
| Milestones substantially aligned | 3/4 | M1, M2, and M3 are in place (skill-create supported, with Claude CLI + template fallback). |
| Milestones partially aligned | 1/4 | M4 has formal version save + snapshot creation; restore + diff are still missing. |
| Milestones not yet delivered | 0/4 | Remaining work is scoped to restore + diff, not the initial save. |

### Drift And Risk Counts

| Metric | Value | Notes |
| --- | --- | --- |
| Spec drift items found | 0 major | `skill-create` vs Claude CLI creation semantics are reconciled (skill-create-first + documented fallbacks). |
| Logic inconsistencies found | 1 | Auto-create eval can downgrade `Validated` to `NeedsEval`, while manual rerun does not. |
| Placeholder or scaffold-only backend surfaces | historical 4 | Superseded by later work: `package_update`, restore, and run-test are now implemented; SQLite placeholder modules were removed from V1 code. |
| Critical runtime regressions fixed this round | 2 | Claude CLI timeout guard; generator title preservation across slug dedupe. |

### Architecture Reality Check

| Metric | Value | Notes |
| --- | --- | --- |
| Current-project root runtime context | implemented | Default and manually opened project roots now drive bootstrap, create, eval, search, and version lookups in the active app session. |
| Storage backends actually in use | 1 | Filesystem-backed package notebooks are the V1 persistence path; SQLite is optional future architecture rather than a checked-in placeholder backend. |
| Formal version actions implemented | 1/3 | Save is wired (including snapshots); restore + diff remain. |

## 2026-04-27

### Spec And Design Alignment Snapshot

| Metric | Value | Notes |
| --- | --- | --- |
| Canonical spec home | `.42cog/` established | PRD, TECH_SPEC, and frontend design spec now live under the intended spec directory. |
| Runnable UI design baseline | `docs/skillnotebook/` retained | Kept as a runnable prototype/design artifact instead of moving it into `.42cog`. |
| Production frontend alignment with design baseline | partial | Current app now has a single workbench shell, skill search/filtering, command palette, Finder-style file columns, export/use modal, and rich Markdown/frontmatter editor preview; it still lacks the prototype's preview-before-save generation flow. |
| Backend alignment with product loop | strong | Rust core supports project-root scanning, create, eval, version save/diff/restore; remaining gaps are mostly UI exposure and placeholder surfaces. |
| Verification baseline | green | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm run lint`, and `npm run build` pass after project-root fixture cleanup and frontend Phase 1. |

### Drift And Cleanup Counts

| Metric | Value | Notes |
| --- | --- | --- |
| Major source-of-truth locations | 2 | `.42cog/` for specifications; `docs/skillnotebook/` for runnable UI prototype artifact. |
| UI baseline gaps identified | 1 | Generation preview flow remains; export/soft-link usage flow is now represented in production UI. |
| Test drift items resolved | 2 | Replaced stale `examples/project_root` references and normalized the sample project root id to `project-root-main`. |
| Placeholder backend surfaces still visible | historical 3+ | Superseded by the current gap map; run-test and SQLite placeholder files have since been cleaned up. |

### Frontend Alignment Phase 1

| Metric | Value | Notes |
| --- | --- | --- |
| Editor daily-surface improvements shipped | 5/5 | Markdown preview, frontmatter summary, copy affordance, word count, and save state are now present in the production editor. |
| New frontend files added | 3 | `MarkdownPreview.tsx`, `FrontmatterCard.tsx`, and `markdown-utils.ts`. |
| Additional runtime dependencies | 0 | Markdown rendering is a scoped React renderer; no new package added. |
| Phase 1 validation | green | `npm run lint`, `npm run build`, and browser smoke check at `http://127.0.0.1:1420/` pass with 0 console errors. |

### Frontend Alignment Phase 2A

| Metric | Value | Notes |
| --- | --- | --- |
| Workbench shell regions shipped | 3/3 | Skill Library, Package Browser, and Content Pane are now visible in one main surface. |
| Skill discovery controls shipped | 3/3 | Search, status filter, and tag filter are available in the library column. |
| Command palette baseline shipped | 5 actions | Open skill, generate skill, open settings, copy current skill path, and run eval. |
| Remaining prototype-level UI gaps | 3 | Finder-style columns, preview-before-save creation, and export/use modal remain. |
| Phase 2A validation | green | `npm run lint`, `npm run build`, and browser smoke check at `http://127.0.0.1:1420/` pass with 0 console errors. |

### Frontend Alignment Phase 2B

| Metric | Value | Notes |
| --- | --- | --- |
| Finder-style file browser shipped | yes | Workbench package browser now uses column navigation instead of the interim recursive file tree. |
| File browser column behavior | root + active directory chain | Selecting a directory opens the next column; selecting a file opens content in the pane. |
| Browser sorting/hiding rules | implemented | Hidden entries and `notebook.json` are filtered; directories sort before files; `SKILL.md` is prioritized among files. |
| Remaining prototype-level UI gaps | 2 | Preview-before-save creation and export/use modal remain. |
| Phase 2B validation | green | `npm run lint`, `npm run build`, and browser smoke check at `http://127.0.0.1:1420/` pass with 0 console errors. |

### Frontend Alignment Phase 3

| Metric | Value | Notes |
| --- | --- | --- |
| Export/use actions shipped | 4/4 | Copy package path, copy `SKILL.md` path, copy global Claude skills symlink command, copy project-local Claude skills symlink command. |
| Commands derived from real paths | yes | Uses selected package `rootPath` and active project root path. |
| New backend requirement | none | Phase 3 is frontend-only and uses existing bootstrap data. |
| Remaining prototype-level UI gaps | 1 | Preview-before-save creation flow remains. |
| Phase 3 validation | green | `npm run lint`, `npm run build`, and browser smoke check at `http://127.0.0.1:1420/` pass with 0 console errors. |

### Frontend Alignment Phase 4

| Metric | Value | Notes |
| --- | --- | --- |
| Preview-before-save create flow shipped | yes | Create now generates a preview, shows generated files, and only writes to `.skills/` after confirmation. |
| Preview workspace contract | implemented | Backend writes previews to `.skill-notebook/create-previews/<preview-id>/package/` and returns file tree + file contents. |
| Commit finalization coverage | package + notebook + eval | Commit copies the preview into `.skills/<slug>/`, saves notebook metadata, runs eval, removes the preview workspace, and refreshes bootstrap in the UI. |
| Major prototype-level UI gaps remaining | 0 | The previously tracked workbench, file browser, export/use modal, and create preview gaps are now represented in production UI. |
| Phase 4 validation | green | `cargo test --manifest-path src-tauri/Cargo.toml`, `npm run lint`, `npm run build`, and browser smoke check at `http://127.0.0.1:1420/` pass with 0 console errors. Browser warnings are expected in non-Tauri fallback mode. |

### Frontend Alignment Phase 5A

| Metric | Value | Notes |
| --- | --- | --- |
| Quality gate panel shipped | yes | Version panel now shows eval score, component scores, structural checks, and suggestions next to version actions. |
| Formal version save modal | implemented | Save flow requires a version note and shows the eval snapshot before creating the formal version. |
| Restore confirmation modal | implemented | Restore flow warns that package files will be overwritten and editor state reloaded. |
| Browser demo eval/version contract | implemented | Browser fallback now supports run eval, save version, diff, and restore using the same frontend API shape. |
| Phase 5A validation | green | `npm run lint`, `npm run build`, and browser smoke check at `http://127.0.0.1:1420/` pass with 0 console errors. |

### Create Preview Lifecycle Hardening

| Metric | Value | Notes |
| --- | --- | --- |
| Explicit preview discard command | implemented | `package_discard_preview` removes abandoned `.skill-notebook/create-previews/<preview-id>/` workspaces and returns `false` for already-missing previews. |
| Frontend discard integration | implemented | Create view discards previews on clear, replacement, and unmount; demo mode follows the same API contract. |
| Commit failure cleanup | implemented | Preview commit removes partial `.skills/<slug>/`, eval workspace, and generator log side effects on post-copy failure while keeping the preview draft. |
| Lifecycle regression tests | 2 | Added tests for discard-without-package and failed commit cleanup. |
| Hardening validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml` (26 tests), `npm run lint`, and `npm run build` pass. |

### Package Smoke Test Execution

| Metric | Value | Notes |
| --- | --- | --- |
| `package_run_test` scaffold status | replaced | Command now calls Rust service logic instead of returning a placeholder string. |
| Test execution mode | local smoke JSON | Reads `tests/*.json`, validates input/expectations, package target, and expectation coverage against package content. |
| Frontend quality gate integration | implemented | Version panel now exposes Run Test and shows pass/fail/missing summaries with check details. |
| Browser demo test contract | implemented | Browser fallback returns the same `PackageTestReport` shape as Tauri. |
| Package test regression tests | 3 | Covers passing smoke tests, missing tests, and malformed JSON failures. |
| Package test validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml` (29 tests), `npm run lint`, and `npm run build` pass. |

### Preview TTL Cleanup

| Metric | Value | Notes |
| --- | --- | --- |
| Create preview TTL | 24 hours | Stale `.skill-notebook/create-previews/<preview-id>/` workspaces are removed after the TTL. |
| Cleanup entry points | 2 | Runs during app bootstrap and before generating a new package preview. |
| Fresh preview preservation | tested | Cleanup removes only expired previews and keeps fresh preview workspaces intact. |
| Preview TTL regression tests | 1 | Added deterministic stale/fresh cleanup coverage using manifest `createdAt`. |
| Preview TTL validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml` and `cargo test --manifest-path src-tauri/Cargo.toml` (30 tests) pass. |

### Local Source Create Flow

| Metric | Value | Notes |
| --- | --- | --- |
| Local file/directory create mode | implemented | Create view now supports text and file/directory modes; URL remains disabled. |
| Backend source preview command | implemented | `package_generate_preview_from_sources` builds a preview from local paths using bounded inventory and text excerpts. |
| Source traceability artifact | implemented | Preview packages include `references/source-inventory.md` before commit. |
| Source summary consistency | implemented | Source-mode preview manifest and response now share the same generation summary before commit. |
| Source collection limits | bounded | Recursion skips hidden/dependency/internal dirs and includes up to 40 files with text excerpts capped per file. |
| Source preview regression tests | 1 | Covers local source path preview and inventory reference attachment. |
| Source create validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml` (31 tests), `npm run lint`, and `npm run build` pass. |

### Package Update And Placeholder Cleanup

| Metric | Value | Notes |
| --- | --- | --- |
| `package_update` scaffold status | replaced | Command now updates package notebook metadata and returns the updated package. |
| Package update regression tests | 2 | Covers metadata persistence and empty-name rejection. |
| Empty placeholder modules removed | 9 files | Removed unused SQLite repository placeholders, filesystem watcher placeholder, SQLite storage placeholder, and shell wrapper placeholder. |
| V1 persistence decision | filesystem notebooks | SQLite is no longer represented by empty code modules; it remains optional future architecture only. |
| Package update cleanup validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml`, `cargo test --manifest-path src-tauri/Cargo.toml` (33 tests), `npm run lint`, and `npm run build` pass. |

### Current Remaining Gap Map

| Metric | Value | Notes |
| --- | --- | --- |
| Prototype main-flow UI gaps | 0 | Workbench, file browser, editor preview, export/use, preview create, eval/version gate, smoke test action, and metadata editing are represented in production UI. |
| Scaffolded `package_run_test` gap | closed | Local smoke JSON execution is now wired through backend, frontend API, store, quality gate UI, and CLI `skill test`. |
| Create source gaps | 0 | Text, file/directory, and URL preview generation are all available; URL previews attach `references/url-source.md`. |
| Preview lifecycle gap | 0 | Active discard, commit cleanup, and passive TTL cleanup are all implemented. |
| Test execution depth gap | 0 | `tests/*.json` can declare package-local `scripts/` or `tests/` script execution with path traversal and symlink checks plus a timeout. |
| Infrastructure gaps | 0 | Recent project roots persist across app launches; clean-editor polling refresh covers bootstrap, file tree, and open-file external edits; sanitized zip export is implemented. |

### Full Spec Gap Closure

| Metric | Value | Notes |
| --- | --- | --- |
| Topbar project-root/status/settings alignment | implemented | Topbar now shows active project root, load status, and settings icon entry. |
| Settings project-root workflow | implemented | Settings exposes current root, clickable recent roots, project-root creation, `.skills`, version cap, and creation bridge status. |
| Dirty editor switch protection | implemented | File/package switches and top-level navigation gate dirty edits with save/discard/cancel decisions. |
| Library metadata density | implemented | Library rows show package status and latest eval score alongside version/date/tags. |
| Package metadata edit UI | implemented | Empty content pane now exposes editable name/description/status/tags/related/bundle metadata backed by `package_update`. |
| URL create mode | implemented | Native mode fetches `http(s)` source text with bounded curl-based ingestion; browser fallback records the URL source. |
| Script-backed package tests | implemented | CLI/API test reports include script execution checks; sample `pdf-brief-builder` smoke test declares `scripts/run.sh`. |
| Native sanitized export | implemented | Tauri command writes zip artifacts to `.skill-notebook/exports/`, excluding hidden files and `notebook.json`. |
| Watch refresh | implemented | Browser workbench refreshes bootstrap/file tree/open file every 5s only when the editor is clean and visible. |
| Gap closure validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --manifest-path src-tauri/Cargo.toml` (36 tests), `npm run lint`, `npm run build`, CLI `skill test`, CLI `skill find`, and Playwright browser checks pass. |

### Prototype Parity Regression Repair

| Metric | Value | Notes |
| --- | --- | --- |
| Hidden version workflow regressions repaired | 4/4 | Eval, test, save version, diff, and restore are available from the top-bar version panel; save remains gated by eval presence. |
| Direct prompt save shortcuts removed | 2 | Removed prompt-based save from top bar and command palette in favor of the guarded VersionPanel flow. |
| Editor save-and-exit behavior | fixed | `完成` now stays in edit mode when persistence fails. |
| Parity repair validation | green | `npm run lint`, `npm run build`, `git diff --check`, and browser interaction checks for version panel + diff modal pass. |

### Original Spec E2E Pass

| Metric | Value | Notes |
| --- | --- | --- |
| Core CLI E2E rail | green | Temporary project root completed `find`, filtered search, `eval`, `version list`, eval-backed `version save`, draft mutation `version diff`, `version restore`, and template-mode `create`. |
| Browser workbench E2E rail | green | Covered three-pane workbench, search/filter, `SKILL.md` open/edit/save, eval/test, diff modal, restore confirmation, note-gated version save modal, export/use modal, command palette, create preview commit, and disabled URL mode. |
| Rust validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `cargo test --manifest-path src-tauri/Cargo.toml` pass with 33 tests. |
| Frontend validation | green | `npm run lint`, `npm run build`, and `git diff --check` pass. |
| Browser console errors | 0 | Non-Tauri browser preview produced expected fallback warnings for missing Tauri `invoke`; no console errors were reported. |
| Dev server inspected | `http://127.0.0.1:1421/` | Server remains available for manual inspection after E2E. |

### Tauri macOS Bundle Pass

| Metric | Value | Notes |
| --- | --- | --- |
| Tauri package command | green | `npm run tauri:build` completed and produced both macOS app and DMG bundles. |
| App bundle artifact | `src-tauri/target/release/bundle/macos/Skill Notebook.app` | Release app bundle is present and launches locally. |
| DMG artifact | `src-tauri/target/release/bundle/dmg/Skill Notebook_0.2.1_aarch64.dmg` | DMG is present after bundle regeneration. |
| Local signing state | valid ad-hoc | Added `bundle.macOS.signingIdentity: "-"`; `codesign --verify --deep --strict --verbose=2` passes. |
| Distribution signing state | not notarized | `spctl` rejects the ad-hoc signed app, as expected without Developer ID notarization credentials. |
| Post-package lint scope | fixed | ESLint now ignores `target` and `src-tauri/target` so generated Tauri assets do not break `npm run lint`. |

### CLI-First Alignment Pass

| Metric | Value | Notes |
| --- | --- | --- |
| CLI generator diagnostics | implemented | Added `skill doctor generator` with preferred generator, resolved paths, availability, and Claude timeout. |
| CLI preview lifecycle | implemented | Added `skill create preview`, `skill create commit`, and `skill create discard` for text, local source path, and URL-backed previews. |
| CLI export | implemented | Added `skill export zip <package-id>` over the same sanitized native export service as the GUI. |
| Local path paste tolerance | implemented | Local source preview accepts quoted paths, `file://` paths, and common shell-escaped path characters. |
| Generator fallback policy | hardened | Detected generator failures now surface as errors instead of silently creating template fallback packages. |
| Claude default timeout | 300s | Default increased from 60s for real source-material generation; override with `SKILL_NOTEBOOK_CLAUDE_TIMEOUT_SECS`. |
| Script-ready CLI JSON | implemented | `create.preview`, `create.commit`, and `export.zip` now expose top-level lifecycle handles while keeping full nested objects. |
| CLI-first validation | green | `skill doctor generator`, quoted local-path `skill create preview --from-file`, `skill create commit`, `skill create discard`, and `skill export zip` pass as one command chain against a temporary project root. |

### V0.2.0 Full Acceptance

| Metric | Value | Notes |
| --- | --- | --- |
| Git release baseline | `e378405` / `v0.2.0` | Code was committed before acceptance and tagged with the semantic version. |
| Static validation | green | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --manifest-path src-tauri/Cargo.toml` (40 tests), `npm run lint`, `npm run build`, and `git diff --check` pass. |
| Release CLI chain | green | Bundled `Skill Notebook.app/Contents/MacOS/skill` completed doctor, text preview discard, quoted local-path preview commit, URL preview discard, eval, test, and export zip against a temporary project root. |
| Real Claude generation | green | `SKILL_NOTEBOOK_CREATOR_MODE=claude_cli` generated, committed, evaluated as usable, passed smoke test, and exported a zip. |
| Tauri package | green | `npm run tauri:build` produced `Skill Notebook.app` and `Skill Notebook_0.2.0_aarch64.dmg`; strict `codesign --verify --deep --strict` passes. |
| Browser GUI smoke | green | Vite preview rendered the workbench, create preview, commit, export modal, and settings page with zero console errors. Browser-only Tauri fallback warnings are expected. |
| Acceptance drift found | fixed | Settings About text now derives from the package version instead of a hard-coded `v0.1.0`. |

### V0.2.1 Claude 429 Resilience

| Metric | Value | Notes |
| --- | --- | --- |
| Patch trigger | Claude 429 overload | Real manual generation hit `API Error: Request rejected (429)` from Claude CLI stdout. |
| Retry policy | implemented | Claude CLI retries transient 429/rate-limit/overload failures with configurable attempts and exponential base backoff. |
| Default retry config | 3 attempts / 8s base | Override with `SKILL_NOTEBOOK_CLAUDE_RETRY_ATTEMPTS` and `SKILL_NOTEBOOK_CLAUDE_RETRY_BACKOFF_SECS`. |
| Fake-success guard | preserved | Exhausted retries still return an error; the system does not silently use template fallback after invoking Claude. |
| Diagnostics | updated | `skill doctor generator` and Settings expose Claude timeout and retry parameters. |
| Validation | green | `cargo test --manifest-path src-tauri/Cargo.toml` passes with 41 tests; `npm run lint`, `npm run build`, `git diff --check`, Tauri build, strict codesign, and bundled CLI `--version` pass. |

### V0.3.0 Pi Sidecar Runtime

| Metric | Value | Notes |
| --- | --- | --- |
| Runtime dependency | added | `@mariozechner/pi-ai` powers the generic OpenAI-compatible sidecar. |
| Sidecar artifact | implemented | Source lives in `sidecars/pi-skill-draft.mjs`; build output is `dist-sidecars/pi-skill-draft.mjs` and is packaged as a Tauri resource. |
| Generator order | updated | Auto mode prefers configured `pi_sidecar`, then `skill-create`, then Claude CLI, then explicit/no-generator template fallback. |
| Runtime configuration | implemented | Agent base URL, API key, model, provider, timeout, retries, Node binary, and sidecar script are env-driven. |
| Diagnostics/UI | implemented | `skill doctor generator`, Settings, and create-preview labels expose pi sidecar availability and provider/model state. |
| Fake-success guard | preserved | A configured pi sidecar failure is surfaced and does not silently become a local template draft. |
| Provider transport | streaming + fallback | Local OpenAI-compatible mock tests pass for pi-ai streaming SSE and non-streaming `/chat/completions` fallback. |
| Validation | green | `cargo test` passes with 43 tests; `npm run lint`, `npm run build`, `git diff --check`, mock pi CLI E2E, Tauri build, strict codesign, bundled CLI `--version`, and bundled sidecar resource resolution pass. |
