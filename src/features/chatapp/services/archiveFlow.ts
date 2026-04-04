import type { ChatRole } from "../types";

export type ArchiveRecord<T> = {
  id: string;
  archivedAt: number;
  title: string;
  messages: T[];
};

export function buildArchiveTitle<T extends { role: ChatRole; text: string }>(messages: T[]) {
  const firstUserMessage = messages.find((item) => item.role === "user" && item.text.trim());
  const firstAssistantMessage = messages.find((item) => item.role === "assistant" && item.text.trim());
  const base = firstUserMessage?.text.trim() || firstAssistantMessage?.text.trim() || "会话归档";
  const clipped = base.replace(/\s+/g, " ").trim();
  if (clipped.length > 22) {
    return `${clipped.slice(0, 22)}...`;
  }
  return clipped || "会话归档";
}

export function collectMeaningfulArchiveMessages<T extends { role: ChatRole }>(
  messages: T[],
  isRuntimeToolMessage: (message: T) => boolean
) {
  return messages.filter((item) => (item.role === "assistant" || item.role === "user") && !isRuntimeToolMessage(item));
}

export function createArchiveRecord<T extends object>(params: {
  messages: T[];
  createArchiveId: () => string;
  buildTitle?: (messages: T[]) => string;
  now?: () => number;
}) {
  const { messages, createArchiveId, buildTitle = () => "会话归档", now = () => Date.now() } = params;
  return {
    id: createArchiveId(),
    archivedAt: now(),
    title: buildTitle(messages),
    messages: messages.map((item) => ({ ...item }))
  } satisfies ArchiveRecord<T>;
}

export function prependArchiveRecord<T>(record: T, archives: T[], limit = 60) {
  return [record, ...archives].slice(0, limit);
}

export function cloneMessages<T extends object>(messages: T[]) {
  return messages.map((message) => ({ ...message }));
}
