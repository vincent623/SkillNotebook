# Claude Skill Specification

## Overview

A skill is a folder of instructions, scripts, and resources that Claude loads dynamically to perform better at specific tasks. The folder must contain a `SKILL.md` file to be recognized as a skill.

## Folder Structure

### Minimal Structure
```
skill-name/
└── SKILL.md
```

### Full Structure
```
skill-name/
├── SKILL.md          # Required: Entry point
├── scripts/          # Optional: Executable code
│   ├── helper.py
│   └── process.sh
├── references/       # Optional: Reference documentation
│   ├── api_docs.md
│   └── schema.md
└── assets/           # Optional: Output files
    ├── template.pptx
    └── logo.png
```

## SKILL.md Format

### YAML Frontmatter (Required)

```yaml
---
name: skill-name
description: "Use when [trigger] - [what it does]"
---
```

**Required Properties:**
- `name`: Hyphen-case identifier
  - Lowercase Unicode alphanumeric + hyphen only
  - Must match directory name
  - Max 64 characters
- `description`: What the skill does and when to use it
  - Max 1024 characters
  - Should start with "Use when..."
  - Third-person voice

**Optional Properties:**
- `license`: License applied to the skill
- `allowed-tools`: List of pre-approved tools (Claude Code only)
- `metadata`: Map of custom string key-value pairs

### Markdown Body

No restrictions on format. Recommended sections:
- Overview
- When to Use
- Quick Reference
- Main content (workflow/tasks/guidelines)
- Common Mistakes
- Resources

## Resource Types

### scripts/
Executable code for deterministic operations.

**When to use:**
- Same code rewritten repeatedly
- Deterministic reliability needed
- Complex file operations

**Benefits:**
- Token efficient (can execute without loading)
- Deterministic results
- Can be patched by Claude if needed

### references/
Documentation loaded into context as needed.

**When to use:**
- API documentation
- Database schemas
- Domain knowledge
- Detailed workflow guides
- Content > 100 lines

**Best practices:**
- Keep SKILL.md lean, move details here
- Include grep patterns for large files (>10k words)
- No duplication with SKILL.md

### assets/
Files used in output (not loaded into context).

**When to use:**
- Templates (pptx, docx)
- Images and icons
- Fonts
- Boilerplate code
- Sample data

## Progressive Disclosure

Skills use three-level loading:

1. **Level 1 - Metadata** (~100 words)
   - Always in context
   - name + description only
   - Determines when skill triggers

2. **Level 2 - SKILL.md Body** (<5k words)
   - Loaded when skill triggers
   - Main instructions and workflow

3. **Level 3 - Bundled Resources** (unlimited)
   - Loaded as needed by Claude
   - Scripts can execute without reading

## Naming Conventions

### Skill Names
- Use gerund form: `creating-skills`, not `skill-creation`
- Lowercase with hyphens: `data-analyzing`
- Descriptive: `pdf-processing` not `helper`
- Avoid reserved words: anthropic, claude

### Description Format
```yaml
# Good
description: "Use when tests have race conditions - replaces timeouts with polling"

# Bad - too vague
description: "For testing"

# Bad - first person
description: "I help with flaky tests"
```

## Quality Guidelines

### Content
- One excellent example beats many mediocre ones
- Challenge each sentence: does Claude need this?
- Default assumption: Claude is smart
- Only add information Claude doesn't already have

### Structure
- Keep SKILL.md under 500 lines
- Move reference content to references/
- Use tables for quick lookups
- Include "When to Use" and "When NOT to use"

### Maintenance
- Keep content evergreen (no dates)
- Use consistent terminology
- Test with Claude Haiku, Sonnet, and Opus

## Version History

- 1.0 (2025-10-16): Public Launch
