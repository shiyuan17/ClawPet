import type { ConversationStorageReadOptions } from "./conversationStorage";

type ChannelSessionLike = {
  id?: string | null;
};

function equalsIgnoreCase(left: string | null | undefined, right: string | null | undefined) {
  return (left ?? "").trim().toLowerCase() === (right ?? "").trim().toLowerCase();
}

export function shouldSkipAgentSwitch(params: {
  previousAgentId: string | null | undefined;
  nextAgentId: string;
  previousChannelSession?: ChannelSessionLike | null;
  nextChannelSession?: ChannelSessionLike | null;
  force?: boolean;
}) {
  if (params.force) {
    return false;
  }
  const sameAgent = equalsIgnoreCase(params.previousAgentId, params.nextAgentId);
  const previousSessionId = params.previousChannelSession?.id ?? "";
  const nextSessionId = params.nextChannelSession?.id ?? "";
  return sameAgent && previousSessionId === nextSessionId;
}

export function resolveConversationReadOptions(params: {
  agentId: string;
  channelSession?: ChannelSessionLike | null;
  isConversationGroupAgent: boolean;
}): ConversationStorageReadOptions {
  if (params.channelSession) {
    return {};
  }
  if (params.isConversationGroupAgent) {
    return {};
  }
  return { legacyAgentId: params.agentId };
}

export function pickConversationHistory<T>(cachedHistory: T[] | undefined, loadedHistory: T[]) {
  return cachedHistory && cachedHistory.length > 0 ? cachedHistory : loadedHistory;
}

export function shouldResetSelectedAgent(selectedAgentId: string | null | undefined, loadedAgentIds: string[]) {
  if (!selectedAgentId) {
    return true;
  }
  return !loadedAgentIds.some((id) => equalsIgnoreCase(id, selectedAgentId));
}

export function resolveFreshSessionId(params: {
  isAgentScope: boolean;
  conversationScopeKey: string;
  agentId: string;
  buildScopedRuntimeSessionKey: (conversationScopeKey: string, agentId: string) => string;
  createSessionId: () => string;
}) {
  if (params.isAgentScope) {
    return params.buildScopedRuntimeSessionKey(params.conversationScopeKey, params.agentId);
  }
  return params.createSessionId();
}

export function cloneConversationMessages<T>(messages: T[]) {
  return [...messages];
}
