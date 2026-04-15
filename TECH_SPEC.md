# Skill Notebook Technical Spec

## Target

- platform: macOS only
- hardware: Apple Silicon first
- shell assumption: `zsh` or `bash`

## Recommended Stack

- desktop shell: `Tauri 2`
- core logic: `Rust`
- UI: `React + TypeScript`
- bundler: `Vite`
- state: `Zustand`
- storage: local filesystem + `SQLite`

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
- provide the three-pane workspace
- host editors, previews, and status surfaces
- invoke Rust commands

### App Core

Responsibilities:

- workspace management
- package discovery
- version logic
- eval orchestration
- shell command execution
- snapshot trimming

### Persistence

- package source lives on disk
- metadata, index, version records, and eval results live in SQLite
- snapshots live under `.skill-notebook/snapshots`

## Workspace Shape

```txt
workspace/
  packages/
    my-skill/
  .skill-notebook/
    notebook.db
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
  repositories/
  storage/
  utils/
  state/
  watchers/
  config/
```

## V1 Command Surface

- `app_bootstrap`
- `workspace_open`
- `workspace_create`
- `workspace_list_recent`
- `package_list`
- `package_get`
- `package_create_from_nl`
- `package_update`
- `package_search`
- `package_run_eval`
- `package_list_versions`
- `package_save_version`
- `package_restore_version`
- `package_run_test`
- `settings_get`
- `settings_update`

## Early Implementation Guidance

- keep commands thin
- keep business truth in Rust services
- treat package directories as first-class objects
- keep draft state out of the formal version table
- do not over-design multi-user or cloud concerns in V1
