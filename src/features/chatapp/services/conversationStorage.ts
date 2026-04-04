import { CHAT_ARCHIVE_STORAGE_PREFIX, CHAT_STORAGE_PREFIX, SESSION_STORAGE_PREFIX } from "../constants";
import { safeStorageGet, safeStorageSet } from "./storage";

export type ConversationStorageReadOptions = {
  legacyAgentId?: string | null;
};

export function createSessionId() {
  return `session-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export function chatStorageKeyFor(conversationScopeKey: string) {
  return `${CHAT_STORAGE_PREFIX}.${conversationScopeKey}`;
}

export function sessionStorageKeyFor(conversationScopeKey: string) {
  return `${SESSION_STORAGE_PREFIX}.${conversationScopeKey}`;
}

export function chatArchiveStorageKeyFor(conversationScopeKey: string) {
  return `${CHAT_ARCHIVE_STORAGE_PREFIX}.${conversationScopeKey}`;
}

export function legacyChatStorageKeyForAgent(agentId: string) {
  return `${CHAT_STORAGE_PREFIX}.${agentId}`;
}

export function legacySessionStorageKeyForAgent(agentId: string) {
  return `${SESSION_STORAGE_PREFIX}.${agentId}`;
}

export function legacyChatArchiveStorageKeyForAgent(agentId: string) {
  return `${CHAT_ARCHIVE_STORAGE_PREFIX}.${agentId}`;
}

export function getStableChatMessages<T>(messages: T[], predicates: { isPending: (message: T) => boolean; isLegacy: (message: T) => boolean }) {
  return messages.filter((item) => !predicates.isPending(item) && !predicates.isLegacy(item));
}

export function loadChatHistory<T>(params: {
  conversationScopeKey: string;
  options?: ConversationStorageReadOptions;
  normalizeMessage: (raw: unknown) => T | null;
  isLegacyWelcomeMessage: (message: T) => boolean;
  isPending: (message: T) => boolean;
}) {
  const { conversationScopeKey, options = {}, normalizeMessage, isLegacyWelcomeMessage, isPending } = params;
  const primaryRaw = safeStorageGet(chatStorageKeyFor(conversationScopeKey));
  const legacyRaw = options.legacyAgentId ? safeStorageGet(legacyChatStorageKeyForAgent(options.legacyAgentId)) : null;
  const raw = primaryRaw && primaryRaw.trim() ? primaryRaw : legacyRaw;
  if (!raw) {
    return [] as T[];
  }

  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [] as T[];
    }

    return parsed
      .map((item) => normalizeMessage(item))
      .filter((item): item is T => item !== null)
      .filter((item) => !isPending(item) && !isLegacyWelcomeMessage(item));
  } catch {
    return [] as T[];
  }
}

export function loadChatArchives<T>(params: {
  conversationScopeKey: string;
  options?: ConversationStorageReadOptions;
  normalizeMessage: (raw: unknown) => T | null;
  isLegacyWelcomeMessage: (message: T) => boolean;
  isPending: (message: T) => boolean;
  createArchiveId: () => string;
  now?: () => number;
}) {
  const {
    conversationScopeKey,
    options = {},
    normalizeMessage,
    isLegacyWelcomeMessage,
    isPending,
    createArchiveId,
    now = () => Date.now()
  } = params;
  const primaryRaw = safeStorageGet(chatArchiveStorageKeyFor(conversationScopeKey));
  const legacyRaw = options.legacyAgentId ? safeStorageGet(legacyChatArchiveStorageKeyForAgent(options.legacyAgentId)) : null;
  const raw = primaryRaw && primaryRaw.trim() ? primaryRaw : legacyRaw;
  if (!raw) {
    return [] as Array<{ id: string; archivedAt: number; title: string; messages: T[] }>;
  }

  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [] as Array<{ id: string; archivedAt: number; title: string; messages: T[] }>;
    }
    return parsed
      .map((entry) => {
        if (!entry || typeof entry !== "object") {
          return null;
        }
        const candidate = entry as {
          id?: unknown;
          archivedAt?: unknown;
          title?: unknown;
          messages?: unknown[];
        };
        if (!Array.isArray(candidate.messages)) {
          return null;
        }
        const messages = candidate.messages
          .map((item) => normalizeMessage(item))
          .filter((item): item is T => item !== null)
          .filter((item) => !isPending(item) && !isLegacyWelcomeMessage(item));
        if (messages.length === 0) {
          return null;
        }
        return {
          id: typeof candidate.id === "string" && candidate.id.trim() ? candidate.id : createArchiveId(),
          archivedAt: typeof candidate.archivedAt === "number" && Number.isFinite(candidate.archivedAt) ? candidate.archivedAt : now(),
          title: typeof candidate.title === "string" && candidate.title.trim() ? candidate.title : "会话归档",
          messages
        };
      })
      .filter((item): item is { id: string; archivedAt: number; title: string; messages: T[] } => item !== null)
      .slice(0, 60);
  } catch {
    return [] as Array<{ id: string; archivedAt: number; title: string; messages: T[] }>;
  }
}

export function persistChatArchives<T extends { id: string; archivedAt: number; title: string }>(
  conversationScopeKey: string,
  archives: Array<T & { messages: unknown[] }>
) {
  safeStorageSet(chatArchiveStorageKeyFor(conversationScopeKey), JSON.stringify(archives.slice(0, 60)));
}

export function loadSessionId(
  conversationScopeKey: string,
  options: ConversationStorageReadOptions = {},
  createSessionIdFn: () => string = createSessionId
) {
  const key = sessionStorageKeyFor(conversationScopeKey);
  const existing = safeStorageGet(key);
  if (existing && existing.trim()) {
    return existing;
  }

  const legacyExisting = options.legacyAgentId ? safeStorageGet(legacySessionStorageKeyForAgent(options.legacyAgentId)) : null;
  if (legacyExisting && legacyExisting.trim()) {
    safeStorageSet(key, legacyExisting);
    return legacyExisting;
  }

  const next = createSessionIdFn();
  safeStorageSet(key, next);
  return next;
}

export function peekSessionId(conversationScopeKey: string, options: ConversationStorageReadOptions = {}) {
  const existing = safeStorageGet(sessionStorageKeyFor(conversationScopeKey));
  if (existing && existing.trim()) {
    return existing.trim();
  }
  const legacyExisting = options.legacyAgentId ? safeStorageGet(legacySessionStorageKeyForAgent(options.legacyAgentId)) : null;
  if (legacyExisting && legacyExisting.trim()) {
    return legacyExisting.trim();
  }
  return "";
}

export function persistChatHistory<T>(params: {
  conversationScopeKey: string;
  messages: T[];
  currentSessionId: string;
  createSessionIdFn?: () => string;
  isPending: (message: T) => boolean;
  isLegacyWelcomeMessage: (message: T) => boolean;
}) {
  const { conversationScopeKey, messages, currentSessionId, createSessionIdFn = createSessionId, isPending, isLegacyWelcomeMessage } = params;
  const stableMessages = messages.filter((item) => !isPending(item) && !isLegacyWelcomeMessage(item));
  safeStorageSet(chatStorageKeyFor(conversationScopeKey), JSON.stringify(stableMessages));
  safeStorageSet(sessionStorageKeyFor(conversationScopeKey), currentSessionId || createSessionIdFn());
}
