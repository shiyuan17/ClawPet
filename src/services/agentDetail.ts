const ZH_AGENT_MARKDOWN_FILES = import.meta.glob(
  ["../agent-data/zh/**/*.md", "!../agent-data/zh/README.md", "!../agent-data/zh/README.zh-TW.md"],
  {
    query: "?raw",
    import: "default"
  }
) as Record<string, () => Promise<unknown>>;

const EN_AGENT_MARKDOWN_FILES = import.meta.glob(["../agent-data/en/**/*.md", "!../agent-data/en/README.md"], {
  query: "?raw",
  import: "default"
}) as Record<string, () => Promise<unknown>>;

const contentCache = new Map<string, string>();

function normalizeSourcePath(sourcePath: string) {
  const trimmed = sourcePath.trim().replace(/^\/+/, "");
  const withoutPrefix = trimmed.replace(/^\.?\/*agent-data\//i, "");
  return withoutPrefix.replace(/^(en|zh)\//i, "");
}

function toKey(locale: "en" | "zh", sourcePath: string) {
  return `../agent-data/${locale}/${normalizeSourcePath(sourcePath)}`;
}

async function loadRawByKey(
  files: Record<string, () => Promise<unknown>>,
  key: string
): Promise<string | null> {
  const loader = files[key];
  if (!loader) {
    return null;
  }

  const cached = contentCache.get(key);
  if (typeof cached === "string") {
    return cached;
  }

  const loaded = await loader();
  const next = typeof loaded === "string" ? loaded : String(loaded ?? "");
  contentCache.set(key, next);
  return next;
}

export function likelyNeedsFurtherZhTranslation(_contentZh: string) {
  return false;
}

export async function loadAgentDetailMarkdownZh(sourcePath: string) {
  const normalizedSourcePath = normalizeSourcePath(sourcePath);
  const zhKey = toKey("zh", normalizedSourcePath);
  const enKey = toKey("en", normalizedSourcePath);

  const [contentZhRaw, contentEnRaw] = await Promise.all([
    loadRawByKey(ZH_AGENT_MARKDOWN_FILES, zhKey),
    loadRawByKey(EN_AGENT_MARKDOWN_FILES, enKey)
  ]);

  if (!contentZhRaw && !contentEnRaw) {
    return {
      found: false,
      sourcePath: normalizedSourcePath,
      contentRaw: "",
      contentZh: `# 文件未找到\n\n未找到对应详情文件：\`${normalizedSourcePath}\`。`
    };
  }

  return {
    found: true,
    sourcePath: normalizedSourcePath,
    contentRaw: contentEnRaw ?? contentZhRaw ?? "",
    contentZh: contentZhRaw ?? contentEnRaw ?? ""
  };
}
