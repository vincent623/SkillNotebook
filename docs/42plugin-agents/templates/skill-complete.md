# Complete Skill Template

Full template with all available frontmatter fields. Remove unused fields before use.

---

## SKILL.md Template (All Fields)

```yaml
---
# === Required Fields ===
name: ${SKILL_NAME}                    # lowercase, hyphens, max 64 chars
description: |                          # what it does + when to use (third person)
  ${WHAT_IT_DOES}.
  Use when ${TRIGGER_CONDITIONS}.

# === Optional: Invocation Control ===
argument-hint: "${ARGUMENT_HINT}"       # shown in autocomplete, e.g., "[filename]"
disable-model-invocation: false         # true = manual /invoke only
user-invocable: true                    # false = hide from /menu

# === Optional: Execution Environment ===
allowed-tools: Read, Grep, Glob, Bash   # restrict available tools
model: sonnet                           # opus, sonnet, haiku, or specific model ID
context: fork                           # fork = run in isolated subagent
agent: Explore                          # with context:fork - Explore, Plan, general-purpose

# === Optional: Lifecycle Hooks ===
hooks:
  PreToolUse:
    - matcher: 'Bash'
      hooks:
        - type: command
          command: './scripts/validate.sh'
  PostToolUse:
    - matcher: 'Edit|Write'
      hooks:
        - type: command
          command: './scripts/format.sh'
  Stop:
    - hooks:
        - type: command
          command: './scripts/cleanup.sh'

# === 42plugin Extended Fields ===
metadata:
  author: ${AUTHOR}
  version: 1.0.0
  title: ${TITLE_ZH}
  description_zh: ${DESCRIPTION_ZH}
---

# ${SKILL_TITLE}

${CORE_PRINCIPLE_IN_ONE_SENTENCE}

## When to Use

- ${TRIGGER_1}
- ${TRIGGER_2}
- ${TRIGGER_3}

**Don't use for:**
- ${ANTI_TRIGGER_1}
- ${ANTI_TRIGGER_2}

## Quick Reference

| Task | Command/Action |
| ---- | -------------- |
| ${TASK_1} | ${ACTION_1} |
| ${TASK_2} | ${ACTION_2} |

## Usage

### Basic Usage

\`\`\`bash
/${SKILL_NAME} ${EXAMPLE_ARGS}
\`\`\`

### With Options

\`\`\`bash
/${SKILL_NAME} ${EXAMPLE_WITH_OPTIONS}
\`\`\`

## Workflow

### Phase 1: ${PHASE_1_NAME}

${PHASE_1_DESCRIPTION}

**Actions:**
1. ${STEP_1}
2. ${STEP_2}

**Exit criteria:** ${EXIT_CRITERIA_1}

### Phase 2: ${PHASE_2_NAME}

${PHASE_2_DESCRIPTION}

**Actions:**
1. ${STEP_1}
2. ${STEP_2}

**Exit criteria:** ${EXIT_CRITERIA_2}

## Common Mistakes

| Mistake | Fix |
| ------- | --- |
| ${MISTAKE_1} | ${FIX_1} |
| ${MISTAKE_2} | ${FIX_2} |

## Resources

| Type | Path | Description |
| ---- | ---- | ----------- |
| Reference | [reference.md](reference.md) | ${REF_DESC} |
| Script | [scripts/helper.py](scripts/helper.py) | ${SCRIPT_DESC} |
```

---

## Field Decision Guide

### When to Use Each Optional Field

| Field | Use When | Example |
| ----- | -------- | ------- |
| `argument-hint` | Skill expects input | `"[file] [format]"` |
| `disable-model-invocation: true` | Has side effects (deploy, publish, delete) | Deploy skill |
| `user-invocable: false` | Background context, not a command | Style guide knowledge |
| `allowed-tools` | Should restrict capabilities | Read-only: `Read, Grep, Glob` |
| `model: haiku` | Fast, simple tasks | Quick lookups |
| `model: opus` | Complex reasoning | Architecture review |
| `context: fork` | Large output, needs isolation | Research tasks |
| `agent: Explore` | Read-only exploration | Codebase analysis |
| `agent: Plan` | Planning without editing | Design review |
| `hooks` | Need automation | Auto-format after edit |

### Minimal vs Full Frontmatter

**Minimal (most skills):**
```yaml
---
name: my-skill
description: Does X. Use when Y.
metadata:
  author: username
  version: 1.0.0
---
```

**With Invocation Control:**
```yaml
---
name: deploy-app
description: Deploys application to production.
disable-model-invocation: true    # Only manual trigger
argument-hint: "[environment]"
metadata:
  author: username
  version: 1.0.0
---
```

**Read-Only Exploration:**
```yaml
---
name: codebase-analyzer
description: Analyzes codebase structure and patterns.
allowed-tools: Read, Grep, Glob
context: fork
agent: Explore
metadata:
  author: username
  version: 1.0.0
---
```

**With Hooks:**
```yaml
---
name: code-formatter
description: Formats code according to project standards.
hooks:
  PostToolUse:
    - matcher: 'Edit|Write'
      hooks:
        - type: command
          command: 'prettier --write "$FILE_PATH"'
metadata:
  author: username
  version: 1.0.0
---
```

---

## Directory Structure Options

### Simple Skill (Single File)
```
skill-name/
└── SKILL.md
```

### With References
```
skill-name/
├── SKILL.md
└── reference/
    ├── api-guide.md
    └── examples.md
```

### With Scripts
```
skill-name/
├── SKILL.md
├── scripts/
│   ├── helper.py
│   └── validate.sh
└── reference/
    └── guide.md
```

### Full Structure
```
skill-name/
├── SKILL.md
├── LICENSE
├── reference/
│   ├── api-guide.md
│   └── examples.md
├── scripts/
│   ├── helper.py
│   └── validate.sh
└── assets/
    └── template.md
```

---

## Checklist Before Use

- [ ] Replace all `${VARIABLE}` placeholders
- [ ] Remove unused optional fields from frontmatter
- [ ] Remove unused sections from body
- [ ] Verify file references exist
- [ ] Run `42plugin __score` validation
- [ ] Test with at least one real scenario
