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

1. manage local skill packages
2. index and retrieve packages locally
3. call `skill-create` when available (with optional Claude CLI + template fallback)
4. run eval and attach its result to a formal version
5. keep at most 10 formal versions per package
6. support preview and CLI test entry points

## Architecture

### UI Layer

Responsibilities:

- render the desktop shell
- provide the three-pane project shell
- host editors, previews, and status surfaces
- invoke Rust commands

### App Core

Responsibilities:

- expose the `skill find/create/eval/version` CLI contract
- project-root management
- package discovery
- version logic
- eval orchestration
- shell command execution
- snapshot trimming

### Persistence

- package source lives on disk
- metadata, version records, and eval results live in package `notebook.json` files in V1
- search and bootstrap are rebuilt from the filesystem in V1
- SQLite may be reconsidered later for indexing/cache, but placeholder modules should not remain in V1 code
- snapshots live under `.skill-notebook/snapshots`

## Project Root Shape

```txt
project-root/
  .skills/
    my-skill/
      notebook.json
  .skill-notebook/
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
  notebook.json
```

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

- CLI:
  - `skill doctor generator`
  - `skill find`
  - `skill create`
  - `skill create preview`
  - `skill create commit`
  - `skill create discard`
  - `skill eval`
  - `skill test`
  - `skill export zip`
  - `skill version`
- CLI JSON output must expose script-ready top-level handles for lifecycle commands:
  - `create.preview`: `previewId`, `slug`, `generatorUsed`, `fileCount`, `commitCommand`
  - `create.commit`: `previewId`, `packageId`, `slug`, `packagePath`, `generatorUsed`
  - `export.zip`: `packageId`, `zipPath`, `sizeBytes`
  - full nested objects remain available for GUI/API parity
- Claude CLI generation retries transient 429/rate-limit/overload failures before surfacing an error:
  - `SKILL_NOTEBOOK_CLAUDE_RETRY_ATTEMPTS` controls total attempts
  - `SKILL_NOTEBOOK_CLAUDE_RETRY_BACKOFF_SECS` controls exponential base backoff
  - retry failure must not silently become template fallback
- `app_bootstrap`
- `project_root_open`
- `project_root_create`
- `project_root_list_recent`
- `package_list`
- `package_get`
- `package_create_from_nl`
- `package_generate_preview_from_nl`
- `package_generate_preview_from_sources`
- `package_generate_preview_from_url`
- `package_commit_preview`
- `package_discard_preview`
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
- `settings_get`
- `settings_update`

## Early Implementation Guidance

- keep commands thin
- keep business truth in Rust services
- treat package directories as first-class objects
- keep draft state out of the formal version table
- do not over-design multi-user or cloud concerns in V1
