import type { ChatMessageKind, ChatRole, ChatStatus, ChatToolStatus } from "../types";
import { isAbsoluteLocalPath, normalizeDetectedFilePath } from "../utils/filePreview";

export type MessageAttachment = {
  id: string;
  name: string;
  size: number;
  type: string;
  localPath?: string;
};

export function normalizeRole(value: unknown): ChatRole | null {
  return value === "assistant" || value === "user" || value === "system" ? value : null;
}

export function normalizeStatus(value: unknown): ChatStatus {
  if (value === "pending" || value === "done" || value === "error") {
    return value;
  }
  return "done";
}

export function normalizeToolStatus(value: unknown): ChatToolStatus | undefined {
  if (value === "running" || value === "done" || value === "error") {
    return value;
  }
  return undefined;
}

export function createChatAttachmentId() {
  return `att-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export function normalizeAttachment(raw: unknown, createAttachmentId: () => string = createChatAttachmentId): MessageAttachment | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }
  const candidate = raw as Partial<MessageAttachment>;
  const name = typeof candidate.name === "string" ? candidate.name.trim() : "";
  if (!name) {
    return null;
  }
  const size = typeof candidate.size === "number" && Number.isFinite(candidate.size) ? Math.max(0, candidate.size) : 0;
  const type = typeof candidate.type === "string" ? candidate.type : "";
  const rawLocalPath = typeof candidate.localPath === "string" ? normalizeDetectedFilePath(candidate.localPath) : "";
  const localPath = rawLocalPath && isAbsoluteLocalPath(rawLocalPath) ? rawLocalPath : undefined;
  return {
    id: typeof candidate.id === "string" && candidate.id.trim() ? candidate.id : createAttachmentId(),
    name,
    size,
    type,
    localPath
  };
}

export function normalizeMessageAttachments(raw: unknown, createAttachmentId: () => string = createChatAttachmentId) {
  if (!Array.isArray(raw)) {
    return undefined as MessageAttachment[] | undefined;
  }
  const list = raw
    .map((item) => normalizeAttachment(item, createAttachmentId))
    .filter((item): item is MessageAttachment => item !== null);
  return list.length > 0 ? list : undefined;
}

export function buildLocalAttachmentSignature(attachments: MessageAttachment[]) {
  if (attachments.length === 0) {
    return "";
  }
  return attachments
    .map((attachment) => `${attachment.name.trim().toLowerCase()}|${Math.max(0, Math.floor(attachment.size))}`)
    .sort()
    .join(";");
}

export function normalizeLocalUserMessageDedupText(text: string, attachments: MessageAttachment[]) {
  const normalizedText = text.replace(/\s+/g, " ").trim();
  if (normalizedText) {
    return normalizedText;
  }
  return attachments.length > 0 ? "(附件)" : "";
}

export function isLegacyWelcomeMessage(message: { id: string; role: ChatRole; text: string }) {
  if (message.id.startsWith("welcome-")) {
    return true;
  }
  if (message.role !== "assistant") {
    return false;
  }
  const text = message.text.trim();
  if (!text) {
    return false;
  }
  return text === "请选择一个 Agent 开始对话。" || (text.startsWith("已切换到 ") && text.endsWith("，可以直接发送消息并由该 Agent 执行。"));
}

export function isPendingChatMessage(message: { status: ChatStatus }) {
  return message.status === "pending";
}

export function isRuntimeToolMessage(message: { kind?: ChatMessageKind }) {
  return message.kind === "runtime_tool";
}

export function getMessageDisplayText(
  message: {
    role: ChatRole;
    text: string;
    attachments?: MessageAttachment[];
  },
  sanitizeText: (text: string) => string
) {
  if (message.role === "user" && message.text.trim() === "(附件)" && message.attachments && message.attachments.length > 0) {
    return "";
  }
  return sanitizeText(message.text);
}

export function buildOpenClawMessageContent(
  message: {
    role: ChatRole;
    text: string;
    attachments?: MessageAttachment[];
  },
  options: {
    sanitizeText: (text: string) => string;
    formatAttachmentSummary: (attachments: MessageAttachment[]) => string;
  }
) {
  const normalizedText = options.sanitizeText(message.text);
  if (message.role !== "user" || !message.attachments || message.attachments.length === 0) {
    return normalizedText;
  }
  const summary = options.formatAttachmentSummary(message.attachments);
  if (!summary) {
    return normalizedText;
  }
  const baseText = normalizedText === "(附件)" ? "" : normalizedText;
  return baseText ? `${baseText}\n\n${summary}` : summary;
}

export function isRecentDuplicateUserSubmit(params: {
  text: string;
  attachments: MessageAttachment[];
  messages: Array<{
    role: ChatRole;
    status: ChatStatus;
    createdAt: number;
    text: string;
    attachments?: MessageAttachment[];
  }>;
  nowMs: number;
  duplicateWindowMs: number;
}) {
  const { text, attachments, messages, nowMs, duplicateWindowMs } = params;
  const dedupText = normalizeLocalUserMessageDedupText(text, attachments);
  if (!dedupText) {
    return false;
  }
  const dedupAttachmentSignature = buildLocalAttachmentSignature(attachments);
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const item = messages[index];
    if (!item || item.role !== "user" || item.status !== "done") {
      continue;
    }
    const createdAt = typeof item.createdAt === "number" && Number.isFinite(item.createdAt) ? item.createdAt : 0;
    if (createdAt <= 0 || nowMs - createdAt > duplicateWindowMs) {
      if (createdAt > 0) {
        break;
      }
      continue;
    }
    const existingText = normalizeLocalUserMessageDedupText(item.text, item.attachments ?? []);
    const existingAttachmentSignature = buildLocalAttachmentSignature(item.attachments ?? []);
    if (existingText === dedupText && existingAttachmentSignature === dedupAttachmentSignature) {
      return true;
    }
  }
  return false;
}

export function normalizeMessage(raw: unknown, options: {
  createMessageId: (prefix: string) => string;
  sanitizeText: (text: string) => string;
  normalizeAttachments: (rawAttachments: unknown) => MessageAttachment[] | undefined;
}) {
  if (!raw || typeof raw !== "object") {
    return null as {
      id: string;
      role: ChatRole;
      text: string;
      status: ChatStatus;
      createdAt: number;
      attachments?: MessageAttachment[];
      kind?: ChatMessageKind;
      toolName?: string;
      toolStatus?: ChatToolStatus;
      toolInput?: string;
      toolOutput?: string;
    } | null;
  }

  const candidate = raw as {
    id?: unknown;
    role?: unknown;
    text?: unknown;
    status?: unknown;
    createdAt?: unknown;
    attachments?: unknown;
    kind?: unknown;
    toolName?: unknown;
    toolStatus?: unknown;
    toolInput?: unknown;
    toolOutput?: unknown;
  };
  const role = normalizeRole(candidate.role);
  if (!role || typeof candidate.text !== "string") {
    return null;
  }
  const kind: ChatMessageKind = candidate.kind === "runtime_tool" ? "runtime_tool" : "default";
  const toolName = typeof candidate.toolName === "string" && candidate.toolName.trim() ? candidate.toolName.trim() : undefined;
  const toolStatus = normalizeToolStatus(candidate.toolStatus);
  const toolInput = typeof candidate.toolInput === "string" && candidate.toolInput.trim() ? candidate.toolInput.trim() : undefined;
  const toolOutput =
    typeof candidate.toolOutput === "string" && candidate.toolOutput.trim() ? candidate.toolOutput.trim() : undefined;
  const attachments = options.normalizeAttachments(candidate.attachments);

  return {
    id: typeof candidate.id === "string" && candidate.id.trim() ? candidate.id : options.createMessageId("msg"),
    role,
    text: options.sanitizeText(candidate.text),
    status: normalizeStatus(candidate.status),
    createdAt: typeof candidate.createdAt === "number" && Number.isFinite(candidate.createdAt) ? candidate.createdAt : Date.now(),
    attachments,
    kind,
    toolName,
    toolStatus,
    toolInput,
    toolOutput
  };
}
