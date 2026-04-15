# Skill Notebook Agent Notes

## Product Position

Skill Notebook is not a public marketplace.

It is:

- a personal local skill workspace
- a skill package editor
- a versioned notebook for reusable agent capabilities

## Platform Rules

- macOS only for V1
- Apple Silicon first
- local filesystem is the source of truth
- local shell execution assumes `zsh` or `bash`

## Product Boundaries

Prioritize these capabilities:

1. version management
2. natural-language package creation
3. eval before formal version save
4. local search and retrieval

Avoid expanding into:

- cloud sync
- team collaboration
- marketplace flows
- multi-platform abstractions

## Architecture Bias

- keep business truth in Rust
- keep React focused on layout, editing, and interaction
- treat a skill as a package directory, not a single file
- separate draft state from formal version history

## UX Bias

- three-pane workspace
- notes-app calm, IDE clarity, lightweight desktop feel
- formal versions should feel significant
- eval should be visible and understandable, not hidden
