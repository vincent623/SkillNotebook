# .42cog

This directory is the canonical home for Skill Notebook product and engineering specifications.

## Contents

- `PRD.md`: product scope, principles, and milestones.
- `TECH_SPEC.md`: architecture, storage shape, command surface, and implementation guidance.
- `frontend-design-spec.md`: production frontend design rules and interaction model.
- `draft-handoff-spec.md`: temporary draft workspace and external-agent handoff contract.

## Design Artifacts

Runnable prototypes and heavyweight design artifacts stay outside `.42cog` unless they are pure specifications. The current UI prototype lives at:

- `docs/skillnotebook/`

Treat that prototype as a design baseline artifact, and treat `.42cog/frontend-design-spec.md` as the normative implementation reference that summarizes what production should follow.

The prototype is still valid as a UI style and workbench interaction reference. Its old generator flow is deprecated: use `docs/skillnotebook/src/library.jsx` as the primary reference, and use `docs/skillnotebook/src/generator.jsx` only for incidental modal/stepper styling, not product logic.
