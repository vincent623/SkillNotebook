# Skill Notebook Technical Spec

## Target

- platform: macOS only
- hardware: Apple Silicon first
- shell assumption: `zsh` or `bash`

## Recommended Stack

- desktop shell: `Tauri 2`
- core CLI: `Rust`
- core logic: `Rust`
- UI: `React + TypeScript`
- bundler: `Vite`
- state: `Zustand`
- storage: local filesystem in V1; SQLite is optional future architecture

## Technical Goals

1. manage local skill packages under a fixed `.skills/` root
2. index, search, and retrieve packages locally
3. run local eval and smoke-test checks for package quality
4. attach eval results to formal package versions
5. keep at most 10 formal versions per package
6. expose quick reference and runtime handoff commands
7. bootstrap temporary draft workspaces without owning skill generation
8. import externally created draft packages into `.skills/`

## Architecture

### UI Layer

Responsibilities:

- render the desktop shell
- provide the three-pane project shell
- host editors, previews, status surfaces, and quick reference actions
- invoke Rust commands
- avoid direct model-provider or generation runtime logic

### App Core

Responsibilities:

- expose the `skill find/eval/test/version/use/import/draft` CLI contract
- project-root management
- package discovery
- metadata update
- version logic
- eval and smoke-test orchestration
- shell command execution behind explicit user action
- export, symlink, and reference command construction
- snapshot trimming
- draft workspace bootstrap and cleanup

### Persistence

- package source lives on disk
- metadata, version records, and eval results live in package `notebook.json` files in V1
- search and bootstrap are rebuilt from the filesystem in V1
- SQLite may be reconsidered later for indexing/cache, but placeholder modules should not remain in V1 code
- formal snapshots live under `.skill-notebook/snapshots`
- temporary draft workspaces live under `.skill-notebook/drafts`

## Project Root Shape

```txt
project-root/
  .skills/
    my-skill/
      notebook.json
  .skill-notebook/
    drafts/
    snapshots/
    logs/
    cache/
    config.json
```

## Package Shape

```txt
my-skill/
  SKILL.md
  prompts/
  examples/
  references/
  scripts/
  tests/
  evals/
  notebook.json
```

## Draft Workspace Shape

Draft workspaces are temporary handoff surfaces for Claude, Codex, OpenClaw, or other external agents. They are not formal assets until imported into `.skills/`.

```txt
.skill-notebook/drafts/
  draft-<slug>-<id>/
    BRIEF.md
    SKILL.md
    prompts/
    examples/
    references/
    scripts/
    tests/
    evals/
    draft.json
```

`BRIEF.md` should contain:

- goal
- intended skill name or slug
- input and output expectations
- boundaries and when not to use the skill
- source material references
- acceptance checks
- suggested external agent command

`draft.json` should contain machine-readable metadata:

- draft id
- source prompt or source paths
- created time
- recommended agent command
- intended import slug
- originating project root

## Frontend Shape

```txt
src/
  app/
  components/
  services/
  stores/
  types/
  styles/
```

## Rust Shape

```txt
src-tauri/src/
  commands/
  domain/
  services/
  storage/
  utils/
  state/
  config/
```

## V1 Command Surface

CLI:

- `skill find`
- `skill list`
- `skill eval`
- `skill test`
- `skill version list`
- `skill version save`
- `skill version diff`
- `skill version restore`
- `skill use`
- `skill reference`
- `skill export zip`
- `skill import`
- `skill draft start`
- `skill draft list`
- `skill draft discard`
- `skill draft import`

CLI JSON output must expose script-ready top-level handles for lifecycle commands:

- `draft.start`: `draftId`, `draftPath`, `briefPath`, `suggestedCommand`, `importCommand`
- `import`: `packageId`, `slug`, `packagePath`, `evalCommand`, `versionCommand`
- `use/reference`: `packageId`, `packagePath`, `skillMdPath`, `copyableReferences`, `linkCommands`, `terminalCommand`
- `export.zip`: `packageId`, `zipPath`, `sizeBytes`
- full nested objects remain available for GUI/API parity

Tauri commands:

- `app_bootstrap`
- `project_root_open`
- `project_root_create`
- `project_root_list_recent`
- `package_list`
- `package_get`
- `package_file_tree`
- `package_file_read`
- `package_file_write`
- `package_update`
- `package_search`
- `package_run_eval`
- `package_run_test`
- `package_list_versions`
- `package_save_version`
- `package_diff_version`
- `package_restore_version`
- `package_export_zip`
- `package_reference`
- `package_import`
- `draft_start`
- `draft_list`
- `draft_discard`
- `draft_import`
- `settings_get`
- `settings_update`

Legacy GUI-owned create commands are removed from the public backend contract. Authoring starts in draft workspaces and is completed by an external agent before import.

## Runtime Handoff Policy

Skill Notebook is runtime-agnostic.

It may generate local commands, open directories, or create temporary draft workspaces, but it must not own the model call used to author a skill. External agents such as Claude, Codex, OpenClaw, or a user-defined command do the creative work in a draft directory.

Settings may store local preferences:

- preferred terminal command
- preferred editor command
- preferred external agent command
- default Claude/Codex/OpenClaw skill-link target directories

Settings should not require model provider, base URL, model, or API key fields for the core product loop.

## Validation Rails

E2E validation should stay split into three rails:

- core CLI/filesystem rail: temporary project root, `skill import`, `skill reference`, `skill draft start`, `skill draft import`, and discovery through `skill find`
- browser workbench rail: Vite shell, draft/import screen, draft bootstrap UI, and quick reference modal
- macOS native smoke rail: Tauri bundle build, strict `codesign` verification, app launch, and window/process detection

The native macOS rail is intentionally a smoke test. Tauri's standard WebDriver support is currently limited to Windows and Linux because macOS does not provide a WKWebView driver. Product behavior that mutates local skill assets must therefore be covered by the shared Rust CLI/core rail, while the browser rail covers frontend interaction semantics.

NPM scripts:

- `npm run test:e2e:core`
- `npm run test:e2e:browser`
- `npm run test:e2e:native`
- `npm run test:e2e`
- `npm run test:e2e:all`

## Quick Reference Outputs

For a selected package, the core should provide:

- absolute package path
- absolute `SKILL.md` path
- package-relative important files
- copyable Markdown reference snippet
- copyable CLI/path reference snippet
- symlink command for `~/.claude/skills/<slug>`
- symlink command for `<project>/.claude/skills/<slug>`
- future symlink/copy targets for Codex/OpenClaw when their local skill conventions are known
- terminal command to open the package root
- zip export command and result

All commands must be derived from real local paths.

## Early Implementation Guidance

- keep commands thin
- keep business truth in Rust services
- treat package directories as first-class objects
- keep draft state out of the formal version table
- do not silently generate skill content in the GUI
- do not require provider/API-key configuration for V1 management workflows
- do not over-design multi-user or cloud concerns in V1
