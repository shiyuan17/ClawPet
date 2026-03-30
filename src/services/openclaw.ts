export type OpenClawRole = "system" | "user" | "assistant";

export type OpenClawMessage = {
  role: OpenClawRole;
  content: string;
};

export type OpenClawResponse = {
  text: string;
  status?: number;
  raw?: string;
  usage?: {
    promptTokens?: number;
    completionTokens?: number;
    totalTokens?: number;
    cacheReadInputTokens?: number;
  };
};

export type OpenClawRequestOptions = {
  endpoint?: string | null;
  apiKey?: string | null;
  model?: string | null;
  protocol?: "openai" | "anthropic" | null;
  agentId?: string | null;
  sessionKey?: string | null;
  signal?: AbortSignal | null;
};

type OpenAIChatCompletionResponse = {
  choices?: Array<{
    message?: {
      content?: string | Array<{ type?: string; text?: string }>;
    };
  }>;
  usage?: {
    prompt_tokens?: number;
    completion_tokens?: number;
    total_tokens?: number;
  };
};

type AnthropicMessageResponse = {
  content?: Array<{
    type?: string;
    text?: string;
  }>;
  usage?: {
    input_tokens?: number;
    output_tokens?: number;
  };
};

function extractUsage(result: unknown) {
  if (!result || typeof result !== "object" || !("usage" in result) || !result.usage || typeof result.usage !== "object") {
    return undefined;
  }

  const usage = result.usage as Record<string, unknown>;
  const promptTokens =
    typeof usage.prompt_tokens === "number"
      ? usage.prompt_tokens
      : typeof usage.input_tokens === "number"
        ? usage.input_tokens
        : undefined;
  const completionTokens =
    typeof usage.completion_tokens === "number"
      ? usage.completion_tokens
      : typeof usage.output_tokens === "number"
        ? usage.output_tokens
        : undefined;
  const totalTokens =
    typeof usage.total_tokens === "number"
      ? usage.total_tokens
      : typeof promptTokens === "number" || typeof completionTokens === "number"
        ? (promptTokens ?? 0) + (completionTokens ?? 0)
        : undefined;
  const cacheReadInputTokens =
    typeof usage.cache_read_input_tokens === "number"
      ? usage.cache_read_input_tokens
      : typeof usage.cache_read_tokens === "number"
        ? usage.cache_read_tokens
        : undefined;

  if (
    typeof promptTokens !== "number" &&
    typeof completionTokens !== "number" &&
    typeof totalTokens !== "number" &&
    typeof cacheReadInputTokens !== "number"
  ) {
    return undefined;
  }

  return {
    promptTokens,
    completionTokens,
    totalTokens,
    cacheReadInputTokens
  };
}

type OpenClawWindowBridge = {
  chat?: (payload: { messages: OpenClawMessage[]; agentId?: string | null }) => Promise<OpenClawResponse | string>;
  endpoint?: string;
};

type RuntimeBridge = Window & {
  __TAURI__?: {
    core?: {
      invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
  };
  __OPENCLAW__?: OpenClawWindowBridge;
  __OPENCLAW_ENDPOINT__?: string;
};

function createAbortError() {
  const error = new Error("请求已中断。");
  error.name = "AbortError";
  return error;
}

function throwIfAborted(signal?: AbortSignal | null) {
  if (signal?.aborted) {
    throw createAbortError();
  }
}

async function withAbortSignal<T>(promise: Promise<T>, signal?: AbortSignal | null) {
  if (!signal) {
    return promise;
  }
  throwIfAborted(signal);
  return await Promise.race([
    promise,
    new Promise<never>((_, reject) => {
      signal.addEventListener("abort", () => reject(createAbortError()), { once: true });
    })
  ]);
}

function toErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") {
    return error.message;
  }

  return "OpenClaw 调用失败。";
}

function getRuntimeBridge() {
  if (typeof window === "undefined") {
    return null;
  }

  return window as RuntimeBridge;
}

function normalizeResponse(result: unknown): OpenClawResponse {
  const usage = extractUsage(result);

  if (typeof result === "string") {
    return { text: result, usage };
  }

  if (result && typeof result === "object") {
    if ("choices" in result && Array.isArray((result as OpenAIChatCompletionResponse).choices)) {
      const firstChoice = (result as OpenAIChatCompletionResponse).choices?.[0];
      const content = firstChoice?.message?.content;

      if (typeof content === "string") {
        return { text: content, raw: JSON.stringify(result, null, 2), usage };
      }

      if (Array.isArray(content)) {
        const text = content
          .map((item) => (item && typeof item === "object" && "text" in item && typeof item.text === "string" ? item.text : ""))
          .filter(Boolean)
          .join("\n");

        if (text) {
          return { text, raw: JSON.stringify(result, null, 2), usage };
        }
      }
    }

    if ("content" in result && Array.isArray((result as AnthropicMessageResponse).content)) {
      const text = (result as AnthropicMessageResponse).content
        ?.map((item) => (item?.type === "text" && typeof item.text === "string" ? item.text : ""))
        .filter(Boolean)
        .join("\n");

      if (text) {
        return { text, raw: JSON.stringify(result, null, 2), usage };
      }
    }

    if ("text" in result && typeof result.text === "string") {
      return { text: result.text, raw: JSON.stringify(result, null, 2), usage };
    }

    if ("content" in result && typeof result.content === "string") {
      return { text: result.content, raw: JSON.stringify(result, null, 2), usage };
    }
  }

  return {
    text: "OpenClaw 返回了无法识别的内容格式。",
    raw: typeof result === "string" ? result : JSON.stringify(result ?? null, null, 2),
    usage
  };
}

function resolveEndpoint() {
  const runtime = getRuntimeBridge();

  return runtime?.__OPENCLAW_ENDPOINT__ ?? runtime?.__OPENCLAW__?.endpoint ?? import.meta.env.VITE_OPENCLAW_API_URL ?? null;
}

function isOpenAICompatibleEndpoint(endpoint: string) {
  return /\/v1\/chat\/completions\/?$/i.test(endpoint);
}

function isLocalProxyEndpoint(endpoint: string) {
  return /^https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?\//i.test(endpoint);
}

function getAnthropicPayload(messages: OpenClawMessage[], model: string) {
  const system = messages
    .filter((message) => message.role === "system")
    .map((message) => message.content)
    .join("\n\n");
  const chatMessages = messages
    .filter((message) => message.role !== "system")
    .map((message) => ({
      role: message.role === "assistant" ? "assistant" : "user",
      content: message.content
    }));

  return {
    model,
    max_tokens: 1024,
    ...(system ? { system } : {}),
    messages: chatMessages
  };
}

export async function sendOpenClawChat(messages: OpenClawMessage[], options: OpenClawRequestOptions = {}) {
  throwIfAborted(options.signal);
  const runtime = getRuntimeBridge();
  const protocol = options.protocol ?? "openai";

  const agentId = options.agentId ?? null;

  if (!agentId && !options.endpoint && !options.apiKey && !options.model && !options.protocol && runtime?.__OPENCLAW__?.chat) {
    const result = await withAbortSignal(runtime.__OPENCLAW__.chat({ messages }), options.signal);
    return normalizeResponse(result);
  }

  if (runtime?.__TAURI__?.core?.invoke) {
    try {
      const result = await withAbortSignal(
        runtime.__TAURI__.core.invoke("openclaw_chat", {
          messages,
          endpoint: options.endpoint ?? null,
          apiKey: options.apiKey ?? null,
          model: options.model ?? null,
          protocol,
          agentId,
          sessionKey: options.sessionKey ?? null
        }),
        options.signal
      );
      return normalizeResponse(result);
    } catch (error) {
      throw new Error(toErrorMessage(error));
    }
  }

  const endpoint = options.endpoint ?? resolveEndpoint();
  if (!endpoint) {
    throw new Error(
      "未找到 OpenClaw 接口。请通过 window.__OPENCLAW__.chat、window.__OPENCLAW_ENDPOINT__ 或 VITE_OPENCLAW_API_URL 提供调用入口。"
    );
  }

  const headers: Record<string, string> = {
    "Content-Type": "application/json"
  };

  // Local OpenClaw is responsible for upstream authentication itself.
  // Only send credentials when the caller explicitly provides one.
  const token = options.apiKey?.trim() || "";
  const model = options.model?.trim() || import.meta.env.VITE_OPENCLAW_MODEL || "";
  const body =
    protocol === "anthropic"
      ? (() => {
          if (!model) {
            throw new Error("Anthropic 协议需要模型配置。");
          }
          return getAnthropicPayload(messages, model);
        })()
      : isOpenAICompatibleEndpoint(endpoint)
        ? model
          ? {
              model,
              messages
            }
          : { messages }
        : { messages };

  if (protocol === "anthropic") {
    if (token) {
      headers["x-api-key"] = token;
    }
    headers["anthropic-version"] = "2023-06-01";
  } else if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  if (protocol === "openai" && isOpenAICompatibleEndpoint(endpoint) && isLocalProxyEndpoint(endpoint)) {
    headers["x-openclaw-scopes"] = "operator.write";
  }

  const response = await fetch(endpoint, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
    signal: options.signal ?? undefined
  });

  if (!response.ok) {
    const errorText = await response.text().catch(() => "");
    throw new Error(errorText ? `OpenClaw 请求失败（${response.status}）：${errorText}` : `OpenClaw 请求失败（${response.status}）`);
  }

  const raw = await response.text();
  let parsed: unknown = raw;
  try {
    parsed = JSON.parse(raw);
  } catch {
    parsed = raw;
  }

  const normalized = normalizeResponse(parsed);
  return {
    ...normalized,
    status: response.status,
    raw: normalized.raw ?? raw
  };
}
