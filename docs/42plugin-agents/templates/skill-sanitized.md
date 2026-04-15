# Sanitized Skill Template

Template for publishable skills. All personal/sensitive information removed.

---

## SKILL.md Template (Sanitized)

```yaml
---
name: ${SKILL_NAME}
description: |
  ${WHAT_IT_DOES}.
  Use when ${TRIGGER_CONDITIONS}.
argument-hint: "${ARGUMENT_HINT}"
metadata:
  author: ${AUTHOR}
  version: 1.0.0
  title: ${TITLE_ZH}
  description_zh: ${DESCRIPTION_ZH}
---

# ${SKILL_TITLE}

${CORE_PRINCIPLE}

## When to Use

- ${TRIGGER_1}
- ${TRIGGER_2}

## Quick Start

${QUICK_START_INSTRUCTIONS}

## Configuration

Before using this skill, configure the following:

| Variable | Description | Example |
| -------- | ----------- | ------- |
| `<your-path>` | Path to your project | `/path/to/project` |
| `<your-api-key>` | Your API key | `sk-xxx...` |

## Usage

${USAGE_INSTRUCTIONS}

## Notes

${CAVEATS_AND_LIMITATIONS}
```

---

## Sanitization Checklist

### Personal Paths

| Before | After |
| ------ | ----- |
| `/Users/<username>/projects/` | `<your-path>/` or `~/` |
| `/home/<username>/` | `<your-path>/` or `~/` |
| `C:\Users\<Username>\` | `<your-path>/` |

### Credentials

| Before | After |
| ------ | ----- |
| `api_key = "sk-abc123..."` | `api_key = "<your-api-key>"` |
| `token: "ghp_xxx"` | `token: "<your-github-token>"` |
| `password: "secret123"` | `password: "<your-password>"` |

### Personal Information

| Before | After |
| ------ | ----- |
| `user@personal.com` | `user@example.com` |
| `192.168.1.100` | `<your-server-ip>` or `localhost` |
| Project-specific paths | Generic placeholders |

### Project-Specific Details

| Before | After |
| ------ | ----- |
| Company-specific APIs | Generic API patterns |
| Internal tool names | Standard tool categories |
| Private repository URLs | Public examples or placeholders |

---

## Sanitization Script

Quick check for sensitive content:

```bash
#!/bin/bash
# sanitize-check.sh

SKILL_PATH="$1"

echo "Checking for sensitive content in: $SKILL_PATH"
echo "=============================================="

# Personal paths
echo -e "\n[Checking personal paths]"
grep -rn '/Users/[^/]\+/' "$SKILL_PATH" && echo "FOUND: Personal macOS paths"
grep -rn '/home/[^/]\+/' "$SKILL_PATH" && echo "FOUND: Personal Linux paths"

# Credentials
echo -e "\n[Checking credentials]"
grep -rni 'api[_-]\?key.*=' "$SKILL_PATH" && echo "FOUND: API keys"
grep -rni 'token.*=' "$SKILL_PATH" && echo "FOUND: Tokens"
grep -rni 'password.*=' "$SKILL_PATH" && echo "FOUND: Passwords"
grep -rn 'BEGIN.*PRIVATE' "$SKILL_PATH" && echo "FOUND: Private keys"

# Network
echo -e "\n[Checking network addresses]"
grep -rn '192\.168\.' "$SKILL_PATH" && echo "FOUND: Private IP addresses"
grep -rn '10\.\d\+\.\d\+\.\d\+' "$SKILL_PATH" && echo "FOUND: Private IP addresses"

# Email
echo -e "\n[Checking email addresses]"
grep -rn '[a-zA-Z0-9._%+-]\+@[a-zA-Z0-9.-]\+\.[a-zA-Z]\{2,\}' "$SKILL_PATH" | grep -v 'example\.com' && echo "FOUND: Real email addresses"

echo -e "\n=============================================="
echo "Review complete. Fix any FOUND items before publishing."
```

---

## Before Publishing

1. **Run sanitization check**
   ```bash
   ./sanitize-check.sh <skill-path>
   ```

2. **Replace all placeholders**
   - Ensure no `${VARIABLE}` remains
   - All `<your-xxx>` are intentional placeholders with documentation

3. **Test with fresh environment**
   - Skill should work without your personal configuration
   - Error messages should guide user to configure

4. **Add configuration section**
   - Document all required environment variables
   - Provide example values (not real ones)

5. **Verify with 42plugin**
   ```bash
   42plugin __score <skill-path> -t skill
   ```
