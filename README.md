# Skill Notebook

Skill Notebook is a macOS-first skill asset workbench built around a Rust CLI core plus a thin desktop shell.

The core assumption is simple: every project has one fixed skill root at `.skills/`, and the app or CLI manages that directory rather than chasing scattered folders.

It is designed to feel like a hybrid of:

- a notes app for long-term knowledge capture
- an IDE for editing skill packages
- a lightweight Git client for version history
- a quick reference surface for using local skills in Claude, Codex, OpenClaw, and shell workflows

The product direction in this repository comes from the source conversation at `chats/202604131741.md`.

## V1 Focus

Skill Notebook V1 is intentionally narrow:

- unified local skill package management
- per-skill formal version management
- local eval and smoke-test visibility
- quick reference, export, and runtime handoff
- draft workspace bootstrap and import for externally created skills

The product loop is:

`Manage -> Evaluate -> Version -> Reference`

Daily usage often looks like:

`Find a skill -> reference/use it -> improve files when needed -> run eval/test -> save a formal version`

The product is intentionally organized around core commands such as:

- `skill find`
- `skill eval`
- `skill test`
- `skill version`
- `skill reference`
- `skill import`

## Stack

- `Tauri 2`
- `Rust`
- `React`
- `TypeScript`
- `Vite`
- `Zustand`

## Current State

This repository currently includes:

- a runnable Tauri + React desktop shell
- a three-pane project UI with real package file editing
- a Rust CLI core for local package discovery, eval, tests, export, and versioning
- formal version save, diff, and restore flows
- `.42cog/PRD.md` and `.42cog/TECH_SPEC.md` distilled from the source chat

## Development

Install dependencies:

```bash
npm install
```

Run the CLI core:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin skill -- --help
```

Run the desktop app:

```bash
npm run tauri:dev
```

Run the web shell only:

```bash
npm run dev
```

## Validation

Run the fast repeatable E2E rails:

```bash
npm run test:e2e
```

This covers the Rust CLI/core filesystem loop and the browser workbench loop.

Run the macOS native bundle smoke as well:

```bash
npm run test:e2e:all
```

The native rail builds the Tauri bundle, verifies ad-hoc signing, launches `Skill Notebook.app`, and checks that the app process/window starts. On macOS, Tauri's standard WebDriver path is not available, so business-command coverage stays in the CLI/core rail.

## Draft handoff bridge

Skill Notebook does not own skill generation. Skill creation usually happens in Claude, Codex, OpenClaw, shell sessions, project folders, exported chats, scripts, notes, and existing package drafts.

The draft flow is a lightweight handoff:

```bash
skill draft start "Turn meeting notes into owner/date/risk/action items"
cd .skill-notebook/drafts/<draft-id>
codex
skill draft import .skill-notebook/drafts/<draft-id>
```

The app prepares `BRIEF.md`, a package skeleton, and import/reference commands. External agents do the authoring; Skill Notebook manages the resulting asset.

## Repo Layout

- `src/`: React UI shell
- `src-tauri/`: Rust core and Tauri commands
- `chats/`: source product conversations
- `.42cog/`: canonical product, technical, and frontend design specifications
- `docs/`: supporting docs and runnable design artifacts
- `examples/`: sample project-root material

## Near-Term Milestones

1. Project-root bootstrapping and local `.skills/` discovery
2. Package detail loading from filesystem
3. quick reference/use commands for selected skills
4. draft workspace bootstrap and import flow
5. formal version save / restore flow with snapshot trimming
