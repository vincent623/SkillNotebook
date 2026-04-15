# Frontmatter Guide

Complete reference for SKILL.md frontmatter fields based on Claude Code official documentation.

## Required Fields

### name

Skill identifier used for invocation and directory matching.

```yaml
name: my-skill-name
```

**Rules:**
- Maximum 64 characters
- Lowercase letters, numbers, and hyphens only
- Cannot contain XML tags
- Cannot contain reserved words: "anthropic", "claude"
- Should match the directory name

**Good examples:**
- `pdf-processing`
- `code-review`
- `git-commit-helper`

**Bad examples:**
- `MySkill` (uppercase)
- `my_skill` (underscores)
- `claude-helper` (reserved word)

### description

Describes what the skill does and when to use it. Claude uses this to decide when to load the skill.

```yaml
description: |
  Extracts text and tables from PDF files, fills forms, and merges documents.
  Use when working with PDF files or when the user mentions PDFs, forms, or document extraction.
```

**Rules:**
- Maximum 1024 characters
- Must be non-empty
- Cannot contain XML tags
- **Must use third person** (not "I can help" or "You can use this")
- Should include both:
  - What the skill does
  - When/triggers for using it

**Good examples:**
```yaml
# PDF Processing
description: Extracts text and tables from PDF files. Use when working with PDFs or document extraction.

# Git Commit
description: Generates descriptive commit messages by analyzing git diffs. Use when asking for help with commit messages.

# Code Review
description: Reviews code for quality, security, and best practices. Use after writing or modifying code.
```

**Bad examples:**
```yaml
# Too vague
description: Helps with documents

# First person
description: I can help you process PDF files

# Second person
description: You can use this to process PDFs

# Missing trigger
description: Processes PDF files  # When should it be used?
```

## Optional Fields

### argument-hint

Hint shown during autocomplete to indicate expected arguments.

```yaml
argument-hint: "[filename] [format]"
```

### disable-model-invocation

Prevents Claude from automatically loading this skill. Use for workflows you want to trigger manually with `/skill-name`.

```yaml
disable-model-invocation: true  # Default: false
```

**Use when:**
- Skill has side effects (deploy, publish)
- You want to control timing precisely
- Skill is expensive or slow

### user-invocable

Controls whether skill appears in the `/` menu.

```yaml
user-invocable: false  # Default: true
```

**Use `false` when:**
- Skill is background knowledge, not a command
- Claude should use it automatically, but users shouldn't invoke directly

### allowed-tools

Restricts which tools Claude can use when skill is active.

```yaml
allowed-tools: Read, Grep, Glob, Bash
```

Or as YAML list:
```yaml
allowed-tools:
  - Read
  - Grep
  - Glob
```

### model

Specifies which model to use when skill is active.

```yaml
model: claude-sonnet-4-20250514
```

### context

Set to `fork` to run in an isolated subagent context.

```yaml
context: fork
```

**Use when:**
- Skill needs isolation from main conversation
- Task produces large output that shouldn't pollute main context

### agent

Used with `context: fork` to specify which subagent type to use.

```yaml
context: fork
agent: Explore
```

Available built-in agents: `Explore`, `Plan`, `general-purpose`

### hooks

Lifecycle hooks scoped to this skill.

```yaml
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
```

## 42plugin Extended Fields

### metadata

Extended properties for 42plugin platform.

```yaml
metadata:
  author: username
  version: 1.0.0
  title: 中文标题
  description_zh: 中文描述
```

| Field           | Description                    |
| --------------- | ------------------------------ |
| `author`        | Author name or username        |
| `version`       | Semantic version (e.g., 1.0.0) |
| `title`         | Display title (supports CJK)   |
| `description_zh`| Chinese description            |

## Complete Example

```yaml
---
name: code-reviewer
description: |
  Reviews code for quality, security, and maintainability. Analyzes git diffs
  and provides actionable feedback. Use immediately after writing or modifying
  code, or when asking for a code review.
argument-hint: "[file or directory]"
disable-model-invocation: false
allowed-tools: Read, Grep, Glob, Bash
metadata:
  author: 42ailab
  version: 1.0.0
  title: 代码审查
  description_zh: 审查代码质量、安全性和可维护性。分析 git diff 并提供可操作的反馈。
---
```

## String Substitutions

Available in skill content (markdown body):

| Variable               | Description                          |
| ---------------------- | ------------------------------------ |
| `$ARGUMENTS`           | Arguments passed when invoking skill |
| `${CLAUDE_SESSION_ID}` | Current session ID                   |

Example:
```markdown
Analyze the code in: $ARGUMENTS

Session: ${CLAUDE_SESSION_ID}
```
