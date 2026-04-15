# Skill Notebook PRD

## Positioning

Skill Notebook is a personal, local-first skill management and editing tool for macOS.

It helps a single user:

- search existing skill packages
- create new draft packages from natural language
- evaluate package quality
- save validated packages as formal versions

## Product Sentence

Let skills be preserved like notes, versioned like code, and reused like tools.

## Core Users

- people already working with local agent or CLI workflows
- users with dozens of skill packages spread across folders and tools
- users who prefer local control over cloud-hosted skill assets

## V1 Scope

### Core loop

`Find -> Create -> Evaluate -> Version`

### Features

1. Skill package version management
2. Natural-language package drafting via `skill-create` (preferred), with optional Claude CLI + template fallback
3. Local eval flow
4. Local search and retrieval

## Product Principles

- local first
- macOS only
- skill package as the core unit
- draft edits and formal versions are separate states
- eval should be visible before a formal version save

## Information Architecture

### Home

- recent workspaces
- create workspace
- import workspace

### Main workspace

Three-pane layout:

- left: library, search, filters, package list
- center: overview, files, preview, test
- right: metadata, eval, versions

### Settings

- workspace root
- shell assumptions
- future command wiring

## Formal Version Rules

- formal versions are tied to eval results
- each package keeps up to 10 formal versions
- future pinned versions may avoid eviction
- restoring a version should feel deliberate

## Explicit Non-Goals

- marketplace publishing
- public skill sharing
- team collaboration
- cloud accounts or sync
- cross-platform abstractions for V1

## Milestones

1. Local workspace shell, package list, and editor frame
2. Filesystem-backed package loading
3. `skill-create` (or Claude CLI) and first eval integration
4. formal version save, diff, and restore
