import {
  CHAT_FILE_AUDIO_EXTENSIONS,
  CHAT_FILE_DETECTION_CACHE_MAX,
  CHAT_FILE_HTML_EXTENSIONS,
  CHAT_FILE_IMAGE_EXTENSIONS,
  CHAT_FILE_PATH_TRAILING_TOKENS,
  CHAT_FILE_UNIX_PATH_PATTERN,
  CHAT_FILE_URL_PATTERN,
  CHAT_FILE_VIDEO_EXTENSIONS,
  CHAT_FILE_WINDOWS_PATH_PATTERN
} from "../constants";
import { formatAttachmentSize } from "./formatters";

export type DetectedFileKind = "image" | "audio" | "video" | "html" | "other";
export type DetectedFile = {
  id: string;
  normalizedPath: string;
  extension: string;
  kind: DetectedFileKind;
  previewUrls: string[];
};

type AttachmentLike = {
  name: string;
  size: number;
  localPath?: string;
};

type ConvertFileSrc = (filePath: string, protocol?: string) => string;

const chatDetectedFilesCache = new Map<string, DetectedFile[]>();

function setLimitedCacheEntry<T>(cache: Map<string, T>, key: string, value: T, maxEntries: number) {
  cache.set(key, value);
  if (cache.size <= maxEntries) {
    return value;
  }
  const firstKey = cache.keys().next().value;
  if (typeof firstKey === "string") {
    cache.delete(firstKey);
  }
  return value;
}

function stripTrailingPathTokens(rawPath: string) {
  let result = rawPath.trim();
  while (result.length > 0) {
    const tail = result.slice(-1);
    if (!CHAT_FILE_PATH_TRAILING_TOKENS.has(tail)) {
      break;
    }
    result = result.slice(0, -1);
  }
  return result;
}

export function normalizeDetectedFilePath(rawPath: string) {
  let normalized = rawPath.trim();
  if (
    normalized.length >= 2 &&
    ((normalized.startsWith('"') && normalized.endsWith('"')) || (normalized.startsWith("'") && normalized.endsWith("'")))
  ) {
    normalized = normalized.slice(1, -1).trim();
  }
  return stripTrailingPathTokens(normalized);
}

export function isAbsoluteLocalPath(path: string) {
  return path.startsWith("/") || /^[a-z]:[\\/]/i.test(path);
}

export function normalizeChatComposerFileExtension(fileName: string) {
  const normalized = fileName.trim().toLowerCase();
  const dotIndex = normalized.lastIndexOf(".");
  if (dotIndex < 0 || dotIndex >= normalized.length - 1) {
    return "";
  }
  return normalized.slice(dotIndex + 1);
}

export function resolveChatComposerFallbackFileName(file: File, index: number) {
  const mimeType = file.type.trim().toLowerCase();
  if (mimeType.startsWith("image/")) {
    const extension = mimeType.split("/")[1] || "png";
    return `粘贴图片-${Date.now()}-${index + 1}.${extension}`;
  }
  const extension = normalizeChatComposerFileExtension(file.name);
  if (extension) {
    return `附件-${Date.now()}-${index + 1}.${extension}`;
  }
  return `附件-${Date.now()}-${index + 1}`;
}

export function resolveChatComposerFileDisplayName(file: File, index: number) {
  const normalized = file.name.trim();
  if (normalized) {
    return normalized;
  }
  return resolveChatComposerFallbackFileName(file, index);
}

export function resolveChatComposerFileLocalPath(file: File) {
  const candidatePath = (file as File & { path?: unknown }).path;
  if (typeof candidatePath !== "string") {
    return "";
  }
  const normalized = normalizeDetectedFilePath(candidatePath);
  if (!normalized || !isAbsoluteLocalPath(normalized)) {
    return "";
  }
  return normalized;
}

export function resolveDetectedFileKindFromMimeType(mimeTypeRaw: string): DetectedFileKind {
  const mimeType = mimeTypeRaw.trim().toLowerCase();
  if (!mimeType) {
    return "other";
  }
  if (mimeType.startsWith("image/")) {
    return "image";
  }
  if (mimeType.startsWith("audio/")) {
    return "audio";
  }
  if (mimeType.startsWith("video/")) {
    return "video";
  }
  if (mimeType === "text/html" || mimeType === "application/xhtml+xml") {
    return "html";
  }
  return "other";
}

export function resolveDetectedFileKind(extension: string): DetectedFileKind {
  const normalized = extension.trim().toLowerCase();
  if (CHAT_FILE_IMAGE_EXTENSIONS.has(normalized)) {
    return "image";
  }
  if (CHAT_FILE_AUDIO_EXTENSIONS.has(normalized)) {
    return "audio";
  }
  if (CHAT_FILE_VIDEO_EXTENSIONS.has(normalized)) {
    return "video";
  }
  if (CHAT_FILE_HTML_EXTENSIONS.has(normalized)) {
    return "html";
  }
  return "other";
}

export function resolveChatComposerAttachmentDetectedKind(attachment: Pick<{ name: string; type: string }, "name" | "type">): DetectedFileKind {
  const fromMimeType = resolveDetectedFileKindFromMimeType(attachment.type);
  if (fromMimeType !== "other") {
    return fromMimeType;
  }
  const extension = normalizeChatComposerFileExtension(attachment.name);
  return extension ? resolveDetectedFileKind(extension) : "other";
}

export function normalizeChatComposerLocalPathCandidate(rawPath: string | null | undefined) {
  if (!rawPath) {
    return "";
  }
  const normalized = normalizeDetectedFilePath(rawPath);
  if (!normalized || !isAbsoluteLocalPath(normalized)) {
    return "";
  }
  return normalized;
}

export function sanitizeChatAttachmentWorkspaceHint(rawWorkspace: string | null | undefined) {
  const normalized = (rawWorkspace ?? "").trim();
  if (!normalized || normalized === "—" || normalized === "-") {
    return "";
  }
  return normalizeChatComposerLocalPathCandidate(normalized);
}

function encodePathForFileUrl(path: string) {
  return path
    .split("/")
    .map((segment, index) => {
      if (index === 0 && segment === "") {
        return "";
      }
      if (/^[a-z]:$/i.test(segment)) {
        return segment;
      }
      return encodeURIComponent(segment);
    })
    .join("/");
}

export function buildLocalFilePreviewUrls(path: string, convertFileSrc?: ConvertFileSrc | null) {
  const normalized = path.trim();
  const candidates: string[] = [];
  const seen = new Set<string>();
  const pushCandidate = (value: string | null | undefined) => {
    const candidate = (value ?? "").trim();
    if (!candidate || seen.has(candidate)) {
      return;
    }
    seen.add(candidate);
    candidates.push(candidate);
  };

  if (convertFileSrc) {
    try {
      pushCandidate(convertFileSrc(normalized));
    } catch {
      // Keep fallback previews below.
    }
    try {
      pushCandidate(convertFileSrc(normalized, "asset"));
    } catch {
      // Keep fallback previews below.
    }
  }

  if (normalized.startsWith("/")) {
    pushCandidate(`file://${encodePathForFileUrl(normalized)}`);
    pushCandidate(`file://${normalized}`);
    return candidates;
  }
  if (/^[a-z]:[\\/]/i.test(normalized)) {
    const windowsPath = normalized.replace(/\\/g, "/");
    pushCandidate(`file:///${encodePathForFileUrl(windowsPath)}`);
    pushCandidate(`file:///${windowsPath}`);
    return candidates;
  }
  return candidates;
}

function decodeFileUrlToPath(rawUrl: string) {
  const source = stripTrailingPathTokens(rawUrl.trim());
  if (!/^file:\/\//i.test(source)) {
    return null;
  }
  try {
    const url = new URL(source);
    if (url.protocol !== "file:") {
      return null;
    }
    let pathname = decodeURIComponent(url.pathname || "");
    if (/^\/[a-z]:/i.test(pathname)) {
      pathname = pathname.slice(1);
    }
    if (url.hostname && url.hostname !== "localhost") {
      const hostPath = pathname.startsWith("/") ? pathname : `/${pathname}`;
      return `//${url.hostname}${hostPath}`;
    }
    return pathname || null;
  } catch {
    const fallback = source.replace(/^file:\/\/+/, "/");
    try {
      return decodeURIComponent(fallback);
    } catch {
      return fallback;
    }
  }
}

export function extractDetectedFilesFromText(text: string, convertFileSrc?: ConvertFileSrc | null) {
  const normalizedText = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  if (!normalizedText.trim()) {
    return [] as DetectedFile[];
  }
  const cached = chatDetectedFilesCache.get(normalizedText);
  if (cached) {
    return cached;
  }

  const files: DetectedFile[] = [];
  const seen = new Set<string>();
  const addDetectedPath = (rawPath: string) => {
    const normalizedPath = normalizeDetectedFilePath(rawPath);
    if (!normalizedPath || !isAbsoluteLocalPath(normalizedPath)) {
      return;
    }
    const extension = normalizedPath.includes(".") ? normalizedPath.split(".").pop()?.toLowerCase() ?? "" : "";
    if (!extension) {
      return;
    }
    const id = normalizedPath.toLowerCase();
    if (seen.has(id)) {
      return;
    }
    seen.add(id);
    const kind = resolveDetectedFileKind(extension);
    files.push({
      id,
      normalizedPath,
      extension,
      kind,
      previewUrls: buildLocalFilePreviewUrls(normalizedPath, convertFileSrc)
    });
  };

  CHAT_FILE_UNIX_PATH_PATTERN.lastIndex = 0;
  let unixMatch: RegExpExecArray | null = null;
  while ((unixMatch = CHAT_FILE_UNIX_PATH_PATTERN.exec(normalizedText)) !== null) {
    if (unixMatch[1]) {
      addDetectedPath(unixMatch[1]);
    }
  }

  CHAT_FILE_WINDOWS_PATH_PATTERN.lastIndex = 0;
  let windowsMatch: RegExpExecArray | null = null;
  while ((windowsMatch = CHAT_FILE_WINDOWS_PATH_PATTERN.exec(normalizedText)) !== null) {
    if (windowsMatch[1]) {
      addDetectedPath(windowsMatch[1]);
    }
  }

  CHAT_FILE_URL_PATTERN.lastIndex = 0;
  let fileUrlMatch: RegExpExecArray | null = null;
  while ((fileUrlMatch = CHAT_FILE_URL_PATTERN.exec(normalizedText)) !== null) {
    const decodedPath = decodeFileUrlToPath(fileUrlMatch[1] ?? "");
    if (decodedPath) {
      addDetectedPath(decodedPath);
    }
  }

  return setLimitedCacheEntry(chatDetectedFilesCache, normalizedText, files, CHAT_FILE_DETECTION_CACHE_MAX);
}

export function formatAttachmentSummaryForPrompt(attachments: AttachmentLike[]) {
  if (attachments.length === 0) {
    return "";
  }
  const details = attachments.map((attachment) => {
    const base = `${attachment.name} (${formatAttachmentSize(attachment.size)})`;
    const localPath = normalizeChatComposerLocalPathCandidate(attachment.localPath);
    if (!localPath) {
      return base;
    }
    return `${base} [localPath: ${localPath}]`;
  });
  return `[附件: ${details.join("；")}]`;
}

export function isImageDetectedFile(file: Pick<DetectedFile, "kind">) {
  return file.kind === "image";
}

export function isAudioDetectedFile(file: Pick<DetectedFile, "kind">) {
  return file.kind === "audio";
}

export function isVideoDetectedFile(file: Pick<DetectedFile, "kind">) {
  return file.kind === "video";
}

export function isHtmlDetectedFile(file: Pick<DetectedFile, "kind">) {
  return file.kind === "html";
}
