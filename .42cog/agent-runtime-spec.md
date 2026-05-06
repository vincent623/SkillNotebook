# Agent Runtime Spec

## Purpose

Skill Notebook is CLI-first. The GUI helps select inputs, inspect previews, evaluate packages, and save versions, but the actual skill draft generator must run through the same Rust core and CLI contract.

The V1 generic runtime is `pi_sidecar`: a Node sidecar that uses `@mariozechner/pi-ai` to call OpenAI-compatible APIs with user-supplied provider settings. It prefers streaming chat completions and falls back to a non-streaming `/chat/completions` request for providers that do not return usable streaming text.

## Runtime Order

Auto mode selects generators in this order:

1. `pi_sidecar`, when fully configured and the sidecar script is present.
2. `skill-create`, when the command exists.
3. Claude CLI, when the command exists.
4. local template fallback, only when no real generator is available.

If a real generator is invoked and fails, the failure is surfaced. It must not silently become a template draft.

## Environment

| Variable | Required | Meaning |
| --- | --- | --- |
| `SKILL_NOTEBOOK_GENERATOR_RUNTIME` | no | `auto`, `pi_sidecar`, `skill_create`, `claude_cli`, or `template`. Supersedes `SKILL_NOTEBOOK_CREATOR_MODE`. |
| `SKILL_NOTEBOOK_AGENT_BASE_URL` | yes for pi | OpenAI-compatible API base URL. |
| `SKILL_NOTEBOOK_AGENT_API_KEY` | yes for pi | API key passed only to the sidecar process. |
| `SKILL_NOTEBOOK_AGENT_MODEL` | yes for pi | Model id sent to the provider. |
| `SKILL_NOTEBOOK_AGENT_PROVIDER` | no | Diagnostic label, defaults to `openai-compatible`. |
| `SKILL_NOTEBOOK_AGENT_TIMEOUT_SECS` | no | Sidecar/model timeout, defaults to 300 seconds. |
| `SKILL_NOTEBOOK_AGENT_RETRY_ATTEMPTS` | no | Total pi-ai attempts, defaults to 3. |
| `SKILL_NOTEBOOK_PI_NODE_BIN` | no | Node binary, defaults to `node`. |
| `SKILL_NOTEBOOK_PI_SIDECAR_SCRIPT` | no | Explicit sidecar script path. |

## GUI Configuration

Settings includes an `Agent Runtime` form for the same fields:

- runtime mode
- provider
- base URL
- model
- API key
- Node binary
- optional sidecar script override
- timeout seconds
- retry attempts

The app persists this to:

```txt
~/Library/Application Support/Skill Notebook/settings.json
```

The file is written with owner-only permissions on macOS. API keys are not returned to the frontend after save; the UI only shows whether a key is configured. Environment variables remain the highest-precedence override for CLI and temporary sessions.

## Sidecar Protocol

Rust starts:

```txt
node <pi-skill-draft.mjs>
```

Rust writes one JSON object to stdin:

```json
{
  "command": "generate_skill_draft",
  "prompt": "full generator prompt",
  "provider": "openai-compatible",
  "baseUrl": "https://api.example.com/v1",
  "apiKey": "redacted",
  "model": "model-id",
  "timeoutSecs": 300,
  "retryAttempts": 3
}
```

The sidecar writes one JSON object to stdout:

```json
{
  "ok": true,
  "runtime": "pi_sidecar",
  "provider": "openai-compatible",
  "model": "model-id",
  "responseModel": "model-id",
  "stopReason": "stop",
  "transport": "pi_ai_stream",
  "text": "<draft_json>{...}</draft_json>"
}
```

`text` is parsed by the existing generator draft parser. API keys are never written to generator logs.

`transport` is diagnostic only. Current values are `pi_ai_stream` and `openai_non_stream`.

## Packaging

The source sidecar is `sidecars/pi-skill-draft.mjs`.

`npm run build:sidecars` bundles it to `dist-sidecars/pi-skill-draft.mjs`.

Tauri bundles the generated file as a resource so the packaged app can run the same runtime without depending on source-tree layout.

## Diagnostics

`skill doctor generator` and Settings must expose:

- selected runtime and preferred generator
- pi configured/available status
- Node resolved path
- sidecar resolved path
- provider label
- base URL/API key/model configured status
- timeout and retry attempts
