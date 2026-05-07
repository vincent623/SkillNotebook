# Draft Handoff Spec

## Purpose

Skill Notebook is CLI-first, but it does not own skill generation.

The GUI and CLI help users create a well-shaped temporary workspace, hand that workspace to Claude, Codex, OpenClaw, or another local agent, then import the result into the managed `.skills/` asset library after inspection and evaluation.

This replaces the older generator-runtime direction. Skill Notebook should not require model provider configuration, OpenAI-compatible API keys, or bundled model sidecars for the core product loop.

## Handoff Model

Skill creation usually happens in a real work context:

- an agent conversation that revealed a repeatable workflow
- a project folder with scripts and references
- a set of prompts, examples, notes, and test expectations
- a local shell session where the user is already working

Skill Notebook turns that context into a draft workspace. An external agent then edits files inside that workspace. When the user is satisfied, Skill Notebook imports the draft into `.skills/`, runs eval/test, and saves a formal version when appropriate.

## Draft Lifecycle

```txt
Start draft
  -> create .skill-notebook/drafts/<draft-id>/
  -> write BRIEF.md, draft.json, and package skeleton
  -> show/open suggested external-agent command
  -> external agent edits the draft workspace
  -> user imports the completed draft
  -> Skill Notebook runs eval/test
  -> user saves a formal version
```

## Draft Workspace Shape

```txt
.skill-notebook/drafts/
  draft-<slug>-<id>/
    BRIEF.md
    draft.json
    SKILL.md
    prompts/
    examples/
    references/
    scripts/
    tests/
    evals/
```

`SKILL.md` may be a scaffold. It does not need to be high quality at draft creation time.

## BRIEF.md Contract

`BRIEF.md` is written for the external agent and the user.

It should include:

- goal
- intended skill name and slug
- target user and use case
- expected inputs
- expected outputs
- when to use the skill
- when not to use the skill
- source paths or source excerpts
- package shape requirements
- acceptance checks
- next commands

Example command block:

```bash
cd <draft-path>
codex
```

or:

```bash
cd <draft-path>
claude
```

## draft.json Contract

`draft.json` is written for Skill Notebook.

Required fields:

- `draftId`
- `projectRootId`
- `draftPath`
- `intendedSlug`
- `createdAt`
- `sourceKind`
- `sourceSummary`
- `suggestedCommand`
- `importCommand`

Optional fields:

- `sourcePaths`
- `sourceUrl`
- `preferredAgentCommand`
- `notes`

## CLI Commands

### `skill draft start`

Creates a draft workspace and prints the suggested handoff command.

```bash
skill draft start "Turn meeting notes into owner/date/risk/action items"
```

Output handles:

- `draftId`
- `draftPath`
- `briefPath`
- `suggestedCommand`
- `importCommand`

### `skill draft import`

Imports a completed draft workspace into `.skills/`.

```bash
skill draft import .skill-notebook/drafts/draft-meeting-actions-1234
```

The import step should:

- validate package shape
- choose or confirm target slug
- copy package files into `.skills/<slug>/`
- write `notebook.json`
- run eval by default when safe
- return next commands for eval/version/reference

### `skill draft discard`

Removes an abandoned draft workspace after user confirmation.

## GUI Behavior

The GUI may expose a lightweight `New Draft` action, but it must not present itself as the place where skills are generated.

The GUI should show:

- draft path
- brief path
- suggested `cd ... && codex` or `cd ... && claude` command
- open in terminal action
- import completed draft action
- discard draft action

The GUI should not show:

- model provider forms
- API key fields for generation
- streaming generation progress
- fake template output as if it were a real generated skill

## Runtime Preferences

Settings may store local handoff preferences:

- preferred terminal command
- preferred editor command
- preferred external agent command
- default skills link targets

Environment variables may override these preferences for CLI sessions.

## Import Boundary

Draft files are not formal package assets until imported.

Importing a draft should not automatically save a formal version unless the eval policy explicitly allows it. The normal path is:

```txt
Import -> Inspect/Edit -> Eval/Test -> Save Formal Version
```

This keeps temporary creation work out of formal version history.
