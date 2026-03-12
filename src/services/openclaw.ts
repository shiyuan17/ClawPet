export type OpenClawRole = "system" | "user" | "assistant";

export type OpenClawMessage = {
  role: OpenClawRole;
  content: string;
};

export type OpenClawResponse = {
  text: string;
  status?: number;
  raw?: string;
};

export type OpenClawRequestOptions = {
  endpoint?: string | null;
  apiKey?: string | null;
  model?: string | null;
  protocol?: "openai" | "anthropic" | null;
};

type OpenAIChatCompletionResponse = {
  choices?: Array<{
    message?: {
      content?: string | Array<{ type?: string; text?: string }>;
    };
  }>;
};

type AnthropicMessageResponse = {
  content?: Array<{
    type?: string;
    text?: string;
  }>;
};

type OpenClawWindowBridge = {
  chat?: (payload: { messages: OpenClawMessage[] }) => Promise<OpenClawResponse | string>;
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
  if (typeof result === "string") {
    return { text: result };
  }

  if (result && typeof result === "object") {
    if ("choices" in result && Array.isArray((result as OpenAIChatCompletionResponse).choices)) {
      const firstChoice = (result as OpenAIChatCompletionResponse).choices?.[0];
      const content = firstChoice?.message?.content;

      if (typeof content === "string") {
        return { text: content, raw: JSON.stringify(result, null, 2) };
      }

      if (Array.isArray(content)) {
        const text = content
          .map((item) => (item && typeof item === "object" && "text" in item && typeof item.text === "string" ? item.text : ""))
          .filter(Boolean)
          .join("\n");

        if (text) {
          return { text, raw: JSON.stringify(result, null, 2) };
        }
      }
    }

    if ("content" in result && Array.isArray((result as AnthropicMessageResponse).content)) {
      const text = (result as AnthropicMessageResponse).content
        ?.map((item) => (item?.type === "text" && typeof item.text === "string" ? item.text : ""))
        .filter(Boolean)
        .join("\n");

      if (text) {
        return { text, raw: JSON.stringify(result, null, 2) };
      }
    }

    if ("text" in result && typeof result.text === "string") {
      return { text: result.text, raw: JSON.stringify(result, null, 2) };
    }

    if ("content" in result && typeof result.content === "string") {
      return { text: result.content, raw: JSON.stringify(result, null, 2) };
    }
  }

  return {
    text: "OpenClaw 返回了无法识别的内容格式。",
    raw: typeof result === "string" ? result : JSON.stringify(result ?? null, null, 2)
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
  const runtime = getRuntimeBridge();
  const protocol = options.protocol ?? "openai";

  if (!options.endpoint && !options.apiKey && !options.model && !options.protocol && runtime?.__OPENCLAW__?.chat) {
    const result = await runtime.__OPENCLAW__.chat({ messages });
    return normalizeResponse(result);
  }

  if (!options.endpoint && runtime?.__TAURI__?.core?.invoke) {
    try {
      const result = await runtime.__TAURI__.core.invoke("openclaw_chat", {
        messages,
        endpoint: options.endpoint ?? null,
        apiKey: options.apiKey ?? null,
        model: options.model ?? null,
        protocol
      });
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

  const token =
    options.apiKey?.trim() ||
    (isLocalProxyEndpoint(endpoint) ? "" : import.meta.env.VITE_OPENCLAW_GATEWAY_TOKEN || "");
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

  const response = await fetch(endpoint, {
    method: "POST",
    headers,
    body: JSON.stringify(body)
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
