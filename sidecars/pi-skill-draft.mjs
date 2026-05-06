import { complete } from "@mariozechner/pi-ai";

function readStdin() {
  return new Promise((resolve, reject) => {
    let data = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => { data += chunk; });
    process.stdin.on("end", () => resolve(data));
    process.stdin.on("error", reject);
  });
}

function toInt(value, fallback) {
  const parsed = Number.parseInt(String(value ?? ""), 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function toBool(value, fallback) {
  if (typeof value === "boolean") return value;
  if (value === undefined || value === null || value === "") return fallback;
  const normalized = String(value).trim().toLowerCase();
  if (["1", "true", "yes", "on"].includes(normalized)) return true;
  if (["0", "false", "no", "off"].includes(normalized)) return false;
  return fallback;
}

function buildModel(input) {
  const modelId = String(input.model ?? "").trim();
  const baseUrl = String(input.baseUrl ?? "").trim();
  if (!modelId) throw new Error("agent model is required");
  if (!baseUrl) throw new Error("agent base URL is required");

  return {
    id: modelId,
    name: modelId,
    api: "openai-completions",
    provider: String(input.provider ?? "openai-compatible"),
    baseUrl,
    reasoning: toBool(input.reasoning, false),
    input: ["text"],
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
    contextWindow: toInt(input.contextWindow, 128000),
    maxTokens: toInt(input.maxTokens, 8192),
    compat: {
      supportsStore: toBool(input.supportsStore, false),
      supportsDeveloperRole: toBool(input.supportsDeveloperRole, false),
      supportsReasoningEffort: toBool(input.supportsReasoningEffort, false),
      supportsUsageInStreaming: toBool(input.supportsUsageInStreaming, false),
      supportsStrictMode: toBool(input.supportsStrictMode, false),
      maxTokensField: String(input.maxTokensField ?? "max_tokens"),
    },
  };
}

function assistantText(message) {
  if (typeof message.content === "string") return message.content.trim();
  if (!Array.isArray(message.content)) return "";
  return message.content
    .map((block) => {
      if (typeof block === "string") return block;
      if (block?.type === "text") return block.text ?? "";
      return "";
    })
    .join("")
    .trim();
}

function apiUrl(baseUrl, path) {
  return `${String(baseUrl).replace(/\/+$/, "")}/${path.replace(/^\/+/, "")}`;
}

function normalizeContentText(content) {
  if (typeof content === "string") return content.trim();
  if (!Array.isArray(content)) return "";
  return content
    .map((block) => {
      if (typeof block === "string") return block;
      if (block?.type === "text") return block.text ?? "";
      return "";
    })
    .join("")
    .trim();
}

function mergeHeaders(inputHeaders, apiKey) {
  const headers = {
    "content-type": "application/json",
    authorization: `Bearer ${apiKey}`,
  };
  if (inputHeaders && typeof inputHeaders === "object") {
    for (const [key, value] of Object.entries(inputHeaders)) {
      if (value !== undefined && value !== null) headers[key] = String(value);
    }
  }
  return headers;
}

async function completeNonStreaming(model, context, input, apiKey) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), toInt(input.timeoutSecs, 300) * 1000);
  const maxTokensField = String(input.maxTokensField ?? model.compat?.maxTokensField ?? "max_tokens");
  const messages = [
    { role: "system", content: context.systemPrompt },
    ...context.messages.map((message) => ({
      role: message.role,
      content: String(message.content ?? ""),
    })),
  ];
  const body = {
    model: model.id,
    messages,
    stream: false,
    temperature: Number.isFinite(input.temperature) ? input.temperature : 0.2,
    [maxTokensField]: toInt(input.outputMaxTokens, model.maxTokens),
  };

  try {
    const response = await fetch(apiUrl(model.baseUrl, "/chat/completions"), {
      method: "POST",
      headers: mergeHeaders(input.headers, apiKey),
      body: JSON.stringify(body),
      signal: controller.signal,
    });
    const raw = await response.text();
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${raw.slice(0, 500)}`);
    }

    const payload = JSON.parse(raw);
    const choice = payload.choices?.[0] ?? {};
    const text = normalizeContentText(choice.message?.content);
    if (!text) throw new Error("non-streaming response contained no assistant text");

    return {
      text,
      responseModel: payload.model ?? model.id,
      stopReason: choice.finish_reason ?? choice.finishReason ?? null,
      usage: payload.usage ?? null,
      transport: "openai_non_stream",
    };
  } finally {
    clearTimeout(timeout);
  }
}

async function main() {
  const input = JSON.parse(await readStdin());
  if (input.command !== "generate_skill_draft") {
    throw new Error(`unknown pi sidecar command: ${input.command}`);
  }

  const apiKey = String(input.apiKey ?? "").trim();
  if (!apiKey) throw new Error("agent API key is required");

  const model = buildModel(input);
  const context = {
    systemPrompt: "You create concise, valid Skill Notebook skill-package drafts. Return only the requested draft_json payload.",
    messages: [{ role: "user", content: String(input.prompt ?? ""), timestamp: Date.now() }],
  };

  const allowNonStreamFallback = toBool(input.allowNonStreamFallback, true);
  let result;
  try {
    const message = await complete(model, context, {
      apiKey,
      temperature: Number.isFinite(input.temperature) ? input.temperature : 0.2,
      maxTokens: toInt(input.outputMaxTokens, model.maxTokens),
      timeoutMs: toInt(input.timeoutSecs, 300) * 1000,
      maxRetries: Math.max(0, toInt(input.retryAttempts, 3) - 1),
      maxRetryDelayMs: toInt(input.maxRetryDelayMs, 60000),
      headers: input.headers && typeof input.headers === "object" ? input.headers : undefined,
    });
    result = {
      text: assistantText(message),
      responseModel: message.responseModel ?? null,
      stopReason: message.stopReason,
      usage: message.usage,
      transport: "pi_ai_stream",
    };
    if (!result.text && allowNonStreamFallback) {
      result = await completeNonStreaming(model, context, input, apiKey);
    }
  } catch (error) {
    if (!allowNonStreamFallback) throw error;
    try {
      result = await completeNonStreaming(model, context, input, apiKey);
    } catch (fallbackError) {
      const primary = error instanceof Error ? error.message : String(error);
      const fallback = fallbackError instanceof Error ? fallbackError.message : String(fallbackError);
      throw new Error(`pi-ai streaming failed: ${primary}; non-streaming fallback failed: ${fallback}`);
    }
  }

  const text = result.text;
  if (!text) {
    throw new Error(`pi sidecar returned no assistant text; stopReason=${result.stopReason}`);
  }

  process.stdout.write(`${JSON.stringify({
    ok: true,
    runtime: "pi_sidecar",
    provider: model.provider,
    model: model.id,
    responseModel: result.responseModel ?? null,
    stopReason: result.stopReason,
    usage: result.usage,
    transport: result.transport,
    text,
  })}\n`);
}

main().catch((error) => {
  process.stderr.write(`${JSON.stringify({
    ok: false,
    runtime: "pi_sidecar",
    error: error instanceof Error ? error.message : String(error),
  })}\n`);
  process.exit(1);
});
