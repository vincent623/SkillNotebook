# Skill Examples with Annotations

This document provides annotated examples of well-designed skills.

## Example 1: PDF Processing Skill

**Why it works:**
- Clear trigger condition in description
- Task-based structure matching the operations
- Specific tool references with commands
- Common mistakes section prevents errors

```yaml
---
name: pdf-processing
description: "Use when manipulating PDF files - provides tools for merging, splitting, extracting text, and filling forms"
---
```

```markdown
# PDF Processing

## Overview

This skill provides tools for common PDF operations without requiring external software.

## When to Use

- Merging multiple PDFs into one
- Splitting a PDF into separate pages
- Extracting text or tables from PDFs
- Filling form fields in PDFs

**Don't use for:**
- Creating PDFs from scratch (use document creation tools)
- Editing PDF content directly (PDFs are essentially images)

## Quick Reference

| Task | Command |
|------|---------|
| Merge PDFs | `scripts/merge_pdf.py file1.pdf file2.pdf -o output.pdf` |
| Split PDF | `scripts/split_pdf.py input.pdf -p 1-3,5,7-10` |
| Extract text | `scripts/extract_text.py input.pdf` |

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Trying to edit PDF text | PDFs are not editable - recreate the document |
| Merging encrypted PDFs | Decrypt first or ask user for password |
```

## Example 2: API Integration Skill

**Why it works:**
- Workflow-based for sequential process
- Includes authentication setup
- Error handling guidance
- Response format options

```yaml
---
name: github-api-integrating
description: "Use when interacting with GitHub API - handles authentication, rate limiting, and provides helpers for common operations like issues, PRs, and repositories"
---
```

```markdown
# GitHub API Integration

## Overview

Simplifies GitHub API interactions with built-in rate limiting, authentication, and pagination handling.

## When to Use

- Querying issues, PRs, or repositories
- Creating or updating GitHub resources
- Automating GitHub workflows

## Quick Reference

| Operation | Endpoint | Auth Required |
|-----------|----------|---------------|
| List repos | GET /users/{user}/repos | No |
| Create issue | POST /repos/{owner}/{repo}/issues | Yes |
| Get PR | GET /repos/{owner}/{repo}/pulls/{number} | No |

## Workflow

### Step 1: Authentication

Set token in environment:
```bash
export GITHUB_TOKEN=ghp_xxxx
```

### Step 2: Make Requests

Use the helper function:
```python
from scripts.github_api import github_request

# GET request
repos = github_request('GET', '/users/octocat/repos')

# POST request with data
issue = github_request('POST', '/repos/owner/repo/issues', {
    'title': 'Bug report',
    'body': 'Description here'
})
```

### Step 3: Handle Pagination

For large result sets:
```python
from scripts.github_api import github_paginate

all_issues = github_paginate('/repos/owner/repo/issues')
```

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Rate limit exceeded | Use `scripts/github_api.py` which handles rate limiting |
| 404 on private repo | Ensure GITHUB_TOKEN has correct scopes |
| Missing pagination | Use `github_paginate()` for list endpoints |
```

## Example 3: Brand Guidelines Skill

**Why it works:**
- Reference-based for standards
- Concrete color codes and values
- Visual examples
- Asset references

```yaml
---
name: brand-styling
description: "Use when applying company brand to artifacts - provides official colors, typography, and logo usage guidelines"
---
```

```markdown
# Brand Styling

## Overview

Apply consistent brand identity across all artifacts using official guidelines.

## When to Use

- Creating presentations or documents
- Designing UI components
- Generating marketing materials

## Brand Colors

| Name | Hex | Usage |
|------|-----|-------|
| Primary Blue | #2563EB | Headers, CTAs |
| Secondary Gray | #6B7280 | Body text |
| Accent Orange | #F97316 | Highlights |
| Background | #F9FAFB | Page background |

## Typography

| Element | Font | Size | Weight |
|---------|------|------|--------|
| H1 | Inter | 32px | 700 |
| H2 | Inter | 24px | 600 |
| Body | Inter | 16px | 400 |
| Code | Fira Code | 14px | 400 |

## Logo Usage

**Do:**
- Use `assets/logo-primary.svg` on light backgrounds
- Use `assets/logo-white.svg` on dark backgrounds
- Maintain minimum clear space of 24px

**Don't:**
- Stretch or distort logo
- Change logo colors
- Place logo on busy backgrounds

## Resources

### Assets
- `assets/logo-primary.svg` - Full color logo
- `assets/logo-white.svg` - White version
- `assets/brand-colors.json` - Color palette in code format
```

## Anti-Patterns to Avoid

### 1. Vague Description
```yaml
# BAD
description: "For testing"

# GOOD
description: "Use when tests are flaky or have race conditions - replaces arbitrary timeouts with condition polling"
```

### 2. First-Person Voice
```yaml
# BAD
description: "I help you create documents"

# GOOD
description: "Use when creating structured documents - provides templates and formatting guidelines"
```

### 3. Narrative Style
```markdown
# BAD
In our experience working with the API, we found that you should always...

# GOOD
## Best Practices
- Always include error handling for API calls
- Use pagination for list endpoints
```

### 4. Multi-Language Duplication
```markdown
# BAD - Same example in 5 languages
## Python
[code]
## JavaScript
[code]
## Ruby
[code]
## Go
[code]
## Rust
[code]

# GOOD - One excellent example
## Implementation
```python
# Well-commented Python example
# Ready to adapt to other languages
```
```

### 5. Over-Documenting Basics
```markdown
# BAD
## What is a PDF?
A PDF (Portable Document Format) is a file format developed by Adobe...

# GOOD
## PDF Operations
[Jump straight to useful operations - Claude knows what PDFs are]
```

## Checklist for New Skills

Before publishing, verify:

- [ ] Description starts with "Use when..."
- [ ] Description is third-person
- [ ] Name matches directory name
- [ ] Name uses gerund form
- [ ] Has "When to Use" section
- [ ] Has "Quick Reference" table
- [ ] Has "Common Mistakes" section
- [ ] Under 500 lines
- [ ] No [TODO] placeholders
- [ ] One excellent example per technique
- [ ] No time-sensitive information
