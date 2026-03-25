<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { sendOpenClawChat, type OpenClawMessage } from "../services/openclaw";
import appLogoUrl from "../../images/logo.png";
import packageJson from "../../package.json";
import {
  DEFAULT_TASK_PROJECT_NAME,
  createTaskDraft,
  loadMemories,
  loadStaff,
  loadTasks,
  upsertTask,
  type MemoryRecord,
  type TaskRecord
} from "../services/consoleData";
import { loadAgencyRosterZh, type AgencyRosterDivision, type AgencyRosterRole } from "../services/agencyRoster";
import {
  fetchSkillTop50,
  fetchSkillsByCategory,
  fetchSkillsGlobal,
  type SkillMarketCategory,
  type SkillMarketSkill,
  type SkillMarketSortBy
} from "../services/skillsMarket";
import { loadAgentDetailMarkdownZh } from "../services/agentDetail";

type SidebarSection = "chat" | "dashboard" | "recruitment" | "skills" | "tasks";
type AgentGroupKind = "staff" | "group";
type AgentStatusTone = "online" | "busy" | "offline";
type ChatRole = "assistant" | "user" | "system";
type ChatStatus = "pending" | "done" | "error";
type AgentPaneTab = "staff" | "group";
type RelatedResourceTarget = "memory" | "skills" | "tools" | "model" | "channel" | "schedule";
type RelatedSkillCategory = "builtIn" | "installed";
type UtilityModalType = "history" | "logs";
type UtilityLogTab = "runtime" | "errorAnalysis";
type UtilityLogDetailTab = "request" | "response" | "stream" | "raw";
type UtilityLogCategory = "all" | "message" | "tool";
type SidebarSettingsMenuGroupId = "general" | "about";
type SidebarSettingsAppearance = "system" | "light" | "dark";
type SidebarSettingsLanguage = "zh-CN" | "en-US";
type SidebarSettingsMenuGroup = {
  id: SidebarSettingsMenuGroupId;
  label: string;
};
type OpenClawProviderProtocol = "openai" | "anthropic";
type OpenClawProviderApiKind = "openai-completions" | "openai-responses" | "anthropic-messages";
type SkillMarketSectionCategory = "top" | SkillMarketCategory;
type SkillMarketCategoryOption = {
  id: SkillMarketSectionCategory;
  label: string;
  hint: string;
  apiCategory: SkillMarketCategory | null;
};
type SkillMarketListResultSnapshot = {
  skills: SkillMarketSkill[];
  total: number;
};
type RoleWorkflowSavedVersion = {
  id: string;
  contentZh: string;
  savedAt: number;
};
type RoleWorkflowOverride = {
  nameZh?: string;
  workflowZh?: string;
  detailContentZh?: string;
  detailVersions?: RoleWorkflowSavedVersion[];
};
type RoleWorkflowModalBase = {
  role: AgencyRosterRole;
  divisionTitleZh: string;
  groupTitleZh: string | null;
};
type RoleWorkflowDetailNotice = {
  tone: "success" | "error";
  text: string;
};

type TaskModuleView = "projects" | "board";
type TaskBoardStatus = TaskRecord["status"];
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

type OpenClawPlatformSnapshotItem = {
  id: string;
  providerId: string;
  name: string;
  protocol: string;
  baseUrl: string;
  pathPrefix: string;
  apiPath: string;
  apiKey: string;
  model: string;
};

type OpenClawPlatformSnapshotResponse = {
  sourcePath: string;
  detail: string;
  platforms: OpenClawPlatformSnapshotItem[];
};

type RelatedModelDraft = {
  providerId: string;
  providerName: string;
  protocol: OpenClawProviderProtocol;
  apiKind: OpenClawProviderApiKind;
  baseUrl: string;
  model: string;
  apiKey: string;
  apiPath: string;
};

type ProxyConfigDraft = {
  providerId: string;
  baseUrl: string;
  apiKey: string;
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
  sessionId?: string;
  platformId?: string;
  platformName: string;
  protocol?: string;
  method: string;
  endpoint: string;
  baseUrl?: string;
  path?: string;
  requestBody?: string;
  responseStatus: number;
  responseBody?: string;
  streamSummary?: string;
  duration: number;
  firstTokenTime?: number;
  promptTokens?: number;
  completionTokens?: number;
  totalTokens?: number;
  cacheReadInputTokens?: number;
  createdAt: number;
  error?: string;
};

type OpenClawMessageLogResponse = {
  detail: string;
  logs: OpenClawMessageLogItem[];
};

type GatewayHealthSnapshotResponse = {
  status: string;
  checkedUrl: string | null;
  detail: string | null;
  latencyMs: number | null;
  gatewayPort: number | null;
};

type LobsterSnapshotResponse = {
  openclawInstalled: boolean;
  openclawVersion: string | null;
};

type LobsterActionResult = {
  action: string;
  command: string;
  success: boolean;
  detail: string;
  exitCode: number | null;
  stdout: string;
  stderr: string;
  durationMs: number;
  backupPath: string | null;
};

type StartupInstallStepStatus = "pending" | "installing" | "done" | "failed";
type StartupInstallStep = {
  id: string;
  title: string;
  etaLabel: string;
  status: StartupInstallStepStatus;
};

type DashboardHealthTone = "online" | "warn" | "offline" | "neutral";
type DashboardIconName =
  | "platform"
  | "request"
  | "gateway"
  | "latency"
  | "tokenToday"
  | "tokenWeek"
  | "tokenTotal"
  | "failure"
  | "runtime"
  | "channel"
  | "model"
  | "uptime"
  | "memory"
  | "message"
  | "connected"
  | "staff";

type DashboardMetricCard = {
  id: string;
  label: string;
  value: string;
  detail: string;
  tone: DashboardHealthTone;
  icon: DashboardIconName;
};

type DashboardActivityItem = {
  id: string;
  timeLabel: string;
  tag: string;
  summary: string;
  tone: DashboardHealthTone;
};

type TaskBoardColumn = {
  id: TaskBoardStatus;
  label: string;
  subtitle: string;
  emptyText: string;
};

type TaskProjectCard = {
  name: string;
  count: number;
  activeCount: number;
  doneCount: number;
  reviewCount: number;
  updatedAt: number | null;
  isDefault: boolean;
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
  { id: "tasks", label: "任务管理" },
  { id: "dashboard", label: "仪表盘" },
  { id: "recruitment", label: "员工招募" },
  { id: "skills", label: "技能市场" }
];
const sidebarSettingsMenuGroups: SidebarSettingsMenuGroup[] = [
  { id: "general", label: "通用设置" },
  { id: "about", label: "关于我们" }
];
const sidebarSettingsAppearanceOptions: Array<{ id: SidebarSettingsAppearance; label: string }> = [
  { id: "system", label: "跟随系统" },
  { id: "light", label: "浅色" },
  { id: "dark", label: "深色" }
];
const sidebarSettingsLanguageOptions: Array<{ id: SidebarSettingsLanguage; label: string }> = [
  { id: "zh-CN", label: "简体中文" },
  { id: "en-US", label: "English" }
];
const sidebarSettingsShortcutItems: Array<{ id: string; label: string; value: string; note: string }> = [
  { id: "toggle-window", label: "显示或隐藏窗口", value: "Ctrl+` / Alt+`", note: "全局快捷键，可在任意应用中呼出。" },
  { id: "open-chat", label: "打开聊天窗口", value: "Alt+1", note: "快速回到主聊天界面。" }
];
const sidebarSettingsTips: string[] = [
  "在聊天页点击头像可快速查看员工状态与排班信息。",
  "日志页支持复制请求/响应详情，便于排查问题。",
  "技能市场可按分类和评分筛选，优先启用高分技能。"
];
const SIDEBAR_SETTINGS_APPEARANCE_STORAGE_KEY = "keai.desktop-pet.sidebar-settings.appearance";
const SIDEBAR_SETTINGS_LANGUAGE_STORAGE_KEY = "keai.desktop-pet.sidebar-settings.language";
const packageVersionFallback =
  typeof packageJson.version === "string" && packageJson.version.trim() ? packageJson.version.trim() : "0.2.0";

const taskProjectStorageKey = "keai.desktop-pet.task-projects";
const taskStatusFlow: TaskBoardStatus[] = ["todo", "in_progress", "in_review", "done", "cancelled"];
const taskBoardColumns: TaskBoardColumn[] = [
  { id: "todo", label: "To do", subtitle: "待办事项", emptyText: "暂无待办任务。" },
  { id: "in_progress", label: "In progress", subtitle: "进行中", emptyText: "暂无进行中的任务。" },
  { id: "in_review", label: "In review", subtitle: "回顾", emptyText: "暂无待回顾任务。" },
  { id: "done", label: "Done", subtitle: "完成", emptyText: "暂无已完成任务。" },
  { id: "cancelled", label: "Cancelled", subtitle: "取消", emptyText: "暂无已取消任务。" }
];
const agentPaneTabs: Array<{ id: AgentPaneTab; label: string }> = [
  { id: "staff", label: "角色" },
  { id: "group", label: "群组" }
];
const agentAvatarModules = import.meta.glob("../../images/avatar/*.{png,jpg,jpeg,webp,avif,svg}", {
  eager: true,
  import: "default"
}) as Record<string, string>;
const agentAvatarPool = Object.entries(agentAvatarModules)
  .sort(([leftPath], [rightPath]) => leftPath.localeCompare(rightPath, "en"))
  .map(([, url]) => url)
  .filter((url) => typeof url === "string" && url.trim().length > 0);

const CHAT_STORAGE_PREFIX = "keai.desktop-pet.openclaw.chat-history";
const SESSION_STORAGE_PREFIX = "keai.desktop-pet.openclaw.session-id";
const CHAT_ARCHIVE_STORAGE_PREFIX = "keai.desktop-pet.openclaw.chat-archives";
const ROLE_WORKFLOW_OVERRIDES_STORAGE_KEY = "keai.desktop-pet.role-workflow-overrides";
const ROLE_WORKFLOW_INSTALL_PROMPT_PREFIX = "请根据以下角色信息创建 agent:";
const STARTUP_OPENCLAW_STEPS_BASE: Array<Omit<StartupInstallStep, "status">> = [
  { id: "env", title: "检测环境", etaLabel: "" },
  { id: "node", title: "准备 Node.js", etaLabel: "" },
  { id: "install", title: "安装 openClaw", etaLabel: "~30秒" },
  { id: "model", title: "配置 AI 大模型", etaLabel: "~3秒" },
  { id: "gateway", title: "启动并连接服务", etaLabel: "~10秒" }
];
const LOCKED_STARTUP_OPENCLAW_PROVIDER = {
  providerId: "openai",
  protocol: "openai" as OpenClawProviderProtocol,
  apiKind: "openai-responses" as OpenClawProviderApiKind,
  baseUrl: "https://api-vip.codex-for.me/v1",
  model: "gpt-5.4",
  apiKey: "clp_a509beff828ec968d29c8fd3e9a0449b51074ab1d193b9a787c6001dd0627320"
};

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
const taskModuleView = ref<TaskModuleView>("projects");
const taskProjectNames = ref<string[]>([DEFAULT_TASK_PROJECT_NAME]);
const taskProjectInput = ref("");
const taskModuleNotice = ref("");
const taskModuleError = ref("");
const activeTaskProject = ref(DEFAULT_TASK_PROJECT_NAME);
const taskDraftTitle = ref("");
const taskDraftSummary = ref("");
const taskDraftOwner = ref("Commander");
const taskDraftPriority = ref<TaskRecord["priority"]>("p1");
const taskDragTaskId = ref<string | null>(null);
const taskDragOverStatus = ref<TaskBoardStatus | null>(null);
const isAgentSettingsOpen = ref(false);
const isSidebarSettingsModalOpen = ref(false);
const sidebarSettingsActiveGroup = ref<SidebarSettingsMenuGroupId>("general");
const sidebarSettingsAppearance = ref<SidebarSettingsAppearance>(
  normalizeSidebarSettingsAppearance(safeStorageGet(SIDEBAR_SETTINGS_APPEARANCE_STORAGE_KEY))
);
const sidebarSettingsLanguage = ref<SidebarSettingsLanguage>(
  normalizeSidebarSettingsLanguage(safeStorageGet(SIDEBAR_SETTINGS_LANGUAGE_STORAGE_KEY))
);
const sidebarSettingsLaunchOnLoginSupported = ref(false);
const sidebarSettingsLaunchOnLoginEnabled = ref(false);
const sidebarSettingsLaunchOnLoginLoading = ref(false);
const sidebarSettingsAppVersion = ref(`v${packageVersionFallback}`);
const sidebarSettingsFeedbackDraft = ref("");
const sidebarSettingsNotice = ref("");
const sidebarSettingsError = ref("");
const isProxyConfigModalOpen = ref(false);
const proxyConfigLoading = ref(false);
const proxyConfigSaving = ref(false);
const proxyConfigError = ref("");
const proxyConfigNotice = ref("");
const proxyConfigSnapshot = ref<OpenClawPlatformSnapshotResponse | null>(null);
const proxyConfigSelectedProviderId = ref<string | null>(null);
const proxyConfigDraft = ref<ProxyConfigDraft | null>(null);
const relatedResourceModalTarget = ref<RelatedResourceTarget | null>(null);
const relatedResourceModalLoading = ref(false);
const relatedResourceModalSaving = ref(false);
const relatedResourceModalError = ref("");
const relatedResourceModalNotice = ref("");
const relatedMemorySnapshot = ref<SourceFileSnapshotResponse | null>(null);
const relatedSkillsSnapshot = ref<OpenClawSkillsListResponse | null>(null);
const relatedToolsSnapshot = ref<OpenClawToolsListResponse | null>(null);
const relatedModelSnapshot = ref<OpenClawPlatformSnapshotResponse | null>(null);
const relatedModelDraft = ref<RelatedModelDraft | null>(null);
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
const dashboardGatewayHealth = ref<GatewayHealthSnapshotResponse | null>(null);
const dashboardChannelSnapshot = ref<OpenClawChannelAccountsSnapshotResponse | null>(null);
const dashboardPlatformSnapshot = ref<OpenClawPlatformSnapshotResponse | null>(null);
const isDashboardRefreshing = ref(false);
const dashboardRefreshError = ref("");
const dashboardLastRefreshedAt = ref<number | null>(null);
const dashboardJsHeapUsageMb = ref<number | null>(null);
const utilityLogTab = ref<UtilityLogTab>("runtime");
const utilityLogDetailTab = ref<UtilityLogDetailTab>("response");
const utilityRuntimeCategory = ref<UtilityLogCategory>("all");
const utilitySelectedLogId = ref<string | null>(null);
const recruitmentKeyword = ref("");
const recruitmentDivisions = loadAgencyRosterZh();
const roleWorkflowOverrides = ref<Record<string, RoleWorkflowOverride>>(loadRoleWorkflowOverrides());
const roleWorkflowDetailRoleId = ref<string | null>(null);
const roleWorkflowDetailDraft = ref({ contentZh: "" });
const roleWorkflowDetailOriginalContent = ref("");
const roleWorkflowNameZhDraft = ref("");
const roleWorkflowNameZhOriginal = ref("");
const roleWorkflowDetailLoading = ref(false);
const roleWorkflowDetailNotice = ref<RoleWorkflowDetailNotice | null>(null);
const isRoleWorkflowInstalling = ref(false);
const startupOpenClawOverlayVisible = ref(false);
const startupOpenClawInstalling = ref(false);
const startupOpenClawStatusText = ref("正在检测 OpenClaw 安装状态...");
const startupOpenClawInstallError = ref("");
const startupOpenClawSteps = ref<StartupInstallStep[]>(cloneStartupOpenClawSteps());
const startupOpenClawRuntimeLogs = ref("");
const activeSkillMarketCategory = ref<SkillMarketSectionCategory>("top");
const skillMarketSortBy = ref<SkillMarketSortBy>("score");
const skillMarketSearch = ref("");
const skillMarketPage = ref(1);
const skillMarketPageSize = ref(18);
const skillMarketLoading = ref(false);
const skillMarketError = ref("");
const skillMarketTopSkills = ref<SkillMarketSkill[]>([]);
const skillMarketTopTotal = ref(0);
const skillMarketCategorySkills = ref<SkillMarketSkill[]>([]);
const skillMarketCategoryTotal = ref(0);
const skillMarketGlobalSkills = ref<SkillMarketSkill[]>([]);
const skillMarketGlobalTotal = ref(0);
const skillMarketCache = new Map<string, SkillMarketListResultSnapshot>();
const skillMarketGlobalCache = new Map<string, SkillMarketListResultSnapshot>();
const activeSkillMarketDetail = ref<SkillMarketSkill | null>(null);
const skillMarketActionNotice = ref("");
let skillMarketRequestToken = 0;
let roleWorkflowDetailRequestToken = 0;
const utilityRuntimeCategories: Array<{ id: UtilityLogCategory; label: string }> = [
  { id: "all", label: "所有" },
  { id: "message", label: "消息日志" },
  { id: "tool", label: "工具调用日志" }
];
const relatedModelProtocolOptions: Array<{ id: OpenClawProviderApiKind; label: string }> = [
  { id: "openai-completions", label: "OpenAI Completions" },
  { id: "openai-responses", label: "OpenAI Responses" },
  { id: "anthropic-messages", label: "Anthropic Messages" }
];

const skillMarketCategories: SkillMarketCategoryOption[] = [
  { id: "top", label: "热门推荐", hint: "Top 50 热门技能", apiCategory: null },
  { id: "ai-intelligence", label: "AI 智能", hint: "智能体与推理增强", apiCategory: "ai-intelligence" },
  { id: "developer-tools", label: "开发工具", hint: "研发与工程效率", apiCategory: "developer-tools" },
  { id: "productivity", label: "效率协作", hint: "办公与流程自动化", apiCategory: "productivity" },
  { id: "data-analytics", label: "数据分析", hint: "洞察与可视化", apiCategory: "data-analytics" },
  { id: "content-creation", label: "内容创作", hint: "文案、媒体与运营", apiCategory: "content-creation" },
  { id: "security-compliance", label: "安全合规", hint: "风险与治理", apiCategory: "security-compliance" },
  { id: "communication-collaboration", label: "沟通协同", hint: "协作与集成", apiCategory: "communication-collaboration" }
];

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

function normalizeSidebarSettingsAppearance(raw: string | null): SidebarSettingsAppearance {
  if (raw === "light" || raw === "dark" || raw === "system") {
    return raw;
  }
  return "system";
}

function normalizeSidebarSettingsLanguage(raw: string | null): SidebarSettingsLanguage {
  if (raw === "zh-CN" || raw === "en-US") {
    return raw;
  }
  return "zh-CN";
}

function normalizeRoleWorkflowSavedVersion(raw: unknown): RoleWorkflowSavedVersion | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }
  const candidate = raw as Record<string, unknown>;
  const id = typeof candidate.id === "string" ? candidate.id.trim() : "";
  const contentZh = typeof candidate.contentZh === "string" ? candidate.contentZh : "";
  const savedAt =
    typeof candidate.savedAt === "number" && Number.isFinite(candidate.savedAt)
      ? Math.floor(candidate.savedAt)
      : 0;
  if (!id || !savedAt) {
    return null;
  }
  return {
    id,
    contentZh,
    savedAt
  };
}

function normalizeRoleWorkflowOverride(raw: unknown): RoleWorkflowOverride | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }
  const candidate = raw as Record<string, unknown>;
  const nameZh = typeof candidate.nameZh === "string" ? candidate.nameZh : undefined;
  const workflowZh = typeof candidate.workflowZh === "string" ? candidate.workflowZh : undefined;
  const detailContentZh = typeof candidate.detailContentZh === "string" ? candidate.detailContentZh : undefined;
  const detailVersions = Array.isArray(candidate.detailVersions)
    ? candidate.detailVersions
        .map(normalizeRoleWorkflowSavedVersion)
        .filter((version): version is RoleWorkflowSavedVersion => Boolean(version))
        .sort((left, right) => right.savedAt - left.savedAt)
        .slice(0, 3)
    : [];

  if (!nameZh && !workflowZh && !detailContentZh && detailVersions.length === 0) {
    return null;
  }

  const normalized: RoleWorkflowOverride = {};
  if (nameZh) {
    normalized.nameZh = nameZh;
  }
  if (workflowZh) {
    normalized.workflowZh = workflowZh;
  }
  if (typeof detailContentZh === "string") {
    normalized.detailContentZh = detailContentZh;
  }
  if (detailVersions.length > 0) {
    normalized.detailVersions = detailVersions;
  }
  return normalized;
}

function loadRoleWorkflowOverrides() {
  const raw = safeStorageGet(ROLE_WORKFLOW_OVERRIDES_STORAGE_KEY);
  if (!raw) {
    return {} as Record<string, RoleWorkflowOverride>;
  }
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {} as Record<string, RoleWorkflowOverride>;
    }
    const normalizedEntries = Object.entries(parsed as Record<string, unknown>)
      .map(([roleId, override]) => [roleId, normalizeRoleWorkflowOverride(override)] as const)
      .filter((entry): entry is readonly [string, RoleWorkflowOverride] => Boolean(entry[1]));
    return Object.fromEntries(normalizedEntries) as Record<string, RoleWorkflowOverride>;
  } catch {
    return {} as Record<string, RoleWorkflowOverride>;
  }
}

function persistRoleWorkflowOverrides() {
  safeStorageSet(ROLE_WORKFLOW_OVERRIDES_STORAGE_KEY, JSON.stringify(roleWorkflowOverrides.value));
}

function normalizeTaskProjectName(value: string) {
  return value.trim();
}

function buildTaskProjectList(items: string[]) {
  const unique = new Set<string>();
  const result = [DEFAULT_TASK_PROJECT_NAME];
  for (const item of items) {
    const normalized = normalizeTaskProjectName(item);
    if (!normalized || normalized === DEFAULT_TASK_PROJECT_NAME || unique.has(normalized)) {
      continue;
    }
    unique.add(normalized);
    result.push(normalized);
  }
  return result;
}

function loadTaskProjectsFromStorage() {
  const parsed = (() => {
    try {
      return JSON.parse(safeStorageGet(taskProjectStorageKey) ?? "[]");
    } catch {
      return [];
    }
  })();
  if (!Array.isArray(parsed)) {
    return [DEFAULT_TASK_PROJECT_NAME];
  }
  return buildTaskProjectList(parsed.filter((item): item is string => typeof item === "string"));
}

function saveTaskProjectsToStorage(projects: string[]) {
  const payload = buildTaskProjectList(projects).filter((project) => project !== DEFAULT_TASK_PROJECT_NAME);
  safeStorageSet(taskProjectStorageKey, JSON.stringify(payload));
}

function isTaskInProject(task: TaskRecord, projectName: string) {
  const normalizedProject = normalizeTaskProjectName(task.project);
  if (projectName === DEFAULT_TASK_PROJECT_NAME) {
    return !normalizedProject || normalizedProject === DEFAULT_TASK_PROJECT_NAME;
  }
  return normalizedProject === projectName;
}

function getTaskStatusWeight(status: TaskBoardStatus) {
  const weightMap: Record<TaskBoardStatus, number> = {
    todo: 0,
    in_progress: 1,
    in_review: 2,
    done: 3,
    cancelled: 4
  };
  return weightMap[status];
}

function getTaskPriorityWeight(priority: TaskRecord["priority"]) {
  if (priority === "p0") return 0;
  if (priority === "p1") return 1;
  return 2;
}

function sortTaskRecords(items: TaskRecord[]) {
  return [...items].sort((left, right) => {
    const statusWeight = getTaskStatusWeight(left.status) - getTaskStatusWeight(right.status);
    if (statusWeight !== 0) {
      return statusWeight;
    }
    const priorityWeight = getTaskPriorityWeight(left.priority) - getTaskPriorityWeight(right.priority);
    if (priorityWeight !== 0) {
      return priorityWeight;
    }
    if (left.dueAt !== right.dueAt) {
      return left.dueAt - right.dueAt;
    }
    return right.updatedAt - left.updatedAt;
  });
}

function sortTaskRecordsForColumn(items: TaskRecord[]) {
  return [...items].sort((left, right) => {
    const priorityWeight = getTaskPriorityWeight(left.priority) - getTaskPriorityWeight(right.priority);
    if (priorityWeight !== 0) {
      return priorityWeight;
    }
    if (left.dueAt !== right.dueAt) {
      return left.dueAt - right.dueAt;
    }
    return right.updatedAt - left.updatedAt;
  });
}

function syncTaskProjectNamesFromTasks() {
  const storageProjects = loadTaskProjectsFromStorage();
  const taskProjects = taskItems.value
    .map((task) => normalizeTaskProjectName(task.project))
    .filter((item) => item && item !== DEFAULT_TASK_PROJECT_NAME);
  const nextProjects = buildTaskProjectList([...storageProjects, ...taskProjects]);
  taskProjectNames.value = nextProjects;
  saveTaskProjectsToStorage(nextProjects);
  if (!taskProjectNames.value.includes(activeTaskProject.value)) {
    activeTaskProject.value = DEFAULT_TASK_PROJECT_NAME;
  }
}

function resetTaskDraftForm() {
  taskDraftTitle.value = "";
  taskDraftSummary.value = "";
  taskDraftOwner.value = "Commander";
  taskDraftPriority.value = "p1";
}

function updateTaskRecord(nextTask: Omit<TaskRecord, "updatedAt">) {
  taskItems.value = upsertTask(taskItems.value, nextTask);
  syncTaskProjectNamesFromTasks();
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

function cloneStartupOpenClawSteps() {
  return STARTUP_OPENCLAW_STEPS_BASE.map((step) => ({
    ...step,
    status: "pending" as StartupInstallStepStatus
  }));
}

function resetStartupOpenClawSteps() {
  startupOpenClawSteps.value = cloneStartupOpenClawSteps();
}

function setStartupOpenClawStepStatus(stepIndex: number, status: StartupInstallStepStatus) {
  startupOpenClawSteps.value = startupOpenClawSteps.value.map((step, index) =>
    index === stepIndex
      ? {
          ...step,
          status
        }
      : step
  );
}

function setStartupOpenClawStepInstalling(stepIndex: number) {
  startupOpenClawSteps.value = startupOpenClawSteps.value.map((step, index) => {
    if (index < stepIndex && step.status !== "done") {
      return {
        ...step,
        status: "done" as StartupInstallStepStatus
      };
    }
    if (index === stepIndex) {
      return {
        ...step,
        status: "installing" as StartupInstallStepStatus
      };
    }
    if (index > stepIndex && step.status !== "pending") {
      return {
        ...step,
        status: "pending" as StartupInstallStepStatus
      };
    }
    return step;
  });
}

function markStartupOpenClawStepDone(stepIndex: number) {
  setStartupOpenClawStepStatus(stepIndex, "done");
}

function markStartupOpenClawStepFailed(stepIndex: number) {
  setStartupOpenClawStepStatus(stepIndex, "failed");
}

function getStartupOpenClawStepBadge(step: StartupInstallStep) {
  if (step.status === "failed") {
    return "失败";
  }
  if (step.status === "installing") {
    return step.etaLabel || "进行中";
  }
  if (step.status === "pending") {
    return step.etaLabel;
  }
  return "";
}

function sleepMs(durationMs: number) {
  return new Promise<void>((resolve) => {
    window.setTimeout(resolve, durationMs);
  });
}

async function waitForStartupGatewayOnline(invoke: TauriInvoke, maxAttempts = 18, waitMs = 800) {
  let lastSnapshot: GatewayHealthSnapshotResponse | null = null;
  for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
    try {
      const result = (await invoke("check_openclaw_gateway")) as GatewayHealthSnapshotResponse;
      lastSnapshot = result;
      if ((result.status ?? "").trim().toLowerCase() === "online") {
        return result;
      }
    } catch {
      // Ignore transient check failures while gateway is starting.
    }
    if (attempt < maxAttempts - 1) {
      await sleepMs(waitMs);
    }
  }
  return lastSnapshot;
}

async function applyLockedStartupProviderConfig(invoke: TauriInvoke) {
  await invoke("save_openclaw_provider_config", {
    config: {
      providerId: LOCKED_STARTUP_OPENCLAW_PROVIDER.providerId,
      protocol: LOCKED_STARTUP_OPENCLAW_PROVIDER.protocol,
      apiKind: LOCKED_STARTUP_OPENCLAW_PROVIDER.apiKind,
      baseUrl: LOCKED_STARTUP_OPENCLAW_PROVIDER.baseUrl,
      model: LOCKED_STARTUP_OPENCLAW_PROVIDER.model,
      apiKey: LOCKED_STARTUP_OPENCLAW_PROVIDER.apiKey
    }
  });
}

async function runStartupOpenClawInstall(invoke: TauriInvoke) {
  startupOpenClawOverlayVisible.value = true;
  startupOpenClawInstalling.value = true;
  startupOpenClawInstallError.value = "";
  startupOpenClawRuntimeLogs.value = "";
  resetStartupOpenClawSteps();

  const appendStartupOpenClawRuntimeLogs = (...entries: Array<string | undefined>) => {
    startupOpenClawRuntimeLogs.value = [startupOpenClawRuntimeLogs.value, ...entries]
      .filter((item) => Boolean(item && item.trim()))
      .join("\n\n");
  };

  try {
    startupOpenClawStatusText.value = "正在检测环境...";
    setStartupOpenClawStepInstalling(0);
    let environmentCheckNotice = "";
    try {
      const installGuide = (await invoke("load_lobster_install_guide")) as {
        ready?: boolean;
        checks?: Array<{ title?: string; status?: string; detail?: string }>;
      };
      const failedChecks = (installGuide.checks ?? []).filter((item) => item.status === "failed");
      if (failedChecks.length > 0 || installGuide.ready === false) {
        const failedDetail = failedChecks
          .map((item) => `${item.title || "检查项"}：${item.detail || "未通过"}`)
          .join("；");
        environmentCheckNotice = failedDetail
          ? `环境检查存在未通过项，已继续安装：${failedDetail}`
          : "环境检查存在未通过项，已继续安装。";
      }
    } catch (error) {
      environmentCheckNotice =
        error instanceof Error && error.message.trim()
          ? `环境检查不可用，已继续安装：${error.message}`
          : "环境检查不可用，已继续安装。";
    }
    markStartupOpenClawStepDone(0);
    if (environmentCheckNotice) {
      startupOpenClawRuntimeLogs.value = environmentCheckNotice;
    }

    startupOpenClawStatusText.value = "正在准备 Node.js...";
    setStartupOpenClawStepInstalling(1);
    await sleepMs(240);
    markStartupOpenClawStepDone(1);

    startupOpenClawStatusText.value = "正在安装 openClaw...";
    setStartupOpenClawStepInstalling(2);
    let installResult: LobsterActionResult;
    try {
      installResult = (await invoke("run_lobster_action", { action: "install" })) as LobsterActionResult;
    } catch (error) {
      const detail =
        error instanceof Error && error.message.trim() ? error.message : "openClaw 安装命令执行异常。";
      installResult = {
        action: "install",
        command: "run_lobster_action install",
        success: false,
        detail,
        exitCode: null,
        stdout: "",
        stderr: detail,
        durationMs: 0,
        backupPath: null
      };
    }
    appendStartupOpenClawRuntimeLogs(installResult.command, installResult.stdout, installResult.stderr);
    if (!installResult.success) {
      startupOpenClawStatusText.value = "安装遇到问题，正在自动修复...";
      try {
        const autoFixResult = (await invoke("run_lobster_action", { action: "auto_fix" })) as LobsterActionResult;
        appendStartupOpenClawRuntimeLogs(
          "[auto-fix]",
          autoFixResult.command,
          autoFixResult.stdout,
          autoFixResult.stderr
        );
        if (!autoFixResult.success) {
          appendStartupOpenClawRuntimeLogs("auto-fix 未完全成功，将继续重试安装。", autoFixResult.detail);
        }
      } catch (error) {
        const detail =
          error instanceof Error && error.message.trim() ? error.message : "auto-fix 执行异常，将继续重试安装。";
        appendStartupOpenClawRuntimeLogs("[auto-fix]", detail);
      }

      startupOpenClawStatusText.value = "自动修复完成，正在重试安装...";
      let retryInstallResult: LobsterActionResult;
      try {
        retryInstallResult = (await invoke("run_lobster_action", { action: "install" })) as LobsterActionResult;
      } catch (error) {
        const detail =
          error instanceof Error && error.message.trim() ? error.message : "重试安装命令执行异常。";
        retryInstallResult = {
          action: "install",
          command: "run_lobster_action install",
          success: false,
          detail,
          exitCode: null,
          stdout: "",
          stderr: detail,
          durationMs: 0,
          backupPath: null
        };
      }
      appendStartupOpenClawRuntimeLogs(
        "[retry-install]",
        retryInstallResult.command,
        retryInstallResult.stdout,
        retryInstallResult.stderr
      );
      installResult = retryInstallResult;
      if (!installResult.success) {
        throw new Error(`${installResult.detail || "openClaw 安装失败。"}（已自动修复并重试一次）`);
      }
    }
    markStartupOpenClawStepDone(2);

    startupOpenClawStatusText.value = "正在配置 AI 大模型...";
    setStartupOpenClawStepInstalling(3);
    await applyLockedStartupProviderConfig(invoke);
    markStartupOpenClawStepDone(3);

    startupOpenClawStatusText.value = "正在启动并连接服务...";
    setStartupOpenClawStepInstalling(4);
    const [gateway, snapshot] = await Promise.all([
      waitForStartupGatewayOnline(invoke),
      invoke("load_lobster_snapshot").then((result) => result as LobsterSnapshotResponse)
    ]);
    if (!snapshot.openclawInstalled) {
      throw new Error("OpenClaw 安装校验失败，请重试。");
    }
    if (!gateway || gateway.status.trim().toLowerCase() !== "online") {
      throw new Error(gateway?.detail?.trim() || "OpenClaw 网关未就绪。");
    }
    markStartupOpenClawStepDone(4);

    startupOpenClawStatusText.value = "OpenClaw 已安装并连接完成。";
    startupOpenClawOverlayVisible.value = false;
  } catch (error) {
    const activeStepIndex = startupOpenClawSteps.value.findIndex((step) => step.status === "installing");
    if (activeStepIndex >= 0) {
      markStartupOpenClawStepFailed(activeStepIndex);
    }
    startupOpenClawInstallError.value = error instanceof Error ? error.message : "OpenClaw 安装失败。";
    startupOpenClawStatusText.value = "OpenClaw 自动安装失败，请重试。";
    startupOpenClawOverlayVisible.value = true;
  } finally {
    startupOpenClawInstalling.value = false;
  }
}

async function ensureStartupOpenClawReady() {
  const invoke = getTauriInvoke();
  if (!invoke) {
    return;
  }

  try {
    const snapshot = (await invoke("load_lobster_snapshot")) as LobsterSnapshotResponse;
    if (snapshot.openclawInstalled) {
      try {
        await applyLockedStartupProviderConfig(invoke);
      } catch {
        // Keep app usable even if default provider write fails.
      }
      return;
    }
  } catch (error) {
    startupOpenClawInstallError.value = error instanceof Error ? error.message : "无法检测 OpenClaw 安装状态。";
    startupOpenClawStatusText.value = "OpenClaw 安装检测失败，请重试。";
    startupOpenClawOverlayVisible.value = true;
    return;
  }

  startupOpenClawStatusText.value = "检测到未安装 OpenClaw，正在自动安装...";
  await runStartupOpenClawInstall(invoke);
}

async function retryStartupOpenClawInstall() {
  const invoke = getTauriInvoke();
  if (!invoke || startupOpenClawInstalling.value) {
    return;
  }
  await runStartupOpenClawInstall(invoke);
}

function dismissStartupOpenClawOverlay() {
  if (startupOpenClawInstalling.value) {
    return;
  }
  startupOpenClawOverlayVisible.value = false;
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

function isLegacyWelcomeMessage(message: AgentChatMessage) {
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

function createWelcomeMessages(_agent: AgentListItem | null): AgentChatMessage[] {
  return [];
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
      .filter((item) => item.status !== "pending" && !isLegacyWelcomeMessage(item));
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
          .filter((item) => item.status !== "pending" && !isLegacyWelcomeMessage(item));
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
  const stableMessages = chatMessages.value.filter((item) => item.status !== "pending" && !isLegacyWelcomeMessage(item));
  safeStorageSet(chatStorageKeyFor(agentId), JSON.stringify(stableMessages));
  safeStorageSet(sessionStorageKeyFor(agentId), currentSessionId.value || createSessionId());
}

function getOpenClawMessages(items: AgentChatMessage[]): OpenClawMessage[] {
  return items
    .filter((item) => item.status !== "pending" && !isLegacyWelcomeMessage(item))
    .filter((item) => item.role === "assistant" || item.role === "user" || item.role === "system")
    .map((item) => ({ role: item.role, content: item.text }));
}

function getAgentInitial(agent: AgentListItem) {
  const name = stripRoleLabel(agent.displayName);
  return name.charAt(0).toUpperCase() || "A";
}

function hashAgentSeed(value: string) {
  let hash = 0;
  for (let index = 0; index < value.length; index += 1) {
    hash = (hash * 31 + value.charCodeAt(index)) >>> 0;
  }
  return hash;
}

function getAgentAvatarUrl(agent: AgentListItem) {
  if (agentAvatarPool.length === 0) {
    return null;
  }
  const seed = `${agent.agentId}|${stripRoleLabel(agent.displayName)}`.trim().toLowerCase();
  const avatarIndex = hashAgentSeed(seed || "agent") % agentAvatarPool.length;
  return agentAvatarPool[avatarIndex] ?? null;
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
  const stable = messages.filter((item) => item.status !== "pending" && !isLegacyWelcomeMessage(item));
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

function clearSidebarSettingsStatus() {
  sidebarSettingsNotice.value = "";
  sidebarSettingsError.value = "";
}

function closeSidebarSettingsModal() {
  isSidebarSettingsModalOpen.value = false;
  clearSidebarSettingsStatus();
}

function setSidebarSettingsAppearance(next: SidebarSettingsAppearance) {
  sidebarSettingsAppearance.value = next;
  safeStorageSet(SIDEBAR_SETTINGS_APPEARANCE_STORAGE_KEY, next);
  const matched = sidebarSettingsAppearanceOptions.find((item) => item.id === next);
  sidebarSettingsNotice.value = `外观已设置为「${matched?.label ?? next}」。`;
  sidebarSettingsError.value = "";
}

function setSidebarSettingsLanguage(next: SidebarSettingsLanguage) {
  sidebarSettingsLanguage.value = next;
  safeStorageSet(SIDEBAR_SETTINGS_LANGUAGE_STORAGE_KEY, next);
  const matched = sidebarSettingsLanguageOptions.find((item) => item.id === next);
  sidebarSettingsNotice.value = `语言已设置为「${matched?.label ?? next}」。`;
  sidebarSettingsError.value = "";
}

async function loadSidebarSettingsAppVersion() {
  const invoke = getTauriInvoke();
  if (!invoke) {
    return;
  }
  try {
    const version = await invoke("plugin:app|version");
    if (typeof version === "string" && version.trim()) {
      sidebarSettingsAppVersion.value = `v${version.trim()}`;
    }
  } catch {
    // Fall back to package.json version.
  }
}

async function loadSidebarSettingsLaunchOnLoginStatus() {
  const invoke = getTauriInvoke();
  if (!invoke) {
    sidebarSettingsLaunchOnLoginSupported.value = false;
    sidebarSettingsLaunchOnLoginEnabled.value = false;
    return;
  }

  sidebarSettingsLaunchOnLoginLoading.value = true;
  try {
    const enabled = await invoke("plugin:autostart|is_enabled");
    sidebarSettingsLaunchOnLoginEnabled.value = Boolean(enabled);
    sidebarSettingsLaunchOnLoginSupported.value = true;
  } catch {
    sidebarSettingsLaunchOnLoginSupported.value = false;
    sidebarSettingsLaunchOnLoginEnabled.value = false;
  } finally {
    sidebarSettingsLaunchOnLoginLoading.value = false;
  }
}

async function handleSidebarSettingsLaunchOnLoginToggle() {
  if (sidebarSettingsLaunchOnLoginLoading.value) {
    return;
  }

  const invoke = getTauriInvoke();
  if (!invoke || !sidebarSettingsLaunchOnLoginSupported.value) {
    sidebarSettingsNotice.value = "";
    sidebarSettingsError.value = "当前环境不支持开机自动启动设置。";
    return;
  }

  const next = !sidebarSettingsLaunchOnLoginEnabled.value;
  sidebarSettingsLaunchOnLoginLoading.value = true;
  sidebarSettingsNotice.value = "";
  sidebarSettingsError.value = "";

  try {
    await invoke(next ? "plugin:autostart|enable" : "plugin:autostart|disable");
    sidebarSettingsLaunchOnLoginEnabled.value = next;
    sidebarSettingsNotice.value = next ? "已开启开机自动启动。" : "已关闭开机自动启动。";
  } catch (error) {
    sidebarSettingsError.value = error instanceof Error ? error.message : "开机自动启动设置失败。";
  } finally {
    sidebarSettingsLaunchOnLoginLoading.value = false;
  }
}

async function handleSidebarSettingsGroupChange(groupId: SidebarSettingsMenuGroupId) {
  sidebarSettingsActiveGroup.value = groupId;
  clearSidebarSettingsStatus();
  if (groupId === "general") {
    await loadSidebarSettingsLaunchOnLoginStatus();
    return;
  }
  await loadSidebarSettingsAppVersion();
}

async function handleSidebarFeedbackCopy() {
  const payload = sidebarSettingsFeedbackDraft.value.trim();
  if (!payload) {
    sidebarSettingsNotice.value = "";
    sidebarSettingsError.value = "请先填写反馈内容。";
    return;
  }

  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(payload);
    } else if (typeof document !== "undefined") {
      const temp = document.createElement("textarea");
      temp.value = payload;
      temp.setAttribute("readonly", "true");
      temp.style.position = "fixed";
      temp.style.opacity = "0";
      temp.style.left = "-9999px";
      document.body.appendChild(temp);
      temp.focus();
      temp.select();
      const copied = document.execCommand("copy");
      document.body.removeChild(temp);
      if (!copied) {
        throw new Error("复制失败，请手动复制。");
      }
    } else {
      throw new Error("当前环境不支持复制。");
    }
    sidebarSettingsError.value = "";
    sidebarSettingsNotice.value = "反馈内容已复制，可直接粘贴发送。";
  } catch (error) {
    sidebarSettingsNotice.value = "";
    sidebarSettingsError.value = error instanceof Error ? error.message : "复制反馈内容失败。";
  }
}

async function openFeedbackLogsFromSidebarSettings() {
  closeSidebarSettingsModal();
  await openUtilityModal("logs");
}

function toggleAgentSettingsPanel() {
  closeSidebarSettingsModal();
  isAgentSettingsOpen.value = !isAgentSettingsOpen.value;
}

function closeAgentSettingsPanel() {
  isAgentSettingsOpen.value = false;
}

async function openSidebarSettings() {
  activeSection.value = "chat";
  isAgentSettingsOpen.value = false;
  closeUtilityModal();
  closeProxyConfigModal();
  closeRelatedResourceModal();
  isSidebarSettingsModalOpen.value = true;
  clearSidebarSettingsStatus();
  if (sidebarSettingsActiveGroup.value === "general") {
    await loadSidebarSettingsLaunchOnLoginStatus();
    return;
  }
  await loadSidebarSettingsAppVersion();
}

async function openSidebarLogs() {
  activeSection.value = "chat";
  isAgentSettingsOpen.value = false;
  closeSidebarSettingsModal();
  closeProxyConfigModal();
  await openUtilityModal("logs");
}

function buildProxyConfigDraft(platform?: OpenClawPlatformSnapshotItem | null): ProxyConfigDraft {
  return {
    providerId: (platform?.providerId ?? "").trim() || "custom",
    baseUrl: platform?.baseUrl ?? "",
    apiKey: platform?.apiKey ?? ""
  };
}

function syncProxyConfigDraft(preferredProviderId?: string | null) {
  const platforms = proxyConfigSnapshot.value?.platforms ?? [];
  if (platforms.length === 0) {
    proxyConfigSelectedProviderId.value = null;
    proxyConfigDraft.value = buildProxyConfigDraft();
    return;
  }

  const preferred = (preferredProviderId ?? proxyConfigSelectedProviderId.value ?? "").trim();
  const matched =
    (preferred ? platforms.find((item) => equalsIgnoreCase(item.providerId, preferred)) : null) ?? platforms[0];
  proxyConfigSelectedProviderId.value = matched.providerId;
  proxyConfigDraft.value = buildProxyConfigDraft(matched);
}

async function loadProxyConfigSnapshot(preferredProviderId?: string | null) {
  const invoke = getTauriInvoke();
  proxyConfigLoading.value = true;
  proxyConfigError.value = "";
  proxyConfigNotice.value = "";

  try {
    if (!invoke) {
      proxyConfigSnapshot.value = {
        sourcePath: "runtime unavailable",
        detail: "当前环境不支持读取代理配置。",
        platforms: []
      };
      proxyConfigError.value = "当前环境不支持代理配置管理。";
      syncProxyConfigDraft(preferredProviderId);
      return;
    }
    proxyConfigSnapshot.value = (await invoke("load_openclaw_platforms_snapshot")) as OpenClawPlatformSnapshotResponse;
    syncProxyConfigDraft(preferredProviderId);
  } catch (error) {
    proxyConfigSnapshot.value = null;
    proxyConfigError.value = error instanceof Error ? error.message : "读取代理配置失败。";
    proxyConfigSelectedProviderId.value = null;
    proxyConfigDraft.value = buildProxyConfigDraft();
  } finally {
    proxyConfigLoading.value = false;
  }
}

function closeProxyConfigModal() {
  isProxyConfigModalOpen.value = false;
  proxyConfigLoading.value = false;
  proxyConfigSaving.value = false;
  proxyConfigError.value = "";
  proxyConfigNotice.value = "";
}

function handleProxyConfigSelect(providerId: string) {
  const platforms = proxyConfigSnapshot.value?.platforms ?? [];
  const matched = platforms.find((item) => equalsIgnoreCase(item.providerId, providerId));
  if (!matched) {
    return;
  }
  proxyConfigSelectedProviderId.value = matched.providerId;
  proxyConfigDraft.value = buildProxyConfigDraft(matched);
  proxyConfigNotice.value = "";
  proxyConfigError.value = "";
}

function handleProxyConfigCreate() {
  proxyConfigSelectedProviderId.value = null;
  proxyConfigDraft.value = buildProxyConfigDraft();
  proxyConfigNotice.value = "";
  proxyConfigError.value = "";
}

async function handleProxyConfigSave() {
  const invoke = getTauriInvoke();
  const draft = proxyConfigDraft.value;
  if (!invoke || !draft || proxyConfigSaving.value) {
    return;
  }

  const providerId = normalizeProviderIdentifier(draft.providerId);
  const baseUrl = draft.baseUrl.trim();

  if (!providerId) {
    proxyConfigError.value = "请先填写 providerId。";
    return;
  }
  if (!baseUrl) {
    proxyConfigError.value = "请先填写基础 URL。";
    return;
  }

  proxyConfigSaving.value = true;
  proxyConfigError.value = "";
  proxyConfigNotice.value = "";

  try {
    await invoke("save_openclaw_provider_config", {
      config: {
        providerId,
        baseUrl,
        apiKey: draft.apiKey.trim()
      }
    });
    await Promise.all([loadProxyConfigSnapshot(providerId), loadAgents(), refreshDashboardData()]);
    proxyConfigNotice.value = `已保存代理配置：${providerId}`;
  } catch (error) {
    proxyConfigError.value = error instanceof Error ? error.message : "保存代理配置失败。";
  } finally {
    proxyConfigSaving.value = false;
  }
}

async function openSidebarProxyConfig() {
  isAgentSettingsOpen.value = false;
  closeSidebarSettingsModal();
  closeUtilityModal();
  closeRelatedResourceModal();
  isProxyConfigModalOpen.value = true;
  await loadProxyConfigSnapshot();
}

async function openSidebarOpenClawWeb() {
  isAgentSettingsOpen.value = false;
  closeSidebarSettingsModal();
  closeUtilityModal();
  closeProxyConfigModal();

  const invoke = getTauriInvoke();
  if (invoke) {
    try {
      await invoke("open_openclaw_control_ui");
      return;
    } catch {
      try {
        const openedUrl = (await invoke("build_openclaw_control_ui_url")) as string;
        if (openedUrl.trim()) {
          await openExternalUrl(openedUrl);
          return;
        }
      } catch {
        // Fallback to default local dashboard URL.
      }
    }
  }

  await openExternalUrl("http://127.0.0.1:18789/");
}

async function openSidebarLegacyConsole() {
  isAgentSettingsOpen.value = false;
  closeSidebarSettingsModal();
  closeUtilityModal();
  closeProxyConfigModal();
  closeRelatedResourceModal();

  const invoke = getTauriInvoke();
  if (invoke) {
    try {
      await invoke("open_console_window", { section: "overview" });
      return;
    } catch {
      // Fallback to opening console mode URL.
    }
  }

  if (typeof window !== "undefined") {
    try {
      const consoleUrl = new URL(window.location.href);
      consoleUrl.searchParams.set("window", "console");
      consoleUrl.searchParams.set("section", "overview");
      await openExternalUrl(consoleUrl.toString());
      return;
    } catch {
      // Fallback to a relative URL when URL parsing fails.
    }
  }

  await openExternalUrl("index.html?window=console&section=overview");
}

function equalsIgnoreCase(left: string | null | undefined, right: string | null | undefined) {
  return (left ?? "").trim().toLowerCase() === (right ?? "").trim().toLowerCase();
}

function normalizeProviderIdentifier(value: string | null | undefined) {
  return (value ?? "")
    .trim()
    .toLowerCase()
    .replace(/^\/+/, "")
    .replace(/\/+$/, "");
}

function parseAgentModelReference(value: string | null | undefined) {
  const rawModel = (value ?? "").trim().replace(/^\/+/, "");
  if (!rawModel) {
    return { rawModel: "", providerId: "", modelId: "" };
  }
  const separatorIndex = rawModel.indexOf("/");
  if (separatorIndex <= 0 || separatorIndex >= rawModel.length - 1) {
    return { rawModel, providerId: "", modelId: rawModel };
  }
  return {
    rawModel,
    providerId: rawModel.slice(0, separatorIndex),
    modelId: rawModel.slice(separatorIndex + 1)
  };
}

function normalizeProviderProtocol(protocol: string | null | undefined): OpenClawProviderProtocol {
  return equalsIgnoreCase(protocol, "anthropic") ? "anthropic" : "openai";
}

function inferProviderApiKind(protocol: OpenClawProviderProtocol, apiPath: string): OpenClawProviderApiKind {
  if (protocol === "anthropic") {
    return "anthropic-messages";
  }
  return apiPath.trim().toLowerCase().includes("responses") ? "openai-responses" : "openai-completions";
}

function normalizeProviderApiKind(
  protocol: OpenClawProviderProtocol,
  apiKind: OpenClawProviderApiKind | string | null | undefined
): OpenClawProviderApiKind {
  if (protocol === "anthropic") {
    return "anthropic-messages";
  }
  return apiKind === "openai-responses" ? "openai-responses" : "openai-completions";
}

function resolveProviderProtocolByApiKind(apiKind: OpenClawProviderApiKind | string | null | undefined): OpenClawProviderProtocol {
  return apiKind === "anthropic-messages" ? "anthropic" : "openai";
}

function getProviderApiKindLabel(apiKind: OpenClawProviderApiKind | string | null | undefined) {
  const protocol = resolveProviderProtocolByApiKind(apiKind);
  const normalizedApiKind = normalizeProviderApiKind(protocol, apiKind);
  if (normalizedApiKind === "anthropic-messages") {
    return "Anthropic Messages";
  }
  if (normalizedApiKind === "openai-responses") {
    return "OpenAI Responses";
  }
  return "OpenAI Completions";
}

function resolveRelatedModelPlatform(agent: AgentListItem | null, platforms: OpenClawPlatformSnapshotItem[]) {
  if (platforms.length === 0) {
    return null;
  }

  const parsedModel = parseAgentModelReference(agent?.model);
  const normalizedProviderId = normalizeProviderIdentifier(parsedModel.providerId);
  if (normalizedProviderId) {
    const providerMatched =
      platforms.find((platform) => normalizeProviderIdentifier(platform.providerId) === normalizedProviderId) ??
      platforms.find((platform) => normalizeProviderIdentifier(platform.pathPrefix) === normalizedProviderId);
    if (providerMatched) {
      return providerMatched;
    }
  }

  const normalizedModelId = parsedModel.modelId.trim().toLowerCase();
  if (normalizedModelId) {
    const modelMatched = platforms.find((platform) => (platform.model ?? "").trim().toLowerCase() === normalizedModelId);
    if (modelMatched) {
      return modelMatched;
    }
  }

  const normalizedRawModel = parsedModel.rawModel.trim().toLowerCase();
  if (normalizedRawModel) {
    const rawMatched = platforms.find((platform) => (platform.model ?? "").trim().toLowerCase() === normalizedRawModel);
    if (rawMatched) {
      return rawMatched;
    }
  }

  return platforms[0];
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

function formatCompactTime(timestampMs: number | null | undefined) {
  if (!timestampMs || !Number.isFinite(timestampMs)) {
    return "--:--:--";
  }
  return new Date(timestampMs).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false
  });
}

function formatRunDurationLabel(durationMs: number | null | undefined) {
  if (!durationMs || !Number.isFinite(durationMs) || durationMs <= 0) {
    return "—";
  }
  const totalMinutes = Math.floor(durationMs / 60000);
  if (totalMinutes < 60) {
    return `${Math.max(1, totalMinutes)} 分`;
  }
  const totalHours = Math.floor(totalMinutes / 60);
  if (totalHours < 24) {
    return `${totalHours} 小时`;
  }
  const totalDays = Math.floor(totalHours / 24);
  return `${totalDays} 天`;
}

function formatInteger(value: number) {
  return Math.max(0, Math.round(value)).toLocaleString("zh-CN");
}

function getEffectiveLogTotalTokens(log: OpenClawMessageLogItem) {
  if (typeof log.totalTokens === "number" && Number.isFinite(log.totalTokens)) {
    return Math.max(0, Math.round(log.totalTokens));
  }
  const prompt = typeof log.promptTokens === "number" && Number.isFinite(log.promptTokens) ? Math.max(0, Math.round(log.promptTokens)) : 0;
  const completion =
    typeof log.completionTokens === "number" && Number.isFinite(log.completionTokens) ? Math.max(0, Math.round(log.completionTokens)) : 0;
  const cache =
    typeof log.cacheReadInputTokens === "number" && Number.isFinite(log.cacheReadInputTokens)
      ? Math.max(0, Math.round(log.cacheReadInputTokens))
      : 0;
  return prompt + completion + cache;
}

function updateDashboardHeapUsage() {
  if (typeof window === "undefined") {
    dashboardJsHeapUsageMb.value = null;
    return;
  }
  const performanceWithMemory = window.performance as Performance & {
    memory?: { usedJSHeapSize?: number };
  };
  const bytes = performanceWithMemory.memory?.usedJSHeapSize;
  if (typeof bytes !== "number" || !Number.isFinite(bytes) || bytes <= 0) {
    dashboardJsHeapUsageMb.value = null;
    return;
  }
  dashboardJsHeapUsageMb.value = Math.max(1, Math.round(bytes / (1024 * 1024)));
}

function mapGatewayStatusTone(status: string): DashboardHealthTone {
  const normalized = status.trim().toLowerCase();
  if (normalized === "online") {
    return "online";
  }
  if (normalized === "checking") {
    return "neutral";
  }
  if (normalized === "unconfigured" || normalized === "unsupported") {
    return "warn";
  }
  return "offline";
}

function mapGatewayStatusLabel(status: string) {
  const normalized = status.trim().toLowerCase();
  if (normalized === "online") {
    return "在线";
  }
  if (normalized === "checking") {
    return "检测中";
  }
  if (normalized === "unconfigured") {
    return "未配置";
  }
  if (normalized === "unsupported") {
    return "不可用";
  }
  return "离线";
}

function clearRelatedResourceSnapshots() {
  relatedMemorySnapshot.value = null;
  relatedSkillsSnapshot.value = null;
  relatedToolsSnapshot.value = null;
  relatedModelSnapshot.value = null;
  relatedModelDraft.value = null;
  relatedChannelSnapshot.value = null;
  relatedTaskSnapshot.value = null;
}

function getStableChatMessages(messages: AgentChatMessage[]) {
  return messages.filter((item) => item.status !== "pending" && !isLegacyWelcomeMessage(item));
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

function getLogRequestUrl(log: OpenClawMessageLogItem) {
  if (log.baseUrl || log.path) {
    return `${log.baseUrl ?? ""}${log.path ?? ""}` || log.endpoint;
  }
  return log.endpoint;
}

function getLogAgentId(log: OpenClawMessageLogItem): string | null {
  const platformId = (log.platformId ?? "").trim();
  if (platformId.startsWith("openclaw-agent-")) {
    return platformId.slice("openclaw-agent-".length);
  }
  if (platformId.startsWith("openclaw-runtime-")) {
    const tail = platformId.slice("openclaw-runtime-".length);
    const separator = tail.indexOf("-");
    return separator > 0 ? tail.slice(0, separator) : tail;
  }
  const sessionId = (log.sessionId ?? "").trim();
  if (sessionId.startsWith("runtime-")) {
    const tail = sessionId.slice("runtime-".length);
    const separator = tail.indexOf("-");
    return separator > 0 ? tail.slice(0, separator) : tail;
  }
  return null;
}

function matchesAgentLog(log: OpenClawMessageLogItem, agentId: string | null) {
  if (!agentId) {
    return false;
  }
  return equalsIgnoreCase(getLogAgentId(log), agentId);
}

function isToolRuntimeLog(log: OpenClawMessageLogItem) {
  const method = (log.method ?? "").trim().toUpperCase();
  if (/^TOOL(?::|\/|_|$)/.test(method)) {
    return true;
  }
  const target = `${log.path ?? ""} ${log.endpoint ?? ""}`.toLowerCase();
  return target.includes("/tools/") || target.includes("tool_call") || target.includes("tool-call");
}

function getRuntimeLogCategory(log: OpenClawMessageLogItem): Exclude<UtilityLogCategory, "all"> {
  return isToolRuntimeLog(log) ? "tool" : "message";
}

function matchesRuntimeLogCategory(log: OpenClawMessageLogItem, category: UtilityLogCategory) {
  if (category === "all") {
    return true;
  }
  return getRuntimeLogCategory(log) === category;
}

function getRuntimeLogCategoryLabel(log: OpenClawMessageLogItem) {
  return getRuntimeLogCategory(log) === "tool" ? "工具" : "消息";
}

function formatJsonText(value?: string) {
  const source = (value ?? "").trim();
  if (!source) {
    return { text: "暂无内容", language: "text" as const };
  }
  try {
    return {
      text: JSON.stringify(JSON.parse(source), null, 2),
      language: "json" as const
    };
  } catch {
    return { text: source, language: "text" as const };
  }
}

function getDefaultLogDetailTab(log: OpenClawMessageLogItem): UtilityLogDetailTab {
  if ((log.responseBody ?? "").trim()) {
    return "response";
  }
  if ((log.streamSummary ?? "").trim()) {
    return "stream";
  }
  if ((log.requestBody ?? "").trim()) {
    return "request";
  }
  return "raw";
}

function buildLogDetailSections(log: OpenClawMessageLogItem) {
  const sections: Array<{ id: UtilityLogDetailTab; label: string; text: string; language: "json" | "text" }> = [];

  if ((log.requestBody ?? "").trim()) {
    const view = formatJsonText(log.requestBody);
    sections.push({ id: "request", label: "请求体", text: view.text, language: view.language });
  }

  if ((log.responseBody ?? "").trim()) {
    const view = formatJsonText(log.responseBody);
    sections.push({ id: "response", label: "响应体", text: view.text, language: view.language });
  }

  if ((log.streamSummary ?? "").trim()) {
    const view = formatJsonText(log.streamSummary);
    sections.push({ id: "stream", label: "流式汇总", text: view.text, language: view.language });
  }

  const rawText = [log.error, log.responseBody].filter((item) => Boolean(item && item.trim())).join("\n\n").trim();
  if (rawText) {
    const view = formatJsonText(rawText);
    sections.push({ id: "raw", label: "原始响应", text: view.text, language: view.language });
  }

  if (sections.length === 0) {
    sections.push({ id: "raw", label: "原始响应", text: "暂无可预览内容", language: "text" });
  }

  return sections;
}

function getErrorSignature(log: OpenClawMessageLogItem) {
  const errorText = (log.error ?? "").trim();
  if (errorText) {
    return errorText.split("\n")[0].slice(0, 120);
  }
  if (log.responseStatus >= 500) {
    return `HTTP ${log.responseStatus} · ${log.method}`;
  }
  if (log.responseStatus >= 400) {
    return `HTTP ${log.responseStatus} · ${log.method}`;
  }
  return "运行异常";
}

function pickLogForDetail(logId: string | null, logs: OpenClawMessageLogItem[]) {
  if (!logId) {
    return logs[0] ?? null;
  }
  return logs.find((log) => log.id === logId) ?? logs[0] ?? null;
}

function selectUtilityLog(log: OpenClawMessageLogItem | null, preferredTab?: UtilityLogDetailTab) {
  utilitySelectedLogId.value = log?.id ?? null;
  if (!log) {
    utilityLogDetailTab.value = "response";
    return;
  }
  utilityLogDetailTab.value = preferredTab ?? getDefaultLogDetailTab(log);
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
    // Keep fallback snapshot compatible with legacy task status chips.
    // `done` and `cancelled` are treated as disabled terminal states.
    id: item.id,
    name: item.title,
    agentId: item.owner,
    sessionTarget: item.project,
    enabled: item.status !== "done" && item.status !== "cancelled",
    deleteAfterRun: item.status === "done" || item.status === "cancelled",
    statusKind: item.status === "in_progress" ? "late" : item.status === "done" || item.status === "cancelled" ? "disabled" : "scheduled",
    statusLabel: item.status === "in_progress" ? "待执行" : item.status === "done" || item.status === "cancelled" ? "已停用" : "已启用",
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

async function loadRelatedModelSnapshot() {
  const invoke = getTauriInvoke();
  if (!invoke) {
    relatedModelSnapshot.value = {
      sourcePath: "runtime unavailable",
      detail: "当前环境不支持读取模型平台配置。",
      platforms: []
    };
    relatedModelDraft.value = null;
    relatedResourceModalError.value = "当前环境不支持模型配置快照。";
    return;
  }

  relatedModelSnapshot.value = (await invoke("load_openclaw_platforms_snapshot")) as OpenClawPlatformSnapshotResponse;
  const target = resolveRelatedModelPlatform(activeAgent.value, relatedModelSnapshot.value?.platforms ?? []);
  if (!target) {
    relatedModelDraft.value = null;
    return;
  }

  const modelRef = parseAgentModelReference(activeAgent.value?.model);
  const fallbackModel = modelRef.modelId || modelRef.rawModel;
  relatedModelDraft.value = {
    providerId: target.providerId || normalizeProviderIdentifier(target.pathPrefix) || "custom",
    providerName: target.name || target.providerId || "未命名 Provider",
    protocol: normalizeProviderProtocol(target.protocol),
    apiKind: inferProviderApiKind(normalizeProviderProtocol(target.protocol), target.apiPath || ""),
    baseUrl: target.baseUrl || "",
    model: target.model || fallbackModel,
    apiKey: target.apiKey || "",
    apiPath: target.apiPath || ""
  };
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

async function refreshDashboardData() {
  if (isDashboardRefreshing.value) {
    return;
  }

  isDashboardRefreshing.value = true;
  dashboardRefreshError.value = "";
  updateDashboardHeapUsage();

  const invoke = getTauriInvoke();
  if (!invoke) {
    dashboardGatewayHealth.value = {
      status: "unsupported",
      checkedUrl: null,
      detail: "当前环境不支持网关探测。",
      latencyMs: null,
      gatewayPort: null
    };
    dashboardChannelSnapshot.value = {
      sourcePath: "runtime unavailable",
      detail: "当前环境不支持频道统计。",
      channels: []
    };
    dashboardPlatformSnapshot.value = {
      sourcePath: "runtime unavailable",
      detail: "当前环境不支持平台统计。",
      platforms: []
    };
    chatRuntimeLogs.value = {
      detail: "当前环境不支持运行日志。",
      logs: []
    };
    dashboardLastRefreshedAt.value = Date.now();
    isDashboardRefreshing.value = false;
    return;
  }

  const [gatewayResult, channelResult, platformResult, logsResult] = await Promise.allSettled([
    invoke("check_openclaw_gateway"),
    invoke("load_openclaw_channel_accounts_snapshot"),
    invoke("load_openclaw_platforms_snapshot"),
    invoke("load_openclaw_message_logs")
  ]);

  if (gatewayResult.status === "fulfilled") {
    dashboardGatewayHealth.value = gatewayResult.value as GatewayHealthSnapshotResponse;
  } else {
    dashboardGatewayHealth.value = {
      status: "offline",
      checkedUrl: null,
      detail: gatewayResult.reason instanceof Error ? gatewayResult.reason.message : "网关状态读取失败。",
      latencyMs: null,
      gatewayPort: null
    };
  }

  if (channelResult.status === "fulfilled") {
    dashboardChannelSnapshot.value = channelResult.value as OpenClawChannelAccountsSnapshotResponse;
  } else {
    dashboardChannelSnapshot.value = {
      sourcePath: "runtime unavailable",
      detail: channelResult.reason instanceof Error ? channelResult.reason.message : "频道统计读取失败。",
      channels: []
    };
  }

  if (platformResult.status === "fulfilled") {
    dashboardPlatformSnapshot.value = platformResult.value as OpenClawPlatformSnapshotResponse;
  } else {
    dashboardPlatformSnapshot.value = {
      sourcePath: "runtime unavailable",
      detail: platformResult.reason instanceof Error ? platformResult.reason.message : "平台统计读取失败。",
      platforms: []
    };
  }

  if (logsResult.status === "fulfilled") {
    chatRuntimeLogs.value = logsResult.value as OpenClawMessageLogResponse;
  } else {
    chatRuntimeLogs.value = {
      detail: logsResult.reason instanceof Error ? logsResult.reason.message : "运行日志读取失败。",
      logs: []
    };
  }

  if (
    gatewayResult.status === "rejected" ||
    channelResult.status === "rejected" ||
    platformResult.status === "rejected" ||
    logsResult.status === "rejected"
  ) {
    dashboardRefreshError.value = "部分数据读取失败，已展示可用信息。";
  }

  dashboardLastRefreshedAt.value = Date.now();
  isDashboardRefreshing.value = false;
}

async function openExternalUrl(url: string) {
  const trimmed = url.trim();
  if (!trimmed) {
    return;
  }

  const invoke = getTauriInvoke();
  if (invoke) {
    try {
      await invoke("open_external_url", { url: trimmed });
      return;
    } catch {
      // Fallback to browser open.
    }
  }

  if (typeof window !== "undefined") {
    window.open(trimmed, "_blank", "noopener,noreferrer");
  }
}

function getSkillMarketCacheKey(category: SkillMarketSectionCategory, sortBy: SkillMarketSortBy) {
  return `${category}:${sortBy}`;
}

function getSkillMarketGlobalCacheKey(sortBy: SkillMarketSortBy) {
  return `global:${sortBy}`;
}

async function loadSkillMarketSection(category: SkillMarketSectionCategory, force = false) {
  const cacheKey = getSkillMarketCacheKey(category, skillMarketSortBy.value);
  if (!force) {
    const cached = skillMarketCache.get(cacheKey);
    if (cached) {
      if (category === "top") {
        skillMarketTopSkills.value = cached.skills;
        skillMarketTopTotal.value = cached.total;
      } else {
        skillMarketCategorySkills.value = cached.skills;
        skillMarketCategoryTotal.value = cached.total;
      }
      return;
    }
  }

  const result =
    category === "top"
      ? await fetchSkillTop50()
      : await fetchSkillsByCategory(category, {
          page: 1,
          pageSize: 200,
          sortBy: skillMarketSortBy.value,
          order: "desc"
        });

  skillMarketCache.set(cacheKey, {
    skills: result.skills,
    total: result.total
  });

  if (category === "top") {
    skillMarketTopSkills.value = result.skills;
    skillMarketTopTotal.value = result.total;
  } else {
    skillMarketCategorySkills.value = result.skills;
    skillMarketCategoryTotal.value = result.total;
  }
}

async function refreshSkillMarket(force = false) {
  const token = ++skillMarketRequestToken;
  const keyword = skillMarketSearch.value.trim();
  skillMarketLoading.value = true;
  skillMarketError.value = "";

  try {
    if (keyword) {
      const globalCacheKey = getSkillMarketGlobalCacheKey(skillMarketSortBy.value);
      if (!force) {
        const cached = skillMarketGlobalCache.get(globalCacheKey);
        if (cached) {
          skillMarketGlobalSkills.value = cached.skills;
          skillMarketGlobalTotal.value = cached.total;
        } else {
          const result = await fetchSkillsGlobal({
            page: 1,
            pageSize: 500,
            sortBy: skillMarketSortBy.value,
            order: "desc"
          });
          if (token !== skillMarketRequestToken) {
            return;
          }
          skillMarketGlobalSkills.value = result.skills;
          skillMarketGlobalTotal.value = result.total;
          skillMarketGlobalCache.set(globalCacheKey, {
            skills: result.skills,
            total: result.total
          });
        }
      } else {
        const result = await fetchSkillsGlobal({
          page: 1,
          pageSize: 500,
          sortBy: skillMarketSortBy.value,
          order: "desc"
        });
        if (token !== skillMarketRequestToken) {
          return;
        }
        skillMarketGlobalSkills.value = result.skills;
        skillMarketGlobalTotal.value = result.total;
        skillMarketGlobalCache.set(globalCacheKey, {
          skills: result.skills,
          total: result.total
        });
      }
    } else {
      skillMarketGlobalSkills.value = [];
      skillMarketGlobalTotal.value = 0;
    }

    await loadSkillMarketSection(activeSkillMarketCategory.value, force);
    if (token !== skillMarketRequestToken) {
      return;
    }
  } catch (error) {
    if (token !== skillMarketRequestToken) {
      return;
    }
    skillMarketError.value = error instanceof Error ? error.message : "技能市场加载失败。";
  } finally {
    if (token === skillMarketRequestToken) {
      skillMarketLoading.value = false;
    }
  }
}

function handleSkillMarketCategorySwitch(category: SkillMarketSectionCategory) {
  if (activeSkillMarketCategory.value === category) {
    return;
  }
  activeSkillMarketCategory.value = category;
  skillMarketPage.value = 1;
  void refreshSkillMarket();
}

function handleSkillMarketSortChange(sortBy: SkillMarketSortBy) {
  if (skillMarketSortBy.value === sortBy) {
    return;
  }
  skillMarketSortBy.value = sortBy;
  skillMarketPage.value = 1;
  skillMarketCache.clear();
  skillMarketGlobalCache.clear();
  void refreshSkillMarket(true);
}

function handleSkillMarketSearchSubmit() {
  skillMarketPage.value = 1;
  void refreshSkillMarket(true);
}

function goToSkillMarketPage(page: number) {
  const total = skillMarketCurrentTotalPages.value;
  const nextPage = Math.min(Math.max(1, page), total);
  if (nextPage === skillMarketPage.value) {
    return;
  }
  skillMarketPage.value = nextPage;
}

function goPrevSkillMarketPage() {
  goToSkillMarketPage(skillMarketPage.value - 1);
}

function goNextSkillMarketPage() {
  goToSkillMarketPage(skillMarketPage.value + 1);
}

function getSkillMarketInitial(name: string) {
  const trimmed = name.trim();
  if (!trimmed) {
    return "技";
  }
  return trimmed.charAt(0).toUpperCase();
}

function getSkillMarketDescription(skill: SkillMarketSkill) {
  return skill.descriptionZh.trim() || skill.description.trim() || "暂无技能描述。";
}

function formatSkillMarketCount(value: number | null | undefined) {
  if (!value || !Number.isFinite(value) || value <= 0) {
    return "0";
  }
  if (value >= 10000) {
    return `${(value / 10000).toFixed(1)}w`;
  }
  return String(Math.round(value));
}

function formatSkillMarketVersion(value: string | null | undefined) {
  const trimmed = (value ?? "").trim();
  return trimmed || "v1.0.0";
}

async function openSkillHomepage(skill: SkillMarketSkill) {
  const url = skill.homepage?.trim();
  if (!url) {
    return;
  }
  await openExternalUrl(url);
}

function openSkillMarketDetailModal(skill: SkillMarketSkill) {
  activeSkillMarketDetail.value = skill;
}

function closeSkillMarketDetailModal() {
  activeSkillMarketDetail.value = null;
}

function getSkillMarketCategoryLabel(category: string | null | undefined) {
  const normalized = (category ?? "").trim().toLowerCase();
  if (!normalized) {
    return "未分类";
  }
  const matched = skillMarketCategories.find((item) => item.id === normalized || item.apiCategory === normalized);
  return matched?.label ?? normalized;
}

function canInstallSkillMarketSkill(skill: SkillMarketSkill) {
  return Boolean(getTauriInvoke()) || Boolean(skill.homepage?.trim());
}

async function installSkillMarketSkill(skill: SkillMarketSkill) {
  const invoke = getTauriInvoke();
  if (invoke) {
    try {
      await invoke("open_console_window", { section: "skill-market" });
      skillMarketActionNotice.value = "已打开旧版技能市场，可继续安装流程。";
      return;
    } catch {
      // Fallback to market homepage.
    }
  }

  const url = skill.homepage?.trim();
  if (!url) {
    skillMarketActionNotice.value = "当前技能缺少可安装入口。";
    return;
  }
  await openExternalUrl(url);
  skillMarketActionNotice.value = "已打开技能详情页，请按页面提示完成安装。";
}

function buildInstallableRoleAgentId(sourcePath: string) {
  const base = sourcePath
    .replace(/\.md$/i, "")
    .replace(/[\\/]+/g, "-")
    .toLowerCase()
    .replace(/[^a-z0-9-]+/g, "-")
    .replace(/-{2,}/g, "-")
    .replace(/^-+|-+$/g, "");
  return base || `role-${Date.now()}`;
}

function resolveInstalledRoleWorkspaceDir(installResult: string, normalizedAgentId: string) {
  const matched = installResult.match(/工作区[:：]\s*([^)）]+)/);
  const workspaceWithFile = matched?.[1]?.trim() ?? "";
  const normalizedPath = workspaceWithFile.replace(/[\\/]AGENTS\.md$/i, "");
  if (normalizedPath) {
    return normalizedPath;
  }
  return `~/.openclaw/workspace-${normalizedAgentId}`;
}

function buildRoleWorkflowInstallPrompt(roleName: string, sourcePath: string, detailMarkdown: string) {
  const normalizedRoleName = roleName.trim() || "未命名角色";
  const normalizedSourcePath = sourcePath.trim() || "unknown";
  const normalizedDetail = detailMarkdown.trim();

  return [
    ROLE_WORKFLOW_INSTALL_PROMPT_PREFIX,
    "若已安装 openclaw-agent-factory 等角色创建 skill，请优先调用对应 skill 完成创建。",
    `角色名称：${normalizedRoleName}`,
    `来源：${normalizedSourcePath}`,
    "详情内容：",
    normalizedDetail
  ].join("\n");
}

async function installRoleWorkflowRole() {
  const found = activeRoleWorkflowBase.value;
  if (!found || roleWorkflowDetailLoading.value || isRoleWorkflowInstalling.value) {
    return null;
  }
  const invoke = getTauriInvoke();
  if (!invoke) {
    roleWorkflowDetailNotice.value = {
      tone: "error",
      text: "当前环境不支持安装角色（仅桌面端可用）。"
    };
    return null;
  }

  const agentId = buildInstallableRoleAgentId(found.role.sourcePath);
  const markdown = roleWorkflowDetailDraft.value.contentZh.trim();
  if (!markdown) {
    roleWorkflowDetailNotice.value = {
      tone: "error",
      text: "详情内容为空，无法安装角色。"
    };
    return null;
  }

  const selectedNameZh = roleWorkflowNameZhDraft.value.trim() || found.role.nameZh;
  roleWorkflowDetailNotice.value = null;
  isRoleWorkflowInstalling.value = true;
  try {
    const result = (await invoke("install_role_workflow_agent", {
      agentId,
      displayName: selectedNameZh || found.role.nameEn || found.role.nameZh,
      content: markdown,
      sourcePath: found.role.sourcePath
    })) as string;

    const roleName = selectedNameZh || found.role.nameZh || found.role.nameEn || agentId;
    const workspaceDir = resolveInstalledRoleWorkspaceDir(result ?? "", agentId);
    const openclawPrompt = buildRoleWorkflowInstallPrompt(roleName, found.role.sourcePath, markdown);

    let openclawDispatchSummary = "已将角色详情发送到 OpenClaw。";
    try {
      await sendOpenClawChat(
        [{ role: "user", content: openclawPrompt }],
        selectedAgentId.value ? { agentId: selectedAgentId.value } : {}
      );
    } catch (dispatchError) {
      openclawDispatchSummary = `角色已安装，但发送角色详情到 OpenClaw 失败：${
        dispatchError instanceof Error ? dispatchError.message : "未知错误"
      }`;
    }

    roleWorkflowDetailNotice.value = {
      tone: "success",
      text: `角色已安装：${roleName}。配置文件目录：${workspaceDir}。${openclawDispatchSummary}`
    };
    await loadAgents();
    const installedAgent =
      agents.value.find((agent) => equalsIgnoreCase(agent.agentId, agentId)) ??
      agents.value.find((agent) => stripRoleLabel(agent.displayName) === roleName);
    if (installedAgent) {
      switchAgent(installedAgent.agentId);
      return installedAgent.agentId;
    }
    return null;
  } catch (error) {
    roleWorkflowDetailNotice.value = {
      tone: "error",
      text: error instanceof Error ? error.message : `安装 ${found.role.nameZh} 失败。`
    };
    return null;
  } finally {
    isRoleWorkflowInstalling.value = false;
  }
}

async function openRoleWorkflowEditor(role: AgencyRosterRole) {
  if (isRoleWorkflowInstalling.value) {
    return;
  }
  const roleId = role.id;
  const override = roleWorkflowOverrides.value[roleId];
  const requestToken = ++roleWorkflowDetailRequestToken;
  roleWorkflowDetailRoleId.value = roleId;
  roleWorkflowDetailNotice.value = null;
  roleWorkflowDetailLoading.value = true;
  roleWorkflowNameZhOriginal.value = override?.nameZh ?? role.nameZh;
  roleWorkflowNameZhDraft.value = roleWorkflowNameZhOriginal.value;
  roleWorkflowDetailOriginalContent.value = "正在加载详情内容...";
  roleWorkflowDetailDraft.value = {
    contentZh: "正在加载详情内容..."
  };

  try {
    const detailSnapshot = await loadAgentDetailMarkdownZh(role.sourcePath);
    if (requestToken !== roleWorkflowDetailRequestToken || roleWorkflowDetailRoleId.value !== roleId) {
      return;
    }
    const baseContent = detailSnapshot.contentZh;
    roleWorkflowDetailOriginalContent.value = baseContent;
    roleWorkflowDetailDraft.value = {
      contentZh: override?.detailContentZh ?? baseContent
    };
    if (!detailSnapshot.found) {
      roleWorkflowDetailNotice.value = {
        tone: "error",
        text: `未找到 ${role.sourcePath}，已加载占位详情。`
      };
    }
  } catch (error) {
    if (requestToken !== roleWorkflowDetailRequestToken || roleWorkflowDetailRoleId.value !== roleId) {
      return;
    }
    roleWorkflowDetailNotice.value = {
      tone: "error",
      text: error instanceof Error ? error.message : "角色详情加载失败。"
    };
    const fallbackText = `# 角色详情加载失败\n\n${roleWorkflowDetailNotice.value.text}`;
    roleWorkflowDetailOriginalContent.value = fallbackText;
    roleWorkflowDetailDraft.value = {
      contentZh: fallbackText
    };
  } finally {
    if (requestToken === roleWorkflowDetailRequestToken && roleWorkflowDetailRoleId.value === roleId) {
      roleWorkflowDetailLoading.value = false;
    }
  }
}

function closeRoleWorkflowDetail() {
  if (isRoleWorkflowInstalling.value) {
    return;
  }
  roleWorkflowDetailRequestToken += 1;
  roleWorkflowDetailRoleId.value = null;
  roleWorkflowDetailDraft.value = { contentZh: "" };
  roleWorkflowDetailOriginalContent.value = "";
  roleWorkflowNameZhDraft.value = "";
  roleWorkflowNameZhOriginal.value = "";
  roleWorkflowDetailLoading.value = false;
  roleWorkflowDetailNotice.value = null;
}

function createRoleWorkflowVersionId() {
  return `role-ver-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function hasRoleWorkflowOverrideContent(override: RoleWorkflowOverride) {
  if ((override.nameZh ?? "").trim()) {
    return true;
  }
  if ((override.workflowZh ?? "").trim()) {
    return true;
  }
  if ((override.detailContentZh ?? "").trim()) {
    return true;
  }
  return (override.detailVersions?.length ?? 0) > 0;
}

function saveRoleWorkflowDetail() {
  const activeId = roleWorkflowDetailRoleId.value;
  const found = activeRoleWorkflowBase.value;
  if (!activeId || !found || roleWorkflowDetailLoading.value) {
    return;
  }

  const nextContent = roleWorkflowDetailDraft.value.contentZh;
  const nextNameZh = roleWorkflowNameZhDraft.value.trim() || found.role.nameZh;
  const current = roleWorkflowOverrides.value[activeId] ?? {};
  const nextVersions = [
    {
      id: createRoleWorkflowVersionId(),
      contentZh: nextContent,
      savedAt: Date.now()
    },
    ...(current.detailVersions ?? [])
  ].slice(0, 3);

  roleWorkflowOverrides.value = {
    ...roleWorkflowOverrides.value,
    [activeId]: {
      ...current,
      nameZh: nextNameZh,
      detailContentZh: nextContent,
      detailVersions: nextVersions
    }
  };
  persistRoleWorkflowOverrides();
  roleWorkflowDetailOriginalContent.value = nextContent;
  roleWorkflowNameZhOriginal.value = nextNameZh;
  roleWorkflowNameZhDraft.value = nextNameZh;
  roleWorkflowDetailNotice.value = {
    tone: "success",
    text: `${nextNameZh} 已保存。`
  };
}

function restoreRoleWorkflowOriginalContent() {
  const found = activeRoleWorkflowBase.value;
  if (!found || roleWorkflowDetailLoading.value) {
    return;
  }
  roleWorkflowDetailDraft.value = {
    contentZh: roleWorkflowDetailOriginalContent.value
  };
  roleWorkflowNameZhDraft.value = roleWorkflowNameZhOriginal.value;
  roleWorkflowDetailNotice.value = {
    tone: "success",
    text: `${roleWorkflowNameZhOriginal.value || found.role.nameZh} 已恢复原始内容。`
  };
}

function applyRoleWorkflowSavedVersion(versionId: string) {
  const found = activeRoleWorkflowBase.value;
  const matched = roleWorkflowDetailSavedVersions.value.find((version) => version.id === versionId);
  if (!found || !matched || roleWorkflowDetailLoading.value) {
    return;
  }
  roleWorkflowDetailDraft.value = {
    contentZh: matched.contentZh
  };
  roleWorkflowDetailNotice.value = {
    tone: "success",
    text: `已载入 ${found.role.nameZh} 的历史版本（${formatDateTime(matched.savedAt)}）。`
  };
}

function deleteRoleWorkflowSavedVersion(versionId: string) {
  const activeId = roleWorkflowDetailRoleId.value;
  if (!activeId || roleWorkflowDetailLoading.value) {
    return;
  }
  const current = roleWorkflowOverrides.value[activeId];
  if (!current) {
    return;
  }
  const nextVersions = (current.detailVersions ?? []).filter((version) => version.id !== versionId);
  const nextOverride: RoleWorkflowOverride = {
    ...current
  };
  if (nextVersions.length > 0) {
    nextOverride.detailVersions = nextVersions;
  } else {
    delete nextOverride.detailVersions;
  }

  if (hasRoleWorkflowOverrideContent(nextOverride)) {
    roleWorkflowOverrides.value = {
      ...roleWorkflowOverrides.value,
      [activeId]: nextOverride
    };
  } else {
    const { [activeId]: _, ...rest } = roleWorkflowOverrides.value;
    roleWorkflowOverrides.value = rest;
  }
  persistRoleWorkflowOverrides();
}

function handleRecruitRole(role: AgencyRosterRole) {
  void (async () => {
    await openRoleWorkflowEditor(role);
    if (roleWorkflowDetailRoleId.value !== role.id) {
      return;
    }
    const installedAgentId = await installRoleWorkflowRole();
    if (!installedAgentId) {
      return;
    }
    activeSection.value = "chat";
    chatInput.value = `请以「${roleWorkflowNameZhDraft.value.trim() || role.nameZh}」的身份协助我，目标是：`;
  })();
}

function handleSidebarSectionChange(section: SidebarSection) {
  activeSection.value = section;
  if (section !== "chat") {
    isAgentSettingsOpen.value = false;
    closeSidebarSettingsModal();
  }

  if (section === "tasks") {
    taskItems.value = loadTasks();
    taskModuleView.value = "projects";
    taskModuleError.value = "";
    taskModuleNotice.value = "";
    activeTaskProject.value = DEFAULT_TASK_PROJECT_NAME;
    syncTaskProjectNamesFromTasks();
  }
}

function openTaskProjectsHome() {
  taskModuleView.value = "projects";
  taskModuleError.value = "";
  taskDragTaskId.value = null;
  taskDragOverStatus.value = null;
}

function openTaskProjectBoard(projectName: string) {
  if (!taskProjectNames.value.includes(projectName)) {
    return;
  }
  activeTaskProject.value = projectName;
  taskModuleView.value = "board";
  taskModuleError.value = "";
  taskModuleNotice.value = "";
  resetTaskDraftForm();
}

function handleCreateTaskProject() {
  const projectName = normalizeTaskProjectName(taskProjectInput.value);
  if (!projectName) {
    taskModuleError.value = "请输入项目名称。";
    taskModuleNotice.value = "";
    return;
  }
  if (projectName === DEFAULT_TASK_PROJECT_NAME) {
    taskModuleError.value = `「${DEFAULT_TASK_PROJECT_NAME}」是默认项目，不能重复创建。`;
    taskModuleNotice.value = "";
    return;
  }
  if (taskProjectNames.value.some((item) => item.toLowerCase() === projectName.toLowerCase())) {
    taskModuleError.value = `项目「${projectName}」已存在。`;
    taskModuleNotice.value = "";
    return;
  }

  taskProjectNames.value = buildTaskProjectList([...taskProjectNames.value, projectName]);
  saveTaskProjectsToStorage(taskProjectNames.value);
  taskProjectInput.value = "";
  taskModuleNotice.value = `项目「${projectName}」已创建。`;
  taskModuleError.value = "";
}

function handleCreateTaskInActiveProject() {
  const title = taskDraftTitle.value.trim();
  if (!title) {
    taskModuleError.value = "请先填写任务标题。";
    taskModuleNotice.value = "";
    return;
  }
  const draft = createTaskDraft();
  const projectName = activeTaskProject.value === DEFAULT_TASK_PROJECT_NAME ? "" : activeTaskProject.value;
  updateTaskRecord({
    ...draft,
    title,
    summary: taskDraftSummary.value.trim() || "暂无说明",
    owner: taskDraftOwner.value.trim() || "Commander",
    priority: taskDraftPriority.value,
    project: projectName,
    status: "todo"
  });
  taskModuleNotice.value = `任务「${title}」已添加到「${activeTaskProject.value}」。`;
  taskModuleError.value = "";
  resetTaskDraftForm();
}

function getTaskStatusText(status: TaskBoardStatus) {
  if (status === "todo") return "待办事项";
  if (status === "in_progress") return "进行中";
  if (status === "in_review") return "回顾";
  if (status === "done") return "完成";
  return "取消";
}

function getTaskPriorityText(priority: TaskRecord["priority"]) {
  if (priority === "p0") return "P0 紧急";
  if (priority === "p1") return "P1 常规";
  return "P2 低优先";
}

function getTaskPrevStatus(status: TaskBoardStatus) {
  const index = taskStatusFlow.indexOf(status);
  if (index <= 0) {
    return null;
  }
  return taskStatusFlow[index - 1] ?? null;
}

function getTaskNextStatus(status: TaskBoardStatus) {
  const index = taskStatusFlow.indexOf(status);
  if (index < 0 || index >= taskStatusFlow.length - 1) {
    return null;
  }
  return taskStatusFlow[index + 1] ?? null;
}

function updateTaskStatus(taskId: string, nextStatus: TaskBoardStatus) {
  const current = taskItems.value.find((task) => task.id === taskId);
  if (!current || current.status === nextStatus) {
    return;
  }
  updateTaskRecord({
    ...current,
    status: nextStatus
  });
  taskModuleError.value = "";
  taskModuleNotice.value = `任务「${current.title}」已移动到「${getTaskStatusText(nextStatus)}」。`;
}

function moveTaskToPrevStatus(task: TaskRecord) {
  const prev = getTaskPrevStatus(task.status);
  if (!prev) {
    return;
  }
  updateTaskStatus(task.id, prev);
}

function moveTaskToNextStatus(task: TaskRecord) {
  const next = getTaskNextStatus(task.status);
  if (!next) {
    return;
  }
  updateTaskStatus(task.id, next);
}

function handleTaskDragStart(taskId: string) {
  taskDragTaskId.value = taskId;
  taskDragOverStatus.value = null;
}

function handleTaskDragEnd() {
  taskDragTaskId.value = null;
  taskDragOverStatus.value = null;
}

function handleTaskColumnDragOver(status: TaskBoardStatus) {
  if (!taskDragTaskId.value) {
    return;
  }
  taskDragOverStatus.value = status;
}

function handleTaskColumnDrop(status: TaskBoardStatus) {
  const taskId = taskDragTaskId.value;
  if (!taskId) {
    return;
  }
  updateTaskStatus(taskId, status);
  handleTaskDragEnd();
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
    if (target === "model") {
      await loadRelatedModelSnapshot();
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
  closeProxyConfigModal();
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
  relatedModelDraft.value = null;
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
  utilityLogTab.value = "runtime";
  utilityLogDetailTab.value = "response";
  utilityRuntimeCategory.value = "all";
  utilitySelectedLogId.value = null;
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
      utilitySelectedLogId.value = null;
      return;
    }
    chatRuntimeLogs.value = (await invoke("load_openclaw_message_logs")) as OpenClawMessageLogResponse;
    const filteredLogs = (chatRuntimeLogs.value?.logs ?? [])
      .filter((log) => matchesAgentLog(log, agent?.agentId ?? null))
      .sort((left, right) => right.createdAt - left.createdAt);
    const categorizedLogs = filteredLogs.filter((log) => matchesRuntimeLogCategory(log, utilityRuntimeCategory.value));
    const selected = pickLogForDetail(utilitySelectedLogId.value, categorizedLogs);
    if (utilityLogTab.value === "errorAnalysis") {
      const errorLog =
        filteredLogs.find((log) => log.responseStatus >= 400 || Boolean((log.error ?? "").trim())) ??
        pickLogForDetail(utilitySelectedLogId.value, filteredLogs);
      selectUtilityLog(errorLog, "raw");
    } else {
      selectUtilityLog(selected);
    }
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
  if (type === "logs") {
    utilityLogTab.value = "runtime";
    utilityLogDetailTab.value = "response";
    utilityRuntimeCategory.value = "all";
    utilitySelectedLogId.value = null;
  }
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

function handleUtilityLogTabChange(tab: UtilityLogTab) {
  utilityLogTab.value = tab;
  if (tab === "runtime") {
    const target = pickLogForDetail(utilitySelectedLogId.value, runtimeCategoryLogItems.value);
    selectUtilityLog(target);
    return;
  }
  const firstErrorLog = errorAnalysisSummaries.value[0]?.latestLog ?? null;
  selectUtilityLog(firstErrorLog, "raw");
}

function handleUtilityRuntimeCategoryChange(category: UtilityLogCategory) {
  utilityRuntimeCategory.value = category;
  if (utilityLogTab.value !== "runtime") {
    return;
  }
  const target = pickLogForDetail(utilitySelectedLogId.value, runtimeCategoryLogItems.value);
  selectUtilityLog(target);
}

function handleUtilityLogSelect(log: OpenClawMessageLogItem) {
  selectUtilityLog(log);
}

function handleUtilityErrorSummarySelect(summaryKey: string) {
  const target = errorAnalysisSummaries.value.find((item) => item.key === summaryKey)?.latestLog ?? null;
  if (!target) {
    return;
  }
  selectUtilityLog(target, "raw");
}

function handleUtilityLogDetailTabSelect(tab: UtilityLogDetailTab) {
  utilityLogDetailTab.value = tab;
}

async function copyTextToClipboard(text: string, successText: string) {
  const payload = text.trim();
  if (!payload) {
    utilityModalError.value = "暂无可复制内容。";
    return;
  }

  try {
    if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(payload);
    } else if (typeof document !== "undefined") {
      const temp = document.createElement("textarea");
      temp.value = payload;
      temp.setAttribute("readonly", "true");
      temp.style.position = "fixed";
      temp.style.opacity = "0";
      temp.style.left = "-9999px";
      document.body.appendChild(temp);
      temp.focus();
      temp.select();
      const copied = document.execCommand("copy");
      document.body.removeChild(temp);
      if (!copied) {
        throw new Error("复制失败，请手动复制。");
      }
    } else {
      throw new Error("当前环境不支持复制。");
    }
    utilityModalNotice.value = successText;
    utilityModalError.value = "";
  } catch (error) {
    utilityModalError.value = error instanceof Error ? error.message : "复制失败。";
  }
}

async function handleCopyRuntimeLogContent() {
  const section = activeRuntimeLogDetailSection.value;
  if (!section) {
    utilityModalError.value = "当前暂无可复制日志详情。";
    return;
  }
  await copyTextToClipboard(section.text, "已复制当前内容。");
}

async function handleRelatedModelSave() {
  const invoke = getTauriInvoke();
  const draft = relatedModelDraft.value;
  if (!invoke || !draft || relatedResourceModalSaving.value) {
    return;
  }

  const providerId = draft.providerId.trim();
  const baseUrl = draft.baseUrl.trim();
  const model = draft.model.trim();
  if (!providerId) {
    relatedResourceModalError.value = "缺少 providerId，无法保存模型配置。";
    return;
  }
  if (!baseUrl) {
    relatedResourceModalError.value = "请先填写基础 URL。";
    return;
  }
  if (!model) {
    relatedResourceModalError.value = "请先填写模型 ID。";
    return;
  }

  relatedResourceModalSaving.value = true;
  relatedResourceModalError.value = "";
  try {
    const selectedProtocol = resolveProviderProtocolByApiKind(draft.apiKind || draft.protocol);
    const selectedApiKind = normalizeProviderApiKind(
      selectedProtocol,
      draft.apiKind || inferProviderApiKind(selectedProtocol, draft.apiPath)
    );
    await invoke("save_openclaw_provider_config", {
      config: {
        providerId,
        protocol: selectedProtocol,
        apiKind: selectedApiKind,
        baseUrl,
        model,
        apiKey: draft.apiKey.trim()
      }
    });
    relatedResourceModalNotice.value = `模型配置已保存：${providerId}/${model}`;
    await Promise.all([loadRelatedModelSnapshot(), loadAgents()]);
  } catch (error) {
    relatedResourceModalError.value = error instanceof Error ? error.message : "模型配置保存失败。";
  } finally {
    relatedResourceModalSaving.value = false;
  }
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
const chatComposerPlaceholder = computed(() =>
  activeAgent.value ? `发送给 ${stripRoleLabel(activeAgent.value.displayName)}` : "发送给 ClawPet"
);
const chatMessagesForDisplay = computed(() => chatMessages.value.filter((item) => !isLegacyWelcomeMessage(item)));
const isConversationEmpty = computed(() => chatMessagesForDisplay.value.length === 0);
const proxyConfigPlatforms = computed(() => proxyConfigSnapshot.value?.platforms ?? []);
const proxyConfigSelectedPlatform = computed(() => {
  const selected = proxyConfigSelectedProviderId.value;
  if (!selected) {
    return null;
  }
  return proxyConfigPlatforms.value.find((item) => equalsIgnoreCase(item.providerId, selected)) ?? null;
});
const proxyConfigModalSubtitle = computed(() => {
  const snapshot = proxyConfigSnapshot.value;
  if (!snapshot) {
    return "从 openclaw.json 读取并维护 provider 连接配置。";
  }
  return `${snapshot.detail || "读取完成"} · 来源：${snapshot.sourcePath || "—"}`;
});
const relatedResourceModalTitle = computed(() => {
  if (relatedResourceModalTarget.value === "memory") return "关联资源 · 记忆";
  if (relatedResourceModalTarget.value === "skills") return "关联资源 · 技能库";
  if (relatedResourceModalTarget.value === "tools") return "关联资源 · 工具权限";
  if (relatedResourceModalTarget.value === "model") return "关联资源 · 模型";
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
  getStableChatMessages(chatMessagesForDisplay.value).filter((item) => item.role === "assistant" || item.role === "user")
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
const runtimeLogItems = computed(() => {
  const agentId = activeAgent.value?.agentId ?? null;
  return [...(chatRuntimeLogs.value?.logs ?? [])]
    .filter((log) => matchesAgentLog(log, agentId))
    .sort((left, right) => right.createdAt - left.createdAt);
});
const runtimeCategoryLogItems = computed(() =>
  runtimeLogItems.value.filter((log) => matchesRuntimeLogCategory(log, utilityRuntimeCategory.value))
);
const selectedRuntimeLog = computed(() => {
  const source = utilityLogTab.value === "runtime" ? runtimeCategoryLogItems.value : runtimeLogItems.value;
  return pickLogForDetail(utilitySelectedLogId.value, source);
});
const runtimeLogDetailSections = computed(() => {
  const selected = selectedRuntimeLog.value;
  if (!selected) {
    return [] as Array<{ id: UtilityLogDetailTab; label: string; text: string; language: "json" | "text" }>;
  }
  return buildLogDetailSections(selected);
});
const activeRuntimeLogDetailSection = computed(() => {
  if (runtimeLogDetailSections.value.length === 0) {
    return null;
  }
  return (
    runtimeLogDetailSections.value.find((section) => section.id === utilityLogDetailTab.value) ?? runtimeLogDetailSections.value[0]
  );
});
const errorAnalysisSummaries = computed(() => {
  const map = new Map<
    string,
    {
      key: string;
      title: string;
      count: number;
      latestAt: number;
      latestLog: OpenClawMessageLogItem;
      logs: OpenClawMessageLogItem[];
    }
  >();
  for (const log of runtimeLogItems.value) {
    if (!(log.responseStatus >= 400 || Boolean((log.error ?? "").trim()))) {
      continue;
    }
    const key = getErrorSignature(log);
    const existing = map.get(key);
    if (!existing) {
      map.set(key, {
        key,
        title: key,
        count: 1,
        latestAt: log.createdAt,
        latestLog: log,
        logs: [log]
      });
      continue;
    }
    existing.count += 1;
    existing.logs.push(log);
    if (log.createdAt > existing.latestAt) {
      existing.latestAt = log.createdAt;
      existing.latestLog = log;
    }
  }
  return Array.from(map.values()).sort((left, right) => {
    if (right.count !== left.count) {
      return right.count - left.count;
    }
    return right.latestAt - left.latestAt;
  });
});
const selectedErrorSummary = computed(() => {
  const selected = selectedRuntimeLog.value;
  if (!selected) {
    return null;
  }
  const key = getErrorSignature(selected);
  return errorAnalysisSummaries.value.find((item) => item.key === key) ?? null;
});
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

const dashboardGatewayState = computed(() => {
  const snapshot = dashboardGatewayHealth.value;
  const rawStatus = snapshot?.status ?? "checking";
  const tone = mapGatewayStatusTone(rawStatus);
  const label = mapGatewayStatusLabel(rawStatus);
  const latencyLabel =
    typeof snapshot?.latencyMs === "number" && Number.isFinite(snapshot.latencyMs)
      ? `${Math.max(0, Math.round(snapshot.latencyMs))} ms`
      : "—";
  const portLabel =
    typeof snapshot?.gatewayPort === "number" && Number.isFinite(snapshot.gatewayPort)
      ? `${Math.max(0, Math.round(snapshot.gatewayPort))}`
      : "—";
  const detail = snapshot?.detail?.trim() || (rawStatus === "online" ? "网关连接正常。" : "正在读取网关状态。");
  return {
    rawStatus,
    tone,
    label,
    detail,
    latencyLabel,
    portLabel
  };
});

const dashboardChannelStats = computed(() => {
  const channels = dashboardChannelSnapshot.value?.channels ?? [];
  let totalAccounts = 0;
  let connectedAccounts = 0;
  let boundAccounts = 0;
  let activeGroups = 0;

  for (const group of channels) {
    let groupHasConnectedAccount = false;
    for (const account of group.accounts ?? []) {
      totalAccounts += 1;
      const status = (account.status ?? "").trim().toLowerCase();
      const connected = account.configured && (status === "connected" || status === "online");
      if (connected) {
        connectedAccounts += 1;
        groupHasConnectedAccount = true;
      }
      if ((account.agentId ?? "").trim()) {
        boundAccounts += 1;
      }
    }
    if (groupHasConnectedAccount) {
      activeGroups += 1;
    }
  }

  return {
    totalGroups: channels.length,
    totalAccounts,
    connectedAccounts,
    boundAccounts,
    activeGroups
  };
});

const dashboardRuntimeLogItems = computed(() =>
  [...(chatRuntimeLogs.value?.logs ?? [])].sort((left, right) => right.createdAt - left.createdAt)
);

const dashboardTodayStats = computed(() => {
  const logs = dashboardRuntimeLogItems.value;
  const todayStart = new Date();
  todayStart.setHours(0, 0, 0, 0);
  const todayStartMs = todayStart.getTime();
  const todayLogs = logs.filter((log) => log.createdAt >= todayStartMs);
  const failedToday = todayLogs.filter((log) => log.responseStatus >= 400 || Boolean((log.error ?? "").trim())).length;
  const oldestLogAt = logs.length > 0 ? logs[logs.length - 1]?.createdAt ?? null : null;
  const newestLogAt = logs.length > 0 ? logs[0]?.createdAt ?? null : null;
  return {
    total: logs.length,
    today: todayLogs.length,
    failedToday,
    oldestLogAt,
    newestLogAt
  };
});

const dashboardLegacyStatsSummary = computed(() => {
  const logs = dashboardRuntimeLogItems.value;
  const failures = logs.filter((log) => log.responseStatus >= 400 || Boolean((log.error ?? "").trim())).length;
  const totalDuration = logs.reduce((sum, log) => sum + (Number.isFinite(log.duration) ? Math.max(0, log.duration) : 0), 0);
  const averageDuration = logs.length > 0 ? Math.round(totalDuration / logs.length) : 0;
  const startOfToday = new Date();
  startOfToday.setHours(0, 0, 0, 0);
  const todayStartMs = startOfToday.getTime();
  const startOfSevenDays = new Date(startOfToday);
  startOfSevenDays.setDate(startOfSevenDays.getDate() - 6);
  const sevenDaysStartMs = startOfSevenDays.getTime();

  const todayTokens = logs.reduce((sum, log) => (log.createdAt >= todayStartMs ? sum + getEffectiveLogTotalTokens(log) : sum), 0);
  const sevenDayTokens = logs.reduce((sum, log) => (log.createdAt >= sevenDaysStartMs ? sum + getEffectiveLogTotalTokens(log) : sum), 0);
  const totalTokens = logs.reduce((sum, log) => sum + getEffectiveLogTotalTokens(log), 0);

  return {
    platformCount: dashboardPlatformSnapshot.value?.platforms?.length ?? 0,
    callCount: logs.length,
    failures,
    averageDuration,
    todayTokens,
    sevenDayTokens,
    totalTokens
  };
});

const dashboardPrimaryModel = computed(() => {
  const preferred = activeAgent.value?.model?.trim();
  if (preferred) {
    return preferred;
  }

  const modelCounter = new Map<string, number>();
  for (const agent of agents.value) {
    const model = (agent.model ?? "").trim();
    if (!model) {
      continue;
    }
    modelCounter.set(model, (modelCounter.get(model) ?? 0) + 1);
  }

  let winner = "";
  let winnerCount = -1;
  for (const [model, count] of modelCounter) {
    if (count > winnerCount) {
      winner = model;
      winnerCount = count;
    }
  }

  return winner || "llm/petclaw-1.0";
});

const dashboardOnlineAgentCount = computed(() => agents.value.filter((agent) => agent.statusTone !== "offline").length);
const dashboardBusyAgentCount = computed(() => agents.value.filter((agent) => agent.statusTone === "busy").length);

const dashboardRuntimeStatus = computed(() => {
  const gateway = dashboardGatewayState.value;
  const onlineAgents = dashboardOnlineAgentCount.value;
  const busyAgents = dashboardBusyAgentCount.value;

  if (isDashboardRefreshing.value && !dashboardLastRefreshedAt.value) {
    return {
      label: "检测中",
      tone: "neutral" as DashboardHealthTone,
      detail: "正在读取 OpenClaw 运行状态。"
    };
  }

  if (gateway.rawStatus === "online" || onlineAgents > 0) {
    return {
      label: "运行中",
      tone: busyAgents > 0 ? ("warn" as DashboardHealthTone) : ("online" as DashboardHealthTone),
      detail:
        gateway.rawStatus === "online"
          ? `网关在线，当前 ${onlineAgents} 个员工在线。`
          : `员工在线 ${onlineAgents} 个，等待网关状态恢复。`
    };
  }

  if (gateway.rawStatus === "unconfigured") {
    return {
      label: "待配置",
      tone: "warn" as DashboardHealthTone,
      detail: gateway.detail || "尚未完成网关配置。"
    };
  }

  if (gateway.rawStatus === "unsupported") {
    return {
      label: "不可用",
      tone: "neutral" as DashboardHealthTone,
      detail: gateway.detail || "当前环境不支持自动探测。"
    };
  }

  return {
    label: "离线",
    tone: "offline" as DashboardHealthTone,
    detail: gateway.detail || "未检测到可用的运行服务。"
  };
});

const dashboardHealthBadge = computed(() => {
  if (isDashboardRefreshing.value) {
    return { label: "状态刷新中", tone: "neutral" as DashboardHealthTone };
  }
  if (dashboardRuntimeStatus.value.label === "离线") {
    return { label: "系统离线", tone: "offline" as DashboardHealthTone };
  }
  if (dashboardTodayStats.value.failedToday > 5) {
    return { label: "异常偏多", tone: "warn" as DashboardHealthTone };
  }
  if (dashboardGatewayState.value.rawStatus === "online") {
    return { label: "系统运行正常", tone: "online" as DashboardHealthTone };
  }
  if (dashboardGatewayState.value.rawStatus === "unconfigured") {
    return { label: "等待配置", tone: "warn" as DashboardHealthTone };
  }
  return { label: "状态待确认", tone: "neutral" as DashboardHealthTone };
});

const dashboardLegacyStatCards = computed<DashboardMetricCard[]>(() => {
  const summary = dashboardLegacyStatsSummary.value;
  const gateway = dashboardGatewayState.value;
  const gatewayValue = gateway.rawStatus === "online" ? `${gateway.label} · ${gateway.latencyLabel}` : gateway.label;
  return [
    {
      id: "platforms",
      label: "代理平台",
      value: formatInteger(summary.platformCount),
      detail: `已读取 ${summary.platformCount} 个平台配置`,
      tone: summary.platformCount > 0 ? "online" : "neutral",
      icon: "platform"
    },
    {
      id: "request-count",
      label: "调用总数",
      value: formatInteger(summary.callCount),
      detail: "基于 OpenClaw 运行日志统计",
      tone: "neutral",
      icon: "request"
    },
    {
      id: "gateway-status",
      label: "网关状态",
      value: gatewayValue,
      detail: `端口 ${gateway.portLabel} · ${gateway.detail}`,
      tone: gateway.tone,
      icon: "gateway"
    },
    {
      id: "avg-duration",
      label: "平均耗时",
      value: `${formatInteger(summary.averageDuration)} ms`,
      detail: "按全部请求平均计算",
      tone: summary.averageDuration > 0 ? "neutral" : "warn",
      icon: "latency"
    },
    {
      id: "today-token",
      label: "今日 Token",
      value: formatInteger(summary.todayTokens),
      detail: "今日 00:00 以来累计",
      tone: summary.todayTokens > 0 ? "online" : "neutral",
      icon: "tokenToday"
    },
    {
      id: "seven-day-token",
      label: "7 天 Token",
      value: formatInteger(summary.sevenDayTokens),
      detail: "最近 7 天滑动窗口",
      tone: summary.sevenDayTokens > 0 ? "neutral" : "warn",
      icon: "tokenWeek"
    },
    {
      id: "total-token",
      label: "累计 Token",
      value: formatInteger(summary.totalTokens),
      detail: "全部可用日志累计",
      tone: summary.totalTokens > 0 ? "neutral" : "warn",
      icon: "tokenTotal"
    },
    {
      id: "failure-count",
      label: "失败请求",
      value: formatInteger(summary.failures),
      detail: summary.callCount > 0 ? `失败率 ${Math.round((summary.failures / summary.callCount) * 100)}%` : "暂无请求",
      tone: summary.failures > 0 ? "warn" : "online",
      icon: "failure"
    }
  ];
});

const dashboardStatusCards = computed<DashboardMetricCard[]>(() => {
  const uptimeMs =
    typeof dashboardTodayStats.value.oldestLogAt === "number"
      ? Math.max(0, Date.now() - dashboardTodayStats.value.oldestLogAt)
      : null;
  return [
    {
      id: "runtime",
      label: "OpenClaw",
      value: dashboardRuntimeStatus.value.label,
      detail: dashboardRuntimeStatus.value.detail,
      tone: dashboardRuntimeStatus.value.tone,
      icon: "runtime"
    },
    {
      id: "channels",
      label: "活跃通道",
      value: `${dashboardChannelStats.value.activeGroups} 个`,
      detail: `共 ${dashboardChannelStats.value.totalGroups} 类频道，账号 ${dashboardChannelStats.value.totalAccounts} 个`,
      tone:
        dashboardChannelStats.value.activeGroups > 0
          ? ("online" as DashboardHealthTone)
          : dashboardChannelStats.value.totalGroups > 0
            ? ("warn" as DashboardHealthTone)
            : ("neutral" as DashboardHealthTone),
      icon: "channel"
    },
    {
      id: "model",
      label: "AI 模型",
      value: dashboardPrimaryModel.value,
      detail: "优先使用当前员工模型，未选择时按全局占比推断。",
      tone: "neutral",
      icon: "model"
    },
    {
      id: "uptime",
      label: "运行时长",
      value: formatRunDurationLabel(uptimeMs),
      detail:
        typeof dashboardTodayStats.value.oldestLogAt === "number"
          ? `起始于 ${formatDateTime(dashboardTodayStats.value.oldestLogAt)}`
          : "暂无可用于估算运行时长的日志。",
      tone: uptimeMs ? "online" : "neutral",
      icon: "uptime"
    },
    {
      id: "memory",
      label: "内存占用",
      value: dashboardJsHeapUsageMb.value === null ? "暂不可用" : `${dashboardJsHeapUsageMb.value} MB`,
      detail: "基于 WebView JS Heap 采样，供趋势观察。",
      tone: dashboardJsHeapUsageMb.value === null ? "warn" : "neutral",
      icon: "memory"
    },
    {
      id: "today",
      label: "今日消息",
      value: `${dashboardTodayStats.value.today} 条`,
      detail: `失败 ${dashboardTodayStats.value.failedToday} 条，累计 ${dashboardTodayStats.value.total} 条`,
      tone: dashboardTodayStats.value.failedToday > 0 ? "warn" : "online",
      icon: "message"
    },
    {
      id: "connected",
      label: "已连接通道",
      value: `${dashboardChannelStats.value.connectedAccounts} 个`,
      detail: `已绑定员工 ${dashboardChannelStats.value.boundAccounts} 个`,
      tone: dashboardChannelStats.value.connectedAccounts > 0 ? "online" : "neutral",
      icon: "connected"
    },
    {
      id: "staff-online",
      label: "在线员工",
      value: `${dashboardOnlineAgentCount.value} 个`,
      detail: `忙碌 ${dashboardBusyAgentCount.value} 个 · 总计 ${agents.value.length} 个`,
      tone: dashboardOnlineAgentCount.value > 0 ? "online" : "offline",
      icon: "staff"
    }
  ];
});

const dashboardRecentActivities = computed<DashboardActivityItem[]>(() =>
  dashboardRuntimeLogItems.value.slice(0, 12).map((log) => {
    const statusTone = log.responseStatus >= 500 ? "offline" : log.responseStatus >= 400 ? "warn" : "online";
    const tag = log.responseStatus >= 500 ? "ERR" : log.responseStatus >= 400 ? "WARN" : "SYS";
    const targetPath = (log.path || log.endpoint || "/").trim() || "/";
    const detail = (log.error ?? "").trim();
    const baseSummary = `${log.method} ${targetPath} · ${log.responseStatus}`;
    const summary = detail ? `${baseSummary} · ${detail}` : baseSummary;
    const compactSummary = summary.length > 130 ? `${summary.slice(0, 130)}...` : summary;
    return {
      id: log.id,
      timeLabel: formatCompactTime(log.createdAt),
      tag,
      summary: compactSummary,
      tone: statusTone
    };
  })
);

const dashboardDetailText = computed(() => {
  const parts = [
    staffSourceDetail.value,
    dashboardPlatformSnapshot.value?.detail ?? "",
    dashboardChannelSnapshot.value?.detail ?? "",
    chatRuntimeLogs.value?.detail ?? ""
  ]
    .map((item) => item.trim())
    .filter(Boolean);
  if (parts.length === 0) {
    return "仪表盘已加载。";
  }
  return parts.join(" · ");
});

function applyRoleWorkflowOverride(role: AgencyRosterRole) {
  const override = roleWorkflowOverrides.value[role.id];
  if (!override) {
    return role;
  }
  return {
    ...role,
    nameZh: override.nameZh ?? role.nameZh,
    workflowZh: override.workflowZh ?? role.workflowZh
  };
}

const recruitmentRoleIndex = computed(() => {
  const index = new Map<string, RoleWorkflowModalBase>();
  for (const division of recruitmentDivisions) {
    for (const group of division.groups) {
      for (const role of group.roles) {
        index.set(role.id, {
          role: applyRoleWorkflowOverride(role),
          divisionTitleZh: division.titleZh,
          groupTitleZh: group.titleZh
        });
      }
    }
  }
  return index;
});
const activeRoleWorkflowBase = computed(() => {
  const activeRoleId = roleWorkflowDetailRoleId.value;
  if (!activeRoleId) {
    return null;
  }
  return recruitmentRoleIndex.value.get(activeRoleId) ?? null;
});
const activeRoleWorkflowOverride = computed(() => {
  const activeRoleId = roleWorkflowDetailRoleId.value;
  if (!activeRoleId) {
    return null;
  }
  return roleWorkflowOverrides.value[activeRoleId] ?? null;
});
const roleWorkflowDetailSavedVersions = computed(() => activeRoleWorkflowOverride.value?.detailVersions ?? []);
const isRoleWorkflowDraftChanged = computed(() => {
  if (!roleWorkflowDetailRoleId.value || roleWorkflowDetailLoading.value) {
    return false;
  }
  return (
    roleWorkflowDetailDraft.value.contentZh !== roleWorkflowDetailOriginalContent.value ||
    roleWorkflowNameZhDraft.value.trim() !== roleWorkflowNameZhOriginal.value.trim()
  );
});
const canSaveRoleWorkflowDraft = computed(
  () => Boolean(roleWorkflowDetailRoleId.value) && isRoleWorkflowDraftChanged.value && !roleWorkflowDetailLoading.value
);

const recruitmentKeywordNormalized = computed(() => recruitmentKeyword.value.trim().toLowerCase());
const filteredRecruitmentDivisions = computed<AgencyRosterDivision[]>(() => {
  const keyword = recruitmentKeywordNormalized.value;

  return recruitmentDivisions
    .map((division) => {
      const groups = division.groups
        .map((group) => {
          const roles = group.roles
            .map((role) => applyRoleWorkflowOverride(role))
            .filter((role) => {
              if (!keyword) {
                return true;
              }
              const blob =
                `${role.nameZh} ${role.nameEn} ${role.workflowZh} ${role.sourcePath} ${division.titleZh} ${group.titleZh ?? ""}`.toLowerCase();
              return blob.includes(keyword);
            });
          return {
            ...group,
            roles
          };
        })
        .filter((group) => group.roles.length > 0);

      return {
        ...division,
        groups,
        count: groups.reduce((sum, group) => sum + group.roles.length, 0)
      };
    })
    .filter((division) => division.count > 0);
});
const recruitmentTotalCount = computed(() => recruitmentDivisions.reduce((sum, division) => sum + division.count, 0));
const recruitmentVisibleCount = computed(() => filteredRecruitmentDivisions.value.reduce((sum, division) => sum + division.count, 0));

const skillMarketCurrentCategory = computed(
  () => skillMarketCategories.find((item) => item.id === activeSkillMarketCategory.value) ?? skillMarketCategories[0]
);
const skillMarketSourceSkills = computed(() =>
  activeSkillMarketCategory.value === "top" ? skillMarketTopSkills.value : skillMarketCategorySkills.value
);
const skillMarketBaseSkills = computed(() =>
  skillMarketSearch.value.trim() ? skillMarketGlobalSkills.value : skillMarketSourceSkills.value
);
const filteredSkillMarketSkills = computed(() => {
  const keyword = skillMarketSearch.value.trim().toLowerCase();
  if (!keyword) {
    return skillMarketBaseSkills.value;
  }
  return skillMarketBaseSkills.value.filter((skill) => {
    const blob = [
      skill.name,
      skill.ownerName,
      skill.category,
      skill.descriptionZh,
      skill.description,
      ...(skill.tags ?? [])
    ]
      .join(" ")
      .toLowerCase();
    return blob.includes(keyword);
  });
});
const sortedSkillMarketSkills = computed(() => {
  return [...filteredSkillMarketSkills.value].sort((left, right) => {
    if (skillMarketSortBy.value === "downloads" && right.downloads !== left.downloads) {
      return right.downloads - left.downloads;
    }
    if (skillMarketSortBy.value === "stars" && right.stars !== left.stars) {
      return right.stars - left.stars;
    }
    if (right.score !== left.score) {
      return right.score - left.score;
    }
    return left.name.localeCompare(right.name, "zh-CN");
  });
});
const skillMarketTotal = computed(() =>
  activeSkillMarketCategory.value === "top" ? skillMarketTopTotal.value : skillMarketCategoryTotal.value
);
const skillMarketLocalTotal = computed(() => sortedSkillMarketSkills.value.length);
const skillMarketCurrentTotalPages = computed(() => {
  const pageSize = Math.max(1, skillMarketPageSize.value);
  return Math.max(1, Math.ceil(skillMarketLocalTotal.value / pageSize));
});
const pagedSkillMarketSkills = computed(() => {
  const page = Math.min(Math.max(skillMarketPage.value, 1), skillMarketCurrentTotalPages.value);
  const size = Math.max(1, skillMarketPageSize.value);
  const start = (page - 1) * size;
  return sortedSkillMarketSkills.value.slice(start, start + size);
});
const skillMarketPageNumbers = computed(() => {
  const total = skillMarketCurrentTotalPages.value;
  const current = Math.min(Math.max(skillMarketPage.value, 1), total);
  const start = Math.max(1, current - 2);
  const end = Math.min(total, start + 4);
  const numbers: number[] = [];
  for (let page = start; page <= end; page += 1) {
    numbers.push(page);
  }
  return numbers;
});
const skillMarketCanPrevPage = computed(() => skillMarketPage.value > 1);
const skillMarketCanNextPage = computed(() => skillMarketPage.value < skillMarketCurrentTotalPages.value);
const skillMarketSummaryText = computed(() => {
  const categoryLabel = skillMarketCurrentCategory.value?.label ?? "技能市场";
  const displayed = pagedSkillMarketSkills.value.length;
  const isSearching = Boolean(skillMarketSearch.value.trim());
  const total = isSearching ? skillMarketLocalTotal.value : Math.max(skillMarketTotal.value, skillMarketLocalTotal.value);
  return `分类：${categoryLabel} · 展示 ${displayed} / ${total} · 第 ${skillMarketPage.value}/${skillMarketCurrentTotalPages.value} 页`;
});

const startupOpenClawProgress = computed(() => {
  const total = startupOpenClawSteps.value.length;
  if (total === 0) {
    return 0;
  }
  let score = 0;
  for (const step of startupOpenClawSteps.value) {
    if (step.status === "done") {
      score += 1;
      continue;
    }
    if (step.status === "installing") {
      score += 0.55;
    }
  }
  return Math.max(4, Math.min(100, Math.round((score / total) * 100)));
});

const taskSummary = computed(() => {
  const total = taskItems.value.length;
  const todo = taskItems.value.filter((item) => item.status === "todo").length;
  const doing = taskItems.value.filter((item) => item.status === "in_progress").length;
  const review = taskItems.value.filter((item) => item.status === "in_review").length;
  const done = taskItems.value.filter((item) => item.status === "done").length;
  const cancelled = taskItems.value.filter((item) => item.status === "cancelled").length;

  return { total, todo, doing, review, done, cancelled };
});

const activeTaskProjectName = computed(() =>
  taskProjectNames.value.includes(activeTaskProject.value) ? activeTaskProject.value : DEFAULT_TASK_PROJECT_NAME
);

const activeProjectTaskItems = computed(() =>
  sortTaskRecords(taskItems.value.filter((item) => isTaskInProject(item, activeTaskProjectName.value)))
);

const activeProjectTaskColumns = computed(() =>
  taskBoardColumns.map((column) => ({
    ...column,
    tasks: sortTaskRecordsForColumn(activeProjectTaskItems.value.filter((item) => item.status === column.id))
  }))
);

const taskProjectCards = computed<TaskProjectCard[]>(() =>
  taskProjectNames.value.map((projectName) => {
    const records = taskItems.value.filter((item) => isTaskInProject(item, projectName));
    return {
      name: projectName,
      count: records.length,
      activeCount: records.filter((item) => item.status !== "done" && item.status !== "cancelled").length,
      doneCount: records.filter((item) => item.status === "done").length,
      reviewCount: records.filter((item) => item.status === "in_review").length,
      updatedAt: records.reduce((latest, item) => (latest === null || item.updatedAt > latest ? item.updatedAt : latest), null as number | null),
      isDefault: projectName === DEFAULT_TASK_PROJECT_NAME
    };
  })
);

onMounted(async () => {
  taskItems.value = loadTasks();
  taskProjectNames.value = loadTaskProjectsFromStorage();
  syncTaskProjectNamesFromTasks();
  await ensureStartupOpenClawReady();
  await Promise.all([loadAgents(), refreshDashboardData()]);
  await scrollMessagesToBottom();
});

watch(
  () => activeSection.value,
  (section) => {
    if (section === "dashboard") {
      void refreshDashboardData();
      return;
    }
    if (section === "skills" && !skillMarketLoading.value && skillMarketTopSkills.value.length === 0 && skillMarketCategorySkills.value.length === 0) {
      void refreshSkillMarket();
    }
  }
);

watch(
  [skillMarketCurrentTotalPages, () => skillMarketSearch.value],
  ([totalPages]) => {
    if (skillMarketPage.value > totalPages) {
      skillMarketPage.value = totalPages;
    }
  }
);
</script>

<template>
  <div class="chat-page">
    <div v-if="startupOpenClawOverlayVisible" class="startup-openclaw-overlay" aria-live="polite">
      <section class="startup-openclaw-card" role="dialog" aria-modal="true" aria-label="安装 OpenClaw">
        <div class="startup-openclaw-orbit" :class="{ 'is-spinning': startupOpenClawInstalling }" aria-hidden="true">
          <span class="startup-openclaw-orbit__arc startup-openclaw-orbit__arc--outer" />
          <span class="startup-openclaw-orbit__arc startup-openclaw-orbit__arc--inner" />
          <span class="startup-openclaw-orbit__core">
            <img class="startup-openclaw-orbit__logo" :src="appLogoUrl" alt="ClawPet Logo" />
          </span>
        </div>
        <h2>安装 OpenClaw</h2>
        <p class="startup-openclaw-status">{{ startupOpenClawStatusText }}</p>

        <div class="startup-openclaw-step-list">
          <article
            v-for="step in startupOpenClawSteps"
            :key="step.id"
            class="startup-openclaw-step"
            :class="`is-${step.status}`"
          >
            <div class="startup-openclaw-step__main">
              <span class="startup-openclaw-step__icon" :class="`is-${step.status}`" />
              <strong>{{ step.title }}</strong>
            </div>
            <em>{{ getStartupOpenClawStepBadge(step) }}</em>
          </article>
        </div>

        <div class="startup-openclaw-progress">
          <i :style="{ width: `${startupOpenClawProgress}%` }" />
        </div>

        <p v-if="startupOpenClawInstallError" class="startup-openclaw-error">{{ startupOpenClawInstallError }}</p>
        <pre v-if="startupOpenClawRuntimeLogs" class="startup-openclaw-logs">{{ startupOpenClawRuntimeLogs }}</pre>

        <div v-if="startupOpenClawInstallError && !startupOpenClawInstalling" class="startup-openclaw-actions">
          <button type="button" @click="retryStartupOpenClawInstall">重试安装</button>
          <button type="button" class="is-ghost" @click="dismissStartupOpenClawOverlay">稍后再说</button>
        </div>
      </section>
    </div>

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
              @click="handleSidebarSectionChange(item.id)"
            >
              <span class="nav-item__icon" aria-hidden="true">
                <svg v-if="item.id === 'chat'" viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
                <svg v-else-if="item.id === 'dashboard'" viewBox="0 0 24 24"><path d="M3 13h8V3H3zm10 8h8V3h-8zm-10 0h8v-6H3z" /></svg>
                <svg v-else-if="item.id === 'recruitment'" viewBox="0 0 24 24"><path d="M10 2a8 8 0 1 0 5.3 14l4.9 4.9 1.4-1.4-4.9-4.9A8 8 0 0 0 10 2zm0 2a6 6 0 1 1-6 6 6 6 0 0 1 6-6z" /></svg>
                <svg v-else-if="item.id === 'skills'" viewBox="0 0 24 24"><path d="M14.7 6.3a4 4 0 0 0-5.4 5.8L3 18.5V21h2.5l6.4-6.3a4 4 0 0 0 2.8-8.4zM14 10a2 2 0 1 1 1.4-.6A2 2 0 0 1 14 10z" /></svg>
                <svg v-else viewBox="0 0 24 24"><path d="M9 11H7v2h2zm4 0h-2v2h2zm4 0h-2v2h2zm2-8H5a2 2 0 0 0-2 2v14l4-4h12a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2z" /></svg>
              </span>
              <span class="nav-item__label">{{ item.label }}</span>
              <span v-if="item.id === 'chat'" class="nav-item__badge">{{ sidebarChatBadge }}</span>
            </button>
          </div>

          <div class="sidebar-spacer" />
          <div class="sidebar-bottom-actions">
            <button
              class="nav-item nav-item--secondary"
              :class="{ active: activeSection === 'chat' && isSidebarSettingsModalOpen }"
              type="button"
              title="设置"
              aria-label="打开设置"
              @click="openSidebarSettings"
            >
              <span class="nav-item__icon" aria-hidden="true">
                <svg viewBox="0 0 24 24">
                  <circle cx="12" cy="12" r="3.2" />
                  <path d="M19.4 15a1 1 0 0 0 .2 1.1l.1.1a1.8 1.8 0 1 1-2.5 2.5l-.1-.1a1 1 0 0 0-1.1-.2h-.1a1 1 0 0 0-.6.9V20a1.8 1.8 0 0 1-3.6 0v-.1a1 1 0 0 0-.6-.9h-.1a1 1 0 0 0-1.1.2l-.1.1a1.8 1.8 0 0 1-2.5-2.5l.1-.1a1 1 0 0 0 .2-1.1v-.1a1 1 0 0 0-.9-.6H4a1.8 1.8 0 0 1 0-3.6h.1a1 1 0 0 0 .9-.6v-.1a1 1 0 0 0-.2-1.1l-.1-.1a1.8 1.8 0 1 1 2.5-2.5l.1.1a1 1 0 0 0 1.1.2h.1a1 1 0 0 0 .6-.9V4a1.8 1.8 0 0 1 3.6 0v.1a1 1 0 0 0 .6.9h.1a1 1 0 0 0 1.1-.2l.1-.1a1.8 1.8 0 1 1 2.5 2.5l-.1.1a1 1 0 0 0-.2 1.1v.1a1 1 0 0 0 .9.6H20a1.8 1.8 0 0 1 0 3.6h-.1a1 1 0 0 0-.9.6z" />
                </svg>
              </span>
              <span class="nav-item__label">设置</span>
            </button>
            <button
              class="nav-item nav-item--secondary"
              :class="{ active: utilityModalType === 'logs' }"
              type="button"
              title="日志"
              aria-label="打开日志"
              @click="openSidebarLogs"
            >
              <span class="nav-item__icon" aria-hidden="true">
                <svg viewBox="0 0 24 24"><path d="M8 4h8M8 4a2 2 0 0 0-2 2v1h12V6a2 2 0 0 0-2-2M6 7h12v11a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2z" /><path d="m8 15 3-3 2 2 3-4" /></svg>
              </span>
              <span class="nav-item__label">日志</span>
            </button>
            <button
              class="nav-item nav-item--secondary"
              :class="{ active: isProxyConfigModalOpen }"
              type="button"
              title="代理配置"
              aria-label="打开代理配置"
              @click="openSidebarProxyConfig"
            >
              <span class="nav-item__icon" aria-hidden="true">
                <svg viewBox="0 0 24 24"><path d="M4 6h16v4H4zM4 14h16v4H4zM8 10v4M16 10v4" /></svg>
              </span>
              <span class="nav-item__label">代理配置</span>
            </button>
            <button
              class="nav-item nav-item--secondary"
              type="button"
              title="OpenClaw 网页版"
              aria-label="访问 OpenClaw 网页版"
              @click="openSidebarOpenClawWeb"
            >
              <span class="nav-item__icon" aria-hidden="true">
                <svg viewBox="0 0 24 24"><path d="M3 12h18M12 3a9 9 0 1 0 0 18M12 3c2.5 2.3 4 5.4 4 9s-1.5 6.7-4 9M12 3c-2.5 2.3-4 5.4-4 9s1.5 6.7 4 9" /></svg>
              </span>
              <span class="nav-item__label">OpenClaw 网页版</span>
            </button>
            <button
              class="nav-item nav-item--secondary nav-item--legacy"
              type="button"
              title="旧版运营控制台"
              aria-label="打开旧版运营控制台"
              @click="openSidebarLegacyConsole"
            >
              <span class="nav-item__icon" aria-hidden="true">
                <svg viewBox="0 0 24 24"><path d="M4 5h16v10H4zM8 19h8M10 15v4M14 15v4" /><path d="M8 9h8M8 12h4" /></svg>
              </span>
              <span class="nav-item__label">旧版运营控制台</span>
            </button>
          </div>
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
                  <img
                    v-if="getAgentAvatarUrl(agent)"
                    class="agent-avatar__image"
                    :src="getAgentAvatarUrl(agent) ?? undefined"
                    :alt="`${stripRoleLabel(agent.displayName)} 头像`"
                    loading="lazy"
                    decoding="async"
                  />
                  <span v-else>{{ getAgentInitial(agent) }}</span>
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
                  <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" /><path d="M12 8v5l3 2" /></svg>
                </button>
                <button
                  type="button"
                  class="header-btn"
                  :class="{ 'is-active': utilityModalType === 'logs' }"
                  title="运行日志"
                  aria-label="打开运行日志"
                  @click="openUtilityModal('logs')"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 4h8M8 4a2 2 0 0 0-2 2v1h12V6a2 2 0 0 0-2-2M6 7h12v11a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2z" /><path d="m8 15 3-3 2 2 3-4" /></svg>
                </button>
                <button
                  type="button"
                  class="header-btn"
                  title="归档当前会话"
                  aria-label="归档当前会话"
                  :disabled="!activeAgent"
                  @click="handleArchiveCurrentChat"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16v4H4zM6 11v8h12v-8" /><path d="M12 13v4m0 0-2-2m2 2 2-2" /></svg>
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
              <div ref="messageScroller" class="chat-window__messages" :class="{ 'chat-window__messages--empty': isConversationEmpty }">
                <div v-if="isConversationEmpty" class="chat-empty-state">
                  <div class="chat-empty-state__logo" aria-hidden="true">
                    <svg viewBox="0 0 48 48">
                      <circle cx="14" cy="15" r="5.2" />
                      <circle cx="34" cy="15" r="5.2" />
                      <circle cx="11.5" cy="27" r="4.2" />
                      <circle cx="36.5" cy="27" r="4.2" />
                      <path d="M24 40c7.8 0 13-4.4 13-10 0-4.6-3.9-8.4-8.5-8.4-2.2 0-4.3.9-5.5 2.5-1.2-1.6-3.2-2.5-5.5-2.5C12.9 21.6 9 25.4 9 30c0 5.6 5.2 10 15 10z" />
                    </svg>
                  </div>
                  <h3>ClawPet</h3>
                  <p>描述你的目标，ClawPet 会分解任务并执行反馈。</p>
                  <div class="chat-empty-state__actions">
                    <button class="chat-empty-action" type="button" :disabled="!activeAgent" @click="openRelatedResource('model')">
                      <span class="chat-empty-action__icon" aria-hidden="true">
                        <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="3.2" /><path d="M19.4 15a1 1 0 0 0 .2 1.1l.1.1a1.8 1.8 0 1 1-2.5 2.5l-.1-.1a1 1 0 0 0-1.1-.2h-.1a1 1 0 0 0-.6.9V20a1.8 1.8 0 0 1-3.6 0v-.1a1 1 0 0 0-.6-.9h-.1a1 1 0 0 0-1.1.2l-.1.1a1.8 1.8 0 0 1-2.5-2.5l.1-.1a1 1 0 0 0 .2-1.1v-.1a1 1 0 0 0-.9-.6H4a1.8 1.8 0 0 1 0-3.6h.1a1 1 0 0 0 .9-.6v-.1a1 1 0 0 0-.2-1.1l-.1-.1a1.8 1.8 0 1 1 2.5-2.5l.1.1a1 1 0 0 0 1.1.2h.1a1 1 0 0 0 .6-.9V4a1.8 1.8 0 0 1 3.6 0v.1a1 1 0 0 0 .6.9h.1a1 1 0 0 0 1.1-.2l.1-.1a1.8 1.8 0 1 1 2.5 2.5l-.1.1a1 1 0 0 0-.2 1.1v.1a1 1 0 0 0 .9.6H20a1.8 1.8 0 0 1 0 3.6h-.1a1 1 0 0 0-.9.6z" /></svg>
                      </span>
                      <span class="chat-empty-action__content">
                        <strong>快速配置</strong>
                        <small>设置名字、角色，让 ClawPet 更了解你</small>
                      </span>
                    </button>
                    <button class="chat-empty-action" type="button" :disabled="!activeAgent" @click="openRelatedResource('channel')">
                      <span class="chat-empty-action__icon" aria-hidden="true">
                        <svg viewBox="0 0 24 24"><path d="M8 10h.01M12 10h.01M16 10h.01" /><path d="M4 5h16v10H7l-3 4z" /></svg>
                      </span>
                      <span class="chat-empty-action__content">
                        <strong>一键接入飞书</strong>
                        <small>自动配置飞书机器人，无需手动操作</small>
                      </span>
                    </button>
                  </div>
                </div>

                <template v-else>
                  <article
                    v-for="message in chatMessagesForDisplay"
                    :key="message.id"
                    class="message-row"
                    :class="[`message-row--${message.role}`, `message-row--${message.status}`]"
                  >
                    <div class="message-bubble">{{ message.text }}</div>
                    <span class="message-time">{{ getMessageTimeLabel(message) }}</span>
                  </article>
                </template>
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
                        <button class="chat-settings-resource-action" type="button" @click="openRelatedResource('model')">
                          <span class="chat-settings-resource-action__icon">模</span>
                          <span class="chat-settings-resource-action__content">
                            <span class="chat-settings-resource-action__main">模型 {{ activeAgent.model || "未配置" }}</span>
                            <span class="chat-settings-resource-action__sub">编辑基础 URL、模型 ID、协议与 API 密钥</span>
                          </span>
                          <span class="chat-settings-resource-action__arrow">›</span>
                        </button>
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
                      <div class="chat-settings-list__row chat-settings-list__row--model">
                        <dt>模型</dt>
                        <dd>
                          <button class="chat-settings-model-trigger" type="button" @click="openRelatedResource('model')">
                            <span class="chat-settings-model-trigger__value">{{ activeAgent.model || "—" }}</span>
                            <span class="chat-settings-model-trigger__hint">修改配置 ›</span>
                          </button>
                        </dd>
                      </div>
                      <div class="chat-settings-list__row">
                        <dt>工具权限</dt>
                        <dd>{{ activeAgent.toolsProfile || "—" }}</dd>
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
              <div class="composer-panel">
                <div class="composer-input-shell">
                  <button class="composer-input-action" type="button" title="附件" aria-label="添加附件">
                    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M16.5 6.5 9 14a3 3 0 1 1-4.2-4.2l8-8a5 5 0 1 1 7.1 7.1l-9 9a7 7 0 1 1-9.9-9.9l8.5-8.5" /></svg>
                  </button>
                  <input v-model="chatInput" type="text" :placeholder="chatComposerPlaceholder" :disabled="!activeAgent" @keydown.enter.prevent="submitChat" />
                  <button class="composer-send" type="button" :disabled="!activeAgent || isSending || !chatInput.trim()" @click="submitChat">
                    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 12h14M13 5l7 7-7 7" /></svg>
                  </button>
                </div>
                <div class="composer-meta">
                  <span class="composer-model-chip">{{ activeAgent?.model || "ClawPet" }}</span>
                  <button
                    class="composer-btn composer-btn--archive"
                    type="button"
                    :disabled="!activeAgent || currentSessionMessages.length === 0"
                    @click="handleArchiveCurrentChat"
                  >
                    归档会话
                  </button>
                </div>
              </div>
            </footer>
          </section>
        </template>

        <main v-else class="module-board" :class="{ 'module-board--dashboard': activeSection === 'dashboard' }">
          <template v-if="activeSection === 'dashboard'">
            <header class="module-board__header module-board__header--dashboard" @mousedown.left="handleRegionMouseDown">
              <div>
                <h2>仪表盘</h2>
                <p>OpenClaw 运行状态总览 · 最近刷新 {{ formatDateTime(dashboardLastRefreshedAt) }}</p>
              </div>
              <div class="dashboard-toolbar" data-no-window-drag>
                <span class="dashboard-health-badge" :data-tone="dashboardHealthBadge.tone">{{ dashboardHealthBadge.label }}</span>
                <button class="dashboard-toolbar__refresh" type="button" :disabled="isDashboardRefreshing" @click="refreshDashboardData()">
                  {{ isDashboardRefreshing ? "刷新中..." : "刷新状态" }}
                </button>
              </div>
            </header>

            <section class="dashboard-section">
              <header class="dashboard-section__header">
                <div>
                  <strong>统计信息</strong>
                  <p>保留原控制台核心指标，集中展示平台、调用、网关和 Token 情况。</p>
                </div>
              </header>

              <div class="dashboard-stats-grid">
                <article v-for="card in dashboardLegacyStatCards" :key="card.id" class="dashboard-metric-card dashboard-metric-card--compact" :data-tone="card.tone">
                  <div class="dashboard-metric-card__head">
                    <span>{{ card.label }}</span>
                    <span class="dashboard-card-icon" :data-tone="card.tone" aria-hidden="true">
                      <svg v-if="card.icon === 'platform'" viewBox="0 0 24 24"><path d="M4 6h16v4H4zM4 14h16v4H4zM8 10v4M16 10v4" /></svg>
                      <svg v-else-if="card.icon === 'request'" viewBox="0 0 24 24"><path d="M4 5h16v10H7l-3 4zM8 9h8M8 12h5" /></svg>
                      <svg v-else-if="card.icon === 'gateway'" viewBox="0 0 24 24"><path d="M5 12a7 7 0 0 1 14 0M8 12a4 4 0 0 1 8 0M11.5 12a.5.5 0 0 1 1 0" /><circle cx="12" cy="17" r="1.4" /></svg>
                      <svg v-else-if="card.icon === 'latency'" viewBox="0 0 24 24"><circle cx="12" cy="12" r="8" /><path d="M12 8v4l3 2" /></svg>
                      <svg v-else-if="card.icon === 'tokenToday'" viewBox="0 0 24 24"><path d="M13 3 6 14h5l-1 7 8-12h-5z" /></svg>
                      <svg v-else-if="card.icon === 'tokenWeek'" viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="2" /><path d="M8 3v4M16 3v4M4 10h16" /></svg>
                      <svg v-else-if="card.icon === 'tokenTotal'" viewBox="0 0 24 24"><ellipse cx="12" cy="6" rx="7" ry="3" /><path d="M5 6v8c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 10c0 1.7 3.1 3 7 3s7-1.3 7-3" /></svg>
                      <svg v-else-if="card.icon === 'failure'" viewBox="0 0 24 24"><path d="M12 4 3.5 19h17L12 4z" /><path d="M12 9v4M12 16h.01" /></svg>
                      <svg v-else-if="card.icon === 'runtime'" viewBox="0 0 24 24"><path d="M7 8a8 8 0 1 0 10 0M12 5v7" /></svg>
                      <svg v-else-if="card.icon === 'channel'" viewBox="0 0 24 24"><path d="M12 4v2M6.3 6.3l1.4 1.4M4 12h2M17.7 7.7l1.4-1.4M18 12h2M8 12a4 4 0 0 1 8 0M6 12a6 6 0 0 1 12 0" /><circle cx="12" cy="16" r="1.4" /></svg>
                      <svg v-else-if="card.icon === 'model'" viewBox="0 0 24 24"><rect x="7" y="7" width="10" height="10" rx="2" /><path d="M9 3v2M15 3v2M9 19v2M15 19v2M3 9h2M3 15h2M19 9h2M19 15h2" /></svg>
                      <svg v-else-if="card.icon === 'uptime'" viewBox="0 0 24 24"><path d="M12 6v6l4 2" /><circle cx="12" cy="12" r="8" /></svg>
                      <svg v-else-if="card.icon === 'memory'" viewBox="0 0 24 24"><rect x="4" y="7" width="16" height="10" rx="2" /><path d="M8 11h8M8 14h5M7 4v3M11 4v3M15 4v3M19 4v3M7 17v3M11 17v3M15 17v3M19 17v3" /></svg>
                      <svg v-else-if="card.icon === 'message'" viewBox="0 0 24 24"><path d="M4 5h16v10H7l-3 4zM8 9h8M8 12h6" /></svg>
                      <svg v-else-if="card.icon === 'staff'" viewBox="0 0 24 24"><circle cx="9" cy="9" r="3" /><circle cx="16.5" cy="8" r="2.5" /><path d="M4 19a5 5 0 0 1 10 0M13 19a4 4 0 0 1 7 0" /></svg>
                      <svg v-else viewBox="0 0 24 24"><path d="M8 12a4 4 0 0 1 8 0M5 12a7 7 0 0 1 14 0" /><path d="M7 15v2a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2v-2" /></svg>
                    </span>
                  </div>
                  <strong>{{ card.value }}</strong>
                  <p>{{ card.detail }}</p>
                </article>
              </div>
            </section>

            <section class="dashboard-section">
              <header class="dashboard-section__header">
                <div>
                  <strong>当前状态</strong>
                  <p>聚焦今天最常看的运行状态，避免在多个页签之间来回切换。</p>
                </div>
              </header>

            <div class="dashboard-status-grid">
              <article v-for="card in dashboardStatusCards" :key="card.id" class="dashboard-status-card" :data-tone="card.tone">
                <div class="dashboard-metric-card__head">
                  <span>{{ card.label }}</span>
                  <span class="dashboard-card-icon" :data-tone="card.tone" aria-hidden="true">
                    <svg v-if="card.icon === 'platform'" viewBox="0 0 24 24"><path d="M4 6h16v4H4zM4 14h16v4H4zM8 10v4M16 10v4" /></svg>
                    <svg v-else-if="card.icon === 'request'" viewBox="0 0 24 24"><path d="M4 5h16v10H7l-3 4zM8 9h8M8 12h5" /></svg>
                    <svg v-else-if="card.icon === 'gateway'" viewBox="0 0 24 24"><path d="M5 12a7 7 0 0 1 14 0M8 12a4 4 0 0 1 8 0M11.5 12a.5.5 0 0 1 1 0" /><circle cx="12" cy="17" r="1.4" /></svg>
                    <svg v-else-if="card.icon === 'latency'" viewBox="0 0 24 24"><circle cx="12" cy="12" r="8" /><path d="M12 8v4l3 2" /></svg>
                    <svg v-else-if="card.icon === 'tokenToday'" viewBox="0 0 24 24"><path d="M13 3 6 14h5l-1 7 8-12h-5z" /></svg>
                    <svg v-else-if="card.icon === 'tokenWeek'" viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="2" /><path d="M8 3v4M16 3v4M4 10h16" /></svg>
                    <svg v-else-if="card.icon === 'tokenTotal'" viewBox="0 0 24 24"><ellipse cx="12" cy="6" rx="7" ry="3" /><path d="M5 6v8c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 10c0 1.7 3.1 3 7 3s7-1.3 7-3" /></svg>
                    <svg v-else-if="card.icon === 'failure'" viewBox="0 0 24 24"><path d="M12 4 3.5 19h17L12 4z" /><path d="M12 9v4M12 16h.01" /></svg>
                    <svg v-else-if="card.icon === 'runtime'" viewBox="0 0 24 24"><path d="M7 8a8 8 0 1 0 10 0M12 5v7" /></svg>
                    <svg v-else-if="card.icon === 'channel'" viewBox="0 0 24 24"><path d="M12 4v2M6.3 6.3l1.4 1.4M4 12h2M17.7 7.7l1.4-1.4M18 12h2M8 12a4 4 0 0 1 8 0M6 12a6 6 0 0 1 12 0" /><circle cx="12" cy="16" r="1.4" /></svg>
                    <svg v-else-if="card.icon === 'model'" viewBox="0 0 24 24"><rect x="7" y="7" width="10" height="10" rx="2" /><path d="M9 3v2M15 3v2M9 19v2M15 19v2M3 9h2M3 15h2M19 9h2M19 15h2" /></svg>
                    <svg v-else-if="card.icon === 'uptime'" viewBox="0 0 24 24"><path d="M12 6v6l4 2" /><circle cx="12" cy="12" r="8" /></svg>
                    <svg v-else-if="card.icon === 'memory'" viewBox="0 0 24 24"><rect x="4" y="7" width="16" height="10" rx="2" /><path d="M8 11h8M8 14h5M7 4v3M11 4v3M15 4v3M19 4v3M7 17v3M11 17v3M15 17v3M19 17v3" /></svg>
                    <svg v-else-if="card.icon === 'message'" viewBox="0 0 24 24"><path d="M4 5h16v10H7l-3 4zM8 9h8M8 12h6" /></svg>
                    <svg v-else-if="card.icon === 'staff'" viewBox="0 0 24 24"><circle cx="9" cy="9" r="3" /><circle cx="16.5" cy="8" r="2.5" /><path d="M4 19a5 5 0 0 1 10 0M13 19a4 4 0 0 1 7 0" /></svg>
                    <svg v-else viewBox="0 0 24 24"><path d="M8 12a4 4 0 0 1 8 0M5 12a7 7 0 0 1 14 0" /><path d="M7 15v2a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2v-2" /></svg>
                  </span>
                </div>
                <strong>{{ card.value }}</strong>
                <p>{{ card.detail }}</p>
              </article>
            </div>
            </section>

            <section class="dashboard-activity-panel">
              <header class="dashboard-activity-panel__header">
                <div>
                  <strong>最近活动</strong>
                  <p>实时日志 {{ dashboardRecentActivities.length }} 条</p>
                </div>
              </header>

              <div v-if="dashboardRecentActivities.length === 0" class="dashboard-activity-panel__empty">暂无活动记录。</div>
              <div v-else class="dashboard-activity-list">
                <article v-for="activity in dashboardRecentActivities" :key="activity.id" class="dashboard-activity-item">
                  <span class="dashboard-activity-item__time">{{ activity.timeLabel }}</span>
                  <span class="dashboard-activity-item__tag" :data-tone="activity.tone">{{ activity.tag }}</span>
                  <p>{{ activity.summary }}</p>
                </article>
              </div>
            </section>

            <p v-if="dashboardRefreshError" class="module-board__error">{{ dashboardRefreshError }}</p>
            <p class="module-board__detail">{{ dashboardDetailText }}</p>
          </template>

          <template v-else>
            <header class="module-board__header" @mousedown.left="handleRegionMouseDown">
              <h2>{{ getModuleTitle(activeSection) }}</h2>
              <p>{{ missionStatement }}</p>
            </header>

            <div v-if="activeSection === 'tasks'" class="module-board__metrics">
              <article>
                <span>任务总数</span>
                <strong>{{ taskSummary.total }}</strong>
              </article>
              <article>
                <span>待办</span>
                <strong>{{ taskSummary.todo }}</strong>
              </article>
              <article>
                <span>进行中</span>
                <strong>{{ taskSummary.doing }}</strong>
              </article>
              <article>
                <span>回顾</span>
                <strong>{{ taskSummary.review }}</strong>
              </article>
              <article>
                <span>已完成</span>
                <strong>{{ taskSummary.done }}</strong>
              </article>
              <article>
                <span>已取消</span>
                <strong>{{ taskSummary.cancelled }}</strong>
              </article>
            </div>

            <template v-if="activeSection === 'recruitment'">
              <section class="module-surface recruitment-surface">
                <div class="module-surface__toolbar">
                  <input
                    v-model="recruitmentKeyword"
                    class="module-surface__search"
                    type="search"
                    placeholder="搜索角色 / 分组 / 领域"
                  />
                  <span class="module-surface__meta">显示 {{ recruitmentVisibleCount }} / {{ recruitmentTotalCount }} 个角色</span>
                </div>
                <div v-if="filteredRecruitmentDivisions.length === 0" class="module-empty">未找到匹配角色，请调整搜索词。</div>
                <div v-else class="recruitment-division-list">
                  <section v-for="division in filteredRecruitmentDivisions" :key="division.id" class="recruitment-division">
                    <header class="recruitment-division__header">
                      <strong>{{ division.titleZh }}</strong>
                      <small>{{ division.count }} 个</small>
                    </header>
                    <div class="recruitment-role-grid">
                      <article
                        v-for="role in division.groups.flatMap((group) => group.roles)"
                        :key="role.id"
                        class="recruitment-role-card"
                      >
                        <div>
                          <strong>{{ role.nameZh }}</strong>
                          <p>{{ role.workflowZh }}</p>
                          <small>{{ role.sourcePath }}</small>
                        </div>
                        <div class="recruitment-role-card__actions">
                          <button
                            class="recruitment-role-card__action recruitment-role-card__action--secondary"
                            type="button"
                            @click="openRoleWorkflowEditor(role)"
                          >
                            查看编辑
                          </button>
                          <button class="recruitment-role-card__action" type="button" @click="handleRecruitRole(role)">招募</button>
                        </div>
                      </article>
                    </div>
                  </section>
                </div>
              </section>
              <p class="module-board__detail">角色库来源：`src/agent-data/zh/README.md`。</p>
            </template>

            <template v-else-if="activeSection === 'skills'">
              <section class="module-surface skill-market-surface">
                <div class="skill-market-category-row" role="tablist" aria-label="技能分类">
                  <button
                    v-for="category in skillMarketCategories"
                    :key="`skill-market-${category.id}`"
                    class="skill-market-category-chip"
                    :class="{ active: activeSkillMarketCategory === category.id }"
                    type="button"
                    @click="handleSkillMarketCategorySwitch(category.id)"
                  >
                    <strong>{{ category.label }}</strong>
                    <small>{{ category.hint }}</small>
                  </button>
                </div>
                <div class="module-surface__toolbar module-surface__toolbar--skills">
                  <input
                    v-model="skillMarketSearch"
                    class="module-surface__search"
                    type="search"
                    placeholder="搜索技能名称、描述或标签"
                    @keydown.enter.prevent="handleSkillMarketSearchSubmit"
                  />
                  <select
                    class="module-surface__select"
                    :value="skillMarketSortBy"
                    @change="handleSkillMarketSortChange(($event.target as HTMLSelectElement).value as SkillMarketSortBy)"
                  >
                    <option value="score">综合评分</option>
                    <option value="downloads">下载量</option>
                    <option value="stars">收藏数</option>
                  </select>
                  <button class="module-surface__button" type="button" :disabled="skillMarketLoading" @click="handleSkillMarketSearchSubmit">
                    {{ skillMarketLoading ? "加载中..." : "搜索" }}
                  </button>
                </div>
                <p class="module-surface__hint">{{ skillMarketSummaryText }}</p>
                <p v-if="skillMarketActionNotice" class="module-surface__hint module-surface__hint--notice">{{ skillMarketActionNotice }}</p>

                <div v-if="skillMarketLoading" class="module-empty">正在加载技能市场数据...</div>
                <div v-else-if="skillMarketError" class="module-empty">{{ skillMarketError }}</div>
                <div v-else-if="pagedSkillMarketSkills.length === 0" class="module-empty">没有匹配的技能，请调整关键词或分类。</div>
                <div v-else class="skill-market-grid">
                  <article v-for="skill in pagedSkillMarketSkills" :key="skill.slug || skill.name" class="skill-market-card-v2">
                    <div class="skill-market-card-v2__header">
                      <div class="skill-market-card-v2__avatar">{{ getSkillMarketInitial(skill.name) }}</div>
                      <div class="skill-market-card-v2__title">
                        <strong>{{ skill.name }}</strong>
                        <p>{{ getSkillMarketDescription(skill) }}</p>
                      </div>
                    </div>
                    <div class="skill-market-card-v2__meta">
                      <span>↓ {{ formatSkillMarketCount(skill.downloads) }}</span>
                      <span>☆ {{ formatSkillMarketCount(skill.stars) }}</span>
                      <span>{{ formatSkillMarketVersion(skill.version) }}</span>
                    </div>
                    <div class="skill-market-card-v2__tags">
                      <span>{{ getSkillMarketCategoryLabel(skill.category) }}</span>
                      <span v-if="skill.ownerName">@{{ skill.ownerName }}</span>
                    </div>
                    <div class="skill-market-card-v2__actions">
                      <button class="skill-market-card-v2__action skill-market-card-v2__action--ghost" type="button" @click="openSkillMarketDetailModal(skill)">
                        查看详情
                      </button>
                      <button
                        class="skill-market-card-v2__action"
                        type="button"
                        :disabled="!canInstallSkillMarketSkill(skill)"
                        @click="installSkillMarketSkill(skill)"
                      >
                        安装技能
                      </button>
                    </div>
                  </article>
                </div>

                <div v-if="!skillMarketLoading && !skillMarketError && skillMarketCurrentTotalPages > 1" class="skill-market-pagination">
                  <button class="skill-market-pagination__button" type="button" :disabled="!skillMarketCanPrevPage" @click="goPrevSkillMarketPage">
                    上一页
                  </button>
                  <button
                    v-for="page in skillMarketPageNumbers"
                    :key="`skill-page-${page}`"
                    class="skill-market-pagination__page"
                    :class="{ active: page === skillMarketPage }"
                    type="button"
                    @click="goToSkillMarketPage(page)"
                  >
                    {{ page }}
                  </button>
                  <button class="skill-market-pagination__button" type="button" :disabled="!skillMarketCanNextPage" @click="goNextSkillMarketPage">
                    下一页
                  </button>
                </div>
              </section>
              <p class="module-board__detail">数据来源：技能市场 API（可通过 Runtime fallback）。</p>
            </template>

            <template v-else-if="activeSection === 'tasks'">
              <section class="module-surface task-module-surface">
                <p v-if="taskModuleNotice" class="module-surface__hint module-surface__hint--notice">{{ taskModuleNotice }}</p>
                <p v-if="taskModuleError" class="module-surface__hint module-surface__hint--error">{{ taskModuleError }}</p>

                <template v-if="taskModuleView === 'projects'">
                  <div class="task-project-toolbar">
                    <input
                      v-model="taskProjectInput"
                      class="module-surface__search"
                      type="text"
                      placeholder="输入项目名称，例如：发布计划"
                      @keydown.enter.prevent="handleCreateTaskProject"
                    />
                    <button class="module-surface__button" type="button" @click="handleCreateTaskProject">添加项目</button>
                  </div>

                  <div class="task-project-grid">
                    <button
                      v-for="project in taskProjectCards"
                      :key="project.name"
                      type="button"
                      class="task-project-card"
                      :class="{ 'task-project-card--default': project.isDefault }"
                      @click="openTaskProjectBoard(project.name)"
                    >
                      <div class="task-project-card__head">
                        <strong>{{ project.name }}</strong>
                        <span>{{ project.count }} 项</span>
                      </div>
                      <p>
                        进行中 {{ project.activeCount }} · 回顾 {{ project.reviewCount }} · 完成 {{ project.doneCount }}
                      </p>
                      <small>
                        {{ project.updatedAt ? `最近更新 ${formatDateTime(project.updatedAt)}` : "暂无任务，点击进入看板开始创建。" }}
                      </small>
                    </button>
                  </div>
                </template>

                <template v-else>
                  <div class="task-board-topbar">
                    <button class="task-board-back" type="button" @click="openTaskProjectsHome">← 返回项目</button>
                    <div>
                      <strong>{{ activeTaskProjectName }}</strong>
                      <p>{{ activeProjectTaskItems.length }} 项任务 · 参照 vibe-kanban 五列流转</p>
                    </div>
                  </div>

                  <div class="task-board-creator">
                    <input
                      v-model="taskDraftTitle"
                      type="text"
                      class="task-board-creator__input task-board-creator__input--title"
                      placeholder="新增任务标题"
                      @keydown.enter.prevent="handleCreateTaskInActiveProject"
                    />
                    <input v-model="taskDraftSummary" type="text" class="task-board-creator__input" placeholder="任务摘要（可选）" />
                    <input v-model="taskDraftOwner" type="text" class="task-board-creator__input" placeholder="负责人（默认 Commander）" />
                    <select v-model="taskDraftPriority" class="task-board-creator__select">
                      <option value="p0">P0 紧急</option>
                      <option value="p1">P1 常规</option>
                      <option value="p2">P2 低优先</option>
                    </select>
                    <button class="task-board-creator__button" type="button" @click="handleCreateTaskInActiveProject">添加任务</button>
                  </div>

                  <div class="task-board-columns">
                    <section
                      v-for="column in activeProjectTaskColumns"
                      :key="column.id"
                      class="task-board-column"
                      :class="{ 'is-drag-over': taskDragOverStatus === column.id }"
                      @dragover.prevent="handleTaskColumnDragOver(column.id)"
                      @drop.prevent="handleTaskColumnDrop(column.id)"
                    >
                      <header class="task-board-column__header">
                        <div>
                          <strong>{{ column.label }}</strong>
                          <span>{{ column.subtitle }}</span>
                        </div>
                        <em>{{ column.tasks.length }}</em>
                      </header>

                      <div v-if="column.tasks.length === 0" class="task-board-column__empty">{{ column.emptyText }}</div>
                      <div v-else class="task-board-column__list">
                        <article
                          v-for="task in column.tasks"
                          :key="task.id"
                          class="task-board-card"
                          :data-status="task.status"
                          :data-priority="task.priority"
                          draggable="true"
                          @dragstart="handleTaskDragStart(task.id)"
                          @dragend="handleTaskDragEnd"
                        >
                          <div class="task-board-card__head">
                            <strong>{{ task.title }}</strong>
                            <span>{{ getTaskPriorityText(task.priority) }}</span>
                          </div>
                          <p>{{ task.summary }}</p>
                          <small>{{ task.owner }} · 截止 {{ formatDateTime(task.dueAt) }} · 更新 {{ formatDateTime(task.updatedAt) }}</small>
                          <div class="task-board-card__actions">
                            <button type="button" :disabled="!getTaskPrevStatus(task.status)" @click="moveTaskToPrevStatus(task)">上一步</button>
                            <button type="button" :disabled="!getTaskNextStatus(task.status)" @click="moveTaskToNextStatus(task)">下一步</button>
                          </div>
                        </article>
                      </div>
                    </section>
                  </div>
                </template>
              </section>
              <p class="module-board__detail">任务来源：localStorage（默认项目 {{ DEFAULT_TASK_PROJECT_NAME }} 收纳未归属项目任务）。</p>
            </template>

            <template v-else>
              <p class="module-board__note">该模块将在下一步接入完整业务面板。当前优先完成聊天壳层与 Agent 会话接入。</p>
              <p class="module-board__detail">{{ staffSourceDetail }}</p>
            </template>
          </template>
        </main>
      </div>
    </div>

    <div v-if="activeRoleWorkflowBase" class="role-workflow-detail-backdrop" @click.self="closeRoleWorkflowDetail">
      <section class="role-workflow-detail-modal" role="dialog" aria-modal="true" aria-label="角色详情">
        <header class="role-workflow-detail-modal__header">
          <div>
            <strong>角色详情</strong>
            <p>
              {{ activeRoleWorkflowBase.divisionTitleZh }}
              <span v-if="activeRoleWorkflowBase.groupTitleZh"> / {{ activeRoleWorkflowBase.groupTitleZh }}</span>
            </p>
          </div>
          <button
            class="role-workflow-detail-modal__close"
            type="button"
            aria-label="关闭"
            :disabled="isRoleWorkflowInstalling"
            @click="closeRoleWorkflowDetail"
          >
            ×
          </button>
        </header>

        <div class="role-workflow-detail-modal__body">
          <p
            v-if="roleWorkflowDetailNotice"
            class="role-workflow-detail-modal__notice"
            :class="`role-workflow-detail-modal__notice--${roleWorkflowDetailNotice.tone}`"
          >
            {{ roleWorkflowDetailNotice.text }}
          </p>

          <label class="role-workflow-detail-modal__field role-workflow-detail-modal__field--name">
            <span>角色中文名称</span>
            <input
              v-model="roleWorkflowNameZhDraft"
              class="role-workflow-detail-modal__name-input"
              type="text"
              placeholder="请输入角色中文名称"
              :disabled="roleWorkflowDetailLoading || isRoleWorkflowInstalling"
            />
          </label>

          <label class="role-workflow-detail-modal__field">
            <span>详情内容（Markdown，可编辑）</span>
            <textarea
              v-model="roleWorkflowDetailDraft.contentZh"
              class="role-workflow-detail-modal__editor"
              rows="18"
              placeholder="在这里编辑角色详情 Markdown"
              :disabled="roleWorkflowDetailLoading || isRoleWorkflowInstalling"
            />
          </label>

          <section class="role-workflow-detail-modal__versions">
            <header class="role-workflow-detail-modal__versions-header">
              <strong>已保存版本（最多 3 个）</strong>
            </header>
            <p v-if="roleWorkflowDetailSavedVersions.length === 0" class="role-workflow-detail-modal__versions-empty">
              暂无已保存版本，点击“保存”后会自动保留历史。
            </p>
            <ul v-else class="role-workflow-detail-modal__versions-list">
              <li v-for="version in roleWorkflowDetailSavedVersions" :key="version.id" class="role-workflow-detail-modal__version-item">
                <span class="role-workflow-detail-modal__version-time">{{ formatDateTime(version.savedAt) }}</span>
                <div class="role-workflow-detail-modal__version-actions">
                  <button type="button" :disabled="roleWorkflowDetailLoading || isRoleWorkflowInstalling" @click="applyRoleWorkflowSavedVersion(version.id)">
                    载入
                  </button>
                  <button type="button" :disabled="roleWorkflowDetailLoading || isRoleWorkflowInstalling" @click="deleteRoleWorkflowSavedVersion(version.id)">
                    删除
                  </button>
                </div>
              </li>
            </ul>
          </section>
        </div>

        <footer class="role-workflow-detail-modal__actions">
          <button type="button" :disabled="!isRoleWorkflowDraftChanged || roleWorkflowDetailLoading || isRoleWorkflowInstalling" @click="restoreRoleWorkflowOriginalContent">
            恢复原始内容
          </button>
          <button type="button" :disabled="!canSaveRoleWorkflowDraft || isRoleWorkflowInstalling" @click="saveRoleWorkflowDetail">
            {{ roleWorkflowDetailLoading ? "加载中..." : "保存" }}
          </button>
          <button type="button" :disabled="roleWorkflowDetailLoading || isRoleWorkflowInstalling" @click="installRoleWorkflowRole">
            {{ isRoleWorkflowInstalling ? "安装中..." : "安装角色" }}
          </button>
        </footer>
      </section>
    </div>

    <div v-if="activeSkillMarketDetail" class="skill-market-detail-backdrop" @click.self="closeSkillMarketDetailModal">
      <section class="skill-market-detail-modal" role="dialog" aria-modal="true" aria-label="技能详情">
        <header class="skill-market-detail-modal__header">
          <div class="skill-market-detail-modal__identity">
            <div class="skill-market-detail-modal__avatar">{{ getSkillMarketInitial(activeSkillMarketDetail.name) }}</div>
            <div>
              <strong>{{ activeSkillMarketDetail.name }}</strong>
              <p>{{ activeSkillMarketDetail.slug || "skill" }}</p>
            </div>
          </div>
          <button class="skill-market-detail-modal__close" type="button" aria-label="关闭" @click="closeSkillMarketDetailModal">×</button>
        </header>

        <div class="skill-market-detail-modal__body">
          <div class="skill-market-detail-modal__chips">
            <span>{{ formatSkillMarketVersion(activeSkillMarketDetail.version) }}</span>
            <span>{{ getSkillMarketCategoryLabel(activeSkillMarketDetail.category) }}</span>
            <span v-if="activeSkillMarketDetail.ownerName">@{{ activeSkillMarketDetail.ownerName }}</span>
          </div>
          <p class="skill-market-detail-modal__description">{{ getSkillMarketDescription(activeSkillMarketDetail) }}</p>
          <div class="skill-market-detail-modal__stats">
            <article>
              <span>下载量</span>
              <strong>{{ formatSkillMarketCount(activeSkillMarketDetail.downloads) }}</strong>
            </article>
            <article>
              <span>收藏</span>
              <strong>{{ formatSkillMarketCount(activeSkillMarketDetail.stars) }}</strong>
            </article>
            <article>
              <span>安装量</span>
              <strong>{{ formatSkillMarketCount(activeSkillMarketDetail.installs) }}</strong>
            </article>
          </div>
        </div>

        <footer class="skill-market-detail-modal__actions">
          <button
            class="skill-market-detail-modal__action skill-market-detail-modal__action--primary"
            type="button"
            :disabled="!canInstallSkillMarketSkill(activeSkillMarketDetail)"
            @click="installSkillMarketSkill(activeSkillMarketDetail)"
          >
            安装技能
          </button>
          <button
            class="skill-market-detail-modal__action"
            type="button"
            :disabled="!activeSkillMarketDetail.homepage"
            @click="openSkillHomepage(activeSkillMarketDetail)"
          >
            访问 SkillHub
          </button>
        </footer>
      </section>
    </div>

    <div v-if="isProxyConfigModalOpen" class="related-resource-modal-backdrop" @click.self="closeProxyConfigModal">
      <section class="related-resource-modal proxy-config-modal" role="dialog" aria-modal="true" aria-label="代理配置中心">
        <header class="related-resource-modal__header">
          <div>
            <strong>代理配置中心</strong>
            <p>{{ proxyConfigModalSubtitle }}</p>
          </div>
          <div class="related-resource-modal__actions">
            <button
              class="related-resource-modal__refresh"
              type="button"
              :disabled="proxyConfigLoading || proxyConfigSaving"
              @click="loadProxyConfigSnapshot(proxyConfigSelectedProviderId)"
            >
              刷新
            </button>
            <button class="related-resource-modal__close" type="button" aria-label="关闭" @click="closeProxyConfigModal">×</button>
          </div>
        </header>

        <div class="related-resource-modal__body">
          <p v-if="proxyConfigNotice" class="related-resource-modal__notice">{{ proxyConfigNotice }}</p>
          <p v-if="proxyConfigError" class="related-resource-modal__error">{{ proxyConfigError }}</p>

          <div v-if="proxyConfigLoading" class="related-resource-modal__empty">正在读取代理配置...</div>
          <template v-else>
            <div class="proxy-config-layout">
              <aside class="proxy-config-nav-pane">
                <div class="proxy-config-nav-pane__toolbar">
                  <button class="related-resource-modal__refresh" type="button" :disabled="proxyConfigSaving" @click="handleProxyConfigCreate">
                    新增平台
                  </button>
                </div>
                <div class="proxy-config-nav-list">
                  <button
                    v-for="platform in proxyConfigPlatforms"
                    :key="`proxy-platform-${platform.providerId}`"
                    class="proxy-config-nav-item"
                    :class="{ active: equalsIgnoreCase(platform.providerId, proxyConfigSelectedProviderId) }"
                    type="button"
                    @click="handleProxyConfigSelect(platform.providerId)"
                  >
                    <strong>{{ platform.name || platform.providerId }}</strong>
                    <p>{{ platform.providerId }}</p>
                  </button>
                  <div v-if="proxyConfigPlatforms.length === 0" class="related-resource-modal__empty related-resource-modal__empty--small">
                    暂无已配置平台，请点击“新增平台”创建。
                  </div>
                </div>
              </aside>

              <section class="proxy-config-editor-pane">
                <form v-if="proxyConfigDraft" class="related-model-form" @submit.prevent="handleProxyConfigSave">
                  <p class="related-model-form__meta">
                    当前选择：{{ proxyConfigSelectedPlatform?.name || "新平台" }} · {{ proxyConfigSelectedPlatform?.providerId || "custom" }}
                  </p>
                  <label class="related-model-form__field">
                    <span>providerId</span>
                    <input
                      v-model="proxyConfigDraft.providerId"
                      class="related-model-form__input"
                      type="text"
                      placeholder="openai / deepseek / custom"
                      :disabled="proxyConfigSaving"
                    />
                  </label>
                  <label class="related-model-form__field">
                    <span>基础 URL</span>
                    <input
                      v-model="proxyConfigDraft.baseUrl"
                      class="related-model-form__input"
                      type="text"
                      placeholder="https://api.example.com/v1"
                      :disabled="proxyConfigSaving"
                    />
                  </label>
                  <label class="related-model-form__field">
                    <span>API 密钥</span>
                    <input
                      v-model="proxyConfigDraft.apiKey"
                      class="related-model-form__input"
                      type="password"
                      placeholder="sk-..."
                      :disabled="proxyConfigSaving"
                    />
                  </label>
                  <div class="related-model-form__actions">
                    <small>仅更新 provider 连接信息，不会改动模型选择。</small>
                    <button class="related-resource-modal__refresh" type="submit" :disabled="proxyConfigSaving">
                      {{ proxyConfigSaving ? "保存中..." : "保存配置" }}
                    </button>
                  </div>
                </form>
              </section>
            </div>
          </template>
        </div>
      </section>
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
            <div v-if="relatedMemoryItems.length > 0" class="related-memory-layout">
              <aside class="related-memory-nav-pane">
                <div class="related-memory-nav-pane__toolbar">
                  <input v-model="relatedMemorySearch" class="related-resource-filter-input" type="text" placeholder="筛选标题、路径或摘要" />
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
                  <div class="related-memory-editor-pane__header">
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
                    <button
                      class="related-resource-modal__refresh"
                      type="button"
                      :disabled="relatedResourceModalSaving || relatedResourceModalLoading"
                      @click="handleRelatedMemorySave"
                    >
                      保存
                    </button>
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

          <template v-else-if="relatedResourceModalTarget === 'model'">
            <div v-if="relatedModelDraft" class="related-model-form">
              <label class="related-model-form__field">
                <span>基础 URL</span>
                <input
                  v-model="relatedModelDraft.baseUrl"
                  class="related-model-form__input"
                  type="text"
                  placeholder="https://api.example.com/v1"
                  :disabled="relatedResourceModalSaving"
                />
              </label>
              <label class="related-model-form__field">
                <span>模型 ID</span>
                <input
                  v-model="relatedModelDraft.model"
                  class="related-model-form__input"
                  type="text"
                  placeholder="gpt-4o-mini"
                  :disabled="relatedResourceModalSaving"
                />
              </label>
              <label class="related-model-form__field">
                <span>协议</span>
                <select v-model="relatedModelDraft.apiKind" class="related-model-form__select" :disabled="relatedResourceModalSaving">
                  <option v-for="option in relatedModelProtocolOptions" :key="`provider-protocol-${option.id}`" :value="option.id">
                    {{ option.label }}
                  </option>
                </select>
              </label>
              <label class="related-model-form__field">
                <span>API 密钥</span>
                <input
                  v-model="relatedModelDraft.apiKey"
                  class="related-model-form__input"
                  type="password"
                  placeholder="sk-..."
                  :disabled="relatedResourceModalSaving"
                />
              </label>
              <div class="related-model-form__actions">
                <small>当前协议：{{ getProviderApiKindLabel(relatedModelDraft.apiKind) }}</small>
                <button
                  class="related-resource-modal__refresh"
                  type="button"
                  :disabled="relatedResourceModalSaving || relatedResourceModalLoading"
                  @click="handleRelatedModelSave"
                >
                  保存
                </button>
              </div>
            </div>
            <div v-else class="related-resource-modal__empty">未找到可编辑的模型平台配置。</div>
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

        <div class="utility-modal__body" :class="{ 'utility-modal__body--logs': utilityModalType === 'logs' }">
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
            <p class="utility-modal__detail">
              {{ chatRuntimeLogs?.detail || "展示 OpenClaw 运行日志。" }} · 当前员工
              {{ utilityLogTab === "runtime" ? runtimeCategoryLogItems.length : runtimeLogItems.length }} / {{ runtimeLogItems.length }} 条
            </p>

            <div class="utility-log-tabs" role="tablist" aria-label="日志视图切换">
              <button
                type="button"
                class="utility-log-tab"
                :class="{ 'is-active': utilityLogTab === 'runtime' }"
                @click="handleUtilityLogTabChange('runtime')"
              >
                运行日志
              </button>
              <button
                type="button"
                class="utility-log-tab"
                :class="{ 'is-active': utilityLogTab === 'errorAnalysis' }"
                @click="handleUtilityLogTabChange('errorAnalysis')"
              >
                错误日志分析
              </button>
            </div>

            <template v-if="utilityLogTab === 'runtime'">
              <div class="utility-log-categories" role="tablist" aria-label="运行日志分类">
                <button
                  v-for="category in utilityRuntimeCategories"
                  :key="category.id"
                  type="button"
                  class="utility-log-category"
                  :class="{ 'is-active': utilityRuntimeCategory === category.id }"
                  @click="handleUtilityRuntimeCategoryChange(category.id)"
                >
                  {{ category.label }}
                </button>
              </div>

              <div v-if="runtimeCategoryLogItems.length === 0" class="utility-modal__empty">当前分类暂无运行日志。</div>
              <div v-else class="utility-log-layout">
                <div class="utility-log-list">
                  <button
                    v-for="log in runtimeCategoryLogItems"
                    :key="log.id"
                    type="button"
                    class="utility-log-item"
                    :class="{ 'is-active': selectedRuntimeLog?.id === log.id }"
                    @click="handleUtilityLogSelect(log)"
                  >
                    <div class="utility-log-item__head">
                      <div class="utility-log-item__meta">
                        <strong>{{ log.method }}</strong>
                        <span class="utility-log-kind" :data-kind="getRuntimeLogCategory(log)">{{ getRuntimeLogCategoryLabel(log) }}</span>
                      </div>
                      <span class="utility-log-status" :data-tone="getLogStatusTone(log.responseStatus)">{{ log.responseStatus }}</span>
                    </div>
                    <p>{{ getLogRequestUrl(log) }}</p>
                    <small>{{ formatDateTime(log.createdAt) }} · 耗时 {{ formatDurationLabel(log.duration) }}</small>
                  </button>
                </div>

                <section v-if="selectedRuntimeLog && activeRuntimeLogDetailSection" class="utility-log-detail">
                  <div class="utility-log-detail__head">
                    <header class="utility-log-detail__header">
                      <div>
                        <strong>{{ selectedRuntimeLog.method }} {{ selectedRuntimeLog.path || selectedRuntimeLog.endpoint || "/" }}</strong>
                        <p>{{ selectedRuntimeLog.platformName }} · {{ formatDateTime(selectedRuntimeLog.createdAt) }}</p>
                      </div>
                      <span class="utility-log-status" :data-tone="getLogStatusTone(selectedRuntimeLog.responseStatus)">{{
                        selectedRuntimeLog.responseStatus
                      }}</span>
                    </header>
                    <div class="utility-log-detail__stats">
                      <span>耗时 {{ formatDurationLabel(selectedRuntimeLog.duration) }}</span>
                      <span v-if="typeof selectedRuntimeLog.firstTokenTime === 'number'">
                        首 Token {{ formatDurationLabel(selectedRuntimeLog.firstTokenTime) }}
                      </span>
                      <span v-if="typeof selectedRuntimeLog.totalTokens === 'number'">Token {{ selectedRuntimeLog.totalTokens }}</span>
                    </div>
                    <div class="utility-log-detail-tabs" role="tablist" aria-label="日志详情分栏">
                      <button
                        v-for="section in runtimeLogDetailSections"
                        :key="section.id"
                        type="button"
                        class="utility-log-detail-tab"
                        :class="{ 'is-active': activeRuntimeLogDetailSection.id === section.id }"
                        @click="handleUtilityLogDetailTabSelect(section.id)"
                      >
                        {{ section.label }}
                      </button>
                    </div>
                  </div>
                  <div class="utility-log-detail__content-wrap">
                    <button
                      type="button"
                      class="utility-log-copy"
                      title="复制当前内容"
                      aria-label="复制当前内容"
                      @click="handleCopyRuntimeLogContent"
                    >
                      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M16 1H6a2 2 0 0 0-2 2v12h2V3h10zm3 4H10a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h9a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2m0 16H10V7h9z" /></svg>
                    </button>
                    <pre class="utility-log-detail__content" tabindex="0">{{ activeRuntimeLogDetailSection.text }}</pre>
                  </div>
                </section>
              </div>
            </template>

            <template v-else>
              <div v-if="errorAnalysisSummaries.length === 0" class="utility-modal__empty">当前员工暂无错误日志。</div>
              <div v-else class="utility-log-layout">
                <div class="utility-error-list">
                  <button
                    v-for="summary in errorAnalysisSummaries"
                    :key="summary.key"
                    type="button"
                    class="utility-error-item"
                    :class="{ 'is-active': selectedErrorSummary?.key === summary.key }"
                    @click="handleUtilityErrorSummarySelect(summary.key)"
                  >
                    <div class="utility-error-item__head">
                      <strong>{{ summary.title }}</strong>
                      <span>{{ summary.count }} 次</span>
                    </div>
                    <p>{{ summary.latestLog.method }} {{ summary.latestLog.path || summary.latestLog.endpoint || "/" }}</p>
                    <small>最近 {{ formatDateTime(summary.latestAt) }}</small>
                  </button>
                </div>

                <section v-if="selectedRuntimeLog && activeRuntimeLogDetailSection" class="utility-log-detail">
                  <div class="utility-log-detail__head">
                    <header class="utility-log-detail__header">
                      <div>
                        <strong>错误详情</strong>
                        <p>{{ selectedRuntimeLog.method }} {{ selectedRuntimeLog.path || selectedRuntimeLog.endpoint || "/" }}</p>
                      </div>
                      <span class="utility-log-status" :data-tone="getLogStatusTone(selectedRuntimeLog.responseStatus)">{{
                        selectedRuntimeLog.responseStatus
                      }}</span>
                    </header>
                    <div class="utility-log-detail-tabs" role="tablist" aria-label="错误详情分栏">
                      <button
                        v-for="section in runtimeLogDetailSections"
                        :key="section.id"
                        type="button"
                        class="utility-log-detail-tab"
                        :class="{ 'is-active': activeRuntimeLogDetailSection.id === section.id }"
                        @click="handleUtilityLogDetailTabSelect(section.id)"
                      >
                        {{ section.label }}
                      </button>
                    </div>
                  </div>
                  <div class="utility-log-detail__content-wrap">
                    <button
                      type="button"
                      class="utility-log-copy"
                      title="复制当前内容"
                      aria-label="复制当前内容"
                      @click="handleCopyRuntimeLogContent"
                    >
                      <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M16 1H6a2 2 0 0 0-2 2v12h2V3h10zm3 4H10a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h9a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2m0 16H10V7h9z" /></svg>
                    </button>
                    <pre class="utility-log-detail__content" tabindex="0">{{ activeRuntimeLogDetailSection.text }}</pre>
                  </div>
                </section>
              </div>
            </template>
          </template>
        </div>
      </section>
    </div>

    <div v-if="isSidebarSettingsModalOpen" class="sidebar-settings-modal-backdrop" @click.self="closeSidebarSettingsModal">
      <section class="sidebar-settings-modal" role="dialog" aria-modal="true" aria-label="设置">
        <header class="sidebar-settings-modal__header">
          <strong>设置</strong>
          <button class="sidebar-settings-modal__close" type="button" aria-label="关闭设置" @click="closeSidebarSettingsModal">×</button>
        </header>

        <div class="sidebar-settings-modal__body">
          <aside class="sidebar-settings-nav">
            <button
              v-for="group in sidebarSettingsMenuGroups"
              :key="`sidebar-settings-group-${group.id}`"
              class="sidebar-settings-nav__item"
              :class="{ 'is-active': sidebarSettingsActiveGroup === group.id }"
              type="button"
              @click="handleSidebarSettingsGroupChange(group.id)"
            >
              {{ group.label }}
            </button>
          </aside>

          <section class="sidebar-settings-panel">
            <p v-if="sidebarSettingsNotice" class="sidebar-settings-panel__notice">{{ sidebarSettingsNotice }}</p>
            <p v-if="sidebarSettingsError" class="sidebar-settings-panel__error">{{ sidebarSettingsError }}</p>

            <template v-if="sidebarSettingsActiveGroup === 'general'">
              <article class="sidebar-settings-card">
                <div>
                  <h4>外观</h4>
                  <p>选择当前应用的显示风格。</p>
                </div>
                <select
                  class="sidebar-settings-select"
                  :value="sidebarSettingsAppearance"
                  @change="setSidebarSettingsAppearance(($event.target as HTMLSelectElement).value as SidebarSettingsAppearance)"
                >
                  <option v-for="option in sidebarSettingsAppearanceOptions" :key="`appearance-${option.id}`" :value="option.id">
                    {{ option.label }}
                  </option>
                </select>
              </article>

              <article class="sidebar-settings-card">
                <div>
                  <h4>语言</h4>
                  <p>设置应用界面语言。</p>
                </div>
                <select
                  class="sidebar-settings-select"
                  :value="sidebarSettingsLanguage"
                  @change="setSidebarSettingsLanguage(($event.target as HTMLSelectElement).value as SidebarSettingsLanguage)"
                >
                  <option v-for="option in sidebarSettingsLanguageOptions" :key="`language-${option.id}`" :value="option.id">
                    {{ option.label }}
                  </option>
                </select>
              </article>

              <article class="sidebar-settings-card sidebar-settings-card--column">
                <div>
                  <h4>快捷键</h4>
                  <p>当前可用的全局快捷键如下。</p>
                </div>
                <div class="sidebar-settings-shortcut-list">
                  <div v-for="shortcut in sidebarSettingsShortcutItems" :key="shortcut.id" class="sidebar-settings-shortcut-item">
                    <div>
                      <strong>{{ shortcut.label }}</strong>
                      <p>{{ shortcut.note }}</p>
                    </div>
                    <code>{{ shortcut.value }}</code>
                  </div>
                </div>
              </article>

              <article class="sidebar-settings-card">
                <div>
                  <h4>开机自动启动</h4>
                  <p>{{ sidebarSettingsLaunchOnLoginSupported ? "系统登录后自动启动 ClawPet。" : "当前环境暂不支持自动启动设置。" }}</p>
                </div>
                <button
                  class="sidebar-settings-toggle"
                  :class="{ 'is-on': sidebarSettingsLaunchOnLoginEnabled }"
                  type="button"
                  aria-label="切换开机自动启动"
                  :aria-pressed="sidebarSettingsLaunchOnLoginEnabled"
                  :disabled="sidebarSettingsLaunchOnLoginLoading || !sidebarSettingsLaunchOnLoginSupported"
                  @click="handleSidebarSettingsLaunchOnLoginToggle"
                >
                  <i />
                </button>
              </article>
            </template>

            <template v-else>
              <article class="sidebar-settings-card">
                <div>
                  <h4>版本号</h4>
                  <p>当前应用版本信息。</p>
                </div>
                <div class="sidebar-settings-version">
                  <strong>{{ sidebarSettingsAppVersion }}</strong>
                  <button type="button" class="sidebar-settings-text-button" @click="loadSidebarSettingsAppVersion">刷新</button>
                </div>
              </article>

              <article class="sidebar-settings-card sidebar-settings-card--column">
                <div>
                  <h4>使用技巧</h4>
                  <p>三个高频提效建议。</p>
                </div>
                <ul class="sidebar-settings-tip-list">
                  <li v-for="(tip, index) in sidebarSettingsTips" :key="`tip-${index}`">{{ tip }}</li>
                </ul>
              </article>

              <article class="sidebar-settings-card sidebar-settings-card--column">
                <div>
                  <h4>意见反馈</h4>
                  <p>填写反馈后可一键复制，也可以先打开日志面板附带运行信息。</p>
                </div>
                <textarea
                  v-model="sidebarSettingsFeedbackDraft"
                  class="sidebar-settings-feedback"
                  rows="7"
                  placeholder="例如：在聊天页切换员工时，建议保留上次输入内容。"
                />
                <div class="sidebar-settings-feedback__actions">
                  <button type="button" class="sidebar-settings-text-button" @click="handleSidebarFeedbackCopy">复制反馈内容</button>
                  <button type="button" class="sidebar-settings-text-button" @click="openFeedbackLogsFromSidebarSettings">
                    打开日志面板
                  </button>
                </div>
              </article>
            </template>
          </section>
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

.sidebar-bottom-actions {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 4px;
}

.nav-item--secondary {
  margin-bottom: 0;
}

.nav-item--legacy {
  border: 1px solid #d9e5f7;
  background: #f4f8ff;
  color: #476792;
}

.nav-item--legacy .nav-item__icon {
  color: #476792;
}

.nav-item--legacy:hover {
  background: #ebf3ff;
  color: #2f5587;
}

.chat-list {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  border-right: 1px solid #dde2ea;
  background: #fdfdfd;
}

.chat-list__header {
  padding: 16px 14px 10px;
  border-bottom: 1px solid #e8ebf1;
  background: #fdfdfd;
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
  border: 1px solid #d9dee7;
  background: #ffffff;
  color: #90a0ba;
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
  border: 1px solid #d9dee7;
  border-radius: 8px;
  background: #ffffff;
  color: #8290a8;
  display: grid;
  place-items: center;
  cursor: pointer;
}

.search-add:hover {
  color: #3f6fd6;
  border-color: #bdcfee;
  background: #edf3ff;
}

.agent-pane-tabs {
  margin-top: 12px;
  padding: 4px;
  border-radius: 999px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px;
  border: 1px solid #d9e0ea;
  background: #e9edf3;
}

.agent-pane-tab {
  height: 32px;
  border: 0;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  color: #7f8ba3;
  background: transparent;
  cursor: pointer;
  transition: color 160ms ease, background 160ms ease, box-shadow 160ms ease;
}

.agent-pane-tab.active {
  color: #33445f;
  background: #ffffff;
  box-shadow: 0 2px 6px rgba(85, 104, 136, 0.18);
}

.chat-list__body {
  min-height: 0;
  overflow: auto;
  padding: 6px 10px 10px;
  background: transparent;
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
  border-color: #d7dde7;
  background: #ffffff;
}

.agent-item.active {
  border-color: #9eb3f7;
  background: #e9efff;
  color: #2f446f;
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

.agent-avatar__image {
  width: 100%;
  height: 100%;
  border-radius: inherit;
  object-fit: cover;
  display: block;
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
  background: #d9e4ff;
  color: #50658f;
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
  color: #647ba6;
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
  color: #7388b3;
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
  background: #7f98ea;
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
  background: #ffffff;
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
  background: #ffffff;
}

.chat-window__content--settings-open {
  grid-template-columns: minmax(0, 1fr) 360px;
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

.chat-settings-list__row--model dd {
  min-width: 0;
}

.chat-settings-model-trigger {
  width: 100%;
  border: 0;
  background: transparent;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  text-align: left;
  color: inherit;
  cursor: pointer;
  font: inherit;
}

.chat-settings-model-trigger__value {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-settings-model-trigger__hint {
  flex-shrink: 0;
  color: #6f86a9;
  font-size: 11px;
  font-weight: 600;
}

.chat-settings-model-trigger:hover .chat-settings-model-trigger__hint {
  color: #2f6fd6;
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
  max-width: min(76%, 640px);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.message-row--user {
  align-self: flex-end;
}

.message-row--assistant,
.message-row--system {
  align-self: flex-start;
}

.message-bubble {
  border-radius: 18px;
  padding: 12px 14px;
  font-size: 14px;
  line-height: 1.45;
  color: #1f2a44;
  background: #ffffff;
  border: 1px solid #e6edf8;
  box-shadow: 0 8px 16px rgba(61, 89, 130, 0.06);
  white-space: pre-wrap;
  word-break: break-word;
}

.message-row--user .message-bubble {
  color: #fff;
  border-color: transparent;
  background: linear-gradient(135deg, #5a9dff, #2f78e4);
}

.message-row--pending .message-bubble {
  color: #55698a;
  border-color: #dce7f8;
  background: #eef4ff;
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

.chat-window__messages {
  grid-column: 1;
  grid-row: 1;
  min-height: 0;
  overflow: auto;
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.chat-window__messages--empty {
  justify-content: center;
}

.chat-empty-state {
  margin: auto;
  width: min(560px, 100%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
  color: #354965;
}

.chat-empty-state__logo {
  width: 66px;
  height: 66px;
  border-radius: 20px;
  border: 1px solid #dbe7f8;
  background: #ffffff;
  color: #2d3f58;
  display: grid;
  place-items: center;
  box-shadow: 0 10px 18px rgba(61, 89, 130, 0.08);
}

.chat-empty-state__logo svg {
  width: 38px;
  height: 38px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.chat-empty-state h3 {
  margin: 6px 0 0;
  font-size: 40px;
  line-height: 1;
  letter-spacing: -0.03em;
  color: #202f45;
}

.chat-empty-state p {
  margin: 0;
  font-size: 13px;
  color: #7b8da9;
}

.chat-empty-state__actions {
  margin-top: 8px;
  width: 100%;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.chat-empty-action {
  width: 100%;
  min-height: 84px;
  border: 1px solid #dbe7f8;
  border-radius: 14px;
  background: #ffffff;
  padding: 12px;
  display: grid;
  grid-template-columns: 38px minmax(0, 1fr);
  gap: 0 10px;
  align-items: center;
  text-align: left;
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease, transform 140ms ease, box-shadow 140ms ease;
}

.chat-empty-action:hover {
  border-color: #8db5f3;
  background: #f5f9ff;
  box-shadow: 0 8px 16px rgba(61, 89, 130, 0.09);
  transform: translateY(-1px);
}

.chat-empty-action:disabled {
  cursor: not-allowed;
  opacity: 0.56;
}

.chat-empty-action__icon {
  width: 36px;
  height: 36px;
  border-radius: 11px;
  background: #e9f2ff;
  color: #3d6ca7;
  display: grid;
  place-items: center;
}

.chat-empty-action__icon svg {
  width: 18px;
  height: 18px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.chat-empty-action__content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.chat-empty-action__content strong {
  color: #253a57;
  font-size: 15px;
  line-height: 1.18;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chat-empty-action__content small {
  color: #7389aa;
  font-size: 11px;
  line-height: 1.3;
}

.chat-window__composer {
  padding: 12px 16px 16px;
  border-top: 1px solid #ededed;
  background: #ffffff;
}

.composer-panel {
  border: 1px solid #d6e1f1;
  border-radius: 18px;
  background: #ffffff;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  box-shadow: 0 8px 18px rgba(64, 89, 128, 0.08);
}

.composer-input-shell {
  position: relative;
  min-width: 0;
}

.chat-window__composer input {
  height: 44px;
  border: 0;
  border-radius: 12px;
  outline: 0;
  background: #f6f9ff;
  color: #22314d;
  padding: 0 48px 0 40px;
  font-size: 14px;
  width: 100%;
  transition: box-shadow 120ms ease, background 120ms ease;
}

.chat-window__composer input:focus {
  background: #ffffff;
  box-shadow: inset 0 0 0 1px #9ec1f7;
}

.chat-window__composer input:disabled {
  background: #eef3fb;
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

.composer-send {
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: 999px;
  display: grid;
  place-items: center;
  cursor: pointer;
  position: absolute;
  right: 6px;
  top: 50%;
  transform: translateY(-50%);
  color: #fff;
  background: linear-gradient(135deg, #5a9dff, #2f78e4);
}

.composer-send:disabled {
  cursor: not-allowed;
  background: #c8d6eb;
  color: #eef4ff;
  opacity: 0.55;
}

.composer-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 0 2px;
}

.composer-model-chip {
  height: 24px;
  border-radius: 999px;
  border: 1px solid #d5e1f4;
  background: #f7f9fe;
  color: #607796;
  padding: 0 9px;
  display: inline-flex;
  align-items: center;
  font-size: 11px;
  font-weight: 600;
  max-width: 65%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.composer-btn {
  height: 24px;
  border: 1px solid #d5e1f4;
  border-radius: 999px;
  background: #f3f7ff;
  color: #607796;
  padding: 0 10px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease, color 140ms ease;
}

.composer-btn:hover {
  border-color: #96bbe9;
  background: #eaf2ff;
  color: #2f5f98;
}

.composer-btn:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.module-board {
  grid-column: 2 / 4;
  padding: 22px;
  overflow: auto;
  background: #ffffff;
  color: #24344f;
}

.module-board--dashboard {
  --dashboard-surface: #ffffff;
  --dashboard-border: #d4e1f2;
  --dashboard-text: #1f3553;
  --dashboard-muted: #617da4;
  --dashboard-shadow: 0 16px 32px rgba(47, 74, 116, 0.09);
  background:
    radial-gradient(120% 72% at 0% -10%, rgba(69, 139, 233, 0.13) 0%, rgba(69, 139, 233, 0) 62%),
    radial-gradient(90% 58% at 100% -4%, rgba(41, 182, 126, 0.15) 0%, rgba(41, 182, 126, 0) 64%),
    linear-gradient(180deg, #f7faff 0%, #edf4ff 48%, #edf3ff 100%);
  color: var(--dashboard-text);
}

.module-board__header h2 {
  margin: 0;
  font-size: 26px;
  line-height: 1;
  letter-spacing: -0.01em;
}

.module-board__header {
  cursor: move;
}

.module-board__header--dashboard {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 15px;
  padding: 18px 20px;
  border: 1px solid rgba(195, 211, 233, 0.9);
  border-radius: 18px;
  background: linear-gradient(155deg, rgba(255, 255, 255, 0.98), rgba(245, 251, 255, 0.95));
  box-shadow: var(--dashboard-shadow);
}

.module-board--dashboard .module-board__header h2 {
  font-size: 31px;
  font-weight: 800;
  line-height: 1.08;
  letter-spacing: -0.02em;
}

.module-board__header p {
  margin: 8px 0 0;
  color: #7084a4;
  font-size: 14px;
}

.module-board--dashboard .module-board__header p {
  margin-top: 10px;
  color: var(--dashboard-muted);
  font-size: 13px;
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
  font-size: 22px;
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

.module-board--dashboard .module-board__detail {
  margin-top: 12px;
  border-radius: 12px;
  border: 1px solid rgba(194, 212, 236, 0.84);
  background: rgba(255, 255, 255, 0.82);
  color: #5f7da2;
  padding: 10px 12px;
}

.module-board__error {
  margin-top: 12px;
  color: #b54708;
  font-size: 13px;
}

.module-board--dashboard .module-board__error {
  margin-top: 12px;
  border-radius: 12px;
  border: 1px solid #f0d3a4;
  background: #fff8ec;
  color: #a6640f;
  padding: 10px 12px;
}

.dashboard-toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  flex-wrap: wrap;
}

.dashboard-health-badge {
  height: 34px;
  border-radius: 999px;
  padding: 0 13px;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.015em;
  border: 1px solid #d3dfef;
  color: #4e617f;
  background: #f4f8ff;
}

.dashboard-health-badge::before {
  content: "";
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
  box-shadow: 0 0 0 5px rgba(92, 120, 154, 0.2);
}

.dashboard-health-badge[data-tone="online"] {
  border-color: #b8e8cb;
  color: #0f6a3f;
  background: #ebfbf1;
}

.dashboard-health-badge[data-tone="online"]::before {
  box-shadow: 0 0 0 5px rgba(15, 106, 63, 0.16);
}

.dashboard-health-badge[data-tone="warn"] {
  border-color: #ffdca3;
  color: #8f5b00;
  background: #fff7e2;
}

.dashboard-health-badge[data-tone="warn"]::before {
  box-shadow: 0 0 0 5px rgba(143, 91, 0, 0.16);
}

.dashboard-health-badge[data-tone="offline"] {
  border-color: #f4c3c3;
  color: #a41f1f;
  background: #fff2f2;
}

.dashboard-health-badge[data-tone="offline"]::before {
  box-shadow: 0 0 0 5px rgba(164, 31, 31, 0.15);
}

.dashboard-toolbar__refresh {
  height: 34px;
  border: 1px solid #c4d8f1;
  border-radius: 10px;
  background: linear-gradient(180deg, #ffffff 0%, #f4f8ff 100%);
  color: #2f5f9b;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: border-color 160ms ease, background 160ms ease, color 160ms ease, box-shadow 160ms ease;
}

.dashboard-toolbar__refresh:hover {
  border-color: #9bbce7;
  background: linear-gradient(180deg, #fefeff 0%, #eaf2ff 100%);
  color: #294f83;
  box-shadow: 0 8px 16px rgba(58, 96, 151, 0.15);
}

.dashboard-toolbar__refresh:disabled {
  cursor: default;
  opacity: 0.65;
}

.dashboard-section {
  margin-top: 14px;
  border: 1px solid var(--dashboard-border);
  border-radius: 18px;
  background: linear-gradient(160deg, rgba(255, 255, 255, 0.98), rgba(246, 251, 255, 0.95));
  box-shadow: var(--dashboard-shadow), inset 0 1px 0 rgba(255, 255, 255, 0.78);
  overflow: hidden;
}

.dashboard-section__header {
  padding: 15px 16px;
  border-bottom: 1px solid #e0ebf9;
  background: linear-gradient(180deg, rgba(242, 248, 255, 0.74), rgba(246, 251, 255, 0.42));
}

.dashboard-section__header strong {
  display: block;
  font-size: 19px;
  letter-spacing: -0.012em;
  color: #203756;
}

.dashboard-section__header p {
  margin: 5px 0 0;
  font-size: 12px;
  color: #6785ad;
  line-height: 1.45;
}

.dashboard-stats-grid,
.dashboard-status-grid {
  display: grid;
  gap: 12px;
  padding: 14px;
}

.dashboard-stats-grid {
  grid-template-columns: repeat(auto-fit, minmax(184px, 1fr));
}

.dashboard-status-grid {
  grid-template-columns: repeat(auto-fit, minmax(236px, 1fr));
}

.dashboard-metric-card,
.dashboard-status-card {
  position: relative;
  border: 1px solid #d3e0f3;
  border-radius: 15px;
  background: linear-gradient(180deg, #ffffff 0%, #f8fbff 100%);
  padding: 13px 14px;
  min-height: 108px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow: hidden;
  transition: transform 150ms ease, box-shadow 150ms ease, border-color 150ms ease;
}

.dashboard-metric-card::before,
.dashboard-status-card::before {
  content: "";
  position: absolute;
  top: 0;
  left: 14px;
  right: 14px;
  height: 3px;
  border-radius: 0 0 8px 8px;
  background: linear-gradient(90deg, #8eb0de 0%, #6090cd 100%);
  opacity: 0.44;
}

.dashboard-metric-card:hover,
.dashboard-status-card:hover {
  transform: translateY(-1px);
  border-color: #b8cfea;
  box-shadow: 0 10px 20px rgba(57, 88, 132, 0.12);
}

.dashboard-metric-card--compact {
  min-height: 96px;
}

.dashboard-metric-card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.dashboard-metric-card__head > span:first-child,
.dashboard-status-card .dashboard-metric-card__head > span:first-child {
  color: #6f89ad;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.dashboard-metric-card strong {
  color: #1f3552;
  font-size: 20px;
  line-height: 1.1;
  margin: 0;
  font-weight: 800;
  letter-spacing: -0.01em;
  overflow-wrap: anywhere;
}

.dashboard-metric-card--compact strong {
  font-size: 19px;
}

.dashboard-metric-card p,
.dashboard-status-card p {
  margin: 0;
  color: #5d7ca3;
  font-size: 12px;
  line-height: 1.42;
  overflow-wrap: anywhere;
}

.dashboard-metric-card p {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}

.dashboard-card-icon {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #c8daf3;
  background: linear-gradient(160deg, #edf5ff, #e5f0ff);
  color: #476b9b;
  flex: 0 0 auto;
}

.dashboard-card-icon svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.dashboard-metric-card[data-tone="online"] {
  border-color: #bce7cd;
  background: linear-gradient(162deg, #f2fcf7, #f8fefb);
}

.dashboard-metric-card[data-tone="warn"] {
  border-color: #f2dcad;
  background: linear-gradient(162deg, #fff9ee, #fffdf6);
}

.dashboard-metric-card[data-tone="offline"] {
  border-color: #efc7c7;
  background: linear-gradient(162deg, #fff6f6, #fffafa);
}

.dashboard-status-card[data-tone="online"] {
  border-color: #bce7cd;
  background: linear-gradient(162deg, #f2fcf7, #f7fef9);
}

.dashboard-status-card[data-tone="warn"] {
  border-color: #f2dcad;
  background: linear-gradient(162deg, #fff9ee, #fffdf6);
}

.dashboard-status-card[data-tone="offline"] {
  border-color: #efc7c7;
  background: linear-gradient(162deg, #fff6f6, #fffafa);
}

.dashboard-metric-card[data-tone="online"]::before,
.dashboard-status-card[data-tone="online"]::before {
  background: linear-gradient(90deg, #5bc48f 0%, #2da472 100%);
}

.dashboard-metric-card[data-tone="warn"]::before,
.dashboard-status-card[data-tone="warn"]::before {
  background: linear-gradient(90deg, #f0b44d 0%, #dc8e1b 100%);
}

.dashboard-metric-card[data-tone="offline"]::before,
.dashboard-status-card[data-tone="offline"]::before {
  background: linear-gradient(90deg, #dd7e7e 0%, #cb5f5f 100%);
}

.dashboard-card-icon[data-tone="online"] {
  border-color: #bfe7cf;
  color: #2d8a61;
  background: linear-gradient(160deg, #e9f9f0, #e3f7ec);
}

.dashboard-card-icon[data-tone="warn"] {
  border-color: #f5deaf;
  color: #9b6b17;
  background: linear-gradient(160deg, #fff6e6, #fff3dd);
}

.dashboard-card-icon[data-tone="offline"] {
  border-color: #f1cccc;
  color: #a94a4a;
  background: linear-gradient(160deg, #fff1f1, #ffeded);
}

.dashboard-status-card {
  min-height: 122px;
}

.dashboard-status-card strong {
  color: #1c3555;
  font-size: 28px;
  line-height: 1.1;
  letter-spacing: -0.018em;
  margin: 1px 0;
  overflow-wrap: anywhere;
}

.dashboard-status-card p {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  overflow: hidden;
}

.dashboard-activity-panel {
  margin-top: 15px;
  border: 1px solid var(--dashboard-border);
  border-radius: 18px;
  background: linear-gradient(165deg, rgba(255, 255, 255, 0.98), rgba(246, 251, 255, 0.95));
  box-shadow: var(--dashboard-shadow);
  overflow: hidden;
}

.dashboard-activity-panel__header {
  padding: 14px 16px 12px;
  border-bottom: 1px solid #e3ecf8;
  background: linear-gradient(180deg, rgba(241, 248, 255, 0.78), rgba(248, 252, 255, 0.5));
}

.dashboard-activity-panel__header strong {
  display: block;
  color: #203654;
  font-size: 18px;
}

.dashboard-activity-panel__header p {
  margin: 5px 0 0;
  color: #5f7ea5;
  font-size: 12px;
}

.dashboard-activity-panel__empty {
  padding: 18px 16px;
  color: #6f86a7;
  font-size: 13px;
}

.dashboard-activity-list {
  max-height: 320px;
  overflow: auto;
}

.dashboard-activity-item {
  padding: 11px 16px 10px;
  display: grid;
  grid-template-columns: 86px 58px minmax(0, 1fr);
  align-items: center;
  gap: 10px;
  border-top: 1px solid #e6eef9;
  transition: background 140ms ease;
}

.dashboard-activity-item:hover {
  background: rgba(237, 245, 255, 0.48);
}

.dashboard-activity-item:first-child {
  border-top: 0;
}

.dashboard-activity-item__time {
  color: #607da3;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.dashboard-activity-item__tag {
  justify-self: start;
  height: 23px;
  min-width: 49px;
  border-radius: 999px;
  padding: 0 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  border: 1px solid #d6dfef;
  color: #516a8e;
  background: #f2f7ff;
}

.dashboard-activity-item__tag[data-tone="online"] {
  border-color: #c2eace;
  color: #186a44;
  background: #ecfbf1;
}

.dashboard-activity-item__tag[data-tone="warn"] {
  border-color: #ffdfa6;
  color: #935f00;
  background: #fff6df;
}

.dashboard-activity-item__tag[data-tone="offline"] {
  border-color: #f4c8c8;
  color: #a22b2b;
  background: #fff2f2;
}

.dashboard-activity-item p {
  margin: 0;
  color: #2f486a;
  font-size: 12px;
  line-height: 1.5;
  overflow-wrap: anywhere;
}

.module-surface {
  margin-top: 14px;
  border: 1px solid #d5e4f7;
  border-radius: 16px;
  background: linear-gradient(165deg, rgba(252, 254, 255, 0.98), rgba(243, 249, 255, 0.95));
  box-shadow: 0 14px 30px rgba(46, 77, 125, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.72);
  padding: 14px;
}

.recruitment-surface {
  background:
    radial-gradient(120% 68% at 0% 0%, rgba(66, 135, 229, 0.13) 0%, rgba(66, 135, 229, 0) 62%),
    linear-gradient(162deg, rgba(252, 254, 255, 0.98), rgba(243, 249, 255, 0.95));
}

.skill-market-surface {
  background:
    radial-gradient(110% 62% at 100% 0%, rgba(43, 179, 134, 0.12) 0%, rgba(43, 179, 134, 0) 64%),
    linear-gradient(162deg, rgba(252, 254, 255, 0.98), rgba(243, 249, 255, 0.95));
}

.module-surface__toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.module-surface__toolbar--skills {
  margin-top: 12px;
}

.module-surface__search {
  flex: 1;
  min-width: 0;
  height: 38px;
  border: 1px solid #cadcf3;
  border-radius: 11px;
  background: linear-gradient(180deg, #ffffff, #f8fbff);
  color: #273754;
  padding: 0 12px;
  font-size: 13px;
  outline: 0;
  transition: border-color 150ms ease, box-shadow 150ms ease, background 150ms ease;
}

.module-surface__search:focus {
  border-color: #8eb6f0;
  box-shadow: 0 0 0 3px rgba(76, 126, 201, 0.14);
  background: #ffffff;
}

.module-surface__meta {
  height: 34px;
  border-radius: 999px;
  border: 1px solid #d1e1f4;
  background: rgba(255, 255, 255, 0.9);
  color: #5f7fa8;
  font-size: 12px;
  display: inline-flex;
  align-items: center;
  padding: 0 11px;
  font-weight: 600;
  letter-spacing: 0.01em;
  white-space: nowrap;
}

.module-surface__select {
  height: 38px;
  border: 1px solid #cadcf3;
  border-radius: 11px;
  background:
    linear-gradient(45deg, transparent 50%, #6283ad 50%) calc(100% - 14px) calc(50% - 1px) / 5px 5px no-repeat,
    linear-gradient(135deg, #6283ad 50%, transparent 50%) calc(100% - 9px) calc(50% - 1px) / 5px 5px no-repeat,
    linear-gradient(180deg, #ffffff 0%, #f7fbff 100%);
  padding: 0 24px 0 11px;
  color: #2f476b;
  font-size: 12px;
  font-weight: 600;
  outline: 0;
  appearance: none;
  transition: border-color 150ms ease, box-shadow 150ms ease;
}

.module-surface__select:focus {
  border-color: #8eb6f0;
  box-shadow: 0 0 0 3px rgba(76, 126, 201, 0.14);
}

.module-surface__button {
  height: 38px;
  border: 1px solid #7fa9e7;
  border-radius: 11px;
  background: linear-gradient(135deg, #4f88de, #366ec8);
  color: #ffffff;
  padding: 0 14px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.01em;
  cursor: pointer;
  box-shadow: 0 8px 16px rgba(58, 98, 165, 0.25);
  transition: transform 150ms ease, box-shadow 150ms ease, filter 150ms ease;
}

.module-surface__button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 12px 20px rgba(58, 98, 165, 0.28);
  filter: brightness(1.03);
}

.module-surface__button:active:not(:disabled) {
  transform: translateY(0);
}

.module-surface__button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  box-shadow: none;
}

.module-surface__hint {
  margin: 10px 0 0;
  border: 1px solid #dce8f7;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.82);
  color: #617c9f;
  font-size: 12px;
  line-height: 1.5;
  padding: 8px 10px;
}

.module-surface__hint--notice {
  border-color: #cdddf4;
  background: rgba(237, 245, 255, 0.72);
  color: #3e5f8d;
}

.module-surface__hint--error {
  border-color: #f1d1d1;
  background: #fff4f4;
  color: #a83b3b;
}

.module-empty {
  margin-top: 10px;
  border: 1px dashed #c8d9ef;
  border-radius: 12px;
  padding: 16px;
  color: #6682a7;
  font-size: 13px;
  text-align: center;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.88), rgba(245, 250, 255, 0.92));
}

.recruitment-division-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.recruitment-division {
  border: 1px solid #d6e5f8;
  border-radius: 14px;
  background: linear-gradient(165deg, #ffffff, #f7fbff);
  box-shadow: 0 12px 24px rgba(54, 82, 125, 0.08);
  padding: 12px;
}

.recruitment-division__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding-bottom: 10px;
  border-bottom: 1px solid #e3edf9;
}

.recruitment-division__header strong {
  color: #284262;
  font-size: 15px;
  letter-spacing: 0.01em;
  display: inline-flex;
  align-items: center;
  gap: 7px;
}

.recruitment-division__header strong::before {
  content: "";
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: linear-gradient(135deg, #4f88de, #2f66bf);
  box-shadow: 0 0 0 4px rgba(79, 136, 222, 0.18);
}

.recruitment-division__header small {
  border-radius: 999px;
  border: 1px solid #d4e2f6;
  background: #f2f7ff;
  color: #5f7da3;
  font-size: 11px;
  font-weight: 700;
  padding: 3px 10px;
}

.recruitment-role-grid {
  margin-top: 10px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
}

.recruitment-role-card {
  border: 1px solid #d6e4f6;
  border-radius: 12px;
  background: linear-gradient(160deg, rgba(255, 255, 255, 0.98), rgba(246, 251, 255, 0.95));
  padding: 11px;
  display: flex;
  flex-direction: column;
  gap: 9px;
  position: relative;
  overflow: hidden;
  transition: transform 150ms ease, box-shadow 150ms ease, border-color 150ms ease;
}

.recruitment-role-card::before {
  content: "";
  position: absolute;
  top: 0;
  left: 14px;
  right: 14px;
  height: 3px;
  border-radius: 0 0 8px 8px;
  background: linear-gradient(90deg, #8ab2f2, #4d86dd);
  opacity: 0.5;
}

.recruitment-role-card:hover {
  transform: translateY(-1px);
  border-color: #c3d8f2;
  box-shadow: 0 12px 22px rgba(58, 91, 140, 0.14);
}

.recruitment-role-card strong {
  color: #233d5f;
  font-size: 14px;
  letter-spacing: 0.005em;
}

.recruitment-role-card p {
  margin: 4px 0 0;
  color: #5f7ca1;
  font-size: 12px;
  line-height: 1.45;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}

.recruitment-role-card small {
  display: inline-flex;
  margin-top: 5px;
  width: fit-content;
  max-width: 100%;
  border-radius: 999px;
  border: 1px solid #d7e4f6;
  background: #f7fbff;
  color: #6782a5;
  font-size: 11px;
  padding: 3px 8px;
  font-family: "SFMono-Regular", "Consolas", "Menlo", monospace;
  word-break: break-word;
}

.recruitment-role-card__action {
  align-self: flex-start;
  height: 32px;
  border: 1px solid #84ace8;
  border-radius: 10px;
  background: linear-gradient(135deg, #4f88de, #376fc8);
  color: #ffffff;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.01em;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  box-shadow: 0 8px 14px rgba(57, 95, 157, 0.24);
  transition: transform 140ms ease, box-shadow 140ms ease, filter 140ms ease;
}

.recruitment-role-card__actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: auto;
  padding-top: 9px;
  border-top: 1px solid #e5edf8;
}

.recruitment-role-card__action--secondary {
  border-color: #d0ddee;
  background: #ffffff;
  color: #4f668a;
  box-shadow: none;
}

.recruitment-role-card__action:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 10px 16px rgba(57, 95, 157, 0.28);
  filter: brightness(1.03);
}

.recruitment-role-card__action--secondary:hover:not(:disabled) {
  box-shadow: 0 8px 14px rgba(64, 93, 136, 0.12);
}

.recruitment-role-card__action:active:not(:disabled) {
  transform: translateY(0);
}

.recruitment-role-card__action:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  box-shadow: none;
}

.role-workflow-detail-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(17, 28, 48, 0.42);
  display: grid;
  place-items: center;
  z-index: 1210;
}

.role-workflow-detail-modal {
  width: min(940px, calc(100vw - 40px));
  max-height: calc(100vh - 56px);
  border: 1px solid #dbe4f3;
  border-radius: 14px;
  background: #ffffff;
  box-shadow: 0 18px 48px rgba(26, 44, 78, 0.26);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
}

.role-workflow-detail-modal__header {
  padding: 14px 16px 12px;
  border-bottom: 1px solid #e8eef8;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.role-workflow-detail-modal__header strong {
  display: block;
  color: #253450;
  font-size: 18px;
}

.role-workflow-detail-modal__header p {
  margin: 4px 0 0;
  color: #7c8da8;
  font-size: 12px;
}

.role-workflow-detail-modal__close {
  width: 30px;
  height: 30px;
  border: 1px solid #d6dfef;
  border-radius: 8px;
  background: #f4f8ff;
  color: #4d607f;
  line-height: 1;
  font-size: 20px;
  cursor: pointer;
  flex: 0 0 auto;
}

.role-workflow-detail-modal__body {
  padding: 14px 16px 16px;
  min-height: 0;
  overflow: auto;
  display: grid;
  gap: 12px;
}

.role-workflow-detail-modal__notice {
  margin: 0;
  border-radius: 10px;
  padding: 9px 10px;
  font-size: 12px;
  line-height: 1.45;
}

.role-workflow-detail-modal__notice--success {
  border: 1px solid #c9e8d4;
  background: #eefaf3;
  color: #186a44;
}

.role-workflow-detail-modal__notice--error {
  border: 1px solid #f1c5c5;
  background: #fff2f2;
  color: #a32e2e;
}

.role-workflow-detail-modal__field {
  display: grid;
  gap: 6px;
}

.role-workflow-detail-modal__field span {
  color: #60728f;
  font-size: 12px;
}

.role-workflow-detail-modal__name-input,
.role-workflow-detail-modal__editor {
  width: 100%;
  border: 1px solid #d5deec;
  border-radius: 9px;
  background: #ffffff;
  color: #2c3a54;
  font-size: 13px;
  outline: 0;
}

.role-workflow-detail-modal__name-input {
  height: 36px;
  padding: 0 10px;
}

.role-workflow-detail-modal__editor {
  min-height: 340px;
  padding: 10px;
  line-height: 1.5;
  resize: vertical;
}

.role-workflow-detail-modal__name-input:disabled,
.role-workflow-detail-modal__editor:disabled,
.role-workflow-detail-modal__close:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.role-workflow-detail-modal__versions {
  border: 1px solid #dfe8f6;
  border-radius: 12px;
  background: #f9fbff;
  padding: 10px;
}

.role-workflow-detail-modal__versions-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.role-workflow-detail-modal__versions-header strong {
  color: #2d3f5d;
  font-size: 12px;
}

.role-workflow-detail-modal__versions-empty {
  margin: 8px 0 0;
  color: #7b8ea9;
  font-size: 12px;
}

.role-workflow-detail-modal__versions-list {
  margin: 8px 0 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 8px;
}

.role-workflow-detail-modal__version-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  border: 1px solid #dbe5f5;
  border-radius: 10px;
  background: #ffffff;
  padding: 8px 9px;
}

.role-workflow-detail-modal__version-time {
  color: #687e9f;
  font-size: 12px;
}

.role-workflow-detail-modal__version-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.role-workflow-detail-modal__version-actions button {
  height: 28px;
  border: 1px solid #d2deef;
  border-radius: 7px;
  background: #ffffff;
  color: #4f668a;
  padding: 0 10px;
  font-size: 12px;
  cursor: pointer;
}

.role-workflow-detail-modal__version-actions button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.role-workflow-detail-modal__actions {
  padding: 12px 16px 14px;
  border-top: 1px solid #e8eef8;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.role-workflow-detail-modal__actions button {
  height: 32px;
  border: 1px solid #d2deef;
  border-radius: 8px;
  background: #ffffff;
  color: #4f668a;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.role-workflow-detail-modal__actions button:last-child {
  border-color: #c9daf6;
  background: #eaf3ff;
  color: #315f95;
}

.role-workflow-detail-modal__actions button:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.skill-market-category-row {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(146px, 1fr));
  gap: 10px;
}

.skill-market-category-chip {
  border: 1px solid #d4e4f6;
  border-radius: 12px;
  background: linear-gradient(160deg, #ffffff, #f5faff);
  padding: 9px 10px;
  text-align: left;
  cursor: pointer;
  position: relative;
  overflow: hidden;
  transition: transform 140ms ease, box-shadow 140ms ease, border-color 140ms ease;
}

.skill-market-category-chip::after {
  content: "";
  position: absolute;
  top: -32px;
  right: -32px;
  width: 74px;
  height: 74px;
  border-radius: 50%;
  background: radial-gradient(circle at center, rgba(79, 136, 222, 0.16), rgba(79, 136, 222, 0));
  pointer-events: none;
}

.skill-market-category-chip:hover {
  transform: translateY(-1px);
  border-color: #c0d8f5;
  box-shadow: 0 10px 18px rgba(60, 95, 144, 0.12);
}

.skill-market-category-chip strong {
  display: block;
  color: #2b4568;
  font-size: 13px;
  letter-spacing: 0.01em;
}

.skill-market-category-chip small {
  display: block;
  margin-top: 3px;
  color: #7089a9;
  font-size: 11px;
  line-height: 1.45;
}

.skill-market-category-chip.active {
  border-color: #8ab2ed;
  background: linear-gradient(160deg, #f5faff, #eaf3ff);
  box-shadow: 0 12px 20px rgba(62, 99, 151, 0.18);
}

.skill-market-category-chip.active strong {
  color: #234a7f;
}

.skill-market-category-chip.active small {
  color: #5578a4;
}

.skill-market-grid {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(255px, 1fr));
  gap: 12px;
}

.skill-market-card-v2 {
  border: 1px solid #d4e3f7;
  border-radius: 14px;
  background: linear-gradient(160deg, rgba(255, 255, 255, 0.98), rgba(246, 251, 255, 0.95));
  box-shadow: 0 10px 22px rgba(52, 84, 132, 0.1);
  padding: 12px;
  display: grid;
  gap: 9px;
  position: relative;
  overflow: hidden;
  transition: transform 160ms ease, box-shadow 160ms ease, border-color 160ms ease;
}

.skill-market-card-v2::before {
  content: "";
  position: absolute;
  top: 0;
  left: 14px;
  right: 14px;
  height: 3px;
  border-radius: 0 0 8px 8px;
  background: linear-gradient(90deg, #62ae8d 0%, #3c8f6e 100%);
  opacity: 0.5;
}

.skill-market-card-v2:hover {
  transform: translateY(-2px);
  border-color: #bdd7ef;
  box-shadow: 0 14px 24px rgba(52, 84, 132, 0.16);
}

.skill-market-card-v2__header {
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr);
  gap: 10px;
  align-items: start;
}

.skill-market-card-v2__avatar {
  width: 42px;
  height: 42px;
  border-radius: 11px;
  display: grid;
  place-items: center;
  font-size: 16px;
  font-weight: 700;
  color: #ffffff;
  background: linear-gradient(145deg, #4d87dc, #376ec6);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.32), 0 8px 16px rgba(58, 99, 164, 0.24);
}

.skill-market-card-v2__title strong {
  display: block;
  color: #243f61;
  font-size: 14px;
  line-height: 1.35;
}

.skill-market-card-v2__title p {
  margin: 4px 0 0;
  color: #5f7ca1;
  font-size: 12px;
  line-height: 1.45;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  overflow: hidden;
}

.skill-market-card-v2__meta,
.skill-market-card-v2__tags {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
}

.skill-market-card-v2__meta span {
  border: 1px solid #d7e4f7;
  background: #f1f7ff;
  color: #4e6f98;
  font-variant-numeric: tabular-nums;
}

.skill-market-card-v2__tags span {
  border: 1px solid #d5e7dd;
  background: #eefaf3;
  color: #2f6f56;
}

.skill-market-card-v2__meta span,
.skill-market-card-v2__tags span {
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 600;
}

.skill-market-card-v2__tags span:last-child {
  border-color: #d8e0f2;
  background: #f6f9ff;
  color: #5c7193;
}

.skill-market-card-v2__action {
  height: 32px;
  border: 1px solid #84ace8;
  border-radius: 10px;
  background: linear-gradient(135deg, #4f88de, #376fc8);
  color: #ffffff;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.01em;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  box-shadow: 0 8px 14px rgba(57, 95, 157, 0.24);
  transition: transform 140ms ease, box-shadow 140ms ease, filter 140ms ease;
}

.skill-market-card-v2__action:disabled {
  cursor: not-allowed;
  opacity: 0.6;
  box-shadow: none;
}

.skill-market-card-v2__actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: auto;
  padding-top: 9px;
  border-top: 1px solid #e5edf8;
}

.skill-market-card-v2__action--ghost {
  border-color: #d0ddee;
  background: #ffffff;
  color: #4f668a;
  box-shadow: none;
}

.skill-market-card-v2__action:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 10px 16px rgba(57, 95, 157, 0.28);
  filter: brightness(1.03);
}

.skill-market-card-v2__action--ghost:hover:not(:disabled) {
  box-shadow: 0 8px 14px rgba(64, 93, 136, 0.12);
}

.skill-market-card-v2__action:active:not(:disabled) {
  transform: translateY(0);
}

.skill-market-detail-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(17, 28, 48, 0.42);
  display: grid;
  place-items: center;
  z-index: 1200;
}

.skill-market-detail-modal {
  width: min(560px, calc(100vw - 40px));
  max-height: calc(100vh - 56px);
  border: 1px solid #dbe4f3;
  border-radius: 14px;
  background: #ffffff;
  box-shadow: 0 18px 48px rgba(26, 44, 78, 0.26);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
}

.skill-market-detail-modal__header {
  padding: 14px 16px 12px;
  border-bottom: 1px solid #e8eef8;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.skill-market-detail-modal__identity {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.skill-market-detail-modal__avatar {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  font-size: 15px;
  font-weight: 700;
  color: #2f5f95;
  background: #eaf3ff;
  flex: 0 0 auto;
}

.skill-market-detail-modal__identity strong {
  display: block;
  color: #2a3b58;
  font-size: 15px;
}

.skill-market-detail-modal__identity p {
  margin: 2px 0 0;
  color: #7a8ca8;
  font-size: 12px;
}

.skill-market-detail-modal__close {
  width: 30px;
  height: 30px;
  border: 1px solid #d6dfef;
  border-radius: 8px;
  background: #f4f8ff;
  color: #4d607f;
  line-height: 1;
  font-size: 20px;
  cursor: pointer;
  flex: 0 0 auto;
}

.skill-market-detail-modal__body {
  padding: 14px 16px;
  min-height: 0;
  overflow: auto;
}

.skill-market-detail-modal__chips {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.skill-market-detail-modal__chips span {
  border: 1px solid #dce8f7;
  border-radius: 999px;
  background: #f8fbff;
  color: #60728f;
  font-size: 11px;
  padding: 2px 8px;
}

.skill-market-detail-modal__description {
  margin: 10px 0 0;
  color: #5c7295;
  font-size: 13px;
  line-height: 1.55;
}

.skill-market-detail-modal__stats {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}

.skill-market-detail-modal__stats article {
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #f9fcff;
  padding: 10px;
}

.skill-market-detail-modal__stats span {
  display: block;
  color: #7a8ca8;
  font-size: 11px;
}

.skill-market-detail-modal__stats strong {
  display: block;
  margin-top: 4px;
  color: #304868;
  font-size: 15px;
}

.skill-market-detail-modal__actions {
  padding: 12px 16px 14px;
  border-top: 1px solid #e8eef8;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.skill-market-detail-modal__action {
  height: 32px;
  border: 1px solid #d2deef;
  border-radius: 8px;
  background: #ffffff;
  color: #4f668a;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.skill-market-detail-modal__action--primary {
  border-color: #c9daf6;
  background: #eaf3ff;
  color: #315f95;
}

.skill-market-detail-modal__action:disabled,
.skill-market-detail-modal__close:disabled {
  cursor: not-allowed;
  opacity: 0.6;
}

.skill-market-pagination {
  margin-top: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 8px;
}

.skill-market-pagination__button,
.skill-market-pagination__page {
  height: 33px;
  min-width: 33px;
  border: 1px solid #c8d9ef;
  border-radius: 10px;
  background: linear-gradient(180deg, #ffffff, #f5f9ff);
  color: #436186;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  padding: 0 10px;
  transition: transform 130ms ease, box-shadow 130ms ease, border-color 130ms ease;
}

.skill-market-pagination__button:hover:not(:disabled),
.skill-market-pagination__page:hover:not(:disabled) {
  transform: translateY(-1px);
  border-color: #b6d0ee;
  box-shadow: 0 8px 14px rgba(62, 97, 146, 0.14);
}

.skill-market-pagination__page.active {
  border-color: #88b1ea;
  background: linear-gradient(135deg, #e8f2ff, #dfeeff);
  color: #2c548c;
  font-weight: 700;
  box-shadow: 0 8px 14px rgba(62, 97, 146, 0.16);
}

.skill-market-pagination__button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
  box-shadow: none;
}

.module-surface__button:focus-visible,
.recruitment-role-card__action:focus-visible,
.skill-market-card-v2__action:focus-visible,
.skill-market-pagination__button:focus-visible,
.skill-market-pagination__page:focus-visible,
.skill-market-category-chip:focus-visible {
  outline: 2px solid rgba(62, 108, 178, 0.4);
  outline-offset: 2px;
}

.task-module-surface {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.task-project-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
}

.task-project-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
}

.task-project-card {
  border: 1px solid #d9e4f5;
  border-radius: 12px;
  background: #ffffff;
  padding: 11px;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 7px;
  cursor: pointer;
  transition: transform 130ms ease, border-color 130ms ease, box-shadow 130ms ease;
}

.task-project-card:hover {
  transform: translateY(-1px);
  border-color: #b7cff0;
  box-shadow: 0 8px 20px rgba(62, 94, 146, 0.12);
}

.task-project-card--default {
  border-color: #9fc0f5;
  background: #f4f8ff;
}

.task-project-card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.task-project-card__head strong {
  color: #2d3f5d;
  font-size: 14px;
}

.task-project-card__head span {
  border-radius: 999px;
  padding: 2px 8px;
  border: 1px solid #c9dcf6;
  background: #ebf3ff;
  color: #466895;
  font-size: 11px;
  font-weight: 700;
}

.task-project-card p {
  margin: 0;
  color: #607895;
  font-size: 12px;
  line-height: 1.45;
}

.task-project-card small {
  color: #8093ae;
  font-size: 11px;
}

.task-board-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.task-board-topbar strong {
  color: #2f3f5b;
  font-size: 16px;
}

.task-board-topbar p {
  margin: 3px 0 0;
  color: #6d83a4;
  font-size: 12px;
}

.task-board-back {
  height: 32px;
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

.task-board-creator {
  display: grid;
  grid-template-columns: minmax(200px, 1.5fr) minmax(160px, 1.5fr) minmax(120px, 1fr) auto auto;
  gap: 8px;
}

.task-board-creator__input,
.task-board-creator__select,
.task-board-creator__button {
  height: 34px;
  border-radius: 8px;
  font-size: 12px;
}

.task-board-creator__input {
  border: 1px solid #d3dff0;
  background: #ffffff;
  color: #304868;
  padding: 0 10px;
  outline: 0;
}

.task-board-creator__select {
  padding-left: 10px;
}

.task-board-creator__input--title {
  font-weight: 600;
}

.task-board-creator__button {
  border: 1px solid #c9daf6;
  background: #eaf3ff;
  color: #315f95;
  padding: 0 12px;
  font-weight: 700;
  cursor: pointer;
}

.task-board-columns {
  display: grid;
  grid-template-columns: repeat(5, minmax(230px, 1fr));
  gap: 10px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.task-board-column {
  border: 1px solid #dbe7f7;
  border-radius: 12px;
  background: #f8fbff;
  padding: 10px;
  min-height: 320px;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 8px;
}

.task-board-column.is-drag-over {
  border-color: #8fb7f3;
  background: #eef5ff;
}

.task-board-column__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.task-board-column__header strong {
  display: block;
  color: #2d3f5d;
  font-size: 14px;
}

.task-board-column__header span {
  color: #6f84a5;
  font-size: 11px;
}

.task-board-column__header em {
  font-style: normal;
  border-radius: 999px;
  border: 1px solid #cfe0f7;
  padding: 1px 8px;
  background: #edf4ff;
  color: #486894;
  font-size: 11px;
  font-weight: 700;
}

.task-board-column__empty {
  border: 1px dashed #d2ddec;
  border-radius: 10px;
  padding: 14px 10px;
  text-align: center;
  color: #7f90aa;
  font-size: 12px;
  align-self: start;
}

.task-board-column__list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
}

.task-board-card {
  border: 1px solid #d8e4f5;
  border-radius: 10px;
  background: #ffffff;
  padding: 9px;
  display: grid;
  gap: 6px;
  cursor: grab;
}

.task-board-card[data-priority="p0"] {
  border-color: #f1c9c9;
  background: #fff9f9;
}

.task-board-card[data-priority="p1"] {
  border-color: #d8e4f5;
}

.task-board-card[data-priority="p2"] {
  border-color: #dce6ef;
  background: #fafcff;
}

.task-board-card[data-status="in_progress"] {
  border-color: #bfe4cc;
  background: #f2fdf6;
}

.task-board-card[data-status="in_review"] {
  border-color: #f4deb0;
  background: #fff9ee;
}

.task-board-card[data-status="done"] {
  border-color: #cbead5;
  background: #eefbf3;
}

.task-board-card[data-status="cancelled"] {
  border-color: #e4e7ed;
  background: #f7f8fb;
}

.task-board-card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.task-board-card__head strong {
  color: #2f3f5b;
  font-size: 13px;
  line-height: 1.35;
}

.task-board-card__head span {
  flex-shrink: 0;
  border: 1px solid #d3dff0;
  border-radius: 999px;
  padding: 1px 7px;
  font-size: 10px;
  font-weight: 700;
  color: #60728f;
  background: #f8fbff;
}

.task-board-card p {
  margin: 0;
  color: #6f83a3;
  font-size: 12px;
  line-height: 1.4;
}

.task-board-card small {
  color: #8596b1;
  font-size: 11px;
}

.task-board-card__actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.task-board-card__actions button {
  flex: 1;
  height: 27px;
  border: 1px solid #d1ddef;
  border-radius: 7px;
  background: #f8fbff;
  color: #587297;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.task-board-card__actions button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
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

.related-model-form {
  border: 1px solid #dfe8f6;
  border-radius: 12px;
  background: #fbfdff;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.related-model-form__meta {
  margin: 0;
  color: #6f83a3;
  font-size: 12px;
}

.related-model-form__field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.related-model-form__field span {
  color: #5f7394;
  font-size: 12px;
  font-weight: 600;
}

.related-model-form__input {
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

.related-model-form__select {
  width: 100%;
  height: 34px;
  border-radius: 9px;
  font-size: 13px;
  padding-left: 10px;
}

.related-model-form__actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.related-model-form__actions small {
  color: #7b8ca7;
  font-size: 11px;
}

.related-memory-layout {
  display: grid;
  grid-template-columns: 270px minmax(0, 1fr);
  gap: 12px;
  min-height: 440px;
}

.proxy-config-modal {
  width: min(980px, calc(100vw - 40px));
}

.proxy-config-layout {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr);
  gap: 12px;
  min-height: 430px;
}

.proxy-config-nav-pane {
  border: 1px solid #dfe8f6;
  border-radius: 12px;
  background: #f9fbff;
  min-height: 0;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
}

.proxy-config-nav-pane__toolbar {
  border-bottom: 1px solid #e7edf8;
  padding: 10px;
}

.proxy-config-nav-list {
  min-height: 0;
  overflow: auto;
  padding: 8px;
}

.proxy-config-nav-item {
  width: 100%;
  border: 1px solid #dfe8f6;
  border-radius: 10px;
  background: #ffffff;
  text-align: left;
  padding: 9px 10px;
  cursor: pointer;
}

.proxy-config-nav-item + .proxy-config-nav-item {
  margin-top: 6px;
}

.proxy-config-nav-item.active {
  border-color: #99bbf3;
  background: #eef5ff;
}

.proxy-config-nav-item strong {
  display: block;
  color: #2f3f5b;
  font-size: 13px;
}

.proxy-config-nav-item p {
  margin: 3px 0 0;
  color: #7386a6;
  font-size: 11px;
}

.proxy-config-editor-pane {
  border: 1px solid #dfe8f6;
  border-radius: 12px;
  background: #ffffff;
  padding: 12px;
  min-height: 0;
  overflow: auto;
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
  display: block;
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
  grid-template-rows: auto minmax(0, 1fr);
  gap: 8px;
  min-height: 0;
}

.related-memory-editor-pane__header {
  border: 1px solid #e2eaf7;
  border-radius: 10px;
  background: #f8fbff;
  padding: 8px 10px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  align-items: start;
}

.related-memory-editor-pane__meta {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
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

.sidebar-settings-modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(17, 28, 48, 0.42);
  display: grid;
  place-items: center;
  z-index: 1230;
}

.sidebar-settings-modal {
  width: min(920px, calc(100vw - 40px));
  max-height: calc(100vh - 56px);
  border-radius: 14px;
  border: 1px solid #dbe4f3;
  background: #ffffff;
  box-shadow: 0 18px 48px rgba(26, 44, 78, 0.26);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.sidebar-settings-modal__header {
  padding: 14px 16px 12px;
  border-bottom: 1px solid #e8eef8;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sidebar-settings-modal__header strong {
  color: #253450;
  font-size: 17px;
}

.sidebar-settings-modal__close {
  width: 30px;
  height: 30px;
  border: 1px solid #d6dfef;
  border-radius: 8px;
  background: #f4f8ff;
  color: #4d607f;
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
}

.sidebar-settings-modal__body {
  min-height: 0;
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
}

.sidebar-settings-nav {
  min-height: 0;
  overflow: auto;
  border-right: 1px solid #e8eef8;
  background: #f8fbff;
  padding: 14px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sidebar-settings-nav__item {
  width: 100%;
  border: 1px solid transparent;
  border-radius: 10px;
  background: transparent;
  color: #3d4f6d;
  text-align: left;
  font-size: 13px;
  font-weight: 600;
  padding: 8px 9px;
  cursor: pointer;
}

.sidebar-settings-nav__item + .sidebar-settings-nav__item {
  margin-top: 0;
}

.sidebar-settings-nav__item:hover {
  border-color: #d2deef;
  background: #ffffff;
}

.sidebar-settings-nav__item.is-active {
  border-color: #b9cdf1;
  background: #eaf2ff;
  color: #2556a8;
}

.sidebar-settings-panel {
  min-height: 0;
  overflow: auto;
  padding: 14px 16px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.sidebar-settings-panel__notice,
.sidebar-settings-panel__error {
  margin: 0;
  border-radius: 9px;
  padding: 8px 10px;
  font-size: 12px;
}

.sidebar-settings-panel__notice {
  border: 1px solid #bde0c8;
  background: #edfff2;
  color: #2d7c4d;
}

.sidebar-settings-panel__error {
  border: 1px solid #f1c2c2;
  background: #fff3f3;
  color: #a83b3b;
}

.sidebar-settings-card {
  border: 1px solid #dfe8f6;
  border-radius: 12px;
  background: #fbfdff;
  padding: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.sidebar-settings-card--column {
  align-items: stretch;
  flex-direction: column;
}

.sidebar-settings-card h4 {
  margin: 0;
  color: #2d405f;
  font-size: 15px;
}

.sidebar-settings-card p {
  margin: 5px 0 0;
  color: #7184a3;
  font-size: 12px;
}

.sidebar-settings-select {
  min-width: 170px;
  height: 34px;
  border-radius: 9px;
  padding-left: 10px;
  font-size: 13px;
}

.sidebar-settings-shortcut-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sidebar-settings-shortcut-item {
  border: 1px solid #dbe5f5;
  border-radius: 10px;
  background: #ffffff;
  padding: 9px 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.sidebar-settings-shortcut-item strong {
  display: block;
  color: #2f3f5b;
  font-size: 13px;
}

.sidebar-settings-shortcut-item p {
  margin: 3px 0 0;
  color: #7a8ba7;
  font-size: 11px;
}

.sidebar-settings-shortcut-item code {
  flex-shrink: 0;
  border-radius: 8px;
  border: 1px solid #d3ddf0;
  background: #f4f8ff;
  color: #2b4f88;
  font-size: 12px;
  font-weight: 700;
  padding: 4px 8px;
}

.sidebar-settings-toggle {
  width: 50px;
  height: 28px;
  border: 1px solid #d3deef;
  border-radius: 999px;
  background: #e6ecf7;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  cursor: pointer;
  transition: background 160ms ease, border-color 160ms ease;
}

.sidebar-settings-toggle i {
  display: block;
  width: 22px;
  height: 22px;
  border-radius: 999px;
  background: #ffffff;
  box-shadow: 0 1px 3px rgba(67, 86, 118, 0.2);
  transition: transform 160ms ease;
}

.sidebar-settings-toggle.is-on {
  border-color: #79c99d;
  background: #43bf78;
}

.sidebar-settings-toggle.is-on i {
  transform: translateX(22px);
}

.sidebar-settings-toggle:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.sidebar-settings-version {
  display: flex;
  align-items: center;
  gap: 10px;
}

.sidebar-settings-version strong {
  color: #214274;
  font-size: 18px;
}

.sidebar-settings-text-button {
  height: 30px;
  border: 1px solid #d5deec;
  border-radius: 9px;
  background: #f4f8ff;
  color: #3b5377;
  font-size: 12px;
  font-weight: 600;
  padding: 0 11px;
  cursor: pointer;
}

.sidebar-settings-text-button:hover {
  border-color: #b7c8e3;
  background: #edf4ff;
}

.sidebar-settings-tip-list {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 7px;
  color: #2f3f5b;
  font-size: 13px;
  line-height: 1.5;
}

.sidebar-settings-feedback {
  width: 100%;
  border: 1px solid #d5deec;
  border-radius: 10px;
  outline: 0;
  resize: vertical;
  min-height: 120px;
  background: #ffffff;
  color: #2c3a54;
  padding: 10px;
  font-size: 13px;
  line-height: 1.45;
}

.sidebar-settings-feedback__actions {
  display: flex;
  align-items: center;
  gap: 8px;
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

.utility-modal__body--logs {
  overflow: hidden;
  display: flex;
  flex-direction: column;
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

.utility-log-tabs {
  margin-bottom: 10px;
  border-radius: 999px;
  padding: 4px;
  background: #f1f3f8;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px;
}

.utility-log-tab {
  height: 32px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: #8a96ad;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.utility-log-tab.is-active {
  color: #384a68;
  background: #ffffff;
  box-shadow: 0 1px 4px rgba(96, 110, 140, 0.16);
}

.utility-log-categories {
  margin-bottom: 10px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.utility-log-category {
  height: 28px;
  border: 1px solid #d8e3f4;
  border-radius: 8px;
  padding: 0 10px;
  background: #f8fbff;
  color: #6880a3;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.utility-log-category.is-active {
  border-color: #9dbdea;
  background: #eaf3ff;
  color: #345986;
}

.utility-log-layout {
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  gap: 10px;
  min-height: 0;
  flex: 1;
}

.utility-log-list,
.utility-error-list {
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #fbfdff;
  padding: 8px;
  min-height: 0;
  height: 100%;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.utility-log-item,
.utility-error-item {
  width: 100%;
  border: 1px solid #dbe6f7;
  border-radius: 9px;
  background: #ffffff;
  padding: 8px 9px;
  text-align: left;
  cursor: pointer;
}

.utility-log-item.is-active,
.utility-error-item.is-active {
  border-color: #93b8ed;
  background: #edf5ff;
}

.utility-log-item__head,
.utility-error-item__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.utility-log-item__head strong,
.utility-error-item__head strong {
  color: #2f3f5b;
  font-size: 12px;
}

.utility-log-item__meta {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.utility-log-kind {
  border-radius: 999px;
  padding: 1px 7px;
  font-size: 10px;
  font-weight: 700;
  color: #315f95;
  background: #eaf3ff;
}

.utility-log-kind[data-kind="tool"] {
  color: #8a4c07;
  background: #ffeecf;
}

.utility-error-item__head span {
  color: #6f84a5;
  font-size: 11px;
  font-weight: 700;
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

.utility-log-item p,
.utility-error-item p {
  margin: 5px 0 0;
  color: #2f3f5b;
  font-size: 12px;
  line-height: 1.35;
  word-break: break-all;
}

.utility-log-item small,
.utility-error-item small {
  display: block;
  margin-top: 4px;
  color: #7f90aa;
  font-size: 11px;
}

.utility-log-detail {
  border: 1px solid #e2eaf6;
  border-radius: 10px;
  background: #ffffff;
  padding: 10px 11px;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  gap: 8px;
}

.utility-log-detail__head {
  position: sticky;
  top: 0;
  z-index: 1;
  background: #ffffff;
  padding-bottom: 2px;
}

.utility-log-detail__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.utility-log-detail__header strong {
  color: #2f3f5b;
  font-size: 13px;
}

.utility-log-detail__header p {
  margin: 4px 0 0;
  color: #6f83a3;
  font-size: 11px;
}

.utility-log-detail__stats {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.utility-log-detail__stats span {
  border-radius: 999px;
  background: #eff4fc;
  color: #5a6d8d;
  font-size: 11px;
  padding: 2px 8px;
}

.utility-log-detail-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.utility-log-detail-tab {
  height: 28px;
  border: 1px solid #d8e3f4;
  border-radius: 8px;
  padding: 0 10px;
  background: #f8fbff;
  color: #6a7d9c;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.utility-log-detail-tab.is-active {
  border-color: #9dbdea;
  background: #eaf3ff;
  color: #345986;
}

.utility-log-detail__content-wrap {
  position: relative;
  margin: 0;
  border: 1px solid #e1e8f6;
  border-radius: 8px;
  background: #f8fbff;
  min-height: 0;
  flex: 1;
  overflow: hidden;
}

.utility-log-copy {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border: 1px solid #d8e3f4;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.9);
  color: #5e7598;
  display: grid;
  place-items: center;
  cursor: pointer;
  z-index: 2;
}

.utility-log-copy svg {
  width: 14px;
  height: 14px;
  fill: currentColor;
}

.utility-log-copy:hover {
  border-color: #b3caec;
  background: #edf5ff;
  color: #345986;
}

.utility-log-detail__content {
  margin: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  padding: 10px 38px 10px 10px;
  min-height: 0;
  height: 100%;
  overflow: auto;
  font-size: 12px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-word;
  user-select: text;
  -webkit-user-select: text;
}

@media (max-width: 1100px) {
  .chat-app {
    grid-template-columns: 186px 280px minmax(0, 1fr);
  }

  .module-board--dashboard {
    padding: 18px;
  }

  .dashboard-stats-grid {
    grid-template-columns: repeat(auto-fit, minmax(172px, 1fr));
  }

  .dashboard-status-grid {
    grid-template-columns: repeat(auto-fit, minmax(208px, 1fr));
  }

  .chat-window__content--settings-open {
    grid-template-columns: minmax(0, 1fr) 316px;
  }

  .related-memory-layout {
    grid-template-columns: 240px minmax(0, 1fr);
  }

  .proxy-config-layout {
    grid-template-columns: 230px minmax(0, 1fr);
  }

  .sidebar-settings-modal__body {
    grid-template-columns: 200px minmax(0, 1fr);
  }

  .task-board-creator {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .task-board-creator__button {
    grid-column: span 2;
  }

  .task-board-columns {
    grid-template-columns: repeat(5, minmax(220px, 1fr));
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
    padding: 10px 12px;
  }

  .composer-panel {
    border-radius: 16px;
    padding: 8px;
  }

  .chat-window__composer input {
    height: 42px;
  }

  .chat-empty-state h3 {
    font-size: 34px;
  }

  .chat-empty-state__actions {
    grid-template-columns: minmax(0, 1fr);
  }

  .chat-empty-action {
    min-height: 78px;
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

  .proxy-config-layout {
    grid-template-columns: minmax(0, 1fr);
    min-height: 0;
  }

  .proxy-config-nav-pane {
    max-height: 220px;
  }

  .related-memory-editor-pane__meta {
    grid-template-columns: 1fr;
  }

  .related-model-form__actions {
    flex-direction: column;
    align-items: flex-start;
  }

  .related-channel-row {
    align-items: flex-start;
    flex-direction: column;
  }

  .utility-modal {
    width: calc(100vw - 20px);
    max-height: calc(100vh - 20px);
  }

  .sidebar-settings-modal {
    width: calc(100vw - 20px);
    max-height: calc(100vh - 20px);
  }

  .sidebar-settings-modal__header {
    padding: 12px;
  }

  .sidebar-settings-modal__body {
    grid-template-columns: minmax(0, 1fr);
  }

  .sidebar-settings-nav {
    border-right: 0;
    border-bottom: 1px solid #e8eef8;
    max-height: 200px;
  }

  .sidebar-settings-panel {
    padding: 12px;
  }

  .sidebar-settings-card {
    flex-direction: column;
    align-items: stretch;
  }

  .sidebar-settings-select {
    width: 100%;
    min-width: 0;
  }

  .sidebar-settings-shortcut-item {
    flex-direction: column;
    align-items: flex-start;
  }

  .sidebar-settings-feedback__actions {
    flex-direction: column;
    align-items: flex-start;
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

  .utility-log-layout {
    grid-template-columns: minmax(0, 1fr);
    min-height: 0;
  }

  .utility-log-list,
  .utility-error-list {
    max-height: 220px;
  }

  .utility-log-detail {
    min-height: 280px;
  }

  .module-board__header--dashboard {
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    padding: 14px;
    border-radius: 15px;
  }

  .module-board--dashboard {
    padding: 12px;
  }

  .module-board--dashboard .module-board__header h2 {
    font-size: 24px;
  }

  .module-board--dashboard .module-board__header p {
    margin-top: 8px;
    font-size: 12px;
  }

  .dashboard-toolbar {
    width: 100%;
    justify-content: flex-start;
    gap: 8px;
  }

  .dashboard-section__header strong {
    font-size: 16px;
  }

  .dashboard-section {
    margin-top: 12px;
    border-radius: 15px;
  }

  .dashboard-section__header,
  .dashboard-activity-panel__header {
    padding: 12px 13px;
  }

  .dashboard-stats-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    padding: 10px;
  }

  .dashboard-status-grid {
    grid-template-columns: minmax(0, 1fr);
    padding: 10px;
  }

  .dashboard-metric-card,
  .dashboard-status-card {
    min-height: auto;
    padding: 11px 12px;
  }

  .dashboard-metric-card strong {
    font-size: 18px;
  }

  .dashboard-metric-card--compact strong {
    font-size: 16px;
  }

  .dashboard-status-card strong {
    font-size: 22px;
  }

  .dashboard-status-card p {
    -webkit-line-clamp: 4;
  }

  .dashboard-activity-item {
    grid-template-columns: 72px 52px minmax(0, 1fr);
    align-items: flex-start;
    gap: 7px;
    padding: 10px 12px;
  }

  .module-surface__toolbar {
    flex-wrap: wrap;
  }

  .module-surface__meta {
    white-space: normal;
  }

  .module-surface__select,
  .module-surface__button {
    flex: 1 1 auto;
  }

  .task-project-toolbar {
    flex-direction: column;
    align-items: stretch;
  }

  .task-project-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .task-board-topbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .task-board-creator {
    grid-template-columns: minmax(0, 1fr);
  }

  .task-board-creator__button {
    grid-column: auto;
  }

  .task-board-columns {
    grid-template-columns: repeat(5, minmax(220px, 1fr));
  }

  .recruitment-role-card__actions,
  .skill-market-card-v2__actions {
    width: 100%;
  }

  .recruitment-role-card__action,
  .skill-market-card-v2__action {
    flex: 1 1 auto;
    justify-content: center;
  }

  .skill-market-category-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .recruitment-role-grid,
  .skill-market-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .module-board {
    grid-column: 2;
  }

  .skill-market-detail-modal {
    width: min(560px, calc(100vw - 20px));
  }

  .skill-market-detail-modal__stats {
    grid-template-columns: minmax(0, 1fr);
  }

  .skill-market-detail-modal__actions {
    flex-wrap: wrap;
  }

  .role-workflow-detail-modal {
    width: min(940px, calc(100vw - 20px));
  }

  .role-workflow-detail-modal__editor {
    min-height: 260px;
  }

  .role-workflow-detail-modal__version-item {
    flex-wrap: wrap;
  }

  .role-workflow-detail-modal__actions {
    flex-wrap: wrap;
  }

  .role-workflow-detail-modal__actions button {
    flex: 1 1 auto;
  }
}
</style>
