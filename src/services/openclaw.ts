export type OpenClawRole = "system" | "user" | "assistant";

export type OpenClawMessage = {
  role: OpenClawRole;
  content: string;
};

export type OpenClawResponse = {
  text: string;
};

type OpenAIChatCompletionResponse = {
  choices?: Array<{
    message?: {
      content?: string | Array<{ type?: string; text?: string }>;
    };
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
        return { text: content };
      }

      if (Array.isArray(content)) {
        const text = content
          .map((item) => (item && typeof item === "object" && "text" in item && typeof item.text === "string" ? item.text : ""))
          .filter(Boolean)
          .join("\n");

        if (text) {
          return { text };
        }
      }
    }

    if ("text" in result && typeof result.text === "string") {
      return { text: result.text };
    }

    if ("content" in result && typeof result.content === "string") {
      return { text: result.content };
    }
  }

  return {
    text: "OpenClaw 返回了无法识别的内容格式。"
  };
}

function resolveEndpoint() {
  const runtime = getRuntimeBridge();

  return runtime?.__OPENCLAW_ENDPOINT__ ?? runtime?.__OPENCLAW__?.endpoint ?? import.meta.env.VITE_OPENCLAW_API_URL ?? null;
}

function isOpenAICompatibleEndpoint(endpoint: string) {
  return /\/v1\/chat\/completions\/?$/i.test(endpoint);
}

export async function sendOpenClawChat(messages: OpenClawMessage[]) {
  const runtime = getRuntimeBridge();

  if (runtime?.__OPENCLAW__?.chat) {
    const result = await runtime.__OPENCLAW__.chat({ messages });
    return normalizeResponse(result);
  }

  if (runtime?.__TAURI__?.core?.invoke) {
    try {
      const result = await runtime.__TAURI__.core.invoke("openclaw_chat", { messages });
      return normalizeResponse(result);
    } catch (error) {
      throw new Error(toErrorMessage(error));
    }
  }

  const endpoint = resolveEndpoint();
  if (!endpoint) {
    throw new Error(
      "未找到 OpenClaw 接口。请通过 window.__OPENCLAW__.chat、window.__OPENCLAW_ENDPOINT__ 或 VITE_OPENCLAW_API_URL 提供调用入口。"
    );
  }

  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(import.meta.env.VITE_OPENCLAW_GATEWAY_TOKEN
        ? { Authorization: `Bearer ${import.meta.env.VITE_OPENCLAW_GATEWAY_TOKEN}` }
        : {})
    },
    body: JSON.stringify(
      isOpenAICompatibleEndpoint(endpoint)
        ? {
            model: import.meta.env.VITE_OPENCLAW_MODEL || "openclaw",
            messages
          }
        : { messages }
    )
  });

  if (!response.ok) {
    throw new Error(`OpenClaw 请求失败（${response.status}）`);
  }

  return normalizeResponse((await response.json()) as unknown);
}
