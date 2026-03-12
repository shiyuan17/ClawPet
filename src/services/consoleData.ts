export type PlatformProtocol = "openai" | "anthropic";

export type PlatformConfig = {
  id: string;
  name: string;
  protocol: PlatformProtocol;
  baseUrl: string;
  pathPrefix: string;
  apiPath: string;
  apiKey: string;
  model: string;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
};

export type PlatformPreset = {
  name: string;
  region: "global" | "china";
  protocol: PlatformProtocol;
  baseUrl: string;
  pathPrefix: string;
  apiPath: string;
  apiKey: string;
  model: string;
  enabled: boolean;
  description: string;
  tags: string[];
  featured?: boolean;
};

export type RequestLog = {
  id: string;
  sessionId: string;
  platformId: string;
  platformName: string;
  protocol: PlatformProtocol;
  method: string;
  endpoint: string;
  baseUrl?: string;
  path?: string;
  requestHeaders?: Record<string, string>;
  requestBody: string;
  responseStatus: number;
  responseBody: string;
  streamSummary?: string;
  duration: number;
  firstTokenTime?: number;
  tokensPerSecond?: number;
  error?: string;
  promptTokens?: number;
  completionTokens?: number;
  totalTokens?: number;
  cacheReadInputTokens?: number;
  createdAt: number;
};

const platformsStorageKey = "keai.desktop-pet.platforms";
const activePlatformStorageKey = "keai.desktop-pet.active-platform";
const requestLogsStorageKey = "keai.desktop-pet.request-logs";
const maxLogCount = 180;

const defaultPlatformPresets: PlatformPreset[] = [
  {
    name: "OpenAI",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.openai.com",
    pathPrefix: "/openai",
    apiPath: "/v1/chat/completions",
    model: "gpt-4o-mini",
    apiKey: "",
    enabled: true,
    description: "官方 OpenAI API，适合 GPT 系列和通用对话。",
    tags: ["官方", "通用", "热门"],
    featured: true
  },
  {
    name: "Claude",
    region: "global",
    protocol: "anthropic",
    baseUrl: "https://api.anthropic.com",
    pathPrefix: "/claude",
    apiPath: "/v1/messages",
    model: "claude-sonnet-4-0",
    apiKey: "",
    enabled: true,
    description: "Anthropic 官方 Messages API，适合长上下文和写作任务。",
    tags: ["官方", "长上下文", "热门"],
    featured: true
  },
  {
    name: "Gemini",
    region: "global",
    protocol: "openai",
    baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
    pathPrefix: "/gemini",
    apiPath: "/chat/completions",
    model: "gemini-2.5-flash",
    apiKey: "",
    enabled: true,
    description: "Google Gemini 的 OpenAI 兼容入口，接入成本低。",
    tags: ["Google", "兼容", "热门"],
    featured: true
  },
  {
    name: "DeepSeek",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.deepseek.com",
    pathPrefix: "/deepseek",
    apiPath: "/v1/chat/completions",
    model: "deepseek-chat",
    apiKey: "",
    enabled: true,
    description: "DeepSeek 官方兼容接口，适合代码和通用问答。",
    tags: ["代码", "兼容", "热门"],
    featured: true
  },
  {
    name: "OpenRouter",
    region: "global",
    protocol: "openai",
    baseUrl: "https://openrouter.ai/api",
    pathPrefix: "/openrouter",
    apiPath: "/v1/chat/completions",
    model: "openai/gpt-4o-mini",
    apiKey: "",
    enabled: true,
    description: "聚合多家模型供应商，方便在不同平台间切换。",
    tags: ["聚合", "路由", "热门"],
    featured: true
  },
  {
    name: "Groq",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.groq.com/openai",
    pathPrefix: "/groq",
    apiPath: "/v1/chat/completions",
    model: "llama-3.3-70b-versatile",
    apiKey: "",
    enabled: true,
    description: "Groq 低延迟 OpenAI 兼容接口，适合快响应场景。",
    tags: ["低延迟", "兼容", "推理"],
    featured: true
  },
  {
    name: "Mistral",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.mistral.ai",
    pathPrefix: "/mistral",
    apiPath: "/v1/chat/completions",
    model: "mistral-small-latest",
    apiKey: "",
    enabled: true,
    description: "Mistral 官方接口，适合轻量对话和多语言任务。",
    tags: ["官方", "欧洲", "多语言"],
    featured: true
  },
  {
    name: "xAI",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.x.ai",
    pathPrefix: "/xai",
    apiPath: "/v1/chat/completions",
    model: "grok-3-mini",
    apiKey: "",
    enabled: true,
    description: "xAI Grok 系列接口，使用 OpenAI 风格调用。",
    tags: ["Grok", "兼容", "新平台"],
    featured: true
  },
  {
    name: "Perplexity",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.perplexity.ai",
    pathPrefix: "/perplexity",
    apiPath: "/chat/completions",
    model: "sonar-pro",
    apiKey: "",
    enabled: true,
    description: "Perplexity Sonar 接口，适合联网问答与检索增强。",
    tags: ["联网", "搜索", "问答"],
    featured: true
  },
  {
    name: "Fireworks",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.fireworks.ai/inference",
    pathPrefix: "/fireworks",
    apiPath: "/v1/chat/completions",
    model: "accounts/fireworks/models/llama-v3p1-70b-instruct",
    apiKey: "",
    enabled: false,
    description: "Fireworks 推理平台，适合高并发和开源模型调用。",
    tags: ["推理", "开源模型", "高并发"]
  },
  {
    name: "Together AI",
    region: "global",
    protocol: "openai",
    baseUrl: "https://api.together.xyz",
    pathPrefix: "/together",
    apiPath: "/v1/chat/completions",
    model: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
    apiKey: "",
    enabled: false,
    description: "Together AI 的开源模型平台，模型选择丰富。",
    tags: ["开源模型", "推理", "模型多"]
  },
  {
    name: "Kimi",
    region: "china",
    protocol: "openai",
    baseUrl: "https://api.moonshot.ai",
    pathPrefix: "/kimi",
    apiPath: "/v1/chat/completions",
    model: "moonshot-v1-8k",
    apiKey: "",
    enabled: false,
    description: "Moonshot/Kimi 官方接口，适合中文场景。",
    tags: ["中文", "国内", "助手"],
    featured: true
  },
  {
    name: "智谱",
    region: "china",
    protocol: "openai",
    baseUrl: "https://open.bigmodel.cn/api/paas",
    pathPrefix: "/zhipu",
    apiPath: "/v4/chat/completions",
    model: "glm-4-plus",
    apiKey: "",
    enabled: false,
    description: "智谱官方 GLM 接口，适合中文和国内部署场景。",
    tags: ["中文", "国内", "GLM"]
  },
  {
    name: "阿里云百炼",
    region: "china",
    protocol: "openai",
    baseUrl: "https://coding.dashscope.aliyuncs.com/coding",
    pathPrefix: "/coding",
    apiPath: "/v1/chat/completions",
    model: "qwen-plus",
    apiKey: "",
    enabled: false,
    description: "阿里云百炼 Coding 入口，适合通过本地代理接入 Qwen 系列模型。",
    tags: ["Qwen", "国内", "Coding"]
  }
];

function createId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

function safeParse<T>(raw: string | null, fallback: T): T {
  if (!raw) {
    return fallback;
  }

  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function getStorage() {
  if (typeof window === "undefined" || !window.localStorage) {
    return null;
  }

  return window.localStorage;
}

function sanitizePlatform(value: Partial<PlatformConfig> | null | undefined): PlatformConfig | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  if (
    typeof value.id !== "string" ||
    typeof value.name !== "string" ||
    (value.protocol !== "openai" && value.protocol !== "anthropic") ||
    typeof value.baseUrl !== "string" ||
    typeof value.pathPrefix !== "string" ||
    typeof value.apiPath !== "string" ||
    typeof value.apiKey !== "string" ||
    typeof value.model !== "string" ||
    typeof value.enabled !== "boolean" ||
    typeof value.createdAt !== "number" ||
    typeof value.updatedAt !== "number"
  ) {
    return null;
  }

  return {
    id: value.id,
    name: value.name.trim() || "未命名平台",
    protocol: value.protocol,
    baseUrl: normalizeBaseUrl(value.baseUrl),
    pathPrefix: normalizePathPrefix(value.pathPrefix),
    apiPath: normalizeApiPath(value.apiPath),
    apiKey: value.apiKey,
    model: value.model.trim(),
    enabled: value.enabled,
    createdAt: value.createdAt,
    updatedAt: value.updatedAt
  };
}

function sanitizeLog(value: Partial<RequestLog> | null | undefined): RequestLog | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  if (
    typeof value.id !== "string" ||
    typeof value.sessionId !== "string" ||
    typeof value.platformId !== "string" ||
    typeof value.platformName !== "string" ||
    (value.protocol !== "openai" && value.protocol !== "anthropic") ||
    typeof value.method !== "string" ||
    typeof value.endpoint !== "string" ||
    typeof value.requestBody !== "string" ||
    typeof value.responseStatus !== "number" ||
    typeof value.responseBody !== "string" ||
    typeof value.duration !== "number" ||
    typeof value.createdAt !== "number"
  ) {
    return null;
  }

  return {
    id: value.id,
    sessionId: value.sessionId,
    platformId: value.platformId,
    platformName: value.platformName,
    protocol: value.protocol,
    method: value.method,
    endpoint: value.endpoint,
    baseUrl: typeof value.baseUrl === "string" ? value.baseUrl : undefined,
    path: typeof value.path === "string" ? value.path : undefined,
    requestHeaders:
      value.requestHeaders && typeof value.requestHeaders === "object"
        ? Object.fromEntries(
            Object.entries(value.requestHeaders)
              .filter((entry): entry is [string, string] => typeof entry[0] === "string" && typeof entry[1] === "string")
              .map(([key, headerValue]) => [key, headerValue])
          )
        : undefined,
    requestBody: value.requestBody,
    responseStatus: value.responseStatus,
    responseBody: value.responseBody,
    streamSummary: typeof value.streamSummary === "string" ? value.streamSummary : undefined,
    duration: value.duration,
    firstTokenTime: typeof value.firstTokenTime === "number" ? value.firstTokenTime : undefined,
    tokensPerSecond: typeof value.tokensPerSecond === "number" ? value.tokensPerSecond : undefined,
    error: typeof value.error === "string" ? value.error : undefined,
    promptTokens: typeof value.promptTokens === "number" ? value.promptTokens : undefined,
    completionTokens: typeof value.completionTokens === "number" ? value.completionTokens : undefined,
    totalTokens: typeof value.totalTokens === "number" ? value.totalTokens : undefined,
    cacheReadInputTokens: typeof value.cacheReadInputTokens === "number" ? value.cacheReadInputTokens : undefined,
    createdAt: value.createdAt
  };
}

export function normalizeBaseUrl(value: string) {
  return value.trim().replace(/\/+$/, "");
}

export function normalizeApiPath(value: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    return "/v1/chat/completions";
  }

  return trimmed.startsWith("/") ? trimmed : `/${trimmed}`;
}

export function normalizePathPrefix(value: string) {
  const trimmed = value.trim();
  if (!trimmed) {
    return "/platform";
  }

  const normalized = trimmed.startsWith("/") ? trimmed : `/${trimmed}`;
  return normalized.replace(/\/+/g, "/").replace(/\/$/, "") || "/platform";
}

export function buildPlatformEndpoint(platform: Pick<PlatformConfig, "baseUrl" | "apiPath">) {
  return `${normalizeBaseUrl(platform.baseUrl)}${normalizeApiPath(platform.apiPath)}`;
}

function savePlatforms(platforms: PlatformConfig[]) {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  storage.setItem(platformsStorageKey, JSON.stringify(platforms));
}

function shouldCollapseToSingleDefault(platforms: PlatformConfig[]) {
  if (platforms.length <= 1 || platforms.length !== defaultPlatformPresets.length) {
    return false;
  }

  const presetNames = new Set(defaultPlatformPresets.map((preset) => preset.name));
  return platforms.every(
    (platform) =>
      presetNames.has(platform.name) &&
      !platform.apiKey.trim()
  );
}

function saveActivePlatformId(platformId: string | null) {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  if (!platformId) {
    storage.removeItem(activePlatformStorageKey);
    return;
  }

  storage.setItem(activePlatformStorageKey, platformId);
}

function saveLogs(logs: RequestLog[]) {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  storage.setItem(requestLogsStorageKey, JSON.stringify(logs.slice(0, maxLogCount)));
}

function isDefaultOpenClawLog(log: Pick<RequestLog, "platformId" | "platformName">) {
  return log.platformId === "openclaw-default" || log.platformName === "OpenClaw 默认通道";
}

function resolvePlatformForLog(log: RequestLog, platforms: PlatformConfig[]) {
  const exactMatch = platforms.find((platform) => platform.id === log.platformId);
  if (exactMatch) {
    return exactMatch;
  }

  const normalizedBaseUrl = typeof log.baseUrl === "string" ? normalizeBaseUrl(log.baseUrl) : "";
  const normalizedPath = typeof log.path === "string" ? normalizeApiPath(log.path) : "";

  return (
    platforms.find(
      (platform) =>
        platform.protocol === log.protocol &&
        normalizeBaseUrl(platform.baseUrl) === normalizedBaseUrl &&
        normalizeApiPath(platform.apiPath) === normalizedPath
    ) ?? null
  );
}

function migrateLogPlatformMetadata(logs: RequestLog[], platforms: PlatformConfig[]) {
  if (!platforms.length) {
    return { logs, changed: false };
  }

  let changed = false;
  const nextLogs = logs.map((log) => {
    const matchedPlatform = resolvePlatformForLog(log, platforms);
    if (!matchedPlatform) {
      return log;
    }

    const shouldUpdateId = log.platformId !== matchedPlatform.id;
    const shouldUpdateName = log.platformName !== matchedPlatform.name;
    const shouldMigrateDefaultLabel = isDefaultOpenClawLog(log);

    if (!shouldUpdateId && !shouldUpdateName && !shouldMigrateDefaultLabel) {
      return log;
    }

    changed = true;
    return {
      ...log,
      platformId: matchedPlatform.id,
      platformName: matchedPlatform.name
    };
  });

  return { logs: nextLogs, changed };
}

export function getPlatformPresets() {
  return defaultPlatformPresets.map((preset) => ({ ...preset }));
}

export function loadPlatforms() {
  const storage = getStorage();
  if (!storage) {
    return seedDefaultPlatforms();
  }

  const parsed = safeParse<unknown[]>(storage.getItem(platformsStorageKey), []);
  const platforms = parsed
    .map((item) => sanitizePlatform(item as Partial<PlatformConfig>))
    .filter((item): item is PlatformConfig => item !== null);

  if (platforms.length > 0) {
    if (shouldCollapseToSingleDefault(platforms)) {
      const seeded = seedDefaultPlatforms();
      savePlatforms(seeded);
      return seeded;
    }

    return platforms;
  }

  const seeded = seedDefaultPlatforms();
  savePlatforms(seeded);
  return seeded;
}

export function loadActivePlatformId() {
  const storage = getStorage();
  if (!storage) {
    return null;
  }

  const value = storage.getItem(activePlatformStorageKey);
  return typeof value === "string" && value ? value : null;
}

export function loadRequestLogs(platforms: PlatformConfig[] = []) {
  const storage = getStorage();
  if (!storage) {
    return [];
  }

  const parsed = safeParse<unknown[]>(storage.getItem(requestLogsStorageKey), []);
  const logs = parsed
    .map((item) => sanitizeLog(item as Partial<RequestLog>))
    .filter((item): item is RequestLog => item !== null)
    .sort((left, right) => right.createdAt - left.createdAt);

  const migrated = migrateLogPlatformMetadata(logs, platforms);
  if (migrated.changed) {
    saveLogs(migrated.logs);
  }

  return migrated.logs;
}

export function seedDefaultPlatforms() {
  const now = Date.now();
  const defaultPreset = defaultPlatformPresets.find((preset) => preset.name === "OpenAI") ?? defaultPlatformPresets[0];

  return [
    {
      id: createId("platform"),
      createdAt: now,
      updatedAt: now,
      ...defaultPreset
    }
  ];
}

export function upsertPlatform(platforms: PlatformConfig[], value: Omit<PlatformConfig, "createdAt" | "updatedAt">) {
  const now = Date.now();
  const current = platforms.find((item) => item.id === value.id);
  const nextPlatform: PlatformConfig = {
    ...value,
    baseUrl: normalizeBaseUrl(value.baseUrl),
    pathPrefix: normalizePathPrefix(value.pathPrefix),
    apiPath: normalizeApiPath(value.apiPath),
    createdAt: current?.createdAt ?? now,
    updatedAt: now
  };

  const nextPlatforms = current
    ? platforms.map((item) => (item.id === value.id ? nextPlatform : item))
    : [nextPlatform, ...platforms];

  savePlatforms(nextPlatforms);
  return nextPlatforms;
}

export function createPlatformDraft(
  value?: Partial<Pick<PlatformConfig, "name" | "protocol" | "baseUrl" | "pathPrefix" | "apiPath" | "model" | "enabled" | "apiKey">>
) {
  return {
    id: createId("platform"),
    name: value?.name ?? "",
    protocol: value?.protocol ?? "openai",
    baseUrl: normalizeBaseUrl(value?.baseUrl ?? "https://api.openai.com"),
    pathPrefix: normalizePathPrefix(value?.pathPrefix ?? "/openai"),
    apiPath: normalizeApiPath(value?.apiPath ?? "/v1/chat/completions"),
    apiKey: value?.apiKey ?? "",
    model: value?.model ?? "gpt-4o-mini",
    enabled: value?.enabled ?? true
  } satisfies Omit<PlatformConfig, "createdAt" | "updatedAt">;
}

export function deletePlatform(platforms: PlatformConfig[], platformId: string) {
  const nextPlatforms = platforms.filter((item) => item.id !== platformId);
  savePlatforms(nextPlatforms);

  const activePlatformId = loadActivePlatformId();
  if (activePlatformId === platformId) {
    saveActivePlatformId(null);
  }

  return nextPlatforms;
}

export function setPlatformEnabled(platforms: PlatformConfig[], platformId: string, enabled: boolean) {
  const nextPlatforms = platforms.map((item) => (item.id === platformId ? { ...item, enabled, updatedAt: Date.now() } : item));
  savePlatforms(nextPlatforms);
  return nextPlatforms;
}

export function setActivePlatform(platformId: string | null) {
  saveActivePlatformId(platformId);
}

export function appendRequestLog(log: Omit<RequestLog, "id" | "createdAt">) {
  const nextLog: RequestLog = {
    ...log,
    id: createId("log"),
    createdAt: Date.now()
  };
  const logs = [nextLog, ...loadRequestLogs()].slice(0, maxLogCount);
  saveLogs(logs);
  return logs;
}

export function clearRequestLogs() {
  saveLogs([]);
  return [];
}

export function exportLogsAsJson(logs: RequestLog[]) {
  return JSON.stringify(logs, null, 2);
}
