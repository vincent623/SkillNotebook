# Quality Checklist

Comprehensive quality standards based on Claude Code official best practices.

## Critical (Must Pass)

These issues block publication. Must be fixed.

### Frontmatter

- [ ] `name` exists and follows rules (lowercase, hyphens, max 64 chars)
- [ ] `name` matches directory name
- [ ] `description` exists and is non-empty
- [ ] `description` is under 1024 characters
- [ ] `description` uses third person (not "I can" or "You can")
- [ ] No XML tags in name or description

### Structure

- [ ] SKILL.md exists in skill directory
- [ ] SKILL.md has valid YAML frontmatter (between `---` markers)
- [ ] All referenced files exist (check links in markdown)

### Security

- [ ] No hardcoded personal paths (`/Users/xxx/`, `/home/xxx/`)
- [ ] No API keys, tokens, or passwords
- [ ] No private keys (`-----BEGIN PRIVATE KEY-----`)
- [ ] No internal network addresses (`192.168.x.x`, `10.x.x.x`)

**Security scan command:**
```bash
grep -rEn '/Users/[^/]+/|/home/[^/]+/|api[_-]?key|token.*=|password.*=|BEGIN.*PRIVATE|AKIA[A-Z0-9]{16}' <skill_path>/
```

## Important (Should Fix)

These issues affect quality. Recommended to fix.

### Discoverability

- [ ] `description` includes what the skill does
- [ ] `description` includes when to use it (trigger conditions)
- [ ] `description` includes relevant keywords users might say
- [ ] `argument-hint` provided if skill accepts arguments

### Conciseness

- [ ] SKILL.md body under 500 lines
- [ ] No explanations of things Claude already knows
- [ ] Each paragraph justifies its token cost
- [ ] Examples are concrete, not abstract

### Structure

- [ ] Uses progressive disclosure (details in separate files)
- [ ] Reference files are one level deep only (no nested references)
- [ ] Long reference files (>100 lines) have table of contents
- [ ] Clear section headers

### Content

- [ ] Has "When to Use" section
- [ ] Has "When NOT to Use" section (if applicable)
- [ ] Consistent terminology throughout
- [ ] No time-sensitive information (or in "old patterns" section)
- [ ] No Windows-style paths (use forward slashes)

### Workflows

- [ ] Complex workflows have clear sequential steps
- [ ] Feedback loops included (validate → fix → retry)
- [ ] Exit criteria defined for each phase

### Scripts (if applicable)

- [ ] Scripts handle errors explicitly (not punt to Claude)
- [ ] No "magic numbers" (all constants documented)
- [ ] Required packages listed
- [ ] Execution instructions clear

## Suggested (Nice to Have)

These enhance quality but aren't required.

### Testing

- [ ] Tested with Claude Haiku (instructions clear enough?)
- [ ] Tested with Claude Sonnet (balanced?)
- [ ] Tested with Claude Opus (not over-explaining?)
- [ ] At least 3 real usage scenarios tested

### Documentation

- [ ] Quick reference table for common operations
- [ ] Common mistakes section
- [ ] Examples with input/output pairs

### Maintainability

- [ ] Version number in metadata
- [ ] Author in metadata
- [ ] Chinese localization (title, description_zh)

## Automated Scoring

Use 42plugin CLI for automated scoring:

```bash
42plugin __score <skill_path> -t skill
```

**Score interpretation:**
| Score   | Level    | Action                         |
| ------- | -------- | ------------------------------ |
| 90-100  | Excellent | Ready to publish              |
| 70-89   | Good     | Minor fixes recommended        |
| 50-69   | Fair     | Significant improvements needed|
| 0-49    | Poor     | Major revision required        |

## Review Output Template

```markdown
## Automated Score
- Total: XX/100
- Level: [Excellent/Good/Fair/Poor]

## Critical Issues (Must Fix)
1. [file:line] Issue description → Fix suggestion

## Important Issues (Should Fix)
1. Issue description → Suggestion

## Security Scan
- [ ] No sensitive information found
- OR: Found issues at [file:line]

## Suggestions
1. Enhancement idea

## Overall Assessment
[Summary and recommendation]
```
