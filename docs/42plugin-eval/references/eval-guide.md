# Eval Case Writing Guide

## Principles

1. **Each case tests one clear capability** — don't test multiple features in a single case
2. **Input must be specific and actionable** — vague instructions lead to non-reproducible results
3. **expectedBehavior must be decidable** — the grader needs to clearly judge pass/fail
4. **Expectations must be differentiated** — focus on the skill's unique contribution

## Expectation Categories

| category | Focus | Key rule |
|----------|-------|----------|
| `structure` | Prescribed headings, tables, code blocks | Verify skill-defined output structure |
| `content` | Specific substantive content | Verify concrete analysis, not generic statements |
| `differential` | Skill's unique contribution | **At least 2 per case** — core of measuring skill value |
| `quality` | Overall output quality | Actionable recommendations, correct code |

## Writing Cases by Quadrant

Each quadrant emphasizes different expectation categories:

| Quadrant | Emphasis | Example focus |
|----------|----------|---------------|
| scaffolding | structure + differential | Template compliance, skill-defined sections |
| leverage | **differential** | With/without skill difference, guided depth |
| codification | content + differential | Domain knowledge accuracy, standard references |
| mastery | balanced | Overall quality across all dimensions |

### Example: leverage quadrant case

```json
{
  "id": "case-001",
  "name": "Code review depth comparison",
  "input": "Review the following Python function for security:\n\ndef login(username, password):\n    query = f\"SELECT * FROM users WHERE name='{username}' AND pass='{password}'\"\n    return db.execute(query)",
  "expectedBehavior": "With skill, should identify SQL injection, plaintext password storage, and deeper issues",
  "expectations": [
    { "id": "exp-1", "description": "Identifies SQL injection vulnerability", "category": "content", "critical": true },
    { "id": "exp-2", "description": "Uses the skill-defined security review framework", "category": "differential", "critical": true },
    { "id": "exp-3", "description": "Provides specific fix code", "category": "quality", "critical": false },
    { "id": "exp-4", "description": "Checks all security dimensions in the skill checklist", "category": "differential", "critical": false }
  ]
}
```

**Bad case:** `"input": "Write a hello world function"` — too simple, differential expectations cannot distinguish.

## Input Files

For skills that operate on existing code, add `files` to the case:

```json
{ "id": "case-001", "input": "Review this code for security issues", "files": ["src/auth/login.ts"] }
```

- Use project-relative paths, keep file count small (1-3)
- Same files are provided to all variants

## Hybrid Calibration Workflow

For first-time expectations:

1. Write draft expectations (use assertion-gen agent)
2. Run with_skill only (skip without_skill in first pass)
3. Review grading — identify ambiguous or always-pass expectations
4. Refine — adjust wording, remove bad ones
5. Full run — both variants with calibrated expectations

## Expectation Health (Post-Benchmark)

| Status | Meaning | Action |
|--------|---------|--------|
| `skill-differential` | Passes with skill, fails without | Keep — proves skill value |
| `always-pass` | Passes in both variants | Remove or strengthen |
| `always-fail` | Fails in both variants | Fix assertion or adjust skill |
| `inverse` | Passes without, fails with | Investigate — skill may hurt |
| `mixed` | Inconsistent | Gather more data or clarify |

**Key rule**: After every benchmark, prune `always-pass` and fix `always-fail` before next iteration.

## Recommended Case Count

| Purpose | Count |
|---------|:-----:|
| Quick validation | 3-5 |
| Standard evaluation | 8-12 |
| Comprehensive | 15-20 |

## Naming Conventions

- Case ID: `case-001` (three-digit zero-padded), file: `case-001.json`
- Expectation ID: `exp-1`, `exp-2` (incremental within each case)

## Common Mistakes

1. **Vague input** — must be a specific task description
2. **Missing differential expectations** — at least 2 per case
3. **All expectations marked critical** — only core value points
4. **Non-judgable descriptions** — must be observable facts
5. **Ignoring health analysis** — always review after benchmarking
6. **Missing edge cases** — test empty input, special characters, very long input
