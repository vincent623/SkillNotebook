# Skill Notebook Frontend Design Spec

Status: canonical frontend specification
Updated: 2026-05-07

This document is the production frontend design contract for Skill Notebook. It translates the runnable UI prototype in `docs/skillnotebook/` into implementable React/Tauri behavior backed by the Rust core.

## 0. Source Order

When product, design, and implementation disagree, resolve conflicts in this order:

1. `.42cog/PRD.md` defines product scope and non-goals.
2. `.42cog/TECH_SPEC.md` defines architecture and runtime constraints.
3. `.42cog/frontend-design-spec.md` defines production frontend behavior and visual rules.
4. `docs/skillnotebook/` is the runnable UI design baseline.
5. `src/` is the current implementation and may lag behind this spec.

Do not copy the prototype code wholesale into production. Use it as a visual and interaction reference, then implement with typed React components, Zustand stores, and Tauri API calls.

## 1. Product Metaphor

Skill Notebook is a local filesystem workbench for reusable agent skill assets.

The user should feel they are looking at real folders and files, not cards in a dashboard. A skill package is a directory. `SKILL.md`, prompts, examples, references, scripts, tests, evals, and `notebook.json` are the durable local objects behind the interface.

The interface should feel like:

- Finder for locating and drilling into package files.
- A notebook editor for reading and writing skill material.
- A light Git client for deliberate eval-backed formal versions.
- A quick handoff surface for referencing a known-good skill in Claude, Codex, OpenClaw, or a shell.

It should not feel like:

- A public marketplace.
- A SaaS admin dashboard.
- A decorative landing page.
- A prompt gallery.
- An AI generation studio.

## 2. Core Frontend Principles

| Principle | Rule |
| --- | --- |
| File-first UI | The visible structure should follow the real package directory structure. |
| Editing is central | Reading, previewing, editing, and saving files are the daily loop. |
| Versions are deliberate | Saving a formal version must feel more significant than editing a file. |
| Eval is visible | Eval status and suggestions must be visible before version save. |
| Reference is fast | Copying paths, snippets, link commands, and terminal handoff commands should be one or two actions away. |
| Local paths matter | Project root and package paths are first-class UI information. |
| CLI first | Core workflows must be available through `skill` CLI before the GUI treats them as complete. |
| Chinese UI | User-facing copy is Chinese; paths, commands, code identifiers, and file names remain literal. |
| No fake power | Prototype-only abilities must be disabled, marked as planned, or backed by real commands. The GUI must not pretend to generate skills through an unbacked model path. |

## 3. Design Baseline

The current runnable design baseline is:

- `docs/skillnotebook/Skillbook.html`
- `docs/skillnotebook/src/library.jsx`
- `docs/skillnotebook/src/generator.jsx`
- `docs/skillnotebook/src/store.js`

This prototype remains the visual and interaction reference for the workbench. `library.jsx` is the primary reference for layout, density, navigation, editing, eval/version panels, and export/use modal behavior.

`generator.jsx` is no longer a product-flow reference. It may be used only as a loose modal or stepper style reference. Do not preserve its "describe -> generate -> preview -> save" generation logic, AI generation progress, regeneration actions, or copy that implies Skill Notebook owns skill creation.

Production should adopt these baseline ideas:

- Finder-style package/file browsing.
- Search and tag filtering for skills.
- Command palette for quick open and primary actions.
- Rich Markdown reading view with frontmatter summary.
- Edit mode separate from preview mode.
- Draft/import flow for externally created candidate packages.
- Version commit modal rather than casual inline saving.
- Quick reference/use modal for local Claude, Codex, OpenClaw, and shell usage.

Production should not inherit these prototype traits:

- `localStorage` as source of truth.
- Inline styles as the production styling pattern.
- Mock AI generation.
- GUI-owned skill generation.
- Mock local paths.
- Browser-only zip/export behavior if a native implementation is available.

## 4. Information Architecture

### 4.1 Target User Model

The user is inside one active project root.

```txt
project-root/
  .skills/
    skill-package/
      SKILL.md
      prompts/
      examples/
      references/
      scripts/
      tests/
      evals/
      notebook.json
  .skill-notebook/
    config.json
    drafts/
    snapshots/
    logs/
    cache/
  .42eval/
```

The UI should reinforce that every operation happens inside the active project root.

### 4.2 Target App Shape

Production should converge toward a single workbench surface:

```txt
┌────────────────────────────────────────────────────────────────┐
│ Top Bar: brand / project root / search / draft/import / settings│
├───────────────┬───────────────────────────────┬────────────────┤
│ Skill Library │ Package/File Browser Columns  │ Content/Ref Pane│
│ search/filter │ directories and files         │ preview/edit/use │
└───────────────┴───────────────────────────────┴────────────────┘
```

Auxiliary flows open as modals or focused overlays:

- New draft workspace.
- Import candidate package.
- Command palette.
- Save version.
- Version diff.
- Quick reference/use.
- Settings.

The current `ExplorerView`, `NotebookView`, `DraftImportView`, and `SettingsPage` may remain as intermediate migration surfaces, but they should not define the long-term user mental model.

## 5. Workbench Regions

### 5.1 Top Bar

Required elements:

- Brand: `技能本` or `Skill Notebook`; clicking returns focus to the main workbench.
- Active project root path, truncated in the middle when needed.
- Status indicator: loading, ready, error.
- Command palette button with search icon and `⌘K` label.
- Draft/import button: `新建草稿` or `导入`.
- Quick reference button for the selected skill.
- Settings icon button.

Optional elements after core alignment:

- Version save button for selected skill.
- Eval status badge for selected skill.

### 5.2 Skill Library Column

Purpose: browse and filter skill packages under `.skills/`.

Required behavior:

- List packages sorted by `updatedAt` descending.
- Search by slug, name, description, and tags.
- Filter by tags.
- Show status, current version, and latest eval score when available.
- Clicking a skill selects it and loads its file browser.

Visual rules:

- Width around `280px`.
- Background uses sidebar surface.
- Rows look like filesystem entries, not cards.
- Slug uses monospace.
- Description is secondary and clamped to two lines.

### 5.3 Package/File Browser

Purpose: navigate the selected skill package's visible files.

Preferred target:

- Finder-style columns from the prototype.
- Directories create the next column.
- Files open in the content pane.
- `SKILL.md` is always sorted first among files.
- Directories sort before files.

Allowed interim:

- A collapsible file tree, as long as it uses the same file sorting and selection semantics.

Rules:

- Hide internal metadata from ordinary editing: `notebook.json`, hidden files, symlinks.
- Show real package-relative paths.
- File names use monospace.
- Current file has a clear selected state.

### 5.4 Content Pane

Purpose: read and edit package files.

Required states:

- Empty package summary when no file is selected.
- Loading state while reading file content.
- Error state if file cannot be read.
- Preview mode.
- Edit mode.
- Dirty/saving/saved states.

Preview mode:

- Render Markdown for `.md` files.
- Render code/preformatted text for non-Markdown text files.
- `SKILL.md` preview must show a frontmatter summary block when frontmatter is present.
- Show word count or character count in the file title bar.
- Include copy-to-clipboard affordance.

Edit mode:

- Plain text editor, monospace.
- Save is explicit unless autosave is later specified.
- Dirty files must be visible.
- Switching away from a dirty file requires save, discard, or cancel.

### 5.5 Eval And Version Surface

Purpose: make quality and formal history visible without making them feel casual.

Required elements:

- Latest eval status.
- Completeness, clarity, and executability scores.
- Suggestions from the latest eval report.
- Formal version list.
- Run eval action.
- Save version action.
- View diff action.
- Restore version action with confirmation.

Version save:

- Use a modal or focused inline panel.
- Requires an eval report.
- Requires or strongly encourages a version note.
- After save, refresh package bootstrap and version list.

Restore:

- Must warn that package files will be overwritten.
- Should close open editors or reload content after restore.

### 5.6 Quick Reference And Use Surface

Purpose: make the selected skill immediately usable in real local workflows.

Required elements:

- Absolute package path.
- Absolute `SKILL.md` path.
- Copyable Markdown reference snippet.
- Copyable CLI/path reference snippet.
- Command to symlink package into `~/.claude/skills/<slug>`.
- Command to symlink package into `<project>/.claude/skills/<slug>`.
- Open terminal at package root action.
- Export sanitized zip action.

Rules:

- All paths and commands must be derived from real local paths.
- Copy actions should provide clear saved/copied feedback.
- Commands should be shown before execution; shell execution must be explicit.
- Future Codex/OpenClaw link targets may be added when their local skill conventions are known.

## 6. Draft And Import Flow

### 6.1 Target Flow

```txt
Open draft/import
  -> choose draft or import mode
  -> create a temporary draft workspace or select an existing candidate package
  -> hand draft workspace to Claude/Codex/OpenClaw/shell when needed
  -> inspect candidate package files
  -> import to .skills/
  -> run eval/test
  -> optionally save a formal version
  -> open quick reference actions
```

### 6.2 Input Modes

| Mode | V1 Status | Notes |
| --- | --- | --- |
| Existing skill folder | Required | Import an already-created package into `.skills/` after validation. |
| Draft workspace | Required | Create `.skill-notebook/drafts/<draft-id>/` with `BRIEF.md`, skeleton files, and external-agent handoff commands. |
| Local files/directories | Supported | Build a source inventory or draft references, then let the user hand the workspace to an external agent. |
| URL | Supported | Capture bounded source text as draft reference material. |
| Text description | Optional | Creates a `BRIEF.md` and skeleton, not a generated final skill. |

### 6.3 Current Backend Contract

The target production flow must create or import candidate package workspaces first, then commit into `.skills/` only after user confirmation.

Target V1 commands:

- `draft_start`: creates `.skill-notebook/drafts/<draft-id>/`, writes `BRIEF.md`, `draft.json`, and a package skeleton, and returns recommended handoff commands.
- `draft_list`: lists active draft workspaces.
- `draft_discard`: removes an abandoned draft workspace after confirmation.
- `draft_import`: imports a completed draft into `.skills/<slug>/`, writes notebook metadata, and returns next eval/version/reference commands.
- `package_import`: imports an existing local package folder into `.skills/<slug>/`.
- `package_reference`: returns copyable paths, snippets, link commands, terminal commands, and export handles for a selected package.

Legacy GUI-owned create commands are removed. The UI must route new work through draft start, external-agent handoff, draft import, or direct package import.

Target lifecycle behavior:

- Draft workspaces older than a configurable TTL should be flagged or cleaned up.
- Import should validate package shape before writing into `.skills/`.
- Import should not automatically save a formal version unless the eval/version policy explicitly allows it.

## 7. Quick Reference And Use Flow

The design baseline includes an export/use modal. In the new product model, this becomes a first-class quick reference surface. Production V1 should support at least:

- Copy absolute package path.
- Copy absolute `SKILL.md` path.
- Copy Markdown reference snippet.
- Copy CLI/path reference snippet.
- Copy command to symlink package into `~/.claude/skills/<slug>`.
- Copy command to symlink package into `<project>/.claude/skills/<slug>`.
- Open terminal at the package root.
- Export a sanitized package zip through the native `package_export_zip` command.

Later:

- Generate reconstruction shell script.
- Add known local link targets for Codex/OpenClaw when their conventions stabilize.

All commands shown to the user must be derived from real local paths, not mock paths.

## 8. Command Palette

Keyboard shortcut: `⌘K`.

Required search targets:

- Skill slug.
- Skill name.
- Description.
- Tags.

Required actions:

- Open selected skill.
- Open quick reference for selected skill.
- Start a draft workspace.
- Import candidate package.
- Open settings.

Later actions:

- Run eval for selected skill.
- Save version.
- Export selected skill.

## 9. Settings

Settings should be functional and plain.

Required:

- Current project root path.
- Open/switch project root.
- Recent project roots.
- Skill root name: `.skills`.
- Preferred terminal command.
- Preferred editor command.
- Preferred external agent command, such as `codex`, `claude`, `opencode`, or a custom shell command.
- Default local skill link targets where known.

Optional:

- Project root creation.
- Environment variables help.

## 10. Visual System

Use the runnable prototype as the primary visual reference.

### 10.1 Tokens

```css
:root {
  --font-ui: Inter, -apple-system, BlinkMacSystemFont, "PingFang SC", system-ui, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
  --bg: #ffffff;
  --bg-sidebar: #fafaf9;
  --bg-hover: #f4f4f2;
  --bg-active: #eeede9;
  --border: #e7e5e0;
  --border-faint: #efede9;
  --ink: #18181b;
  --ink-muted: #57564e;
  --ink-faint: #9a988f;
  --accent: #b8590a;
  --danger: #b91c1c;
  --warn: #a16207;
  --success: #15803d;
}
```

Production can use bundled/system fonts if external font loading is undesirable, but spacing, contrast, and hierarchy should stay close to the prototype.

### 10.2 Components

- Cards are used for modals, repeated rows only when needed, and focused information panels.
- Do not nest cards inside cards.
- Rows should feel dense and scannable.
- Buttons should use icons for common actions: search, draft/import, edit, copy, save, settings, export, delete.
- Use 5-10px radius for compact controls.
- Avoid large marketing hero sections.

### 10.3 Motion

Use motion sparingly:

- Modal open/close fade.
- Draft/import status pulse.
- Hover state transitions under 150ms.
- Avoid decorative background animation.

## 11. Component Architecture

Target production shape:

```txt
src/
  app/
    App.tsx
    views/
      WorkbenchView.tsx
  components/
    command/
      CommandPalette.tsx
    draft/
      DraftStartModal.tsx
      DraftList.tsx
      DraftImportPanel.tsx
    import/
      ImportPackageModal.tsx
      ImportPreview.tsx
    reference/
      QuickReferenceModal.tsx
      ReferenceCommandList.tsx
    library/
      SkillLibraryColumn.tsx
      SkillRow.tsx
      TagFilter.tsx
    browser/
      FileColumnBrowser.tsx
      FileTree.tsx
      FileRow.tsx
    editor/
      ContentPane.tsx
      MarkdownPreview.tsx
      FrontmatterCard.tsx
      TextEditor.tsx
    version/
      EvalVersionPanel.tsx
      VersionSaveModal.tsx
      VersionDiffModal.tsx
    settings/
      SettingsModal.tsx
    common/
      IconButton.tsx
      StatusBadge.tsx
      ScoreBar.tsx
      InlineBanner.tsx
  stores/
    project-store.ts
    editor-store.ts
    command-store.ts
    ui-store.ts
  services/
    tauri-api.ts
  styles/
    globals.css
```

Interim files may remain, but new implementation work should move toward these ownership boundaries.

## 12. Tauri API Mapping

Target commands to use:

| Frontend Need | Tauri Command |
| --- | --- |
| Bootstrap app data | `app_bootstrap` |
| Open project root | `project_root_open` |
| Create project root | `project_root_create` |
| List recent project roots | `project_root_list_recent` |
| Package file tree | `package_file_tree` |
| Read package file | `package_file_read` |
| Write package file | `package_file_write` |
| Update package metadata | `package_update` |
| Search package | `package_search` |
| Run eval | `package_run_eval` |
| Run package smoke tests | `package_run_test` |
| Save version | `package_save_version` |
| Diff version | `package_diff_version` |
| Restore version | `package_restore_version` |
| Quick reference data | `package_reference` |
| Import package folder | `package_import` |
| Export sanitized zip | `package_export_zip` |
| Start draft workspace | `draft_start` |
| List draft workspaces | `draft_list` |
| Import draft workspace | `draft_import` |
| Discard draft workspace | `draft_discard` |
| Settings | `settings_get` |

Current implementation state:

- Existing generation-preview surfaces and backend commands have been removed from the target contract.
- Draft/import/reference commands are the supported path and replace GUI-owned generation.
- Native sanitized zip export, shell/script-backed package test execution, and clean-editor filesystem refresh remain useful.

## 13. Migration Plan

### Phase 1: Spec And Token Alignment

- Keep `.42cog/frontend-design-spec.md` as the implementation guide.
- Add design artifact links from `.42cog/README.md`.
- Align global CSS tokens with the prototype.
- Add Markdown preview and frontmatter summary.

### Phase 2: Workbench Shell

- Introduce `WorkbenchView`.
- Add skill library search and tag filters.
- Add command palette.
- Keep current file tree if column browser is too large for the first pass.

### Phase 3: File Browser And Editor

- Implement Finder-style column browser or make the existing tree visually match the target.
- Add file dirty-state guards.
- Improve preview/edit title bar and copy action.

### Phase 4: Reference And Use

- Promote quick reference/use to a primary selected-skill action.
- Add copyable path, Markdown reference, CLI reference, symlink, terminal, and export actions.
- Keep commands visible and derived from real local paths.

### Phase 5: Draft And Import

- Replace GUI-owned generation with draft workspace bootstrap.
- Add import package and import draft flows.
- Add external-agent handoff commands and terminal open actions.

### Phase 6: Eval And Version

- Move eval/version actions into a deliberate quality gate panel.
- Add version save modal with eval snapshot and required version note.
- Improve diff and restore UI with dedicated modal flows.

## 14. Acceptance Checklist

Before considering frontend alignment complete:

- App opens into a workbench that visually resembles `docs/skillnotebook`.
- User can search/filter skills without leaving the workbench.
- User can open `SKILL.md`, preview Markdown, edit text, save, and see saved feedback.
- User can run eval and understand scores/suggestions.
- User can save a formal version only after eval.
- User can view diff and restore a version with confirmation.
- User can copy local usage paths, reference snippets, symlink commands, terminal commands, or export a sanitized zip.
- User can start a draft workspace and see the recommended external-agent command.
- User can import an existing candidate package or completed draft into `.skills/`.
- No UI exposes mock-only capabilities as real.
- `npm run build` passes.

## 15. Explicit Non-Goals For V1

- Public marketplace.
- GUI-owned AI skill generation.
- Model provider/runtime configuration as a core product requirement.
- Cloud sync.
- Team collaboration.
- Account system.
- Multi-language UI.
- WYSIWYG Markdown editing.
- Multi-tab editor.
- Drag-and-drop file restructuring.
- Real-time collaborative editing.
- Decorative landing page.
