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

export type StaffStatus = "active" | "busy" | "offline";

export type StaffMember = {
  id: string;
  name: string;
  role: string;
  workspace: string;
  status: StaffStatus;
  focus: string;
  tags: string[];
  memoryCount: number;
  taskCount: number;
  updatedAt: number;
};

export type MemoryStatus = "active" | "archived";

export type MemoryRecord = {
  id: string;
  title: string;
  owner: string;
  scope: string;
  summary: string;
  relatedDocCount: number;
  status: MemoryStatus;
  updatedAt: number;
};

export type DocumentStatus = "draft" | "published";

export type DocumentRecord = {
  id: string;
  title: string;
  category: string;
  owner: string;
  source: string;
  summary: string;
  status: DocumentStatus;
  updatedAt: number;
};

export const DEFAULT_TASK_PROJECT_NAME = "常规任务";

export type TaskStatus = "todo" | "in_progress" | "in_review" | "done" | "cancelled";

export type TaskRecord = {
  id: string;
  title: string;
  project: string;
  owner: string;
  priority: "p0" | "p1" | "p2";
  status: TaskStatus;
  dueAt: number;
  summary: string;
  updatedAt: number;
};

const platformsStorageKey = "keai.desktop-pet.platforms";
const activePlatformStorageKey = "keai.desktop-pet.active-platform";
const requestLogsStorageKey = "keai.desktop-pet.request-logs";
const staffStorageKey = "keai.desktop-pet.staff";
const memoryStorageKey = "keai.desktop-pet.memory";
const documentsStorageKey = "keai.desktop-pet.documents";
const tasksStorageKey = "keai.desktop-pet.tasks";
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

function sanitizeStringArray(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }

  return value.filter((item): item is string => typeof item === "string").map((item) => item.trim()).filter(Boolean);
}

function sanitizeStaff(value: Partial<StaffMember> | null | undefined): StaffMember | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  if (
    typeof value.id !== "string" ||
    typeof value.name !== "string" ||
    typeof value.role !== "string" ||
    typeof value.workspace !== "string" ||
    (value.status !== "active" && value.status !== "busy" && value.status !== "offline") ||
    typeof value.focus !== "string" ||
    typeof value.memoryCount !== "number" ||
    typeof value.taskCount !== "number" ||
    typeof value.updatedAt !== "number"
  ) {
    return null;
  }

  return {
    id: value.id,
    name: value.name.trim() || "未命名员工",
    role: value.role.trim() || "未分配角色",
    workspace: value.workspace.trim() || "主工作区",
    status: value.status,
    focus: value.focus.trim() || "待补充当前职责",
    tags: sanitizeStringArray(value.tags),
    memoryCount: Math.max(0, Math.round(value.memoryCount)),
    taskCount: Math.max(0, Math.round(value.taskCount)),
    updatedAt: value.updatedAt
  };
}

function sanitizeMemory(value: Partial<MemoryRecord> | null | undefined): MemoryRecord | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  if (
    typeof value.id !== "string" ||
    typeof value.title !== "string" ||
    typeof value.owner !== "string" ||
    typeof value.scope !== "string" ||
    typeof value.summary !== "string" ||
    typeof value.relatedDocCount !== "number" ||
    (value.status !== "active" && value.status !== "archived") ||
    typeof value.updatedAt !== "number"
  ) {
    return null;
  }

  return {
    id: value.id,
    title: value.title.trim() || "未命名记忆",
    owner: value.owner.trim() || "主控台",
    scope: value.scope.trim() || "长期记忆",
    summary: value.summary.trim() || "暂无摘要",
    relatedDocCount: Math.max(0, Math.round(value.relatedDocCount)),
    status: value.status,
    updatedAt: value.updatedAt
  };
}

function sanitizeDocument(value: Partial<DocumentRecord> | null | undefined): DocumentRecord | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  if (
    typeof value.id !== "string" ||
    typeof value.title !== "string" ||
    typeof value.category !== "string" ||
    typeof value.owner !== "string" ||
    typeof value.source !== "string" ||
    typeof value.summary !== "string" ||
    (value.status !== "draft" && value.status !== "published") ||
    typeof value.updatedAt !== "number"
  ) {
    return null;
  }

  return {
    id: value.id,
    title: value.title.trim() || "未命名文档",
    category: value.category.trim() || "运行文档",
    owner: value.owner.trim() || "主控台",
    source: value.source.trim() || "/docs",
    summary: value.summary.trim() || "暂无摘要",
    status: value.status,
    updatedAt: value.updatedAt
  };
}

function sanitizeTask(value: Partial<TaskRecord> | null | undefined): TaskRecord | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const rawStatus = typeof (value as { status?: unknown }).status === "string" ? ((value as { status?: string }).status ?? "") : "";
  const normalizedStatus =
    rawStatus === "blocked"
      ? "in_review"
      : rawStatus === "todo" ||
          rawStatus === "in_progress" ||
          rawStatus === "in_review" ||
          rawStatus === "done" ||
          rawStatus === "cancelled"
        ? rawStatus
        : null;

  if (
    typeof value.id !== "string" ||
    typeof value.title !== "string" ||
    typeof value.project !== "string" ||
    typeof value.owner !== "string" ||
    (value.priority !== "p0" && value.priority !== "p1" && value.priority !== "p2") ||
    !normalizedStatus ||
    typeof value.dueAt !== "number" ||
    typeof value.summary !== "string" ||
    typeof value.updatedAt !== "number"
  ) {
    return null;
  }

  return {
    id: value.id,
    title: value.title.trim() || "未命名任务",
    project: value.project.trim(),
    owner: value.owner.trim() || "待分配",
    priority: value.priority,
    status: normalizedStatus,
    dueAt: value.dueAt,
    summary: value.summary.trim() || "暂无说明",
    updatedAt: value.updatedAt
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

function saveStaff(staff: StaffMember[]) {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  storage.setItem(staffStorageKey, JSON.stringify(staff));
}

function saveMemories(memories: MemoryRecord[]) {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  storage.setItem(memoryStorageKey, JSON.stringify(memories));
}

function saveDocuments(documents: DocumentRecord[]) {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  storage.setItem(documentsStorageKey, JSON.stringify(documents));
}

function saveTasks(tasks: TaskRecord[]) {
  const storage = getStorage();
  if (!storage) {
    return;
  }

  storage.setItem(tasksStorageKey, JSON.stringify(tasks));
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

export function seedDefaultStaff() {
  const now = Date.now();
  return [
    {
      id: createId("staff"),
      name: "Commander",
      role: "平台负责人",
      workspace: "control-center",
      status: "active" as const,
      focus: "负责平台策略、入口配置和版本推进",
      tags: ["平台", "调度", "负责人"],
      memoryCount: 8,
      taskCount: 3,
      updatedAt: now
    },
    {
      id: createId("staff"),
      name: "Archivist",
      role: "记忆与文档管理员",
      workspace: "memory/docs",
      status: "busy" as const,
      focus: "整理长期记忆、归档文档和维护知识索引",
      tags: ["记忆", "文档", "归档"],
      memoryCount: 21,
      taskCount: 5,
      updatedAt: now - 30 * 60 * 1000
    },
    {
      id: createId("staff"),
      name: "Operator",
      role: "任务执行协调员",
      workspace: "tasks",
      status: "active" as const,
      focus: "跟进执行状态、阻塞项和交付节奏",
      tags: ["任务", "执行", "协作"],
      memoryCount: 5,
      taskCount: 7,
      updatedAt: now - 12 * 60 * 1000
    }
  ];
}

export function seedDefaultMemories() {
  const now = Date.now();
  return [
    {
      id: createId("memory"),
      title: "平台接入约定",
      owner: "Commander",
      scope: "长期记忆",
      summary: "记录各模型平台的协议、路径前缀和默认模型命名约定。",
      relatedDocCount: 4,
      status: "active" as const,
      updatedAt: now - 2 * 60 * 60 * 1000
    },
    {
      id: createId("memory"),
      title: "员工轮值备忘",
      owner: "Archivist",
      scope: "共享记忆",
      summary: "沉淀员工职责分配、轮值安排和当前工作区归属。",
      relatedDocCount: 2,
      status: "active" as const,
      updatedAt: now - 90 * 60 * 1000
    },
    {
      id: createId("memory"),
      title: "任务复盘摘录",
      owner: "Operator",
      scope: "当日记录",
      summary: "记录最近一次任务推进中的阻塞原因与处理结果。",
      relatedDocCount: 3,
      status: "active" as const,
      updatedAt: now - 20 * 60 * 1000
    }
  ];
}

export function seedDefaultDocuments() {
  const now = Date.now();
  return [
    {
      id: createId("doc"),
      title: "代理配置接入说明",
      category: "运行文档",
      owner: "Commander",
      source: "/docs/platforms.md",
      summary: "维护新增模型平台时的地址、密钥和路由规范。",
      status: "published" as const,
      updatedAt: now - 4 * 60 * 60 * 1000
    },
    {
      id: createId("doc"),
      title: "员工协同手册",
      category: "团队文档",
      owner: "Archivist",
      source: "/docs/staff-playbook.md",
      summary: "描述员工角色、协同边界与交接方式。",
      status: "published" as const,
      updatedAt: now - 3 * 60 * 60 * 1000
    },
    {
      id: createId("doc"),
      title: "任务推进草案",
      category: "任务文档",
      owner: "Operator",
      source: "/docs/task-runbook.md",
      summary: "整理当前任务分解、风险和预期交付节点。",
      status: "draft" as const,
      updatedAt: now - 40 * 60 * 1000
    }
  ];
}

export function seedDefaultTasks() {
  const now = Date.now();
  return [
    {
      id: createId("task"),
      title: "补齐代理配置入口",
      project: "",
      owner: "Commander",
      priority: "p0" as const,
      status: "in_progress" as const,
      dueAt: now + 2 * 60 * 60 * 1000,
      summary: "统一平台配置、默认入口和接入说明。",
      updatedAt: now - 10 * 60 * 1000
    },
    {
      id: createId("task"),
      title: "建立员工管理面板",
      project: "",
      owner: "Archivist",
      priority: "p1" as const,
      status: "todo" as const,
      dueAt: now + 8 * 60 * 60 * 1000,
      summary: "可查看员工角色、职责焦点和当前状态。",
      updatedAt: now - 50 * 60 * 1000
    },
    {
      id: createId("task"),
      title: "同步记忆与文档索引",
      project: "",
      owner: "Operator",
      priority: "p1" as const,
      status: "in_review" as const,
      dueAt: now + 24 * 60 * 60 * 1000,
      summary: "等待进一步接入真实文件源后完成联动。",
      updatedAt: now - 80 * 60 * 1000
    }
  ];
}

export function loadStaff() {
  const storage = getStorage();
  if (!storage) {
    return seedDefaultStaff();
  }

  const parsed = safeParse<unknown[]>(storage.getItem(staffStorageKey), []);
  const staff = parsed
    .map((item) => sanitizeStaff(item as Partial<StaffMember>))
    .filter((item): item is StaffMember => item !== null)
    .sort((left, right) => right.updatedAt - left.updatedAt);

  if (staff.length > 0) {
    return staff;
  }

  const seeded = seedDefaultStaff();
  saveStaff(seeded);
  return seeded;
}

export function loadMemories() {
  const storage = getStorage();
  if (!storage) {
    return seedDefaultMemories();
  }

  const parsed = safeParse<unknown[]>(storage.getItem(memoryStorageKey), []);
  const memories = parsed
    .map((item) => sanitizeMemory(item as Partial<MemoryRecord>))
    .filter((item): item is MemoryRecord => item !== null)
    .sort((left, right) => right.updatedAt - left.updatedAt);

  if (memories.length > 0) {
    return memories;
  }

  const seeded = seedDefaultMemories();
  saveMemories(seeded);
  return seeded;
}

export function loadDocuments() {
  const storage = getStorage();
  if (!storage) {
    return seedDefaultDocuments();
  }

  const parsed = safeParse<unknown[]>(storage.getItem(documentsStorageKey), []);
  const documents = parsed
    .map((item) => sanitizeDocument(item as Partial<DocumentRecord>))
    .filter((item): item is DocumentRecord => item !== null)
    .sort((left, right) => right.updatedAt - left.updatedAt);

  if (documents.length > 0) {
    return documents;
  }

  const seeded = seedDefaultDocuments();
  saveDocuments(seeded);
  return seeded;
}

export function loadTasks() {
  const storage = getStorage();
  if (!storage) {
    return seedDefaultTasks();
  }

  const parsed = safeParse<unknown[]>(storage.getItem(tasksStorageKey), []);
  const tasks = parsed
    .map((item) => sanitizeTask(item as Partial<TaskRecord>))
    .filter((item): item is TaskRecord => item !== null)
    .sort((left, right) => right.updatedAt - left.updatedAt);

  if (tasks.length > 0) {
    return tasks;
  }

  const seeded = seedDefaultTasks();
  saveTasks(seeded);
  return seeded;
}

export function createStaffDraft() {
  return {
    id: createId("staff"),
    name: "",
    role: "",
    workspace: "control-center",
    status: "active" as StaffStatus,
    focus: "",
    tags: "",
    memoryCount: 0,
    taskCount: 0
  };
}

export function createMemoryDraft() {
  return {
    id: createId("memory"),
    title: "",
    owner: "Commander",
    scope: "长期记忆",
    summary: "",
    relatedDocCount: 0,
    status: "active" as MemoryStatus
  };
}

export function createDocumentDraft() {
  return {
    id: createId("doc"),
    title: "",
    category: "运行文档",
    owner: "Commander",
    source: "/docs/",
    summary: "",
    status: "draft" as DocumentStatus
  };
}

export function createTaskDraft() {
  return {
    id: createId("task"),
    title: "",
    project: "",
    owner: "Commander",
    priority: "p1" as TaskRecord["priority"],
    status: "todo" as TaskStatus,
    dueAt: Date.now() + 4 * 60 * 60 * 1000,
    summary: ""
  };
}

export function upsertStaff(staff: StaffMember[], value: Omit<StaffMember, "updatedAt">) {
  const nextItem: StaffMember = { ...value, tags: sanitizeStringArray(value.tags), updatedAt: Date.now() };
  const current = staff.find((item) => item.id === value.id);
  const next = current ? staff.map((item) => (item.id === value.id ? nextItem : item)) : [nextItem, ...staff];
  saveStaff(next);
  return next;
}

export function upsertMemory(memories: MemoryRecord[], value: Omit<MemoryRecord, "updatedAt">) {
  const nextItem: MemoryRecord = { ...value, updatedAt: Date.now() };
  const current = memories.find((item) => item.id === value.id);
  const next = current ? memories.map((item) => (item.id === value.id ? nextItem : item)) : [nextItem, ...memories];
  saveMemories(next);
  return next;
}

export function upsertDocument(documents: DocumentRecord[], value: Omit<DocumentRecord, "updatedAt">) {
  const nextItem: DocumentRecord = { ...value, updatedAt: Date.now() };
  const current = documents.find((item) => item.id === value.id);
  const next = current ? documents.map((item) => (item.id === value.id ? nextItem : item)) : [nextItem, ...documents];
  saveDocuments(next);
  return next;
}

export function upsertTask(tasks: TaskRecord[], value: Omit<TaskRecord, "updatedAt">) {
  const nextItem: TaskRecord = { ...value, updatedAt: Date.now() };
  const current = tasks.find((item) => item.id === value.id);
  const next = current ? tasks.map((item) => (item.id === value.id ? nextItem : item)) : [nextItem, ...tasks];
  saveTasks(next);
  return next;
}

export function deleteStaff(staff: StaffMember[], id: string) {
  const next = staff.filter((item) => item.id !== id);
  saveStaff(next);
  return next;
}

export function deleteMemory(memories: MemoryRecord[], id: string) {
  const next = memories.filter((item) => item.id !== id);
  saveMemories(next);
  return next;
}

export function deleteDocument(documents: DocumentRecord[], id: string) {
  const next = documents.filter((item) => item.id !== id);
  saveDocuments(next);
  return next;
}

export function deleteTask(tasks: TaskRecord[], id: string) {
  const next = tasks.filter((item) => item.id !== id);
  saveTasks(next);
  return next;
}
