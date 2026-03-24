<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import { sendOpenClawChat, type OpenClawMessage } from "../services/openclaw";
import { loadMemories, loadStaff, loadTasks, type MemoryRecord, type TaskRecord } from "../services/consoleData";

type SidebarSection = "chat" | "dashboard" | "staff" | "recruitment" | "skills" | "tasks";
type AgentGroupKind = "staff" | "group";
type AgentStatusTone = "online" | "busy" | "offline";
type ChatRole = "assistant" | "user" | "system";
type ChatStatus = "pending" | "done" | "error";
type AgentPaneTab = "staff" | "group";
type RelatedResourceTarget = "memory" | "skills" | "tools" | "channel" | "schedule";
type RelatedSkillCategory = "builtIn" | "installed";
type UtilityModalType = "history" | "logs";

type SidebarItem = {
  id: SidebarSection;
  label: string;
};

type StaffMemberSnapshot = {
  agentId: string;
  displayName: string;
  roleLabel: string;
  channel?: string;
  model: string;
  workspace: string;
  toolsProfile: string;
  toolsEnabledCount?: number;
  statusLabel: string;
  currentWorkLabel: string;
  currentWork: string;
  recentOutput: string;
  scheduledLabel: string;
};

type StaffSnapshotResponse = {
  missionStatement: string;
  sourcePath: string;
  detail: string;
  members: StaffMemberSnapshot[];
};

type AgentListItem = {
  agentId: string;
  displayName: string;
  roleLabel: string;
  channel: string;
  model: string;
  workspace: string;
  toolsProfile: string;
  toolsEnabledCount: number | null;
  statusLabel: string;
  statusTone: AgentStatusTone;
  currentWorkLabel: string;
  currentWork: string;
  recentOutput: string;
  scheduledLabel: string;
  groupKind: AgentGroupKind;
};

type AgentChatMessage = {
  id: string;
  role: ChatRole;
  text: string;
  status: ChatStatus;
  createdAt: number;
};

type AgentChatMeta = {
  preview: string;
  timeLabel: string;
  unread: number;
};

type SourceFileSnapshotItem = {
  id: string;
  title: string;
  summary: string;
  content: string;
  sourcePath: string;
  relativePath: string;
  facetKey: string;
  facetLabel: string;
  category: string;
  updatedAtMs: number;
  exists: boolean;
};

type SourceFileSnapshotResponse = {
  sourcePath: string;
  detail: string;
  items: SourceFileSnapshotItem[];
};

type OpenClawSkillListItem = {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  relativePath: string;
  sourcePath: string;
};

type OpenClawSkillsListResponse = {
  sourcePath: string;
  builtIn: OpenClawSkillListItem[];
  installed: OpenClawSkillListItem[];
};

type OpenClawToolListItem = {
  id: string;
  name: string;
  description: string;
  category: string;
  enabled: boolean;
};

type OpenClawToolsListResponse = {
  profile: string;
  profileLabel: string;
  tools: OpenClawToolListItem[];
};

type OpenClawChannelAccountSnapshotItem = {
  accountId: string;
  name: string;
  configured: boolean;
  status: string;
  isDefault: boolean;
  agentId: string | null;
};

type OpenClawChannelGroupSnapshotItem = {
  channelType: string;
  defaultAccountId: string;
  status: string;
  accounts: OpenClawChannelAccountSnapshotItem[];
};

type OpenClawChannelAccountsSnapshotResponse = {
  sourcePath: string;
  detail: string;
  channels: OpenClawChannelGroupSnapshotItem[];
};

type TaskSnapshotItem = {
  id: string;
  name: string;
  agentId: string;
  sessionTarget: string;
  enabled: boolean;
  deleteAfterRun: boolean;
  statusKind: string;
  statusLabel: string;
  summary: string;
  nextRunAtMs: number | null;
  createdAtMs: number | null;
  updatedAtMs: number | null;
  scheduleKind: string;
};

type TaskSnapshotResponse = {
  sourcePath: string;
  detail: string;
  jobs: TaskSnapshotItem[];
};

type OpenClawMessageLogItem = {
  id: string;
  platformName: string;
  method: string;
  endpoint: string;
  path?: string;
  responseStatus: number;
  duration: number;
  createdAt: number;
  error?: string;
};

type OpenClawMessageLogResponse = {
  detail: string;
  logs: OpenClawMessageLogItem[];
};

type ChatArchiveRecord = {
  id: string;
  archivedAt: number;
  title: string;
  messages: AgentChatMessage[];
};

type TauriInvoke = (command: string, args?: Record<string, unknown>) => Promise<unknown>;
type TauriWindowApi = {
  close?: () => Promise<void> | void;
  minimize?: () => Promise<void> | void;
  maximize?: () => Promise<void> | void;
  unmaximize?: () => Promise<void> | void;
  toggleMaximize?: () => Promise<void> | void;
  isMaximized?: () => Promise<boolean> | boolean;
  toggleFullscreen?: () => Promise<void> | void;
  setFullscreen?: (value: boolean) => Promise<void> | void;
  isFullscreen?: () => Promise<boolean> | boolean;
};
type TauriNamespace = {
  core?: {
    invoke?: TauriInvoke;
  };
  window?: {
    getCurrentWindow?: () => TauriWindowApi;
  };
};

const sidebarItems: SidebarItem[] = [
  { id: "chat", label: "聊天" },
  { id: "dashboard", label: "仪表盘" },
  { id: "staff", label: "员工管理" },
  { id: "recruitment", label: "员工招募" },
  { id: "skills", label: "技能市场" },
  { id: "tasks", label: "任务管理" }
];
const agentPaneTabs: Array<{ id: AgentPaneTab; label: string }> = [
  { id: "staff", label: "角色" },
  { id: "group", label: "群组" }
];

const CHAT_STORAGE_PREFIX = "keai.desktop-pet.openclaw.chat-history";
const SESSION_STORAGE_PREFIX = "keai.desktop-pet.openclaw.session-id";
const CHAT_ARCHIVE_STORAGE_PREFIX = "keai.desktop-pet.openclaw.chat-archives";

const activeSection = ref<SidebarSection>("chat");
const activeAgentPaneTab = ref<AgentPaneTab>("staff");
const searchQuery = ref("");
const chatInput = ref("");
const isSending = ref(false);
const agents = ref<AgentListItem[]>([]);
const selectedAgentId = ref<string | null>(null);
const chatMessages = ref<AgentChatMessage[]>([]);
const agentHistories = ref<Record<string, AgentChatMessage[]>>({});
const agentMetaMap = ref<Record<string, AgentChatMeta>>({});
const currentSessionId = ref("");
const missionStatement = ref("构建可持续自治的 AI 员工体系，持续完成高价值任务。");
const staffSourceDetail = ref("正在读取 Agent 列表...");
const messageScroller = ref<HTMLElement | null>(null);
const taskItems = ref<TaskRecord[]>([]);
const isAgentSettingsOpen = ref(false);
const relatedResourceModalTarget = ref<RelatedResourceTarget | null>(null);
const relatedResourceModalLoading = ref(false);
const relatedResourceModalSaving = ref(false);
const relatedResourceModalError = ref("");
const relatedResourceModalNotice = ref("");
const relatedMemorySnapshot = ref<SourceFileSnapshotResponse | null>(null);
const relatedSkillsSnapshot = ref<OpenClawSkillsListResponse | null>(null);
const relatedToolsSnapshot = ref<OpenClawToolsListResponse | null>(null);
const relatedChannelSnapshot = ref<OpenClawChannelAccountsSnapshotResponse | null>(null);
const relatedTaskSnapshot = ref<TaskSnapshotResponse | null>(null);
const relatedSkillCategory = ref<RelatedSkillCategory>("builtIn");
const relatedSkillSearch = ref("");
const relatedMemorySearch = ref("");
const relatedMemorySelectedId = ref<string | null>(null);
const relatedMemoryDraftContent = ref("");
const utilityModalType = ref<UtilityModalType | null>(null);
const utilityModalLoading = ref(false);
const utilityModalError = ref("");
const utilityModalNotice = ref("");
const chatHistoryArchives = ref<ChatArchiveRecord[]>([]);
const chatRuntimeLogs = ref<OpenClawMessageLogResponse | null>(null);

function getStorage() {
  if (typeof window === "undefined") {
    return null;
  }
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function safeStorageGet(key: string) {
  try {
    return getStorage()?.getItem(key) ?? null;
  } catch {
    return null;
  }
}

function safeStorageSet(key: string, value: string) {
  try {
    getStorage()?.setItem(key, value);
  } catch {
    // Ignore storage failures.
  }
}

function getTauriNamespace(): TauriNamespace | null {
  if (typeof window === "undefined") {
    return null;
  }
  return (window as Window & { __TAURI__?: TauriNamespace }).__TAURI__ ?? null;
}

function getTauriInvoke(): TauriInvoke | null {
  return getTauriNamespace()?.core?.invoke ?? null;
}

function getTauriWindow(): TauriWindowApi | null {
  return getTauriNamespace()?.window?.getCurrentWindow?.() ?? null;
}

function createMessageId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

function createSessionId() {
  return `session-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function chatStorageKeyFor(agentId: string) {
  return `${CHAT_STORAGE_PREFIX}.${agentId}`;
}

function sessionStorageKeyFor(agentId: string) {
  return `${SESSION_STORAGE_PREFIX}.${agentId}`;
}

function chatArchiveStorageKeyFor(agentId: string) {
  return `${CHAT_ARCHIVE_STORAGE_PREFIX}.${agentId}`;
}

function stripRoleLabel(name: string) {
  return name.replace(/[（(][^）)]*[）)]$/, "").trim();
}

function inferStatusTone(statusLabel: string): AgentStatusTone {
  const value = statusLabel.trim().toLowerCase();
  if (!value) {
    return "online";
  }
  if (value.includes("offline") || value.includes("离线") || value.includes("断开")) {
    return "offline";
  }
  if (value.includes("busy") || value.includes("运行") || value.includes("执行") || value.includes("处理中") || value.includes("调度")) {
    return "busy";
  }
  return "online";
}

function inferGroupKind(displayName: string, roleLabel: string): AgentGroupKind {
  const text = `${displayName} ${roleLabel}`.toLowerCase();
  if (text.includes("群") || text.includes("group")) {
    return "group";
  }
  return "staff";
}

function formatTimeLabel(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  });
}

function normalizeRole(value: unknown): ChatRole | null {
  return value === "assistant" || value === "user" || value === "system" ? value : null;
}

function normalizeStatus(value: unknown): ChatStatus {
  if (value === "pending" || value === "done" || value === "error") {
    return value;
  }
  return "done";
}

function normalizeMessage(raw: unknown): AgentChatMessage | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }

  const candidate = raw as Partial<AgentChatMessage>;
  const role = normalizeRole(candidate.role);
  if (!role || typeof candidate.text !== "string") {
    return null;
  }

  return {
    id: typeof candidate.id === "string" && candidate.id.trim() ? candidate.id : createMessageId("msg"),
    role,
    text: candidate.text,
    status: normalizeStatus(candidate.status),
    createdAt: typeof candidate.createdAt === "number" && Number.isFinite(candidate.createdAt) ? candidate.createdAt : Date.now()
  };
}

function createWelcomeMessages(agent: AgentListItem | null): AgentChatMessage[] {
  return [
    {
      id: createMessageId("welcome"),
      role: "assistant",
      status: "done",
      text: agent
        ? `已切换到 ${stripRoleLabel(agent.displayName)}，可以直接发送消息并由该 Agent 执行。`
        : "请选择一个 Agent 开始对话。",
      createdAt: Date.now()
    }
  ];
}

function loadChatHistory(agentId: string) {
  const raw = safeStorageGet(chatStorageKeyFor(agentId));
  if (!raw) {
    return [] as AgentChatMessage[];
  }

  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [] as AgentChatMessage[];
    }

    return parsed
      .map((item) => normalizeMessage(item))
      .filter((item): item is AgentChatMessage => item !== null)
      .filter((item) => item.status !== "pending");
  } catch {
    return [] as AgentChatMessage[];
  }
}

function loadChatArchives(agentId: string) {
  const raw = safeStorageGet(chatArchiveStorageKeyFor(agentId));
  if (!raw) {
    return [] as ChatArchiveRecord[];
  }
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [] as ChatArchiveRecord[];
    }
    return parsed
      .map((entry) => {
        if (!entry || typeof entry !== "object") {
          return null;
        }
        const candidate = entry as Partial<ChatArchiveRecord> & { messages?: unknown[] };
        if (!Array.isArray(candidate.messages)) {
          return null;
        }
        const messages = candidate.messages
          .map((item) => normalizeMessage(item))
          .filter((item): item is AgentChatMessage => item !== null)
          .filter((item) => item.status !== "pending");
        if (messages.length === 0) {
          return null;
        }
        return {
          id: typeof candidate.id === "string" && candidate.id.trim() ? candidate.id : createMessageId("archive"),
          archivedAt:
            typeof candidate.archivedAt === "number" && Number.isFinite(candidate.archivedAt) ? candidate.archivedAt : Date.now(),
          title: typeof candidate.title === "string" && candidate.title.trim() ? candidate.title : "会话归档",
          messages
        } satisfies ChatArchiveRecord;
      })
      .filter((item): item is ChatArchiveRecord => item !== null)
      .slice(0, 60);
  } catch {
    return [] as ChatArchiveRecord[];
  }
}

function persistChatArchives(agentId: string, archives: ChatArchiveRecord[]) {
  safeStorageSet(chatArchiveStorageKeyFor(agentId), JSON.stringify(archives.slice(0, 60)));
}

function loadSessionId(agentId: string) {
  const key = sessionStorageKeyFor(agentId);
  const existing = safeStorageGet(key);
  if (existing && existing.trim()) {
    return existing;
  }

  const next = createSessionId();
  safeStorageSet(key, next);
  return next;
}

function persistChatHistory(agentId: string) {
  const stableMessages = chatMessages.value.filter((item) => item.status !== "pending");
  safeStorageSet(chatStorageKeyFor(agentId), JSON.stringify(stableMessages));
  safeStorageSet(sessionStorageKeyFor(agentId), currentSessionId.value || createSessionId());
}

function getOpenClawMessages(items: AgentChatMessage[]): OpenClawMessage[] {
  return items
    .filter((item) => item.status !== "pending")
    .filter((item) => item.role === "assistant" || item.role === "user" || item.role === "system")
    .map((item) => ({ role: item.role, content: item.text }));
}

function getAgentInitial(agent: AgentListItem) {
  const name = stripRoleLabel(agent.displayName);
  return name.charAt(0).toUpperCase() || "A";
}

function getAgentMeta(agentId: string): AgentChatMeta {
  return (
    agentMetaMap.value[agentId] ?? {
      preview: "暂无会话",
      timeLabel: "--:--",
      unread: 0
    }
  );
}

function setAgentMeta(agentId: string, patch: Partial<AgentChatMeta>) {
  const current = getAgentMeta(agentId);
  agentMetaMap.value = {
    ...agentMetaMap.value,
    [agentId]: {
      ...current,
      ...patch
    }
  };
}

function refreshAgentMetaFromHistory(agentId: string, messages: AgentChatMessage[], fallback: string) {
  const stable = messages.filter((item) => item.status !== "pending");
  const latest = stable[stable.length - 1];
  if (!latest) {
    setAgentMeta(agentId, {
      preview: fallback || "暂无会话",
      timeLabel: "--:--"
    });
    return;
  }

  const latestPreview = latest.text.trim() || fallback || "暂无会话";
  setAgentMeta(agentId, {
    preview: latestPreview,
    timeLabel: formatTimeLabel(latest.createdAt)
  });
}

function mapSnapshotMember(member: StaffMemberSnapshot): AgentListItem {
  const statusLabel = member.statusLabel || "在线";
  return {
    agentId: member.agentId,
    displayName: member.displayName || member.agentId,
    roleLabel: member.roleLabel || "未标注角色",
    channel: member.channel || "main",
    model: member.model || "llm/petclaw-1.0",
    workspace: member.workspace || "—",
    toolsProfile: member.toolsProfile || "default",
    toolsEnabledCount: typeof member.toolsEnabledCount === "number" ? member.toolsEnabledCount : null,
    statusLabel,
    statusTone: inferStatusTone(statusLabel),
    currentWorkLabel: member.currentWorkLabel || "正在处理什么",
    currentWork: member.currentWork || member.currentWorkLabel || "暂无工作描述",
    recentOutput: member.recentOutput || "",
    scheduledLabel: member.scheduledLabel || "",
    groupKind: inferGroupKind(member.displayName || member.agentId, member.roleLabel || "")
  };
}

function mapFallbackMember() {
  const fallbackMembers = loadStaff();

  return fallbackMembers.map((member) => {
    const statusLabel = member.status === "busy" ? "忙碌" : member.status === "offline" ? "离线" : "在线";
    return {
      agentId: member.id,
      displayName: member.name,
      roleLabel: member.role,
      channel: "local",
      model: "llm/petclaw-1.0",
      workspace: member.workspace || "control-center",
      toolsProfile: "default",
      toolsEnabledCount: null,
      statusLabel,
      statusTone: inferStatusTone(statusLabel),
      currentWorkLabel: "正在处理什么",
      currentWork: member.focus || "暂无工作描述",
      recentOutput: "",
      scheduledLabel: member.status === "offline" ? "未排班" : "值班中",
      groupKind: inferGroupKind(member.name, member.role)
    } satisfies AgentListItem;
  });
}

async function loadAgents() {
  const invoke = getTauriInvoke();
  let loadedAgents: AgentListItem[] = [];

  if (invoke) {
    try {
      const result = (await invoke("load_staff_snapshot")) as StaffSnapshotResponse;
      missionStatement.value = result.missionStatement || missionStatement.value;
      staffSourceDetail.value = result.detail || "Agent 列表已更新。";
      loadedAgents = Array.isArray(result.members) ? result.members.map(mapSnapshotMember) : [];
    } catch (error) {
      staffSourceDetail.value = error instanceof Error ? error.message : "读取 Agent 列表失败，已切换本地数据。";
    }
  }

  if (loadedAgents.length === 0) {
    loadedAgents = mapFallbackMember();
    if (!invoke) {
      staffSourceDetail.value = "当前环境不支持 runtime staff snapshot，已使用本地员工数据。";
    }
  }

  loadedAgents.sort((left, right) => {
    const toneWeight = { online: 0, busy: 1, offline: 2 } as const;
    const toneDiff = toneWeight[left.statusTone] - toneWeight[right.statusTone];
    if (toneDiff !== 0) {
      return toneDiff;
    }
    return stripRoleLabel(left.displayName).localeCompare(stripRoleLabel(right.displayName), "zh-CN");
  });

  agents.value = loadedAgents;

  for (const agent of loadedAgents) {
    const history = loadChatHistory(agent.agentId);
    if (history.length > 0) {
      agentHistories.value[agent.agentId] = history;
    }
    refreshAgentMetaFromHistory(agent.agentId, history, agent.currentWork);
  }

  if (!selectedAgentId.value || !loadedAgents.some((agent) => agent.agentId === selectedAgentId.value)) {
    switchAgent(loadedAgents[0]?.agentId ?? null);
  }
}

async function scrollMessagesToBottom() {
  await nextTick();
  if (messageScroller.value) {
    messageScroller.value.scrollTop = messageScroller.value.scrollHeight;
  }
}

function switchAgent(agentId: string | null) {
  if (!agentId || agentId === selectedAgentId.value) {
    return;
  }

  const previousAgentId = selectedAgentId.value;
  if (previousAgentId) {
    agentHistories.value[previousAgentId] = [...chatMessages.value];
    persistChatHistory(previousAgentId);
  }

  selectedAgentId.value = agentId;
  currentSessionId.value = loadSessionId(agentId);

  const cachedHistory = agentHistories.value[agentId];
  const loadedHistory = cachedHistory && cachedHistory.length > 0 ? cachedHistory : loadChatHistory(agentId);

  const active = agents.value.find((item) => item.agentId === agentId) ?? null;
  chatMessages.value = loadedHistory.length > 0 ? [...loadedHistory] : createWelcomeMessages(active);

  setAgentMeta(agentId, { unread: 0 });
  if (utilityModalType.value) {
    void refreshUtilityModalData(utilityModalType.value);
  }
  void scrollMessagesToBottom();
}

function handleNewChat() {
  const activeId = selectedAgentId.value;
  if (!activeId) {
    return;
  }

  const active = agents.value.find((item) => item.agentId === activeId) ?? null;
  chatMessages.value = createWelcomeMessages(active);
  persistChatHistory(activeId);
  refreshAgentMetaFromHistory(activeId, chatMessages.value, active?.currentWork || "暂无会话");
  void scrollMessagesToBottom();
}

async function submitChat() {
  const text = chatInput.value.trim();
  const activeAgent = agents.value.find((item) => item.agentId === selectedAgentId.value) ?? null;

  if (!text || isSending.value || !activeAgent) {
    return;
  }

  const history = getOpenClawMessages(chatMessages.value);
  const startedAt = Date.now();
  const pendingId = createMessageId("assistant");

  chatMessages.value.push({
    id: createMessageId("user"),
    role: "user",
    text,
    status: "done",
    createdAt: startedAt
  });

  chatMessages.value.push({
    id: pendingId,
    role: "assistant",
    text: `${stripRoleLabel(activeAgent.displayName)} 正在思考中...`,
    status: "pending",
    createdAt: Date.now()
  });

  chatInput.value = "";
  setAgentMeta(activeAgent.agentId, {
    preview: `你：${text}`,
    timeLabel: formatTimeLabel(startedAt),
    unread: 0
  });

  isSending.value = true;
  void scrollMessagesToBottom();

  try {
    const response = await sendOpenClawChat([...history, { role: "user", content: text }], { agentId: activeAgent.agentId });
    const doneAt = Date.now();

    const pendingMessage = chatMessages.value.find((item) => item.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = response.text;
      pendingMessage.status = "done";
      pendingMessage.createdAt = doneAt;
    }

    setAgentMeta(activeAgent.agentId, {
      preview: response.text.trim() || "已回复",
      timeLabel: formatTimeLabel(doneAt),
      unread: 0
    });
  } catch (error) {
    const failedAt = Date.now();
    const pendingMessage = chatMessages.value.find((item) => item.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = error instanceof Error ? error.message : "Agent 回复失败。";
      pendingMessage.status = "error";
      pendingMessage.createdAt = failedAt;
    }

    setAgentMeta(activeAgent.agentId, {
      preview: "消息发送失败",
      timeLabel: formatTimeLabel(failedAt),
      unread: 0
    });
  } finally {
    isSending.value = false;
    persistChatHistory(activeAgent.agentId);
    agentHistories.value[activeAgent.agentId] = [...chatMessages.value];
    void scrollMessagesToBottom();
  }
}

async function handleWindowClose() {
  const tauriWindow = getTauriWindow();
  if (tauriWindow?.close) {
    try {
      await tauriWindow.close();
      return;
    } catch {
      // Continue fallback flow.
    }
  }

  const invoke = getTauriInvoke();
  if (invoke) {
    try {
      await invoke("quit_app");
      return;
    } catch {
      // Continue fallback flow.
    }
  }

  if (typeof window !== "undefined") {
    window.close();
  }
}

async function handleWindowMinimize() {
  const tauriWindow = getTauriWindow();
  if (!tauriWindow?.minimize) {
    return;
  }
  try {
    await tauriWindow.minimize();
  } catch {
    // Ignore runtime minimize errors.
  }
}

async function handleWindowExpand() {
  const tauriWindow = getTauriWindow();
  if (!tauriWindow) {
    return;
  }

  try {
    if (tauriWindow.toggleFullscreen) {
      await tauriWindow.toggleFullscreen();
      return;
    }

    if (tauriWindow.isFullscreen && tauriWindow.setFullscreen) {
      const isFullscreen = await tauriWindow.isFullscreen();
      await tauriWindow.setFullscreen(!isFullscreen);
      return;
    }

    if (tauriWindow.toggleMaximize) {
      await tauriWindow.toggleMaximize();
      return;
    }

    if (tauriWindow.isMaximized && tauriWindow.unmaximize && tauriWindow.maximize) {
      const isMaximized = await tauriWindow.isMaximized();
      if (isMaximized) {
        await tauriWindow.unmaximize();
      } else {
        await tauriWindow.maximize();
      }
      return;
    }

    if (tauriWindow.maximize) {
      await tauriWindow.maximize();
    }
  } catch {
    // Ignore runtime maximize errors.
  }
}

async function handleWindowDragStart() {
  const invoke = getTauriInvoke();
  if (!invoke) {
    return;
  }
  try {
    await invoke("start_main_window_drag");
  } catch {
    // Ignore runtime drag errors.
  }
}

function isInteractiveDragTarget(target: EventTarget | null) {
  if (!(target instanceof Element)) {
    return false;
  }
  return Boolean(
    target.closest(
      "button, input, textarea, select, a, label, [role='button'], [data-no-window-drag], [contenteditable='true']"
    )
  );
}

function handleRegionMouseDown(event: MouseEvent) {
  if (event.button !== 0) {
    return;
  }
  if (isInteractiveDragTarget(event.target)) {
    return;
  }
  event.preventDefault();
  void handleWindowDragStart();
}

function getMessageTimeLabel(message: AgentChatMessage) {
  return formatTimeLabel(message.createdAt);
}

function getAgentStatusLabel(agent: AgentListItem) {
  if (!agent.statusLabel.trim()) {
    return "在线";
  }
  return agent.statusLabel;
}

function getAgentScheduledLabel(agent: AgentListItem) {
  if (agent.scheduledLabel.trim()) {
    return agent.scheduledLabel;
  }
  return agent.statusTone === "offline" ? "未排班" : "值班中";
}

function getAgentCurrentWorkLabel(agent: AgentListItem) {
  return agent.currentWorkLabel.trim() || "正在处理什么";
}

function getAgentRecentOutput(agent: AgentListItem) {
  const trimmed = agent.recentOutput.trim();
  return trimmed || "最近暂无产出。";
}

function getAgentToolsEnabledLabel(agent: AgentListItem) {
  if (typeof agent.toolsEnabledCount === "number" && Number.isFinite(agent.toolsEnabledCount)) {
    return String(agent.toolsEnabledCount);
  }
  return "—";
}

function toggleAgentSettingsPanel() {
  isAgentSettingsOpen.value = !isAgentSettingsOpen.value;
}

function closeAgentSettingsPanel() {
  isAgentSettingsOpen.value = false;
}

function equalsIgnoreCase(left: string | null | undefined, right: string | null | undefined) {
  return (left ?? "").trim().toLowerCase() === (right ?? "").trim().toLowerCase();
}

function formatDateTime(timestampMs: number | null | undefined) {
  if (!timestampMs || !Number.isFinite(timestampMs)) {
    return "—";
  }
  return new Date(timestampMs).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  });
}

function clearRelatedResourceSnapshots() {
  relatedMemorySnapshot.value = null;
  relatedSkillsSnapshot.value = null;
  relatedToolsSnapshot.value = null;
  relatedChannelSnapshot.value = null;
  relatedTaskSnapshot.value = null;
}

function getStableChatMessages(messages: AgentChatMessage[]) {
  return messages.filter((item) => item.status !== "pending");
}

function buildArchiveTitle(messages: AgentChatMessage[]) {
  const firstUserMessage = messages.find((item) => item.role === "user" && item.text.trim());
  const firstAssistantMessage = messages.find((item) => item.role === "assistant" && item.text.trim());
  const base = firstUserMessage?.text.trim() || firstAssistantMessage?.text.trim() || "会话归档";
  const clipped = base.replace(/\s+/g, " ").trim();
  if (clipped.length > 22) {
    return `${clipped.slice(0, 22)}...`;
  }
  return clipped || "会话归档";
}

function getArchivePreviewText(record: ChatArchiveRecord) {
  const latest = record.messages[record.messages.length - 1];
  if (!latest) {
    return "暂无内容";
  }
  const text = latest.text.replace(/\s+/g, " ").trim();
  if (!text) {
    return "暂无内容";
  }
  if (text.length > 64) {
    return `${text.slice(0, 64)}...`;
  }
  return text;
}

function syncRelatedMemorySelection(preferredId: string | null = null) {
  const items = relatedMemoryItems.value;
  if (items.length === 0) {
    relatedMemorySelectedId.value = null;
    relatedMemoryDraftContent.value = "";
    return;
  }

  const nextSelectedId = preferredId || relatedMemorySelectedId.value;
  const selected =
    (nextSelectedId ? items.find((item) => item.id === nextSelectedId) : null) ??
    items[0];

  relatedMemorySelectedId.value = selected.id;
  relatedMemoryDraftContent.value = selected.content;
}

function getLogStatusTone(status: number) {
  if (status >= 500) {
    return "error";
  }
  if (status >= 400) {
    return "warn";
  }
  return "ok";
}

function formatDurationLabel(durationMs: number) {
  if (!Number.isFinite(durationMs)) {
    return "—";
  }
  if (durationMs >= 1000) {
    return `${(durationMs / 1000).toFixed(2)}s`;
  }
  return `${Math.max(0, Math.round(durationMs))}ms`;
}

const MEMORY_FILE_NAME_EXPLANATION_MAP: Record<string, string> = {
  "AGENTS.MD": "员工工作规则",
  "MEMORY.MD": "长期记忆",
  "IDENTITY.MD": "身份定义",
  "USER.MD": "用户身份定义",
  "TOOLS.MD": "工具使用规范",
  "SOUL.MD": "性格与价值观",
  "HEARTBEAT.MD": "状态心跳记录",
  "BOOTSTRAP.MD": "启动说明"
};

function getFileNameFromPath(pathValue: string) {
  const normalized = pathValue.replace(/\\/g, "/").trim();
  if (!normalized) {
    return "";
  }
  const segments = normalized.split("/").filter(Boolean);
  return segments[segments.length - 1] ?? normalized;
}

function getMemoryDisplayName(item: SourceFileSnapshotItem) {
  const fileName = getFileNameFromPath(item.relativePath || item.sourcePath || item.title) || item.title || "记忆文件";
  const explanation = MEMORY_FILE_NAME_EXPLANATION_MAP[fileName.toUpperCase()];
  if (explanation) {
    return `${fileName} · ${explanation}`;
  }

  const fallbackTitle = item.title.trim();
  if (fallbackTitle && fallbackTitle.toUpperCase() !== fileName.toUpperCase()) {
    return `${fileName} · ${fallbackTitle}`;
  }
  return fileName;
}

function buildMemoryFallbackSnapshot(items: MemoryRecord[]): SourceFileSnapshotResponse {
  return {
    sourcePath: "localStorage:keai.desktop-pet.memory",
    detail: "当前环境未启用 runtime 记忆快照，以下为本地记忆数据。",
    items: items.map((item) => ({
      id: item.id,
      title: item.title,
      summary: item.summary,
      content: item.summary,
      sourcePath: "localStorage",
      relativePath: `${item.scope}/${item.title}`,
      facetKey: item.scope,
      facetLabel: item.scope,
      category: item.scope,
      updatedAtMs: item.updatedAt,
      exists: true
    }))
  };
}

function buildTaskFallbackSnapshot(items: TaskRecord[]): TaskSnapshotResponse {
  const jobs: TaskSnapshotItem[] = items.map((item) => ({
    id: item.id,
    name: item.title,
    agentId: item.owner,
    sessionTarget: item.project,
    enabled: item.status !== "done",
    deleteAfterRun: item.status === "done",
    statusKind: item.status === "blocked" ? "late" : item.status === "done" ? "disabled" : "scheduled",
    statusLabel: item.status === "blocked" ? "待执行" : item.status === "done" ? "已停用" : "已启用",
    summary: item.summary,
    nextRunAtMs: item.dueAt,
    createdAtMs: item.updatedAt,
    updatedAtMs: item.updatedAt,
    scheduleKind: "manual"
  }));

  return {
    sourcePath: "localStorage:keai.desktop-pet.tasks",
    detail: "当前环境未启用 runtime 任务快照，以下为本地任务数据。",
    jobs
  };
}

async function loadRelatedMemorySnapshot() {
  const invoke = getTauriInvoke();
  if (invoke) {
    relatedMemorySnapshot.value = (await invoke("load_memory_file_snapshot")) as SourceFileSnapshotResponse;
    return;
  }
  relatedMemorySnapshot.value = buildMemoryFallbackSnapshot(loadMemories());
}

async function loadRelatedSkillsSnapshot() {
  const invoke = getTauriInvoke();
  if (!invoke) {
    relatedSkillsSnapshot.value = {
      sourcePath: "runtime unavailable",
      builtIn: [],
      installed: []
    };
    relatedResourceModalError.value = "当前环境不支持读取 OpenClaw 技能库。";
    return;
  }
  relatedSkillsSnapshot.value = (await invoke("load_openclaw_skills_list")) as OpenClawSkillsListResponse;
}

async function loadRelatedToolsSnapshot(agentId: string) {
  const invoke = getTauriInvoke();
  if (!invoke) {
    relatedToolsSnapshot.value = {
      profile: "default",
      profileLabel: "default",
      tools: []
    };
    relatedResourceModalError.value = "当前环境不支持读取 OpenClaw 工具权限。";
    return;
  }
  relatedToolsSnapshot.value = (await invoke("load_openclaw_tools_list", { agentId })) as OpenClawToolsListResponse;
}

async function loadRelatedChannelSnapshot() {
  const invoke = getTauriInvoke();
  if (!invoke) {
    relatedChannelSnapshot.value = {
      sourcePath: "runtime unavailable",
      detail: "当前环境不支持读取频道配置。",
      channels: []
    };
    relatedResourceModalError.value = "当前环境不支持频道配置快照。";
    return;
  }
  relatedChannelSnapshot.value = (await invoke("load_openclaw_channel_accounts_snapshot")) as OpenClawChannelAccountsSnapshotResponse;
}

async function loadRelatedTaskSnapshot() {
  const invoke = getTauriInvoke();
  if (invoke) {
    relatedTaskSnapshot.value = (await invoke("load_task_snapshot")) as TaskSnapshotResponse;
    return;
  }
  relatedTaskSnapshot.value = buildTaskFallbackSnapshot(taskItems.value);
}

async function refreshRelatedResourceData(target: RelatedResourceTarget) {
  const agent = activeAgent.value;
  if (!agent) {
    return;
  }
  relatedResourceModalLoading.value = true;
  relatedResourceModalError.value = "";
  relatedResourceModalNotice.value = "";

  try {
    if (target === "memory") {
      await loadRelatedMemorySnapshot();
      syncRelatedMemorySelection();
      return;
    }
    if (target === "skills") {
      await loadRelatedSkillsSnapshot();
      return;
    }
    if (target === "tools") {
      await loadRelatedToolsSnapshot(agent.agentId);
      return;
    }
    if (target === "channel") {
      await loadRelatedChannelSnapshot();
      return;
    }
    await loadRelatedTaskSnapshot();
  } catch (error) {
    relatedResourceModalError.value = error instanceof Error ? error.message : "加载关联资源失败。";
  } finally {
    relatedResourceModalLoading.value = false;
  }
}

async function openRelatedResource(target: RelatedResourceTarget) {
  if (!activeAgent.value) {
    return;
  }
  relatedResourceModalTarget.value = target;
  relatedSkillSearch.value = "";
  relatedMemorySearch.value = "";
  relatedResourceModalError.value = "";
  relatedResourceModalNotice.value = "";
  if (target === "skills") {
    relatedSkillCategory.value = "builtIn";
  }
  if (target === "memory") {
    relatedMemorySelectedId.value = null;
    relatedMemoryDraftContent.value = "";
  }
  clearRelatedResourceSnapshots();
  await refreshRelatedResourceData(target);
}

function closeRelatedResourceModal() {
  relatedResourceModalTarget.value = null;
  relatedResourceModalError.value = "";
  relatedResourceModalNotice.value = "";
  relatedMemorySearch.value = "";
  relatedMemorySelectedId.value = null;
  relatedMemoryDraftContent.value = "";
  relatedSkillSearch.value = "";
}

async function handleRelatedResourceRefresh() {
  if (!relatedResourceModalTarget.value) {
    return;
  }
  await refreshRelatedResourceData(relatedResourceModalTarget.value);
}

function handleRelatedMemorySelect(item: SourceFileSnapshotItem) {
  relatedMemorySelectedId.value = item.id;
  relatedMemoryDraftContent.value = item.content;
  relatedResourceModalNotice.value = "";
  relatedResourceModalError.value = "";
}

async function handleRelatedMemorySave() {
  const invoke = getTauriInvoke();
  const selected = relatedMemorySelectedItem.value;
  if (!selected || relatedResourceModalSaving.value) {
    return;
  }
  if (!invoke) {
    relatedResourceModalError.value = "当前环境不支持保存记忆文件。";
    return;
  }

  relatedResourceModalSaving.value = true;
  relatedResourceModalError.value = "";
  try {
    await invoke("save_source_file", {
      kind: "memory",
      sourcePath: selected.sourcePath,
      content: relatedMemoryDraftContent.value
    });
    relatedResourceModalNotice.value = `记忆文件「${getMemoryDisplayName(selected)}」已保存。`;
    await loadRelatedMemorySnapshot();
    syncRelatedMemorySelection(selected.id);
  } catch (error) {
    relatedResourceModalError.value = error instanceof Error ? error.message : "记忆文件保存失败。";
  } finally {
    relatedResourceModalSaving.value = false;
  }
}

function closeUtilityModal() {
  utilityModalType.value = null;
  utilityModalLoading.value = false;
  utilityModalError.value = "";
  utilityModalNotice.value = "";
}

async function refreshUtilityModalData(type: UtilityModalType) {
  const agent = activeAgent.value;
  utilityModalLoading.value = true;
  utilityModalError.value = "";

  try {
    if (type === "history") {
      if (!agent) {
        chatHistoryArchives.value = [];
        return;
      }
      chatHistoryArchives.value = loadChatArchives(agent.agentId);
      return;
    }

    const invoke = getTauriInvoke();
    if (!invoke) {
      chatRuntimeLogs.value = {
        detail: "当前环境不支持读取运行日志。",
        logs: []
      };
      return;
    }
    chatRuntimeLogs.value = (await invoke("load_openclaw_message_logs")) as OpenClawMessageLogResponse;
  } catch (error) {
    utilityModalError.value = error instanceof Error ? error.message : "加载数据失败。";
  } finally {
    utilityModalLoading.value = false;
  }
}

async function openUtilityModal(type: UtilityModalType) {
  utilityModalType.value = type;
  utilityModalNotice.value = "";
  utilityModalError.value = "";
  await refreshUtilityModalData(type);
}

async function handleUtilityModalRefresh() {
  if (!utilityModalType.value) {
    return;
  }
  await refreshUtilityModalData(utilityModalType.value);
}

async function handleArchiveCurrentChat() {
  const agent = activeAgent.value;
  if (!agent) {
    return;
  }

  const stableMessages = getStableChatMessages(chatMessages.value);
  const meaningfulMessages = stableMessages.filter((item) => item.role === "assistant" || item.role === "user");
  if (meaningfulMessages.length === 0) {
    utilityModalNotice.value = "当前会话暂无可归档的消息。";
    return;
  }

  const archives = loadChatArchives(agent.agentId);
  const archiveRecord: ChatArchiveRecord = {
    id: createMessageId("archive"),
    archivedAt: Date.now(),
    title: buildArchiveTitle(meaningfulMessages),
    messages: meaningfulMessages.map((item) => ({ ...item }))
  };
  const nextArchives = [archiveRecord, ...archives].slice(0, 60);
  persistChatArchives(agent.agentId, nextArchives);
  chatHistoryArchives.value = nextArchives;

  chatMessages.value = createWelcomeMessages(agent);
  currentSessionId.value = createSessionId();
  persistChatHistory(agent.agentId);
  agentHistories.value[agent.agentId] = [...chatMessages.value];
  refreshAgentMetaFromHistory(agent.agentId, chatMessages.value, agent.currentWork);

  utilityModalNotice.value = "当前会话已归档，可在聊天记录中查看。";
  if (utilityModalType.value === "history") {
    await refreshUtilityModalData("history");
  }
  void scrollMessagesToBottom();
}

function handleRestoreArchive(record: ChatArchiveRecord) {
  const agent = activeAgent.value;
  if (!agent) {
    return;
  }
  const restoredMessages = record.messages.map((message) => ({ ...message }));
  chatMessages.value = restoredMessages.length > 0 ? restoredMessages : createWelcomeMessages(agent);
  persistChatHistory(agent.agentId);
  agentHistories.value[agent.agentId] = [...chatMessages.value];
  refreshAgentMetaFromHistory(agent.agentId, chatMessages.value, agent.currentWork);
  utilityModalNotice.value = `已恢复归档会话「${record.title}」。`;
  closeUtilityModal();
  void scrollMessagesToBottom();
}

async function handleRelatedSkillToggle(skillId: string, enabled: boolean) {
  const invoke = getTauriInvoke();
  if (!invoke || relatedResourceModalSaving.value) {
    return;
  }

  relatedResourceModalSaving.value = true;
  relatedResourceModalError.value = "";
  try {
    await invoke("save_openclaw_skill_enabled", { skillId, enabled });
    const patchList = (items: OpenClawSkillListItem[]) => items.map((item) => (item.id === skillId ? { ...item, enabled } : item));
    if (relatedSkillsSnapshot.value) {
      relatedSkillsSnapshot.value = {
        ...relatedSkillsSnapshot.value,
        builtIn: patchList(relatedSkillsSnapshot.value.builtIn ?? []),
        installed: patchList(relatedSkillsSnapshot.value.installed ?? [])
      };
    }
    relatedResourceModalNotice.value = `技能「${skillId}」已${enabled ? "启用" : "禁用"}。`;
  } catch (error) {
    relatedResourceModalError.value = error instanceof Error ? error.message : "技能保存失败。";
  } finally {
    relatedResourceModalSaving.value = false;
  }
}

async function handleRelatedToolToggle(toolId: string, enabled: boolean) {
  const invoke = getTauriInvoke();
  const agent = activeAgent.value;
  const snapshot = relatedToolsSnapshot.value;
  if (!invoke || !agent || !snapshot || relatedResourceModalSaving.value) {
    return;
  }

  const previousTools = snapshot.tools;
  const nextTools = previousTools.map((tool) => (tool.id === toolId ? { ...tool, enabled } : tool));
  const enabledToolIds = nextTools.filter((tool) => tool.enabled).map((tool) => tool.id);

  relatedToolsSnapshot.value = { ...snapshot, tools: nextTools };
  relatedResourceModalSaving.value = true;
  relatedResourceModalError.value = "";

  try {
    await invoke("save_openclaw_tools_config", {
      agentId: agent.agentId,
      scope: "agent",
      profile: snapshot.profile || "default",
      enabledToolIds
    });
    relatedResourceModalNotice.value = `工具权限已更新：启用 ${enabledToolIds.length} 项。`;
    await loadAgents();
  } catch (error) {
    relatedToolsSnapshot.value = { ...snapshot, tools: previousTools };
    relatedResourceModalError.value = error instanceof Error ? error.message : "工具权限保存失败。";
  } finally {
    relatedResourceModalSaving.value = false;
  }
}

function isChannelAccountBoundToActiveAgent(account: OpenClawChannelAccountSnapshotItem) {
  const agent = activeAgent.value;
  if (!agent) {
    return false;
  }
  return equalsIgnoreCase(account.agentId ?? "", agent.agentId);
}

async function handleRelatedChannelBinding(channelType: string, accountId: string, account: OpenClawChannelAccountSnapshotItem) {
  const invoke = getTauriInvoke();
  const agent = activeAgent.value;
  if (!invoke || !agent || relatedResourceModalSaving.value) {
    return;
  }

  const nextAgentId = isChannelAccountBoundToActiveAgent(account) ? null : agent.agentId;
  relatedResourceModalSaving.value = true;
  relatedResourceModalError.value = "";

  try {
    await invoke("save_openclaw_channel_binding", {
      channelType,
      accountId,
      agentId: nextAgentId
    });
    relatedResourceModalNotice.value = nextAgentId ? "已绑定到当前员工。" : "已取消与当前员工的绑定。";
    await loadRelatedChannelSnapshot();
  } catch (error) {
    relatedResourceModalError.value = error instanceof Error ? error.message : "频道绑定保存失败。";
  } finally {
    relatedResourceModalSaving.value = false;
  }
}

function getModuleTitle(section: SidebarSection) {
  if (section === "dashboard") return "仪表盘";
  if (section === "staff") return "员工管理";
  if (section === "recruitment") return "员工招募";
  if (section === "skills") return "技能市场";
  if (section === "tasks") return "任务管理";
  return "聊天";
}

const activeAgent = computed(() => {
  if (!selectedAgentId.value) {
    return null;
  }
  return agents.value.find((item) => item.agentId === selectedAgentId.value) ?? null;
});

const normalizedQuery = computed(() => searchQuery.value.trim().toLowerCase());

const filteredAgents = computed(() => {
  if (!normalizedQuery.value) {
    return agents.value;
  }

  return agents.value.filter((agent) => {
    const blob = `${agent.displayName} ${agent.roleLabel} ${agent.channel} ${agent.currentWork}`.toLowerCase();
    return blob.includes(normalizedQuery.value);
  });
});

const staffAgents = computed(() => filteredAgents.value.filter((agent) => agent.groupKind === "staff"));
const groupAgents = computed(() => filteredAgents.value.filter((agent) => agent.groupKind === "group"));
const currentPaneAgents = computed(() => (activeAgentPaneTab.value === "staff" ? staffAgents.value : groupAgents.value));
const currentPaneEmptyText = computed(() => (activeAgentPaneTab.value === "staff" ? "暂无角色 Agent" : "暂无群组 Agent"));
const sidebarChatBadge = computed(() => {
  const unread = agents.value.reduce((sum, agent) => sum + getAgentMeta(agent.agentId).unread, 0);
  const value = unread > 0 ? unread : agents.value.length;
  if (value > 99) {
    return "99+";
  }
  return String(value);
});
const sidebarDisplayName = computed(() => (activeAgent.value ? stripRoleLabel(activeAgent.value.displayName) : "ClawPet"));
const relatedResourceModalTitle = computed(() => {
  if (relatedResourceModalTarget.value === "memory") return "关联资源 · 记忆";
  if (relatedResourceModalTarget.value === "skills") return "关联资源 · 技能库";
  if (relatedResourceModalTarget.value === "tools") return "关联资源 · 工具权限";
  if (relatedResourceModalTarget.value === "channel") return "关联资源 · 频道";
  if (relatedResourceModalTarget.value === "schedule") return "关联资源 · 任务";
  return "关联资源";
});
const relatedResourceModalSubtitle = computed(() => {
  const agent = activeAgent.value;
  if (!agent) {
    return "当前未选择员工。";
  }
  return `${stripRoleLabel(agent.displayName)} · ${agent.agentId}`;
});
const relatedMemoryItems = computed(() => {
  const items = relatedMemorySnapshot.value?.items ?? [];
  const agent = activeAgent.value;
  if (!agent || items.length === 0) {
    return items;
  }
  const keywords = [stripRoleLabel(agent.displayName), agent.agentId, agent.roleLabel]
    .map((item) => item.trim().toLowerCase())
    .filter(Boolean);
  const matched = items.filter((item) => {
    const blob = `${item.title} ${item.summary} ${item.relativePath} ${item.facetLabel}`.toLowerCase();
    return keywords.some((keyword) => blob.includes(keyword));
  });
  return matched.length > 0 ? matched : items;
});
const relatedMemoryFilteredItems = computed(() => {
  const keyword = relatedMemorySearch.value.trim().toLowerCase();
  if (!keyword) {
    return relatedMemoryItems.value;
  }
  return relatedMemoryItems.value.filter((item) => {
    const blob = `${item.title} ${item.summary} ${item.relativePath} ${item.facetLabel}`.toLowerCase();
    return blob.includes(keyword);
  });
});
const relatedMemorySelectedItem = computed(() => {
  const items = relatedMemoryItems.value;
  if (items.length === 0) {
    return null;
  }
  if (!relatedMemorySelectedId.value) {
    return items[0];
  }
  return items.find((item) => item.id === relatedMemorySelectedId.value) ?? items[0];
});
const relatedMemorySaveLabel = computed(() => {
  if (!relatedMemorySelectedItem.value) {
    return "保存记忆文件";
  }
  return relatedMemorySelectedItem.value.exists === false ? "创建并保存记忆文件" : "保存记忆文件";
});
const filteredRelatedBuiltInSkills = computed(() => {
  const keyword = relatedSkillSearch.value.trim().toLowerCase();
  const list = relatedSkillsSnapshot.value?.builtIn ?? [];
  if (!keyword) {
    return list;
  }
  return list.filter((skill) => `${skill.name} ${skill.description}`.toLowerCase().includes(keyword));
});
const filteredRelatedInstalledSkills = computed(() => {
  const keyword = relatedSkillSearch.value.trim().toLowerCase();
  const list = relatedSkillsSnapshot.value?.installed ?? [];
  if (!keyword) {
    return list;
  }
  return list.filter((skill) => `${skill.name} ${skill.description} ${skill.relativePath}`.toLowerCase().includes(keyword));
});
const activeRelatedSkills = computed(() =>
  relatedSkillCategory.value === "installed" ? filteredRelatedInstalledSkills.value : filteredRelatedBuiltInSkills.value
);
const activeRelatedSkillsTitle = computed(() => (relatedSkillCategory.value === "installed" ? "安装技能" : "内置技能"));
const activeRelatedSkillsEmptyText = computed(() =>
  relatedSkillCategory.value === "installed" ? "暂无安装技能。" : "暂无内置技能。"
);
const utilityModalTitle = computed(() => {
  if (utilityModalType.value === "history") {
    return "聊天记录";
  }
  if (utilityModalType.value === "logs") {
    return "运行日志";
  }
  return "";
});
const utilityModalSubtitle = computed(() => {
  const agent = activeAgent.value;
  if (!agent) {
    return "当前未选择员工。";
  }
  return `${stripRoleLabel(agent.displayName)} · ${agent.agentId}`;
});
const currentSessionMessages = computed(() =>
  getStableChatMessages(chatMessages.value).filter((item) => item.role === "assistant" || item.role === "user")
);
const currentSessionPreviewText = computed(() => {
  const latest = currentSessionMessages.value[currentSessionMessages.value.length - 1];
  if (!latest) {
    return "当前会话暂无消息";
  }
  const text = latest.text.replace(/\s+/g, " ").trim();
  if (!text) {
    return "当前会话暂无消息";
  }
  if (text.length > 72) {
    return `${text.slice(0, 72)}...`;
  }
  return text;
});
const runtimeLogItems = computed(() =>
  [...(chatRuntimeLogs.value?.logs ?? [])].sort((left, right) => right.createdAt - left.createdAt)
);
const relatedToolsEnabledCount = computed(() => (relatedToolsSnapshot.value?.tools ?? []).filter((tool) => tool.enabled).length);
const relatedToolsByCategory = computed(() => {
  const groups = new Map<string, OpenClawToolListItem[]>();
  for (const tool of relatedToolsSnapshot.value?.tools ?? []) {
    const category = tool.category || "other";
    const list = groups.get(category) ?? [];
    list.push(tool);
    groups.set(category, list);
  }
  return Array.from(groups.entries()).map(([category, tools]) => ({ category, tools }));
});
const relatedScheduleJobs = computed(() => {
  const jobs = relatedTaskSnapshot.value?.jobs ?? [];
  const agent = activeAgent.value;
  if (!agent) {
    return jobs;
  }
  const matched = jobs.filter((job) => equalsIgnoreCase(job.agentId, agent.agentId) || equalsIgnoreCase(job.agentId, stripRoleLabel(agent.displayName)));
  return matched.length > 0 ? matched : jobs;
});

const taskSummary = computed(() => {
  const total = taskItems.value.length;
  const todo = taskItems.value.filter((item) => item.status === "todo").length;
  const doing = taskItems.value.filter((item) => item.status === "in_progress").length;
  const blocked = taskItems.value.filter((item) => item.status === "blocked").length;
  const done = taskItems.value.filter((item) => item.status === "done").length;

  return { total, todo, doing, blocked, done };
});

onMounted(async () => {
  taskItems.value = loadTasks();
  await loadAgents();
  await scrollMessagesToBottom();
});
</script>

<template>
  <div class="chat-page">
    <div class="window-shell">
      <div class="chat-app">
        <aside class="sidebar-icons" aria-label="功能模块" @mousedown.left="handleRegionMouseDown">
          <div class="sidebar-top">
            <div class="window-controls" aria-label="窗口控制">
              <button class="window-control window-control--close" type="button" title="关闭" @click="handleWindowClose" />
              <button class="window-control window-control--minimize" type="button" title="最小化" @click="handleWindowMinimize" />
              <button class="window-control window-control--expand" type="button" title="全屏 / 还原" @click="handleWindowExpand" />
            </div>
            <div class="sidebar-profile" data-no-window-drag>
              <span class="sidebar-profile__avatar">{{ activeAgent ? getAgentInitial(activeAgent) : "M" }}</span>
              <i class="sidebar-profile__status" aria-hidden="true" />
              <div class="sidebar-profile__meta">
                <small>在线中</small>
                <strong>{{ sidebarDisplayName }}</strong>
              </div>
            </div>
          </div>

          <div class="sidebar-nav">
            <button
              v-for="item in sidebarItems"
              :key="item.id"
              class="nav-item"
              :class="{ active: activeSection === item.id }"
              :title="item.label"
              type="button"
              @click="activeSection = item.id"
            >
              <span class="nav-item__icon" aria-hidden="true">
                <svg v-if="item.id === 'chat'" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
                <svg v-else-if="item.id === 'dashboard'" viewBox="0 0 24 24"><path d="M3 13h8V3H3zm10 8h8V3h-8zm-10 0h8v-6H3z" /></svg>
                <svg v-else-if="item.id === 'staff'" viewBox="0 0 24 24"><path d="M16 11a4 4 0 1 0-4-4 4 4 0 0 0 4 4zM8 11a4 4 0 1 0-4-4 4 4 0 0 0 4 4zM8 13c-3.3 0-6 1.8-6 4v2h8v-2a4.7 4.7 0 0 1 2-3.8A8.8 8.8 0 0 0 8 13zm8 0a7 7 0 0 0-3.9 1.1A4.8 4.8 0 0 1 14 17v2h8v-2c0-2.2-2.7-4-6-4z" /></svg>
                <svg v-else-if="item.id === 'recruitment'" viewBox="0 0 24 24"><path d="M10 2a8 8 0 1 0 5.3 14l4.9 4.9 1.4-1.4-4.9-4.9A8 8 0 0 0 10 2zm0 2a6 6 0 1 1-6 6 6 6 0 0 1 6-6z" /></svg>
                <svg v-else-if="item.id === 'skills'" viewBox="0 0 24 24"><path d="M14.7 6.3a4 4 0 0 0-5.4 5.8L3 18.5V21h2.5l6.4-6.3a4 4 0 0 0 2.8-8.4zM14 10a2 2 0 1 1 1.4-.6A2 2 0 0 1 14 10z" /></svg>
                <svg v-else viewBox="0 0 24 24"><path d="M9 11H7v2h2zm4 0h-2v2h2zm4 0h-2v2h2zm2-8H5a2 2 0 0 0-2 2v14l4-4h12a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2z" /></svg>
              </span>
              <span class="nav-item__label">{{ item.label }}</span>
              <span v-if="item.id === 'chat'" class="nav-item__badge">{{ sidebarChatBadge }}</span>
            </button>
          </div>

          <div class="sidebar-spacer" />
          <button class="nav-item nav-item--secondary" type="button" title="帮助">
            <span class="nav-item__icon" aria-hidden="true">
              <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" /><path d="M9.5 9a2.5 2.5 0 1 1 4.5 1.5c-.6.7-1.2 1-1.6 1.5-.3.4-.4.8-.4 1.5" /><circle cx="12" cy="17" r="1" /></svg>
            </span>
            <span class="nav-item__label">帮助</span>
          </button>
        </aside>

        <template v-if="activeSection === 'chat'">
          <section class="chat-list">
            <header class="chat-list__header" @mousedown.left="handleRegionMouseDown">
              <div class="search-row">
                <label class="search-box" aria-label="搜索 Agent">
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10 2a8 8 0 1 0 4.9 14.3l4.4 4.4 1.4-1.4-4.4-4.4A8 8 0 0 0 10 2zm0 2a6 6 0 1 1-6 6 6 6 0 0 1 6-6z" /></svg>
                  <input v-model="searchQuery" type="text" placeholder="搜索 Agent" />
                </label>
                <button class="search-add" type="button" title="新会话" @click="handleNewChat">
                  <svg viewBox="0 0 20 20" aria-hidden="true"><path d="M10 4v12M4 10h12" /></svg>
                </button>
              </div>
              <div class="agent-pane-tabs" role="tablist" aria-label="Agent 分组">
                <button
                  v-for="tab in agentPaneTabs"
                  :key="tab.id"
                  class="agent-pane-tab"
                  :class="{ active: activeAgentPaneTab === tab.id }"
                  type="button"
                  @click="activeAgentPaneTab = tab.id"
                >
                  {{ tab.label }}
                </button>
              </div>
            </header>

            <div class="chat-list__body">
              <button
                v-for="agent in currentPaneAgents"
                :key="agent.agentId"
                class="agent-item"
                :class="{ active: selectedAgentId === agent.agentId }"
                type="button"
                @click="switchAgent(agent.agentId)"
              >
                <div class="agent-avatar" :class="{ 'agent-avatar--group': agent.groupKind === 'group' }">
                  <span>{{ getAgentInitial(agent) }}</span>
                  <i class="status-dot" :data-tone="agent.statusTone" />
                </div>
                <div class="agent-content">
                  <div class="agent-top-line">
                    <strong>{{ stripRoleLabel(agent.displayName) }}</strong>
                    <span class="agent-channel">{{ agent.channel }}</span>
                  </div>
                  <p class="agent-preview">{{ getAgentMeta(agent.agentId).preview }}</p>
                  <small class="agent-status">{{ getAgentStatusLabel(agent) }}</small>
                </div>
                <div class="agent-meta">
                  <span>{{ getAgentMeta(agent.agentId).timeLabel }}</span>
                  <span v-if="getAgentMeta(agent.agentId).unread" class="agent-unread">{{ getAgentMeta(agent.agentId).unread }}</span>
                </div>
              </button>
              <p v-if="currentPaneAgents.length === 0" class="list-empty">
                {{ normalizedQuery ? "没有匹配的 Agent" : currentPaneEmptyText }}
              </p>
            </div>
          </section>

          <section class="chat-window" :class="{ 'chat-window--settings-open': isAgentSettingsOpen }">
            <header class="chat-window__header" @mousedown.left="handleRegionMouseDown">
              <div class="chat-agent-header">
                <div class="chat-agent-header__avatar">{{ activeAgent ? getAgentInitial(activeAgent) : "?" }}</div>
                <div>
                  <strong>{{ activeAgent ? stripRoleLabel(activeAgent.displayName) : "请选择 Agent" }}</strong>
                  <p>
                    {{ activeAgent ? getAgentStatusLabel(activeAgent) : "未选择" }}
                    <span v-if="activeAgent">· {{ activeAgent.channel }}</span>
                  </p>
                </div>
              </div>
              <div class="chat-window__actions">
                <button
                  type="button"
                  class="header-btn"
                  :class="{ 'is-active': utilityModalType === 'history' }"
                  title="聊天记录"
                  aria-label="打开聊天记录"
                  @click="openUtilityModal('history')"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 5h16M4 10h10M4 15h16M4 20h10" /></svg>
                </button>
                <button
                  type="button"
                  class="header-btn"
                  :class="{ 'is-active': utilityModalType === 'logs' }"
                  title="运行日志"
                  aria-label="打开运行日志"
                  @click="openUtilityModal('logs')"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 18h16M7 14l3-4 3 2 4-6" /><circle cx="7" cy="14" r="1.5" /><circle cx="10" cy="10" r="1.5" /><circle cx="13" cy="12" r="1.5" /><circle cx="17" cy="6" r="1.5" /></svg>
                </button>
                <button
                  type="button"
                  class="header-btn"
                  title="归档当前会话"
                  aria-label="归档当前会话"
                  :disabled="!activeAgent"
                  @click="handleArchiveCurrentChat"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16v4H4zM6 11v8h12v-8M9 4h6v3H9z" /></svg>
                </button>
                <button
                  type="button"
                  class="header-btn"
                  :class="{ 'is-active': isAgentSettingsOpen }"
                  title="设置"
                  aria-label="打开会话设置"
                  @click="toggleAgentSettingsPanel"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <circle cx="12" cy="12" r="3.2" />
                    <path d="M19.4 15a1 1 0 0 0 .2 1.1l.1.1a1.8 1.8 0 1 1-2.5 2.5l-.1-.1a1 1 0 0 0-1.1-.2h-.1a1 1 0 0 0-.6.9V20a1.8 1.8 0 0 1-3.6 0v-.1a1 1 0 0 0-.6-.9h-.1a1 1 0 0 0-1.1.2l-.1.1a1.8 1.8 0 0 1-2.5-2.5l.1-.1a1 1 0 0 0 .2-1.1v-.1a1 1 0 0 0-.9-.6H4a1.8 1.8 0 0 1 0-3.6h.1a1 1 0 0 0 .9-.6v-.1a1 1 0 0 0-.2-1.1l-.1-.1a1.8 1.8 0 1 1 2.5-2.5l.1.1a1 1 0 0 0 1.1.2h.1a1 1 0 0 0 .6-.9V4a1.8 1.8 0 0 1 3.6 0v.1a1 1 0 0 0 .6.9h.1a1 1 0 0 0 1.1-.2l.1-.1a1.8 1.8 0 1 1 2.5 2.5l-.1.1a1 1 0 0 0-.2 1.1v.1a1 1 0 0 0 .9.6H20a1.8 1.8 0 0 1 0 3.6h-.1a1 1 0 0 0-.9.6z" />
                  </svg>
                </button>
              </div>
            </header>

            <div class="chat-window__content" :class="{ 'chat-window__content--settings-open': isAgentSettingsOpen }">
              <div ref="messageScroller" class="chat-window__messages">
                <article
                  v-for="message in chatMessages"
                  :key="message.id"
                  class="message-row"
                  :class="[`message-row--${message.role}`, `message-row--${message.status}`]"
                >
                  <div class="message-bubble">{{ message.text }}</div>
                  <span class="message-time">{{ getMessageTimeLabel(message) }}</span>
                </article>
              </div>

              <aside v-if="isAgentSettingsOpen" class="chat-settings-sidebar" data-no-window-drag @mousedown.left.stop>
                <header class="chat-settings-sidebar__header chat-settings-sidebar__header--plain">
                  <button type="button" class="chat-settings-sidebar__close" aria-label="关闭设置" @click="closeAgentSettingsPanel">×</button>
                </header>

                <div v-if="activeAgent" class="chat-settings-sidebar__body">
                  <div class="chat-settings-agent-card">
                    <div class="chat-settings-agent-card__head">
                      <div class="chat-settings-agent-card__avatar">{{ getAgentInitial(activeAgent) }}</div>
                      <div class="chat-settings-agent-card__identity">
                        <strong>{{ stripRoleLabel(activeAgent.displayName) }}</strong>
                        <div class="chat-settings-chip-row">
                          <span class="chat-settings-status-chip" :data-tone="activeAgent.statusTone">{{ getAgentStatusLabel(activeAgent) }}</span>
                          <span class="chat-settings-soft-chip">{{ getAgentScheduledLabel(activeAgent) }}</span>
                        </div>
                      </div>
                    </div>

                    <section class="chat-settings-resource-quick">
                      <div class="chat-settings-resource-grid">
                        <button class="chat-settings-resource-action" type="button" @click="openRelatedResource('memory')">
                          <span class="chat-settings-resource-action__icon">记</span>
                          <span class="chat-settings-resource-action__content">
                            <span class="chat-settings-resource-action__main">记忆</span>
                            <span class="chat-settings-resource-action__sub">查看与编辑记忆文件</span>
                          </span>
                          <span class="chat-settings-resource-action__arrow">›</span>
                        </button>
                        <button class="chat-settings-resource-action" type="button" @click="openRelatedResource('skills')">
                          <span class="chat-settings-resource-action__icon">技</span>
                          <span class="chat-settings-resource-action__content">
                            <span class="chat-settings-resource-action__main">技能库</span>
                            <span class="chat-settings-resource-action__sub">启用或禁用技能能力</span>
                          </span>
                          <span class="chat-settings-resource-action__arrow">›</span>
                        </button>
                        <button class="chat-settings-resource-action" type="button" @click="openRelatedResource('tools')">
                          <span class="chat-settings-resource-action__icon">工</span>
                          <span class="chat-settings-resource-action__content">
                            <span class="chat-settings-resource-action__main">工具权限 {{ getAgentToolsEnabledLabel(activeAgent) }}</span>
                            <span class="chat-settings-resource-action__sub">配置可用工具权限</span>
                          </span>
                          <span class="chat-settings-resource-action__arrow">›</span>
                        </button>
                        <button class="chat-settings-resource-action" type="button" @click="openRelatedResource('channel')">
                          <span class="chat-settings-resource-action__icon">频</span>
                          <span class="chat-settings-resource-action__content">
                            <span class="chat-settings-resource-action__main">频道 {{ activeAgent.channel || "main" }}</span>
                            <span class="chat-settings-resource-action__sub">管理渠道账号绑定</span>
                          </span>
                          <span class="chat-settings-resource-action__arrow">›</span>
                        </button>
                        <button class="chat-settings-resource-action" type="button" @click="openRelatedResource('schedule')">
                          <span class="chat-settings-resource-action__icon">排</span>
                          <span class="chat-settings-resource-action__content">
                            <span class="chat-settings-resource-action__main">任务</span>
                            <span class="chat-settings-resource-action__sub">查看与调整任务</span>
                          </span>
                          <span class="chat-settings-resource-action__arrow">›</span>
                        </button>
                      </div>
                    </section>

                    <dl class="chat-settings-list">
                      <div class="chat-settings-list__row">
                        <dt>工具权限</dt>
                        <dd>{{ activeAgent.toolsProfile || "—" }}</dd>
                      </div>
                      <div class="chat-settings-list__row">
                        <dt>模型</dt>
                        <dd>{{ activeAgent.model || "—" }}</dd>
                      </div>
                      <div class="chat-settings-list__row">
                        <dt>工作目录</dt>
                        <dd>{{ activeAgent.workspace || "—" }}</dd>
                      </div>
                      <div class="chat-settings-list__row">
                        <dt>所属渠道</dt>
                        <dd>{{ activeAgent.channel || "—" }}</dd>
                      </div>
                      <div class="chat-settings-list__row">
                        <dt>{{ getAgentCurrentWorkLabel(activeAgent) }}</dt>
                        <dd>{{ activeAgent.currentWork }}</dd>
                      </div>
                    </dl>

                    <section class="chat-settings-output">
                      <h5>最近产出</h5>
                      <div class="chat-settings-recent-output">{{ getAgentRecentOutput(activeAgent) }}</div>
                    </section>
                  </div>
                </div>

                <div v-else class="chat-settings-sidebar__empty">请选择员工后打开设置。</div>
              </aside>
            </div>

            <footer class="chat-window__composer">
              <div class="composer-input-shell">
                <button class="composer-input-action" type="button" title="附件" aria-label="添加附件">
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M16.5 6.5 9 14a3 3 0 1 1-4.2-4.2l8-8a5 5 0 1 1 7.1 7.1l-9 9a7 7 0 1 1-9.9-9.9l8.5-8.5" /></svg>
                </button>
                <input
                  v-model="chatInput"
                  type="text"
                  :placeholder="activeAgent ? `给 ${stripRoleLabel(activeAgent.displayName)} 发送消息...` : '请选择 Agent 后发送消息'"
                  :disabled="!activeAgent"
                  @keydown.enter.prevent="submitChat"
                />
              </div>
              <button class="composer-btn composer-btn--archive" type="button" title="归档" :disabled="!activeAgent" @click="handleArchiveCurrentChat">
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16v4H4zM6 11v8h12v-8M9 4h6v3H9z" /></svg>
              </button>
              <button class="composer-send" type="button" :disabled="!activeAgent || isSending || !chatInput.trim()" @click="submitChat">
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 12h14M13 5l7 7-7 7" /></svg>
              </button>
            </footer>
          </section>
        </template>

        <main v-else class="module-board">
          <header class="module-board__header" @mousedown.left="handleRegionMouseDown">
            <h2>{{ getModuleTitle(activeSection) }}</h2>
            <p>{{ missionStatement }}</p>
          </header>

          <div class="module-board__metrics">
            <article>
              <span>任务总数</span>
              <strong>{{ taskSummary.total }}</strong>
            </article>
            <article>
              <span>进行中</span>
              <strong>{{ taskSummary.doing }}</strong>
            </article>
            <article>
              <span>阻塞</span>
              <strong>{{ taskSummary.blocked }}</strong>
            </article>
            <article>
              <span>已完成</span>
              <strong>{{ taskSummary.done }}</strong>
            </article>
            <article>
              <span>Agent 数量</span>
              <strong>{{ agents.length }}</strong>
            </article>
          </div>

          <p class="module-board__note">该模块将在下一步接入完整业务面板。当前优先完成聊天壳层与 Agent 会话接入。</p>
          <p class="module-board__detail">{{ staffSourceDetail }}</p>
        </main>
      </div>
    </div>

    <div v-if="relatedResourceModalTarget" class="related-resource-modal-backdrop" @click.self="closeRelatedResourceModal">
      <section class="related-resource-modal" role="dialog" aria-modal="true" :aria-label="relatedResourceModalTitle">
        <header class="related-resource-modal__header">
          <div>
            <strong>{{ relatedResourceModalTitle }}</strong>
            <p>{{ relatedResourceModalSubtitle }}</p>
          </div>
          <div class="related-resource-modal__actions">
            <button
              class="related-resource-modal__refresh"
              type="button"
              :disabled="relatedResourceModalLoading || relatedResourceModalSaving"
              @click="handleRelatedResourceRefresh"
            >
              刷新
            </button>
            <button class="related-resource-modal__close" type="button" aria-label="关闭" @click="closeRelatedResourceModal">×</button>
          </div>
        </header>

        <div class="related-resource-modal__body">
          <p v-if="relatedResourceModalNotice" class="related-resource-modal__notice">{{ relatedResourceModalNotice }}</p>
          <p v-if="relatedResourceModalError" class="related-resource-modal__error">{{ relatedResourceModalError }}</p>

          <div v-if="relatedResourceModalLoading" class="related-resource-modal__empty">正在读取资源配置...</div>

          <template v-else-if="relatedResourceModalTarget === 'memory'">
            <p class="related-resource-modal__detail">{{ relatedMemorySnapshot?.detail || "暂无记忆说明。" }}</p>
            <div v-if="relatedMemoryItems.length > 0" class="related-memory-layout">
              <aside class="related-memory-nav-pane">
                <div class="related-memory-nav-pane__toolbar">
                  <input v-model="relatedMemorySearch" class="related-resource-filter-input" type="text" placeholder="筛选标题、路径或摘要" />
                  <small>{{ relatedMemoryFilteredItems.length }} 项</small>
                </div>
                <div class="related-memory-nav-list">
                  <button
                    v-for="item in relatedMemoryFilteredItems"
                    :key="`memory-${item.id}`"
                    class="related-memory-nav-item"
                    :class="{ active: relatedMemorySelectedItem?.id === item.id }"
                    type="button"
                    @click="handleRelatedMemorySelect(item)"
                  >
                    <strong>{{ getMemoryDisplayName(item) }}</strong>
                    <p>{{ item.summary || "暂无摘要" }}</p>
                  </button>
                  <div v-if="relatedMemoryFilteredItems.length === 0" class="related-resource-modal__empty related-resource-modal__empty--small">
                    未找到匹配的记忆文件。
                  </div>
                </div>
              </aside>
              <section class="related-memory-editor-pane">
                <div v-if="relatedMemorySelectedItem" class="related-memory-editor-pane__inner">
                  <div class="related-memory-editor-pane__toolbar">
                    <small :title="relatedMemorySelectedItem.sourcePath">{{ relatedMemorySelectedItem.sourcePath || "—" }}</small>
                    <button
                      class="related-resource-modal__refresh"
                      type="button"
                      :disabled="relatedResourceModalSaving || relatedResourceModalLoading"
                      @click="handleRelatedMemorySave"
                    >
                      {{ relatedMemorySaveLabel }}
                    </button>
                  </div>
                  <div class="related-memory-editor-pane__meta">
                    <div>
                      <span>标题</span>
                      <strong>{{ getMemoryDisplayName(relatedMemorySelectedItem) }}</strong>
                    </div>
                    <div>
                      <span>分类</span>
                      <strong>{{ relatedMemorySelectedItem.facetLabel || "—" }}</strong>
                    </div>
                    <div>
                      <span>路径</span>
                      <strong>{{ relatedMemorySelectedItem.relativePath || "—" }}</strong>
                    </div>
                  </div>
                  <textarea v-model="relatedMemoryDraftContent" class="related-memory-editor-pane__textarea" placeholder="记忆文件内容" />
                </div>
                <div v-else class="related-resource-modal__empty">请选择一份记忆文件进行编辑。</div>
              </section>
            </div>
            <div v-else class="related-resource-modal__empty">暂无可展示的记忆资源。</div>
          </template>

          <template v-else-if="relatedResourceModalTarget === 'skills'">
            <p class="related-resource-modal__detail">
              来源：{{ relatedSkillsSnapshot?.sourcePath || "—" }} · 内置 {{ relatedSkillsSnapshot?.builtIn?.length ?? 0 }} 项 / 安装
              {{ relatedSkillsSnapshot?.installed?.length ?? 0 }} 项
            </p>
            <div class="related-skill-toolbar">
              <input v-model="relatedSkillSearch" class="related-resource-filter-input" type="text" placeholder="筛选技能名称或描述" />
            </div>
            <div class="related-skill-switch" role="tablist" aria-label="技能分类切换">
              <button
                class="related-skill-switch__button"
                :class="{ 'is-active': relatedSkillCategory === 'builtIn' }"
                type="button"
                @click="relatedSkillCategory = 'builtIn'"
              >
                内置技能
                <em>{{ filteredRelatedBuiltInSkills.length }}</em>
              </button>
              <button
                class="related-skill-switch__button"
                :class="{ 'is-active': relatedSkillCategory === 'installed' }"
                type="button"
                @click="relatedSkillCategory = 'installed'"
              >
                安装技能
                <em>{{ filteredRelatedInstalledSkills.length }}</em>
              </button>
            </div>
            <section class="related-resource-section">
              <h4>{{ activeRelatedSkillsTitle }}</h4>
              <div v-if="activeRelatedSkills.length === 0" class="related-resource-modal__empty related-resource-modal__empty--small">
                {{ activeRelatedSkillsEmptyText }}
              </div>
              <label v-for="skill in activeRelatedSkills" :key="`skill-${skill.id}`" class="related-switch-row related-switch-row--skill">
                <input
                  type="checkbox"
                  :checked="skill.enabled"
                  :disabled="relatedResourceModalSaving"
                  @change="handleRelatedSkillToggle(skill.id, ($event.target as HTMLInputElement).checked)"
                />
                <div>
                  <strong>{{ skill.name }}</strong>
                  <p>{{ skill.description }}</p>
                  <small v-if="relatedSkillCategory === 'installed' && skill.relativePath">{{ skill.relativePath }}</small>
                </div>
              </label>
            </section>
          </template>

          <template v-else-if="relatedResourceModalTarget === 'tools'">
            <p class="related-resource-modal__detail">
              Profile：{{ relatedToolsSnapshot?.profileLabel || relatedToolsSnapshot?.profile || "default" }} · 已启用
              {{ relatedToolsEnabledCount }} / {{ relatedToolsSnapshot?.tools?.length ?? 0 }}
            </p>
            <div v-if="relatedToolsByCategory.length === 0" class="related-resource-modal__empty">暂无工具配置数据。</div>
            <section v-for="group in relatedToolsByCategory" :key="`tools-${group.category}`" class="related-resource-section">
              <h4>{{ group.category }}</h4>
              <label v-for="tool in group.tools" :key="`tool-${tool.id}`" class="related-switch-row">
                <input
                  type="checkbox"
                  :checked="tool.enabled"
                  :disabled="relatedResourceModalSaving"
                  @change="handleRelatedToolToggle(tool.id, ($event.target as HTMLInputElement).checked)"
                />
                <div>
                  <strong>{{ tool.name || tool.id }}</strong>
                  <p>{{ tool.description }}</p>
                </div>
              </label>
            </section>
          </template>

          <template v-else-if="relatedResourceModalTarget === 'channel'">
            <p class="related-resource-modal__detail">{{ relatedChannelSnapshot?.detail || "暂无频道配置数据。" }}</p>
            <div v-if="(relatedChannelSnapshot?.channels?.length ?? 0) === 0" class="related-resource-modal__empty">暂无可配置频道。</div>
            <section v-for="group in relatedChannelSnapshot?.channels ?? []" :key="`channel-${group.channelType}`" class="related-resource-section">
              <h4>{{ group.channelType }}</h4>
              <article
                v-for="account in group.accounts"
                :key="`channel-${group.channelType}-${account.accountId}`"
                class="related-channel-row"
                :class="{ 'is-bound': isChannelAccountBoundToActiveAgent(account) }"
              >
                <div>
                  <strong>{{ account.name || account.accountId }}</strong>
                  <p>{{ account.accountId }} · {{ account.status }} · {{ account.configured ? "已配置" : "未配置" }}</p>
                </div>
                <button
                  class="related-channel-row__action"
                  type="button"
                  :disabled="relatedResourceModalSaving"
                  @click="handleRelatedChannelBinding(group.channelType, account.accountId, account)"
                >
                  {{ isChannelAccountBoundToActiveAgent(account) ? "取消绑定" : "绑定当前员工" }}
                </button>
              </article>
            </section>
          </template>

          <template v-else>
            <p class="related-resource-modal__detail">{{ relatedTaskSnapshot?.detail || "暂无排班数据。" }}</p>
            <div v-if="relatedScheduleJobs.length === 0" class="related-resource-modal__empty">当前员工暂无排班任务。</div>
            <div v-else class="related-resource-list">
              <article v-for="job in relatedScheduleJobs" :key="`schedule-${job.id}`" class="related-resource-card">
                <strong>{{ job.name }}</strong>
                <p>{{ job.summary || "暂无任务描述。" }}</p>
                <small>
                  {{ job.statusLabel }} · 下次运行 {{ formatDateTime(job.nextRunAtMs) }} · 更新时间 {{ formatDateTime(job.updatedAtMs) }}
                </small>
              </article>
            </div>
          </template>
        </div>
      </section>
    </div>

    <div v-if="utilityModalType" class="utility-modal-backdrop" @click.self="closeUtilityModal">
      <section class="utility-modal" role="dialog" aria-modal="true" :aria-label="utilityModalTitle">
        <header class="utility-modal__header">
          <div>
            <strong>{{ utilityModalTitle }}</strong>
            <p>{{ utilityModalSubtitle }}</p>
          </div>
          <div class="utility-modal__actions">
            <button class="utility-modal__refresh" type="button" :disabled="utilityModalLoading" @click="handleUtilityModalRefresh">刷新</button>
            <button class="utility-modal__close" type="button" aria-label="关闭" @click="closeUtilityModal">×</button>
          </div>
        </header>

        <div class="utility-modal__body">
          <p v-if="utilityModalNotice" class="utility-modal__notice">{{ utilityModalNotice }}</p>
          <p v-if="utilityModalError" class="utility-modal__error">{{ utilityModalError }}</p>
          <div v-if="utilityModalLoading" class="utility-modal__empty">正在加载数据...</div>

          <template v-else-if="utilityModalType === 'history'">
            <p class="utility-modal__detail">
              当前会话 {{ currentSessionMessages.length }} 条消息 · 归档记录 {{ chatHistoryArchives.length }} 条
            </p>
            <section class="utility-history-current">
              <div>
                <strong>当前会话</strong>
                <p>{{ currentSessionPreviewText }}</p>
              </div>
              <button class="utility-history-current__action" type="button" :disabled="currentSessionMessages.length === 0" @click="handleArchiveCurrentChat">
                归档当前会话
              </button>
            </section>
            <div v-if="chatHistoryArchives.length === 0" class="utility-modal__empty">暂无归档记录。</div>
            <div v-else class="utility-history-list">
              <article v-for="record in chatHistoryArchives" :key="record.id" class="utility-history-card">
                <div class="utility-history-card__head">
                  <strong>{{ record.title }}</strong>
                  <small>{{ formatDateTime(record.archivedAt) }}</small>
                </div>
                <p>{{ getArchivePreviewText(record) }}</p>
                <div class="utility-history-card__foot">
                  <span>{{ record.messages.length }} 条消息</span>
                  <button class="utility-history-card__action" type="button" @click="handleRestoreArchive(record)">恢复到当前会话</button>
                </div>
              </article>
            </div>
          </template>

          <template v-else>
            <p class="utility-modal__detail">{{ chatRuntimeLogs?.detail || "展示 OpenClaw 运行日志。" }}</p>
            <div v-if="runtimeLogItems.length === 0" class="utility-modal__empty">暂无日志记录。</div>
            <div v-else class="utility-log-list">
              <article v-for="log in runtimeLogItems" :key="log.id" class="utility-log-card">
                <div class="utility-log-card__head">
                  <strong>{{ log.platformName || "runtime" }}</strong>
                  <span class="utility-log-status" :data-tone="getLogStatusTone(log.responseStatus)">{{ log.responseStatus }}</span>
                </div>
                <p>{{ log.method }} {{ log.path || log.endpoint || "/" }}</p>
                <small>{{ formatDateTime(log.createdAt) }} · 耗时 {{ formatDurationLabel(log.duration) }}</small>
                <small v-if="log.error" class="utility-log-card__error">{{ log.error }}</small>
              </article>
            </div>
          </template>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
* {
  box-sizing: border-box;
}

.chat-page {
  width: 100%;
  height: 100%;
  min-height: 100%;
  padding: 0;
  overflow: hidden;
  background: transparent;
  color: #293145;
}

.window-shell {
  width: 100%;
  height: 100%;
  min-height: 0;
  display: block;
  border-radius: 20px;
  overflow: hidden;
  border: 1px solid rgba(224, 232, 244, 0.86);
  background: rgba(255, 255, 255, 0.75);
  backdrop-filter: blur(18px) saturate(1.05);
}

.chat-app {
  width: 100%;
  height: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: 206px 320px minmax(0, 1fr);
  border-radius: 0;
  overflow: hidden;
  border: 0;
  background: rgba(248, 251, 255, 0.86);
  box-shadow: none;
}

.sidebar-icons {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0;
  padding: 10px 12px 10px;
  border-right: 1px solid #eceff4;
  background: rgba(244, 249, 255, 0.85);
  cursor: move;
}

.sidebar-top {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 12px;
  padding: 0 0 10px;
}

.sidebar-profile {
  width: 100%;
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr);
  align-items: center;
  gap: 10px;
  border: 0;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.68);
  padding: 8px;
  cursor: default;
  position: relative;
}

.sidebar-profile__avatar {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-weight: 700;
  color: #ffffff;
  background: linear-gradient(135deg, #5e7dff, #3f66ea);
  box-shadow: 0 2px 10px rgba(79, 111, 255, 0.26);
}

.sidebar-profile__status {
  position: absolute;
  left: 40px;
  top: 10px;
  width: 9px;
  height: 9px;
  border-radius: 50%;
  border: 2px solid #ffffff;
  background: #35cc7a;
}

.sidebar-profile__meta {
  min-width: 0;
}

.sidebar-profile__meta small {
  display: block;
  font-size: 11px;
  color: #95a2bb;
}

.sidebar-profile__meta strong {
  display: block;
  margin-top: 1px;
  font-size: 18px;
  line-height: 1.1;
  color: #293145;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar-nav {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 8px;
  margin-top: 10px;
}

.window-controls {
  flex-shrink: 0;
  width: auto;
  padding: 4px 0 0 2px;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
}

.window-control {
  width: 12px;
  height: 12px;
  border: 0;
  border-radius: 50%;
  padding: 0;
  cursor: pointer;
  transition: transform 120ms ease, filter 120ms ease;
}

.window-control:hover {
  transform: scale(1.06);
  filter: brightness(0.95);
}

.window-control--close {
  background: #ff5f57;
}

.window-control--minimize {
  background: #febc2e;
}

.window-control--expand {
  background: #28c840;
}

.nav-item {
  position: relative;
  width: 100%;
  min-height: 40px;
  border: 0;
  border-radius: 10px;
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  text-align: left;
  padding: 9px 10px;
  color: #6b788f;
  background: transparent;
  cursor: pointer;
  transition: transform 140ms ease, color 180ms ease, background 180ms ease, box-shadow 180ms ease;
}

.nav-item__icon {
  width: 22px;
  height: 22px;
  display: grid;
  place-items: center;
  border-radius: 7px;
  color: #8a96ad;
  transition: background 180ms ease, color 180ms ease;
}

.nav-item svg {
  width: 17px;
  height: 17px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.nav-item__label {
  font-size: 15px;
  font-weight: 600;
  line-height: 1.1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-item__badge {
  min-width: 28px;
  height: 20px;
  border-radius: 999px;
  padding: 0 8px;
  display: grid;
  place-items: center;
  font-size: 11px;
  font-weight: 700;
  background: rgba(132, 145, 173, 0.26);
  color: #6a7898;
}

.nav-item:hover {
  color: #5f6f8c;
  background: rgba(243, 247, 253, 0.96);
  transform: translateY(-1px);
}

.nav-item.active {
  color: #2f3b57;
  background: rgba(255, 255, 255, 0.9);
  box-shadow:
    0 6px 16px rgba(115, 133, 168, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.nav-item.active .nav-item__icon {
  color: #ffffff;
  background: linear-gradient(135deg, #5d82ff, #4c70ef);
}

.nav-item.active .nav-item__badge {
  background: rgba(117, 132, 164, 0.2);
  color: #66738e;
}

.sidebar-spacer {
  flex: 1;
}

.nav-item--secondary {
  margin-bottom: 6px;
}

.chat-list {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  border-right: 1px solid #eceff4;
  background: #ffffff;
}

.chat-list__header {
  padding: 16px 14px 10px;
  background: #ffffff;
  cursor: move;
}

.search-row {
  margin-top: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 38px;
  gap: 8px;
}

.search-box {
  height: 38px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 8px;
  border: 1px solid #eaedf3;
  background: #f5f7fb;
  color: #9da8be;
  padding: 0 10px;
}

.search-box svg,
.search-add svg,
.header-btn svg,
.composer-input-action svg,
.composer-btn svg,
.composer-send svg {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.search-box input {
  flex: 1;
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: #3f4a63;
  font-size: 14px;
}

.search-box input::placeholder {
  color: #a5afc2;
}

.search-add {
  border: 1px solid #e4e8f0;
  border-radius: 8px;
  background: #ffffff;
  color: #8d99b2;
  display: grid;
  place-items: center;
  cursor: pointer;
}

.search-add:hover {
  color: #4f6fff;
  border-color: #d7dff0;
  background: #f7f9ff;
}

.agent-pane-tabs {
  margin-top: 12px;
  padding: 4px;
  border-radius: 999px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px;
  background: #f1f3f8;
}

.agent-pane-tab {
  height: 32px;
  border: 0;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  color: #8f9ab0;
  background: transparent;
  cursor: pointer;
  transition: color 160ms ease, background 160ms ease, box-shadow 160ms ease;
}

.agent-pane-tab.active {
  color: #3f4a63;
  background: #ffffff;
  box-shadow: 0 1px 4px rgba(103, 116, 146, 0.15);
}

.chat-list__body {
  min-height: 0;
  overflow: auto;
  padding: 6px 10px 10px;
  background: #ffffff;
}

.agent-item {
  width: 100%;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  text-align: left;
  padding: 8px;
  display: grid;
  grid-template-columns: 46px minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
  cursor: pointer;
  transition: background 180ms ease, border-color 180ms ease;
}

.agent-item + .agent-item {
  margin-top: 4px;
}

.agent-item:hover {
  border-color: #e6eaf2;
  background: #f7f9fd;
}

.agent-item.active {
  border-color: #6e86ff;
  background: #4f6fff;
  color: #fff;
}

.agent-avatar {
  position: relative;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  font-weight: 800;
  color: #5a6b8a;
  background: #eaf0fb;
}

.agent-avatar--group {
  border-radius: 8px;
}

.status-dot {
  position: absolute;
  right: 1px;
  bottom: 1px;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid #fff;
  background: #26c45d;
}

.status-dot[data-tone="busy"] {
  background: #ffb62e;
}

.status-dot[data-tone="offline"] {
  background: #94a3b8;
}

.agent-content {
  min-width: 0;
}

.agent-top-line {
  display: flex;
  align-items: center;
  gap: 6px;
}

.agent-top-line strong {
  display: block;
  font-size: 14px;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.agent-channel {
  flex-shrink: 0;
  font-size: 10px;
  border-radius: 999px;
  padding: 1px 6px;
  background: #edf1f8;
  color: #6a7898;
  text-transform: lowercase;
}

.agent-item.active .agent-channel {
  background: rgba(255, 255, 255, 0.24);
  color: #f7fbff;
}

.agent-preview {
  margin: 2px 0 1px;
  font-size: 12px;
  color: #6f7f9d;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.agent-status {
  font-size: 11px;
  color: #8996b0;
}

.agent-item.active .agent-preview,
.agent-item.active .agent-status {
  color: rgba(255, 255, 255, 0.86);
}

.agent-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 4px;
  color: #8a97b0;
  font-size: 11px;
}

.agent-item.active .agent-meta {
  color: rgba(255, 255, 255, 0.84);
}

.agent-unread {
  min-width: 18px;
  height: 18px;
  border-radius: 999px;
  display: grid;
  place-items: center;
  font-size: 10px;
  color: #fff;
  background: #2f79ed;
}

.agent-item.active .agent-unread {
  background: rgba(255, 255, 255, 0.28);
}

.list-empty {
  margin: 16px 8px;
  color: #8193ad;
  font-size: 13px;
}

.chat-window {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  min-height: 0;
  background: #fafbfe;
}

.chat-window__header {
  padding: 14px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border-bottom: 1px solid #eceff4;
  background: #ffffff;
  cursor: move;
}

.chat-agent-header {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.chat-agent-header__avatar {
  width: 42px;
  height: 42px;
  border-radius: 12px;
  display: grid;
  place-items: center;
  font-weight: 800;
  color: #2e5ca3;
  background: linear-gradient(135deg, #d8ecff, #b5d6ff);
}

.chat-agent-header strong {
  display: block;
  font-size: 18px;
  line-height: 1.1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chat-agent-header p {
  margin: 3px 0 0;
  color: #8094b4;
  font-size: 12px;
}

.chat-window__actions {
  display: flex;
  gap: 4px;
}

.header-btn {
  width: 34px;
  height: 34px;
  border: 0;
  border-radius: 10px;
  display: grid;
  place-items: center;
  color: #7c91b0;
  background: transparent;
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease, opacity 150ms ease;
}

.header-btn:hover {
  color: #2c6de0;
  background: #e9f1ff;
}

.header-btn.is-active {
  color: #2c6de0;
  background: #e9f1ff;
}

.header-btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.chat-window__content {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  position: relative;
}

.chat-window__content--settings-open {
  grid-template-columns: minmax(0, 1fr) 360px;
}

.chat-window__messages {
  grid-column: 1;
  grid-row: 1;
  min-height: 0;
  overflow: auto;
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chat-settings-sidebar {
  grid-column: 2;
  grid-row: 1;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  border-left: 1px solid #e7edf7;
  background: linear-gradient(180deg, #fbfdff 0%, #f7fbff 100%);
  display: flex;
  flex-direction: column;
}

.chat-settings-sidebar__header {
  padding: 14px 14px 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #eef2f8;
}

.chat-settings-sidebar__header--plain {
  padding: 10px 12px 8px;
  justify-content: flex-end;
}

.chat-settings-sidebar__close {
  width: 30px;
  height: 30px;
  border: 0;
  border-radius: 8px;
  color: #6f82a3;
  background: transparent;
  font-size: 22px;
  line-height: 1;
  display: grid;
  place-items: center;
  cursor: pointer;
}

.chat-settings-sidebar__close:hover {
  color: #2a6be0;
  background: #e9f1ff;
}

.chat-settings-sidebar__body {
  padding: 8px 12px 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.chat-settings-agent-card {
  border: 0;
  border-radius: 0;
  background: transparent;
  padding: 4px 2px 10px;
  box-shadow: none;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.chat-settings-agent-card__head {
  display: grid;
  grid-template-columns: 54px minmax(0, 1fr);
  gap: 10px;
  align-items: center;
}

.chat-settings-agent-card__avatar {
  width: 54px;
  height: 54px;
  border-radius: 16px;
  display: grid;
  place-items: center;
  font-size: 20px;
  font-weight: 800;
  color: #ffffff;
  background: linear-gradient(135deg, #7f5bff, #4e75ee);
}

.chat-settings-agent-card__identity strong {
  display: block;
  font-size: 21px;
  line-height: 1.1;
  color: #222f48;
}

.chat-settings-agent-card__identity p {
  margin: 4px 0 0;
  color: #7a8ba7;
  font-size: 12px;
}

.chat-settings-chip-row {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chat-settings-status-chip,
.chat-settings-soft-chip {
  padding: 2px 9px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
}

.chat-settings-status-chip {
  color: #1a8250;
  background: #dcfce8;
}

.chat-settings-status-chip[data-tone="busy"] {
  color: #956607;
  background: #fff3d2;
}

.chat-settings-status-chip[data-tone="offline"] {
  color: #576780;
  background: #e8edf4;
}

.chat-settings-soft-chip {
  color: #6c7990;
  background: #eceff5;
}

.chat-settings-list {
  margin: 0;
  border-top: 1px solid #e8eef9;
  padding-top: 2px;
}

.chat-settings-list__row {
  display: grid;
  grid-template-columns: 70px minmax(0, 1fr);
  gap: 8px;
  align-items: start;
  padding: 6px 0;
  border-top: 1px solid #e9eff9;
}

.chat-settings-list__row:first-of-type {
  border-top: 0;
}

.chat-settings-list__row dt {
  margin: 0;
  color: #8a98ae;
  font-size: 11px;
}

.chat-settings-list__row dd {
  margin: 0;
  color: #2c3a54;
  font-size: 12px;
  line-height: 1.42;
  word-break: break-word;
}

.chat-settings-resource-quick {
  margin-top: 2px;
}

.chat-settings-resource-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.chat-settings-resource-action {
  width: 100%;
  border: 1px solid #bfd7f8;
  border-radius: 11px;
  background: #ffffff;
  min-height: 50px;
  padding: 7px 8px;
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) 12px;
  align-items: center;
  gap: 0 7px;
  text-align: left;
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease, transform 140ms ease, box-shadow 140ms ease;
}

.chat-settings-resource-action:hover {
  border-color: #79abea;
  background: #edf5ff;
  box-shadow: 0 2px 6px rgba(95, 131, 190, 0.14);
  transform: translateY(-1px);
}

.chat-settings-resource-action__icon {
  width: 24px;
  height: 24px;
  border-radius: 7px;
  display: grid;
  place-items: center;
  color: #2f5d97;
  background: #e9f3ff;
  font-size: 11px;
  font-weight: 700;
}

.chat-settings-resource-action__content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.chat-settings-resource-action__main {
  color: #24497b;
  font-size: 12px;
  font-weight: 700;
  line-height: 1.22;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chat-settings-resource-action__sub {
  color: #6d83a5;
  font-size: 10px;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chat-settings-resource-action__arrow {
  color: #6489bd;
  font-size: 14px;
  font-weight: 700;
  justify-self: end;
}

.chat-settings-output {
  border-top: 1px solid #e8eef9;
  padding-top: 8px;
}

.chat-settings-output h5 {
  margin: 0 0 6px;
  color: #7184a3;
  font-size: 11px;
  font-weight: 600;
}

.chat-settings-recent-output {
  white-space: pre-wrap;
  line-height: 1.45;
  max-height: 132px;
  overflow: auto;
  padding: 8px 9px;
  border-radius: 10px;
  background: #edf3fc;
  color: #2c3a54;
  font-size: 12px;
}

.chat-settings-sidebar__empty {
  margin: 14px;
  border: 1px dashed #d5deec;
  border-radius: 12px;
  padding: 14px;
  color: #7d8fa8;
  font-size: 13px;
}

.message-row {
  max-width: 72%;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.message-row--user {
  align-self: flex-end;
}

.message-row--assistant,
.message-row--system {
  align-self: flex-start;
}

.message-bubble {
  border-radius: 16px;
  padding: 10px 13px;
  font-size: 14px;
  line-height: 1.45;
  color: #1f2a44;
  background: #ffffff;
  box-shadow: 0 1px 4px rgba(61, 89, 130, 0.1);
  white-space: pre-wrap;
  word-break: break-word;
}

.message-row--user .message-bubble {
  color: #fff;
  background: linear-gradient(135deg, #60a2ff, #3d88f2);
}

.message-row--error .message-bubble {
  color: #b42318;
  background: #fff1f1;
  border: 1px solid #f4b4b4;
}

.message-time {
  align-self: flex-end;
  font-size: 11px;
  color: #8ea1bc;
}

.chat-window__composer {
  padding: 12px 16px;
  border-top: 1px solid #eceff4;
  background: #ffffff;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 36px 40px;
  gap: 8px;
  align-items: center;
}

.composer-input-shell {
  position: relative;
  min-width: 0;
}

.chat-window__composer input {
  height: 40px;
  border: 1px solid #cfdced;
  border-radius: 12px;
  outline: 0;
  background: #fff;
  color: #22314d;
  padding: 0 12px 0 40px;
  font-size: 14px;
  width: 100%;
}

.chat-window__composer input:disabled {
  background: #f0f4fb;
  color: #90a1bb;
}

.composer-input-action {
  position: absolute;
  left: 6px;
  top: 50%;
  width: 28px;
  height: 28px;
  transform: translateY(-50%);
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: #7d93b3;
  display: grid;
  place-items: center;
  cursor: pointer;
}

.composer-input-action:hover {
  color: #2e6fde;
  background: #edf4ff;
}

.composer-btn,
.composer-send {
  width: 36px;
  height: 36px;
  border: 0;
  border-radius: 10px;
  display: grid;
  place-items: center;
  cursor: pointer;
  color: #7d93b3;
  background: #eaf2ff;
}

.composer-btn:hover {
  color: #2e6fde;
}

.composer-btn--archive {
  background: #edf4ff;
}

.composer-send {
  width: 40px;
  height: 40px;
  color: #fff;
  background: linear-gradient(135deg, #60a2ff, #3d88f2);
}

.composer-send:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.module-board {
  grid-column: 2 / 4;
  padding: 22px;
  overflow: auto;
  background: #ffffff;
}

.module-board__header h2 {
  margin: 0;
  font-size: 30px;
  line-height: 1;
}

.module-board__header {
  cursor: move;
}

.module-board__header p {
  margin: 8px 0 0;
  color: #7084a4;
}

.module-board__metrics {
  margin-top: 16px;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 10px;
}

.module-board__metrics article {
  padding: 12px;
  border: 1px solid #d5e1f0;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.88);
}

.module-board__metrics span {
  color: #7990b0;
  font-size: 12px;
}

.module-board__metrics strong {
  display: block;
  margin-top: 6px;
  font-size: 24px;
}

.module-board__note {
  margin-top: 14px;
  color: #4b5f7f;
}

.module-board__detail {
  margin-top: 10px;
  color: #7b8faa;
  font-size: 13px;
}

.related-resource-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(17, 28, 48, 0.42);
  display: grid;
  place-items: center;
  z-index: 1200;
}

.related-resource-modal {
  width: min(900px, calc(100vw - 40px));
  max-height: calc(100vh - 56px);
  border-radius: 14px;
  border: 1px solid #dbe4f3;
  background: #ffffff;
  box-shadow: 0 18px 48px rgba(26, 44, 78, 0.26);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.related-resource-modal__header {
  padding: 14px 16px 12px;
  border-bottom: 1px solid #e8eef8;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.related-resource-modal__header strong {
  display: block;
  color: #253450;
  font-size: 17px;
}

.related-resource-modal__header p {
  margin: 4px 0 0;
  color: #7c8da8;
  font-size: 12px;
}

.related-resource-modal__actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.related-resource-modal__refresh,
.related-resource-modal__close {
  height: 30px;
  border: 1px solid #d6dfef;
  border-radius: 8px;
  background: #f4f8ff;
  color: #4d607f;
  cursor: pointer;
}

.related-resource-modal__refresh {
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
}

.related-resource-modal__close {
  width: 30px;
  font-size: 20px;
  line-height: 1;
}

.related-resource-modal__refresh:disabled,
.related-resource-modal__close:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.related-resource-modal__body {
  padding: 14px 16px 16px;
  min-height: 0;
  overflow: auto;
}

.related-resource-modal__detail {
  margin: 0 0 12px;
  color: #6f83a3;
  font-size: 12px;
}

.related-resource-filter-input {
  width: 100%;
  height: 34px;
  border: 1px solid #d5deec;
  border-radius: 9px;
  outline: 0;
  background: #ffffff;
  color: #2c3a54;
  padding: 0 10px;
  font-size: 13px;
}

.related-memory-layout {
  display: grid;
  grid-template-columns: 270px minmax(0, 1fr);
  gap: 12px;
  min-height: 440px;
}

.related-memory-nav-pane {
  border: 1px solid #dfe8f6;
  border-radius: 12px;
  background: #f9fbff;
  min-height: 0;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
}

.related-memory-nav-pane__toolbar {
  border-bottom: 1px solid #e7edf8;
  padding: 10px;
  display: grid;
  gap: 7px;
}

.related-memory-nav-pane__toolbar small {
  color: #6d809f;
  font-size: 11px;
}

.related-memory-nav-list {
  min-height: 0;
  overflow: auto;
  padding: 8px;
}

.related-memory-nav-item {
  width: 100%;
  border: 1px solid #dfe8f6;
  border-radius: 10px;
  background: #ffffff;
  text-align: left;
  padding: 8px 9px;
  min-height: 82px;
  max-height: 82px;
  display: grid;
  grid-template-rows: auto 1fr;
  cursor: pointer;
}

.related-memory-nav-item + .related-memory-nav-item {
  margin-top: 6px;
}

.related-memory-nav-item.active {
  border-color: #99bbf3;
  background: #eef5ff;
}

.related-memory-nav-item strong {
  display: block;
  color: #2f3f5b;
  font-size: 13px;
  line-height: 1.35;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.related-memory-nav-item p {
  margin: 3px 0 0;
  color: #6f83a3;
  font-size: 12px;
  line-height: 1.35;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;
}

.related-memory-editor-pane {
  border: 1px solid #dfe8f6;
  border-radius: 12px;
  background: #ffffff;
  min-height: 0;
  display: flex;
}

.related-memory-editor-pane__inner {
  padding: 12px;
  width: 100%;
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 10px;
  min-height: 0;
}

.related-memory-editor-pane__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.related-memory-editor-pane__toolbar small {
  min-width: 0;
  max-width: 72%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #7f8fa9;
  font-size: 11px;
}

.related-memory-editor-pane__meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}

.related-memory-editor-pane__meta span {
  display: block;
  color: #8a97ac;
  font-size: 11px;
}

.related-memory-editor-pane__meta strong {
  display: block;
  margin-top: 3px;
  color: #2f3f5b;
  font-size: 12px;
  word-break: break-word;
}

.related-memory-editor-pane__textarea {
  width: 100%;
  min-height: 300px;
  border: 1px solid #d4deee;
  border-radius: 10px;
  outline: 0;
  padding: 10px;
  font-size: 13px;
  line-height: 1.45;
  color: #253450;
  resize: vertical;
}

.related-skill-toolbar {
  margin-bottom: 10px;
}

.related-skill-switch {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
  padding: 4px;
  border-radius: 999px;
  background: #f1f3f8;
  margin-bottom: 10px;
}

.related-skill-switch__button {
  height: 32px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: #8190a8;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.related-skill-switch__button em {
  margin-left: 4px;
  font-style: normal;
  font-size: 11px;
  opacity: 0.86;
}

.related-skill-switch__button.is-active {
  background: #ffffff;
  color: #3a4c68;
  box-shadow: 0 1px 4px rgba(96, 110, 140, 0.16);
}

.related-resource-modal__notice,
.related-resource-modal__error {
  margin: 0 0 10px;
  border-radius: 9px;
  padding: 8px 10px;
  font-size: 12px;
}

.related-resource-modal__notice {
  border: 1px solid #bde0c8;
  background: #edfff2;
  color: #2d7c4d;
}

.related-resource-modal__error {
  border: 1px solid #f1c2c2;
  background: #fff3f3;
  color: #a83b3b;
}

.related-resource-modal__empty {
  border: 1px dashed #d2ddec;
  border-radius: 10px;
  padding: 14px;
  color: #7f90aa;
  font-size: 13px;
  text-align: center;
}

.related-resource-modal__empty--small {
  padding: 8px 10px;
  font-size: 12px;
}

.related-resource-section + .related-resource-section {
  margin-top: 12px;
}

.related-resource-section h4 {
  margin: 0 0 8px;
  color: #324667;
  font-size: 13px;
}

.related-switch-row {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr);
  align-items: start;
  gap: 8px;
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #fbfdff;
  padding: 8px 10px;
}

.related-switch-row + .related-switch-row {
  margin-top: 6px;
}

.related-switch-row input {
  margin-top: 2px;
}

.related-switch-row strong {
  display: block;
  color: #2f3f5b;
  font-size: 13px;
}

.related-switch-row p {
  margin: 2px 0 0;
  color: #7587a3;
  font-size: 12px;
}

.related-switch-row--skill small {
  display: block;
  margin-top: 5px;
  color: #8596b1;
  font-size: 11px;
}

.related-resource-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.related-resource-card {
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #fbfdff;
  padding: 10px 11px;
}

.related-resource-card strong {
  display: block;
  color: #2f3f5b;
  font-size: 13px;
}

.related-resource-card p {
  margin: 4px 0 0;
  color: #6f83a3;
  font-size: 12px;
}

.related-resource-card small {
  display: block;
  margin-top: 6px;
  color: #8596b1;
  font-size: 11px;
}

.related-channel-row {
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #fbfdff;
  padding: 10px 11px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.related-channel-row + .related-channel-row {
  margin-top: 6px;
}

.related-channel-row.is-bound {
  border-color: #9bc0ff;
  background: #eef5ff;
}

.related-channel-row strong {
  display: block;
  color: #2f3f5b;
  font-size: 13px;
}

.related-channel-row p {
  margin: 3px 0 0;
  color: #7386a6;
  font-size: 11px;
}

.related-channel-row__action {
  flex-shrink: 0;
  height: 30px;
  border: 1px solid #c9daf6;
  border-radius: 8px;
  background: #eaf3ff;
  color: #315f95;
  padding: 0 11px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.related-channel-row__action:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.utility-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(17, 28, 48, 0.42);
  display: grid;
  place-items: center;
  z-index: 1220;
}

.utility-modal {
  width: min(840px, calc(100vw - 40px));
  max-height: calc(100vh - 56px);
  border-radius: 14px;
  border: 1px solid #dbe4f3;
  background: #ffffff;
  box-shadow: 0 18px 48px rgba(26, 44, 78, 0.26);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.utility-modal__header {
  padding: 14px 16px 12px;
  border-bottom: 1px solid #e8eef8;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.utility-modal__header strong {
  display: block;
  color: #253450;
  font-size: 17px;
}

.utility-modal__header p {
  margin: 4px 0 0;
  color: #7c8da8;
  font-size: 12px;
}

.utility-modal__actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.utility-modal__refresh,
.utility-modal__close {
  height: 30px;
  border: 1px solid #d6dfef;
  border-radius: 8px;
  background: #f4f8ff;
  color: #4d607f;
  cursor: pointer;
}

.utility-modal__refresh {
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
}

.utility-modal__close {
  width: 30px;
  font-size: 20px;
  line-height: 1;
}

.utility-modal__body {
  padding: 14px 16px 16px;
  min-height: 0;
  overflow: auto;
}

.utility-modal__detail {
  margin: 0 0 12px;
  color: #6f83a3;
  font-size: 12px;
}

.utility-modal__notice,
.utility-modal__error {
  margin: 0 0 10px;
  border-radius: 9px;
  padding: 8px 10px;
  font-size: 12px;
}

.utility-modal__notice {
  border: 1px solid #bde0c8;
  background: #edfff2;
  color: #2d7c4d;
}

.utility-modal__error {
  border: 1px solid #f1c2c2;
  background: #fff3f3;
  color: #a83b3b;
}

.utility-modal__empty {
  border: 1px dashed #d2ddec;
  border-radius: 10px;
  padding: 14px;
  color: #7f90aa;
  font-size: 13px;
  text-align: center;
}

.utility-history-current {
  border: 1px solid #dfe8f6;
  border-radius: 11px;
  background: #f7fbff;
  padding: 10px 11px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.utility-history-current strong {
  display: block;
  color: #30405f;
  font-size: 13px;
}

.utility-history-current p {
  margin: 4px 0 0;
  color: #6f83a3;
  font-size: 12px;
}

.utility-history-current__action {
  height: 30px;
  border: 1px solid #c9daf6;
  border-radius: 8px;
  background: #eaf3ff;
  color: #315f95;
  padding: 0 11px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  flex-shrink: 0;
}

.utility-history-current__action:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.utility-history-list {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.utility-history-card {
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #fbfdff;
  padding: 10px 11px;
}

.utility-history-card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.utility-history-card__head strong {
  color: #2f3f5b;
  font-size: 13px;
}

.utility-history-card__head small {
  color: #8596b1;
  font-size: 11px;
}

.utility-history-card p {
  margin: 5px 0 0;
  color: #6f83a3;
  font-size: 12px;
}

.utility-history-card__foot {
  margin-top: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.utility-history-card__foot span {
  color: #8a98ae;
  font-size: 11px;
}

.utility-history-card__action {
  height: 28px;
  border: 1px solid #c9daf6;
  border-radius: 8px;
  background: #eaf3ff;
  color: #315f95;
  padding: 0 10px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.utility-log-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.utility-log-card {
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #fbfdff;
  padding: 10px 11px;
}

.utility-log-card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.utility-log-card__head strong {
  color: #2f3f5b;
  font-size: 13px;
}

.utility-log-status {
  min-width: 54px;
  text-align: center;
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 700;
  color: #2f5f95;
  background: #e8f2ff;
}

.utility-log-status[data-tone="warn"] {
  color: #9a6206;
  background: #fff2d5;
}

.utility-log-status[data-tone="error"] {
  color: #b42318;
  background: #fde8e8;
}

.utility-log-card p {
  margin: 5px 0 0;
  color: #2f3f5b;
  font-size: 12px;
}

.utility-log-card small {
  display: block;
  margin-top: 5px;
  color: #7f90aa;
  font-size: 11px;
}

.utility-log-card__error {
  color: #b42318;
  white-space: pre-wrap;
}

@media (max-width: 1100px) {
  .chat-app {
    grid-template-columns: 186px 280px minmax(0, 1fr);
  }

  .chat-window__content--settings-open {
    grid-template-columns: minmax(0, 1fr) 316px;
  }

  .related-memory-layout {
    grid-template-columns: 240px minmax(0, 1fr);
  }
}

@media (max-width: 880px) {
  .chat-page {
    padding: 0;
  }

  .window-shell {
    border-radius: 0;
    border: 0;
  }

  .chat-app {
    border: 0;
    grid-template-columns: 78px minmax(0, 1fr);
  }

  .sidebar-icons {
    padding: 8px 8px 10px;
  }

  .sidebar-profile {
    grid-template-columns: 1fr;
    justify-items: center;
    padding: 6px;
  }

  .sidebar-profile__meta,
  .nav-item__label,
  .nav-item__badge {
    display: none;
  }

  .sidebar-nav {
    align-items: center;
  }

  .nav-item {
    width: 42px;
    min-height: 42px;
    grid-template-columns: 1fr;
    justify-items: center;
    padding: 0;
    border-radius: 12px;
  }

  .nav-item__icon {
    width: 22px;
    height: 22px;
  }

  .chat-list {
    display: none;
  }

  .chat-window {
    grid-column: 2;
  }

  .chat-window__composer {
    grid-template-columns: minmax(0, 1fr) 34px 38px;
    padding: 10px 12px;
    gap: 6px;
  }

  .chat-window__content--settings-open {
    grid-template-columns: minmax(0, 1fr);
  }

  .chat-settings-sidebar {
    grid-column: 1;
    justify-self: end;
    width: min(100%, 320px);
    border-left: 1px solid #dfe7f4;
    box-shadow: -8px 0 24px rgba(57, 78, 114, 0.16);
    z-index: 4;
  }

  .chat-settings-agent-card__identity strong {
    font-size: 19px;
  }

  .related-resource-modal {
    width: calc(100vw - 20px);
    max-height: calc(100vh - 20px);
  }

  .related-resource-modal__header {
    padding: 12px;
  }

  .related-resource-modal__body {
    padding: 12px;
  }

  .related-memory-layout {
    grid-template-columns: minmax(0, 1fr);
    min-height: 0;
  }

  .related-memory-nav-pane {
    max-height: 220px;
  }

  .related-memory-editor-pane__meta {
    grid-template-columns: 1fr;
  }

  .related-channel-row {
    align-items: flex-start;
    flex-direction: column;
  }

  .utility-modal {
    width: calc(100vw - 20px);
    max-height: calc(100vh - 20px);
  }

  .utility-modal__header {
    padding: 12px;
  }

  .utility-modal__body {
    padding: 12px;
  }

  .utility-history-current {
    align-items: flex-start;
    flex-direction: column;
  }

  .module-board {
    grid-column: 2;
  }
}
</style>
