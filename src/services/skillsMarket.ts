export type SkillMarketSortBy = "score" | "downloads" | "stars";
export type SkillMarketOrder = "asc" | "desc";

export type SkillMarketCategory =
  | "ai-intelligence"
  | "developer-tools"
  | "productivity"
  | "data-analysis"
  | "content-creation"
  | "security-compliance"
  | "communication-collaboration";

export type SkillMarketSkill = {
  category: string;
  description: string;
  descriptionZh: string;
  downloads: number;
  homepage: string;
  installs: number;
  name: string;
  ownerName: string;
  score: number;
  slug: string;
  stars: number;
  tags: string[];
  updatedAt: number;
  version: string;
};

export type SkillMarketListResult = {
  skills: SkillMarketSkill[];
  total: number;
};

type SkillMarketApiSkill = {
  category?: unknown;
  description?: unknown;
  description_zh?: unknown;
  downloads?: unknown;
  homepage?: unknown;
  installs?: unknown;
  name?: unknown;
  ownerName?: unknown;
  score?: unknown;
  slug?: unknown;
  stars?: unknown;
  tags?: unknown;
  updated_at?: unknown;
  version?: unknown;
};

type SkillMarketApiEnvelope = {
  code?: unknown;
  data?: {
    skills?: unknown;
    total?: unknown;
  } | null;
  message?: unknown;
};

const SKILL_MARKET_BASE_URL = "https://lightmake.site/api";

type TauriInvoke = (command: string, args?: Record<string, unknown>) => Promise<unknown>;

function getTauriInvoke(): TauriInvoke | null {
  if (typeof window === "undefined") {
    return null;
  }
  const runtime = window as Window & {
    __TAURI__?: {
      core?: {
        invoke?: TauriInvoke;
      };
    };
  };
  return runtime.__TAURI__?.core?.invoke ?? null;
}

function toNumber(value: unknown, fallback = 0): number {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string" && value.trim()) {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return fallback;
}

function toStringValue(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function normalizeSkillMarketSkill(raw: SkillMarketApiSkill): SkillMarketSkill {
  return {
    category: toStringValue(raw.category),
    description: toStringValue(raw.description),
    descriptionZh: toStringValue(raw.description_zh),
    downloads: toNumber(raw.downloads),
    homepage: toStringValue(raw.homepage),
    installs: toNumber(raw.installs),
    name: toStringValue(raw.name, "Unknown Skill"),
    ownerName: toStringValue(raw.ownerName),
    score: toNumber(raw.score),
    slug: toStringValue(raw.slug),
    stars: toNumber(raw.stars),
    tags: Array.isArray(raw.tags) ? raw.tags.filter((item): item is string => typeof item === "string") : [],
    updatedAt: toNumber(raw.updated_at),
    version: toStringValue(raw.version, "v1.0.0")
  };
}

function normalizeSkillMarketEnvelope(raw: SkillMarketApiEnvelope): SkillMarketListResult {
  const code = toNumber(raw.code, -1);
  if (code !== 0) {
    const message = toStringValue(raw.message, "技能市场接口返回异常。");
    throw new Error(message);
  }

  const list = Array.isArray(raw.data?.skills) ? (raw.data?.skills as SkillMarketApiSkill[]) : [];
  const skills = list.map((item) => normalizeSkillMarketSkill(item));
  const total = Math.max(toNumber(raw.data?.total, skills.length), skills.length);
  return { skills, total };
}

function normalizeSkillMarketRequestError(error: unknown): Error {
  if (!(error instanceof Error)) {
    return new Error("技能市场加载失败。");
  }
  const lower = error.message.trim().toLowerCase();
  if (lower === "load failed" || lower.includes("failed to fetch") || lower.includes("networkerror")) {
    return new Error("技能市场网络请求失败，请检查网络或稍后重试。");
  }
  return error;
}

async function requestSkillMarket(
  url: string,
  fallback?: {
    command: string;
    args?: Record<string, unknown>;
  }
): Promise<SkillMarketListResult> {
  try {
    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "application/json"
      }
    });

    if (!response.ok) {
      const detail = await response.text().catch(() => "");
      throw new Error(detail ? `技能市场请求失败（${response.status}）：${detail}` : `技能市场请求失败（${response.status}）。`);
    }

    const parsed = (await response.json()) as SkillMarketApiEnvelope;
    return normalizeSkillMarketEnvelope(parsed);
  } catch (error) {
    const invoke = getTauriInvoke();
    if (invoke && fallback?.command) {
      try {
        const payload = (await invoke(fallback.command, fallback.args)) as SkillMarketApiEnvelope;
        return normalizeSkillMarketEnvelope(payload);
      } catch (invokeError) {
        throw normalizeSkillMarketRequestError(invokeError);
      }
    }
    throw normalizeSkillMarketRequestError(error);
  }
}

export async function fetchSkillTop50(): Promise<SkillMarketListResult> {
  return requestSkillMarket(`${SKILL_MARKET_BASE_URL}/skills/top`, {
    command: "load_skill_market_top"
  });
}

export async function fetchSkillsByCategory(
  category: SkillMarketCategory,
  options: {
    page?: number;
    pageSize?: number;
    sortBy?: SkillMarketSortBy;
    order?: SkillMarketOrder;
  } = {}
): Promise<SkillMarketListResult> {
  const page = options.page ?? 1;
  const pageSize = options.pageSize ?? 24;
  const sortBy = options.sortBy ?? "score";
  const order = options.order ?? "desc";
  const searchParams = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    sortBy,
    order,
    category
  });
  return requestSkillMarket(`${SKILL_MARKET_BASE_URL}/skills?${searchParams.toString()}`, {
    command: "load_skill_market_by_category",
    args: {
      page,
      pageSize,
      sortBy,
      order,
      category
    }
  });
}

export async function fetchSkillsGlobal(
  options: {
    page?: number;
    pageSize?: number;
    sortBy?: SkillMarketSortBy;
    order?: SkillMarketOrder;
  } = {}
): Promise<SkillMarketListResult> {
  const page = options.page ?? 1;
  const pageSize = options.pageSize ?? 200;
  const sortBy = options.sortBy ?? "score";
  const order = options.order ?? "desc";
  const searchParams = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    sortBy,
    order
  });
  return requestSkillMarket(`${SKILL_MARKET_BASE_URL}/skills?${searchParams.toString()}`, {
    command: "load_skill_market_by_category",
    args: {
      page,
      pageSize,
      sortBy,
      order,
      category: ""
    }
  });
}
