# JSON Data Structures

## Workspace Directory Structure

```
<project>/.42eval/<skill-name>/
├── config.json              # Classification + config
├── skill-snapshot/          # Current skill snapshot
│   ├── SKILL.md
│   └── references/
├── skill-snapshot-prev/     # Previous snapshot (auto-archived on --force)
├── cases/                   # Eval cases
│   └── case-001.json
└── iterations/
    └── v1/
        ├── eval-1/
        │   ├── eval_metadata.json
        │   ├── with_skill/      # output.md, grading.json, metadata.json, feedback.json
        │   ├── without_skill/   # output.md, grading.json, metadata.json
        │   └── old_skill/       # Optional: previous version run
        ├── benchmark.json
        ├── benchmark.md
        └── report.html
```

## Workspace Config (config.json)

```json
{
  "version": "2.0",
  "skillName": "my-skill",
  "skillPath": "/absolute/path/to/skill",
  "classification": {
    "quadrant": "mastery", "strategy": "comparison",
    "modelCapability": "strong", "humanPractice": "best",
    "modelScore": 72, "practiceScore": 68, "confidence": 0.85,
    "signals": [{ "dimension": "model", "direction": "strong", "keyword": "code generation", "weight": 9 }]
  },
  "currentIteration": 2,
  "createdAt": "...", "updatedAt": "..."
}
```

## Eval Case (cases/case-xxx.json)

```json
{
  "caseId": "case-001",
  "caseName": "Core functionality test",
  "prompt": "Use this skill to complete the following task...",
  "expectations": [
    { "id": "exp-1", "description": "Output contains ## Entity Analysis section", "category": "structure", "critical": true },
    { "id": "exp-2", "description": "Uses three-phase analysis instead of single-pass", "category": "differential", "critical": true },
    { "id": "exp-3", "description": "Each entity includes attributes and relationships", "category": "content", "critical": false }
  ],
  "files": ["src/utils/parser.ts"]
}
```

- `files` (optional): project-relative paths the agent reads before executing. Same files provided to all variants.
- `expectations`: at least 2 `differential` per case. Categories: `structure`, `content`, `differential`, `quality`.

## Eval Metadata (eval_metadata.json)

```json
{
  "evalId": "eval-1",
  "caseId": "case-001",
  "caseName": "Core functionality test",
  "prompt": "...",
  "expectations": [{ "id": "exp-1", "description": "...", "category": "structure", "critical": true }],
  "config": { "skillName": "dev-req", "quadrant": "leverage", "strategy": "delta", "iteration": 1 },
  "createdAt": "..."
}
```

## Grading Result (grading.json)

```json
{
  "expectations": [
    { "expectationId": "exp-1", "description": "...", "passed": true, "evidence": "Found '## Entity Analysis' heading..." },
    { "expectationId": "exp-2", "description": "...", "passed": false, "evidence": "Output provides results without phased analysis" }
  ],
  "passRate": 0.6,
  "passedCount": 3,
  "totalCount": 5,
  "reasoning": "Output performs well on structure but lacks phased methodology.",
  "gradedAt": "..."
}
```

## Run Metadata (metadata.json)

```json
{ "startedAt": "...", "completedAt": "...", "durationMs": 150000, "totalTokens": 84852, "toolCalls": 12, "model": "claude-sonnet-4-6" }
```

## Human Feedback (feedback.json)

Stored per variant (e.g., `eval-1/with_skill/feedback.json`).

```json
{
  "evalId": "eval-1",
  "variant": "with_skill",
  "expectations": [
    { "expectationId": "exp-1", "agree": true },
    { "expectationId": "exp-2", "agree": false, "humanOverride": true, "note": "Grader missed implicit structure" }
  ],
  "overallNote": "Output quality is better than grading suggests",
  "createdAt": "..."
}
```

## Benchmark Result (benchmark.json)

```json
{
  "version": "2.0",
  "skillName": "dev-req",
  "classification": { "...same as config.json..." },
  "iteration": 1,
  "runs": [{
    "evalId": "eval-1", "caseId": "case-001", "caseName": "...",
    "withSkill":    { "expectations": [{"expectationId":"exp-1","passed":true,"evidence":"..."}], "passRate": 0.8, "outputLength": 12500, "metadata": {"durationMs":150000,"totalTokens":84852} },
    "withoutSkill": { "expectations": [{"expectationId":"exp-1","passed":false,"evidence":"..."}], "passRate": 0.4, "outputLength": 8200, "metadata": {"durationMs":90000,"totalTokens":42100} },
    "oldSkill":     { "...same structure, optional..." }
  }],
  "runSummary": {
    "withSkill":    { "meanPassRate": 0.85, "stddev": 0.1, "min": 0.6, "max": 1.0 },
    "withoutSkill": { "meanPassRate": 0.55, "stddev": 0.15, "min": 0.2, "max": 0.8 },
    "oldSkill":     { "...optional..." }
  },
  "delta":        { "meanPassRate": 0.3, "stddev": 0.12, "min": 0.1, "max": 0.5 },
  "versionDelta": { "meanPassRate": 0.2, "...optional, with_skill minus old_skill..." },
  "timing": {
    "withSkill": { "meanDurationMs": 150000, "meanTokens": 84852, "totalRuns": 3 },
    "withoutSkill": { "meanDurationMs": 90000, "meanTokens": 42100, "totalRuns": 3 },
    "delta": { "durationMs": 60000, "tokens": 42752 }
  },
  "expectationHealth": [{
    "expectationId": "exp-2", "description": "...", "category": "differential",
    "status": "skill-differential",
    "withSkillPassCount": 3, "withoutSkillPassCount": 0, "totalRuns": 3,
    "suggestion": "Skill's differentiated value demonstrated. Keep and monitor."
  }],
  "evaluatedAt": "..."
}
```

### Key benchmark fields

- `oldSkill` / `versionDelta`: Optional. Present when `skill-snapshot-prev/` exists. Enables version comparison (positive versionDelta = new version better).
- `timing.delta`: Positive = skill costs more time/tokens.
- `expectationHealth.status`: `skill-differential` | `always-pass` | `always-fail` | `inverse` | `mixed`. See [eval-guide.md](eval-guide.md) for actions.
