# Dev Loop Queue

## Status

- [x] Implement formal version save so the `Find -> Create -> Evaluate -> Version` loop is complete.
- [x] Implement restore + diff flow for formal versions, including snapshot handling.
- [x] Repair project-root test drift so Rust tests pass against the current `examples/project-root` fixture.
- [x] Frontend alignment Phase 1: align visual tokens, Markdown preview, frontmatter summary, copy affordance, word count, and editor save feedback with `.42cog/frontend-design-spec.md`.
- [x] Frontend alignment Phase 2A: introduce a workbench shell with skill search/filtering and command palette.
- [x] Frontend alignment Phase 2B: replace the interim file tree with Finder-style package/file columns.
- [x] Frontend alignment Phase 3: add export/use modal for local Claude skill usage.
- [x] Frontend alignment Phase 4: evolve create flow toward preview-before-save when backend support exists.
- [x] Replace `package_run_test` scaffold with local smoke test execution and quality gate UI.
- [x] Add preview TTL cleanup for orphaned preview workspaces.
- [x] Add file/directory-based package generation.
- [x] Replace `package_update` scaffold with real package metadata updates.
- [x] Remove unused placeholder SQLite/repository/watcher/shell modules from V1 code.
- [x] Decide SQLite is optional future architecture, not checked-in V1 placeholder code.
- [x] Add URL-based package generation.
- [x] Add shell/script-backed package test execution behind an explicit safety boundary.
- [x] Add filesystem watch refresh.

## Completed In This Round

- [x] Implement formal version save end-to-end (Rust command + snapshot copy + UI wiring).
- [x] Reconcile the product spec and implementation around `skill-create` versus `Claude CLI` draft generation.
- [x] Make project root a first-class runtime context so bootstrap, create, eval, search, and recent/open flows use the current project root instead of a fixed default sample.
- [x] Establish `.42cog/` as the canonical spec home and retain `docs/skillnotebook/` as the runnable UI design baseline.
- [x] Fix stale `examples/project_root` test fixture references and normalize the sample project root id to `project-root-main`.
- [x] Ship frontend alignment Phase 1 for the editor surface.
- [x] Ship frontend alignment Phase 2A for the main workbench surface.
- [x] Ship frontend alignment Phase 2B for Finder-style file navigation.
- [x] Ship frontend alignment Phase 3 for export/use commands.
- [x] Ship frontend alignment Phase 4 for preview-before-save creation.
- [x] Harden preview lifecycle with discard and failed-commit cleanup.
- [x] Ship local smoke test execution through `package_run_test`.
- [x] Ship preview TTL cleanup for crash/force-quit orphan workspaces.
- [x] Ship file/directory-based package generation with source inventory traceability.
- [x] Ship real `package_update` metadata persistence and clean unused placeholder modules.
- [x] Close spec gaps for project-root/settings alignment, dirty editor switching, library eval/status density, metadata editing, URL create, script-backed tests, native sanitized export, and clean-editor filesystem refresh.
