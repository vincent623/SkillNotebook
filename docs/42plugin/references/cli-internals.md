# CLI Internal Reference

Technical reference for CLI commands and parameters.

---

## Non-interactive mode

CLI commands may enter interactive mode. Use these flags to prevent blocking:

| Command     | Non-interactive flags                                       |
| ----------- | ----------------------------------------------------------- |
| search      | `--json --limit N`                                          |
| suggest     | `--json --limit N`                                          |
| update      | `--check` (read-only) or `-y` (auto-confirm)                |
| info        | `--json`                                                    |
| list        | `--json`                                                    |
| chat export | `--session <id>` or filter flags (`--date`/`--from`/`--to`) |
| kit delete  | `--force`                                                   |
| uninstall   | `--purge`                                                   |

> **Note**: `--all` means "export all without confirmation". Do not combine with `--session`.

---

## Search

```bash
# Basic search (JSON output for parsing)
42plugin search "keyword" --json --limit 10

# Filter by type (skill, agent, command, hook)
42plugin search "writing" --type skill --json --limit 10

# Search kits only
42plugin search "research" --kit --json

# Search plugins only
42plugin search "pdf" --plugin --json

# AI semantic search (requires login)
42plugin search "keyword" --ai --json
```

**Keyword search output fields:**

```json
{
  "id": "author/kit/plugin-name",
  "type": "skill",
  "title": "Plugin Title",
  "description": "What it does",
  "downloads": 1234
}
```

**AI search (`--ai`) JSON output:**

```json
{
  "plugins": [{ "full_name": "...", "similarity": 0.87, ... }],
  "kits":    [{ "full_name": "...", "similarity": 0.81, ... }],
  "meta": { "total": 12 }
}
```

- Calls plugin and kit APIs in parallel; results sorted by `similarity` desc
- If one API fails, shows the other's results with a yellow warning

---

## Install / Uninstall

```bash
# Install to current project
42plugin install author/kit/plugin-name

# Install globally
42plugin install author/kit/plugin-name --global

# Uninstall (complete removal)
42plugin uninstall author/kit/plugin-name --purge
```

---

## Plugin info

```bash
42plugin info author/kit/plugin-name --json
```

**Output fields:**

```json
{
  "id": "author/kit/plugin-name",
  "type": "skill",
  "title": "Plugin Title",
  "description": "Detailed description",
  "author": "username",
  "version": "1.0.0",
  "downloads": 1234,
  "readme": "# Plugin README..."
}
```

---

## List installed

```bash
# List project plugins
42plugin list --json

# List global plugins
42plugin list --global --json
```

---

## Publish

```bash
# Validate without publishing
42plugin publish ./path --dry-run

# Publish as private (default)
42plugin publish ./path --type skill

# Publish as public
42plugin publish ./path --type skill --public

# Publish to specific kit
42plugin publish ./path --type skill --kit my-kit
```

**Required file structure:**

- Skills: `SKILL.md` with frontmatter
- Agents: `AGENT.md` with frontmatter
- Commands: `COMMAND.md` with frontmatter

---

## Kit management

```bash
# List kits
42plugin kit list --json

# Create kit
42plugin kit create my-kit --title "My Kit"

# Delete kit (requires --force)
42plugin kit delete my-kit --force
```

---

## Suggest

Smart plugin recommendation based on project context. Requires login.

```bash
# Scan current directory
42plugin suggest --json --limit 10

# Scan specific path
42plugin suggest ~/my-project --json

# By role or scenario (skips file scanning)
42plugin suggest --for "product manager" --json
42plugin suggest --for "data analysis, python" --json
```

**Scan signals (auto mode):**

| Signal | Source | Examples |
| ------ | ------ | -------- |
| File types | Directory listing | `.pdf`, `.xlsx`, `.fig`, `.ipynb` |
| Role keywords | CLAUDE.md / README | 产品经理, UI designer, researcher |
| Node deps | package.json | react/vue → frontend; jest → testing |
| Python deps | requirements.txt / pyproject.toml | fastapi, pandas, pytest |
| Lang markers | go.mod / Cargo.toml / pubspec.yaml | Go / Rust / Flutter |

Scan patterns fetched from `/v1/ai-search/suggest/patterns`, cached locally 24h.

**Quota:** Free 5/day · Pro 420/day (Pro also sees private & team plugins)

---

## Update

Check and update installed plugins.

```bash
# Check for updates only (no download)
42plugin update --check

# Interactive update (current project)
42plugin update

# Update specific plugin or kit
42plugin update author/name
42plugin update author/kit/slug     # updates all plugins from that kit

# Update global plugins
42plugin update --global

# Update all projects, skip confirmation
42plugin update --all --yes

# Force reinstall even if version matches
42plugin update --force
```

**Status indicators:**

| Symbol | Status | Meaning |
| ------ | ------ | ------- |
| `↑` | outdated | newer version available |
| `✓` | current | already latest |
| `⚠ (远程已删除)` | missing | removed from registry; use `uninstall` |
| `⚠ (权限不足)` | forbidden | no access, skipped |

Checks run in parallel (batches of 5); updates also run in parallel.

---

## Status & upgrade

```bash
# Show plan, quota usage, VIP expiry
42plugin status

# Upgrade CLI to latest version
42plugin upgrade
```

---

## Authentication

```bash
# Login (opens browser)
42plugin auth

# Check login status
42plugin auth status

# Logout
42plugin auth logout
```

---

## Chat export

```bash
# Export current session
42plugin chat export --session "${CLAUDE_SESSION_ID}" -o ./chats

# Export with full details (tool calls, thinking)
42plugin chat export --session "${CLAUDE_SESSION_ID}" -o ./chats --detail

# Export by date range
42plugin chat export --from 2024-01-01 --to 2024-01-31 -o ./chats

# Export all (batch mode, skips confirmation)
42plugin chat export --all -o ./chats
```

---

## Error codes

| Code             | Meaning                      | Solution                                          |
| ---------------- | ---------------------------- | ------------------------------------------------- |
| UNAUTHORIZED     | Not logged in                | Run `42plugin auth`                               |
| NOT_FOUND        | Plugin/kit not found         | Check spelling, try search                        |
| QUOTA_EXCEEDED   | Quota limit reached          | Free: upgrade to Pro; Pro: daily limit, retry tmr |
| INVALID_FORMAT   | Invalid plugin format        | Check SKILL.md frontmatter                        |
| NETWORK_ERROR    | Connection failed            | Retry later                                       |
