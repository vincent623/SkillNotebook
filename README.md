# Skill Notebook

Skill Notebook is a macOS-first skill workbench built around a Rust CLI core plus a thin desktop shell.

The core assumption is simple: every project has one fixed skill root at `.skills/`, and the app or CLI manages that directory rather than chasing scattered folders.

It is designed to feel like a hybrid of:

- a notes app for long-term knowledge capture
- an IDE for editing skill packages
- a lightweight Git client for version history

The product direction in this repository comes from the source conversation at `chats/202604131741.md`.

## V1 Focus

Skill Notebook V1 is intentionally narrow:

- skill package version management
- natural-language skill drafting via `skill-create` (preferred), with optional Claude CLI + template fallback
- local eval flow
- local search and retrieval

The product loop is:

`Find -> Create -> Evaluate -> Version`

The product is intentionally organized around four core commands:

- `skill find`
- `skill create`
- `skill eval`
- `skill version`

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
- a Rust CLI core for `find / create / eval / version`
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

## Draft generation bridge

Natural-language package creation supports multiple generators:

- `skill-create` (preferred when installed)
- Claude CLI (`claude`) as an optional bridge
- local template fallback

Configuration via env vars:

- `SKILL_NOTEBOOK_CREATOR_MODE`: `auto` (default), `skill_create`, `claude_cli`, `template`
- `SKILL_NOTEBOOK_SKILL_CREATE_BIN`: path/name for `skill-create` (default: `skill-create`)
- `SKILL_NOTEBOOK_SKILL_CREATE_TIMEOUT_SECS`: timeout for `skill-create` (default: `60`)
- `SKILL_NOTEBOOK_CLAUDE_BIN`: path/name for Claude CLI (default: `claude`)
- `SKILL_NOTEBOOK_CLAUDE_MODEL`: optional model name passed to Claude CLI
- `SKILL_NOTEBOOK_CLAUDE_TIMEOUT_SECS`: timeout for Claude CLI (default: `60`)

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
3. `skill-create` (or Claude CLI) integration and first eval pass
4. formal version save / restore flow with snapshot trimming
