# Skill Notebook PRD

## Positioning

Skill Notebook is a personal, local-first skill asset workbench for macOS.

It helps a single user:

- manage local skill packages in one fixed project `.skills/` root
- inspect, edit, search, and organize skill packages as durable local assets
- evaluate package quality before promoting a skill state
- save evaluated package states as formal versions
- quickly reference or hand off a selected skill to real agent runtimes and local CLI workflows

Skill Notebook does not own skill generation. Skill creation usually happens in real work contexts: Claude, Codex, OpenClaw, shell sessions, project folders, exported chats, scripts, notes, and existing package drafts. Skill Notebook receives those artifacts, makes them inspectable, evaluates them, versions them, and makes them easy to use again.

## Product Sentence

Let skills be preserved like notes, evaluated like assets, versioned like code, and reused like tools.

## Core Users

- people already working with local agent or CLI workflows
- users with dozens of skill packages spread across folders, tools, and agent runtimes
- users who prefer local control over cloud-hosted skill assets
- users who need to quickly reference a known-good skill while working in Claude, Codex, OpenClaw, or a shell

## V1 Scope

### Core loop

`Manage -> Evaluate -> Version -> Reference`

Daily usage often looks like:

`Find a skill -> reference/use it -> improve files when needed -> run eval/test -> save a formal version`

Draft creation is a peripheral path:

`Start/import a draft -> external agent edits it -> inspect/evaluate -> commit into .skills/`

### Core Capabilities

1. Skill library and unified local package management
2. Per-skill formal version management
3. Local asset evaluation and smoke-test visibility
4. Quick reference, export, and runtime handoff

### Peripheral Capabilities

- Draft workspace bootstrap for external agents
- Import from local folders, files, URLs, or temporary draft directories
- Compatibility shims for older direct-create flows until the implementation is migrated

## Product Principles

- local first
- macOS only
- CLI first, shell second
- one fixed `.skills/` directory per project root
- skill package as the core unit
- draft workspaces and formal asset versions are separate states
- eval should be visible before a formal version save
- reference/use actions should be faster than browsing the filesystem manually
- desktop and future web surfaces should stay thin and call the same core commands
- Skill Notebook should not become a model provider, prompt generator, or agent runtime

## Core Commands

1. `skill find`
2. `skill eval`
3. `skill test`
4. `skill version`
5. `skill use` or `skill reference`
6. `skill import`

Draft helpers are allowed, but they are not the primary product loop:

- `skill draft start`
- `skill draft discard`
- `skill draft import`

## Information Architecture

### Home

- recent project roots
- open project root
- create project root

### Main project

Three-pane layout:

- left: library, search, filters, package list
- center: package/file browser, preview, editing, test output
- right: metadata, eval, versions, quick reference/use actions

### Settings

- project root
- fixed `.skills/` directory
- shell assumptions
- preferred terminal/editor/agent command for draft handoff
- future command wiring

## Formal Version Rules

- formal versions are tied to eval results
- each package keeps up to 10 formal versions
- future pinned versions may avoid eviction
- restoring a version should feel deliberate
- quick reference should default to the current formal or explicitly selected package state

## Explicit Non-Goals

- owning skill generation inside the GUI
- acting as a model provider runtime or OpenAI-compatible client
- marketplace publishing
- public skill sharing
- team collaboration
- cloud accounts or sync
- cross-platform abstractions for V1
- full benchmark platforms or team scoreboards

## Milestones

1. Local project shell, package list, and editor frame
2. Filesystem-backed package loading, search, and metadata editing
3. Local eval/test visibility and formal version save, diff, and restore
4. Quick reference/use modal, export, symlink, and terminal handoff actions
5. Draft workspace bootstrap and import flow for externally created skills
