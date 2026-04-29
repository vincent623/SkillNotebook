// Data model: a skill is a directory tree.
// node: { type: 'file'|'dir', name, children?, content? }
// skill: { id, name, description, version, updatedAt, createdAt, tags, tree }

window.SEED_SKILLS = [
  {
    id: "brand-guidelines",
    name: "brand-guidelines",
    displayName: "Brand Guidelines",
    description: "Apply Acme Corp brand guidelines to all presentations and documents",
    version: "1.2.0",
    createdAt: "2026-03-14T09:12:00Z",
    updatedAt: "2026-04-18T14:30:00Z",
    tags: ["brand", "design", "docs"],
    tree: {
      type: "dir", name: "brand-guidelines", children: [
        {
          type: "file", name: "SKILL.md",
          content: `---
name: brand-guidelines
description: Apply Acme Corp brand guidelines to all presentations and documents. Use when creating client-facing materials, slide decks, marketing copy, or any external document.
---

# Brand Guidelines

This skill provides Acme Corp's official brand guidelines for creating consistent, professional materials.

## When to apply

Whenever Claude creates presentations, documents, or marketing materials, apply these standards to ensure all outputs match Acme's visual identity.

## Core principles

- **Warmth over formality** — Acme speaks like a trusted colleague, not a corporation.
- **Clarity over cleverness** — Every sentence earns its place.
- **Evidence over assertion** — Back claims with data or user quotes.

## Visual tokens

See \`references/colors.md\` for the full color palette and usage rules.
See \`references/typography.md\` for typographic scale and pairing.
See \`templates/slide-cover.html\` for the default slide cover.

## Voice checklist

Before shipping any external copy, run through \`scripts/voice-check.sh\` which lints for:
- Jargon flagged in the banned-words list
- Passive voice over 20%
- Sentences longer than 30 words
`
        },
        {
          type: "dir", name: "references", children: [
            {
              type: "file", name: "colors.md",
              content: `# Color palette

## Primary
- **Ink** \`#18181b\` — body text, headlines
- **Paper** \`#fffdf8\` — backgrounds
- **Ember** \`#b8590a\` — accents, links, CTAs

## Usage rules
- Never place Ember on Paper backgrounds below 4.5:1 contrast.
- Reserve Ember for ≤10% of any composition.
`
            },
            {
              type: "file", name: "typography.md",
              content: `# Typography

## Type pairing
- **Display** — Fraunces (serif, weight 600)
- **Body** — Inter (weight 400/500)
- **Mono** — JetBrains Mono (weight 400)

## Scale
- h1: 42px / 1.1
- h2: 28px / 1.2
- body: 16px / 1.6
- caption: 13px / 1.4
`
            }
          ]
        },
        {
          type: "dir", name: "scripts", children: [
            {
              type: "file", name: "voice-check.sh",
              content: `#!/bin/bash
# Lints copy for brand-voice violations.
# Usage: ./voice-check.sh <file.md>

FILE="$1"
if [ -z "$FILE" ]; then
  echo "usage: voice-check.sh <file>" >&2
  exit 1
fi

BANNED=("synergy" "leverage" "disrupt" "ecosystem" "paradigm")
for word in "\${BANNED[@]}"; do
  grep -in "$word" "$FILE" && echo "  ↑ banned word: $word"
done
`
            }
          ]
        },
        {
          type: "dir", name: "templates", children: [
            {
              type: "file", name: "slide-cover.html.template",
              content: `<section class="cover">
  <h1>{{TITLE}}</h1>
  <p class="subtitle">{{SUBTITLE}}</p>
  <footer>Acme Corp · {{DATE}}</footer>
</section>
`
            }
          ]
        },
        {
          type: "file", name: "CHANGELOG.md",
          content: `# Changelog

## 1.2.0 — 2026-04-18
- Added voice-check.sh lint script
- Expanded banned-words list

## 1.1.0 — 2026-03-28
- Split typography and colors into references/

## 1.0.0 — 2026-03-14
- Initial release
`
        }
      ]
    }
  },
  {
    id: "api-conventions",
    name: "api-conventions",
    displayName: "API Conventions",
    description: "REST API design patterns — naming, errors, versioning — for this codebase",
    version: "0.4.1",
    createdAt: "2026-02-02T10:00:00Z",
    updatedAt: "2026-04-11T16:22:00Z",
    tags: ["api", "backend", "conventions"],
    tree: {
      type: "dir", name: "api-conventions", children: [
        {
          type: "file", name: "SKILL.md",
          content: `---
name: api-conventions
description: REST API design patterns for this codebase. Use when writing new endpoints, reviewing API PRs, or drafting OpenAPI specs.
---

# API Conventions

When writing or reviewing API endpoints, follow these rules.

## Naming
- Resource names are plural nouns: \`/users\`, \`/orders\`, never \`/user\` or \`/getOrder\`.
- Actions on a resource use sub-paths: \`/orders/{id}/cancel\`.
- Query params are snake_case: \`?created_after=...\`.

## Errors
All error responses match the envelope in \`references/error-format.md\`.

## Versioning
See \`references/versioning.md\` for the URL-prefix strategy.
`
        },
        {
          type: "dir", name: "references", children: [
            {
              type: "file", name: "error-format.md",
              content: `# Error envelope

\`\`\`json
{
  "error": {
    "code": "resource_not_found",
    "message": "Order 42 does not exist",
    "request_id": "req_01HF..."
  }
}
\`\`\`

Codes are snake_case, stable across versions.
`
            },
            {
              type: "file", name: "versioning.md",
              content: `# Versioning

URL-prefix versioning: \`/v1/...\`, \`/v2/...\`.
Breaking changes require a new major. Deprecate with \`Sunset:\` header 6 months ahead.
`
            }
          ]
        }
      ]
    }
  },
  {
    id: "commit-writer",
    name: "commit-writer",
    displayName: "Commit Writer",
    description: "Write conventional commit messages from a staged diff",
    version: "2.0.0",
    createdAt: "2026-01-18T11:00:00Z",
    updatedAt: "2026-04-20T08:45:00Z",
    tags: ["git", "workflow"],
    tree: {
      type: "dir", name: "commit-writer", children: [
        {
          type: "file", name: "SKILL.md",
          content: `---
name: commit-writer
description: Write conventional-commit messages from a staged git diff. Use when the user asks to "write a commit" or runs /commit.
---

# Commit Writer

Run \`scripts/parse-diff.sh\` to get the staged hunks, then produce a conventional-commit message.

## Format
\`\`\`
<type>(<scope>): <summary>

<body>

<footer>
\`\`\`

Types: feat, fix, refactor, docs, test, chore, perf, style.
Summary is imperative, under 72 chars.
`
        },
        {
          type: "dir", name: "scripts", children: [
            {
              type: "file", name: "parse-diff.sh",
              content: `#!/bin/bash
git diff --cached --stat
echo "---"
git diff --cached
`
            }
          ]
        }
      ]
    }
  },
  {
    id: "pdf-extract",
    name: "pdf-extract",
    displayName: "PDF Extract",
    description: "Extract form fields, text, and tables from PDF files",
    version: "0.9.0",
    createdAt: "2026-03-01T14:20:00Z",
    updatedAt: "2026-04-05T09:15:00Z",
    tags: ["pdf", "data", "extract"],
    tree: {
      type: "dir", name: "pdf-extract", children: [
        {
          type: "file", name: "SKILL.md",
          content: `---
name: pdf-extract
description: Extract structured data (form fields, text, tables) from PDF files. Use when the user provides a PDF and asks to pull data out of it.
---

# PDF Extract

## For form fields
Use \`scripts/extract_fields.py\` — returns JSON of all AcroForm fields.

## For tables
Use \`scripts/extract_tables.py\` — returns a list of CSV strings.

## For plain text
Use \`scripts/extract_text.py\`.
`
        },
        {
          type: "dir", name: "scripts", children: [
            {
              type: "file", name: "extract_fields.py",
              content: `import sys, json
from pypdf import PdfReader

def extract_fields(path):
    reader = PdfReader(path)
    fields = reader.get_form_text_fields() or {}
    return fields

if __name__ == "__main__":
    print(json.dumps(extract_fields(sys.argv[1]), indent=2))
`
            },
            {
              type: "file", name: "extract_tables.py",
              content: `import sys
import pdfplumber

def extract_tables(path):
    tables = []
    with pdfplumber.open(path) as pdf:
        for page in pdf.pages:
            for tbl in page.extract_tables():
                tables.append(tbl)
    return tables

if __name__ == "__main__":
    for t in extract_tables(sys.argv[1]):
        for row in t:
            print(",".join(str(c or "") for c in row))
        print("---")
`
            },
            {
              type: "file", name: "extract_text.py",
              content: `import sys
from pypdf import PdfReader

def extract_text(path):
    return "\\n\\n".join(p.extract_text() or "" for p in PdfReader(path).pages)

if __name__ == "__main__":
    print(extract_text(sys.argv[1]))
`
            }
          ]
        }
      ]
    }
  },
  {
    id: "security-review",
    name: "security-review",
    displayName: "Security Review",
    description: "Review PRs for common security issues: injection, auth gaps, secrets, insecure defaults",
    version: "1.0.0",
    createdAt: "2026-02-20T13:00:00Z",
    updatedAt: "2026-03-30T11:40:00Z",
    tags: ["security", "review"],
    tree: {
      type: "dir", name: "security-review", children: [
        {
          type: "file", name: "SKILL.md",
          content: `---
name: security-review
description: Review a pull request for security issues. Use when the user says "review this PR for security" or runs /security-review.
---

# Security Review

Check for:
1. Injection (SQL, command, XSS)
2. Hardcoded secrets / credentials
3. Insecure default configurations
4. Authentication and authorization gaps

Report findings with severity (low/med/high/critical) and remediation steps.
Reference \`references/owasp-top-10.md\` for our threat model.
`
        },
        {
          type: "dir", name: "references", children: [
            {
              type: "file", name: "owasp-top-10.md",
              content: `# OWASP Top 10 — our focus

1. Broken access control
2. Cryptographic failures
3. Injection
4. Insecure design
5. Security misconfiguration
`
            }
          ]
        }
      ]
    }
  }
];
