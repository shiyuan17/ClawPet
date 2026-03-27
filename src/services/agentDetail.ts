const ZH_AGENT_MARKDOWN_FILES = import.meta.glob(
  ["../agent-data/zh/**/*.md", "!../agent-data/zh/README*.md"],
  {
    query: "?raw",
    import: "default"
  }
) as Record<string, () => Promise<unknown>>;

const EN_AGENT_MARKDOWN_FILES = import.meta.glob(["../agent-data/en/**/*.md", "!../agent-data/en/README*.md"], {
  query: "?raw",
  import: "default"
}) as Record<string, () => Promise<unknown>>;

const contentCache = new Map<string, string>();
const MACHINE_SUFFIX_RE = /\([^)]+\.local\)(?=\.|$)/gi;

function stripMachineSuffix(segment: string) {
  return segment.replace(MACHINE_SUFFIX_RE, "");
}

function normalizePathSegments(path: string) {
  return path
    .replace(/\\/g, "/")
    .split("/")
    .filter(Boolean)
    .map((segment) => stripMachineSuffix(segment))
    .join("/");
}

function normalizeSourcePath(sourcePath: string) {
  const trimmed = sourcePath.trim().replace(/^\/+/, "").replace(/\\/g, "/");
  const withoutPrefix = trimmed.replace(/^\.?\/*agent-data\//i, "");
  const withoutLocale = withoutPrefix.replace(/^(en|zh)\//i, "");
  return normalizePathSegments(withoutLocale);
}

function toKey(locale: "en" | "zh", sourcePath: string) {
  return `../agent-data/${locale}/${normalizeSourcePath(sourcePath)}`;
}

function buildCanonicalKeyIndex(
  files: Record<string, () => Promise<unknown>>,
  locale: "en" | "zh"
) {
  const index = new Map<string, string>();
  const expectedPrefix = `../agent-data/${locale}/`;
  for (const key of Object.keys(files)) {
    const relativePath = key.startsWith(expectedPrefix) ? key.slice(expectedPrefix.length) : key;
    const canonicalKey = toKey(locale, relativePath);
    if (!index.has(canonicalKey)) {
      index.set(canonicalKey, key);
    }
  }
  return index;
}

const ZH_AGENT_CANONICAL_KEY_INDEX = buildCanonicalKeyIndex(ZH_AGENT_MARKDOWN_FILES, "zh");
const EN_AGENT_CANONICAL_KEY_INDEX = buildCanonicalKeyIndex(EN_AGENT_MARKDOWN_FILES, "en");

function resolveExistingKey(
  files: Record<string, () => Promise<unknown>>,
  canonicalIndex: Map<string, string>,
  preferredKey: string
) {
  if (files[preferredKey]) {
    return preferredKey;
  }
  return canonicalIndex.get(preferredKey) ?? preferredKey;
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
  const zhKey = resolveExistingKey(
    ZH_AGENT_MARKDOWN_FILES,
    ZH_AGENT_CANONICAL_KEY_INDEX,
    toKey("zh", normalizedSourcePath)
  );
  const enKey = resolveExistingKey(
    EN_AGENT_MARKDOWN_FILES,
    EN_AGENT_CANONICAL_KEY_INDEX,
    toKey("en", normalizedSourcePath)
  );

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
