<script setup lang="ts">
import { interpolate } from "remotion/no-react";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import actCuteRotationConfig from "../images/animate/act_cute_rotation/index.json";
import actCuteRotationSprite from "../images/animate/act_cute_rotation/sprite.png";
import chatTypingConfig from "../images/animate/chat_typing/index.json";
import chatTypingSprite from "../images/animate/chat_typing/sprite.png";
import haveMealConfig from "../images/animate/have_meal/index.json";
import haveMealSprite from "../images/animate/have_meal/sprite.png";
import sleepConfig from "../images/animate/sleep/index.json";
import sleepSprite from "../images/animate/sleep/sprite.png";
import smileAndBlinkConfig from "../images/animate/smile_and_blink/index.json";
import smileAndBlinkSprite from "../images/animate/smile_and_blink/sprite.png";
import smileBlinkConfig from "../images/animate/smile_blink/index.json";
import smileBlinkSprite from "../images/animate/smile_blink/sprite.png";
import stompFeetConfig from "../images/animate/stomp_feet/index.json";
import stompFeetSprite from "../images/animate/stomp_feet/sprite.png";
import stretchYawnConfig from "../images/animate/stretch_yawn_and_rub_your_eyes/index.json";
import stretchYawnSprite from "../images/animate/stretch_yawn_and_rub_your_eyes/sprite.png";
import thinkConfig from "../images/animate/think/index.json";
import thinkSprite from "../images/animate/think/sprite.png";
import applauseToCelebrateConfig from "../images/animate/applause_to_celebrate/index.json";
import applauseToCelebrateSprite from "../images/animate/applause_to_celebrate/sprite.png";
import confusionConfig from "../images/animate/confusion/index.json";
import confusionSprite from "../images/animate/confusion/sprite.png";
import rubYourEyesConfig from "../images/animate/rub_your_eyes/index.json";
import rubYourEyesSprite from "../images/animate/rub_your_eyes/sprite.png";
import stretchBodyConfig from "../images/animate/stretch_body/index.json";
import stretchBodySprite from "../images/animate/stretch_body/sprite.png";
import theBodyRisesAndFallsConfig from "../images/animate/the_body_rises_and_falls/index.json";
import theBodyRisesAndFallsSprite from "../images/animate/the_body_rises_and_falls/sprite.png";
import winkQuietlyConfig from "../images/animate/wink_quietly/index.json";
import winkQuietlySprite from "../images/animate/wink_quietly/sprite.png";
import feishuChannelIcon from "../images/channels/feishu.svg";
import telegramChannelIcon from "../images/channels/telegram.svg";
import discordChannelIcon from "../images/channels/discord.svg";
import whatsappChannelIcon from "../images/channels/whatsapp.svg";
import dingtalkChannelIcon from "../images/channels/dingtalk.svg";
import wecomChannelIcon from "../images/channels/wecom.svg";
import qqBotChannelIcon from "../images/channels/qq.svg";
import { usePetSound } from "../composables/usePetSound";
import { sendOpenClawChat, type OpenClawMessage } from "../services/openclaw";
import {
  fetchSkillTop50,
  fetchSkillsGlobal,
  fetchSkillsByCategory,
  type SkillMarketCategory,
  type SkillMarketSkill,
  type SkillMarketSortBy
} from "../services/skillsMarket";
import {
  appendRequestLog,
  normalizeBaseUrl,
  normalizeApiPath,
  clearRequestLogs,
  createPlatformDraft,
  deletePlatform,
  exportLogsAsJson,
  getPlatformPresets,
  loadActivePlatformId,
  loadPlatforms,
  loadRequestLogs,
  normalizePathPrefix,
  setActivePlatform,
  setPlatformEnabled,
  upsertPlatform,
  type PlatformConfig,
  type PlatformProtocol,
  type RequestLog
} from "../services/consoleData";
import { loadAgencyRosterZh, type AgencyRosterRole } from "../services/agencyRoster";
import { loadAgentDetailMarkdownZh } from "../services/agentDetail";

type Frame = {
  i: number;
  x: number;
  y: number;
  w: number;
  h: number;
  t: number;
};

type AnimationConfig = {
  version: string;
  frame_size: { w: number; h: number };
  sheet_size: { w: number; h: number };
  frames: Frame[];
};

type AnimationName =
  | "act_cute_rotation"
  | "applause_to_celebrate"
  | "chat_typing"
  | "confusion"
  | "have_meal"
  | "rub_your_eyes"
  | "sleep"
  | "smile_and_blink"
  | "smile_blink"
  | "stomp_feet"
  | "stretch_body"
  | "stretch_yawn_and_rub_your_eyes"
  | "the_body_rises_and_falls"
  | "think"
  | "wink_quietly";
type ConsoleSection =
  | "overview"
  | "platforms"
  | "staff"
  | "role-workflow"
  | "skill-market"
  | "channels"
  | "bindings"
  | "tasks";

type MessageChannelType = "feishu" | "telegram" | "discord" | "whatsapp" | "dingtalk" | "wecom" | "qqbot";

type MessageChannelField = {
  key: string;
  label: string;
  placeholder: string;
  required?: boolean;
  secret?: boolean;
  description?: string;
  envVar?: string;
};

type MessageChannelCatalogItem = {
  id: MessageChannelType;
  name: string;
  description: string;
  icon: string;
  plugin?: boolean;
  featured?: boolean;
  docsUrl?: string;
  instructions?: string[];
  fields?: MessageChannelField[];
};

type ChannelAccountSnapshotItem = {
  accountId: string;
  name: string;
  configured: boolean;
  status: "connected" | "connecting" | "disconnected" | "error" | string;
  isDefault: boolean;
  agentId?: string;
};

type ChannelGroupSnapshotItem = {
  channelType: string;
  defaultAccountId: string;
  status: "connected" | "connecting" | "disconnected" | "error" | string;
  accounts: ChannelAccountSnapshotItem[];
};

type ChannelAccountsSnapshotResponse = {
  sourcePath: string;
  detail: string;
  channels: ChannelGroupSnapshotItem[];
};

type ResourceModalKind = "memory" | "skill" | "tool";

type LogAnalysisView = "timeline" | "sessions" | "failures";
type PanelMode = "console" | "logs" | "subscriptions" | "lobster";
type SkillMarketSectionCategory = "top" | SkillMarketCategory;
type SkillMarketCategoryOption = {
  id: SkillMarketSectionCategory;
  label: string;
  icon: string;
  hint: string;
  apiCategory: SkillMarketCategory | null;
};
type SkillMarketListResultSnapshot = {
  skills: SkillMarketSkill[];
  total: number;
};

type CodingPlanRecommendation = {
  id: string;
  category: "cloud" | "model";
  name: string;
  summary: string;
  latest: string;
  highlights: string[];
  pricing: string;
  pricingNote?: string;
  platformUrl: string;
  accent: "amber" | "sky" | "mint" | "rose" | "default";
};

type SubscriptionScenarioRecommendation = {
  scene: string;
  primary: string;
  secondary: string;
  caution: string;
};

type SubscriptionPlatformBenchmark = {
  platform: string;
  firstResponse: string;
  outputSpeed: string;
  tokenUsage: string;
  simpleAccuracy: string;
  logicAccuracy: string;
  spatialAccuracy: string;
  stability: string;
  note: string;
};

type AnimationDefinition = {
  name: AnimationName;
  label: string;
  description: string;
  loop: boolean;
  sprite: string;
  config: AnimationConfig;
};

type ChatAttachment = {
  id: string;
  name: string;
  size: number;
  type: string;
  dataUrl: string;
};

type ChatMessage = {
  id: string;
  role: "assistant" | "user" | "system";
  text: string;
  status: "pending" | "done" | "error";
  createdAt?: number;
  attachments?: ChatAttachment[];
};

type BoundPetAgentCapability = {
  id: string;
  label: string;
  enabled: boolean;
};

type BoundPetConnection = {
  id: string;
  petName: string;
  ownerLabel: string;
  bindingCode: string;
  linkedAt: number;
  capabilities: BoundPetAgentCapability[];
};

type AudioFilePayload = {
  dataUrl: string;
  mimeType: string;
  fileName: string;
};

type PlatformDraft = {
  id: string;
  name: string;
  protocol: PlatformProtocol;
  baseUrl: string;
  pathPrefix: string;
  apiPath: string;
  apiKey: string;
  model: string;
  enabled: boolean;
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

type MemoryRecord = {
  id: string;
  title: string;
  owner: string;
  scope: string;
  summary: string;
  content: string;
  sourcePath: string;
  relativePath: string;
  updatedAt: number;
  exists: boolean;
};

type DocumentRecord = {
  id: string;
  title: string;
  category: string;
  owner: string;
  source: string;
  relativePath: string;
  summary: string;
  content: string;
  updatedAt: number;
  exists: boolean;
};

type MemoryDraft = {
  id: string;
  title: string;
  owner: string;
  scope: string;
  summary: string;
  content: string;
  sourcePath: string;
  relativePath: string;
};

type DocumentDraft = {
  id: string;
  title: string;
  category: string;
  owner: string;
  source: string;
  relativePath: string;
  summary: string;
  content: string;
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

/** OpenClaw 已安装技能项（来自 ~/.openclaw/skills），非文档编辑 */
type OpenClawSkillListItem = {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  relativePath: string;
  sourcePath: string;
};

/** OpenClaw 工具配置项（来自 tools.profile/allow/deny），非 TOOLS.md 编辑 */
type OpenClawToolListItem = {
  id: string;
  name: string;
  description: string;
  category: string;
  enabled: boolean;
};
type OpenClawSkillCategory = "builtIn" | "installed";

type OpenClawToolsScope = "agent" | "global";

type OpenClawToolsProfileOption = {
  value: string;
  label: string;
};

type TaskSnapshotItem = {
  id: string;
  name: string;
  agentId: string;
  sessionTarget: string;
  enabled: boolean;
  deleteAfterRun: boolean;
  statusKind: "scheduled" | "late" | "disabled" | string;
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

type LocalProxyPlatformPayload = {
  protocol: PlatformProtocol;
  baseUrl: string;
  pathPrefix: string;
  apiKey: string;
};

type OpenClawPlatformSnapshotItem = {
  id: string;
  providerId: string;
  name: string;
  protocol: PlatformProtocol;
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

type SessionSummary = {
  id: string;
  startedAt: number;
  lastAt: number;
  platformName: string;
  requestCount: number;
  failureCount: number;
  totalDuration: number;
  totalTokens: number;
  promptTokens: number;
  completionTokens: number;
  cacheReadInputTokens: number;
  logs: RequestLog[];
  previewText: string;
  fullOutput: string;
  latestError: string | null;
};

type FailureSummary = {
  key: string;
  title: string;
  count: number;
  latestAt: number;
  platformNames: string[];
  logs: RequestLog[];
  nextStep: string;
};

type GatewayMonitorState = {
  status: "checking" | "online" | "offline" | "unconfigured" | "unsupported";
  checkedUrl?: string | null;
  detail?: string | null;
  latencyMs?: number | null;
};

type LobsterActionId = "install" | "restart_gateway" | "auto_fix" | "backup" | "restore" | "upgrade";

type LobsterBackupItem = {
  name: string;
  path: string;
  createdAtMs: number;
  sizeBytes: number;
};

type LobsterSnapshotResponse = {
  openclawInstalled: boolean;
  openclawVersion: string | null;
  openclawBinary: string | null;
  openclawHome: string;
  backupDir: string;
  detail: string;
  backups: LobsterBackupItem[];
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

type LobsterInstallCheckStatus = "success" | "warning" | "failed" | "checking";

type LobsterInstallCheckItem = {
  id: string;
  title: string;
  status: LobsterInstallCheckStatus;
  detail: string;
};

type LobsterInstallGuideResponse = {
  os: string;
  ready: boolean;
  checks: LobsterInstallCheckItem[];
};

type RoleWorkflowOverride = {
  nameZh?: string;
  workflowZh?: string;
  detailContentZh?: string;
  detailVersions?: RoleWorkflowSavedVersion[];
};

type RoleWorkflowDetailDraft = {
  contentZh: string;
};

type RoleWorkflowSavedVersion = {
  id: string;
  contentZh: string;
  savedAt: number;
};

type RoleWorkflowInstallNotice = {
  tone: "success" | "error";
  text: string;
};

const LOBSTER_INSTALL_CHECK_BLUEPRINT: Array<Pick<LobsterInstallCheckItem, "id" | "title">> = [
  { id: "runtime", title: "Bundled OpenClaw 运行时" },
  { id: "nodejs", title: "Node.js 执行器" },
  { id: "openclaw-cli", title: "OpenClaw CLI 可执行性" },
  { id: "state-layout", title: "OpenClaw 状态目录" },
  { id: "plugins", title: "Bundled 插件镜像" },
  { id: "skills", title: "预装 Skills 资源" },
  { id: "config", title: "OpenClaw 配置目录" }
];

type LobsterInstallWizardStep = 1 | 2 | 3 | 4 | 5;
type LobsterInstallComponentStatus = "pending" | "installing" | "installed" | "failed";

type LobsterProviderOption = {
  id: string;
  name: string;
  icon?: string;
  group: "official" | "compatible" | "cn" | "local" | "custom";
  protocol: PlatformProtocol;
  defaultBaseUrl: string;
  defaultModelId: string;
  modelPlaceholder: string;
  apiKeyPlaceholder: string;
  docsUrl?: string;
  pathPrefix: string;
  requiresApiKey: boolean;
};
type LobsterProviderApiKind = "openai-completions" | "openai-responses" | "anthropic-messages";
type LobsterProviderApiKindOption = {
  value: LobsterProviderApiKind;
  label: string;
};

type AppLocale = "zh-CN" | "en-US" | "ja-JP";
type AppTheme = "light" | "dark" | "system";

type OpenClawMessageLogResponse = {
  detail: string;
  logs: RequestLog[];
};

function createEmptyMemoryDraft(): MemoryDraft {
  return {
    id: "",
    title: "",
    owner: "",
    scope: "",
    summary: "",
    content: "",
    sourcePath: "",
    relativePath: ""
  };
}

function createEmptyDocumentDraft(): DocumentDraft {
  return {
    id: "",
    title: "",
    category: "",
    owner: "",
    source: "",
    relativePath: "",
    summary: "",
    content: ""
  };
}

type PreviewSection = "request" | "response" | "stream" | "raw";

type JsonViewResult = {
  text: string;
  language: "json" | "text";
};

const animations: Record<AnimationName, AnimationDefinition> = {
  act_cute_rotation: {
    name: "act_cute_rotation",
    label: "转圈卖萌",
    description: "拖拽或空闲巡航时会转圈撒娇。",
    loop: true,
    sprite: actCuteRotationSprite,
    config: actCuteRotationConfig as AnimationConfig
  },
  applause_to_celebrate: {
    name: "applause_to_celebrate",
    label: "鼓掌庆祝",
    description: "任务完成或收到好消息时会兴奋鼓掌。",
    loop: false,
    sprite: applauseToCelebrateSprite,
    config: applauseToCelebrateConfig as AnimationConfig
  },
  chat_typing: {
    name: "chat_typing",
    label: "打字回复",
    description: "等待 OpenClaw 回复时保持输入状态。",
    loop: true,
    sprite: chatTypingSprite,
    config: chatTypingConfig as AnimationConfig
  },
  confusion: {
    name: "confusion",
    label: "困惑疑问",
    description: "遇到无法理解的指令时会露出困惑表情。",
    loop: false,
    sprite: confusionSprite,
    config: confusionConfig as AnimationConfig
  },
  have_meal: {
    name: "have_meal",
    label: "开心进食",
    description: "收到回复或被投喂灵感后会满足地吃一口。",
    loop: false,
    sprite: haveMealSprite,
    config: haveMealConfig as AnimationConfig
  },
  rub_your_eyes: {
    name: "rub_your_eyes",
    label: "揉眼睛",
    description: "长时间工作后会揉揉眼睛，提醒你也该休息了。",
    loop: false,
    sprite: rubYourEyesSprite,
    config: rubYourEyesConfig as AnimationConfig
  },
  sleep: {
    name: "sleep",
    label: "呼呼睡觉",
    description: "长时间无人打扰时进入睡眠陪伴。",
    loop: true,
    sprite: sleepSprite,
    config: sleepConfig as AnimationConfig
  },
  smile_and_blink: {
    name: "smile_and_blink",
    label: "贴贴微笑",
    description: "聊天面板打开时，保持专注又温柔的陪伴。",
    loop: true,
    sprite: smileAndBlinkSprite,
    config: smileAndBlinkConfig as AnimationConfig
  },
  smile_blink: {
    name: "smile_blink",
    label: "微笑眨眼",
    description: "默认待机，会持续微笑并轻轻眨眼。",
    loop: true,
    sprite: smileBlinkSprite,
    config: smileBlinkConfig as AnimationConfig
  },
  stomp_feet: {
    name: "stomp_feet",
    label: "跺脚抗议",
    description: "点击宠物后触发一次跺脚动作，然后回到待机。",
    loop: false,
    sprite: stompFeetSprite,
    config: stompFeetConfig as AnimationConfig
  },
  stretch_body: {
    name: "stretch_body",
    label: "伸展身体",
    description: "久坐之后会站起来伸展一下筋骨。",
    loop: false,
    sprite: stretchBodySprite,
    config: stretchBodyConfig as AnimationConfig
  },
  stretch_yawn_and_rub_your_eyes: {
    name: "stretch_yawn_and_rub_your_eyes",
    label: "伸懒腰醒神",
    description: "从睡眠中被唤醒时先打个哈欠，再恢复互动。",
    loop: false,
    sprite: stretchYawnSprite,
    config: stretchYawnConfig as AnimationConfig
  },
  the_body_rises_and_falls: {
    name: "the_body_rises_and_falls",
    label: "身体起伏",
    description: "安静待机时身体会轻轻起伏，像在呼吸一样。",
    loop: true,
    sprite: theBodyRisesAndFallsSprite,
    config: theBodyRisesAndFallsConfig as AnimationConfig
  },
  think: {
    name: "think",
    label: "歪头思考",
    description: "查看控制台或进入空闲思考时会露出思索表情。",
    loop: true,
    sprite: thinkSprite,
    config: thinkConfig as AnimationConfig
  },
  wink_quietly: {
    name: "wink_quietly",
    label: "安静眨眼",
    description: "静静地眨眨眼，适合安静陪伴的时刻。",
    loop: true,
    sprite: winkQuietlySprite,
    config: winkQuietlyConfig as AnimationConfig
  }
};

const actionTips: Record<AnimationName, string> = {
  act_cute_rotation: "被你拖起来后，它开始原地转圈卖萌。",
  applause_to_celebrate: "太棒了！它正在为你鼓掌庆祝呢。",
  chat_typing: "正在替你盯着回复，等 OpenClaw 把字打完。",
  confusion: "它有点困惑，歪着头不太理解刚才发生了什么。",
  have_meal: "像被投喂到一样，收到回应后会满足地吃一口。",
  rub_your_eyes: "它在揉眼睛，也许你也该休息一下了。",
  sleep: "",
  smile_and_blink: "聊天窗口打开时，它会保持更专注的陪伴表情。",
  smile_blink: "今天状态不错，适合放在页面右下角陪你工作。",
  stomp_feet: "你刚刚戳到它了，它正在跺脚表达情绪。",
  stretch_body: "坐太久了，它站起来伸展了一下身体。",
  stretch_yawn_and_rub_your_eyes: "刚被你叫醒，先伸个懒腰再继续营业。",
  the_body_rises_and_falls: "它在安静地呼吸，陪你度过平静的时光。",
  think: "它在认真琢磨眼前的信息，像在陪你一起排查。",
  wink_quietly: "它在安静地眨着眼睛，默默陪着你。"
};

const CHAT_STORAGE_PREFIX = "keai.desktop-pet.openclaw.chat-history";
const SESSION_STORAGE_PREFIX = "keai.desktop-pet.openclaw.session-id";
const PET_BIND_CODE_STORAGE_KEY = "keai.desktop-pet.binding.code";
const BOUND_PETS_STORAGE_KEY = "keai.desktop-pet.binding.peers";
const PLATFORM_PROXY_ENABLED_STORAGE_KEY = "keai.desktop-pet.platform-proxy-enabled";
const PLATFORM_DIRECT_BASEURL_STORAGE_KEY = "keai.desktop-pet.openclaw.platform-direct-baseurl";
const OPENCLAW_PROVIDER_DIRECT_BASEURL_PRESETS: Record<string, string> = {
  "coding-plan": "https://coding.dashscope.aliyuncs.com/v1"
};
const OPENCLAW_PROVIDER_PROXY_BASEURL_PRESETS: Record<string, string> = {
  "coding-plan": "http://localhost:3100/coding-plan"
};
const BOUND_PET_CHAT_PREFIX = "__bound_pet__:";
const DEFAULT_BOUND_CAPABILITIES: Array<Omit<BoundPetAgentCapability, "enabled">> = [
  { id: "__main__", label: "主对话 Agent" }
];
function chatStorageKeyFor(agentId: string | null) {
  return agentId ? `${CHAT_STORAGE_PREFIX}.${agentId}` : CHAT_STORAGE_PREFIX;
}
function sessionStorageKeyFor(agentId: string | null) {
  return agentId ? `${SESSION_STORAGE_PREFIX}.${agentId}` : SESSION_STORAGE_PREFIX;
}
function createDefaultChatMessages(): ChatMessage[] {
  return [
    {
      id: "welcome",
      role: "assistant",
      text: "点一下我就会展开 OpenClaw 对话框，回复会用文字气泡显示。",
      status: "done",
      createdAt: Date.now()
    }
  ];
}

const stage = ref<HTMLDivElement | null>(null);
const pet = ref<HTMLButtonElement | null>(null);
const sound = usePetSound();
const contextMenuRef = ref<HTMLDivElement | null>(null);
const consolePanelRef = ref<HTMLDivElement | null>(null);
const chatPanelRef = ref<HTMLDivElement | null>(null);
const messageScrollerRef = ref<HTMLDivElement | null>(null);
const currentAnimationName = ref<AnimationName>("smile_blink");
const currentFrameIndex = ref(0);
const petPosition = ref({ x: 0, y: 0 });
const isDragging = ref(false);
const dragDistance = ref(0);
const statusText = ref(actionTips.smile_blink);
const isWindowActive = ref(typeof document !== "undefined" ? document.hasFocus() : true);
const contextMenu = ref({ visible: false, x: 0, y: 0 });
const isChatOpen = ref(false);
const isConsoleOpen = ref(false);
const activePanelMode = ref<PanelMode>("console");
const activeSection = ref<ConsoleSection>("overview");
const activeLogAnalysisView = ref<LogAnalysisView>("timeline");
const isSending = ref(false);
const chatInput = ref("");
const chatMessages = ref<ChatMessage[]>(createDefaultChatMessages());
const chatAttachments = ref<ChatAttachment[]>([]);
const isDragOver = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
const activeChatAgentId = ref<string | null>(null);
const agentChatHistories = ref<Record<string, ChatMessage[]>>({});
const chatMotionValue = ref(0);
const panelMotionValue = ref(0);
const bubbleMotionValue = ref(1);
const localRequestLogs = ref<RequestLog[]>([]);
const runtimeRequestLogs = ref<RequestLog[]>([]);
const platforms = ref<PlatformConfig[]>([]);
const openClawProviderIdMap = ref<Record<string, string>>({});
const platformDirectBaseUrlMap = ref<Record<string, string>>(loadPlatformDirectBaseUrlMap());
const staffMembers = ref<StaffMemberSnapshot[]>([]);
const roleWorkflowKeyword = ref("");
const roleWorkflowOverrides = ref<Record<string, RoleWorkflowOverride>>(loadRoleWorkflowOverrides());
const roleWorkflowDetailRoleId = ref<string | null>(null);
const roleWorkflowDetailDraft = ref<RoleWorkflowDetailDraft>({ contentZh: "" });
const roleWorkflowDetailOriginalContent = ref("");
const roleWorkflowNameZhDraft = ref("");
const roleWorkflowNameZhOriginal = ref("");
const isRoleWorkflowInstalling = ref(false);
const roleWorkflowInstallNotice = ref<RoleWorkflowInstallNotice | null>(null);
const staffDeleteTargetMember = ref<StaffMemberSnapshot | null>(null);
const staffDeleteRemoveFiles = ref(false);
const isStaffDeleting = ref(false);
const staffDeleteError = ref("");
const channelGroups = ref<ChannelGroupSnapshotItem[]>([]);
const recentOutputModalMemberId = ref<string | null>(null);
const petBindingCode = ref("");
const bindingCodeDraft = ref("");
const incomingBindingCode = ref("");
const boundPets = ref<BoundPetConnection[]>([]);
const memoryRecords = ref<MemoryRecord[]>([]);
const documentRecords = ref<DocumentRecord[]>([]);
const openClawSkillRecords = ref<DocumentRecord[]>([]);
const resourceDocumentRecords = ref<DocumentRecord[]>([]);
/** OpenClaw 技能库：内置 + 安装（用于技能弹窗展示） */
const openClawSkillsList = ref<{ builtIn: OpenClawSkillListItem[]; installed: OpenClawSkillListItem[] }>({ builtIn: [], installed: [] });
const openClawSkillCategory = ref<OpenClawSkillCategory>("builtIn");
/** OpenClaw 当前员工的工具配置（用于工具弹窗展示，非 TOOLS.md 编辑） */
const openClawToolsList = ref<{ profile: string; profileLabel: string; tools: OpenClawToolListItem[] }>({ profile: "", profileLabel: "", tools: [] });
const openClawToolsScope = ref<OpenClawToolsScope>("agent");
const isOpenClawToolsSaving = ref(false);
const taskRecords = ref<TaskSnapshotItem[]>([]);
type CronTaskTab = "all" | "late" | "scheduled" | "disabled";
const cronTaskTab = ref<CronTaskTab>("all");
const cronTaskAgentFilter = ref<string>("all");
const activePlatformId = ref<string | null>(null);
const isEditingPlatform = ref(false);
const editingPlatformId = ref<string | null>(null);
const platformForm = ref<PlatformDraft>(createPlatformDraft());
const showPlatformTips = ref(false);
const isPlatformModalOpen = ref(false);
const isChannelConfigModalOpen = ref(false);
const channelConfigEditingType = ref<MessageChannelType | null>(null);
const channelConfigEditingAccountId = ref("");
const channelConfigAllowEditAccountId = ref(false);
const channelConfigExistingAccountIds = ref<string[]>([]);
const channelConfigForm = ref<Record<string, string>>({});
const channelConfigError = ref("");
const isChannelConfigSaving = ref(false);
const channelConfigSecretVisibility = ref<Record<string, boolean>>({});
const isSystemSettingsOpen = ref(false);

const OPENCLAW_TOOLS_PROFILE_PRESETS: OpenClawToolsProfileOption[] = [
  { value: "default", label: "Default（全量）" },
  { value: "minimal", label: "Minimal（最小权限）" },
  { value: "coding", label: "Coding（代码执行）" },
  { value: "messaging", label: "Messaging（消息会话）" }
];

type PetSizeLevel = "small" | "medium" | "large";
const PET_SIZE_MAP: Record<PetSizeLevel, number> = { small: 180, medium: 280, large: 380 };
const CONTEXT_MENU_VIEWPORT_MARGIN = 8;
const CONTEXT_MENU_FALLBACK_WIDTH = 208;
const CONTEXT_MENU_FALLBACK_HEIGHT = 224;
const agencyRosterDivisions = loadAgencyRosterZh();

function getSafeLocalStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function safeLocalStorageGetItem(key: string): string | null {
  try {
    return getSafeLocalStorage()?.getItem(key) ?? null;
  } catch {
    return null;
  }
}

function safeLocalStorageSetItem(key: string, value: string) {
  try {
    getSafeLocalStorage()?.setItem(key, value);
  } catch {
    // Ignore storage write failures in restricted environments.
  }
}

function normalizeRoleWorkflowSavedVersion(raw: unknown): RoleWorkflowSavedVersion | null {
  if (!raw || typeof raw !== "object") {
    return null;
  }
  const candidate = raw as Record<string, unknown>;
  const id = typeof candidate.id === "string" ? candidate.id.trim() : "";
  const contentZh = typeof candidate.contentZh === "string" ? candidate.contentZh : "";
  const savedAt = typeof candidate.savedAt === "number" && Number.isFinite(candidate.savedAt)
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
  const raw = safeLocalStorageGetItem("keai.desktop-pet.role-workflow-overrides");
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
  safeLocalStorageSetItem("keai.desktop-pet.role-workflow-overrides", JSON.stringify(roleWorkflowOverrides.value));
}

const APP_LOCALE_STORAGE_KEY = "keai.desktop-pet.locale";
const APP_LOCALE_OPTIONS: AppLocale[] = ["zh-CN", "en-US", "ja-JP"];
const APP_THEME_STORAGE_KEY = "keai.desktop-pet.theme";
const APP_THEME_OPTIONS: AppTheme[] = ["light", "dark", "system"];
const APP_I18N_MESSAGES: Record<AppLocale, Record<string, string>> = {
  "zh-CN": {
    "locale.zh-CN": "中文",
    "locale.en-US": "English",
    "locale.ja-JP": "日本語",
    "context.chat": "聊天",
    "context.platforms": "代理配置",
    "context.channels": "消息频道",
    "context.bindings": "宠物绑定",
    "context.lobster": "龙虾配置",
    "context.logs": "日志分析",
    "context.subscription": "订阅推荐",
    "context.system": "系统设置",
    "context.quit": "退出",
    "console.section.overview": "总览",
    "console.section.platforms": "代理配置",
    "console.section.staff": "员工管理",
    "console.section.role_workflow": "角色工作流",
    "console.section.skill_market": "技能市场",
    "console.section.channels": "消息频道",
    "console.section.bindings": "宠物绑定",
    "console.section.tasks": "任务管理",
    "lobster.action.install.title": "龙虾安装",
    "lobster.action.install.description": "参考 ClawPet 引导安装流程，优先检测环境，再执行安装。",
    "lobster.action.install.button": "开始安装",
    "lobster.action.restart_gateway.title": "重启网关",
    "lobster.action.restart_gateway.description": "优先执行 `openclaw gateway restart`，失败时自动回退 stop/start。",
    "lobster.action.restart_gateway.button": "立即重启",
    "lobster.action.auto_fix.title": "自动修复",
    "lobster.action.auto_fix.description": "执行 `openclaw doctor --fix --yes --non-interactive` 自动修复常见问题。",
    "lobster.action.auto_fix.button": "开始修复",
    "lobster.action.backup.title": "龙虾备份",
    "lobster.action.backup.description": "将当前 `~/.openclaw` 备份到独立目录，便于回滚。",
    "lobster.action.backup.button": "创建备份",
    "lobster.action.restore.title": "备份恢复",
    "lobster.action.restore.description": "从选中的备份恢复配置，恢复前会自动保留当前目录快照。",
    "lobster.action.restore.button": "执行恢复",
    "lobster.action.upgrade.title": "升级龙虾",
    "lobster.action.upgrade.description": "尝试使用 npm/pnpm/yarn 升级 OpenClaw 到最新版本。",
    "lobster.action.upgrade.button": "开始升级",
    "wizard.step.1.title": "欢迎使用 ClawPet",
    "wizard.step.1.description": "ClewPet 是 OpenClaw 的图形助手，让你养龙虾就像“抓娃娃”一样简单好玩！",
    "wizard.step.2.title": "环境检查",
    "wizard.step.2.description": "参照 ClawPet 先检查运行环境",
    "wizard.step.3.title": "AI 提供商",
    "wizard.step.3.description": "配置安装后的默认 AI 服务",
    "wizard.step.4.title": "设置中",
    "wizard.step.4.description": "安装必要组件",
    "wizard.step.5.title": "准备就绪！",
    "wizard.step.5.description": "ClawPet 已准备好使用",
    "wizard.welcome.title": "欢迎使用 ClawPet",
    "wizard.welcome.subtitle": "ClawPet 是 OpenClaw 的图形界面，将通过 1-5 步完成安装与配置。",
    "wizard.welcome.feature1": "无需命令行，可视化管理",
    "wizard.welcome.feature2": "现代化界面，极简体验",
    "wizard.welcome.feature3": "一键安装技能与插件",
    "wizard.welcome.feature4": "跨平台支持",
    "wizard.risk.title": "权限风险声明",
    "wizard.risk.body": "OpenClaw 不是普通聊天机器人，而是高权限 Agent，它可以：",
    "wizard.risk.item1": "读写你电脑上的文件",
    "wizard.risk.item2": "调用系统命令、控制浏览器",
    "wizard.risk.item3": "调用外部 API 或操作第三方软件（飞书 / QQ / Slack 等）",
    "wizard.risk.item4": "访问你的账号环境变量、密钥等敏感信息",
    "wizard.risk.item5": "使用大模型 API 并产生费用",
    "wizard.risk.accept": "我已了解并接受以上风险，继续开始配置。",
    "wizard.button.cancel": "取消",
    "wizard.button.back": "返回",
    "wizard.button.next": "下一步",
    "wizard.button.start_install": "开始安装",
    "wizard.button.finish": "完成",
    "wizard.button.get_started": "开始使用",
    "wizard.check.success": "通过",
    "wizard.check.warning": "注意",
    "wizard.check.failed": "失败",
    "wizard.runtime.title": "检查环境",
    "wizard.runtime.lead": "先确认运行条件，再进入安装步骤。",
    "wizard.runtime.checking": "检查中...",
    "wizard.runtime.recheck": "重新检查",
    "wizard.runtime.ready": "环境检查通过，可继续下一步。",
    "wizard.runtime.blocked": "仍有检查项未通过，请先修复后继续。",
    "wizard.runtime.check_count_unit": "项检查",
    "wizard.runtime.empty": "点击“重新检查”加载环境检查结果。",
    "wizard.provider.title": "AI 提供商",
    "wizard.provider.lead": "配置您的默认模型渠道，安装完成后可直接使用。",
    "wizard.provider.need_key": "需要 API 密钥",
    "wizard.provider.no_key": "支持免密钥接入",
    "wizard.provider.field.provider": "模型提供商",
    "wizard.provider.keep_current": "保持当前配置",
    "wizard.provider.field.model": "模型 ID",
    "wizard.provider.model_hint": "提供商的模型标识符（例如 deepseek-ai/DeepSeek-V3）",
    "wizard.provider.field.base_url": "API Base URL",
    "wizard.provider.field.api_key": "API 密钥",
    "wizard.provider.no_key_hint": "当前提供商支持 OAuth 或本地调用，可留空 API 密钥。",
    "wizard.provider.protocol": "协议",
    "wizard.provider.path_prefix": "路径前缀",
    "wizard.provider.docs": "查看接入文档",
    "wizard.provider.saving": "保存中...",
    "wizard.provider.saved": "已保存，可继续下一步",
    "wizard.provider.save": "验证并保存",
    "wizard.provider.save_simple": "保存",
    "wizard.provider.skip": "跳过设置",
    "wizard.provider.skip_done": "已跳过 AI 提供商设置，可继续下一步。",
    "wizard.provider.key_tip": "您的 API 密钥仅保存在本地机器。",
    "wizard.installing.title": "安装必要组件",
    "wizard.installing.subtitle": "正在设置 AI 助手所需的工具",
    "wizard.installing.progress": "进度",
    "wizard.installing.wait": "这可能需要一点时间...",
    "wizard.installing.status.pending": "等待中",
    "wizard.installing.status.installing": "安装中...",
    "wizard.installing.status.installed": "已安装",
    "wizard.installing.status.failed": "失败",
    "wizard.installing.component.opencode.name": "OpenCode",
    "wizard.installing.component.opencode.description": "AI 编程助手后端",
    "wizard.installing.component.python_env.name": "Python 环境",
    "wizard.installing.component.python_env.description": "技能所需的 Python 运行时",
    "wizard.installing.component.code_assist.name": "代码辅助",
    "wizard.installing.component.code_assist.description": "代码分析与建议",
    "wizard.installing.component.file_tools.name": "文件工具",
    "wizard.installing.component.file_tools.description": "文件操作与管理",
    "wizard.installing.component.terminal.name": "终端",
    "wizard.installing.component.terminal.description": "Shell 命令执行",
    "wizard.complete.title": "设置完成！",
    "wizard.complete.subtitle": "ClawPet 已配置并准备就绪。您现在可以开始与您的 AI 助手聊天了。",
    "wizard.complete.provider": "AI 提供商",
    "wizard.complete.components": "组件",
    "wizard.complete.gateway": "网关",
    "wizard.complete.gateway.running": "运行中",
    "wizard.complete.gateway.offline": "未运行",
    "wizard.complete.footer": "您可以在设置中自定义技能并连接渠道",
    "system.settings.title": "系统设置",
    "system.settings.general": "通用",
    "system.settings.theme": "主题",
    "system.settings.theme.light": "浅色",
    "system.settings.theme.dark": "深色",
    "system.settings.theme.system": "跟随系统",
    "system.settings.language": "界面语言",
    "system.settings.language.hint": "选择后会立即应用到整个软件界面。",
    "system.settings.startup": "开机自动启动",
    "system.settings.startup.hint": "登录系统后自动启动 ClawPet",
    "system.settings.startup.unsupported": "当前环境暂不支持自动启动设置。",
    "system.settings.pet_size": "宠物大小",
    "system.settings.window_behavior": "窗口行为",
    "system.settings.always_top": "始终置顶",
    "system.settings.always_top.hint": "宠物窗口保持在所有应用上方",
    "system.settings.cancel": "取消",
    "system.settings.save": "保存",
    "size.small": "小",
    "size.medium": "中",
    "size.large": "大",
    "status.settings_saved": "系统设置已保存。",
    "provider.group.official": "官方",
    "provider.group.compatible": "兼容平台",
    "provider.group.cn": "国内渠道",
    "provider.group.local": "本地模型",
    "provider.group.custom": "自定义渠道",
    "provider.option.custom": "自定义接入",
    "provider.protocol.openai": "OpenAI 兼容",
    "provider.protocol.anthropic": "Anthropic Messages",
    "provider.protocol.openai_completions": "OpenAI Completions",
    "provider.protocol.openai_responses": "OpenAI Responses",
    "provider.protocol.anthropic_compatible": "Anthropic 兼容"
  },
  "en-US": {
    "locale.zh-CN": "Chinese",
    "locale.en-US": "English",
    "locale.ja-JP": "Japanese",
    "context.chat": "Chat",
    "context.platforms": "Proxy Config",
    "context.channels": "Channels",
    "context.bindings": "Pet Binding",
    "context.lobster": "Lobster Config",
    "context.logs": "Log Analysis",
    "context.subscription": "Subscription Tips",
    "context.system": "System Settings",
    "context.quit": "Quit",
    "console.section.overview": "Overview",
    "console.section.platforms": "Proxy Config",
    "console.section.staff": "Staff",
    "console.section.role_workflow": "Role Workflows",
    "console.section.skill_market": "Skill Market",
    "console.section.channels": "Channels",
    "console.section.bindings": "Bindings",
    "console.section.tasks": "Tasks",
    "lobster.action.install.title": "Install Lobster",
    "lobster.action.install.description": "Follow the ClawPet guided flow: check environment first, then install.",
    "lobster.action.install.button": "Start Install",
    "lobster.action.restart_gateway.title": "Restart Gateway",
    "lobster.action.restart_gateway.description": "Run `openclaw gateway restart`; fallback to stop/start if needed.",
    "lobster.action.restart_gateway.button": "Restart Now",
    "lobster.action.auto_fix.title": "Auto Fix",
    "lobster.action.auto_fix.description": "Run `openclaw doctor --fix --yes --non-interactive` to fix common issues.",
    "lobster.action.auto_fix.button": "Start Fix",
    "lobster.action.backup.title": "Backup Lobster",
    "lobster.action.backup.description": "Backup current `~/.openclaw` for rollback.",
    "lobster.action.backup.button": "Create Backup",
    "lobster.action.restore.title": "Restore Backup",
    "lobster.action.restore.description": "Restore selected backup; a snapshot is created before restore.",
    "lobster.action.restore.button": "Restore",
    "lobster.action.upgrade.title": "Upgrade Lobster",
    "lobster.action.upgrade.description": "Try npm/pnpm/yarn to upgrade OpenClaw to latest.",
    "lobster.action.upgrade.button": "Start Upgrade",
    "wizard.step.1.title": "Welcome to ClawPet",
    "wizard.step.1.description": "ClewPet is OpenClaw’s visual helper, making lobster-raising as easy and fun as a claw machine!",
    "wizard.step.2.title": "Environment Check",
    "wizard.step.2.description": "Check runtime requirements before installation",
    "wizard.step.3.title": "AI Provider",
    "wizard.step.3.description": "Configure default AI service after installation",
    "wizard.step.4.title": "Setting Up",
    "wizard.step.4.description": "Installing essential components",
    "wizard.step.5.title": "Ready!",
    "wizard.step.5.description": "ClawPet is ready to use",
    "wizard.welcome.title": "Welcome to ClawPet",
    "wizard.welcome.subtitle": "ClawPet is a GUI for OpenClaw. We'll complete setup in 5 guided steps.",
    "wizard.welcome.feature1": "No command line needed, visual management",
    "wizard.welcome.feature2": "Modern UI with a minimal experience",
    "wizard.welcome.feature3": "One-click install for skills and plugins",
    "wizard.welcome.feature4": "Cross-platform support",
    "wizard.risk.title": "Permission Risk Notice",
    "wizard.risk.body": "OpenClaw is a high-privilege agent and can:",
    "wizard.risk.item1": "Read/write files on your computer",
    "wizard.risk.item2": "Run system commands and control browsers",
    "wizard.risk.item3": "Call external APIs or third-party apps",
    "wizard.risk.item4": "Access env vars and sensitive keys",
    "wizard.risk.item5": "Use LLM APIs and incur costs",
    "wizard.risk.accept": "I understand and accept the risks above.",
    "wizard.button.cancel": "Cancel",
    "wizard.button.back": "Back",
    "wizard.button.next": "Next",
    "wizard.button.start_install": "Start Install",
    "wizard.button.finish": "Finish",
    "wizard.button.get_started": "Get Started",
    "wizard.check.success": "Pass",
    "wizard.check.warning": "Warn",
    "wizard.check.failed": "Fail",
    "wizard.runtime.title": "Environment Check",
    "wizard.runtime.lead": "Confirm requirements before installation.",
    "wizard.runtime.checking": "Checking...",
    "wizard.runtime.recheck": "Re-check",
    "wizard.runtime.ready": "Environment is ready. You can continue.",
    "wizard.runtime.blocked": "Some checks are not passed yet.",
    "wizard.runtime.check_count_unit": "checks",
    "wizard.runtime.empty": "Click “Re-check” to load results.",
    "wizard.provider.title": "AI Provider",
    "wizard.provider.lead": "Set the default provider for immediate use.",
    "wizard.provider.need_key": "API key required",
    "wizard.provider.no_key": "Keyless supported",
    "wizard.provider.field.provider": "Provider",
    "wizard.provider.keep_current": "Keep current config",
    "wizard.provider.field.model": "Model ID",
    "wizard.provider.model_hint": "Provider model identifier (e.g. deepseek-ai/DeepSeek-V3)",
    "wizard.provider.field.base_url": "API Base URL",
    "wizard.provider.field.api_key": "API Key",
    "wizard.provider.no_key_hint": "This provider supports OAuth/local mode. API key can be empty.",
    "wizard.provider.protocol": "Protocol",
    "wizard.provider.path_prefix": "Path Prefix",
    "wizard.provider.docs": "View Docs",
    "wizard.provider.saving": "Saving...",
    "wizard.provider.saved": "Saved, continue to next step",
    "wizard.provider.save": "Validate & Save",
    "wizard.provider.save_simple": "Save",
    "wizard.provider.skip": "Skip Setup",
    "wizard.provider.skip_done": "AI provider setup skipped. You can continue.",
    "wizard.provider.key_tip": "API keys are stored locally only.",
    "wizard.installing.title": "Installing Essential Components",
    "wizard.installing.subtitle": "Setting up the tools required by your AI assistant",
    "wizard.installing.progress": "Progress",
    "wizard.installing.wait": "This may take a little while...",
    "wizard.installing.status.pending": "Pending",
    "wizard.installing.status.installing": "Installing...",
    "wizard.installing.status.installed": "Installed",
    "wizard.installing.status.failed": "Failed",
    "wizard.installing.component.opencode.name": "OpenCode",
    "wizard.installing.component.opencode.description": "Backend for AI coding assistant",
    "wizard.installing.component.python_env.name": "Python Runtime",
    "wizard.installing.component.python_env.description": "Python runtime required by skills",
    "wizard.installing.component.code_assist.name": "Code Assist",
    "wizard.installing.component.code_assist.description": "Code analysis and suggestions",
    "wizard.installing.component.file_tools.name": "File Tools",
    "wizard.installing.component.file_tools.description": "File operations and management",
    "wizard.installing.component.terminal.name": "Terminal",
    "wizard.installing.component.terminal.description": "Shell command execution",
    "wizard.complete.title": "Setup Complete!",
    "wizard.complete.subtitle": "ClawPet is configured and ready. You can start chatting with your AI assistant now.",
    "wizard.complete.provider": "AI Provider",
    "wizard.complete.components": "Components",
    "wizard.complete.gateway": "Gateway",
    "wizard.complete.gateway.running": "Running",
    "wizard.complete.gateway.offline": "Offline",
    "wizard.complete.footer": "You can customize skills and channels in Settings.",
    "system.settings.title": "System Settings",
    "system.settings.general": "General",
    "system.settings.theme": "Theme",
    "system.settings.theme.light": "Light",
    "system.settings.theme.dark": "Dark",
    "system.settings.theme.system": "Use System",
    "system.settings.language": "Language",
    "system.settings.language.hint": "Changes apply to the whole app immediately.",
    "system.settings.startup": "Launch on Login",
    "system.settings.startup.hint": "Start ClawPet automatically after system login",
    "system.settings.startup.unsupported": "Launch on login is not supported in this environment.",
    "system.settings.pet_size": "Pet Size",
    "system.settings.window_behavior": "Window Behavior",
    "system.settings.always_top": "Always On Top",
    "system.settings.always_top.hint": "Keep the pet window above all apps",
    "system.settings.cancel": "Cancel",
    "system.settings.save": "Save",
    "size.small": "Small",
    "size.medium": "Medium",
    "size.large": "Large",
    "status.settings_saved": "System settings saved.",
    "provider.group.official": "Official",
    "provider.group.compatible": "Compatible",
    "provider.group.cn": "China",
    "provider.group.local": "Local",
    "provider.group.custom": "Custom",
    "provider.option.custom": "Custom (Manual Setup)",
    "provider.protocol.openai": "OpenAI Compatible",
    "provider.protocol.anthropic": "Anthropic Messages",
    "provider.protocol.openai_completions": "OpenAI Completions",
    "provider.protocol.openai_responses": "OpenAI Responses",
    "provider.protocol.anthropic_compatible": "Anthropic Compatible"
  },
  "ja-JP": {
    "locale.zh-CN": "中国語",
    "locale.en-US": "英語",
    "locale.ja-JP": "日本語",
    "context.chat": "チャット",
    "context.platforms": "プロキシ設定",
    "context.channels": "メッセージチャンネル",
    "context.bindings": "ペット連携",
    "context.lobster": "ロブスター設定",
    "context.logs": "ログ分析",
    "context.subscription": "購読ガイド",
    "context.system": "システム設定",
    "context.quit": "終了",
    "console.section.overview": "概要",
    "console.section.platforms": "プロキシ設定",
    "console.section.staff": "スタッフ",
    "console.section.role_workflow": "ロールワークフロー",
    "console.section.skill_market": "スキルマーケット",
    "console.section.channels": "メッセージチャンネル",
    "console.section.bindings": "連携",
    "console.section.tasks": "タスク",
    "lobster.action.install.title": "ロブスターをインストール",
    "lobster.action.install.description": "ClawPet のガイド手順で、先に環境確認してからインストールします。",
    "lobster.action.install.button": "インストール開始",
    "lobster.action.restart_gateway.title": "ゲートウェイ再起動",
    "lobster.action.restart_gateway.description": "`openclaw gateway restart` を優先し、失敗時は stop/start にフォールバックします。",
    "lobster.action.restart_gateway.button": "今すぐ再起動",
    "lobster.action.auto_fix.title": "自動修復",
    "lobster.action.auto_fix.description": "`openclaw doctor --fix --yes --non-interactive` を実行して一般的な問題を修復します。",
    "lobster.action.auto_fix.button": "修復開始",
    "lobster.action.backup.title": "ロブスターをバックアップ",
    "lobster.action.backup.description": "現在の `~/.openclaw` をバックアップします。",
    "lobster.action.backup.button": "バックアップ作成",
    "lobster.action.restore.title": "バックアップ復元",
    "lobster.action.restore.description": "選択したバックアップを復元します（復元前にスナップショット作成）。",
    "lobster.action.restore.button": "復元実行",
    "lobster.action.upgrade.title": "ロブスターをアップグレード",
    "lobster.action.upgrade.description": "npm/pnpm/yarn で OpenClaw を最新化します。",
    "lobster.action.upgrade.button": "アップグレード開始",
    "wizard.step.1.title": "ClawPet へようこそ",
    "wizard.step.1.description": "ClewPet は OpenClaw のビジュアルアシスタント。ロブスター育成をクレーンゲームのように簡単で楽しくします！",
    "wizard.step.2.title": "環境チェック",
    "wizard.step.2.description": "インストール前に実行環境を確認します",
    "wizard.step.3.title": "AI プロバイダー",
    "wizard.step.3.description": "インストール後の既定 AI を設定します",
    "wizard.step.4.title": "セットアップ中",
    "wizard.step.4.description": "必要コンポーネントをインストール",
    "wizard.step.5.title": "準備完了！",
    "wizard.step.5.description": "ClawPet は利用準備ができました",
    "wizard.welcome.title": "ClawPet へようこそ",
    "wizard.welcome.subtitle": "ClawPet は OpenClaw の GUI です。5 ステップでセットアップします。",
    "wizard.welcome.feature1": "コマンドライン不要、可視化管理",
    "wizard.welcome.feature2": "モダンなUI、シンプル体験",
    "wizard.welcome.feature3": "スキルとプラグインをワンクリック導入",
    "wizard.welcome.feature4": "クロスプラットフォーム対応",
    "wizard.risk.title": "権限リスク通知",
    "wizard.risk.body": "OpenClaw は高権限エージェントであり、次の操作が可能です：",
    "wizard.risk.item1": "PC 上のファイルの読み書き",
    "wizard.risk.item2": "システムコマンド実行・ブラウザ制御",
    "wizard.risk.item3": "外部 API / サードパーティアプリ操作",
    "wizard.risk.item4": "環境変数や秘密鍵などへのアクセス",
    "wizard.risk.item5": "LLM API 利用による課金発生",
    "wizard.risk.accept": "上記リスクを理解し同意します。",
    "wizard.button.cancel": "キャンセル",
    "wizard.button.back": "戻る",
    "wizard.button.next": "次へ",
    "wizard.button.start_install": "インストール開始",
    "wizard.button.finish": "完了",
    "wizard.button.get_started": "開始する",
    "wizard.check.success": "正常",
    "wizard.check.warning": "注意",
    "wizard.check.failed": "失敗",
    "wizard.runtime.title": "環境チェック",
    "wizard.runtime.lead": "インストール前に実行条件を確認します。",
    "wizard.runtime.checking": "チェック中...",
    "wizard.runtime.recheck": "再チェック",
    "wizard.runtime.ready": "環境チェックに合格しました。次へ進めます。",
    "wizard.runtime.blocked": "未解決のチェック項目があります。",
    "wizard.runtime.check_count_unit": "項目",
    "wizard.runtime.empty": "「再チェック」を押して結果を読み込みます。",
    "wizard.provider.title": "AI プロバイダー",
    "wizard.provider.lead": "インストール後に使う既定のプロバイダーを設定します。",
    "wizard.provider.need_key": "API キー必須",
    "wizard.provider.no_key": "キーなし接続対応",
    "wizard.provider.field.provider": "プロバイダー",
    "wizard.provider.keep_current": "現在の設定を維持",
    "wizard.provider.field.model": "モデル ID",
    "wizard.provider.model_hint": "プロバイダーのモデル識別子（例: deepseek-ai/DeepSeek-V3）",
    "wizard.provider.field.base_url": "API Base URL",
    "wizard.provider.field.api_key": "API キー",
    "wizard.provider.no_key_hint": "このプロバイダーは OAuth / ローカル接続対応で、API キーは空でも可。",
    "wizard.provider.protocol": "プロトコル",
    "wizard.provider.path_prefix": "パス接頭辞",
    "wizard.provider.docs": "接続ドキュメント",
    "wizard.provider.saving": "保存中...",
    "wizard.provider.saved": "保存済み、次へ進めます",
    "wizard.provider.save": "検証して保存",
    "wizard.provider.save_simple": "保存",
    "wizard.provider.skip": "設定をスキップ",
    "wizard.provider.skip_done": "AI プロバイダー設定をスキップしました。次へ進めます。",
    "wizard.provider.key_tip": "API キーはローカル端末にのみ保存されます。",
    "wizard.installing.title": "必要コンポーネントをインストール",
    "wizard.installing.subtitle": "AI アシスタントに必要なツールをセットアップ中です",
    "wizard.installing.progress": "進捗",
    "wizard.installing.wait": "少し時間がかかる場合があります...",
    "wizard.installing.status.pending": "待機中",
    "wizard.installing.status.installing": "インストール中...",
    "wizard.installing.status.installed": "インストール済み",
    "wizard.installing.status.failed": "失敗",
    "wizard.installing.component.opencode.name": "OpenCode",
    "wizard.installing.component.opencode.description": "AI コーディングアシスタントのバックエンド",
    "wizard.installing.component.python_env.name": "Python 環境",
    "wizard.installing.component.python_env.description": "スキル実行に必要な Python ランタイム",
    "wizard.installing.component.code_assist.name": "コードアシスト",
    "wizard.installing.component.code_assist.description": "コード解析と提案",
    "wizard.installing.component.file_tools.name": "ファイルツール",
    "wizard.installing.component.file_tools.description": "ファイル操作と管理",
    "wizard.installing.component.terminal.name": "ターミナル",
    "wizard.installing.component.terminal.description": "Shell コマンド実行",
    "wizard.complete.title": "セットアップ完了！",
    "wizard.complete.subtitle": "ClawPet の設定が完了しました。今すぐ AI アシスタントとの会話を始められます。",
    "wizard.complete.provider": "AI プロバイダー",
    "wizard.complete.components": "コンポーネント",
    "wizard.complete.gateway": "ゲートウェイ",
    "wizard.complete.gateway.running": "稼働中",
    "wizard.complete.gateway.offline": "停止中",
    "wizard.complete.footer": "設定画面でスキルと接続チャネルをカスタマイズできます。",
    "system.settings.title": "システム設定",
    "system.settings.general": "一般",
    "system.settings.theme": "テーマ",
    "system.settings.theme.light": "ライト",
    "system.settings.theme.dark": "ダーク",
    "system.settings.theme.system": "システムに従う",
    "system.settings.language": "言語",
    "system.settings.language.hint": "変更はアプリ全体に即時反映されます。",
    "system.settings.startup": "ログイン時に起動",
    "system.settings.startup.hint": "システムログイン後に ClawPet を自動起動",
    "system.settings.startup.unsupported": "この環境では自動起動設定を利用できません。",
    "system.settings.pet_size": "ペットサイズ",
    "system.settings.window_behavior": "ウィンドウ動作",
    "system.settings.always_top": "常に前面表示",
    "system.settings.always_top.hint": "ペットウィンドウを常に最前面に保つ",
    "system.settings.cancel": "キャンセル",
    "system.settings.save": "保存",
    "size.small": "小",
    "size.medium": "中",
    "size.large": "大",
    "status.settings_saved": "システム設定を保存しました。",
    "provider.group.official": "公式",
    "provider.group.compatible": "互換プラットフォーム",
    "provider.group.cn": "中国向け",
    "provider.group.local": "ローカル",
    "provider.group.custom": "カスタム",
    "provider.option.custom": "カスタム接続",
    "provider.protocol.openai": "OpenAI 互換",
    "provider.protocol.anthropic": "Anthropic Messages",
    "provider.protocol.openai_completions": "OpenAI Completions",
    "provider.protocol.openai_responses": "OpenAI Responses",
    "provider.protocol.anthropic_compatible": "Anthropic 互換"
  }
};

function normalizeAppLocale(value: string | null): AppLocale {
  if (value === "en-US" || value === "ja-JP" || value === "zh-CN") {
    return value;
  }
  return "zh-CN";
}

function normalizeAppTheme(value: string | null): AppTheme {
  if (value === "light" || value === "dark" || value === "system") {
    return value;
  }
  return "system";
}

function resolveAppTheme(theme: AppTheme): "light" | "dark" {
  if (theme === "light" || theme === "dark") {
    return theme;
  }
  if (typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }
  return "light";
}

function tr(key: string) {
  const current = APP_I18N_MESSAGES[appLocale.value];
  return current[key] ?? APP_I18N_MESSAGES["zh-CN"][key] ?? key;
}

function setAppLocale(nextLocale: AppLocale, options?: { persist?: boolean }) {
  appLocale.value = nextLocale;
  if (options?.persist !== false) {
    safeLocalStorageSetItem(APP_LOCALE_STORAGE_KEY, nextLocale);
  }
}

function setAppTheme(nextTheme: AppTheme, options?: { persist?: boolean }) {
  appTheme.value = nextTheme;
  if (options?.persist !== false) {
    safeLocalStorageSetItem(APP_THEME_STORAGE_KEY, nextTheme);
  }

  if (typeof document === "undefined") {
    return;
  }

  const resolvedTheme = resolveAppTheme(nextTheme);
  const root = document.documentElement;
  root.dataset.appTheme = nextTheme;
  root.dataset.appThemeResolved = resolvedTheme;
  root.style.colorScheme = resolvedTheme;
}

function loadPetSizeLevel(): PetSizeLevel {
  const raw = safeLocalStorageGetItem("keai.desktop-pet.size-level");
  if (raw === "small" || raw === "medium" || raw === "large") return raw;
  return "medium";
}

function clampContextMenuPosition(x: number, y: number, width: number, height: number) {
  const maxX = Math.max(CONTEXT_MENU_VIEWPORT_MARGIN, window.innerWidth - width - CONTEXT_MENU_VIEWPORT_MARGIN);
  const maxY = Math.max(CONTEXT_MENU_VIEWPORT_MARGIN, window.innerHeight - height - CONTEXT_MENU_VIEWPORT_MARGIN);
  return {
    x: Math.min(Math.max(CONTEXT_MENU_VIEWPORT_MARGIN, x), maxX),
    y: Math.min(Math.max(CONTEXT_MENU_VIEWPORT_MARGIN, y), maxY)
  };
}

function adjustContextMenuToViewport() {
  if (!contextMenu.value.visible) {
    return;
  }

  const menuWidth = contextMenuRef.value?.offsetWidth ?? CONTEXT_MENU_FALLBACK_WIDTH;
  const menuHeight = contextMenuRef.value?.offsetHeight ?? CONTEXT_MENU_FALLBACK_HEIGHT;
  const next = clampContextMenuPosition(contextMenu.value.x, contextMenu.value.y, menuWidth, menuHeight);

  if (next.x === contextMenu.value.x && next.y === contextMenu.value.y) {
    return;
  }

  contextMenu.value = {
    ...contextMenu.value,
    ...next
  };
}
function loadAlwaysOnTop(): boolean {
  const raw = safeLocalStorageGetItem("keai.desktop-pet.always-on-top");
  return raw === "true";
}

function normalizeBindingCode(value: string) {
  const raw = value.toUpperCase().replace(/[^A-Z0-9]/g, "");
  return raw.replace(/(.{4})/g, "$1-").replace(/-$/, "").slice(0, 19);
}

function createBindingCode() {
  const raw = Math.random().toString(36).slice(2, 10).toUpperCase();
  return normalizeBindingCode(raw);
}

function loadPetBindingCode() {
  const stored = safeLocalStorageGetItem(PET_BIND_CODE_STORAGE_KEY);
  const normalized = stored ? normalizeBindingCode(stored) : "";
  return normalized || createBindingCode();
}

function createBoundPetCapabilities(seed?: BoundPetAgentCapability[]) {
  const presets: Array<Omit<BoundPetAgentCapability, "enabled">> = [
    ...DEFAULT_BOUND_CAPABILITIES,
    ...staffMembers.value.map((member) => ({
      id: member.agentId,
      label: `${stripRoleLabel(member.displayName)} Agent`
    }))
  ];
  const enabledMap = new Map((seed ?? []).map((item) => [item.id, item.enabled]));
  return presets.map((preset) => ({
    id: preset.id,
    label: preset.label,
    enabled: enabledMap.get(preset.id) ?? false
  }));
}

function normalizeBoundPetConnection(value: unknown): BoundPetConnection | null {
  const item = value as Partial<BoundPetConnection>;
  if (!item || typeof item.id !== "string" || typeof item.petName !== "string") {
    return null;
  }

  const bindingCode = typeof item.bindingCode === "string" ? normalizeBindingCode(item.bindingCode) : "";
  if (!bindingCode) {
    return null;
  }

  const linkedAt = typeof item.linkedAt === "number" ? item.linkedAt : Date.now();
  return {
    id: item.id,
    petName: item.petName,
    ownerLabel: typeof item.ownerLabel === "string" ? item.ownerLabel : "远程用户",
    bindingCode,
    linkedAt,
    capabilities: createBoundPetCapabilities(Array.isArray(item.capabilities) ? item.capabilities : [])
  };
}

function loadBoundPets() {
  const raw = safeLocalStorageGetItem(BOUND_PETS_STORAGE_KEY);
  if (!raw) {
    return [] as BoundPetConnection[];
  }
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [] as BoundPetConnection[];
    }
    return parsed
      .map((item) => normalizeBoundPetConnection(item))
      .filter((item): item is BoundPetConnection => item !== null);
  } catch {
    return [] as BoundPetConnection[];
  }
}

function persistBoundPets() {
  try {
    safeLocalStorageSetItem(BOUND_PETS_STORAGE_KEY, JSON.stringify(boundPets.value));
  } catch {
    // Ignore storage write failures.
  }
}

petBindingCode.value = loadPetBindingCode();
bindingCodeDraft.value = petBindingCode.value;
boundPets.value = loadBoundPets();
safeLocalStorageSetItem(PET_BIND_CODE_STORAGE_KEY, petBindingCode.value);

const petSizeLevel = ref<PetSizeLevel>(loadPetSizeLevel());
const petAlwaysOnTop = ref<boolean>(loadAlwaysOnTop());

// 系统设置弹窗草稿（弹窗打开时初始化，保存时才写入真实状态）
const draftSizeLevel = ref<PetSizeLevel>(petSizeLevel.value);
const draftAlwaysOnTop = ref<boolean>(petAlwaysOnTop.value);
const selectedPresetKey = ref("");
const selectedLogId = ref<string | null>(null);
const selectedSessionId = ref<string | null>(null);
const selectedSessionLogId = ref<string | null>(null);
const selectedFailureKey = ref<string | null>(null);
const timelinePreviewSection = ref<PreviewSection>("request");
const sessionPreviewSection = ref<PreviewSection>("response");
const sessionOverlayLogId = ref<string | null>(null);
const logFilterPlatform = ref<string | null>(null);
const logFilterAgent = ref<string | null>(null);
const currentSessionId = ref("");
const proxyPort = ref(5005);
const memoryDraft = ref<MemoryDraft>(createEmptyMemoryDraft());
const documentDraft = ref<DocumentDraft>(createEmptyDocumentDraft());
const selectedMemoryId = ref<string | null>(null);
const selectedDocumentId = ref<string | null>(null);
const memoryFilterText = ref("");
const documentFilterText = ref("");
const activeMemoryScope = ref("all");
const activeDocumentCategory = ref("all");
const activeResourceModal = ref<ResourceModalKind | null>(null);
const activeResourceMemberId = ref<string | null>(null);
const resourceModalFilterText = ref("");
const staffSnapshotDetail = ref("正在读取员工配置...");
const staffSnapshotSourcePath = ref("");
const staffMissionStatement = ref("构建可持续自治的 AI 员工体系，持续完成高价值任务。");
const isStaffSnapshotRefreshing = ref(false);
const taskSnapshotDetail = ref("正在读取任务调度...");
const taskSnapshotSourcePath = ref("");
const activeSkillMarketCategory = ref<SkillMarketSectionCategory>("top");
const skillMarketSortBy = ref<SkillMarketSortBy>("score");
const skillMarketSearch = ref("");
const skillMarketPage = ref(1);
const skillMarketPageSize = ref(30);
const skillMarketLoading = ref(false);
const skillMarketError = ref("");
const skillMarketTopSkills = ref<SkillMarketSkill[]>([]);
const skillMarketTopTotal = ref(0);
const skillMarketCategorySkills = ref<SkillMarketSkill[]>([]);
const skillMarketCategoryTotal = ref(0);
const skillMarketGlobalSkills = ref<SkillMarketSkill[]>([]);
const skillMarketGlobalTotal = ref(0);
const activeSkillMarketDetail = ref<SkillMarketSkill | null>(null);
const runtimeLogDetail = ref("正在读取 OpenClaw 运行时消息...");
const memorySnapshotDetail = ref("正在读取记忆文件...");
const memorySnapshotSourcePath = ref("");
const documentSnapshotDetail = ref("正在读取核心文档...");
const documentSnapshotSourcePath = ref("");
const gatewayMonitor = ref<GatewayMonitorState>({
  status: "checking",
  checkedUrl: null,
  detail: null,
  latencyMs: null
});
const lobsterSnapshot = ref<LobsterSnapshotResponse | null>(null);
const lobsterActionResult = ref<LobsterActionResult | null>(null);
const lobsterActionRunning = ref<LobsterActionId | null>(null);
const selectedLobsterBackupPath = ref<string | null>(null);
const isLobsterInstallWizardPrimed = ref(false);
const isLobsterInstallWizardOpen = ref(false);
const lobsterInstallWizardStep = ref<LobsterInstallWizardStep>(1);
const lobsterInstallGuide = ref<LobsterInstallGuideResponse | null>(null);
const lobsterInstallGuideLoading = ref(false);
const lobsterInstallRuntimeLogs = ref("");
const lobsterInstallRunning = ref(false);
const lobsterInstallFinishedResult = ref<LobsterActionResult | null>(null);
const lobsterInstallProgressValue = ref(0);
const lobsterInstallRiskAccepted = ref(false);
const appLocale = ref<AppLocale>(normalizeAppLocale(safeLocalStorageGetItem(APP_LOCALE_STORAGE_KEY)));
const appTheme = ref<AppTheme>(normalizeAppTheme(safeLocalStorageGetItem(APP_THEME_STORAGE_KEY)));
const draftAppLocale = ref<AppLocale>(appLocale.value);
const draftAppTheme = ref<AppTheme>(appTheme.value);
const systemSettingsPreviewBaseLocale = ref<AppLocale | null>(null);
const systemSettingsPreviewBaseTheme = ref<AppTheme | null>(null);
const launchOnLoginEnabled = ref(false);
const draftLaunchOnLoginEnabled = ref(false);
const launchOnLoginSupported = ref(false);
const lobsterProviderPresetKey = ref("");
const lobsterProviderForm = ref(createPlatformDraft());
const lobsterProviderConfigured = ref(false);
const lobsterProviderSaving = ref(false);
const lobsterProviderShowKey = ref(false);
const chatPlacement = ref({
  mode: "auto" as "auto" | "manual",
  x: 0,
  y: 0,
  width: 0,
  height: 0
});
const panelPlacement = ref({
  mode: "auto" as "auto" | "manual",
  x: 0,
  y: 0,
  width: 0,
  height: 0
});

const viewportSize = computed(() => PET_SIZE_MAP[petSizeLevel.value]);
const autoplayDelayMs = 9000;
const playbackRate = 3;
const sleepDelayMs = 24000;
const idleShowcaseSequence: AnimationName[] = ["think", "smile_and_blink", "have_meal", "act_cute_rotation", "wink_quietly", "the_body_rises_and_falls", "stretch_body", "rub_your_eyes", "confusion", "applause_to_celebrate"];
const platformPresets = getPlatformPresets();
const globalPlatformPresets = computed(() => platformPresets.filter((preset) => preset.region === "global"));
const chinaPlatformPresets = computed(() => platformPresets.filter((preset) => preset.region === "china"));
const openClawDefaultPlatformName = "OpenClaw 默认通道";
const messageChannelCatalog: MessageChannelCatalogItem[] = [
  {
    id: "feishu",
    name: "Feishu / Lark",
    description: "通过飞书官方推出的 OpenClaw 插件连接飞书/Lark 机器人",
    icon: feishuChannelIcon,
    plugin: true,
    featured: true,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/GKn8wOvHnibpPNkNkPzcAvGlnzK#Py88dTltfoJc1jxAhIBcW3Pkn7b",
    instructions: [
      "前往 飞书开放平台 (open.feishu.cn) 并创建企业自建应用",
      "在应用详情页获取 App ID 和 App Secret 并填入下方",
      "确保应用已开通“机器人”能力",
      "保存配置后，根据网关提示扫码完成机器人创建"
    ],
    fields: [
      { key: "appId", label: "应用 ID (App ID)", placeholder: "cli_xxxxxx", required: true, envVar: "FEISHU_APP_ID" },
      { key: "appSecret", label: "应用密钥 (App Secret)", placeholder: "输入应用密钥", required: true, secret: true, envVar: "FEISHU_APP_SECRET" }
    ]
  },
  {
    id: "telegram",
    name: "Telegram",
    description: "使用 @BotFather 提供的机器人令牌连接 Telegram",
    icon: telegramChannelIcon,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/TjiGwxsMWi7hpDkDAQBc0ydMnEf#PL8ndvsEwoYVWIx1T4mcB1EvnSb",
    instructions: [
      "打开 Telegram 并搜索 @BotFather",
      "发送 /newbot 并按照说明操作，复制机器人令牌",
      "从 @userinfobot 获取你的用户 ID",
      "将令牌和允许用户 ID 填入下方后保存"
    ],
    fields: [
      { key: "botToken", label: "机器人令牌", placeholder: "123456:ABC-DEF...", required: true, secret: true, envVar: "TELEGRAM_BOT_TOKEN" },
      { key: "allowedUsers", label: "允许的用户 ID", placeholder: "例如 123456789, 987654321", required: true }
    ]
  },
  {
    id: "discord",
    name: "Discord",
    description: "使用开发者门户提供的机器人令牌连接 Discord",
    icon: discordChannelIcon,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/BkOywJYCAiYRN9k4KTTceKPMnxg#C9zjdBRT1oqZ4VxF8q7ceRxQnLk",
    instructions: [
      "前往 Discord Developer Portal 创建应用并添加 Bot",
      "复制 Bot Token，并启用 Message Content Intent",
      "通过 OAuth2 URL Generator 生成邀请链接并拉机器人入群",
      "填入 Token，按需补充服务器 ID 和频道 ID 后保存"
    ],
    fields: [
      { key: "token", label: "机器人令牌", placeholder: "输入 Discord Bot Token", required: true, secret: true, envVar: "DISCORD_BOT_TOKEN" },
      { key: "guildId", label: "服务器 ID", placeholder: "例如 123456789012345678" },
      { key: "channelId", label: "频道 ID（可选）", placeholder: "例如 123456789012345678" }
    ]
  },
  {
    id: "whatsapp",
    name: "WhatsApp",
    description: "通过扫描二维码连接 WhatsApp（无需手机号）",
    icon: whatsappChannelIcon,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/ES7fwUfH8iGl8FkHYfFcyWB3n4d#doxcnsDDFbeEzqgjN6HSlyonLvc",
    instructions: [
      "保存配置后会自动触发二维码登录流程",
      "在手机上打开 WhatsApp，进入“已关联设备”",
      "扫描网关展示的二维码完成登录",
      "登录成功后即可在 OpenClaw 接收和发送消息"
    ],
    fields: []
  },
  {
    id: "dingtalk",
    name: "DingTalk",
    description: "通过 OpenClaw 渠道插件连接钉钉（Stream 模式）",
    icon: dingtalkChannelIcon,
    plugin: true,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/Y5eNwiSiZidkLskrwtJc1rUln0b#doxcnr8KfaA2mNPeQUeHO83eDPh",
    instructions: [
      "先安装并启用 dingtalk 插件",
      "在钉钉开发者后台创建企业内部应用并开启 Stream 模式",
      "填写 Client ID 与 Client Secret（必填）",
      "Robot Code / Corp ID / Agent ID 按需填写"
    ],
    fields: [
      { key: "clientId", label: "Client ID (AppKey)", placeholder: "dingxxxxxx", required: true },
      { key: "clientSecret", label: "Client Secret (AppSecret)", placeholder: "输入应用密钥", required: true, secret: true },
      { key: "robotCode", label: "Robot Code（可选）", placeholder: "通常与 Client ID 相同" },
      { key: "corpId", label: "Corp ID（可选）", placeholder: "dingxxxxxx" },
      { key: "agentId", label: "Agent ID（可选）", placeholder: "123456789" }
    ]
  },
  {
    id: "wecom",
    name: "WeCom",
    description: "通过插件连接企业微信机器人",
    icon: wecomChannelIcon,
    plugin: true,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/JTGnwoV0RixKPtkr4w7c7gpAnDc",
    instructions: [
      "在企业微信管理后台创建应用并获取配置",
      "确保已启用接收消息服务器配置",
      "填写 Bot ID 和 Secret 后保存"
    ],
    fields: [
      { key: "botId", label: "机器人 Bot ID", placeholder: "ww_xxxxxx", required: true },
      { key: "secret", label: "应用 Secret", placeholder: "输入企业微信 Secret", required: true, secret: true }
    ]
  },
  {
    id: "qqbot",
    name: "QQ Bot",
    description: "通过 @sliverp/qqbot 插件连接 QQ 机器人",
    icon: qqBotChannelIcon,
    plugin: true,
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/KPIJwlyiGiupMrkiS9ice39Zn2c",
    instructions: [
      "前往 QQ 机器人开放平台创建应用",
      "获取 App ID 与 Client Secret",
      "填写凭证后保存并连接"
    ],
    fields: [
      { key: "appId", label: "App ID", placeholder: "输入 QQ 机器人 App ID", required: true },
      { key: "clientSecret", label: "Client Secret", placeholder: "输入 Client Secret", required: true, secret: true }
    ]
  }
];
const MESSAGE_CHANNEL_ALIASES: Record<string, MessageChannelType> = {
  feishu: "feishu",
  lark: "feishu",
  telegram: "telegram",
  discord: "discord",
  whatsapp: "whatsapp",
  dingtalk: "dingtalk",
  wecom: "wecom",
  qq: "qqbot",
  qqbot: "qqbot",
  "qq bot": "qqbot"
};
const skillMarketCategories: SkillMarketCategoryOption[] = [
  { id: "top", label: "精选50Top", icon: "50", hint: "精选榜单", apiCategory: null },
  { id: "ai-intelligence", label: "AI 智能", icon: "AI", hint: "智能能力", apiCategory: "ai-intelligence" },
  { id: "developer-tools", label: "开发工具", icon: "</>", hint: "工程研发", apiCategory: "developer-tools" },
  { id: "productivity", label: "效率提升", icon: "⚡", hint: "个人效率", apiCategory: "productivity" },
  { id: "data-analytics", label: "数据分析", icon: "图", hint: "洞察分析", apiCategory: "data-analytics" },
  { id: "content-creation", label: "内容创作", icon: "创", hint: "文案视频", apiCategory: "content-creation" },
  { id: "security-compliance", label: "安全合规", icon: "盾", hint: "安全审计", apiCategory: "security-compliance" },
  {
    id: "communication-collaboration",
    label: "通讯协作",
    icon: "聊",
    hint: "团队协作",
    apiCategory: "communication-collaboration"
  }
];
const skillMarketSortOptions: Array<{ id: SkillMarketSortBy; label: string }> = [
  { id: "score", label: "综合排序" },
  { id: "downloads", label: "下载量" },
  { id: "stars", label: "收藏量" }
];
const skillMarketCategoryMap = new Map<SkillMarketSectionCategory, SkillMarketCategoryOption>(
  skillMarketCategories.map((item) => [item.id, item])
);
const skillMarketCache = new Map<string, SkillMarketListResultSnapshot>();
const skillMarketGlobalCache = new Map<string, SkillMarketListResultSnapshot>();
let skillMarketRequestToken = 0;
const visibleMessageChannels = computed(() => messageChannelCatalog);
const configuredMessageChannelIds = computed(() => {
  const configured = new Set<MessageChannelType>();
  for (const group of channelGroups.value) {
    const normalized = normalizeMessageChannelType(group.channelType);
    if (normalized) {
      configured.add(normalized);
    }
  }
  return configured;
});
const channelGroupsByType = computed(() =>
  Object.fromEntries(channelGroups.value.map((group) => [group.channelType.trim().toLowerCase(), group]))
);
const activeChannelConfigMeta = computed(
  () => messageChannelCatalog.find((item) => item.id === channelConfigEditingType.value) ?? null
);
const activeSkillMarketCategoryMeta = computed(
  () => skillMarketCategoryMap.get(activeSkillMarketCategory.value) ?? skillMarketCategories[0]
);
const skillMarketSourceSkills = computed(() =>
  activeSkillMarketCategory.value === "top" ? skillMarketTopSkills.value : skillMarketCategorySkills.value
);
const skillMarketBaseSkills = computed(() =>
  skillMarketSearch.value.trim() ? skillMarketGlobalSkills.value : skillMarketSourceSkills.value
);
const skillMarketTotal = computed(() =>
  activeSkillMarketCategory.value === "top" ? skillMarketTopTotal.value : skillMarketCategoryTotal.value
);
const filteredSkillMarketSkills = computed(() => {
  const keyword = skillMarketSearch.value.trim().toLowerCase();
  const records = skillMarketBaseSkills.value;
  const matched = keyword
    ? records.filter((item) => {
      const searchBlob = [
        item.name,
        item.description,
        item.descriptionZh,
        item.ownerName,
        item.category,
        item.tags.join(" ")
      ].join(" ").toLowerCase();
      return searchBlob.includes(keyword);
    })
    : records.slice();

  const sortBy = skillMarketSortBy.value;
  matched.sort((left, right) => {
    if (sortBy === "downloads") {
      return right.downloads - left.downloads;
    }
    if (sortBy === "stars") {
      return right.stars - left.stars;
    }
    return right.score - left.score;
  });
  return matched;
});
const skillMarketLocalTotal = computed(() => filteredSkillMarketSkills.value.length);
const skillMarketRemoteTotalPages = computed(() => {
  const size = Math.max(skillMarketPageSize.value, 1);
  const total = Math.max(skillMarketSearch.value.trim() ? skillMarketGlobalTotal.value : skillMarketTotal.value, 0);
  return Math.max(1, Math.ceil(total / size));
});
const skillMarketCurrentTotalPages = computed(() => {
  const size = Math.max(skillMarketPageSize.value, 1);
  if (activeSkillMarketCategory.value === "top" || skillMarketSearch.value.trim()) {
    return Math.max(1, Math.ceil(skillMarketLocalTotal.value / size));
  }
  return skillMarketRemoteTotalPages.value;
});
const pagedSkillMarketSkills = computed(() => {
  if (activeSkillMarketCategory.value !== "top" && !skillMarketSearch.value.trim()) {
    return filteredSkillMarketSkills.value;
  }
  const page = Math.max(1, skillMarketPage.value);
  const size = Math.max(1, skillMarketPageSize.value);
  const start = (page - 1) * size;
  return filteredSkillMarketSkills.value.slice(start, start + size);
});
const skillMarketPageNumbers = computed(() => {
  const total = skillMarketCurrentTotalPages.value;
  if (total <= 7) {
    return Array.from({ length: total }, (_, index) => index + 1);
  }
  const current = Math.min(Math.max(skillMarketPage.value, 1), total);
  let start = Math.max(1, current - 3);
  let end = start + 6;
  if (end > total) {
    end = total;
    start = end - 6;
  }
  const pages: number[] = [];
  for (let page = start; page <= end; page += 1) {
    pages.push(page);
  }
  return pages;
});
const skillMarketCanPrevPage = computed(() => skillMarketPage.value > 1);
const skillMarketCanNextPage = computed(() => skillMarketPage.value < skillMarketCurrentTotalPages.value);
const skillMarketSummaryText = computed(() => {
  const category = activeSkillMarketCategoryMeta.value.label;
  const displayed = pagedSkillMarketSkills.value.length;
  const isTop = activeSkillMarketCategory.value === "top";
  const isSearching = Boolean(skillMarketSearch.value.trim());
  const total = isTop || isSearching
    ? skillMarketLocalTotal.value
    : skillMarketTotal.value || skillMarketSourceSkills.value.length;
  if (isSearching) {
    return `分类：${category} · 命中 ${total} 条 · 第 ${skillMarketPage.value}/${skillMarketCurrentTotalPages.value} 页`;
  }
  return `分类：${category} · 展示 ${displayed} / ${total} · 第 ${skillMarketPage.value}/${skillMarketCurrentTotalPages.value} 页`;
});
const roleWorkflowNormalizedKeyword = computed(() => roleWorkflowKeyword.value.trim().toLowerCase());
const roleWorkflowTotalCount = computed(() => agencyRosterDivisions.reduce((sum, division) => sum + division.count, 0));
const roleWorkflowRoleIndex = computed(
  () =>
    new Map<
      string,
      {
        role: AgencyRosterRole;
        divisionTitleZh: string;
        divisionTitleEn: string;
        groupTitleZh: string | null;
        groupTitleEn: string | null;
      }
    >(
      agencyRosterDivisions.flatMap((division) =>
        division.groups.flatMap((group) =>
          group.roles.map((role) => [
            role.id,
            {
              role,
              divisionTitleZh: division.titleZh,
              divisionTitleEn: division.titleEn,
              groupTitleZh: group.titleZh,
              groupTitleEn: group.titleEn
            }
          ] as const)
        )
      )
    )
);
const activeRoleWorkflowBase = computed(() => {
  if (!roleWorkflowDetailRoleId.value) {
    return null;
  }
  return roleWorkflowRoleIndex.value.get(roleWorkflowDetailRoleId.value) ?? null;
});
const activeRoleWorkflowOverride = computed(() => {
  if (!roleWorkflowDetailRoleId.value) {
    return null;
  }
  return roleWorkflowOverrides.value[roleWorkflowDetailRoleId.value] ?? null;
});
const roleWorkflowDetailSavedVersions = computed(() => {
  return activeRoleWorkflowOverride.value?.detailVersions ?? [];
});
const isRoleWorkflowDraftChanged = computed(() => {
  if (!roleWorkflowDetailRoleId.value) {
    return false;
  }
  return (
    roleWorkflowDetailDraft.value.contentZh !== roleWorkflowDetailOriginalContent.value ||
    roleWorkflowNameZhDraft.value.trim() !== roleWorkflowNameZhOriginal.value.trim()
  );
});
const roleWorkflowDivisions = computed(() => {
  const keyword = roleWorkflowNormalizedKeyword.value;

  return agencyRosterDivisions
    .map((division) => {
      const groups = division.groups
        .map((group) => {
          const roles = group.roles
            .map((role) => {
              const override = roleWorkflowOverrides.value[role.id];
              return {
                ...role,
                nameZh: override?.nameZh ?? role.nameZh,
                workflowZh: override?.workflowZh ?? role.workflowZh
              };
            })
            .filter((role) => {
              if (!keyword) {
                return true;
              }
              const searchBlob = [
                role.nameZh,
                role.nameEn,
                role.workflowZh,
                division.titleZh,
                division.titleEn,
                group.titleZh ?? "",
                group.titleEn ?? "",
                role.sourcePath
              ]
                .join(" ")
                .toLowerCase();
              return searchBlob.includes(keyword);
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
const roleWorkflowVisibleCount = computed(() =>
  roleWorkflowDivisions.value.reduce((sum, division) => sum + division.count, 0)
);
const consoleSections = computed<Array<{ id: ConsoleSection; label: string }>>(() => [
  { id: "overview", label: tr("console.section.overview") },
  { id: "platforms", label: tr("console.section.platforms") },
  { id: "staff", label: tr("console.section.staff") },
  { id: "role-workflow", label: tr("console.section.role_workflow") },
  { id: "skill-market", label: tr("console.section.skill_market") },
  { id: "channels", label: tr("console.section.channels") },
  { id: "bindings", label: tr("console.section.bindings") },
  { id: "tasks", label: tr("console.section.tasks") }
]);
const lobsterActionCards = computed<Array<{ id: LobsterActionId; title: string; description: string; buttonLabel: string; danger?: boolean }>>(() => [
  {
    id: "install",
    title: tr("lobster.action.install.title"),
    description: tr("lobster.action.install.description"),
    buttonLabel: tr("lobster.action.install.button")
  },
  {
    id: "restart_gateway",
    title: tr("lobster.action.restart_gateway.title"),
    description: tr("lobster.action.restart_gateway.description"),
    buttonLabel: tr("lobster.action.restart_gateway.button")
  },
  {
    id: "auto_fix",
    title: tr("lobster.action.auto_fix.title"),
    description: tr("lobster.action.auto_fix.description"),
    buttonLabel: tr("lobster.action.auto_fix.button")
  },
  {
    id: "backup",
    title: tr("lobster.action.backup.title"),
    description: tr("lobster.action.backup.description"),
    buttonLabel: tr("lobster.action.backup.button")
  },
  {
    id: "restore",
    title: tr("lobster.action.restore.title"),
    description: tr("lobster.action.restore.description"),
    buttonLabel: tr("lobster.action.restore.button"),
    danger: true
  },
  {
    id: "upgrade",
    title: tr("lobster.action.upgrade.title"),
    description: tr("lobster.action.upgrade.description"),
    buttonLabel: tr("lobster.action.upgrade.button")
  }
]);
const lobsterInstallWizardSteps = computed<Array<{ id: LobsterInstallWizardStep; title: string; description: string }>>(() => [
  { id: 1, title: tr("wizard.step.1.title"), description: tr("wizard.step.1.description") },
  { id: 2, title: tr("wizard.step.2.title"), description: tr("wizard.step.2.description") },
  { id: 3, title: tr("wizard.step.3.title"), description: tr("wizard.step.3.description") },
  { id: 4, title: tr("wizard.step.4.title"), description: tr("wizard.step.4.description") },
  { id: 5, title: tr("wizard.step.5.title"), description: tr("wizard.step.5.description") }
]);
const lobsterInstallComponentItems = computed<Array<{ id: string; name: string; description: string }>>(() => [
  {
    id: "opencode",
    name: tr("wizard.installing.component.opencode.name"),
    description: tr("wizard.installing.component.opencode.description")
  },
  {
    id: "python_env",
    name: tr("wizard.installing.component.python_env.name"),
    description: tr("wizard.installing.component.python_env.description")
  },
  {
    id: "code_assist",
    name: tr("wizard.installing.component.code_assist.name"),
    description: tr("wizard.installing.component.code_assist.description")
  },
  {
    id: "file_tools",
    name: tr("wizard.installing.component.file_tools.name"),
    description: tr("wizard.installing.component.file_tools.description")
  },
  {
    id: "terminal",
    name: tr("wizard.installing.component.terminal.name"),
    description: tr("wizard.installing.component.terminal.description")
  }
]);
const lobsterInstallComponentStatus = computed<LobsterInstallComponentStatus>(() => {
  if (lobsterInstallRunning.value) {
    return "installing";
  }
  if (!lobsterInstallFinishedResult.value) {
    return "pending";
  }
  return lobsterInstallFinishedResult.value.success ? "installed" : "failed";
});
const lobsterInstallComponentStates = computed(() =>
  lobsterInstallComponentItems.value.map((item) => ({
    ...item,
    status: lobsterInstallComponentStatus.value
  }))
);
const lobsterInstallProgressDisplay = computed(() => {
  if (lobsterInstallFinishedResult.value) {
    return 100;
  }
  if (lobsterInstallRunning.value) {
    return Math.max(10, Math.min(95, Math.round(lobsterInstallProgressValue.value)));
  }
  return 0;
});
const lobsterCompleteProviderSummary = computed(() => {
  if (lobsterSelectedProviderOption.value) {
    return getLobsterProviderOptionLabel(lobsterSelectedProviderOption.value);
  }
  const fallbackName = lobsterProviderForm.value.name.trim();
  return fallbackName || tr("provider.option.custom");
});
const lobsterCompleteComponentsSummary = computed(() => {
  const separator = appLocale.value === "en-US" ? ", " : "、";
  return lobsterInstallComponentItems.value.map((item) => item.name).join(separator);
});
const lobsterCompleteGatewayRunning = computed(() => gatewayMonitor.value.status === "online");
const lobsterCompleteGatewaySummary = computed(() =>
  lobsterCompleteGatewayRunning.value ? `✓ ${tr("wizard.complete.gateway.running")}` : tr("wizard.complete.gateway.offline")
);
const lobsterProviderOptions: LobsterProviderOption[] = [
  {
    id: "anthropic",
    name: "Anthropic",
    icon: "🤖",
    group: "official",
    protocol: "anthropic",
    defaultBaseUrl: "https://api.anthropic.com",
    defaultModelId: "claude-sonnet-4-5",
    modelPlaceholder: "claude-sonnet-4-5",
    apiKeyPlaceholder: "sk-ant-api03-...",
    docsUrl: "https://platform.claude.com/docs/en/api/overview",
    pathPrefix: "/anthropic",
    requiresApiKey: true
  },
  {
    id: "openai",
    name: "OpenAI",
    icon: "💚",
    group: "official",
    protocol: "openai",
    defaultBaseUrl: "https://api.openai.com",
    defaultModelId: "gpt-5.4",
    modelPlaceholder: "gpt-5.4",
    apiKeyPlaceholder: "sk-proj-...",
    docsUrl: "https://platform.openai.com/api-keys",
    pathPrefix: "/openai",
    requiresApiKey: true
  },
  {
    id: "google",
    name: "Google",
    icon: "🔷",
    group: "official",
    protocol: "openai",
    defaultBaseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
    defaultModelId: "gemini-3.1-pro-preview",
    modelPlaceholder: "gemini-3.1-pro-preview",
    apiKeyPlaceholder: "AIza...",
    docsUrl: "https://aistudio.google.com/app/apikey",
    pathPrefix: "/google",
    requiresApiKey: true
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    icon: "🌐",
    group: "compatible",
    protocol: "openai",
    defaultBaseUrl: "https://openrouter.ai/api",
    defaultModelId: "openai/gpt-5.4",
    modelPlaceholder: "openai/gpt-5.4",
    apiKeyPlaceholder: "sk-or-v1-...",
    docsUrl: "https://openrouter.ai/models",
    pathPrefix: "/openrouter",
    requiresApiKey: true
  },
  {
    id: "ark",
    name: "ByteDance Ark",
    icon: "A",
    group: "cn",
    protocol: "openai",
    defaultBaseUrl: "https://ark.cn-beijing.volces.com/api/v3",
    defaultModelId: "ep-20260228000000-xxxxx",
    modelPlaceholder: "ep-20260228000000-xxxxx",
    apiKeyPlaceholder: "your-ark-api-key",
    docsUrl: "https://www.volcengine.com/",
    pathPrefix: "/ark",
    requiresApiKey: true
  },
  {
    id: "moonshot",
    name: "Moonshot (CN)",
    icon: "🌙",
    group: "cn",
    protocol: "openai",
    defaultBaseUrl: "https://api.moonshot.cn/v1",
    defaultModelId: "kimi-k2.5",
    modelPlaceholder: "kimi-k2.5",
    apiKeyPlaceholder: "sk-...",
    docsUrl: "https://platform.moonshot.cn/",
    pathPrefix: "/moonshot",
    requiresApiKey: true
  },
  {
    id: "siliconflow",
    name: "SiliconFlow (CN)",
    icon: "🌊",
    group: "cn",
    protocol: "openai",
    defaultBaseUrl: "https://api.siliconflow.cn/v1",
    defaultModelId: "deepseek-ai/DeepSeek-V3",
    modelPlaceholder: "deepseek-ai/DeepSeek-V3",
    apiKeyPlaceholder: "sk-...",
    docsUrl: "https://docs.siliconflow.cn/cn/userguide/introduction",
    pathPrefix: "/siliconflow",
    requiresApiKey: true
  },
  {
    id: "minimax-portal-cn",
    name: "MiniMax (CN)",
    icon: "☁️",
    group: "cn",
    protocol: "openai",
    defaultBaseUrl: "https://api.minimaxi.com/v1",
    defaultModelId: "MiniMax-M2.5",
    modelPlaceholder: "MiniMax-M2.5",
    apiKeyPlaceholder: "OAuth 或 API Key",
    docsUrl: "https://platform.minimaxi.com/",
    pathPrefix: "/minimax-cn",
    requiresApiKey: false
  },
  {
    id: "minimax-portal",
    name: "MiniMax (Global)",
    icon: "☁️",
    group: "compatible",
    protocol: "openai",
    defaultBaseUrl: "https://api.minimax.io/v1",
    defaultModelId: "MiniMax-M2.5",
    modelPlaceholder: "MiniMax-M2.5",
    apiKeyPlaceholder: "OAuth 或 API Key",
    docsUrl: "https://intl.minimaxi.com/",
    pathPrefix: "/minimax-global",
    requiresApiKey: false
  },
  {
    id: "qwen-portal",
    name: "Qwen (Global)",
    icon: "☁️",
    group: "compatible",
    protocol: "openai",
    defaultBaseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    defaultModelId: "coder-model",
    modelPlaceholder: "coder-model",
    apiKeyPlaceholder: "OAuth",
    pathPrefix: "/qwen",
    requiresApiKey: false
  },
  {
    id: "ollama",
    name: "Ollama",
    icon: "🦙",
    group: "local",
    protocol: "openai",
    defaultBaseUrl: "http://localhost:11434/v1",
    defaultModelId: "qwen3:latest",
    modelPlaceholder: "qwen3:latest",
    apiKeyPlaceholder: "无需填写",
    pathPrefix: "/ollama",
    requiresApiKey: false
  },
  {
    id: "custom",
    name: "Custom",
    icon: "⚙️",
    group: "custom",
    protocol: "openai",
    defaultBaseUrl: "https://api.openai.com",
    defaultModelId: "your-provider/model-id",
    modelPlaceholder: "your-provider/model-id",
    apiKeyPlaceholder: "API key...",
    docsUrl: "https://icnnp7d0dymg.feishu.cn/wiki/BmiLwGBcEiloZDkdYnGc8RWnn6d",
    pathPrefix: "/custom",
    requiresApiKey: true
  }
];
const lobsterProviderGroups = computed<Array<{ key: LobsterProviderOption["group"]; label: string }>>(() => [
  { key: "official", label: tr("provider.group.official") },
  { key: "compatible", label: tr("provider.group.compatible") },
  { key: "cn", label: tr("provider.group.cn") },
  { key: "local", label: tr("provider.group.local") },
  { key: "custom", label: tr("provider.group.custom") }
]);
const lobsterProviderGroupOptions = computed(() =>
  lobsterProviderGroups.value
    .map((group) => ({
      ...group,
      options: lobsterProviderOptions.filter((item) => item.group === group.key)
    }))
    .filter((group) => group.options.length > 0)
);
const lobsterSelectedProviderOption = computed(
  () => lobsterProviderOptions.find((item) => item.id === lobsterProviderPresetKey.value) ?? null
);
const lobsterEffectiveProviderOption = computed(
  () => lobsterSelectedProviderOption.value ?? getLobsterProviderOptionByDraft(lobsterProviderForm.value)
);
const lobsterProviderRequiresApiKey = computed(() => lobsterEffectiveProviderOption.value?.requiresApiKey !== false);
const lobsterProviderModelPlaceholder = computed(
  () => lobsterEffectiveProviderOption.value?.modelPlaceholder ?? "如 openai/gpt-5.4"
);
const lobsterProviderApiKeyPlaceholder = computed(
  () => lobsterEffectiveProviderOption.value?.apiKeyPlaceholder ?? "sk-..."
);
const lobsterProviderSelectedIcon = computed(
  () => lobsterSelectedProviderOption.value?.icon ?? lobsterEffectiveProviderOption.value?.icon ?? "🧩"
);
const lobsterProviderApiKind = computed<LobsterProviderApiKind>(() => getLobsterProviderApiKindByDraft(lobsterProviderForm.value));
const lobsterProviderApiKindOptions = computed<LobsterProviderApiKindOption[]>(() => [
  {
    value: "openai-completions",
    label: tr("provider.protocol.openai_completions")
  },
  {
    value: "openai-responses",
    label: tr("provider.protocol.openai_responses")
  },
  {
    value: "anthropic-messages",
    label: tr("provider.protocol.anthropic_compatible")
  }
]);
const lobsterShowCustomProtocolPicker = computed(() => lobsterEffectiveProviderOption.value?.id === "custom");
const appLocaleOptions = computed(() =>
  APP_LOCALE_OPTIONS.map((value) => ({
    value,
    label: tr(`locale.${value}`)
  }))
);
const appThemeOptions = computed(() =>
  APP_THEME_OPTIONS.map((value) => ({
    value,
    label: tr(`system.settings.theme.${value}`)
  }))
);
const lobsterInstallGuideCheckTotal = computed(
  () => lobsterInstallGuide.value?.checks.length ?? LOBSTER_INSTALL_CHECK_BLUEPRINT.length
);
const lobsterInstallGuideCheckedCount = computed(
  () => lobsterInstallGuide.value?.checks.filter((item) => item.status !== "checking").length ?? 0
);
const lobsterInstallGuideSummaryText = computed(() => {
  if (lobsterInstallGuideLoading.value) {
    return tr("wizard.runtime.checking");
  }
  return lobsterInstallGuide.value?.ready ? tr("wizard.runtime.ready") : tr("wizard.runtime.blocked");
});
const lobsterInstallCanGoNext = computed(() => {
  if (lobsterInstallWizardStep.value === 1) return lobsterInstallRiskAccepted.value;
  if (lobsterInstallWizardStep.value === 2) return Boolean(lobsterInstallGuide.value?.ready);
  if (lobsterInstallWizardStep.value === 3) return lobsterProviderConfigured.value;
  if (lobsterInstallWizardStep.value === 4) return false;
  return true;
});
const lobsterProviderCanSave = computed(() =>
  lobsterProviderForm.value.name.trim().length > 0 &&
  lobsterProviderForm.value.baseUrl.trim().length > 0 &&
  lobsterProviderForm.value.model.trim().length > 0 &&
  (!lobsterProviderRequiresApiKey.value || lobsterProviderForm.value.apiKey.trim().length > 0)
);
const lobsterInstallStepTitle = computed(
  () => lobsterInstallWizardSteps.value.find((item) => item.id === lobsterInstallWizardStep.value)?.title ?? tr("wizard.step.1.title")
);
const lobsterInstallStepDescription = computed(
  () => lobsterInstallWizardSteps.value.find((item) => item.id === lobsterInstallWizardStep.value)?.description ?? ""
);
const subscriptionReferenceUrl = "https://mp.weixin.qq.com/s/AD4nB87oGu4s40Dvo4p2PA?scene=1";
const subscriptionDataUpdatedAt = "2026-03-17";
const subscriptionScenarioRecommendations: SubscriptionScenarioRecommendation[] = [
  {
    scene: "快速简单问答",
    primary: "Kimi",
    secondary: "智谱 GLM5",
    caution: "阿里云百炼（过度思考）"
  },
  {
    scene: "逻辑推理 / 数学题",
    primary: "火山方舟",
    secondary: "智谱 GLM5",
    caution: "腾讯云、MiniMax（易卡死）"
  },
  {
    scene: "空间想象 / 陷阱题",
    primary: "智谱 GLM5",
    secondary: "火山方舟",
    caution: "Kimi（答案不稳定）"
  },
  {
    scene: "追求性价比（省 Token）",
    primary: "火山方舟",
    secondary: "Kimi / 智谱 GLM5",
    caution: "阿里云、腾讯云"
  },
  {
    scene: "追求稳定性（生产环境）",
    primary: "智谱 GLM5",
    secondary: "火山方舟",
    caution: "所有平台高峰期都可能波动"
  }
];
const subscriptionPlatformBenchmarks: SubscriptionPlatformBenchmark[] = [
  {
    platform: "Kimi",
    firstResponse: "★★★★★（约 7s）",
    outputSpeed: "★★★★★",
    tokenUsage: "★★★★（偏省）",
    simpleAccuracy: "通过",
    logicAccuracy: "通过",
    spatialAccuracy: "不稳定（3 次结果不一致）",
    stability: "★★★",
    note: "速度最快，但空间推理偶发“硬猜”现象"
  },
  {
    platform: "火山方舟",
    firstResponse: "★★★",
    outputSpeed: "★★★★★（逻辑题约 15s）",
    tokenUsage: "★★★★★（最省）",
    simpleAccuracy: "通过",
    logicAccuracy: "通过",
    spatialAccuracy: "通过",
    stability: "★★★★★",
    note: "综合表现最优，简单题思考略长但无答错记录"
  },
  {
    platform: "智谱 GLM5",
    firstResponse: "★★★★",
    outputSpeed: "★★★★",
    tokenUsage: "★★★★（适中）",
    simpleAccuracy: "通过",
    logicAccuracy: "通过",
    spatialAccuracy: "通过",
    stability: "★★★★★",
    note: "速度 / 消耗 / 质量三者平衡最好"
  },
  {
    platform: "阿里云百炼",
    firstResponse: "★（约 27.8s）",
    outputSpeed: "★（逻辑题约 98s）",
    tokenUsage: "★★（较高）",
    simpleAccuracy: "通过",
    logicAccuracy: "通过",
    spatialAccuracy: "通过",
    stability: "★★",
    note: "思考模式偏重，简单问题也会深思熟虑，整体偏慢"
  },
  {
    platform: "腾讯云",
    firstResponse: "★★★",
    outputSpeed: "失败（常卡住截断）",
    tokenUsage: "不可用（深度任务易中断）",
    simpleAccuracy: "通过",
    logicAccuracy: "交白卷",
    spatialAccuracy: "交白卷",
    stability: "★",
    note: "深度思考场景表现不稳定，不建议逻辑 / 空间任务"
  },
  {
    platform: "MiniMax",
    firstResponse: "★★★★",
    outputSpeed: "失败（常卡住截断）",
    tokenUsage: "不可用（深度任务易中断）",
    simpleAccuracy: "通过",
    logicAccuracy: "交白卷",
    spatialAccuracy: "不稳定",
    stability: "★★",
    note: "简单任务速度尚可，深度任务易卡死，答案波动大"
  }
];
const subscriptionPlatformUrlMap: Record<string, string> = {
  Kimi: "https://www.kimi.com/code/zh",
  "火山方舟": "https://www.volcengine.com/docs/82379/2188957",
  "智谱 GLM5": "https://docs.bigmodel.cn/cn/coding-plan/overview",
  "阿里云百炼": "https://www.aliyun.com/benefit/scene/codingplan",
  "腾讯云": "https://cloud.tencent.com/act/pro/codingplan",
  MiniMax: "https://platform.minimaxi.com/docs/guides/pricing-coding-plan"
};
const codingPlanRecommendations: CodingPlanRecommendation[] = [
  {
    id: "alibaba",
    category: "cloud",
    name: "阿里云百炼 Coding Plan",
    summary: "首家集齐四大国产顶尖大模型的订阅服务，适合希望一次性覆盖多模型能力的团队。",
    latest: "近期升级支持了 Qwen3.5-Plus、GLM-5、MiniMax M2.5、Kimi K2.5 等最新模型。",
    highlights: ["月度总量额度，适合偶发高强度开发", "支持 Qwen Code、Claude Code、Cline", "多模型覆盖完整"],
    pricing: "Lite 版首月 7.9 元起，Pro 版首月 39.9 元起",
    platformUrl: "https://www.aliyun.com/benefit/scene/codingplan",
    accent: "amber"
  },
  {
    id: "tencent",
    category: "cloud",
    name: "腾讯云 Coding Plan",
    summary: "2026 年 3 月全新上架，适合已经在腾讯云生态内做研发协同的团队。",
    latest: "支持 Tencent HY 2.0 Instruct、GLM-5、Kimi-K2.5、MiniMax-M2.5。",
    highlights: ["支持 CodeBuddy、OpenClaw、Claude Code", "新用户专享首月低至 7.9 元", "新上架平台迭代节奏快"],
    pricing: "新用户专享首月低至 7.9 元",
    platformUrl: "https://cloud.tencent.com/act/pro/codingplan",
    accent: "sky"
  },
  {
    id: "volcengine",
    category: "cloud",
    name: "火山引擎方舟 Coding Plan",
    summary: "工具兼容和接入灵活度最强，尤其适合 Claude Code 用户直接原生接入。",
    latest: "支持 Doubao-Seed-Code、DeepSeek-V3.2、Kimi-K2.5 等模型。",
    highlights: ["唯一支持 Anthropic 协议", "Claude Code 无需配置代理即可原生接入", "客户端覆盖最广，已覆盖 11 款"],
    pricing: "首购 9.9 元起",
    platformUrl: "https://www.volcengine.com/docs/82379/2188957",
    accent: "rose"
  },
  {
    id: "baidu",
    category: "cloud",
    name: "百度千帆 Coding Plan",
    summary: "偏向全流程研发辅助，适合希望把代码编写、逻辑理解、系统优化打通的团队。",
    latest: "2026 年 2 月正式上线，首批集成 GLM-4.7、DeepSeek-V3.2 等模型。",
    highlights: ["覆盖代码编写", "覆盖逻辑理解", "覆盖系统优化全流程"],
    pricing: "价格信息待补充",
    pricingNote: "当前更强调全流程能力覆盖。",
    platformUrl: "https://cloud.baidu.com/doc/qianfan/s/imlg0beiu",
    accent: "default"
  },
  {
    id: "chinaunicom",
    category: "cloud",
    name: "联通云 Coding Plan",
    summary: "强调多工具自由切换，适合不同研发习惯并存的团队统一采购。",
    latest: "2026 年 3 月上线，支持 GLM-5、MiniMax M2.5、Qwen3.5、DSV3.X 系列五大模型。",
    highlights: ["可在 OpenCode、Claude Code、OpenClaw、CoPaw 四个工具中自由切换", "多模型覆盖广", "适合多工具共存团队"],
    pricing: "价格信息待补充",
    pricingNote: "当前更适合关注工具兼容性的团队。",
    platformUrl: "https://www.cucloud.cn/",
    accent: "mint"
  },
  {
    id: "ucloud",
    category: "cloud",
    name: "优刻得 Coding Plan",
    summary: "作为中立云厂商，不绑定单一模型生态，适合想做多模型横向比较的团队。",
    latest: "2026 年 3 月上线，支持 DeepSeek-V3.2、GLM-5、MiniMax M2.5、Kimi K2.5。",
    highlights: ["中立云厂商", "不绑定单一模型生态", "提供一站式聚合服务"],
    pricing: "价格信息待补充",
    pricingNote: "核心卖点在于聚合中立性。",
    platformUrl: "https://www.compshare.cn/",
    accent: "default"
  },
  {
    id: "zhipu",
    category: "model",
    name: "智谱 GLM Coding Plan",
    summary: "模型厂商平台里扩展能力最完整的一档，适合重度 MCP 和工具链协同场景。",
    latest: "已支持 GLM-5、GLM-4.7；并上线用量看板、Zread MCP 等功能更新。",
    highlights: ["附赠 4 个专属 MCP", "支持联网搜索、视觉理解等", "客户端兼容性极强，支持 20+ 款"],
    pricing: "入门档约 49 元/月",
    platformUrl: "https://docs.bigmodel.cn/cn/coding-plan/overview",
    accent: "mint"
  },
  {
    id: "minimax",
    category: "model",
    name: "MiniMax Coding Plan",
    summary: "主打编程性价比，适合预算敏感但希望稳定获得编码能力的个人与小团队。",
    latest: "支持 MiniMax M2.5、M2.1 等模型。",
    highlights: ["入门仅 29 元/月", "首月 9.9 元", "极速版可选"],
    pricing: "入门档 29 元/月",
    pricingNote: "首月优惠后更适合低成本试用。",
    platformUrl: "https://platform.minimaxi.com/docs/guides/pricing-coding-plan",
    accent: "amber"
  },
  {
    id: "kimi",
    category: "model",
    name: "Kimi Coding Plan",
    summary: "采用 Token 计量制，是唯一不限 5 小时窗口的平台，适合长时间连续编程。",
    latest: "支持 Kimi K2.5。",
    highlights: ["Token 计量制", "适合长时间连续编程", "更适合重上下文会话"],
    pricing: "入门档 49 元/月",
    platformUrl: "https://www.kimi.com/code/zh",
    accent: "sky"
  }
];
const cloudCodingPlans = computed(() => codingPlanRecommendations.filter((plan) => plan.category === "cloud"));
const modelCodingPlans = computed(() => codingPlanRecommendations.filter((plan) => plan.category === "model"));

function isImplicitSeededOpenAiPlatform(platform: PlatformConfig | null) {
  if (!platform) {
    return false;
  }

  return (
    platform.name === "OpenAI" &&
    platform.protocol === "openai" &&
    platform.baseUrl === "https://api.openai.com" &&
    platform.pathPrefix === "/openai" &&
    platform.apiPath === "/v1/chat/completions" &&
    platform.model === "gpt-4o-mini" &&
    !platform.apiKey.trim()
  );
}

const activeAnimation = computed(() => animations[currentAnimationName.value]);
const actionTipValues = new Set(Object.values(actionTips));
const shouldShowHint = computed(() => !actionTipValues.has(statusText.value));
const currentFrame = computed(() => activeAnimation.value.config.frames[currentFrameIndex.value]);
const activePlatform = computed(
  () => platforms.value.find((platform) => platform.id === activePlatformId.value) || null
);
const requestLogs = computed<RequestLog[]>(() =>
  [...localRequestLogs.value, ...runtimeRequestLogs.value]
    .sort((left, right) => right.createdAt - left.createdAt)
    .filter((log, index, items) => items.findIndex((candidate) => candidate.id === log.id) === index)
);
const enabledPlatformCount = computed(() => platforms.value.filter((platform) => platform.enabled).length);
const configuredSubscriptionCount = computed(() => platforms.value.filter((platform) => platform.apiKey.trim()).length);
const activeChatAgent = computed(() =>
  activeChatAgentId.value
    ? staffMembers.value.find((m) => m.agentId === activeChatAgentId.value) ?? null
    : null
);
const activeBoundPet = computed(() => {
  const key = activeChatAgentId.value;
  if (!key || !key.startsWith(BOUND_PET_CHAT_PREFIX)) {
    return null;
  }
  const petId = key.slice(BOUND_PET_CHAT_PREFIX.length);
  return boundPets.value.find((petItem) => petItem.id === petId) ?? null;
});
const chatHeaderTitle = computed(() => {
  if (activeBoundPet.value) {
    return `${activeBoundPet.value.petName}（远程宠物）`;
  }
  return activeChatAgent.value ? stripRoleLabel(activeChatAgent.value.displayName) : "OpenClaw";
});
const openClawMessages = computed<OpenClawMessage[]>(() =>
  chatMessages.value
    .filter((message) => message.status !== "pending" && message.role !== "system")
    .map((message) => ({
      role: message.role,
      content: message.text
    }))
);
const hintStyle = computed(() => {
  const viewportWidth = typeof window === "undefined" ? 360 : window.innerWidth;
  const viewportHeight = typeof window === "undefined" ? 640 : window.innerHeight;
  const bubbleWidth = Math.min(340, viewportWidth - 40);
  const petCenterX = petPosition.value.x + viewportSize.value / 2;
  const preferredLeft = petCenterX - bubbleWidth / 2;
  const left = Math.min(Math.max(16, preferredLeft), Math.max(16, viewportWidth - bubbleWidth - 16));
  const topAbovePet = petPosition.value.y - 108;
  const topBelowPet = petPosition.value.y + viewportSize.value + 14;
  const top = topAbovePet >= 16 ? topAbovePet : Math.min(topBelowPet, Math.max(16, viewportHeight - 96));

  return {
    left: `${left}px`,
    top: `${top}px`,
    maxWidth: `${bubbleWidth}px`
  };
});
const petStyle = computed(() => {
  const animation = activeAnimation.value;
  const frame = currentFrame.value;
  const scale = viewportSize.value / animation.config.frame_size.w;

  return {
    width: `${viewportSize.value}px`,
    height: `${viewportSize.value}px`,
    left: `${petPosition.value.x}px`,
    top: `${petPosition.value.y}px`,
    backgroundImage: `url(${animation.sprite})`,
    backgroundRepeat: "no-repeat",
    backgroundSize: `${animation.config.sheet_size.w * scale}px ${animation.config.sheet_size.h * scale}px`,
    backgroundPosition: `-${frame.x * scale}px -${frame.y * scale}px`
  };
});
const chatPanelStyle = computed(() => {
  const bounds = stage.value?.getBoundingClientRect();
  const viewportWidth = bounds?.width ?? (typeof window === "undefined" ? 360 : window.innerWidth);
  const viewportHeight = bounds?.height ?? (typeof window === "undefined" ? 640 : window.innerHeight);
  const defaultWidth = Math.min(620, Math.max(420, viewportWidth - 32));
  const defaultHeight = Math.min(620, Math.max(420, viewportHeight - 32));
  const panelWidth =
    chatPlacement.value.mode === "manual" && chatPlacement.value.width > 0
      ? Math.min(Math.max(420, chatPlacement.value.width), viewportWidth - 32)
      : defaultWidth;
  const panelHeight =
    chatPlacement.value.mode === "manual" && chatPlacement.value.height > 0
      ? Math.min(Math.max(420, chatPlacement.value.height), viewportHeight - 32)
      : defaultHeight;
  const gap = 18;
  const petLeft = petPosition.value.x;
  const petRight = petPosition.value.x + viewportSize.value;
  const petCenterY = petPosition.value.y + viewportSize.value / 2;
  const leftSpace = petLeft - gap - 16;
  const rightSpace = viewportWidth - petRight - gap - 16;
  const canPlaceLeft = leftSpace >= panelWidth;
  const canPlaceRight = rightSpace >= panelWidth;
  const autoLeft = canPlaceLeft
    ? petLeft - panelWidth - gap
    : canPlaceRight
      ? petRight + gap
      : Math.min(Math.max(16, viewportWidth - panelWidth - 16), Math.max(16, petLeft - panelWidth * 0.5));
  const autoTop = Math.min(Math.max(16, petCenterY - panelHeight / 2), Math.max(16, viewportHeight - panelHeight - 16));
  const left =
    chatPlacement.value.mode === "manual"
      ? Math.min(Math.max(16, chatPlacement.value.x), Math.max(16, viewportWidth - panelWidth - 16))
      : autoLeft;
  const top =
    chatPlacement.value.mode === "manual"
      ? Math.min(Math.max(16, chatPlacement.value.y), Math.max(16, viewportHeight - panelHeight - 16))
      : autoTop;
  const progress = chatMotionValue.value;

  return {
    width: `${panelWidth}px`,
    height: `${panelHeight}px`,
    left: `${left}px`,
    top: `${top}px`,
    opacity: `${interpolate(progress, [0, 1], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: easeOutCubic
    })}`,
    transform: `translateY(${interpolate(progress, [0, 1], [20, 0])}px) scale(${interpolate(progress, [0, 1], [0.94, 1])})`
  };
});
const consolePanelStyle = computed(() => {
  const bounds = stage.value?.getBoundingClientRect();
  const viewportWidth = bounds?.width ?? (typeof window === "undefined" ? 360 : window.innerWidth);
  const viewportHeight = bounds?.height ?? (typeof window === "undefined" ? 640 : window.innerHeight);
  if (isConsoleWindowMode) {
    if (isWindowsRuntime) {
      const margin = 18;
      const width = Math.max(320, viewportWidth - margin * 2);
      const height = Math.max(320, viewportHeight - margin * 2);
      return {
        width: `${width}px`,
        height: `${height}px`,
        left: `${margin}px`,
        top: `${margin}px`,
        opacity: "1",
        transform: "none",
        transformOrigin: "center center"
      };
    }

    return {
      width: `${Math.max(320, viewportWidth)}px`,
      height: `${Math.max(320, viewportHeight)}px`,
      left: "0px",
      top: "0px",
      opacity: "1",
      transform: "none",
      transformOrigin: "center center"
    };
  }
  const prefersWide = true;
  const availableWidth = Math.max(320, viewportWidth - 32);
  const availableHeight = Math.max(320, viewportHeight - 32);
  const minWidth = Math.min(getPanelMinWidth(prefersWide), availableWidth);
  const minHeight = Math.min(420, availableHeight);
  const defaultWidth = Math.min(availableWidth, Math.max(minWidth, Math.round(viewportWidth * 0.8)));
  const defaultHeight = Math.min(availableHeight, Math.max(minHeight, Math.round(viewportHeight * 0.8)));
  const panelWidth =
    panelPlacement.value.mode === "manual" && panelPlacement.value.width > 0
      ? Math.min(Math.max(minWidth, panelPlacement.value.width), availableWidth)
      : defaultWidth;
  const panelHeight =
    panelPlacement.value.mode === "manual" && panelPlacement.value.height > 0
      ? Math.min(Math.max(minHeight, panelPlacement.value.height), availableHeight)
      : defaultHeight;
  const margin = 16;
  const gap = 18;
  const petClearance = 18;
  const petCenterX = petPosition.value.x + viewportSize.value / 2;
  const petCenterY = petPosition.value.y + viewportSize.value / 2;
  const petSafeLeft = Math.max(0, petPosition.value.x - petClearance);
  const petSafeTop = Math.max(0, petPosition.value.y - petClearance);
  const petSafeRight = Math.min(viewportWidth, petPosition.value.x + viewportSize.value + petClearance);
  const petSafeBottom = Math.min(viewportHeight, petPosition.value.y + viewportSize.value + petClearance);
  const clampAutoPosition = (nextLeft: number, nextTop: number) => ({
    left: Math.min(Math.max(margin, nextLeft), Math.max(margin, viewportWidth - panelWidth - margin)),
    top: Math.min(Math.max(margin, nextTop), Math.max(margin, viewportHeight - panelHeight - margin))
  });
  const overlapsPet = (nextLeft: number, nextTop: number) =>
    !(
      nextLeft + panelWidth <= petSafeLeft ||
      petSafeRight <= nextLeft ||
      nextTop + panelHeight <= petSafeTop ||
      petSafeBottom <= nextTop
    );
  const autoCandidates = [
    clampAutoPosition(petPosition.value.x - panelWidth - gap, petCenterY - panelHeight / 2),
    clampAutoPosition(petPosition.value.x + viewportSize.value + gap, petCenterY - panelHeight / 2),
    clampAutoPosition(petCenterX - panelWidth / 2, petPosition.value.y - panelHeight - gap),
    clampAutoPosition(petCenterX - panelWidth / 2, petPosition.value.y + viewportSize.value + gap),
    clampAutoPosition(margin, margin),
    clampAutoPosition(viewportWidth - panelWidth - margin, margin),
    clampAutoPosition(margin, viewportHeight - panelHeight - margin),
    clampAutoPosition(viewportWidth - panelWidth - margin, viewportHeight - panelHeight - margin)
  ];
  const bestAutoPosition =
    autoCandidates.find((candidate) => !overlapsPet(candidate.left, candidate.top)) ??
    autoCandidates.reduce((best, candidate) => {
      const bestCenterX = best.left + panelWidth / 2;
      const bestCenterY = best.top + panelHeight / 2;
      const candidateCenterX = candidate.left + panelWidth / 2;
      const candidateCenterY = candidate.top + panelHeight / 2;
      const bestDistance = Math.hypot(bestCenterX - petCenterX, bestCenterY - petCenterY);
      const candidateDistance = Math.hypot(candidateCenterX - petCenterX, candidateCenterY - petCenterY);
      return candidateDistance > bestDistance ? candidate : best;
    });
  const left =
    panelPlacement.value.mode === "manual"
      ? Math.min(Math.max(16, panelPlacement.value.x), Math.max(16, viewportWidth - panelWidth - 16))
      : bestAutoPosition.left;
  const top =
    panelPlacement.value.mode === "manual"
      ? Math.min(Math.max(16, panelPlacement.value.y), Math.max(16, viewportHeight - panelHeight - 16))
      : bestAutoPosition.top;
  const progress = panelMotionValue.value;
  const originX = "center";

  return {
    width: `${panelWidth}px`,
    height: `${panelHeight}px`,
    left: `${left}px`,
    top: `${top}px`,
    opacity: `${interpolate(progress, [0, 1], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: easeOutCubic
    })}`,
    transform: `translateY(${interpolate(progress, [0, 1], [20, 0])}px) scale(${interpolate(progress, [0, 1], [0.94, 1])})`,
    transformOrigin: `${originX} center`
  };
});
const metrics = computed(() => {
  const failures = requestLogs.value.filter((log) => isFailedLog(log)).length;
  const totalDuration = requestLogs.value.reduce((sum, log) => sum + log.duration, 0);
  const averageDuration = requestLogs.value.length > 0 ? Math.round(totalDuration / requestLogs.value.length) : 0;
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const startOfSevenDays = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 6).getTime();
  const todayTokens = requestLogs.value.reduce(
    (sum, log) => (log.createdAt >= startOfToday ? sum + getEffectiveTotalTokens(log) : sum),
    0
  );
  const sevenDayTokens = requestLogs.value.reduce(
    (sum, log) => (log.createdAt >= startOfSevenDays ? sum + getEffectiveTotalTokens(log) : sum),
    0
  );
  const totalTokens = requestLogs.value.reduce((sum, log) => sum + getEffectiveTotalTokens(log), 0);
  const gatewayStatusValue =
    gatewayMonitor.value.status === "online"
      ? `在线${typeof gatewayMonitor.value.latencyMs === "number" ? ` · ${gatewayMonitor.value.latencyMs} ms` : ""}`
      : gatewayMonitor.value.status === "offline"
        ? "离线"
        : gatewayMonitor.value.status === "unconfigured"
          ? "未配置"
          : gatewayMonitor.value.status === "unsupported"
            ? "不可用"
            : "检测中";

  return [
    { label: "代理平台", value: `${enabledPlatformCount.value}` },
    { label: "调用总数", value: `${requestLogs.value.length}` },
    { label: "网关状态", value: gatewayStatusValue },
    { label: "平均耗时", value: `${averageDuration} ms` },
    { label: "今日 Token", value: `${todayTokens}` },
    { label: "7 天 Token", value: `${sevenDayTokens}` },
    { label: "累计 Token", value: `${totalTokens}` },
    { label: "失败请求", value: `${failures}` }
  ];
});
const availableLogPlatformGroups = computed(() => {
  const groups = new Set<string>();
  for (const log of requestLogs.value) {
    groups.add(getLogPlatformGroup(log));
  }
  return Array.from(groups).sort();
});
const availableLogAgentOptions = computed(() => {
  const agents = new Map<string, string>();
  for (const log of requestLogs.value) {
    const agentId = getLogAgentId(log);
    if (agentId && !agents.has(agentId)) {
      agents.set(agentId, getLogAgentDisplayName(log));
    }
  }
  for (const member of staffMembers.value) {
    if (!agents.has(member.agentId)) {
      agents.set(member.agentId, member.displayName);
    }
  }
  const options: { id: string; name: string }[] = [{ id: "__main__", name: "主控台" }];
  for (const [id, name] of agents) {
    options.push({ id, name });
  }
  return options;
});
const filteredRequestLogs = computed(() => {
  if (!logFilterPlatform.value && !logFilterAgent.value) {
    return requestLogs.value;
  }
  return requestLogs.value.filter((log) => {
    if (logFilterPlatform.value && getLogPlatformGroup(log) !== logFilterPlatform.value) {
      return false;
    }
    if (logFilterAgent.value) {
      const agentId = getLogAgentId(log);
      if (logFilterAgent.value === "__main__") {
        if (agentId !== null) return false;
      } else {
        if (agentId !== logFilterAgent.value) return false;
      }
    }
    return true;
  });
});
const hasActiveLogFilter = computed(() => logFilterPlatform.value !== null || logFilterAgent.value !== null);
const timelineEntries = computed(() => filteredRequestLogs.value);
const scheduledTaskCount = computed(() => 0);
const memoryStatusSummary = computed(() => ({
  main: memoryRecords.value.filter((item) => item.owner === "Main").length,
  agents: memoryRecords.value.filter((item) => item.owner !== "Main").length
}));
const documentStatusSummary = computed(() => ({
  main: documentRecords.value.filter((item) => item.owner === "Main").length,
  agents: documentRecords.value.filter((item) => item.owner !== "Main").length
}));
const memoryScopeOptions = computed(() =>
  Array.from(new Set(memoryRecords.value.map((item) => item.scope.trim()).filter(Boolean))).map((scope) => ({
    key: scope,
    label: scope
  }))
);
const documentCategoryOptions = computed(() => [
  { key: "all", label: "全部分类" },
  ...Array.from(new Set(documentRecords.value.map((item) => item.category.trim()).filter(Boolean))).map((category) => ({
    key: category,
    label: category
  }))
]);
const filteredMemoryRecords = computed(() => {
  const keyword = memoryFilterText.value.trim().toLowerCase();
  const fallbackScope = memoryScopeOptions.value[0]?.key ?? "";
  const activeScope = activeMemoryScope.value || fallbackScope;

  return memoryRecords.value.filter((record) => {
    const matchesScope = !activeScope || record.scope === activeScope;
    if (!matchesScope) {
      return false;
    }

    if (!keyword) {
      return true;
    }

    const haystack = [record.title, record.owner, record.scope, record.summary].join(" ").toLowerCase();
    return haystack.includes(keyword);
  });
});
const filteredDocumentRecords = computed(() => {
  const keyword = documentFilterText.value.trim().toLowerCase();

  return documentRecords.value.filter((record) => {
    const matchesCategory = activeDocumentCategory.value === "all" || record.category === activeDocumentCategory.value;
    if (!matchesCategory) {
      return false;
    }

    if (!keyword) {
      return true;
    }

    const haystack = [record.title, record.category, record.owner, record.source, record.summary].join(" ").toLowerCase();
    return haystack.includes(keyword);
  });
});
const activeMemorySelectionRecords = computed(() =>
  activeResourceModal.value === "memory" ? filteredMemberMemoryRecords.value : filteredMemoryRecords.value
);
const activeDocumentSelectionRecords = computed(() =>
  activeResourceModal.value && activeResourceModal.value !== "memory" ? filteredMemberDocumentRecords.value : filteredDocumentRecords.value
);
const selectedMemoryRecord = computed(
  () =>
    activeMemorySelectionRecords.value.find((record) => record.id === selectedMemoryId.value) ?? activeMemorySelectionRecords.value[0] ?? null
);
const selectedDocumentRecord = computed(
  () =>
    activeDocumentSelectionRecords.value.find((record) => record.id === selectedDocumentId.value) ?? activeDocumentSelectionRecords.value[0] ?? null
);
const activeResourceMember = computed(
  () => staffMembers.value.find((member) => member.agentId === activeResourceMemberId.value) ?? null
);
const activeRecentOutputMember = computed(
  () => staffMembers.value.find((member) => member.agentId === recentOutputModalMemberId.value) ?? null
);
const recentOutputModalTitle = computed(() => `${activeRecentOutputMember.value?.displayName ?? "员工"} · 最近产出`);
const activeResourceModalTitle = computed(() => {
  if (activeResourceModal.value === "memory") return "记忆";
  if (activeResourceModal.value === "skill") return "技能";
  if (activeResourceModal.value === "tool") return "工具";
  return "";
});
/** 弹窗主标题：技能为 OpenClaw 技能库（全员共享），工具为员工工具权限配置 */
const resourceModalHeaderTitle = computed(() => {
  const member = activeResourceMember.value;
  if (activeResourceModal.value === "memory") {
    return `${member?.displayName ?? "员工"} · 记忆`;
  }
  if (activeResourceModal.value === "skill") {
    return "OpenClaw 技能库";
  }
  if (activeResourceModal.value === "tool") {
    return `${member?.displayName ?? "员工"} · 工具权限设置`;
  }
  return "";
});
const activeResourceModalDescription = computed(() => {
  const member = activeResourceMember.value;
  const name = member?.displayName ?? "当前员工";
  if (activeResourceModal.value === "memory") {
    return `${name} 的真实记忆文件，可直接查看与编辑。`;
  }
  if (activeResourceModal.value === "skill") {
    return "来自 ~/.openclaw/skills 与 workspace/skills，每个子目录的 SKILL.md 为一项技能，全员共享。可查看与编辑。";
  }
  if (activeResourceModal.value === "tool") {
    return "可切换权限范围（当前员工 / 全局默认），调整 Profile，并对每个工具做启用或禁用。";
  }
  return "";
});
/** 侧边栏标题：技能列表 / 工具文档 / 记忆 */
const resourceSidebarHeadline = computed(() => {
  if (activeResourceModal.value === "memory") return "记忆文件";
  if (activeResourceModal.value === "skill") return "技能列表";
  if (activeResourceModal.value === "tool") return "工具权限";
  return "";
});
/** 技能弹窗内按关键词筛选的内置技能列表 */
const filteredOpenClawBuiltInSkills = computed(() => {
  if (activeResourceModal.value !== "skill") return [];
  const keyword = resourceModalFilterText.value.trim().toLowerCase();
  const list = openClawSkillsList.value.builtIn ?? [];
  if (!keyword) return list;
  return list.filter(
    (s) =>
      s.name.toLowerCase().includes(keyword) ||
      s.description.toLowerCase().includes(keyword)
  );
});
/** 技能弹窗内按关键词筛选的安装技能列表 */
const filteredOpenClawInstalledSkills = computed(() => {
  if (activeResourceModal.value !== "skill") return [];
  const keyword = resourceModalFilterText.value.trim().toLowerCase();
  const list = openClawSkillsList.value.installed ?? [];
  if (!keyword) return list;
  return list.filter(
    (s) =>
      s.name.toLowerCase().includes(keyword) ||
      s.description.toLowerCase().includes(keyword) ||
      (s.relativePath && s.relativePath.toLowerCase().includes(keyword))
  );
});
const activeOpenClawSkills = computed(() =>
  openClawSkillCategory.value === "installed" ? filteredOpenClawInstalledSkills.value : filteredOpenClawBuiltInSkills.value
);
const activeOpenClawSkillsTitle = computed(() => (openClawSkillCategory.value === "installed" ? "安装技能" : "内置技能"));
const activeOpenClawSkillsEmptyText = computed(() =>
  openClawSkillCategory.value === "installed"
    ? "暂无安装技能。在 ~/.openclaw/skills 或 workspace/skills 下为每个技能建子目录并放入 SKILL.md。"
    : "暂无内置技能。"
);
/** 技能总数（内置 + 安装） */
const openClawSkillsTotalCount = computed(
  () => (openClawSkillsList.value.builtIn?.length ?? 0) + (openClawSkillsList.value.installed?.length ?? 0)
);
/** 工具弹窗内按分类分组的工具列表 */
const openClawToolsByCategory = computed(() => {
  const tools = openClawToolsList.value.tools;
  const map = new Map<string, OpenClawToolListItem[]>();
  for (const t of tools) {
    const cat = t.category || "Other";
    if (!map.has(cat)) map.set(cat, []);
    map.get(cat)!.push(t);
  }
  const order = ["Files", "Runtime", "Web", "Memory", "Sessions", "Messaging", "UI", "Automation", "Nodes", "Other"];
  return order.filter((c) => map.has(c)).map((c) => ({ category: c, tools: map.get(c)! }));
});
const openClawToolsEnabledCount = computed(() => openClawToolsList.value.tools.filter((tool) => tool.enabled).length);
const openClawToolsScopeLabel = computed(() => (openClawToolsScope.value === "global" ? "全局默认" : "当前员工"));
const openClawToolsProfileOptions = computed<OpenClawToolsProfileOption[]>(() => {
  const current = openClawToolsList.value.profile.trim();
  if (!current) return OPENCLAW_TOOLS_PROFILE_PRESETS;
  if (OPENCLAW_TOOLS_PROFILE_PRESETS.some((item) => item.value === current)) {
    return OPENCLAW_TOOLS_PROFILE_PRESETS;
  }
  return [{ value: current, label: `${current}（自定义）` }, ...OPENCLAW_TOOLS_PROFILE_PRESETS];
});
const filteredMemberMemoryRecords = computed(() => {
  const member = activeResourceMember.value;
  if (!member || activeResourceModal.value !== "memory") {
    return [];
  }

  const keyword = resourceModalFilterText.value.trim().toLowerCase();
  return getMemberMemoryRecords(member).filter((record) => {
    if (!keyword) {
      return true;
    }

    return [record.title, record.owner, record.scope, record.summary, record.relativePath].join(" ").toLowerCase().includes(keyword);
  });
});
const filteredMemberDocumentRecords = computed(() => {
  if (!activeResourceModal.value || activeResourceModal.value === "memory") {
    return [];
  }

  const keyword = resourceModalFilterText.value.trim().toLowerCase();
  const records = resourceDocumentRecords.value;

  return records.filter((record) => {
    if (!keyword) {
      return true;
    }

    return [record.title, record.owner, record.category, record.summary, record.relativePath].join(" ").toLowerCase().includes(keyword);
  });
});
const activeResourceTotalCount = computed(() =>
  activeResourceModal.value === "memory" ? filteredMemberMemoryRecords.value.length : filteredMemberDocumentRecords.value.length
);
const activeResourceSelectedLabel = computed(() => {
  if (activeResourceModal.value === "memory") {
    return selectedMemoryId.value ? "编辑记忆文件" : "选择记忆文件";
  }
  if (activeResourceModal.value === "skill") {
    return selectedDocumentId.value ? "编辑 SKILL.md" : "选择技能";
  }
  if (activeResourceModal.value === "tool") {
    return selectedDocumentId.value ? "编辑 TOOLS.md" : "选择或创建 TOOLS.md";
  }
  return "选择资源";
});
const selectedMemoryPurposeDescription = computed(() => {
  const record = selectedMemoryRecord.value;
  if (!record) {
    return "这里按员工维度展示真实记忆文件，可在选中后查看该文件承载的角色与作用。";
  }

  const fileName = record.relativePath.split("/").pop()?.toUpperCase() ?? "";
  const suffix = record.exists ? "当前文件已存在，修改后会直接影响该员工后续的行为与上下文。" : "当前文件缺失，保存后会创建并作为该员工后续记忆的一部分。";

  if (fileName === "AGENTS.MD") {
    return `用于描述该员工的角色定位、协作边界与执行准则，帮助它明确“该怎么工作”。${suffix}`;
  }
  if (fileName === "MEMORY.MD") {
    return `用于沉淀该员工需要长期记住的背景信息、偏好和稳定事实，帮助它持续保持上下文一致。${suffix}`;
  }
  if (fileName === "IDENTITY.MD") {
    return `用于定义该员工的身份、职责和表达风格，帮助它在多轮任务中维持稳定人设与职责感。${suffix}`;
  }
  if (fileName === "SOUL.MD") {
    return `用于记录该员工更偏长期的价值观、行为倾向和判断基调，帮助它在复杂任务中保持一致决策风格。${suffix}`;
  }
  if (fileName === "BOOTSTRAP.MD") {
    return `用于存放该员工启动时需要先读取的基础说明，帮助它快速进入正确的工作状态。${suffix}`;
  }
  if (fileName === "HEARTBEAT.MD") {
    return `用于记录该员工的运行节奏、巡检点或持续关注事项，帮助它维持稳定执行节拍。${suffix}`;
  }
  if (fileName === "TOOLS.MD") {
    return `用于记录该员工可调用工具及使用约束，帮助它知道“能用什么、该怎么用”。${suffix}`;
  }
  if (record.scope.includes("记忆记录")) {
    return `用于补充该员工的具体记忆条目或历史沉淀，帮助它保留更细粒度的长期信息。${suffix}`;
  }

  return `该文件用于补充 ${record.owner} 的长期工作记忆与上下文设定，帮助它在后续任务中保持连续性。${suffix}`;
});
const memoryEditorModeLabel = computed(() =>
  !memoryDraft.value.sourcePath ? "选择记忆文件" : selectedMemoryRecord.value?.exists === false ? "创建并保存记忆文件" : "保存记忆文件"
);
const documentEditorModeLabel = computed(() => {
  if (!documentDraft.value.source) return "选择文档文件";
  const exists = selectedDocumentRecord.value?.exists === false;
  if (activeResourceModal.value === "skill") return exists ? "创建并保存 SKILL.md" : "保存 SKILL.md";
  if (activeResourceModal.value === "tool") return exists ? "创建并保存 TOOLS.md" : "保存 TOOLS.md";
  return exists ? "创建并保存文档文件" : "保存文档文件";
});
const taskStatusSummary = computed(() => ({
  scheduled: taskRecords.value.filter((item) => item.statusKind === "scheduled").length,
  late: taskRecords.value.filter((item) => item.statusKind === "late").length,
  disabled: taskRecords.value.filter((item) => item.statusKind === "disabled").length
}));
const sortedTaskRecords = computed(() =>
  [...taskRecords.value].sort((left, right) => {
    const statusWeight = getTaskStatusWeight(left.statusKind) - getTaskStatusWeight(right.statusKind);
    if (statusWeight !== 0) {
      return statusWeight;
    }

    const leftRunAt = left.nextRunAtMs ?? Number.MAX_SAFE_INTEGER;
    const rightRunAt = right.nextRunAtMs ?? Number.MAX_SAFE_INTEGER;
    if (leftRunAt !== rightRunAt) {
      return leftRunAt - rightRunAt;
    }

    return (right.updatedAtMs ?? 0) - (left.updatedAtMs ?? 0);
  })
);
const nextTaskDueRecord = computed(() => {
  const pending = sortedTaskRecords.value.filter((item) => item.enabled && item.nextRunAtMs !== null);
  return pending[0] ?? null;
});
const taskBoardMetrics = computed(() => {
  const now = Date.now();
  const pending = taskRecords.value.filter((item) => item.enabled);
  const dueSoon = pending.filter((item) => item.nextRunAtMs !== null && item.nextRunAtMs <= now + 12 * 60 * 60 * 1000).length;
  const overdue = pending.filter((item) => item.nextRunAtMs !== null && item.nextRunAtMs < now).length;
  const unassigned = pending.filter((item) => item.agentId.trim() === "" || item.agentId === "未标注").length;

  return {
    total: taskRecords.value.length,
    pending: pending.length,
    dueSoon,
    overdue,
    unassigned
  };
});
const taskScheduleCards = computed(() => {
  const now = Date.now();
  const todayEnd = new Date(now);
  todayEnd.setHours(23, 59, 59, 999);
  const todayEndTime = todayEnd.getTime();
  const tomorrowEnd = new Date(todayEndTime);
  tomorrowEnd.setDate(tomorrowEnd.getDate() + 1);

  const today = sortedTaskRecords.value.filter((item) => item.enabled && item.nextRunAtMs !== null && item.nextRunAtMs <= todayEndTime);
  const tomorrow = sortedTaskRecords.value.filter(
    (item) => item.enabled && item.nextRunAtMs !== null && item.nextRunAtMs > todayEndTime && item.nextRunAtMs <= tomorrowEnd.getTime()
  );
  const disabled = sortedTaskRecords.value.filter((item) => item.statusKind === "disabled");

  return [
    {
      id: "today",
      title: "今日与下一批排程",
      subtitle: today.length > 0 ? `${today.length} 条待执行` : "今天没有紧急排程",
      tone: "today",
      records: today
    },
    {
      id: "tomorrow",
      title: "下一批待推进",
      subtitle: tomorrow.length > 0 ? `${tomorrow.length} 条排程` : "明日排程较轻",
      tone: "upcoming",
      records: tomorrow
    },
    {
      id: "disabled",
      title: "停用与暂停任务",
      subtitle: disabled.length > 0 ? `${disabled.length} 条未启用` : "当前没有停用项",
      tone: "blocked",
      records: disabled
    }
  ];
});
const taskBoardGroups = computed(() => [
  {
    key: "late",
    label: "待执行",
    summary: "运行时间已到或已过，应该优先确认是否被执行。",
    count: taskStatusSummary.value.late,
    records: sortedTaskRecords.value.filter((item) => item.statusKind === "late")
  },
  {
    key: "scheduled",
    label: "调度器",
    summary: "cron/jobs.json 中已启用的任务，将按下一次运行时间排序。",
    count: taskStatusSummary.value.scheduled,
    records: sortedTaskRecords.value.filter((item) => item.statusKind === "scheduled")
  },
  {
    key: "disabled",
    label: "已停用",
    summary: "仍保留在 cron/jobs.json 中，但当前不会继续运行。",
    count: taskStatusSummary.value.disabled,
    records: sortedTaskRecords.value.filter((item) => item.statusKind === "disabled")
  }
]);
const cronTaskAgents = computed(() => {
  const agents = new Set<string>();
  for (const task of taskRecords.value) {
    const id = task.agentId.trim();
    if (id && id !== "未标注") agents.add(id);
  }
  return Array.from(agents).sort();
});
const cronTaskStatusCounts = computed(() => {
  const agentFilter = cronTaskAgentFilter.value;
  const filtered = agentFilter === "all" ? taskRecords.value : taskRecords.value.filter((t) => t.agentId === agentFilter);
  return {
    all: filtered.length,
    late: filtered.filter((t) => t.statusKind === "late").length,
    scheduled: filtered.filter((t) => t.statusKind === "scheduled").length,
    disabled: filtered.filter((t) => t.statusKind === "disabled").length
  };
});
const filteredCronTasks = computed(() => {
  let tasks = [...sortedTaskRecords.value];
  if (cronTaskAgentFilter.value !== "all") {
    tasks = tasks.filter((t) => t.agentId === cronTaskAgentFilter.value);
  }
  if (cronTaskTab.value === "late") {
    tasks = tasks.filter((t) => t.statusKind === "late");
  } else if (cronTaskTab.value === "scheduled") {
    tasks = tasks.filter((t) => t.statusKind === "scheduled");
  } else if (cronTaskTab.value === "disabled") {
    tasks = tasks.filter((t) => t.statusKind === "disabled");
  }
  return tasks;
});
const controlCenterCards = computed(() => [
  {
    label: "员工编制",
    value: `${staffMembers.value.length}`,
    description: staffSnapshotDetail.value
  },
  {
    label: "记忆条目",
    value: `${memoryRecords.value.length}`,
    description: `Main ${memoryStatusSummary.value.main} 条，员工记忆 ${memoryStatusSummary.value.agents} 条。`
  },
  {
    label: "文档资产",
    value: `${documentRecords.value.length}`,
    description: `Main 文档 ${documentStatusSummary.value.main} 份，员工文档 ${documentStatusSummary.value.agents} 份。`
  },
  {
    label: "执行任务",
    value: `${taskRecords.value.length}`,
    description: `${taskStatusSummary.value.scheduled} 条已启用，${taskStatusSummary.value.late} 条待执行。`
  }
]);
function getSessionSummaryGroupId(log: RequestLog) {
  if (log.sessionId.startsWith("runtime-")) {
    return `runtime-group:${log.platformId}`;
  }

  return log.sessionId;
}

const sessionSummaries = computed<SessionSummary[]>(() => {
  const map = new Map<string, SessionSummary>();

  for (const log of filteredRequestLogs.value) {
    const sessionGroupId = getSessionSummaryGroupId(log);
    const current = map.get(sessionGroupId);
    const preview = summarizeLogText(log);

    if (current) {
      current.lastAt = Math.max(current.lastAt, log.createdAt);
      current.requestCount += 1;
      current.failureCount += isFailedLog(log) ? 1 : 0;
      current.totalDuration += log.duration;
      current.totalTokens += getEffectiveTotalTokens(log);
      current.promptTokens += getEffectivePromptTokens(log);
      current.completionTokens += getEffectiveCompletionTokens(log);
      current.cacheReadInputTokens += log.cacheReadInputTokens ?? 0;
      current.logs.push(log);
      if (preview.length > current.previewText.length) {
        current.previewText = preview;
      }
      if (log.streamSummary?.trim() || log.responseBody?.trim()) {
        current.fullOutput = buildSessionOutput(current.logs);
      }
      if (log.error?.trim()) {
        current.latestError = log.error.trim();
      }
    } else {
      map.set(sessionGroupId, {
        id: sessionGroupId,
        startedAt: log.createdAt,
        lastAt: log.createdAt,
        platformName: log.platformName,
        requestCount: 1,
        failureCount: isFailedLog(log) ? 1 : 0,
        totalDuration: log.duration,
        totalTokens: getEffectiveTotalTokens(log),
        promptTokens: getEffectivePromptTokens(log),
        completionTokens: getEffectiveCompletionTokens(log),
        cacheReadInputTokens: log.cacheReadInputTokens ?? 0,
        logs: [log],
        previewText: preview,
        fullOutput: buildSessionOutput([log]),
        latestError: log.error?.trim() || null
      });
    }
  }

  return Array.from(map.values()).sort((left, right) => right.lastAt - left.lastAt);
});
const failureSummaries = computed<FailureSummary[]>(() => {
  const map = new Map<string, FailureSummary>();

  for (const log of filteredRequestLogs.value.filter((item) => isFailedLog(item))) {
    const title = normalizeFailureTitle(log);
    const nextStep = getFailureNextStep(log);
    const key = `${log.responseStatus}:${title}`;
    const current = map.get(key);

    if (current) {
      current.count += 1;
      current.latestAt = Math.max(current.latestAt, log.createdAt);
      current.logs.push(log);
      if (!current.platformNames.includes(log.platformName)) {
        current.platformNames.push(log.platformName);
      }
    } else {
      map.set(key, {
        key,
        title,
        count: 1,
        latestAt: log.createdAt,
        platformNames: [log.platformName],
        logs: [log],
        nextStep
      });
    }
  }

  return Array.from(map.values()).sort((left, right) => {
    if (right.count !== left.count) {
      return right.count - left.count;
    }

    return right.latestAt - left.latestAt;
  });
});
const selectedTimelineLog = computed(() => {
  if (!timelineEntries.value.length) {
    return null;
  }

  return timelineEntries.value.find((log) => log.id === selectedLogId.value) ?? timelineEntries.value[0];
});
const selectedSession = computed(() => {
  if (!sessionSummaries.value.length) {
    return null;
  }

  return sessionSummaries.value.find((session) => session.id === selectedSessionId.value) ?? sessionSummaries.value[0];
});
const selectedFailure = computed(() => {
  if (!failureSummaries.value.length) {
    return null;
  }

  return failureSummaries.value.find((failure) => failure.key === selectedFailureKey.value) ?? failureSummaries.value[0];
});
const selectedSessionLog = computed(() => {
  const session = selectedSession.value;
  if (!session || !session.logs.length) {
    return null;
  }

  return session.logs.find((log) => log.id === selectedSessionLogId.value) ?? session.logs[0];
});
const sessionOverlayLog = computed(() => {
  const session = selectedSession.value;
  if (!session || !session.logs.length || !sessionOverlayLogId.value) {
    return null;
  }

  return session.logs.find((log) => log.id === sessionOverlayLogId.value) ?? null;
});
const selectedFailureLog = computed(() => selectedFailure.value?.logs[0] ?? null);
const latestRequestLog = computed(() => timelineEntries.value[0] ?? null);
const overviewStatusCards = computed(() => [
  {
    label: "定时任务数",
    value: `${scheduledTaskCount.value}`,
    description: scheduledTaskCount.value > 0 ? "后台任务正在按计划执行。" : "当前版本暂未配置自动任务。"
  },
  {
    label: "订阅统计",
    value: `${configuredSubscriptionCount.value}/${platforms.value.length}`,
    description:
      platforms.value.length > 0
        ? `已配置密钥 ${configuredSubscriptionCount.value} 个，本地代理开启 ${enabledPlatformCount.value} 个。`
        : "还没有接入平台，暂时没有可统计的订阅。"
  },
  {
    label: "默认平台",
    value: activePlatform.value?.name ?? openClawDefaultPlatformName,
    description: activePlatform.value
      ? `${activePlatform.value.protocol.toUpperCase()} · ${activePlatform.value.model}`
      : "可在代理配置中设置默认接入平台。"
  },
  {
    label: "最近调用",
    value: latestRequestLog.value ? formatTime(latestRequestLog.value.createdAt) : "暂无记录",
    description: latestRequestLog.value
      ? `${latestRequestLog.value.platformName} · ${isFailedLog(latestRequestLog.value) ? "失败" : "成功"}`
      : "还没有请求日志，先发起一次对话试试。"
  },
  {
    label: "当前员工",
    value: `${staffMembers.value.length}`,
    description: staffSnapshotDetail.value
  },
  {
    label: "当前任务",
    value: `${taskRecords.value.length}`,
    description: `${taskStatusSummary.value.scheduled} 条已启用，${taskStatusSummary.value.late} 条待执行。`
  }
]);

let rafId = 0;
let idleTimer = 0;
let animationStartedAt = 0;
let queuedAnimationName: AnimationName | null = null;
let lastInteractionAt = 0;
let idleShowcaseIndex = 0;
let dragPointerId: number | null = null;
let dragStart = { x: 0, y: 0, petX: 0, petY: 0 };
/** 多显示器：当前窗口所在显示器索引 */
let currentMonitorIndex = 0;
/** 多显示器：窗口刚切到另一屏后，下一帧用此偏移校正宠物位置 */
let pendingDragOffset: { x: number; y: number } | null = null;
/** 可用显示器列表（逻辑坐标，与 screenX/screenY 一致） */
let availableMonitors: Array<{ position: [number, number]; size: [number, number] }> = [];
let windowPointerMoveListener: ((event: PointerEvent) => void) | null = null;
let windowPointerUpListener: ((event: PointerEvent) => void) | null = null;
let cursorPassThroughTimer = 0;
let ignoreCursorEvents = false;
let chatAnimationFrame = 0;
let chatAnimationStartedAt = 0;
let panelAnimationFrame = 0;
let panelAnimationStartedAt = 0;
let bubbleAnimationFrame = 0;
let bubbleAnimationStartedAt = 0;
let chatMovePointerId: number | null = null;
let chatResizePointerId: number | null = null;
let chatMoveStart = { x: 0, y: 0, panelX: 0, panelY: 0 };
let chatResizeStart = { x: 0, y: 0, width: 0, height: 0 };
let panelMovePointerId: number | null = null;
let panelResizePointerId: number | null = null;
let panelMoveStart = { x: 0, y: 0, panelX: 0, panelY: 0 };
let panelResizeStart = { x: 0, y: 0, width: 0, height: 0 };
let gatewayMonitorTimer = 0;
let runtimeLogTimer = 0;
let staffSnapshotPollTimer = 0;
let lobsterInstallProgressTimer = 0;
let lobsterInstallGuideRefreshToken = 0;
let runtimeLogFollowTimer = 0;
let runtimeLogRefreshTask: Promise<RequestLog[]> | null = null;
let runtimeLogLastFingerprint = "";
let runtimeLogFollowActiveUntil = 0;
let roleWorkflowDetailRequestToken = 0;
let lastSelectInteractionAt = 0;
let unlistenConsoleOpenEvent: (() => void) | null = null;
let activeVoiceAudio: HTMLAudioElement | null = null;
const activeVoiceMessageId = ref<string | null>(null);
const audioPayloadCache = new Map<string, AudioFilePayload>();
const runtimeLogPollIntervalMs = 2500;
const staffSnapshotPollIntervalMs = 3000;
const runtimeLogFollowWindowMs = 4000;
let systemThemeMediaQuery: MediaQueryList | null = null;
let systemThemeMediaListener: ((event: MediaQueryListEvent) => void) | null = null;

type TauriWindowApi = {
  label?: string;
  close: () => Promise<void> | void;
  destroy: () => Promise<void> | void;
  show?: () => Promise<void> | void;
  setFocus?: () => Promise<void> | void;
  setAlwaysOnTop?: (value: boolean) => Promise<void> | void;
  startDragging?: () => Promise<void> | void;
  setIgnoreCursorEvents: (value: boolean, options?: { forward?: boolean }) => Promise<void> | void;
};

type TauriNamespace = {
  app?: {
    exit?: (code?: number) => Promise<void> | void;
  };
  autostart?: {
    enable?: () => Promise<void> | void;
    disable?: () => Promise<void> | void;
    isEnabled?: () => Promise<boolean> | boolean;
  };
  core?: {
    invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
  };
  window?: {
    getCurrentWindow?: () => TauriWindowApi;
    cursorPosition?: () => Promise<{ x: number; y: number }>;
    WebviewWindow?: new (
      label: string,
      options?: {
        url?: string;
        title?: string;
        width?: number;
        height?: number;
        minWidth?: number;
        minHeight?: number;
        center?: boolean;
        focus?: boolean;
        alwaysOnTop?: boolean;
        transparent?: boolean;
        decorations?: boolean;
        resizable?: boolean;
      }
    ) => unknown;
  };
  event?: {
    listen?: (
      event: string,
      handler: (event: { payload: unknown }) => void
    ) => Promise<() => void> | (() => void);
  };
};

function parseConsoleSection(raw: string | null): ConsoleSection | null {
  if (
    raw === "overview" ||
    raw === "platforms" ||
    raw === "staff" ||
    raw === "role-workflow" ||
    raw === "skill-market" ||
    raw === "channels" ||
    raw === "bindings" ||
    raw === "tasks"
  ) {
    return raw;
  }
  return null;
}

const isConsoleWindowMode = (() => {
  if (typeof window === "undefined") {
    return false;
  }
  const tauriWindow = window as Window & {
    __CLAWPET_CONSOLE_MODE?: boolean;
    __TAURI__?: TauriNamespace;
  };
  if (tauriWindow.__CLAWPET_CONSOLE_MODE) {
    return true;
  }
  let queryWindowLabel: string | null = null;
  try {
    queryWindowLabel = new URL(window.location.href).searchParams.get("window");
  } catch {
    queryWindowLabel = null;
  }
  if (queryWindowLabel === "console") {
    return true;
  }
  const label = tauriWindow.__TAURI__?.window?.getCurrentWindow?.().label;
  return label === "console";
})();

const isWindowsRuntime = (() => {
  if (typeof navigator === "undefined") {
    return false;
  }
  const platform = (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform;
  return /windows/i.test(platform ?? navigator.userAgent);
})();

const initialConsoleSection = (() => {
  if (typeof window === "undefined") {
    return null;
  }
  const tauriWindow = window as Window & {
    __CLAWPET_CONSOLE_SECTION?: string;
  };
  const bootstrapped = parseConsoleSection(tauriWindow.__CLAWPET_CONSOLE_SECTION ?? null);
  if (bootstrapped) {
    return bootstrapped;
  }
  if (!isConsoleWindowMode) {
    return null;
  }
  try {
    return parseConsoleSection(new URL(window.location.href).searchParams.get("section"));
  } catch {
    return null;
  }
})();

function createMessageId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function createSessionId() {
  return `session-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function safeJson(value: unknown) {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function extractAudioPath(text: string) {
  const match = text.match(/(?:^|\s|MEDIA:)(\/[^\s"'<>]+\.(?:mp3|wav|m4a|aac|ogg|flac))(?:$|\s)/i);
  return match?.[1] ?? null;
}

function getAudioMessagePath(message: ChatMessage) {
  return extractAudioPath(message.text);
}

function isAudioMessage(message: ChatMessage) {
  return Boolean(getAudioMessagePath(message));
}

function getAudioMessageLabel(message: ChatMessage) {
  const audioPath = getAudioMessagePath(message);
  if (!audioPath) {
    return message.text;
  }

  return audioPath.split("/").filter(Boolean).pop() ?? "语音消息";
}

const chatTimeFormatter =
  typeof Intl !== "undefined"
    ? new Intl.DateTimeFormat("zh-CN", {
      hour: "2-digit",
      minute: "2-digit"
    })
    : null;

function extractTimestampFromMessageId(id: string) {
  const matched = id.match(/-(\d{13})-[a-z0-9]+$/i);
  if (!matched) {
    return null;
  }
  const parsed = Number(matched[1]);
  return Number.isFinite(parsed) ? parsed : null;
}

function getMessageTimestamp(message: ChatMessage) {
  if (typeof message.createdAt === "number" && Number.isFinite(message.createdAt)) {
    return message.createdAt;
  }
  return extractTimestampFromMessageId(message.id);
}

function getMessageTimeLabel(message: ChatMessage) {
  const timestamp = getMessageTimestamp(message);
  if (!timestamp) {
    return "";
  }
  if (chatTimeFormatter) {
    return chatTimeFormatter.format(timestamp);
  }
  return new Date(timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function getMessageRoleLabel(message: ChatMessage) {
  if (message.role === "user") {
    return "你";
  }
  if (message.role === "system") {
    return "系统";
  }
  return activeChatAgent.value ? stripRoleLabel(activeChatAgent.value.displayName) : "ClawPet";
}

function isAudioMessagePlaying(messageId: string) {
  return activeVoiceMessageId.value === messageId;
}

async function loadAudioPayload(path: string) {
  const cached = audioPayloadCache.get(path);
  if (cached) {
    return cached;
  }

  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    throw new Error("当前环境不支持读取本地音频文件。");
  }

  const payload = (await invoke("read_local_audio_file", { path })) as AudioFilePayload;
  audioPayloadCache.set(path, payload);
  return payload;
}

function stopVoicePlayback() {
  if (activeVoiceAudio) {
    activeVoiceAudio.pause();
    activeVoiceAudio.currentTime = 0;
    activeVoiceAudio = null;
  }

  activeVoiceMessageId.value = null;
}

function buildRuntimeToolMessage(log: RequestLog): ChatMessage {
  const text = (log.streamSummary?.trim() || log.responseBody?.trim() || "").trim();
  return {
    id: `runtime-tool-${log.id}`,
    role: "assistant",
    text,
    status: "done",
    createdAt: log.createdAt
  };
}

function insertRuntimeToolMessages(beforeMessageId: string, logs: RequestLog[], afterMs: number) {
  const pendingIndex = chatMessages.value.findIndex((message) => message.id === beforeMessageId);
  if (pendingIndex < 0) {
    return;
  }

  const toolMessages = logs
    .filter((log) => log.platformId.startsWith("openclaw-runtime-"))
    .filter((log) => log.method.startsWith("TOOL:"))
    .filter((log) => log.createdAt >= afterMs)
    .filter((log) => !chatMessages.value.some((message) => message.id === `runtime-tool-${log.id}`))
    .filter((log) => Boolean(extractAudioPath(log.streamSummary || log.responseBody || "")))
    .sort((left, right) => left.createdAt - right.createdAt)
    .map(buildRuntimeToolMessage);

  if (!toolMessages.length) {
    return;
  }

  chatMessages.value.splice(pendingIndex, 0, ...toolMessages);
}

async function handleAudioMessageClick(message: ChatMessage) {
  const path = getAudioMessagePath(message);
  if (!path) {
    return;
  }

  if (isAudioMessagePlaying(message.id)) {
    stopVoicePlayback();
    return;
  }

  try {
    stopVoicePlayback();
    const payload = await loadAudioPayload(path);
    const audio = new Audio(payload.dataUrl);
    activeVoiceAudio = audio;
    activeVoiceMessageId.value = message.id;
    audio.addEventListener("ended", () => {
      if (activeVoiceAudio === audio) {
        activeVoiceAudio = null;
        activeVoiceMessageId.value = null;
      }
    });
    audio.addEventListener("pause", () => {
      if (activeVoiceAudio === audio && audio.currentTime < audio.duration) {
        activeVoiceAudio = null;
        activeVoiceMessageId.value = null;
      }
    });
    await audio.play();
    statusText.value = `正在播放 ${payload.fileName}。`;
  } catch (error) {
    stopVoicePlayback();
    statusText.value = error instanceof Error ? error.message : "语音播放失败。";
  }
}

function normalizeChatMessage(value: unknown): ChatMessage | null {
  if (!value || typeof value !== "object") {
    return null;
  }

  const message = value as Partial<ChatMessage>;
  const role = message.role;
  const status = message.status;

  if (
    typeof message.id !== "string" ||
    typeof message.text !== "string" ||
    (role !== "assistant" && role !== "user" && role !== "system") ||
    (status !== "pending" && status !== "done" && status !== "error")
  ) {
    return null;
  }

  return {
    id: message.id,
    role,
    text: message.text,
    status,
    createdAt: typeof message.createdAt === "number" && Number.isFinite(message.createdAt)
      ? message.createdAt
      : undefined
  };
}

function getStableChatMessages(messages: ChatMessage[]) {
  return messages.filter((message) => message.status !== "pending");
}

function loadChatHistory(agentId: string | null = null) {
  try {
    const raw = safeLocalStorageGetItem(chatStorageKeyFor(agentId));
    if (!raw) {
      return createDefaultChatMessages();
    }

    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return createDefaultChatMessages();
    }

    const messages = parsed
      .map((item) => normalizeChatMessage(item))
      .filter((message): message is ChatMessage => message !== null)
      .filter((message) => message.status !== "pending");

    return messages.length > 0 ? messages : createDefaultChatMessages();
  } catch {
    return createDefaultChatMessages();
  }
}

function loadStoredSessionId(agentId: string | null = null) {
  const value = safeLocalStorageGetItem(sessionStorageKeyFor(agentId));
  if (value) {
    return value;
  }

  const next = createSessionId();
  safeLocalStorageSetItem(sessionStorageKeyFor(agentId), next);
  return next;
}

function persistChatHistory(agentId: string | null = null) {
  try {
    safeLocalStorageSetItem(chatStorageKeyFor(agentId), JSON.stringify(getStableChatMessages(chatMessages.value)));
    safeLocalStorageSetItem(sessionStorageKeyFor(agentId), currentSessionId.value);
  } catch {
    // Ignore storage errors so chat remains usable even in restricted environments.
  }
}

function stripRoleLabel(name: string) {
  return name.replace(/[（(][^）)]*[）)]$/, "").trim();
}

function buildBoundPetChatAgentId(petId: string) {
  return `${BOUND_PET_CHAT_PREFIX}${petId}`;
}

function isBoundPetChatAgentId(agentId: string | null) {
  return Boolean(agentId && agentId.startsWith(BOUND_PET_CHAT_PREFIX));
}

function savePetBindingCode() {
  const normalized = normalizeBindingCode(bindingCodeDraft.value);
  if (!normalized) {
    statusText.value = "绑定码不能为空。";
    return;
  }
  petBindingCode.value = normalized;
  bindingCodeDraft.value = normalized;
  safeLocalStorageSetItem(PET_BIND_CODE_STORAGE_KEY, normalized);
  statusText.value = "绑定码已保存。";
}

function regeneratePetBindingCode() {
  const nextCode = createBindingCode();
  petBindingCode.value = nextCode;
  bindingCodeDraft.value = nextCode;
  safeLocalStorageSetItem(PET_BIND_CODE_STORAGE_KEY, nextCode);
  statusText.value = "已重新生成绑定码。";
}

async function copyPetBindingCode() {
  const value = petBindingCode.value || normalizeBindingCode(bindingCodeDraft.value);
  if (!value) {
    statusText.value = "当前没有可复制的绑定码。";
    return;
  }
  const clipboard = globalThis.navigator?.clipboard;
  if (!clipboard?.writeText) {
    statusText.value = "当前环境不支持自动复制，请手动复制绑定码。";
    return;
  }
  try {
    await clipboard.writeText(value);
    statusText.value = "绑定码已复制。";
  } catch {
    statusText.value = "复制失败，请手动复制绑定码。";
  }
}

function bindRemotePetByCode() {
  const normalized = normalizeBindingCode(incomingBindingCode.value);
  if (!normalized) {
    statusText.value = "请输入有效绑定码。";
    return;
  }
  if (normalized === petBindingCode.value) {
    statusText.value = "不能绑定自己的宠物。";
    return;
  }
  if (boundPets.value.some((pet) => pet.bindingCode === normalized)) {
    statusText.value = "该绑定码已在绑定列表中。";
    return;
  }

  const nextPet: BoundPetConnection = {
    id: createMessageId("bound-pet"),
    petName: `远程宠物 ${boundPets.value.length + 1}`,
    ownerLabel: "远程用户",
    bindingCode: normalized,
    linkedAt: Date.now(),
    capabilities: createBoundPetCapabilities()
  };
  boundPets.value = [nextPet, ...boundPets.value];
  persistBoundPets();
  incomingBindingCode.value = "";
  statusText.value = `已绑定 ${nextPet.petName}，可在聊天窗口直接互聊。`;
}

function removeBoundPet(petId: string) {
  const removed = boundPets.value.find((pet) => pet.id === petId);
  boundPets.value = boundPets.value.filter((pet) => pet.id !== petId);
  persistBoundPets();

  const activeId = activeChatAgentId.value;
  if (activeId === buildBoundPetChatAgentId(petId)) {
    switchChatAgent(null);
  }
  statusText.value = removed ? `已解除 ${removed.petName} 的绑定。` : "已解除绑定。";
}

function openBoundPetChat(petId: string) {
  switchChatAgent(buildBoundPetChatAgentId(petId));
  toggleChatPanel(true);
}

function setBoundPetCapability(petId: string, capabilityId: string, enabled: boolean) {
  const next = boundPets.value.map((pet) => {
    if (pet.id !== petId) return pet;
    return {
      ...pet,
      capabilities: pet.capabilities.map((capability) =>
        capability.id === capabilityId ? { ...capability, enabled } : capability
      )
    };
  });
  boundPets.value = next;
  persistBoundPets();
}

function updateBoundPetName(petId: string, name: string) {
  const trimmed = name.trim();
  const nextName = trimmed || "远程宠物";
  boundPets.value = boundPets.value.map((pet) => (pet.id === petId ? { ...pet, petName: nextName } : pet));
  persistBoundPets();
}

function canBoundPetUseAgent(pet: BoundPetConnection, agentId: string | null) {
  const key = agentId ?? "__main__";
  return pet.capabilities.some((capability) => capability.id === key && capability.enabled);
}

function resolveBoundPetResponseText(pet: BoundPetConnection, text: string) {
  const commandMatch = text.match(/^\/agent\s+(\S+)\s+([\s\S]+)$/i);
  if (!commandMatch) {
    const presets = [
      "收到，我这边已经同步到远程宠物频道。",
      "这条消息我看到了，我们可以继续跨设备协作。",
      "我在远程端已记录，下一步可以直接派发给 agent。"
    ];
    return presets[Math.floor(Math.random() * presets.length)];
  }

  const targetToken = commandMatch[1].trim();
  const prompt = commandMatch[2].trim();
  if (!prompt) {
    return "请在 /agent 指令后补充具体任务。";
  }

  const targetAgentId =
    targetToken === "main" || targetToken === "default" || targetToken === "主对话"
      ? null
      : staffMembers.value.find((member) => member.agentId === targetToken)?.agentId ?? targetToken;
  const allowed = canBoundPetUseAgent(pet, targetAgentId);
  if (!allowed) {
    return `当前未授权 ${targetToken}，请在绑定列表里开启对应 Agent 能力。`;
  }

  return `已收到指令，准备通过 ${targetToken} 执行：${prompt}`;
}

async function submitBoundPetChat(pet: BoundPetConnection, text: string, pendingAttachments: ChatAttachment[]) {
  const startedAt = Date.now();
  const pendingId = createMessageId("assistant");
  chatMessages.value.push({
    id: createMessageId("user"),
    role: "user",
    text: text || "(附件)",
    status: "done",
    createdAt: startedAt,
    attachments: pendingAttachments.length > 0 ? pendingAttachments : undefined
  });
  chatMessages.value.push({
    id: pendingId,
    role: "assistant",
    text: `${pet.petName} 正在同步中...`,
    status: "pending",
    createdAt: Date.now()
  });

  chatInput.value = "";
  chatAttachments.value = [];
  isSending.value = true;
  noteInteraction();
  applyBaseAnimation(true);
  startBubbleAnimation();
  scrollMessagesToBottom();

  const commandMatch = text.match(/^\/agent\s+(\S+)\s+([\s\S]+)$/i);
  try {
    let responseText = resolveBoundPetResponseText(pet, text);
    if (commandMatch && responseText.startsWith("已收到指令，准备通过")) {
      const targetToken = commandMatch[1].trim();
      const prompt = commandMatch[2].trim();
      const targetAgentId =
        targetToken === "main" || targetToken === "default" || targetToken === "主对话"
          ? null
          : staffMembers.value.find((member) => member.agentId === targetToken)?.agentId ?? targetToken;
      const response = await sendOpenClawChat([{ role: "user", content: prompt }], { agentId: targetAgentId });
      responseText = `[${targetToken}] ${response.text}`;
    } else {
      await new Promise((resolve) => window.setTimeout(resolve, 420));
    }

    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = responseText;
      pendingMessage.status = "done";
    }
    statusText.value = `${pet.petName} 已回复。`;
  } catch (error) {
    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = error instanceof Error ? error.message : "远程宠物回复失败。";
      pendingMessage.status = "error";
    }
    statusText.value = "远程宠物通信失败，请稍后重试。";
  } finally {
    isSending.value = false;
    applyBaseAnimation();
    startBubbleAnimation();
    scrollMessagesToBottom();
  }
}

function switchChatAgent(agentId: string | null) {
  if (agentId === activeChatAgentId.value) return;
  agentChatHistories.value[activeChatAgentId.value ?? "__main__"] = [...chatMessages.value];
  persistChatHistory(activeChatAgentId.value);
  activeChatAgentId.value = agentId;
  const cached = agentChatHistories.value[agentId ?? "__main__"];
  chatMessages.value = cached && cached.length > 0 ? [...cached] : loadChatHistory(agentId);
  currentSessionId.value = loadStoredSessionId(agentId);
  chatInput.value = "";
  chatAttachments.value = [];
  if (isBoundPetChatAgentId(agentId)) {
    const pet = boundPets.value.find((item) => buildBoundPetChatAgentId(item.id) === agentId);
    statusText.value = pet ? `已切换到 ${pet.petName} 的远程会话。` : "已切换远程宠物会话。";
  }
  scrollMessagesToBottom();
}

function loadProxyPort() {
  const raw = safeLocalStorageGetItem("keai.desktop-pet.proxy-port");
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 5005;
}

function loadPlatformProxyEnabledMap() {
  const raw = safeLocalStorageGetItem(PLATFORM_PROXY_ENABLED_STORAGE_KEY);
  if (!raw) {
    return {} as Record<string, boolean>;
  }

  try {
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    return Object.fromEntries(
      Object.entries(parsed).filter((entry): entry is [string, boolean] => typeof entry[0] === "string" && typeof entry[1] === "boolean")
    );
  } catch {
    return {} as Record<string, boolean>;
  }
}

function loadPlatformDirectBaseUrlMap() {
  const raw = safeLocalStorageGetItem(PLATFORM_DIRECT_BASEURL_STORAGE_KEY);
  if (!raw) {
    return {} as Record<string, string>;
  }

  try {
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    return Object.fromEntries(
      Object.entries(parsed).filter((entry): entry is [string, string] => typeof entry[0] === "string" && typeof entry[1] === "string")
    );
  } catch {
    return {} as Record<string, string>;
  }
}

function persistPlatformProxyEnabledStates(platformList: PlatformConfig[]) {
  const payload = Object.fromEntries(platformList.map((platform) => [platform.id, Boolean(platform.enabled)]));
  safeLocalStorageSetItem(PLATFORM_PROXY_ENABLED_STORAGE_KEY, JSON.stringify(payload));
}

function persistPlatformDirectBaseUrlStates(baseUrlMap: Record<string, string>) {
  safeLocalStorageSetItem(PLATFORM_DIRECT_BASEURL_STORAGE_KEY, JSON.stringify(baseUrlMap));
}

function isLocalProxyBaseUrl(value: string) {
  return /^https?:\/\/(?:127\.0\.0\.1|localhost)(?::\d+)?\//i.test(value.trim());
}

function normalizeLocalProxyBaseUrlForPersist(value: string) {
  const normalized = normalizeBaseUrl(value);
  if (!isLocalProxyBaseUrl(normalized)) {
    return normalized;
  }

  try {
    const parsed = new URL(normalized);
    let path = parsed.pathname.replace(/\/+$/, "");
    const lower = path.toLowerCase();
    if (lower.endsWith("/v1/chat/completions")) {
      path = path.slice(0, -"/v1/chat/completions".length);
    } else if (lower.endsWith("/chat/completions")) {
      path = path.slice(0, -"/chat/completions".length);
    } else if (lower.endsWith("/v1/responses")) {
      path = path.slice(0, -"/v1/responses".length);
    } else if (lower.endsWith("/responses")) {
      path = path.slice(0, -"/responses".length);
    } else if (lower.endsWith("/v1/messages")) {
      path = path.slice(0, -"/v1/messages".length);
    } else if (lower.endsWith("/messages")) {
      path = path.slice(0, -"/messages".length);
    }

    if (path.toLowerCase().endsWith("/v1")) {
      path = path.slice(0, -"/v1".length);
    }
    parsed.pathname = path || "/";
    parsed.search = "";
    parsed.hash = "";
    return parsed.toString().replace(/\/$/, "");
  } catch {
    return normalized.replace(/\/v1$/i, "");
  }
}

function inferApiPathForBaseUrl(protocol: PlatformProtocol, baseUrl: string, currentApiPath?: string) {
  const normalizedBase = normalizeBaseUrl(baseUrl).toLowerCase();
  const normalizedApiPath = normalizeApiPath(currentApiPath ?? "");
  const usesResponses = normalizedApiPath.includes("responses");
  const endsWithV1 = normalizedBase.endsWith("/v1");

  if (usesResponses) {
    return endsWithV1 ? "/responses" : "/v1/responses";
  }
  if (protocol === "anthropic") {
    return endsWithV1 ? "/messages" : "/v1/messages";
  }
  return endsWithV1 ? "/chat/completions" : "/v1/chat/completions";
}

function easeOutCubic(value: number) {
  return 1 - (1 - value) ** 3;
}

function easeOutExpo(value: number) {
  if (value >= 1) {
    return 1;
  }

  return 1 - 2 ** (-10 * value);
}

function springProgress(frame: number) {
  const normalized = Math.min(Math.max(frame / 18, 0), 1);
  return 1 - Math.exp(-6 * normalized) * Math.cos(8 * normalized);
}

function getFrameInterval(frames: Frame[]) {
  if (frames.length < 2) {
    return 0.16;
  }

  return Math.max(0.016, frames[1].t - frames[0].t);
}

function getAnimationDuration(animation: AnimationDefinition) {
  const frames = animation.config.frames;
  const lastFrame = frames[frames.length - 1];
  return (lastFrame.t + getFrameInterval(frames)) / playbackRate;
}

function noteInteraction() {
  lastInteractionAt = performance.now();
}

function getIdleElapsed(now = performance.now()) {
  return Math.max(0, now - lastInteractionAt);
}

function shouldSleep(now = performance.now()) {
  return getIdleElapsed(now) >= sleepDelayMs;
}

function shouldFollowRuntimeLogAnimation(now = performance.now()) {
  return runtimeLogFollowActiveUntil > now;
}

function resolveBaseAnimationName(now = performance.now()): AnimationName {
  if (isSending.value) {
    return "think";
  }

  if (shouldFollowRuntimeLogAnimation(now)) {
    return "think";
  }

  if (isChatOpen.value && chatInput.value.trim()) {
    return "chat_typing";
  }

  if (isDragging.value) {
    return "act_cute_rotation";
  }

  if (shouldSleep(now)) {
    return "sleep";
  }

  if (isConsoleOpen.value) {
    return "think";
  }

  if (isChatOpen.value) {
    return "smile_and_blink";
  }

  return "smile_blink";
}

function setAnimation(name: AnimationName, nextName?: AnimationName | null) {
  if (currentAnimationName.value === name && queuedAnimationName === (nextName ?? null)) {
    return;
  }

  currentAnimationName.value = name;
  currentFrameIndex.value = 0;
  animationStartedAt = performance.now();
  queuedAnimationName = nextName ?? null;
  statusText.value = actionTips[name];
  window.clearTimeout(idleTimer);
  sound.animation(name, animations[name].loop);

  if (animations[name].loop) {
    if (queuedAnimationName && queuedAnimationName !== name) {
      idleTimer = window.setTimeout(() => {
        const nextAnimation = queuedAnimationName ?? resolveBaseAnimationName();
        setAnimation(nextAnimation);
      }, getAnimationDuration(animations[name]) * 1000);
    } else {
      queueIdleAction();
    }
  }
}

function applyBaseAnimation(force = false) {
  const nextName = resolveBaseAnimationName();

  if (!force && currentAnimationName.value === nextName) {
    return;
  }

  if (!force && !animations[currentAnimationName.value].loop) {
    queuedAnimationName = nextName;
    return;
  }

  setAnimation(nextName);
}

function wakeThenAnimate(name: AnimationName, nextName?: AnimationName | null) {
  const fallbackName = nextName ?? resolveBaseAnimationName();
  const sleeping =
    currentAnimationName.value === "sleep" ||
    shouldSleep() ||
    currentAnimationName.value === "stretch_yawn_and_rub_your_eyes";

  if (sleeping && name !== "sleep" && name !== "stretch_yawn_and_rub_your_eyes") {
    setAnimation("stretch_yawn_and_rub_your_eyes", name);
    return;
  }

  setAnimation(name, fallbackName === name ? null : fallbackName);
}

function scrollMessagesToBottom() {
  void nextTick(() => {
    const scroller = messageScrollerRef.value;
    if (!scroller) {
      return;
    }

    scroller.scrollTop = scroller.scrollHeight;
  });
}

function animatePanel() {
  const elapsedFrames = ((performance.now() - panelAnimationStartedAt) / 1000) * 30;
  const motion = springProgress(elapsedFrames);

  panelMotionValue.value = isConsoleOpen.value ? motion : 1 - motion;

  const shouldContinue = isConsoleOpen.value ? panelMotionValue.value < 0.995 : panelMotionValue.value > 0.005;
  if (shouldContinue) {
    panelAnimationFrame = window.requestAnimationFrame(animatePanel);
    return;
  }

  panelMotionValue.value = isConsoleOpen.value ? 1 : 0;
  panelAnimationFrame = 0;
}

function animateChatPanel() {
  const elapsedFrames = ((performance.now() - chatAnimationStartedAt) / 1000) * 30;
  const motion = springProgress(elapsedFrames);

  chatMotionValue.value = isChatOpen.value ? motion : 1 - motion;

  const shouldContinue = isChatOpen.value ? chatMotionValue.value < 0.995 : chatMotionValue.value > 0.005;
  if (shouldContinue) {
    chatAnimationFrame = window.requestAnimationFrame(animateChatPanel);
    return;
  }

  chatMotionValue.value = isChatOpen.value ? 1 : 0;
  chatAnimationFrame = 0;
}

function startChatAnimation() {
  window.cancelAnimationFrame(chatAnimationFrame);
  chatAnimationStartedAt = performance.now();
  chatAnimationFrame = window.requestAnimationFrame(animateChatPanel);
}

function startPanelAnimation() {
  window.cancelAnimationFrame(panelAnimationFrame);
  panelAnimationStartedAt = performance.now();
  panelAnimationFrame = window.requestAnimationFrame(animatePanel);
}

function animateBubble() {
  const elapsedFrames = ((performance.now() - bubbleAnimationStartedAt) / 1000) * 30;
  bubbleMotionValue.value = interpolate(elapsedFrames, [0, 10], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: easeOutExpo
  });

  if (bubbleMotionValue.value < 0.999) {
    bubbleAnimationFrame = window.requestAnimationFrame(animateBubble);
    return;
  }

  bubbleMotionValue.value = 1;
  bubbleAnimationFrame = 0;
}

function startBubbleAnimation() {
  window.cancelAnimationFrame(bubbleAnimationFrame);
  bubbleMotionValue.value = 0;
  bubbleAnimationStartedAt = performance.now();
  bubbleAnimationFrame = window.requestAnimationFrame(animateBubble);
}

function getBubbleStyle(index: number) {
  const isLatest = index === chatMessages.value.length - 1;
  const progress = isLatest ? bubbleMotionValue.value : 1;

  return {
    opacity: `${progress}`,
    transform: `translateY(${interpolate(progress, [0, 1], [10, 0])}px) scale(${interpolate(progress, [0, 1], [0.96, 1])})`
  };
}

function getPanelMinWidth(prefersWide: boolean) {
  return prefersWide ? 720 : 420;
}

function syncManualChatPlacement(width: number, height: number, left: number, top: number) {
  chatPlacement.value = {
    mode: "manual",
    width,
    height,
    x: left,
    y: top
  };
}

function resetChatPlacement() {
  chatPlacement.value = {
    mode: "auto",
    x: 0,
    y: 0,
    width: 0,
    height: 0
  };
}

function captureCurrentChatPlacement() {
  const rect = chatPanelRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  syncManualChatPlacement(rect.width, rect.height, rect.left, rect.top);
}

function syncManualPanelPlacement(width: number, height: number, left: number, top: number) {
  panelPlacement.value = {
    mode: "manual",
    width,
    height,
    x: left,
    y: top
  };
}

function resetPanelPlacement() {
  panelPlacement.value = {
    mode: "auto",
    x: 0,
    y: 0,
    width: 0,
    height: 0
  };
}

function captureCurrentPanelPlacement() {
  const rect = consolePanelRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  syncManualPanelPlacement(rect.width, rect.height, rect.left, rect.top);
}

function openChatPanel() {
  hideContextMenu();
  noteInteraction();

  if (!isChatOpen.value) {
    isChatOpen.value = true;
    startChatAnimation();
  }

  applyBaseAnimation();
  startBubbleAnimation();
  scrollMessagesToBottom();
}

function toggleChatPanel(nextValue?: boolean) {
  const finalValue = nextValue ?? !isChatOpen.value;
  if (finalValue === isChatOpen.value) {
    return;
  }

  isChatOpen.value = finalValue;
  if (finalValue) {
    noteInteraction();
  }
  startChatAnimation();
  applyBaseAnimation(true);
  statusText.value = finalValue ? "对话窗口已打开。" : "对话窗口已收起。";
}

function normalizeMessageChannelType(value: string): MessageChannelType | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) {
    return null;
  }
  return MESSAGE_CHANNEL_ALIASES[normalized] ?? MESSAGE_CHANNEL_ALIASES[normalized.replace(/\s+/g, "")] ?? null;
}

function getMessageChannelMeta(value: string): MessageChannelCatalogItem | null {
  const normalized = normalizeMessageChannelType(value);
  if (!normalized) {
    return null;
  }
  return messageChannelCatalog.find((item) => item.id === normalized) ?? null;
}

function getMessageChannelDisplayName(value: string) {
  return getMessageChannelMeta(value)?.name ?? value;
}

function getMessageChannelIcon(value: string) {
  return getMessageChannelMeta(value)?.icon ?? "";
}

function getMessageChannelGroup(value: string) {
  return channelGroupsByType.value[value.trim().toLowerCase()] ?? null;
}

function isMessageChannelConfigured(channelId: MessageChannelType) {
  return configuredMessageChannelIds.value.has(channelId);
}

function createNextChannelAccountId(channelType: MessageChannelType, existingIds: string[]) {
  const existing = new Set(existingIds.map((item) => item.trim().toLowerCase()));
  let candidate = `${channelType}-${Math.random().toString(36).slice(2, 10)}`;
  while (existing.has(candidate.toLowerCase())) {
    candidate = `${channelType}-${Math.random().toString(36).slice(2, 10)}`;
  }
  return candidate;
}

function resetChannelConfigSecretVisibility() {
  channelConfigSecretVisibility.value = {};
}

function isChannelConfigSecretVisible(fieldKey: string) {
  return channelConfigSecretVisibility.value[fieldKey] === true;
}

function toggleChannelConfigSecretVisibility(fieldKey: string) {
  channelConfigSecretVisibility.value = {
    ...channelConfigSecretVisibility.value,
    [fieldKey]: !channelConfigSecretVisibility.value[fieldKey]
  };
}

async function handleOpenChannelConfigDocs() {
  const url = activeChannelConfigMeta.value?.docsUrl?.trim();
  if (!url) {
    statusText.value = "当前频道暂未提供文档链接。";
    return;
  }
  const invoke = getTauriApi()?.core?.invoke;
  if (invoke) {
    try {
      await invoke("open_external_url", { url });
      statusText.value = `已打开 ${activeChannelConfigMeta.value?.name ?? "频道"} 配置文档。`;
      return;
    } catch (error) {
      statusText.value = error instanceof Error ? error.message : "打开配置文档失败。";
    }
  }
  if (typeof window !== "undefined") {
    window.open(url, "_blank", "noopener,noreferrer");
    statusText.value = "已尝试在浏览器中打开配置文档。";
  }
}

async function openChannelConfigModal(
  channelType: MessageChannelType,
  accountId: string,
  options?: {
    allowEditAccountId?: boolean;
    loadExisting?: boolean;
    existingAccountIds?: string[];
  }
) {
  const allowEdit = options?.allowEditAccountId === true;
  const loadExisting = options?.loadExisting === true;
  channelConfigEditingType.value = channelType;
  channelConfigEditingAccountId.value = accountId;
  channelConfigAllowEditAccountId.value = allowEdit;
  channelConfigExistingAccountIds.value = options?.existingAccountIds ?? [];
  channelConfigError.value = "";
  channelConfigForm.value = {};
  resetChannelConfigSecretVisibility();

  if (loadExisting) {
    const invoke = getTauriApi()?.core?.invoke;
    if (invoke) {
      try {
        const values = (await invoke("load_openclaw_channel_form_values", {
          channelType,
          accountId
        })) as Record<string, string>;
        channelConfigForm.value = values && typeof values === "object" ? values : {};
      } catch (error) {
        channelConfigForm.value = {};
        channelConfigError.value = error instanceof Error ? error.message : "读取频道配置失败。";
      }
    }
  }

  isChannelConfigModalOpen.value = true;
  void syncCursorPassThrough();
}

function closeChannelConfigModal() {
  isChannelConfigModalOpen.value = false;
  channelConfigEditingType.value = null;
  channelConfigEditingAccountId.value = "";
  channelConfigAllowEditAccountId.value = false;
  channelConfigExistingAccountIds.value = [];
  channelConfigForm.value = {};
  channelConfigError.value = "";
  resetChannelConfigSecretVisibility();
  isChannelConfigSaving.value = false;
  void syncCursorPassThrough();
}

async function handleOpenChannelConfigFromCard(channelType: MessageChannelType) {
  const group = getMessageChannelGroup(channelType);
  if (group && group.accounts.length > 0) {
    const target = group.accounts.find((item) => item.isDefault) ?? group.accounts[0];
    await openChannelConfigModal(channelType, target.accountId, {
      allowEditAccountId: false,
      loadExisting: true,
      existingAccountIds: group.accounts.map((item) => item.accountId)
    });
    return;
  }

  await openChannelConfigModal(channelType, "default", {
    allowEditAccountId: false,
    loadExisting: false,
    existingAccountIds: []
  });
}

async function handleAddChannelAccount(channelTypeRaw: string) {
  const normalized = normalizeMessageChannelType(channelTypeRaw);
  if (!normalized) {
    return;
  }
  const group = getMessageChannelGroup(channelTypeRaw);
  const existingAccountIds = group?.accounts.map((item) => item.accountId) ?? [];
  const nextAccountId = createNextChannelAccountId(normalized, existingAccountIds);
  await openChannelConfigModal(normalized, nextAccountId, {
    allowEditAccountId: true,
    loadExisting: false,
    existingAccountIds
  });
}

async function handleEditChannelAccount(channelTypeRaw: string, accountId: string) {
  const normalized = normalizeMessageChannelType(channelTypeRaw);
  if (!normalized) {
    return;
  }
  const group = getMessageChannelGroup(channelTypeRaw);
  await openChannelConfigModal(normalized, accountId, {
    allowEditAccountId: false,
    loadExisting: true,
    existingAccountIds: group?.accounts.map((item) => item.accountId) ?? []
  });
}

async function handleSaveChannelConfig() {
  if (!channelConfigEditingType.value) {
    return;
  }
  const meta = activeChannelConfigMeta.value;
  if (!meta) {
    return;
  }
  const accountId = channelConfigEditingAccountId.value.trim();
  if (!accountId) {
    channelConfigError.value = "账号 ID 不能为空。";
    return;
  }

  if (
    channelConfigAllowEditAccountId.value &&
    channelConfigExistingAccountIds.value.some((item) => item.trim().toLowerCase() === accountId.toLowerCase())
  ) {
    channelConfigError.value = `账号 ID ${accountId} 已存在。`;
    return;
  }

  const payloadConfig: Record<string, string> = {};
  for (const field of meta.fields ?? []) {
    const value = (channelConfigForm.value[field.key] ?? "").trim();
    if (field.required && !value) {
      channelConfigError.value = `${field.label} 不能为空。`;
      return;
    }
    if (value) {
      payloadConfig[field.key] = value;
    }
  }

  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) {
    channelConfigError.value = "当前环境不支持写入频道配置。";
    return;
  }

  try {
    isChannelConfigSaving.value = true;
    channelConfigError.value = "";
    await invoke("save_openclaw_channel_config", {
      payload: {
        channelType: channelConfigEditingType.value,
        accountId,
        config: payloadConfig
      }
    });
    closeChannelConfigModal();
    await refreshMessageChannelSnapshot();
    statusText.value = `${meta.name} 频道配置已保存。`;
  } catch (error) {
    channelConfigError.value = error instanceof Error ? error.message : "频道配置保存失败。";
  } finally {
    isChannelConfigSaving.value = false;
  }
}

async function handleBindChannelAccount(channelType: string, accountId: string, agentId: string) {
  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) {
    statusText.value = "当前环境不支持修改频道绑定。";
    return;
  }

  try {
    await invoke("save_openclaw_channel_binding", {
      payload: {
        channelType,
        accountId,
        agentId: agentId.trim() ? agentId.trim() : null
      }
    });
    await refreshMessageChannelSnapshot();
    statusText.value = `${channelType} / ${accountId} 绑定已更新。`;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "频道绑定更新失败。";
  }
}

async function handleDeleteChannelAccount(channelType: string, accountId: string) {
  if (typeof window !== "undefined" && typeof window.confirm === "function") {
    const confirmed = window.confirm(`确定删除 ${channelType} 的账号 ${accountId} 吗？`);
    if (!confirmed) {
      return;
    }
  }

  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) {
    statusText.value = "当前环境不支持删除频道账号。";
    return;
  }

  try {
    await invoke("delete_openclaw_channel_account_config", {
      payload: { channelType, accountId }
    });
    await refreshMessageChannelSnapshot();
    statusText.value = `${channelType} / ${accountId} 已删除。`;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "删除频道账号失败。";
  }
}

async function handleDeleteChannel(channelType: string) {
  if (typeof window !== "undefined" && typeof window.confirm === "function") {
    const confirmed = window.confirm(`确定删除频道 ${channelType} 的全部配置吗？`);
    if (!confirmed) {
      return;
    }
  }

  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) {
    statusText.value = "当前环境不支持删除频道配置。";
    return;
  }

  try {
    await invoke("delete_openclaw_channel_config", { channelType });
    await refreshMessageChannelSnapshot();
    statusText.value = `${channelType} 频道配置已删除。`;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "删除频道配置失败。";
  }
}

async function refreshMessageChannelSnapshot() {
  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) {
    channelGroups.value = [];
    statusText.value = "当前环境不支持读取频道配置。";
    return;
  }

  await refreshStaffSnapshot();
  try {
    const result = (await invoke("load_openclaw_channel_accounts_snapshot")) as ChannelAccountsSnapshotResponse;
    channelGroups.value = Array.isArray(result.channels) ? result.channels : [];
    statusText.value = result.detail ?? `消息频道列表已刷新，共 ${channelGroups.value.length} 个已配置频道。`;
  } catch (error) {
    channelGroups.value = [];
    statusText.value = error instanceof Error ? error.message : "消息频道读取失败。";
  }
}

function getSkillMarketCategoryLabel(category: string) {
  const matched = skillMarketCategories.find((item) => item.apiCategory === category);
  return matched?.label ?? (category || "未分类");
}

function getSkillMarketDescription(skill: SkillMarketSkill) {
  if (appLocale.value === "zh-CN") {
    return skill.descriptionZh || skill.description || "暂无描述";
  }
  return skill.description || skill.descriptionZh || "No description available.";
}

function getSkillMarketInitial(name: string) {
  const trimmed = name.trim();
  return trimmed ? trimmed.charAt(0).toUpperCase() : "S";
}

function formatSkillMarketCount(value: number) {
  if (value >= 100000000) {
    return `${(value / 100000000).toFixed(1)}亿`;
  }
  if (value >= 10000) {
    return `${(value / 10000).toFixed(1)}万`;
  }
  return value.toLocaleString("zh-CN");
}

function formatSkillMarketVersion(value: string) {
  if (!value.trim()) {
    return "v1.0.0";
  }
  return value.startsWith("v") ? value : `v${value}`;
}

function getSkillMarketCacheKey(category: SkillMarketSectionCategory, sortBy: SkillMarketSortBy) {
  if (category === "top") {
    return "top";
  }
  return `${category}:${sortBy}:${skillMarketPage.value}:${skillMarketPageSize.value}`;
}

function getSkillMarketGlobalCacheKey(sortBy: SkillMarketSortBy) {
  return `global:${sortBy}:1:300`;
}

function applySkillMarketSnapshot(category: SkillMarketSectionCategory, result: SkillMarketListResultSnapshot) {
  if (category === "top") {
    skillMarketTopSkills.value = result.skills;
    skillMarketTopTotal.value = result.total;
    return;
  }
  skillMarketCategorySkills.value = result.skills;
  skillMarketCategoryTotal.value = result.total;
}

async function refreshSkillMarket(force = false) {
  const category = activeSkillMarketCategory.value;
  const keyword = skillMarketSearch.value.trim();
  const globalCacheKey = getSkillMarketGlobalCacheKey(skillMarketSortBy.value);
  if (keyword) {
    if (!force) {
      const cachedGlobal = skillMarketGlobalCache.get(globalCacheKey);
      if (cachedGlobal) {
        skillMarketGlobalSkills.value = cachedGlobal.skills;
        skillMarketGlobalTotal.value = cachedGlobal.total;
        skillMarketError.value = "";
        return;
      }
    }
    const token = ++skillMarketRequestToken;
    skillMarketLoading.value = true;
    skillMarketError.value = "";
    try {
      const globalResult = await fetchSkillsGlobal({
        page: 1,
        pageSize: 300,
        sortBy: skillMarketSortBy.value,
        order: "desc"
      });
      if (token !== skillMarketRequestToken) {
        return;
      }
      skillMarketGlobalSkills.value = globalResult.skills;
      skillMarketGlobalTotal.value = globalResult.total;
      skillMarketGlobalCache.set(globalCacheKey, {
        skills: globalResult.skills.slice(),
        total: globalResult.total
      });
      statusText.value = `全站技能搜索数据已刷新，共 ${globalResult.skills.length} 条。`;
    } catch (error) {
      if (token !== skillMarketRequestToken) {
        return;
      }
      const message = error instanceof Error ? error.message : "技能市场加载失败。";
      skillMarketError.value = message;
      statusText.value = message;
    } finally {
      if (token === skillMarketRequestToken) {
        skillMarketLoading.value = false;
      }
    }
    return;
  }

  const cacheKey = getSkillMarketCacheKey(category, skillMarketSortBy.value);
  if (!force) {
    const cached = skillMarketCache.get(cacheKey);
    if (cached) {
      applySkillMarketSnapshot(category, cached);
      skillMarketError.value = "";
      return;
    }
  }

  const token = ++skillMarketRequestToken;
  skillMarketLoading.value = true;
  skillMarketError.value = "";

  try {
    let result: SkillMarketListResultSnapshot;
    if (category === "top") {
      const topResult = await fetchSkillTop50();
      result = { skills: topResult.skills, total: topResult.total };
    } else {
      const categoryMeta = skillMarketCategoryMap.get(category);
      if (!categoryMeta?.apiCategory) {
        throw new Error("当前分类暂不支持加载。");
      }
      const listResult = await fetchSkillsByCategory(categoryMeta.apiCategory, {
        page: skillMarketPage.value,
        pageSize: skillMarketPageSize.value,
        sortBy: skillMarketSortBy.value,
        order: "desc"
      });
      result = { skills: listResult.skills, total: listResult.total };
    }

    if (token !== skillMarketRequestToken) {
      return;
    }

    applySkillMarketSnapshot(category, result);
    skillMarketCache.set(cacheKey, {
      skills: result.skills.slice(),
      total: result.total
    });
    const categoryLabel = skillMarketCategoryMap.get(category)?.label ?? "技能市场";
    statusText.value = `${categoryLabel}已刷新，当前展示 ${result.skills.length} 条技能。`;
  } catch (error) {
    if (token !== skillMarketRequestToken) {
      return;
    }
    const message = error instanceof Error ? error.message : "技能市场加载失败。";
    skillMarketError.value = message;
    statusText.value = message;
  } finally {
    if (token === skillMarketRequestToken) {
      skillMarketLoading.value = false;
    }
  }
}

function selectSkillMarketCategory(category: SkillMarketSectionCategory) {
  skillMarketPage.value = 1;
  if (activeSkillMarketCategory.value === category) {
    void refreshSkillMarket();
    return;
  }
  activeSkillMarketCategory.value = category;
  void refreshSkillMarket();
}

function handleSkillMarketSortChange() {
  skillMarketPage.value = 1;
  skillMarketGlobalCache.clear();
  if (activeSkillMarketCategory.value === "top" && !skillMarketSearch.value.trim()) {
    return;
  }
  void refreshSkillMarket();
}

function goToSkillMarketPage(page: number) {
  const nextPage = Math.min(Math.max(page, 1), skillMarketCurrentTotalPages.value);
  if (nextPage === skillMarketPage.value) {
    return;
  }
  skillMarketPage.value = nextPage;
  if (activeSkillMarketCategory.value !== "top" && !skillMarketSearch.value.trim()) {
    void refreshSkillMarket();
  }
}

function goPrevSkillMarketPage() {
  goToSkillMarketPage(skillMarketPage.value - 1);
}

function goNextSkillMarketPage() {
  goToSkillMarketPage(skillMarketPage.value + 1);
}

function openSkillMarketHomepage(url: string) {
  if (!url.trim()) {
    statusText.value = "该技能暂未提供主页地址。";
    return;
  }
  void openCodingPlanPlatform(url);
}

function openSkillMarketDetailModal(skill: SkillMarketSkill) {
  activeSkillMarketDetail.value = skill;
}

function closeSkillMarketDetailModal() {
  activeSkillMarketDetail.value = null;
}

async function openRoleWorkflowDetail(roleId: string) {
  const found = roleWorkflowRoleIndex.value.get(roleId);
  if (!found) {
    return;
  }
  roleWorkflowInstallNotice.value = null;
  const requestToken = ++roleWorkflowDetailRequestToken;
  const override = roleWorkflowOverrides.value[roleId];
  roleWorkflowDetailRoleId.value = roleId;
  roleWorkflowDetailOriginalContent.value = "正在加载详情内容...";
  roleWorkflowDetailDraft.value = {
    contentZh: "正在加载详情内容..."
  };
  statusText.value = `正在加载 ${found.role.sourcePath}...`;

  const detailSnapshot = await loadAgentDetailMarkdownZh(found.role.sourcePath);
  if (requestToken !== roleWorkflowDetailRequestToken || roleWorkflowDetailRoleId.value !== roleId) {
    return;
  }

  const baseContent = detailSnapshot.contentZh;
  const baseNameZh = override?.nameZh ?? found.role.nameZh;
  roleWorkflowDetailOriginalContent.value = baseContent;
  roleWorkflowDetailDraft.value = {
    contentZh: override?.detailContentZh ?? baseContent
  };
  roleWorkflowNameZhOriginal.value = baseNameZh;
  roleWorkflowNameZhDraft.value = baseNameZh;
  statusText.value = detailSnapshot.found
    ? `已打开 ${found.role.nameZh} 的详情（${found.role.sourcePath}）。`
    : `未找到 ${found.role.sourcePath}，已打开占位详情。`;

  void syncCursorPassThrough();
}

function closeRoleWorkflowDetail() {
  roleWorkflowDetailRequestToken += 1;
  roleWorkflowDetailRoleId.value = null;
  roleWorkflowDetailDraft.value = { contentZh: "" };
  roleWorkflowDetailOriginalContent.value = "";
  roleWorkflowNameZhDraft.value = "";
  roleWorkflowNameZhOriginal.value = "";
  roleWorkflowInstallNotice.value = null;
  void syncCursorPassThrough();
}

function createRoleWorkflowVersionId() {
  return `role-ver-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function saveRoleWorkflowDetail() {
  const activeId = roleWorkflowDetailRoleId.value;
  const found = activeRoleWorkflowBase.value;
  if (!activeId || !found) {
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
  roleWorkflowDetailOriginalContent.value = nextContent;
  roleWorkflowNameZhOriginal.value = nextNameZh;
  roleWorkflowNameZhDraft.value = nextNameZh;
  persistRoleWorkflowOverrides();
  statusText.value = `${nextNameZh} 详情已保存（当前保留 ${nextVersions.length} 个版本）。`;
}

function restoreRoleWorkflowOriginalContent() {
  const found = activeRoleWorkflowBase.value;
  if (!found) {
    return;
  }
  roleWorkflowDetailDraft.value = {
    contentZh: roleWorkflowDetailOriginalContent.value
  };
  roleWorkflowNameZhDraft.value = roleWorkflowNameZhOriginal.value;
  statusText.value = `${roleWorkflowNameZhOriginal.value || found.role.nameZh} 已恢复原始内容。`;
}

function applyRoleWorkflowSavedVersion(versionId: string) {
  const found = activeRoleWorkflowBase.value;
  const matched = roleWorkflowDetailSavedVersions.value.find((version) => version.id === versionId);
  if (!found || !matched) {
    return;
  }
  roleWorkflowDetailDraft.value = {
    contentZh: matched.contentZh
  };
  statusText.value = `已载入 ${found.role.nameZh} 的历史版本（${formatTime(matched.savedAt)}）。`;
}

function deleteRoleWorkflowSavedVersion(versionId: string) {
  const activeId = roleWorkflowDetailRoleId.value;
  const found = activeRoleWorkflowBase.value;
  if (!activeId || !found) {
    return;
  }
  const current = roleWorkflowOverrides.value[activeId];
  if (!current) {
    return;
  }

  const nextVersions = (current.detailVersions ?? []).filter((version) => version.id !== versionId);
  const nextOverride: RoleWorkflowOverride = {
    ...current,
    detailVersions: nextVersions
  };
  if ((nextOverride.detailVersions?.length ?? 0) === 0) {
    delete nextOverride.detailVersions;
  }

  if (!nextOverride.nameZh && !nextOverride.workflowZh && !nextOverride.detailContentZh && !nextOverride.detailVersions) {
    const { [activeId]: _, ...rest } = roleWorkflowOverrides.value;
    roleWorkflowOverrides.value = rest;
  } else {
    roleWorkflowOverrides.value = {
      ...roleWorkflowOverrides.value,
      [activeId]: nextOverride
    };
  }
  persistRoleWorkflowOverrides();
  statusText.value = `${found.role.nameZh} 的历史版本已删除。`;
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

async function installRoleWorkflowRole() {
  const found = activeRoleWorkflowBase.value;
  if (!found || isRoleWorkflowInstalling.value) {
    return;
  }
  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) {
    statusText.value = "当前环境不支持安装角色（仅桌面端可用）。";
    return;
  }

  const agentId = buildInstallableRoleAgentId(found.role.sourcePath);
  const markdown = roleWorkflowDetailDraft.value.contentZh.trim();
  if (!markdown) {
    statusText.value = "详情内容为空，无法安装角色。";
    return;
  }

  roleWorkflowInstallNotice.value = null;
  const selectedNameZh = roleWorkflowNameZhDraft.value.trim() || found.role.nameZh;
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
    const successMessage = `角色已安装，\n角色名称：${roleName}\n配置文件目录：${workspaceDir}`;
    roleWorkflowInstallNotice.value = { tone: "success", text: successMessage };
    await refreshStaffSnapshot();
  } catch (error) {
    roleWorkflowInstallNotice.value = {
      tone: "error",
      text: error instanceof Error ? error.message : `安装 ${found.role.nameZh} 失败。`
    };
  } finally {
    isRoleWorkflowInstalling.value = false;
  }
}

function openStaffDeleteConfirm(member: StaffMemberSnapshot) {
  staffDeleteTargetMember.value = member;
  staffDeleteRemoveFiles.value = false;
  staffDeleteError.value = "";
}

function closeStaffDeleteConfirm() {
  if (isStaffDeleting.value) {
    return;
  }
  staffDeleteTargetMember.value = null;
  staffDeleteRemoveFiles.value = false;
  staffDeleteError.value = "";
}

async function confirmDeleteStaffMember() {
  const target = staffDeleteTargetMember.value;
  if (!target || isStaffDeleting.value) {
    return;
  }

  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) {
    staffDeleteError.value = "当前环境不支持删除角色（仅桌面端可用）。";
    return;
  }

  staffDeleteError.value = "";
  isStaffDeleting.value = true;
  let successMessage = "";
  try {
    const result = (await invoke("remove_role_workflow_agent", {
      agentId: target.agentId,
      deleteFiles: staffDeleteRemoveFiles.value
    })) as string;
    successMessage = result || `角色 ${target.displayName} 已删除。`;
    await refreshStaffSnapshot();
  } catch (error) {
    staffDeleteError.value = error instanceof Error ? error.message : `删除 ${target.displayName} 失败。`;
  } finally {
    isStaffDeleting.value = false;
    if (successMessage) {
      closeStaffDeleteConfirm();
      if (typeof window !== "undefined" && typeof window.alert === "function") {
        window.alert(successMessage);
      }
    }
  }
}

function getConsoleSectionTitle(section: ConsoleSection) {
  if (section === "overview") return "总览";
  if (section === "platforms") return "代理配置";
  if (section === "staff") return "员工管理";
  if (section === "role-workflow") return "角色工作流";
  if (section === "skill-market") return "技能市场";
  if (section === "channels") return "消息频道";
  if (section === "bindings") return "宠物绑定";
  return "任务管理";
}

async function openConsole(section: ConsoleSection) {
  const shouldOpenDetachedWindow = false;
  if (shouldOpenDetachedWindow) {
    const invoke = getTauriApi()?.core?.invoke;
    if (invoke) {
      try {
        await invoke("open_console_window", { section });
        hideContextMenu();
        noteInteraction();
        statusText.value = `${getConsoleSectionTitle(section)}窗口已独立打开。`;
        return;
      } catch {
        // Fallback to the embedded panel when window creation is unavailable.
      }
    }
  }

  activePanelMode.value = "console";
  activeSection.value = section;
  hideContextMenu();
  noteInteraction();

  if (!isConsoleOpen.value) {
    if (panelPlacement.value.mode === "auto") {
      resetPanelPlacement();
    }
    isConsoleOpen.value = true;
    startPanelAnimation();
  }

  if (section === "overview") {
    statusText.value = "总览已展开，可以先快速查看平台、员工、记忆、文档和任务状态。";
  } else if (section === "platforms") {
    statusText.value = "代理配置已展开，可以新增、切换默认平台或修改接口配置。";
  } else if (section === "staff") {
    statusText.value = "员工管理已展开，适合维护角色、职责和轮值状态。";
    void refreshStaffSnapshot();
    void refreshOpenClawSkillSnapshot();
    void refreshOpenClawSkillsList();
  } else if (section === "role-workflow") {
    statusText.value = "角色工作流已展开，按 The Agency Roster 分类查看全量角色。";
  } else if (section === "skill-market") {
    statusText.value = "技能市场已展开，可按分类浏览并快速查看热门技能。";
    void refreshSkillMarket();
  } else if (section === "channels") {
    statusText.value = "消息频道已展开，可查看当前可接入渠道和已配置状态。";
    void refreshMessageChannelSnapshot();
  } else if (section === "bindings") {
    statusText.value = "宠物绑定已展开，可以配置绑定码、远程多宠连接与 Agent 授权。";
  } else if (section === "tasks") {
    statusText.value = "任务管理已展开，当前展示的是 openclaw cron 的真实调度快照。";
    void refreshTaskSnapshot();
  }

  applyBaseAnimation();
}

function updateLogAnalysisStatus(view = activeLogAnalysisView.value) {
  if (view === "timeline") {
    statusText.value = "日志分析已打开，当前查看时间线。";
  } else if (view === "sessions") {
    statusText.value = "日志分析已打开，当前查看会话视图。";
  } else {
    statusText.value = "日志分析已打开，当前查看失败分析。";
  }
}

function clearLogFilters() {
  logFilterPlatform.value = null;
  logFilterAgent.value = null;
}

function openLogAnalysis(view: LogAnalysisView = "timeline") {
  activePanelMode.value = "logs";
  activeLogAnalysisView.value = view;
  hideContextMenu();
  noteInteraction();

  if (!isConsoleOpen.value) {
    if (panelPlacement.value.mode === "auto") {
      resetPanelPlacement();
    }
    isConsoleOpen.value = true;
    startPanelAnimation();
  }

  updateLogAnalysisStatus(view);
  applyBaseAnimation();
}

function openLobsterConfig() {
  activePanelMode.value = "lobster";
  ensureLobsterInstallWizardPrimed();
  hideContextMenu();
  noteInteraction();

  if (!isConsoleOpen.value) {
    if (panelPlacement.value.mode === "auto") {
      resetPanelPlacement();
    }
    isConsoleOpen.value = true;
    startPanelAnimation();
  }

  statusText.value = "龙虾配置已打开，可执行安装、重启、修复、备份、恢复和升级。";
  void refreshLobsterSnapshot();
  applyBaseAnimation();
}

function openSubscriptionRecommendations() {
  activePanelMode.value = "subscriptions";
  hideContextMenu();
  noteInteraction();

  if (!isConsoleOpen.value) {
    if (panelPlacement.value.mode === "auto") {
      resetPanelPlacement();
    }
    isConsoleOpen.value = true;
    startPanelAnimation();
  }

  statusText.value = "订阅推荐已打开，当前查看场景推荐与平台实测对比。";
  applyBaseAnimation();
}

function toggleConsolePanel(nextValue?: boolean) {
  const finalValue = nextValue ?? !isConsoleOpen.value;
  if (finalValue === isConsoleOpen.value) {
    return;
  }

  if (isConsoleWindowMode && !finalValue) {
    const currentWindow = getTauriApi()?.window?.getCurrentWindow?.();
    if (currentWindow?.close) {
      void currentWindow.close();
      return;
    }
  }

  isConsoleOpen.value = finalValue;
  if (finalValue) {
    noteInteraction();
  }
  startPanelAnimation();
  applyBaseAnimation(true);
  statusText.value = finalValue ? "控制台面板已打开。" : "控制台已收起，我继续在这里陪你。";
}

function queueIdleAction() {
  window.clearTimeout(idleTimer);
  idleTimer = window.setTimeout(() => {
    if (isDragging.value || isSending.value) {
      queueIdleAction();
      return;
    }

    const baseAnimation = resolveBaseAnimationName();
    if (baseAnimation === "sleep") {
      applyBaseAnimation(true);
      return;
    }

    const nextShowcase = idleShowcaseSequence[idleShowcaseIndex % idleShowcaseSequence.length];
    idleShowcaseIndex += 1;
    if (nextShowcase === baseAnimation) {
      queueIdleAction();
      return;
    }

    setAnimation(nextShowcase, baseAnimation);
  }, autoplayDelayMs);
}

function clampPetPosition(nextX: number, nextY: number) {
  const bounds = stage.value?.getBoundingClientRect();

  if (!bounds) {
    return { x: nextX, y: nextY };
  }

  const maxX = Math.max(0, bounds.width - viewportSize.value);
  const maxY = Math.max(0, bounds.height - viewportSize.value);

  return {
    x: Math.min(Math.max(0, nextX), maxX),
    y: Math.min(Math.max(0, nextY), maxY)
  };
}

function centerPet() {
  const bounds = stage.value?.getBoundingClientRect();

  if (!bounds) {
    return;
  }

  petPosition.value = {
    x: Math.max(0, bounds.width - viewportSize.value - 48),
    y: Math.max(0, bounds.height - viewportSize.value - 56)
  };
}

function tick(now: number) {
  const animation = activeAnimation.value;
  const frames = animation.config.frames;
  const duration = getAnimationDuration(animation) * 1000;
  const elapsed = now - animationStartedAt;
  const normalized = animation.loop ? elapsed % duration : Math.min(elapsed, duration);
  const timeInSeconds = (normalized / 1000) * playbackRate;

  let nextFrameIndex = frames.length - 1;
  for (let index = 0; index < frames.length; index += 1) {
    const frame = frames[index];
    const nextFrameTime = frames[index + 1]?.t ?? Number.POSITIVE_INFINITY;
    if (timeInSeconds >= frame.t && timeInSeconds < nextFrameTime) {
      nextFrameIndex = index;
      break;
    }
  }

  currentFrameIndex.value = nextFrameIndex;

  if (!animation.loop && elapsed >= duration) {
    const nextAnimation = queuedAnimationName ?? resolveBaseAnimationName();
    setAnimation(nextAnimation);
  }

  rafId = window.requestAnimationFrame(tick);
}

function handlePetClick() {
  if (dragDistance.value > 6) {
    dragDistance.value = 0;
    return;
  }

  noteInteraction();
  openChatPanel();
  wakeThenAnimate("stomp_feet", "smile_and_blink");
}

async function handlePointerDown(event: PointerEvent) {
  const petEl = pet.value;
  if (!petEl) {
    return;
  }

  if (availableMonitors.length === 0) {
    await loadAvailableMonitors();
  }
  currentMonitorIndex = getMonitorIndexAtScreenPoint(event.screenX, event.screenY);
  pendingDragOffset = null;

  dragPointerId = event.pointerId;
  isDragging.value = true;
  dragDistance.value = 0;
  dragStart = {
    x: event.clientX,
    y: event.clientY,
    petX: petPosition.value.x,
    petY: petPosition.value.y
  };
  petEl.setPointerCapture(event.pointerId);
  noteInteraction();
  wakeThenAnimate("act_cute_rotation", "act_cute_rotation");
  statusText.value = "拖着我走吧，我会老老实实待在舞台里。";
}

async function handlePointerMove(event: PointerEvent) {
  if (!isDragging.value || dragPointerId !== event.pointerId) {
    return;
  }

  if (pendingDragOffset) {
    const nextX = event.clientX - pendingDragOffset.x;
    const nextY = event.clientY - pendingDragOffset.y;
    petPosition.value = clampPetPosition(nextX, nextY);
    dragStart = {
      x: event.clientX,
      y: event.clientY,
      petX: petPosition.value.x,
      petY: petPosition.value.y
    };
    pendingDragOffset = null;
    noteInteraction();
    return;
  }

  const monitorIndex = getMonitorIndexAtScreenPoint(event.screenX, event.screenY);
  if (
    availableMonitors.length > 1 &&
    monitorIndex !== currentMonitorIndex
  ) {
    const invoke = getTauriApi()?.core?.invoke;
    if (invoke) {
      try {
        await invoke("move_window_to_monitor", { index: monitorIndex });
        currentMonitorIndex = monitorIndex;
        pendingDragOffset = {
          x: event.clientX - petPosition.value.x,
          y: event.clientY - petPosition.value.y
        };
        noteInteraction();
        return;
      } catch {
        // 跨屏移动失败时继续在当前屏内拖拽
      }
    }
  }

  const nextX = dragStart.petX + event.clientX - dragStart.x;
  const nextY = dragStart.petY + event.clientY - dragStart.y;
  const dx = event.clientX - dragStart.x;
  const dy = event.clientY - dragStart.y;

  dragDistance.value = Math.hypot(dx, dy);
  petPosition.value = clampPetPosition(nextX, nextY);
  noteInteraction();
}

function finishDrag(event?: PointerEvent) {
  if (!isDragging.value || (event && dragPointerId !== event.pointerId)) {
    return;
  }

  if (event && pet.value?.hasPointerCapture(event.pointerId)) {
    pet.value.releasePointerCapture(event.pointerId);
  }

  pendingDragOffset = null;
  isDragging.value = false;
  dragPointerId = null;
  noteInteraction();
  statusText.value = "位置记住了，继续待机陪伴。";
  applyBaseAnimation(true);
}

function handleResize() {
  petPosition.value = clampPetPosition(petPosition.value.x, petPosition.value.y);
  adjustContextMenuToViewport();
  if (chatPlacement.value.mode === "manual") {
    const bounds = stage.value?.getBoundingClientRect();
    if (bounds) {
      chatPlacement.value = {
        ...chatPlacement.value,
        width: Math.min(chatPlacement.value.width, bounds.width - 32),
        height: Math.min(chatPlacement.value.height, bounds.height - 32),
        x: Math.min(Math.max(16, chatPlacement.value.x), Math.max(16, bounds.width - chatPlacement.value.width - 16)),
        y: Math.min(Math.max(16, chatPlacement.value.y), Math.max(16, bounds.height - chatPlacement.value.height - 16))
      };
    }
  }

  if (panelPlacement.value.mode === "manual") {
    const bounds = stage.value?.getBoundingClientRect();
    if (!bounds) {
      return;
    }

    const prefersWide = true;
    const availableWidth = Math.max(320, bounds.width - 32);
    const availableHeight = Math.max(320, bounds.height - 32);
    const minWidth = Math.min(getPanelMinWidth(prefersWide), availableWidth);
    const minHeight = Math.min(420, availableHeight);
    const width = Math.min(Math.max(minWidth, panelPlacement.value.width), availableWidth);
    const height = Math.min(Math.max(minHeight, panelPlacement.value.height), availableHeight);

    panelPlacement.value = {
      ...panelPlacement.value,
      width,
      height,
      x: Math.min(Math.max(16, panelPlacement.value.x), Math.max(16, bounds.width - width - 16)),
      y: Math.min(Math.max(16, panelPlacement.value.y), Math.max(16, bounds.height - height - 16))
    };
  }
}

function hideContextMenu() {
  contextMenu.value.visible = false;
}

function getTauriApi() {
  return (window as Window & { __TAURI__?: TauriNamespace }).__TAURI__;
}

async function revealConsoleWindowIfNeeded() {
  if (!isConsoleWindowMode) {
    return;
  }

  const currentWindow = getTauriApi()?.window?.getCurrentWindow?.();
  if (!currentWindow?.show) {
    return;
  }

  await currentWindow.show();
  await currentWindow.setFocus?.();
}

type MonitorInfoFromBackend = {
  position: [number, number];
  size: [number, number];
  scaleFactor?: number;
};

async function loadAvailableMonitors(): Promise<void> {
  const invoke = getTauriApi()?.core?.invoke;
  if (!invoke) return;
  try {
    const list = (await invoke("get_available_monitors")) as MonitorInfoFromBackend[];
    availableMonitors = list.map((m) => ({
      position: m.position,
      size: m.size
    }));
  } catch {
    availableMonitors = [];
  }
}

/** 返回包含 (screenX, screenY) 的显示器索引，未找到返回 0 */
function getMonitorIndexAtScreenPoint(screenX: number, screenY: number): number {
  for (let i = 0; i < availableMonitors.length; i++) {
    const m = availableMonitors[i];
    const [px, py] = m.position;
    const [w, h] = m.size;
    if (screenX >= px && screenX < px + w && screenY >= py && screenY < py + h) {
      return i;
    }
  }
  return 0;
}

async function openCodingPlanPlatform(url: string) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!url.trim()) {
    statusText.value = "暂未配置对应平台地址。";
    return;
  }

  if (invoke) {
    try {
      await invoke("open_external_url", { url });
      statusText.value = "已在默认浏览器中打开对应 Coding Plan 平台。";
      return;
    } catch (error) {
      statusText.value = error instanceof Error ? error.message : "打开外部平台失败。";
    }
  }

  if (typeof window !== "undefined") {
    window.open(url, "_blank", "noopener,noreferrer");
    statusText.value = "已尝试在浏览器中打开对应 Coding Plan 平台。";
  }
}

function openSubscriptionPlatformByName(platformName: string) {
  const url = subscriptionPlatformUrlMap[platformName];
  if (!url) {
    return;
  }
  void openCodingPlanPlatform(url);
}

function getSubscriptionStatusTone(value: string): "good" | "warn" | "bad" | "neutral" {
  if (value.includes("通过")) {
    return "good";
  }
  if (value.includes("不稳定")) {
    return "warn";
  }
  if (value.includes("交白卷") || value.includes("失败") || value.includes("不可用")) {
    return "bad";
  }
  return "neutral";
}

function getSubscriptionStarCount(value: string): number {
  const matched = value.trim().match(/^([★⭐]+)/);
  if (!matched) {
    return 0;
  }
  return matched[1].replace(/⭐/g, "★").length;
}

function getSubscriptionMetricNote(value: string): string {
  const matched = value.match(/（([^）]+)）/);
  return matched ? `（${matched[1]}）` : "";
}

function isSubscriptionMetricFailure(value: string): boolean {
  return value.includes("失败") || value.includes("不可用") || value.includes("交白卷");
}

function getSubscriptionStatusIcon(value: string): string {
  if (value.includes("通过")) {
    return "✅";
  }
  if (value.includes("不稳定")) {
    return "⚠️";
  }
  return "❌";
}

function getSubscriptionStatusLabel(value: string): string {
  if (value.includes("通过")) {
    return "通过";
  }
  if (value.includes("交白卷")) {
    return "交白卷";
  }
  return value;
}

function buildPlatformProxyBaseUrl(platform: Pick<PlatformConfig, "pathPrefix">) {
  return `http://127.0.0.1:${proxyPort.value}${normalizePathPrefix(platform.pathPrefix)}`;
}

function getOpenClawProviderId(platformId: string) {
  return openClawProviderIdMap.value[platformId] ?? null;
}

function getPlatformProxyPersistBaseUrl(platform: Pick<PlatformConfig, "id" | "pathPrefix" | "baseUrl">) {
  const providerId = getOpenClawProviderId(platform.id);
  if (providerId && OPENCLAW_PROVIDER_PROXY_BASEURL_PRESETS[providerId]) {
    return normalizeLocalProxyBaseUrlForPersist(OPENCLAW_PROVIDER_PROXY_BASEURL_PRESETS[providerId]);
  }

  if (isLocalProxyBaseUrl(platform.baseUrl)) {
    return normalizeLocalProxyBaseUrlForPersist(platform.baseUrl);
  }

  return normalizeLocalProxyBaseUrlForPersist(buildPlatformProxyBaseUrl(platform));
}

function getPlatformProxyRequestBaseUrl(platform: Pick<PlatformConfig, "pathPrefix">) {
  return buildPlatformProxyBaseUrl(platform);
}

function getPlatformDirectTargetBaseUrl(platform: Pick<PlatformConfig, "id" | "baseUrl">) {
  if (!isLocalProxyBaseUrl(platform.baseUrl)) {
    return normalizeBaseUrl(platform.baseUrl);
  }

  const direct = platformDirectBaseUrlMap.value[platform.id];
  if (direct?.trim()) {
    return normalizeBaseUrl(direct);
  }

  const providerId = getOpenClawProviderId(platform.id);
  if (providerId && OPENCLAW_PROVIDER_DIRECT_BASEURL_PRESETS[providerId]) {
    return normalizeBaseUrl(OPENCLAW_PROVIDER_DIRECT_BASEURL_PRESETS[providerId]);
  }

  return normalizeBaseUrl(platform.baseUrl);
}

function getPlatformDisplayBaseUrl(platform: Pick<PlatformConfig, "id" | "baseUrl" | "pathPrefix" | "enabled">) {
  return platform.enabled ? getPlatformProxyRequestBaseUrl(platform) : normalizeBaseUrl(platform.baseUrl);
}

function buildPlatformRequestEndpoint(platform: Pick<PlatformConfig, "id" | "baseUrl" | "pathPrefix" | "apiPath" | "enabled">) {
  const baseUrl = platform.enabled ? getPlatformProxyRequestBaseUrl(platform) : getPlatformDirectTargetBaseUrl(platform);
  return `${baseUrl}${normalizeApiPath(platform.apiPath)}`;
}

function normalizeOpenClawProviderId(raw: string) {
  const cleaned = raw
    .trim()
    .split("")
    .map((ch) => {
      if ((ch >= "0" && ch <= "9") || (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") || ch === "-" || ch === "_") {
        return ch.toLowerCase();
      }
      if (ch === "/" || ch === ":" || ch === "." || /\s/.test(ch)) {
        return "-";
      }
      return "";
    })
    .join("");
  const normalized = cleaned
    .replace(/-+/g, "-")
    .replace(/^-+|-+$/g, "");
  return normalized || "platform";
}

function resolveLobsterProviderIdForOpenClaw() {
  const fromOption = lobsterEffectiveProviderOption.value?.id?.trim();
  if (fromOption) {
    return normalizeOpenClawProviderId(fromOption);
  }

  const fromMap = getOpenClawProviderId(lobsterProviderForm.value.id)?.trim();
  if (fromMap) {
    return normalizeOpenClawProviderId(fromMap);
  }

  const fromName = lobsterProviderForm.value.name.trim();
  if (fromName) {
    return normalizeOpenClawProviderId(fromName);
  }

  return "custom";
}

function findPlatformIdByProviderId(platformList: PlatformConfig[], providerId: string) {
  const normalizedProviderId = providerId.trim().toLowerCase();
  if (!normalizedProviderId) {
    return null;
  }
  return (
    platformList.find((item) => (getOpenClawProviderId(item.id) ?? "").trim().toLowerCase() === normalizedProviderId)?.id ?? null
  );
}

async function saveOpenClawProviderBaseUrl(platformId: string, baseUrl: string) {
  const providerId = getOpenClawProviderId(platformId);
  if (!providerId) {
    return;
  }

  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    return;
  }

  await invoke("save_openclaw_provider_base_url", {
    providerId,
    baseUrl
  });
}

async function saveOpenClawProviderConfig(config: {
  providerId: string;
  name: string;
  protocol: PlatformProtocol;
  apiKind?: LobsterProviderApiKind;
  baseUrl: string;
  model: string;
  apiKey: string;
}) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    throw new Error("当前环境不支持写入 openclaw.json。");
  }

  await invoke("save_openclaw_provider_config", { config });
}

async function syncLocalProxyServer() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    return;
  }

  const payload: LocalProxyPlatformPayload[] = platforms.value
    .filter((platform) => platform.enabled)
    .map((platform) => ({
      protocol: platform.protocol,
      baseUrl: getPlatformDirectTargetBaseUrl(platform),
      pathPrefix: platform.pathPrefix,
      apiKey: platform.apiKey
    }));

  try {
    await invoke("sync_local_proxy", { port: proxyPort.value, platforms: payload });
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "本地代理启动失败。";
  }
}

async function loadPlatformsFromOpenClawConfig() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    return null as PlatformConfig[] | null;
  }

  const now = Date.now();
  const enabledMap = loadPlatformProxyEnabledMap();
  const nextProviderIdMap: Record<string, string> = {};
  const nextDirectBaseUrlMap = { ...platformDirectBaseUrlMap.value };
  try {
    const result = (await invoke("load_openclaw_platforms_snapshot")) as OpenClawPlatformSnapshotResponse;
    const configuredPlatforms = Array.isArray(result.platforms) ? result.platforms : [];
    const mapped = configuredPlatforms
      .map((item, index) => {
        const fallbackId = `platform-openclaw-${index + 1}`;
        const id = typeof item.id === "string" && item.id.trim() ? item.id.trim() : fallbackId;
        const providerId = typeof item.providerId === "string" ? item.providerId.trim() : "";
        if (providerId) {
          nextProviderIdMap[id] = providerId;
        }
        const normalizedBaseUrl = normalizeBaseUrl(typeof item.baseUrl === "string" ? item.baseUrl : "");
        const inferredProxy = typeof item.baseUrl === "string" ? isLocalProxyBaseUrl(item.baseUrl) : false;
        if (!isLocalProxyBaseUrl(normalizedBaseUrl)) {
          nextDirectBaseUrlMap[id] = normalizedBaseUrl;
        } else if (!nextDirectBaseUrlMap[id] && providerId && OPENCLAW_PROVIDER_DIRECT_BASEURL_PRESETS[providerId]) {
          nextDirectBaseUrlMap[id] = OPENCLAW_PROVIDER_DIRECT_BASEURL_PRESETS[providerId];
        }
        const effectiveBaseUrl =
          inferredProxy && nextDirectBaseUrlMap[id] ? normalizeBaseUrl(nextDirectBaseUrlMap[id]) : normalizedBaseUrl;
        const protocol: PlatformProtocol = item.protocol === "anthropic" ? "anthropic" : "openai";
        const normalizedApiPath = normalizeApiPath(typeof item.apiPath === "string" ? item.apiPath : "/v1/chat/completions");
        const effectiveApiPath = inferApiPathForBaseUrl(protocol, effectiveBaseUrl || normalizedBaseUrl, normalizedApiPath);
        const enabled = typeof enabledMap[id] === "boolean" ? enabledMap[id] : inferredProxy;
        return {
          id,
          name: typeof item.name === "string" && item.name.trim() ? item.name.trim() : `平台 ${index + 1}`,
          protocol,
          baseUrl: effectiveBaseUrl,
          pathPrefix: normalizePathPrefix(typeof item.pathPrefix === "string" ? item.pathPrefix : `/provider-${index + 1}`),
          apiPath: effectiveApiPath,
          apiKey: typeof item.apiKey === "string" ? item.apiKey : "",
          model: typeof item.model === "string" ? item.model : "",
          enabled,
          createdAt: now - index,
          updatedAt: now
        } satisfies PlatformConfig;
      })
      .filter((item) => item.baseUrl);

    if (!mapped.length) {
      openClawProviderIdMap.value = {};
      return [] as PlatformConfig[];
    }

    openClawProviderIdMap.value = nextProviderIdMap;
    platformDirectBaseUrlMap.value = nextDirectBaseUrlMap;
    persistPlatformDirectBaseUrlStates(nextDirectBaseUrlMap);
    persistPlatformProxyEnabledStates(mapped);
    return mapped;
  } catch {
    openClawProviderIdMap.value = {};
    return null as PlatformConfig[] | null;
  }
}

async function refreshGatewayMonitor() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    gatewayMonitor.value = {
      status: "unsupported",
      checkedUrl: null,
      detail: "当前运行环境不支持网关探测。",
      latencyMs: null
    };
    return;
  }

  gatewayMonitor.value = {
    ...gatewayMonitor.value,
    status: "checking"
  };

  try {
    const result = (await invoke("check_openclaw_gateway")) as Partial<GatewayMonitorState>;
    gatewayMonitor.value = {
      status:
        result.status === "online" ||
        result.status === "offline" ||
        result.status === "unconfigured" ||
        result.status === "unsupported" ||
        result.status === "checking"
          ? result.status
          : "offline",
      checkedUrl: typeof result.checkedUrl === "string" ? result.checkedUrl : null,
      detail: typeof result.detail === "string" ? result.detail : null,
      latencyMs: typeof result.latencyMs === "number" ? result.latencyMs : null
    };
  } catch (error) {
    gatewayMonitor.value = {
      status: "offline",
      checkedUrl: null,
      detail: error instanceof Error ? error.message : "网关状态检查失败。",
      latencyMs: null
    };
  }
}

function getLobsterActionTitle(action: LobsterActionId | string) {
  if (action === "install") return tr("lobster.action.install.title");
  if (action === "restart_gateway") return tr("lobster.action.restart_gateway.title");
  if (action === "auto_fix") return tr("lobster.action.auto_fix.title");
  if (action === "backup") return tr("lobster.action.backup.title");
  if (action === "restore") return tr("lobster.action.restore.title");
  if (action === "upgrade") return tr("lobster.action.upgrade.title");
  return action;
}

function getLobsterInstallCheckStatusLabel(status: LobsterInstallCheckStatus) {
  if (status === "checking") return tr("wizard.runtime.checking");
  if (status === "success") return tr("wizard.check.success");
  if (status === "warning") return tr("wizard.check.warning");
  return tr("wizard.check.failed");
}

function getLobsterInstallComponentStatusLabel(status: LobsterInstallComponentStatus) {
  if (status === "pending") return tr("wizard.installing.status.pending");
  if (status === "installing") return tr("wizard.installing.status.installing");
  if (status === "installed") return tr("wizard.installing.status.installed");
  return tr("wizard.installing.status.failed");
}

function getLobsterInstallGuideOs() {
  if (isWindowsRuntime) {
    return "windows";
  }
  if (typeof navigator === "undefined") {
    return "unknown";
  }
  if (/mac os|macintosh|darwin/i.test(navigator.userAgent)) {
    return "macos";
  }
  if (/linux/i.test(navigator.userAgent)) {
    return "linux";
  }
  return "unknown";
}

function buildPendingLobsterInstallChecks() {
  return LOBSTER_INSTALL_CHECK_BLUEPRINT.map((item) => ({
    id: item.id,
    title: item.title,
    status: "checking" as LobsterInstallCheckStatus,
    detail: "正在检查..."
  }));
}

function createPendingLobsterInstallGuide() {
  return {
    os: getLobsterInstallGuideOs(),
    ready: false,
    checks: buildPendingLobsterInstallChecks()
  } satisfies LobsterInstallGuideResponse;
}

function normalizeLobsterInstallGuideResponse(guide: LobsterInstallGuideResponse) {
  const map = new Map(guide.checks.map((item) => [item.id, item]));
  const orderedChecks: LobsterInstallCheckItem[] = LOBSTER_INSTALL_CHECK_BLUEPRINT.map((base) => {
    const matched = map.get(base.id);
    if (matched) {
      return matched;
    }
    return {
      id: base.id,
      title: base.title,
      status: "warning",
      detail: "该项未返回检测结果，请手动确认。"
    };
  });

  const knownIds = new Set(LOBSTER_INSTALL_CHECK_BLUEPRINT.map((item) => item.id));
  const extraChecks = guide.checks.filter((item) => !knownIds.has(item.id));
  const checks = [...orderedChecks, ...extraChecks];
  return {
    os: guide.os,
    ready: !checks.some((item) => item.status === "failed"),
    checks
  } satisfies LobsterInstallGuideResponse;
}

function ensureLobsterInstallWizardPrimed() {
  if (isLobsterInstallWizardPrimed.value) {
    return;
  }
  isLobsterInstallWizardPrimed.value = true;
}

function getLobsterProviderOptionByDraft(draft: Pick<PlatformDraft, "name" | "protocol" | "baseUrl" | "pathPrefix">) {
  const baseUrl = normalizeBaseUrl(draft.baseUrl);
  const pathPrefix = normalizePathPrefix(draft.pathPrefix);
  const name = draft.name.trim().toLowerCase();

  return (
    lobsterProviderOptions.find(
      (item) =>
        item.protocol === draft.protocol &&
        normalizeBaseUrl(item.defaultBaseUrl) === baseUrl &&
        normalizePathPrefix(item.pathPrefix) === pathPrefix
    ) ??
    lobsterProviderOptions.find((item) => item.protocol === draft.protocol && item.name.toLowerCase() === name) ??
    lobsterProviderOptions.find(
      (item) =>
        item.id === "custom" &&
        (normalizePathPrefix(item.pathPrefix) === pathPrefix || item.name.toLowerCase() === name)
    ) ??
    null
  );
}

function getLobsterProviderOptionDisplayName(option: Pick<LobsterProviderOption, "id" | "name">) {
  return option.id === "custom" ? tr("provider.option.custom") : option.name;
}

function getLobsterProviderOptionLabel(option: Pick<LobsterProviderOption, "id" | "icon" | "name">) {
  const localizedName = getLobsterProviderOptionDisplayName(option);
  return option.icon ? `${option.icon} ${localizedName}` : localizedName;
}

function getLobsterProviderApiKindByDraft(draft: Pick<PlatformDraft, "protocol" | "apiPath">): LobsterProviderApiKind {
  const normalizedApiPath = normalizeApiPath(draft.apiPath).toLowerCase();
  if (normalizedApiPath.includes("responses")) {
    return "openai-responses";
  }
  if (draft.protocol === "anthropic" || normalizedApiPath.includes("messages")) {
    return "anthropic-messages";
  }
  return "openai-completions";
}

function applyLobsterProviderApiKind(apiKind: LobsterProviderApiKind) {
  const nextProtocol: PlatformProtocol = apiKind === "anthropic-messages" ? "anthropic" : "openai";
  const defaultApiPath = apiKind === "openai-responses"
    ? "/v1/responses"
    : apiKind === "anthropic-messages"
      ? "/v1/messages"
      : "/v1/chat/completions";
  lobsterProviderForm.value.protocol = nextProtocol;
  lobsterProviderForm.value.apiPath = inferApiPathForBaseUrl(nextProtocol, lobsterProviderForm.value.baseUrl, defaultApiPath);
  lobsterProviderConfigured.value = false;
}

function isLobsterProviderDraftReady(
  draft: Pick<PlatformDraft, "name" | "model" | "apiKey">,
  option: Pick<LobsterProviderOption, "requiresApiKey"> | null
) {
  if (!draft.name.trim() || !draft.model.trim()) {
    return false;
  }
  if (option?.requiresApiKey === false) {
    return true;
  }
  return draft.apiKey.trim().length > 0;
}

function resetLobsterProviderDraft() {
  const active = activePlatform.value ?? platforms.value[0] ?? null;
  if (active) {
    lobsterProviderForm.value = createPlatformDraft({
      name: active.name,
      protocol: active.protocol,
      baseUrl: active.baseUrl,
      pathPrefix: active.pathPrefix,
      apiPath: active.apiPath,
      apiKey: active.apiKey,
      model: active.model,
      enabled: true
    });
  } else {
    const preset = lobsterProviderOptions.find((item) => item.id === "openai") ?? lobsterProviderOptions[0];
    lobsterProviderForm.value = createPlatformDraft(
      preset
        ? {
          name: preset.name,
          protocol: preset.protocol,
          baseUrl: preset.defaultBaseUrl,
          pathPrefix: preset.pathPrefix,
          apiPath: inferApiPathForBaseUrl(
            preset.protocol,
            preset.defaultBaseUrl,
            preset.protocol === "anthropic" ? "/v1/messages" : "/v1/chat/completions"
          ),
          model: preset.defaultModelId,
          enabled: true
        }
        : undefined
    );
  }

  const matchedOption = getLobsterProviderOptionByDraft(lobsterProviderForm.value);
  lobsterProviderPresetKey.value = matchedOption?.id ?? "";
  lobsterProviderConfigured.value = isLobsterProviderDraftReady(lobsterProviderForm.value, matchedOption);
}

function handleLobsterProviderPresetChange() {
  const selectedOption = lobsterProviderOptions.find((item) => item.id === lobsterProviderPresetKey.value);
  if (!selectedOption) {
    lobsterProviderConfigured.value = false;
    return;
  }

  lobsterProviderForm.value = createPlatformDraft({
    name: selectedOption.name,
    protocol: selectedOption.protocol,
    baseUrl: selectedOption.defaultBaseUrl,
    pathPrefix: selectedOption.pathPrefix,
    apiPath: inferApiPathForBaseUrl(
      selectedOption.protocol,
      selectedOption.defaultBaseUrl,
      selectedOption.protocol === "anthropic" ? "/v1/messages" : "/v1/chat/completions"
    ),
    model: selectedOption.defaultModelId,
    apiKey: lobsterProviderForm.value.apiKey.trim(),
    enabled: true
  });
  lobsterProviderConfigured.value = false;
}

function openLobsterProviderDocs() {
  const url = lobsterEffectiveProviderOption.value?.docsUrl?.trim();
  if (!url) {
    statusText.value = "当前提供商暂无可用文档链接。";
    return;
  }
  void openCodingPlanPlatform(url);
}

function handleLobsterProviderSkip() {
  if (lobsterInstallWizardStep.value !== 3 || lobsterInstallRunning.value || lobsterProviderSaving.value) {
    return;
  }
  lobsterProviderConfigured.value = true;
  statusText.value = tr("wizard.provider.skip_done");
}

async function saveLobsterProviderFromWizard() {
  if (!lobsterProviderCanSave.value || lobsterProviderSaving.value) {
    return;
  }

  lobsterProviderSaving.value = true;
  try {
    const nextName = lobsterProviderForm.value.name.trim();
    const nextProtocol = lobsterProviderForm.value.protocol;
    const nextApiKind = getLobsterProviderApiKindByDraft(lobsterProviderForm.value);
    const nextApiKey = lobsterProviderForm.value.apiKey.trim();
    const nextBaseUrl = normalizeBaseUrl(lobsterProviderForm.value.baseUrl);
    const nextApiPath = inferApiPathForBaseUrl(nextProtocol, nextBaseUrl, lobsterProviderForm.value.apiPath);
    const nextPathPrefix = normalizePathPrefix(lobsterProviderForm.value.pathPrefix);
    const nextModel = lobsterProviderForm.value.model.trim();
    const providerId = resolveLobsterProviderIdForOpenClaw();
    const selectedOption = lobsterEffectiveProviderOption.value;
    if (selectedOption?.requiresApiKey !== false && !nextApiKey) {
      statusText.value = "当前提供商需要 API 密钥，请先填写。";
      return;
    }

    await saveOpenClawProviderConfig({
      providerId,
      name: nextName,
      protocol: nextProtocol,
      apiKind: nextApiKind,
      baseUrl: nextBaseUrl,
      model: nextModel,
      apiKey: nextApiKey
    });

    const openClawPlatforms = await loadPlatformsFromOpenClawConfig();
    if (openClawPlatforms) {
      platforms.value = openClawPlatforms;
      const matchedPlatformId =
        findPlatformIdByProviderId(openClawPlatforms, providerId) ??
        openClawPlatforms[0]?.id ??
        null;
      if (matchedPlatformId) {
        activePlatformId.value = matchedPlatformId;
        setActivePlatform(matchedPlatformId);
      }
      await syncLocalProxyServer();
    } else {
      const nextPlatforms = upsertPlatform(platforms.value, {
        ...lobsterProviderForm.value,
        name: nextName,
        model: nextModel,
        apiKey: nextApiKey,
        enabled: true,
        baseUrl: nextBaseUrl,
        pathPrefix: nextPathPrefix,
        apiPath: nextApiPath
      });
      platforms.value = nextPlatforms;
      activePlatformId.value = lobsterProviderForm.value.id;
      setActivePlatform(lobsterProviderForm.value.id);
    }

    lobsterProviderConfigured.value = true;
    statusText.value = `AI 提供商「${nextName}」已写入 openclaw.json。`;
  } catch (error) {
    lobsterProviderConfigured.value = false;
    statusText.value = error instanceof Error ? error.message : "写入 openclaw.json 失败。";
  } finally {
    lobsterProviderSaving.value = false;
  }
}

async function refreshLobsterSnapshot() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    lobsterSnapshot.value = null;
    return;
  }

  try {
    const result = (await invoke("load_lobster_snapshot")) as LobsterSnapshotResponse;
    lobsterSnapshot.value = result;
    const paths = new Set((result.backups ?? []).map((item) => item.path));
    if (selectedLobsterBackupPath.value && paths.has(selectedLobsterBackupPath.value)) {
      return;
    }
    selectedLobsterBackupPath.value = result.backups?.[0]?.path ?? null;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "读取龙虾状态失败。";
  }
}

async function refreshLobsterInstallGuide() {
  const requestToken = ++lobsterInstallGuideRefreshToken;
  const invoke = getTauriApi()?.core?.invoke;
  lobsterInstallGuide.value = createPendingLobsterInstallGuide();
  lobsterInstallGuideLoading.value = true;

  const applyGuideWithProgress = async (guide: LobsterInstallGuideResponse) => {
    const normalized = normalizeLobsterInstallGuideResponse(guide);
    const checkingChecks: LobsterInstallCheckItem[] = normalized.checks.map((item) => ({
      id: item.id,
      title: item.title,
      status: "checking",
      detail: "正在检查..."
    }));
    lobsterInstallGuide.value = {
      os: normalized.os,
      ready: false,
      checks: checkingChecks
    };

    for (let index = 0; index < normalized.checks.length; index += 1) {
      if (requestToken !== lobsterInstallGuideRefreshToken) {
        return;
      }
      const currentChecks: LobsterInstallCheckItem[] =
        (lobsterInstallGuide.value?.checks.slice() ?? checkingChecks.slice()) as LobsterInstallCheckItem[];
      currentChecks[index] = normalized.checks[index];
      lobsterInstallGuide.value = {
        os: normalized.os,
        ready: false,
        checks: currentChecks
      };
      if (index < normalized.checks.length - 1) {
        await new Promise((resolve) => window.setTimeout(resolve, 80));
      }
    }

    if (requestToken !== lobsterInstallGuideRefreshToken) {
      return;
    }
    lobsterInstallGuide.value = normalized;
  };

  const buildErrorGuide = (detail: string): LobsterInstallGuideResponse => ({
    os: getLobsterInstallGuideOs(),
    ready: false,
    checks: LOBSTER_INSTALL_CHECK_BLUEPRINT.map((item, index) => ({
      id: item.id,
      title: item.title,
      status: index === LOBSTER_INSTALL_CHECK_BLUEPRINT.length - 1 ? "warning" : "failed",
      detail
    }))
  });

  try {
    if (!invoke) {
      await applyGuideWithProgress(buildErrorGuide("当前环境不支持安装检查，请在桌面端执行。"));
      return;
    }

    const guide = (await invoke("load_lobster_install_guide")) as LobsterInstallGuideResponse;
    await applyGuideWithProgress(guide);
  } catch (error) {
    await applyGuideWithProgress(buildErrorGuide(error instanceof Error ? error.message : "加载安装检查失败。"));
  } finally {
    if (requestToken === lobsterInstallGuideRefreshToken) {
      lobsterInstallGuideLoading.value = false;
    }
  }
}

function startLobsterInstallProgress() {
  window.clearInterval(lobsterInstallProgressTimer);
  lobsterInstallProgressValue.value = 10;
  lobsterInstallProgressTimer = window.setInterval(() => {
    if (!lobsterInstallRunning.value) {
      return;
    }
    const nextValue = lobsterInstallProgressValue.value + 3 + Math.floor(Math.random() * 5);
    lobsterInstallProgressValue.value = Math.min(92, nextValue);
  }, 420);
}

function stopLobsterInstallProgress() {
  window.clearInterval(lobsterInstallProgressTimer);
}

function openLobsterInstallWizard() {
  ensureLobsterInstallWizardPrimed();
  isLobsterInstallWizardOpen.value = true;
  lobsterInstallWizardStep.value = 1;
  lobsterInstallRiskAccepted.value = false;
  lobsterInstallRuntimeLogs.value = "";
  lobsterInstallFinishedResult.value = null;
  lobsterInstallRunning.value = false;
  lobsterInstallProgressValue.value = 0;
  stopLobsterInstallProgress();
  lobsterInstallGuide.value = null;
  lobsterProviderShowKey.value = false;
  statusText.value = "已进入龙虾安装引导。";
  void syncCursorPassThrough();

  // Defer heavier form initialization to the next frame so the modal appears instantly.
  window.requestAnimationFrame(() => {
    resetLobsterProviderDraft();
  });
}

function closeLobsterInstallWizard() {
  if (lobsterInstallRunning.value) {
    return;
  }
  stopLobsterInstallProgress();
  isLobsterInstallWizardOpen.value = false;
}

async function startLobsterInstallFromWizard() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke || lobsterInstallRunning.value || lobsterActionRunning.value) {
    return;
  }

  lobsterInstallRunning.value = true;
  lobsterActionRunning.value = "install";
  lobsterInstallRuntimeLogs.value = "";
  lobsterInstallFinishedResult.value = null;
  startLobsterInstallProgress();

  try {
    const result = (await invoke("run_lobster_action", { action: "install" })) as LobsterActionResult;
    lobsterActionResult.value = result;
    lobsterInstallFinishedResult.value = result;
    lobsterInstallProgressValue.value = 100;
    lobsterInstallRuntimeLogs.value = [result.command, result.stdout, result.stderr].filter(Boolean).join("\n\n").trim();
    statusText.value = result.success ? "龙虾安装执行完成。" : `龙虾安装执行失败：${result.detail}`;
    await new Promise((resolve) => window.setTimeout(resolve, 220));
    await refreshLobsterSnapshot();
    await refreshGatewayMonitor();
  } catch (error) {
    const detail = error instanceof Error ? error.message : "龙虾安装执行失败。";
    lobsterInstallFinishedResult.value = {
      action: "install",
      command: "npm install -g openclaw@latest",
      success: false,
      detail,
      exitCode: null,
      stdout: "",
      stderr: detail,
      durationMs: 0,
      backupPath: null
    };
    lobsterInstallProgressValue.value = 100;
    lobsterInstallRuntimeLogs.value = detail;
    statusText.value = detail;
    await new Promise((resolve) => window.setTimeout(resolve, 220));
  } finally {
    stopLobsterInstallProgress();
    lobsterActionRunning.value = null;
    lobsterInstallRunning.value = false;
    lobsterInstallWizardStep.value = 5;
  }
}

async function handleLobsterInstallWizardNext() {
  if (lobsterInstallWizardStep.value === 1) {
    lobsterInstallWizardStep.value = 2;
    if (!lobsterInstallGuide.value) {
      await refreshLobsterInstallGuide();
    }
    return;
  }

  if (lobsterInstallWizardStep.value === 2) {
    if (!lobsterInstallGuide.value?.ready) {
      statusText.value = "请先完成环境检查，再进入下一步。";
      return;
    }
    lobsterInstallWizardStep.value = 3;
    return;
  }

  if (lobsterInstallWizardStep.value === 3) {
    lobsterInstallWizardStep.value = 4;
    await startLobsterInstallFromWizard();
    return;
  }

  if (lobsterInstallWizardStep.value === 5) {
    closeLobsterInstallWizard();
  }
}

function handleLobsterInstallWizardBack() {
  if (lobsterInstallRunning.value) {
    return;
  }
  if (lobsterInstallWizardStep.value > 1 && lobsterInstallWizardStep.value < 5) {
    lobsterInstallWizardStep.value = (lobsterInstallWizardStep.value - 1) as LobsterInstallWizardStep;
  }
}

async function runLobsterAction(action: LobsterActionId) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke || lobsterActionRunning.value) {
    return;
  }

  if (action === "install") {
    openLobsterInstallWizard();
    return;
  }

  if (action === "restore") {
    if (!selectedLobsterBackupPath.value) {
      statusText.value = "请先创建备份，或选择一个可恢复备份。";
      return;
    }
    const backupLabel =
      selectedLobsterBackupPath.value.split(/[\\/]/).filter(Boolean).pop() ?? selectedLobsterBackupPath.value;
    const confirmed = window.confirm(`确认从备份「${backupLabel}」恢复吗？当前目录会先自动留存一份快照。`);
    if (!confirmed) {
      return;
    }
  }

  lobsterActionRunning.value = action;
  try {
    const payload = action === "restore"
      ? { action, backupPath: selectedLobsterBackupPath.value }
      : { action };
    const result = (await invoke("run_lobster_action", payload)) as LobsterActionResult;
    lobsterActionResult.value = result;
    statusText.value = result.success
      ? `${getLobsterActionTitle(action)}执行完成。`
      : `${getLobsterActionTitle(action)}执行失败：${result.detail}`;
    await refreshLobsterSnapshot();
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : `${getLobsterActionTitle(action)}执行失败。`;
  } finally {
    lobsterActionRunning.value = null;
  }
}

async function setWindowIgnoreCursorEvents(nextValue: boolean) {
  if (ignoreCursorEvents === nextValue) {
    return;
  }

  const tauriApi = getTauriApi();
  const currentWindow = tauriApi?.window?.getCurrentWindow?.();

  if (!currentWindow?.setIgnoreCursorEvents) {
    return;
  }

  await currentWindow.setIgnoreCursorEvents(nextValue, nextValue ? { forward: true } : undefined);
  ignoreCursorEvents = nextValue;
}

async function applyAlwaysOnTop(value: boolean) {
  const tauriApi = getTauriApi();
  const currentWindow = tauriApi?.window?.getCurrentWindow?.();
  const win = currentWindow as (typeof currentWindow & { setAlwaysOnTop?: (v: boolean) => Promise<void> }) | undefined;
  if (win?.setAlwaysOnTop) {
    await win.setAlwaysOnTop(value);
  }
}

function getAutostartApi() {
  const autostart = getTauriApi()?.autostart as
    | {
      enable?: () => Promise<void> | void;
      disable?: () => Promise<void> | void;
      isEnabled?: () => Promise<boolean> | boolean;
    }
    | undefined;
  if (typeof autostart?.enable !== "function" || typeof autostart.disable !== "function" || typeof autostart.isEnabled !== "function") {
    return null;
  }
  return autostart as {
    enable: () => Promise<void> | void;
    disable: () => Promise<void> | void;
    isEnabled: () => Promise<boolean> | boolean;
  };
}

async function refreshLaunchOnLoginState() {
  const autostart = getAutostartApi();
  if (!autostart) {
    launchOnLoginSupported.value = false;
    launchOnLoginEnabled.value = false;
    return;
  }

  try {
    const enabled = await Promise.resolve(autostart.isEnabled());
    launchOnLoginSupported.value = true;
    launchOnLoginEnabled.value = Boolean(enabled);
  } catch {
    launchOnLoginSupported.value = false;
    launchOnLoginEnabled.value = false;
  }
}

async function applyLaunchOnLogin(nextValue: boolean) {
  const autostart = getAutostartApi();
  if (!autostart) {
    return false;
  }
  try {
    if (nextValue) {
      await Promise.resolve(autostart.enable());
    } else {
      await Promise.resolve(autostart.disable());
    }
    launchOnLoginEnabled.value = nextValue;
    return true;
  } catch {
    return false;
  }
}

function bindSystemThemeListener() {
  if (typeof window === "undefined" || !window.matchMedia) {
    return;
  }
  systemThemeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeMediaListener = () => {
    if (appTheme.value === "system") {
      setAppTheme("system", { persist: false });
    }
  };

  if (systemThemeMediaQuery.addEventListener) {
    systemThemeMediaQuery.addEventListener("change", systemThemeMediaListener);
  } else {
    systemThemeMediaQuery.addListener(systemThemeMediaListener);
  }
}

function unbindSystemThemeListener() {
  if (!systemThemeMediaQuery || !systemThemeMediaListener) {
    return;
  }
  if (systemThemeMediaQuery.removeEventListener) {
    systemThemeMediaQuery.removeEventListener("change", systemThemeMediaListener);
  } else {
    systemThemeMediaQuery.removeListener(systemThemeMediaListener);
  }
  systemThemeMediaQuery = null;
  systemThemeMediaListener = null;
}

function openSystemSettings() {
  // 打开时用当前真实值初始化草稿
  systemSettingsPreviewBaseLocale.value = appLocale.value;
  systemSettingsPreviewBaseTheme.value = appTheme.value;
  draftSizeLevel.value = petSizeLevel.value;
  draftAlwaysOnTop.value = petAlwaysOnTop.value;
  draftAppLocale.value = appLocale.value;
  draftAppTheme.value = appTheme.value;
  draftLaunchOnLoginEnabled.value = launchOnLoginEnabled.value;
  isSystemSettingsOpen.value = true;
  void refreshLaunchOnLoginState().then(() => {
    draftLaunchOnLoginEnabled.value = launchOnLoginEnabled.value;
  });
  void syncCursorPassThrough();
}

function closeSystemSettingsInternal(options?: { revertPreview?: boolean }) {
  const shouldRevertPreview = options?.revertPreview !== false;
  if (shouldRevertPreview) {
    if (systemSettingsPreviewBaseLocale.value) {
      setAppLocale(systemSettingsPreviewBaseLocale.value, { persist: false });
    }
    if (systemSettingsPreviewBaseTheme.value) {
      setAppTheme(systemSettingsPreviewBaseTheme.value, { persist: false });
    }
  }
  systemSettingsPreviewBaseLocale.value = null;
  systemSettingsPreviewBaseTheme.value = null;
  isSystemSettingsOpen.value = false;
  void syncCursorPassThrough();
}

function closeSystemSettings() {
  closeSystemSettingsInternal();
}

async function handleSystemSettingsSave() {
  petSizeLevel.value = draftSizeLevel.value;
  petAlwaysOnTop.value = draftAlwaysOnTop.value;
  setAppLocale(draftAppLocale.value);
  setAppTheme(draftAppTheme.value);
  safeLocalStorageSetItem("keai.desktop-pet.size-level", petSizeLevel.value);
  safeLocalStorageSetItem("keai.desktop-pet.always-on-top", String(petAlwaysOnTop.value));
  await applyAlwaysOnTop(petAlwaysOnTop.value);

  let startupFailed = false;
  if (launchOnLoginSupported.value && draftLaunchOnLoginEnabled.value !== launchOnLoginEnabled.value) {
    startupFailed = !(await applyLaunchOnLogin(draftLaunchOnLoginEnabled.value));
  }

  closeSystemSettingsInternal({ revertPreview: false });
  statusText.value = startupFailed ? tr("system.settings.startup.unsupported") : tr("status.settings_saved");
}

async function closeDesktopPet() {
  const tauriApi = getTauriApi();
  const currentWindow = tauriApi?.window?.getCurrentWindow?.();

  if (tauriApi?.core?.invoke) {
    try {
      await tauriApi.core.invoke("quit_app");
      return;
    } catch {
      // Fall through to window-level termination.
    }
  }

  if (currentWindow?.destroy) {
    await currentWindow.destroy();
    return;
  }

  if (currentWindow?.close) {
    await currentWindow.close();
    return;
  }

  window.close();
}

function handleEscape(event: KeyboardEvent) {
  if (event.key !== "Escape" || !isWindowActive.value) {
    return;
  }

  if (isLobsterInstallWizardOpen.value) {
    closeLobsterInstallWizard();
    event.preventDefault();
    return;
  }

  if (activeSkillMarketDetail.value) {
    closeSkillMarketDetailModal();
    event.preventDefault();
    return;
  }

  if (activeRecentOutputMember.value) {
    closeRecentOutputModal();
    event.preventDefault();
    return;
  }

  if (sessionOverlayLog.value) {
    sessionOverlayLogId.value = null;
    event.preventDefault();
    return;
  }

  if (isSystemSettingsOpen.value) {
    closeSystemSettings();
    event.preventDefault();
    return;
  }

  if (staffDeleteTargetMember.value) {
    closeStaffDeleteConfirm();
    event.preventDefault();
    return;
  }

  if (contextMenu.value.visible) {
    hideContextMenu();
    event.preventDefault();
    return;
  }

  if (isChatOpen.value) {
    toggleChatPanel(false);
    event.preventDefault();
    return;
  }

  if (isConsoleOpen.value) {
    toggleConsolePanel(false);
    event.preventDefault();
    return;
  }

  event.preventDefault();
  const confirmed = window.confirm("确认关闭桌宠程序吗？");

  if (confirmed) {
    void closeDesktopPet();
  }
}

function handleFocus() {
  isWindowActive.value = true;
}

function handleBlur() {
  isWindowActive.value = false;
}

function handleVisibilityChange() {
  isWindowActive.value = !document.hidden && document.hasFocus();
  if (shouldAutoRefreshStaffSnapshot()) {
    void refreshStaffSnapshot();
  }
}

function handleContextMenu(event: MouseEvent) {
  event.preventDefault();
  const initialPosition = clampContextMenuPosition(
    event.clientX,
    event.clientY,
    CONTEXT_MENU_FALLBACK_WIDTH,
    CONTEXT_MENU_FALLBACK_HEIGHT
  );

  contextMenu.value = {
    visible: true,
    x: initialPosition.x,
    y: initialPosition.y
  };

  void nextTick(() => {
    adjustContextMenuToViewport();
  });
}

function handleWindowPointerDown(event: PointerEvent) {
  if (event.target instanceof HTMLElement && event.target.closest("select")) {
    lastSelectInteractionAt = performance.now();
  }

  if (activeRoleWorkflowBase.value && event.target instanceof HTMLElement && event.target.closest(".role-workflow-detail-modal")) {
    return;
  }

  if (staffDeleteTargetMember.value && event.target instanceof HTMLElement && event.target.closest(".staff-delete-modal")) {
    return;
  }

  if (isLobsterInstallWizardOpen.value && event.target instanceof HTMLElement && event.target.closest(".lobster-install-wizard-modal")) {
    return;
  }

  if (activeSkillMarketDetail.value && event.target instanceof HTMLElement && event.target.closest(".skill-market-detail-modal")) {
    return;
  }

  if (activeRecentOutputMember.value && event.target instanceof HTMLElement && event.target.closest(".recent-output-modal")) {
    return;
  }

  if (sessionOverlayLog.value && event.target instanceof HTMLElement && event.target.closest(".session-log-overlay")) {
    return;
  }

  if (
    !(event.target instanceof HTMLElement) ||
    (!event.target.closest(".desktop-context-menu") &&
      !event.target.closest(".desktop-console-panel") &&
      !event.target.closest(".desktop-chat-window"))
  ) {
    hideContextMenu();
  }
}

function handleChatDragStart(event: PointerEvent) {
  if (!(event.target instanceof HTMLElement) || event.target.closest("button, textarea, input, select")) {
    return;
  }

  const rect = chatPanelRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  chatMovePointerId = event.pointerId;
  chatMoveStart = {
    x: event.clientX,
    y: event.clientY,
    panelX: rect.left,
    panelY: rect.top
  };
  captureCurrentChatPlacement();
  chatPanelRef.value?.setPointerCapture(event.pointerId);
}

function handleChatResizeStart(event: PointerEvent) {
  event.preventDefault();
  event.stopPropagation();
  const rect = chatPanelRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  chatResizePointerId = event.pointerId;
  chatResizeStart = {
    x: event.clientX,
    y: event.clientY,
    width: rect.width,
    height: rect.height
  };
  captureCurrentChatPlacement();
  chatPanelRef.value?.setPointerCapture(event.pointerId);
}

function handlePanelDragStart(event: PointerEvent) {
  if (!(event.target instanceof HTMLElement) || event.target.closest("button, textarea, input, select")) {
    return;
  }

  const rect = consolePanelRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  panelMovePointerId = event.pointerId;
  panelMoveStart = {
    x: event.clientX,
    y: event.clientY,
    panelX: rect.left,
    panelY: rect.top
  };
  captureCurrentPanelPlacement();
  consolePanelRef.value?.setPointerCapture(event.pointerId);
}

function startConsoleWindowDrag(event: PointerEvent) {
  const invoke = getTauriApi()?.core?.invoke;
  if (invoke) {
    event.preventDefault();
    void invoke("start_console_window_drag");
    return;
  }

  const currentWindow = getTauriApi()?.window?.getCurrentWindow?.();
  if (currentWindow?.startDragging) {
    event.preventDefault();
    void currentWindow.startDragging();
  }
}

function handleConsoleHeaderPointerDown(event: PointerEvent) {
  if (!isConsoleWindowMode) {
    handlePanelDragStart(event);
  }
}

function handleConsolePanelPointerDown(event: PointerEvent) {
  if (!isConsoleWindowMode || event.button !== 0 || !(event.target instanceof HTMLElement)) {
    return;
  }

  const target = event.target;
  if (
    target.closest(
      "button, textarea, input, select, a, label, [role='switch'], .desktop-console-panel__resize-handle"
    )
  ) {
    return;
  }

  const className = target.className;
  const isBodySurface =
    typeof className === "string" && (className.includes("desktop-console-body") || className.includes("desktop-console-nav"));
  const isDragSurface =
    target === consolePanelRef.value ||
    target.classList.contains("desktop-console-panel__header") ||
    target.classList.contains("desktop-console-panel__actions") ||
    isBodySurface;

  if (!isDragSurface) {
    return;
  }

  startConsoleWindowDrag(event);
}

async function closeConsoleWindow() {
  if (!isConsoleWindowMode) {
    toggleConsolePanel(false);
    return;
  }

  const invoke = getTauriApi()?.core?.invoke;
  if (invoke) {
    try {
      await invoke("close_console_window");
      return;
    } catch {
      // Ignore and fallback to direct window API below.
    }
  }

  const currentWindow = getTauriApi()?.window?.getCurrentWindow?.();
  if (currentWindow?.close) {
    await currentWindow.close();
    return;
  }

  toggleConsolePanel(false);
}

function handlePanelResizeStart(event: PointerEvent) {
  event.preventDefault();
  event.stopPropagation();
  const rect = consolePanelRef.value?.getBoundingClientRect();
  if (!rect) {
    return;
  }

  panelResizePointerId = event.pointerId;
  panelResizeStart = {
    x: event.clientX,
    y: event.clientY,
    width: rect.width,
    height: rect.height
  };
  captureCurrentPanelPlacement();
  consolePanelRef.value?.setPointerCapture(event.pointerId);
}

async function submitChat() {
  const text = chatInput.value.trim();
  const pendingAttachments = [...chatAttachments.value];
  if ((!text && pendingAttachments.length === 0) || isSending.value) {
    return;
  }

  if (activeBoundPet.value) {
    await submitBoundPetChat(activeBoundPet.value, text, pendingAttachments);
    return;
  }

  const platform = activePlatform.value;
  const pendingId = createMessageId("assistant");
  const conversationHistory = [...openClawMessages.value];
  const agent = activeChatAgent.value;
  const attachmentSummary = pendingAttachments.length > 0
    ? `\n\n[附件: ${pendingAttachments.map((a) => a.name).join(", ")}]`
    : "";
  const userContent = (text || "(附件)") + attachmentSummary;
  const messages: OpenClawMessage[] = [
    ...conversationHistory,
    {
      role: "user",
      content: userContent
    }
  ];
  const effectivePlatform = !agent ? platform : null;
  const agentId = agent?.agentId ?? null;
  const requestEndpoint = effectivePlatform ? buildPlatformRequestEndpoint(effectivePlatform) : null;
  const endpoint = agentId ? `openclaw://agent/${agentId}` : (requestEndpoint ?? "openclaw://default");
  const protocol: PlatformProtocol = effectivePlatform?.protocol ?? "openai";
  const payload = { messages };
  const requestBody = safeJson(payload);
  const requestHeaders = buildRequestHeaders(protocol, effectivePlatform?.apiKey);
  const baseUrl = agentId
    ? `openclaw://agent/${agentId}`
    : (effectivePlatform ? getPlatformDisplayBaseUrl(effectivePlatform) : "openclaw://default");
  const path = agentId ? "" : (effectivePlatform ? normalizeApiPath(effectivePlatform.apiPath) : "");
  const platformId = agentId ? `openclaw-agent-${agentId}` : (effectivePlatform?.id ?? "openclaw-default");
  const platformName = agent ? `OpenClaw / ${stripRoleLabel(agent.displayName)}` : (effectivePlatform?.name ?? "OpenClaw 默认通道");
  const startedAt = performance.now();
  const startedAtMs = Date.now();

  chatMessages.value.push({
    id: createMessageId("user"),
    role: "user",
    text: text || "(附件)",
    status: "done",
    createdAt: startedAtMs,
    attachments: pendingAttachments.length > 0 ? pendingAttachments : undefined
  });
  chatMessages.value.push({
    id: pendingId,
    role: "assistant",
    text: agent ? `${stripRoleLabel(agent.displayName)} 正在思考中...` : "OpenClaw 正在思考中...",
    status: "pending",
    createdAt: Date.now()
  });
  chatInput.value = "";
  chatAttachments.value = [];
  isSending.value = true;
  noteInteraction();
  applyBaseAnimation(true);
  statusText.value = `消息已经发给 ${platformName}，正在等待回复。`;
  startBubbleAnimation();
  scrollMessagesToBottom();

  try {
    const response = await sendOpenClawChat(messages, {
      agentId: agentId ?? null,
      endpoint: agentId ? null : requestEndpoint,
      apiKey: agentId ? null : (effectivePlatform?.apiKey ?? null),
      model: agentId ? null : (effectivePlatform?.model ?? null),
      protocol
    });
    const completedAt = performance.now();
    const duration = Math.round(completedAt - startedAt);
    const promptTokens = response.usage?.promptTokens ?? estimateTokenCount(requestBody);
    const completionTokens = response.usage?.completionTokens ?? estimateTokenCount(response.text);
    const totalTokens = response.usage?.totalTokens ?? promptTokens + completionTokens;
    const runtimeLogs = await refreshOpenClawMessageLogs();
    insertRuntimeToolMessages(pendingId, runtimeLogs, startedAtMs);
    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = response.text;
      pendingMessage.status = "done";
    }

    localRequestLogs.value = appendRequestLog({
      sessionId: currentSessionId.value,
      platformId,
      platformName,
      protocol,
      method: "POST",
      endpoint,
      baseUrl,
      path,
      requestHeaders,
      requestBody,
      responseStatus: response.status ?? 200,
      responseBody: response.raw ?? response.text,
      streamSummary: response.text,
      duration,
      firstTokenTime: duration,
      tokensPerSecond:
        completionTokens && completedAt - startedAt > 0
          ? (completionTokens / (completedAt - startedAt)) * 1000
          : undefined,
      promptTokens,
      completionTokens,
      totalTokens,
      cacheReadInputTokens: response.usage?.cacheReadInputTokens
    });

    noteInteraction();
    wakeThenAnimate("have_meal", resolveBaseAnimationName());
    statusText.value = "OpenClaw 已回复，你可以继续追问。";
  } catch (error) {
    const duration = Math.round(performance.now() - startedAt);
    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    const errorText = error instanceof Error ? error.message : "OpenClaw 调用失败，请稍后再试。";
    const promptTokens = estimateTokenCount(requestBody);
    if (pendingMessage) {
      pendingMessage.text = errorText;
      pendingMessage.status = "error";
    }

    localRequestLogs.value = appendRequestLog({
      sessionId: currentSessionId.value,
      platformId,
      platformName,
      protocol,
      method: "POST",
      endpoint,
      baseUrl,
      path,
      requestHeaders,
      requestBody,
      responseStatus: 0,
      responseBody: "",
      duration,
      error: errorText,
      promptTokens,
      completionTokens: 0,
      totalTokens: promptTokens
    });

    statusText.value = "这次没有连上目标平台，我已经把失败原因记到日志里了。";
    openLogAnalysis("failures");
  } finally {
    isSending.value = false;
    applyBaseAnimation();
    startBubbleAnimation();
    scrollMessagesToBottom();
  }
}

function handleComposerKeydown(event: KeyboardEvent) {
  if (event.key !== "Enter" || event.shiftKey) {
    return;
  }

  event.preventDefault();
  void submitChat();
}

const MAX_ATTACHMENT_SIZE = 10 * 1024 * 1024;
const MAX_ATTACHMENT_COUNT = 5;

function createAttachmentId() {
  return `att-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error(`无法读取文件: ${file.name}`));
    reader.readAsDataURL(file);
  });
}

async function addFiles(files: FileList | File[]) {
  const remaining = MAX_ATTACHMENT_COUNT - chatAttachments.value.length;
  if (remaining <= 0) {
    statusText.value = `最多只能添加 ${MAX_ATTACHMENT_COUNT} 个附件。`;
    return;
  }

  const toAdd = Array.from(files).slice(0, remaining);
  for (const file of toAdd) {
    if (file.size > MAX_ATTACHMENT_SIZE) {
      statusText.value = `文件「${file.name}」超过 10 MB 限制，已跳过。`;
      continue;
    }

    try {
      const dataUrl = await readFileAsDataUrl(file);
      chatAttachments.value.push({
        id: createAttachmentId(),
        name: file.name,
        size: file.size,
        type: file.type,
        dataUrl
      });
    } catch {
      statusText.value = `文件「${file.name}」读取失败。`;
    }
  }
}

function removeAttachment(id: string) {
  chatAttachments.value = chatAttachments.value.filter((a) => a.id !== id);
}

function triggerFileInput() {
  fileInputRef.value?.click();
}

function handleFileInputChange(event: Event) {
  const input = event.target as HTMLInputElement;
  if (input.files && input.files.length > 0) {
    void addFiles(input.files);
    input.value = "";
  }
}

function handleComposerDragEnter(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = true;
}

function handleComposerDragOver(event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = "copy";
  }
}

function handleComposerDragLeave(event: DragEvent) {
  const related = event.relatedTarget as Node | null;
  const container = (event.currentTarget as HTMLElement);
  if (!related || !container.contains(related)) {
    isDragOver.value = false;
  }
}

function handleComposerDrop(event: DragEvent) {
  event.preventDefault();
  isDragOver.value = false;
  if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
    void addFiles(event.dataTransfer.files);
  }
}

function isImageAttachment(attachment: ChatAttachment) {
  return attachment.type.startsWith("image/");
}

async function handlePaste(event: ClipboardEvent) {
  const items = event.clipboardData?.items;
  if (!items) return;

  const files: File[] = [];
  for (const item of items) {
    if (item.kind === "file") {
      const file = item.getAsFile();
      if (file) files.push(file);
    }
  }

  if (files.length > 0) {
    event.preventDefault();
    void addFiles(files);
  }
}

async function handleQuitClick() {
  hideContextMenu();
  await closeDesktopPet();
}

function handleOpenSessionLog(log: RequestLog) {
  selectedSessionLogId.value = log.id;
  sessionOverlayLogId.value = log.id;
}

function closeSessionLogOverlay() {
  sessionOverlayLogId.value = null;
  void syncCursorPassThrough();
}

function handleUsePreset(preset: Omit<PlatformDraft, "id">) {
  platformForm.value = createPlatformDraft({
    ...preset,
    enabled: true
  });
  editingPlatformId.value = null;
  isEditingPlatform.value = true;
  isPlatformModalOpen.value = true;
  openConsole("platforms");
}

function resetPresetSelection() {
  selectedPresetKey.value = "";
}

function handlePresetSelect() {
  const preset = platformPresets.find((item) => `${item.region}:${item.name}` === selectedPresetKey.value);
  if (!preset) {
    return;
  }

  handleUsePreset(preset);
  resetPresetSelection();
}

function handleCreatePlatform() {
  platformForm.value = createPlatformDraft();
  editingPlatformId.value = null;
  isEditingPlatform.value = true;
  isPlatformModalOpen.value = true;
  resetPresetSelection();
}

function handleEditPlatform(platform: PlatformConfig) {
  platformForm.value = {
    id: platform.id,
    name: platform.name,
    protocol: platform.protocol,
    baseUrl: platform.baseUrl,
    pathPrefix: platform.pathPrefix,
    apiPath: platform.apiPath,
    apiKey: platform.apiKey,
    model: platform.model,
    enabled: platform.enabled
  };
  editingPlatformId.value = platform.id;
  isEditingPlatform.value = true;
  isPlatformModalOpen.value = true;
}

function handleCancelPlatformEdit() {
  isEditingPlatform.value = false;
  editingPlatformId.value = null;
  isPlatformModalOpen.value = false;
  platformForm.value = createPlatformDraft();
  resetPresetSelection();
  void syncCursorPassThrough();
}

function handleSavePlatform() {
  const nextName = platformForm.value.name.trim();
  if (!nextName) {
    window.alert("平台名称不能为空。");
    return;
  }

  const nextProtocol = platformForm.value.protocol;
  const nextApiPath = nextProtocol === "anthropic" ? "/v1/messages" : "/v1/chat/completions";

  const nextPlatforms = upsertPlatform(platforms.value, {
    ...platformForm.value,
    name: nextName,
    pathPrefix: normalizePathPrefix(platformForm.value.pathPrefix),
    apiPath: nextApiPath
  });

  platforms.value = nextPlatforms;
  if (!activePlatformId.value || editingPlatformId.value === null) {
    activePlatformId.value = platformForm.value.id;
    setActivePlatform(platformForm.value.id);
  }
  statusText.value = `${nextName} 已保存。`;
  handleCancelPlatformEdit();
}

function handleDeletePlatform(platformId: string) {
  const target = platforms.value.find((item) => item.id === platformId);
  if (!target) {
    return;
  }

  const confirmed = window.confirm(`确定删除平台 “${target.name}” 吗？`);
  if (!confirmed) {
    return;
  }

  platforms.value = deletePlatform(platforms.value, platformId);
  activePlatformId.value = loadActivePlatformId();
  statusText.value = `${target.name} 已删除。`;
}

function resetMemoryDraft() {
  memoryDraft.value = createEmptyMemoryDraft();
  selectedMemoryId.value = null;
}

function resetDocumentDraft() {
  documentDraft.value = createEmptyDocumentDraft();
  selectedDocumentId.value = null;
}

async function handleSaveMemory() {
  if (!memoryDraft.value.sourcePath) {
    window.alert("请先选择一份记忆文件。");
    return;
  }
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    window.alert("当前环境不支持保存记忆文件。");
    return;
  }
  await invoke("save_source_file", {
    kind: "memory",
    sourcePath: memoryDraft.value.sourcePath,
    content: memoryDraft.value.content
  });
  statusText.value = `记忆文件“${memoryDraft.value.title}”已保存。`;
  await refreshMemorySnapshot();
}

async function handleSaveDocument() {
  if (!documentDraft.value.source) {
    window.alert("请先选择一份文档文件。");
    return;
  }
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    window.alert("当前环境不支持保存文档文件。");
    return;
  }
  const documentKind = activeResourceModal.value === "skill" ? "skill" : activeResourceModal.value === "tool" ? "tool" : "document";
  await invoke("save_source_file", {
    kind: documentKind,
    sourcePath: documentDraft.value.source,
    content: documentDraft.value.content
  });
  statusText.value = `文档文件“${documentDraft.value.title}”已保存。`;
  if (activeResourceModal.value === "skill") {
    await refreshOpenClawSkillSnapshot();
    await refreshOpenClawResourceDocuments("skill", null);
  } else if (activeResourceModal.value === "tool" && activeResourceMemberId.value) {
    await refreshOpenClawResourceDocuments("tool", activeResourceMemberId.value);
  } else {
    await refreshDocumentSnapshot();
  }
}

function createMemoryDraftFromRecord(record: MemoryRecord): MemoryDraft {
  return {
    id: record.id,
    title: record.title,
    owner: record.owner,
    scope: record.scope,
    summary: record.summary,
    content: record.content,
    sourcePath: record.sourcePath,
    relativePath: record.relativePath
  };
}

function createDocumentDraftFromRecord(record: DocumentRecord): DocumentDraft {
  return {
    id: record.id,
    title: record.title,
    category: record.category,
    owner: record.owner,
    source: record.source,
    summary: record.summary,
    content: record.content,
    relativePath: record.relativePath
  };
}

function handleSelectMemory(record: MemoryRecord) {
  selectedMemoryId.value = record.id;
  memoryDraft.value = createMemoryDraftFromRecord(record);
}

function handleSelectDocument(record: DocumentRecord) {
  selectedDocumentId.value = record.id;
  documentDraft.value = createDocumentDraftFromRecord(record);
}

function getStaffInitials(name: string) {
  const trimmed = name.trim();
  if (!trimmed) {
    return "AI";
  }

  return trimmed.slice(0, 2).toUpperCase();
}

const STAFF_AVATAR_COLORS = [
  "avatar-color-0",
  "avatar-color-1",
  "avatar-color-2",
  "avatar-color-3",
  "avatar-color-4",
  "avatar-color-5",
  "avatar-color-6",
  "avatar-color-7",
] as const;

function getStaffAvatarColorClass(agentId: string): string {
  let hash = 0;
  for (let i = 0; i < agentId.length; i++) {
    hash = (hash * 31 + (agentId.codePointAt(i) ?? 0)) >>> 0;
  }
  return STAFF_AVATAR_COLORS[hash % STAFF_AVATAR_COLORS.length];
}

function getStaffStatusClass(member: StaffMemberSnapshot) {
  const normalized = member.statusLabel.trim();
  if (normalized === "待命") return "is-active";
  if (normalized === "工作中" || normalized === "处理中") return "is-busy";
  if (normalized === "等待审核" || normalized === "需要支援" || normalized === "需要关注") return "is-offline";
  return "is-offline";
}

function getStaffRoleLabel(member: StaffMemberSnapshot) {
  return member.roleLabel || member.agentId;
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function escapeHtmlAttr(value: string) {
  return escapeHtml(value).replace(/`/g, "&#96;");
}

function sanitizeMarkdownHref(rawHref: string): string | null {
  const href = rawHref.trim();
  if (!href) {
    return null;
  }

  if (href.startsWith("/") || href.startsWith("./") || href.startsWith("../") || href.startsWith("#")) {
    return href;
  }

  try {
    const parsed = new URL(href);
    if (parsed.protocol === "http:" || parsed.protocol === "https:" || parsed.protocol === "mailto:") {
      return href;
    }
    return null;
  } catch {
    return null;
  }
}

function createMarkdownTokenStore() {
  const tokens: string[] = [];
  return {
    stash(html: string) {
      const key = `\u0000${tokens.length}\u0000`;
      tokens.push(html);
      return key;
    },
    restore(text: string) {
      return text.replace(/\u0000(\d+)\u0000/g, (_, index) => tokens[Number(index)] ?? "");
    }
  };
}

function renderInlineMarkdown(raw: string) {
  const store = createMarkdownTokenStore();
  let text = raw;

  text = text.replace(/`([^`\n]+)`/g, (_, code: string) => store.stash(`<code>${escapeHtml(code)}</code>`));
  text = text.replace(/\[([^\]\n]+)\]\(([^)\n]+)\)/g, (_, label: string, href: string) => {
    const safeHref = sanitizeMarkdownHref(href);
    if (!safeHref) {
      return store.stash(escapeHtml(label));
    }
    return store.stash(
      `<a href="${escapeHtmlAttr(safeHref)}" target="_blank" rel="noopener noreferrer">${escapeHtml(label)}</a>`
    );
  });

  text = escapeHtml(text);
  text = text.replace(/\*\*([^*\n]+)\*\*/g, "<strong>$1</strong>");
  text = text.replace(/__([^_\n]+)__/g, "<strong>$1</strong>");
  text = text.replace(/\*([^*\n]+)\*/g, "<em>$1</em>");
  text = text.replace(/_([^_\n]+)_/g, "<em>$1</em>");
  text = text.replace(/~~([^~\n]+)~~/g, "<del>$1</del>");

  return store.restore(text);
}

function renderMarkdown(raw: string) {
  const normalized = raw.replace(/\r\n/g, "\n").trim();
  if (!normalized) {
    return "<p>暂无产出。</p>";
  }

  const lines = normalized.split("\n");
  const blocks: string[] = [];
  const paragraphLines: string[] = [];
  const quoteLines: string[] = [];
  const listItems: string[] = [];
  let listType: "ul" | "ol" | null = null;

  function flushParagraph() {
    if (paragraphLines.length === 0) return;
    blocks.push(`<p>${paragraphLines.map((line) => renderInlineMarkdown(line)).join("<br />")}</p>`);
    paragraphLines.length = 0;
  }

  function flushQuote() {
    if (quoteLines.length === 0) return;
    blocks.push(`<blockquote><p>${quoteLines.map((line) => renderInlineMarkdown(line)).join("<br />")}</p></blockquote>`);
    quoteLines.length = 0;
  }

  function flushList() {
    if (!listType || listItems.length === 0) return;
    blocks.push(`<${listType}>${listItems.map((item) => `<li>${item}</li>`).join("")}</${listType}>`);
    listItems.length = 0;
    listType = null;
  }

  for (const line of lines) {
    const trimmedRight = line.trimEnd();
    if (!trimmedRight.trim()) {
      flushParagraph();
      flushQuote();
      flushList();
      continue;
    }

    const headingMatch = trimmedRight.match(/^(#{1,6})\s+(.+)$/);
    if (headingMatch) {
      flushParagraph();
      flushQuote();
      flushList();
      const level = headingMatch[1].length;
      blocks.push(`<h${level}>${renderInlineMarkdown(headingMatch[2])}</h${level}>`);
      continue;
    }

    const quoteMatch = trimmedRight.match(/^>\s?(.*)$/);
    if (quoteMatch) {
      flushParagraph();
      flushList();
      quoteLines.push(quoteMatch[1]);
      continue;
    }

    const unorderedMatch = trimmedRight.match(/^[-*+]\s+(.+)$/);
    if (unorderedMatch) {
      flushParagraph();
      flushQuote();
      if (listType !== "ul") {
        flushList();
        listType = "ul";
      }
      listItems.push(renderInlineMarkdown(unorderedMatch[1]));
      continue;
    }

    const orderedMatch = trimmedRight.match(/^\d+\.\s+(.+)$/);
    if (orderedMatch) {
      flushParagraph();
      flushQuote();
      if (listType !== "ol") {
        flushList();
        listType = "ol";
      }
      listItems.push(renderInlineMarkdown(orderedMatch[1]));
      continue;
    }

    flushQuote();
    flushList();
    paragraphLines.push(trimmedRight);
  }

  flushParagraph();
  flushQuote();
  flushList();

  return blocks.join("");
}

function hasRecentOutput(member: StaffMemberSnapshot) {
  return member.recentOutput.trim().length > 0;
}

function openRecentOutputModal(member: StaffMemberSnapshot) {
  if (!hasRecentOutput(member)) {
    return;
  }
  recentOutputModalMemberId.value = member.agentId;
  void syncCursorPassThrough();
}

function closeRecentOutputModal() {
  recentOutputModalMemberId.value = null;
  void syncCursorPassThrough();
}

function renderStaffRecentOutputMarkdown(output: string) {
  return renderMarkdown(output);
}

function normalizeStaffFacet(value: string) {
  return value.trim().toLowerCase();
}

function isRecordOwnedByMember(owner: string, member: StaffMemberSnapshot) {
  const normalizedOwner = normalizeStaffFacet(owner);
  if (!normalizedOwner) {
    return false;
  }

  return (
    normalizedOwner === normalizeStaffFacet(member.agentId) ||
    normalizedOwner === normalizeStaffFacet(member.displayName) ||
    normalizedOwner === normalizeStaffFacet(getStaffRoleLabel(member))
  );
}

function getMemberMemoryRecords(member: StaffMemberSnapshot) {
  return memoryRecords.value.filter((record) => isRecordOwnedByMember(record.owner, member));
}

function getMemberDocumentRecords(member: StaffMemberSnapshot) {
  return documentRecords.value.filter((record) => isRecordOwnedByMember(record.owner, member));
}

function estimateToolsEnabledCountByProfile(profile: string): number {
  const normalized = profile.trim().toLowerCase();
  if (normalized === "minimal") return 1;
  if (normalized === "coding") return 15;
  if (normalized === "messaging") return 5;
  return 23;
}

function getStaffLinkedResourceCounts(member: StaffMemberSnapshot) {
  const toolCount =
    typeof member.toolsEnabledCount === "number" && Number.isFinite(member.toolsEnabledCount)
      ? Math.max(0, Math.floor(member.toolsEnabledCount))
      : estimateToolsEnabledCountByProfile(member.toolsProfile);
  return {
    memory: getMemberMemoryRecords(member).length,
    skill: openClawSkillsTotalCount.value,
    tool: toolCount
  };
}

function closeResourceModal() {
  activeResourceModal.value = null;
  activeResourceMemberId.value = null;
  openClawToolsScope.value = "agent";
  openClawSkillCategory.value = "builtIn";
  resourceModalFilterText.value = "";
  resourceDocumentRecords.value = [];
  void syncCursorPassThrough();
}

async function openMemberResourceModal(member: StaffMemberSnapshot, kind: ResourceModalKind) {
  activeResourceMemberId.value = member.agentId;
  activeResourceModal.value = kind;
  resourceModalFilterText.value = "";
  if (kind === "skill") {
    openClawSkillCategory.value = "builtIn";
  }

  if (kind === "memory") {
    await refreshMemorySnapshot();
    const records = getMemberMemoryRecords(member);
    if (records[0]) {
      handleSelectMemory(records[0]);
      statusText.value = `已打开 ${member.displayName} 的记忆弹窗，可直接查看或编辑。`;
    } else {
      selectedMemoryId.value = null;
      memoryDraft.value = createEmptyMemoryDraft();
      statusText.value = `${member.displayName} 暂未发现可编辑记忆文件。`;
    }
    void syncCursorPassThrough();
    return;
  }

  if (kind === "skill") {
    await refreshOpenClawSkillsList();
    statusText.value = `已打开 OpenClaw 技能库，内置 ${openClawSkillsList.value.builtIn?.length ?? 0} 项、安装 ${openClawSkillsList.value.installed?.length ?? 0} 项。`;
    void syncCursorPassThrough();
    return;
  }

  if (kind === "tool") {
    openClawToolsScope.value = "agent";
    await refreshOpenClawToolsList(member.agentId, "agent");
    statusText.value = `已打开 ${member.displayName} 的工具权限配置，当前 Profile：${openClawToolsList.value.profileLabel}，${openClawToolsEnabledCount.value} 项已启用。`;
    void syncCursorPassThrough();
    return;
  }
}

async function handleOpenMemberMemory(member: StaffMemberSnapshot) {
  await openMemberResourceModal(member, "memory");
}

async function handleOpenMemberSkill(member: StaffMemberSnapshot) {
  await openMemberResourceModal(member, "skill");
}

async function handleOpenMemberTool(member: StaffMemberSnapshot) {
  await openMemberResourceModal(member, "tool");
}

async function refreshStaffSnapshot() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    staffMembers.value = [];
    staffSnapshotSourcePath.value = "";
    staffSnapshotDetail.value = "当前环境不支持读取 openclaw.json。";
    return false;
  }

  if (isStaffSnapshotRefreshing.value) {
    return false;
  }

  isStaffSnapshotRefreshing.value = true;
  try {
    const result = (await invoke("load_staff_snapshot")) as StaffSnapshotResponse;
    staffMembers.value = Array.isArray(result.members) ? result.members : [];
    if (recentOutputModalMemberId.value && !staffMembers.value.some((member) => member.agentId === recentOutputModalMemberId.value)) {
      closeRecentOutputModal();
    }
    boundPets.value = boundPets.value.map((pet) => ({
      ...pet,
      capabilities: createBoundPetCapabilities(pet.capabilities)
    }));
    persistBoundPets();
    staffSnapshotSourcePath.value = result.sourcePath ?? "";
    staffSnapshotDetail.value = result.detail ?? "员工配置读取完成。";
    staffMissionStatement.value = result.missionStatement || staffMissionStatement.value;
    return true;
  } catch (error) {
    staffMembers.value = [];
    closeRecentOutputModal();
    staffSnapshotSourcePath.value = "";
    staffSnapshotDetail.value = error instanceof Error ? error.message : "员工配置读取失败。";
    return false;
  } finally {
    isStaffSnapshotRefreshing.value = false;
  }
}

function shouldAutoRefreshStaffSnapshot() {
  return activeSection.value === "staff" && isConsoleOpen.value && !document.hidden;
}

async function handleRefreshStaffSnapshot() {
  const refreshed = await refreshStaffSnapshot();
  if (refreshed) {
    statusText.value = "员工配置已刷新，并已同步最新 OpenClaw Agent 状态。";
  } else {
    statusText.value = staffSnapshotDetail.value || "员工配置刷新失败。";
  }
}

function mapMemorySnapshotItem(item: SourceFileSnapshotItem): MemoryRecord {
  return {
    id: item.id,
    title: item.title,
    owner: item.facetLabel,
    scope: item.category,
    summary: item.summary,
    content: item.content,
    sourcePath: item.sourcePath,
    relativePath: item.relativePath,
    updatedAt: item.updatedAtMs,
    exists: item.exists
  };
}

function mapDocumentSnapshotItem(item: SourceFileSnapshotItem): DocumentRecord {
  return {
    id: item.id,
    title: item.title,
    category: item.category,
    owner: item.facetLabel,
    source: item.sourcePath,
    relativePath: item.relativePath,
    summary: item.summary,
    content: item.content,
    updatedAt: item.updatedAtMs,
    exists: item.exists
  };
}

async function refreshMemorySnapshot() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    memoryRecords.value = [];
    memorySnapshotSourcePath.value = "";
    memorySnapshotDetail.value = "当前环境不支持读取记忆文件。";
    return;
  }

  try {
    const result = (await invoke("load_memory_file_snapshot")) as SourceFileSnapshotResponse;
    memoryRecords.value = Array.isArray(result.items) ? result.items.map(mapMemorySnapshotItem) : [];
    memorySnapshotSourcePath.value = result.sourcePath ?? "";
    memorySnapshotDetail.value = result.detail ?? "记忆文件读取完成。";
    if (!memoryScopeOptions.value.some((scope) => scope.key === activeMemoryScope.value)) {
      activeMemoryScope.value = memoryScopeOptions.value[0]?.key ?? "";
    }
  } catch (error) {
    memoryRecords.value = [];
    memorySnapshotSourcePath.value = "";
    memorySnapshotDetail.value = error instanceof Error ? error.message : "记忆文件读取失败。";
    activeMemoryScope.value = "";
  }
}

async function refreshDocumentSnapshot() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    documentRecords.value = [];
    documentSnapshotSourcePath.value = "";
    documentSnapshotDetail.value = "当前环境不支持读取核心文档。";
    return;
  }

  try {
    const result = (await invoke("load_document_file_snapshot")) as SourceFileSnapshotResponse;
    documentRecords.value = Array.isArray(result.items) ? result.items.map(mapDocumentSnapshotItem) : [];
    documentSnapshotSourcePath.value = result.sourcePath ?? "";
    documentSnapshotDetail.value = result.detail ?? "核心文档读取完成。";
  } catch (error) {
    documentRecords.value = [];
    documentSnapshotSourcePath.value = "";
    documentSnapshotDetail.value = error instanceof Error ? error.message : "核心文档读取失败。";
  }
}

async function refreshOpenClawSkillSnapshot() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    openClawSkillRecords.value = [];
    return;
  }

  try {
    const result = (await invoke("load_openclaw_resource_snapshot", {
      kind: "skill",
      agentId: null
    })) as SourceFileSnapshotResponse;
    openClawSkillRecords.value = Array.isArray(result.items) ? result.items.map(mapDocumentSnapshotItem) : [];
  } catch {
    openClawSkillRecords.value = [];
  }
}

async function refreshOpenClawResourceDocuments(kind: Exclude<ResourceModalKind, "memory">, agentId: string | null) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    resourceDocumentRecords.value = [];
    return;
  }

  try {
    const result = (await invoke("load_openclaw_resource_snapshot", {
      kind,
      agentId: kind === "tool" && agentId ? agentId : null
    })) as SourceFileSnapshotResponse;
    resourceDocumentRecords.value = Array.isArray(result.items) ? result.items.map(mapDocumentSnapshotItem) : [];
  } catch {
    resourceDocumentRecords.value = [];
  }
}

/** 加载 OpenClaw 技能库（内置 + 安装） */
async function refreshOpenClawSkillsList() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    openClawSkillsList.value = { builtIn: [], installed: [] };
    return;
  }
  try {
    const result = (await invoke("load_openclaw_skills_list")) as {
      builtIn: OpenClawSkillListItem[];
      installed: OpenClawSkillListItem[];
    };
    openClawSkillsList.value = {
      builtIn: Array.isArray(result?.builtIn) ? result.builtIn : [],
      installed: Array.isArray(result?.installed) ? result.installed : []
    };
  } catch {
    openClawSkillsList.value = { builtIn: [], installed: [] };
  }
}

/** 切换技能启用状态并写回 openclaw.json */
async function setOpenClawSkillEnabled(skillId: string, enabled: boolean) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) return;
  try {
    await invoke("save_openclaw_skill_enabled", { skillId, enabled });
    const list = openClawSkillsList.value;
    const update = (arr: OpenClawSkillListItem[]) =>
      arr.map((s) => (s.id === skillId ? { ...s, enabled } : s));
    openClawSkillsList.value = {
      builtIn: update(list.builtIn ?? []),
      installed: update(list.installed ?? [])
    };
    statusText.value = `技能「${skillId}」已${enabled ? "启用" : "禁用"}。`;
  } catch (e) {
    statusText.value = (e instanceof Error ? e.message : "保存失败") + "";
  }
}

function resolveOpenClawToolsTargetAgentId(agentId: string | null, scope: OpenClawToolsScope) {
  if (scope === "global") return null;
  return agentId;
}

function resolveToolIdsByProfile(profile: string, fallbackTools: OpenClawToolListItem[]): Set<string> {
  const normalized = profile.trim().toLowerCase();
  if (normalized === "minimal") {
    return new Set(["session_status"]);
  }
  if (normalized === "coding") {
    return new Set([
      "read",
      "write",
      "edit",
      "apply_patch",
      "exec",
      "bash",
      "process",
      "sessions_list",
      "sessions_history",
      "sessions_send",
      "sessions_spawn",
      "session_status",
      "memory_search",
      "memory_get",
      "image"
    ]);
  }
  if (normalized === "messaging") {
    return new Set([
      "message",
      "sessions_list",
      "sessions_history",
      "sessions_send",
      "session_status"
    ]);
  }
  if (normalized === "default" || normalized === "full" || normalized === "") {
    return new Set(fallbackTools.map((tool) => tool.id));
  }
  return new Set(fallbackTools.filter((tool) => tool.enabled).map((tool) => tool.id));
}

/** 加载 OpenClaw 工具配置列表（支持当前员工 / 全局默认） */
async function refreshOpenClawToolsList(agentId: string | null, scope: OpenClawToolsScope = openClawToolsScope.value) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  const targetAgentId = resolveOpenClawToolsTargetAgentId(agentId, scope);
  if (!invoke) {
    openClawToolsList.value = { profile: "", profileLabel: "", tools: [] };
    return;
  }
  try {
    const result = (await invoke("load_openclaw_tools_list", { agentId: targetAgentId })) as {
      profile: string;
      profileLabel: string;
      tools: OpenClawToolListItem[];
    };
    openClawToolsList.value = {
      profile: result?.profile ?? "",
      profileLabel: result?.profileLabel ?? "",
      tools: Array.isArray(result?.tools) ? result.tools : []
    };
  } catch {
    openClawToolsList.value = { profile: "", profileLabel: "", tools: [] };
  }
}

async function saveOpenClawToolsConfig(nextProfile: string, nextTools: OpenClawToolListItem[]) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke || isOpenClawToolsSaving.value) return;

  const memberAgentId = activeResourceMember.value?.agentId ?? activeResourceMemberId.value ?? null;
  const scope = openClawToolsScope.value;
  const targetAgentId = resolveOpenClawToolsTargetAgentId(memberAgentId, scope);
  const enabledToolIds = nextTools.filter((tool) => tool.enabled).map((tool) => tool.id);

  isOpenClawToolsSaving.value = true;
  try {
    await invoke("save_openclaw_tools_config", {
      agentId: targetAgentId,
      scope,
      profile: nextProfile,
      enabledToolIds
    });
    await refreshOpenClawToolsList(memberAgentId, scope);
    await refreshStaffSnapshot();

    const profileLabel = openClawToolsList.value.profileLabel || openClawToolsList.value.profile || "default";
    statusText.value = `已保存${openClawToolsScopeLabel.value}工具权限：Profile ${profileLabel}，启用 ${openClawToolsEnabledCount.value} 项。`;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "工具权限保存失败。";
    throw error;
  } finally {
    isOpenClawToolsSaving.value = false;
  }
}

async function setOpenClawToolsProfile(profile: string) {
  const nextProfile = profile.trim() || "default";
  const previousTools = openClawToolsList.value.tools;
  const profileToolIds = resolveToolIdsByProfile(nextProfile, previousTools);
  const nextTools = previousTools.map((tool) => ({
    ...tool,
    enabled: profileToolIds.has(tool.id)
  }));
  const previous = {
    profile: openClawToolsList.value.profile,
    profileLabel: openClawToolsList.value.profileLabel,
    tools: previousTools
  };
  openClawToolsList.value = {
    ...openClawToolsList.value,
    profile: nextProfile,
    tools: nextTools
  };
  try {
    await saveOpenClawToolsConfig(nextProfile, nextTools);
  } catch {
    openClawToolsList.value = { ...openClawToolsList.value, ...previous, tools: previous.tools };
  }
}

async function setOpenClawToolEnabled(toolId: string, enabled: boolean) {
  const previousTools = openClawToolsList.value.tools;
  const nextTools = previousTools.map((tool) => (tool.id === toolId ? { ...tool, enabled } : tool));
  openClawToolsList.value = {
    ...openClawToolsList.value,
    tools: nextTools
  };
  try {
    await saveOpenClawToolsConfig(openClawToolsList.value.profile || "default", nextTools);
  } catch {
    openClawToolsList.value = {
      ...openClawToolsList.value,
      tools: previousTools
    };
  }
}

async function handleOpenClawToolsScopeChange(value: string) {
  const nextScope: OpenClawToolsScope = value === "global" ? "global" : "agent";
  openClawToolsScope.value = nextScope;
  const memberAgentId = activeResourceMember.value?.agentId ?? activeResourceMemberId.value ?? null;
  await refreshOpenClawToolsList(memberAgentId, nextScope);
  statusText.value = `已切换到${nextScope === "global" ? "全局默认" : "当前员工"}工具权限范围。`;
}

async function refreshTaskSnapshot() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    taskRecords.value = [];
    taskSnapshotSourcePath.value = "";
    taskSnapshotDetail.value = "当前环境不支持读取 cron/jobs.json。";
    return;
  }

  try {
    const result = (await invoke("load_task_snapshot")) as TaskSnapshotResponse;
    taskRecords.value = Array.isArray(result.jobs) ? result.jobs : [];
    taskSnapshotSourcePath.value = result.sourcePath ?? "";
    taskSnapshotDetail.value = result.detail ?? "任务调度读取完成。";
  } catch (error) {
    taskRecords.value = [];
    taskSnapshotSourcePath.value = "";
    taskSnapshotDetail.value = error instanceof Error ? error.message : "任务调度读取失败。";
  }
}

async function refreshOpenClawMessageLogs() {
  if (runtimeLogRefreshTask) {
    return runtimeLogRefreshTask;
  }

  runtimeLogRefreshTask = (async () => {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    runtimeRequestLogs.value = [];
    runtimeLogDetail.value = "当前环境不支持读取 OpenClaw 运行时消息。";
    return [];
  }

  try {
    const result = (await invoke("load_openclaw_message_logs")) as OpenClawMessageLogResponse;
    const nextLogs = Array.isArray(result.logs) ? result.logs : [];
    const nextFingerprint = nextLogs
      .slice(0, 8)
      .map((log) => `${log.id}:${log.createdAt}:${log.responseStatus}`)
      .join("|");
    const hasRuntimeLogUpdate = nextFingerprint !== runtimeLogLastFingerprint;

    runtimeRequestLogs.value = nextLogs;
    runtimeLogLastFingerprint = nextFingerprint;
    runtimeLogDetail.value = result.detail ?? "OpenClaw 运行时消息读取完成。";

    if (hasRuntimeLogUpdate && nextLogs.length > 0) {
      runtimeLogFollowActiveUntil = performance.now() + runtimeLogFollowWindowMs;
      window.clearTimeout(runtimeLogFollowTimer);
      runtimeLogFollowTimer = window.setTimeout(() => {
        applyBaseAnimation(true);
      }, runtimeLogFollowWindowMs + 80);
      applyBaseAnimation(true);
    }

    return runtimeRequestLogs.value;
  } catch (error) {
    runtimeRequestLogs.value = [];
    runtimeLogDetail.value = error instanceof Error ? error.message : "OpenClaw 运行时消息读取失败。";
    return [];
  } finally {
    runtimeLogRefreshTask = null;
  }
  })();

  return runtimeLogRefreshTask;
}

function formatDocumentStatus(exists: boolean) {
  return exists ? "源文件" : "缺失";
}

function formatTaskStatus(status: TaskSnapshotItem["statusKind"]) {
  if (status === "late") return "待执行";
  if (status === "scheduled") return "已启用";
  if (status === "disabled") return "已停用";
  return "未知";
}

function getTaskStatusWeight(status: TaskSnapshotItem["statusKind"]) {
  if (status === "late") return 0;
  if (status === "scheduled") return 1;
  if (status === "disabled") return 2;
  return 3;
}

function getTaskStatusClass(status: TaskSnapshotItem["statusKind"]) {
  if (status === "late") return "is-blocked";
  if (status === "scheduled") return "is-active";
  if (status === "disabled") return "is-done";
  return "is-queued";
}

function formatTaskScheduleKind(kind: string, deleteAfterRun: boolean) {
  if (deleteAfterRun) {
    return "单次";
  }

  if (kind === "cron") {
    return "周期";
  }

  if (kind === "at") {
    return "定时";
  }

  return "任务";
}

function getTaskScheduleClass(kind: string, deleteAfterRun: boolean) {
  if (deleteAfterRun) {
    return "is-focused";
  }

  if (kind === "cron") {
    return "is-routine";
  }

  return "is-critical";
}

async function toggleCronTaskEnabled(taskId: string, enabled: boolean) {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) return;
  try {
    await invoke("set_task_enabled", { taskId, enabled });
    await refreshTaskSnapshot();
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "任务状态切换失败。";
  }
}

function formatDueAt(value: number) {
  return `${formatTime(value)} 执行`;
}

function formatTaskRelativeDueAt(value: number) {
  const delta = value - Date.now();
  const hours = Math.round(Math.abs(delta) / (60 * 60 * 1000));

  if (hours < 1) {
    return delta >= 0 ? "1 小时内" : "已逾期";
  }

  if (delta >= 0) {
    return `${hours} 小时后`;
  }

  return `逾期 ${hours} 小时`;
}

function getCronTaskCardClass(status: string) {
  if (status === "late") return "ptask-card--in_progress";
  if (status === "scheduled") return "ptask-card--todo";
  if (status === "disabled") return "ptask-card--done";
  return "";
}

function formatCronNextRun(ms: number | null) {
  if (ms === null) return "—";
  const delta = ms - Date.now();
  const absDelta = Math.abs(delta);
  if (absDelta < 60 * 60 * 1000) {
    return delta >= 0 ? "1 小时内" : "已逾期";
  }
  const hours = Math.round(absDelta / (60 * 60 * 1000));
  if (hours < 24) {
    return delta >= 0 ? `${hours} 小时后` : `逾期 ${hours} 小时`;
  }
  const days = Math.round(hours / 24);
  return delta >= 0 ? `${days} 天后` : `逾期 ${days} 天`;
}

function formatCronTimestamp(ms: number | null) {
  if (ms === null) return "从未";
  return formatTime(ms);
}

async function handleTogglePlatform(platformId: string, enabled: boolean) {
  const current = platforms.value.find((item) => item.id === platformId);

  if (!current) {
    return;
  }

  const nextDirectBaseUrlMap = { ...platformDirectBaseUrlMap.value };
  if (!isLocalProxyBaseUrl(current.baseUrl)) {
    nextDirectBaseUrlMap[platformId] = normalizeBaseUrl(current.baseUrl);
  }
  const targetBaseUrl = enabled ? getPlatformProxyPersistBaseUrl(current) : getPlatformDirectTargetBaseUrl(current);
  const nextStoredBaseUrl = enabled ? getPlatformDirectTargetBaseUrl(current) : targetBaseUrl;

  try {
    await saveOpenClawProviderBaseUrl(platformId, targetBaseUrl);
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "写回 openclaw.json 失败。";
    return;
  }

  if (!enabled && !isLocalProxyBaseUrl(targetBaseUrl)) {
    nextDirectBaseUrlMap[platformId] = normalizeBaseUrl(targetBaseUrl);
  }
  platformDirectBaseUrlMap.value = nextDirectBaseUrlMap;
  persistPlatformDirectBaseUrlStates(nextDirectBaseUrlMap);

  const toggledPlatforms = setPlatformEnabled(platforms.value, platformId, enabled);
  platforms.value = toggledPlatforms.map((item) =>
    item.id === platformId
      ? {
          ...item,
          baseUrl: normalizeBaseUrl(nextStoredBaseUrl),
          apiPath: inferApiPathForBaseUrl(item.protocol, normalizeBaseUrl(nextStoredBaseUrl), item.apiPath),
          updatedAt: Date.now()
        }
      : item
  );
  persistPlatformProxyEnabledStates(platforms.value);

  if (enabled && !activePlatformId.value) {
    activePlatformId.value = platformId;
    setActivePlatform(platformId);
  }

  statusText.value = enabled ? `${current.name} 已切换到本地代理链接。` : `${current.name} 已恢复默认平台链接。`;
}

function handleSetActivePlatform(platformId: string) {
  activePlatformId.value = platformId;
  setActivePlatform(platformId);
  const target = platforms.value.find((item) => item.id === platformId);
  if (target) {
    statusText.value = `${target.name} 已设为当前默认平台。`;
  }
}

function handleNewConversation() {
  currentSessionId.value = createSessionId();
  chatMessages.value = createDefaultChatMessages();
  chatInput.value = "";
  chatAttachments.value = [];
  persistChatHistory(activeChatAgentId.value);
  agentChatHistories.value[activeChatAgentId.value ?? "__main__"] = createDefaultChatMessages();
  statusText.value = "新会话已创建，后续调用会归档到新的会话视图里。";
  openChatPanel();
}

function handleClearLogs() {
  const confirmed = window.confirm("确认清空桌宠本地记录的调用日志吗？OpenClaw 运行时消息会在下次刷新后继续显示。");
  if (!confirmed) {
    return;
  }

  localRequestLogs.value = clearRequestLogs();
  selectedLogId.value = null;
  selectedSessionId.value = null;
  selectedFailureKey.value = null;
  statusText.value = "本地调用日志已清空，OpenClaw 运行时消息仍会继续监控。";
}

function handleExportLogs() {
  const content = exportLogsAsJson(requestLogs.value);
  const blob = new Blob([content], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `clawpet-logs-${new Date().toISOString().slice(0, 10)}.json`;
  link.click();
  URL.revokeObjectURL(url);
}

async function syncCursorPassThrough() {
  const tauriApi = getTauriApi();
  const cursorPosition = tauriApi?.window?.cursorPosition;
  const invoke = tauriApi?.core?.invoke;

  if (!cursorPosition) {
    return;
  }

  const isAnyModalOpen =
    !!activeRoleWorkflowBase.value ||
    !!staffDeleteTargetMember.value ||
    isLobsterInstallWizardOpen.value ||
    !!activeRecentOutputMember.value ||
    (activeResourceModal.value && activeResourceMember.value) ||
    !!sessionOverlayLog.value ||
    isPlatformModalOpen.value ||
    isChannelConfigModalOpen.value ||
    isSystemSettingsOpen.value;

  if (isDragging.value || contextMenu.value.visible || isAnyModalOpen) {
    await setWindowIgnoreCursorEvents(false);
    if (isAnyModalOpen) return;
  }

  // Windows fallback: while console is open, keep the whole window interactive.
  // This avoids WebView2 dropdowns being interrupted by pass-through toggling.
  if (isWindowsRuntime && isConsoleOpen.value) {
    await setWindowIgnoreCursorEvents(false);
    return;
  }

  const isRecentSelectInteraction = performance.now() - lastSelectInteractionAt < 1400;
  // Windows/WebView2: keep native <select> interactive while the option
  // popup is open, otherwise cursor pass-through can swallow dropdown clicks.
  if (document.activeElement instanceof HTMLSelectElement || isRecentSelectInteraction) {
    await setWindowIgnoreCursorEvents(false);
    return;
  }

  const cursor = await cursorPosition();
  const scale = window.devicePixelRatio || 1;
  let cursorX = cursor.x / scale;
  let cursorY = cursor.y / scale;
  if (invoke) {
    try {
      const winPos = (await invoke("get_window_inner_position")) as { x: number; y: number };
      cursorX = (cursor.x - winPos.x) / scale;
      cursorY = (cursor.y - winPos.y) / scale;
    } catch {
      // 降级：不转换坐标（单屏或主屏时可能仍正确）
    }
  }
  const petRect = pet.value?.getBoundingClientRect();
  const menuRect = contextMenuRef.value?.getBoundingClientRect();
  const chatRect = chatPanelRef.value?.getBoundingClientRect();
  const panelRect = consolePanelRef.value?.getBoundingClientRect();

  const isInPet =
    !!petRect &&
    cursorX >= petRect.left &&
    cursorX <= petRect.right &&
    cursorY >= petRect.top &&
    cursorY <= petRect.bottom;

  const isInMenu =
    !!menuRect &&
    cursorX >= menuRect.left &&
    cursorX <= menuRect.right &&
    cursorY >= menuRect.top &&
    cursorY <= menuRect.bottom;

  const isInPanel =
    !!panelRect &&
    cursorX >= panelRect.left &&
    cursorX <= panelRect.right &&
    cursorY >= panelRect.top &&
    cursorY <= panelRect.bottom;

  const isInChat =
    !!chatRect &&
    cursorX >= chatRect.left &&
    cursorX <= chatRect.right &&
    cursorY >= chatRect.top &&
    cursorY <= chatRect.bottom;

  await setWindowIgnoreCursorEvents(!(isInPet || isInMenu || isInPanel || isInChat));
}

function handleWindowChatPointerMove(event: PointerEvent) {
  const bounds = stage.value?.getBoundingClientRect();
  if (!bounds) {
    return;
  }

  if (chatMovePointerId === event.pointerId && chatPlacement.value.mode === "manual") {
    const nextX = chatMoveStart.panelX + event.clientX - chatMoveStart.x;
    const nextY = chatMoveStart.panelY + event.clientY - chatMoveStart.y;
    chatPlacement.value = {
      ...chatPlacement.value,
      x: Math.min(Math.max(16, nextX), Math.max(16, bounds.width - chatPlacement.value.width - 16)),
      y: Math.min(Math.max(16, nextY), Math.max(16, bounds.height - chatPlacement.value.height - 16))
    };
  }

  if (chatResizePointerId === event.pointerId && chatPlacement.value.mode === "manual") {
    const nextWidth = chatResizeStart.width + event.clientX - chatResizeStart.x;
    const nextHeight = chatResizeStart.height + event.clientY - chatResizeStart.y;
    chatPlacement.value = {
      ...chatPlacement.value,
      width: Math.min(Math.max(420, nextWidth), bounds.width - 32),
      height: Math.min(Math.max(420, nextHeight), bounds.height - 32),
      x: Math.min(Math.max(16, chatPlacement.value.x), Math.max(16, bounds.width - Math.min(Math.max(420, nextWidth), bounds.width - 32) - 16)),
      y: Math.min(Math.max(16, chatPlacement.value.y), Math.max(16, bounds.height - Math.min(Math.max(420, nextHeight), bounds.height - 32) - 16))
    };
  }
}

function handleWindowChatPointerUp(event: PointerEvent) {
  if (chatMovePointerId === event.pointerId) {
    chatPanelRef.value?.releasePointerCapture(event.pointerId);
    chatMovePointerId = null;
  }

  if (chatResizePointerId === event.pointerId) {
    chatPanelRef.value?.releasePointerCapture(event.pointerId);
    chatResizePointerId = null;
  }
}

function handleWindowPanelPointerMove(event: PointerEvent) {
  const bounds = stage.value?.getBoundingClientRect();
  if (!bounds) {
    return;
  }

  if (panelMovePointerId === event.pointerId && panelPlacement.value.mode === "manual") {
    const nextX = panelMoveStart.panelX + event.clientX - panelMoveStart.x;
    const nextY = panelMoveStart.panelY + event.clientY - panelMoveStart.y;
    panelPlacement.value = {
      ...panelPlacement.value,
      x: Math.min(Math.max(16, nextX), Math.max(16, bounds.width - panelPlacement.value.width - 16)),
      y: Math.min(Math.max(16, nextY), Math.max(16, bounds.height - panelPlacement.value.height - 16))
    };
  }

  if (panelResizePointerId === event.pointerId && panelPlacement.value.mode === "manual") {
    const prefersWide = true;
    const nextWidth = panelResizeStart.width + event.clientX - panelResizeStart.x;
    const nextHeight = panelResizeStart.height + event.clientY - panelResizeStart.y;
    const availableWidth = Math.max(320, bounds.width - 32);
    const availableHeight = Math.max(320, bounds.height - 32);
    const minWidth = Math.min(getPanelMinWidth(prefersWide), availableWidth);
    const minHeight = Math.min(420, availableHeight);
    const width = Math.min(Math.max(minWidth, nextWidth), availableWidth);
    const height = Math.min(Math.max(minHeight, nextHeight), availableHeight);
    panelPlacement.value = {
      ...panelPlacement.value,
      width,
      height,
      x: Math.min(Math.max(16, panelPlacement.value.x), Math.max(16, bounds.width - width - 16)),
      y: Math.min(Math.max(16, panelPlacement.value.y), Math.max(16, bounds.height - height - 16))
    };
  }
}

function handleWindowPanelPointerUp(event: PointerEvent) {
  if (panelMovePointerId === event.pointerId) {
    consolePanelRef.value?.releasePointerCapture(event.pointerId);
    panelMovePointerId = null;
  }

  if (panelResizePointerId === event.pointerId) {
    consolePanelRef.value?.releasePointerCapture(event.pointerId);
    panelResizePointerId = null;
  }
}

function formatTime(value: number) {
  return new Date(value).toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit"
  });
}

function formatDuration(value: number) {
  return `${Math.max(0, Math.round(value))} ms`;
}

function formatFileSize(value?: number | null) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    return "-";
  }
  if (value < 1024) {
    return `${Math.round(value)} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  if (value < 1024 * 1024 * 1024) {
    return `${(value / (1024 * 1024)).toFixed(2)} MB`;
  }
  return `${(value / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatLatencyStat(value?: number) {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    return "-";
  }

  if (value < 1000) {
    return `${Math.round(value)}ms`;
  }

  return `${(value / 1000).toFixed(2)}s`;
}

function formatSpeed(value?: number) {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    return "-";
  }

  return `${value.toFixed(1)} tok/s`;
}

function getEffectivePromptTokens(log: RequestLog) {
  return typeof log.promptTokens === "number" ? log.promptTokens : estimateTokenCount(log.requestBody);
}

function getEffectiveCompletionTokens(log: RequestLog) {
  return typeof log.completionTokens === "number"
    ? log.completionTokens
    : estimateTokenCount(log.streamSummary || log.responseBody);
}

function getEffectiveTotalTokens(log: RequestLog) {
  if (typeof log.totalTokens === "number") {
    return log.totalTokens;
  }

  return getEffectivePromptTokens(log) + getEffectiveCompletionTokens(log);
}

function formatTokenPair(log: RequestLog) {
  return `${getEffectivePromptTokens(log)}/${getEffectiveCompletionTokens(log)}`;
}

function formatCacheTokens(log: RequestLog) {
  return typeof log.cacheReadInputTokens === "number" ? `${log.cacheReadInputTokens}` : "-";
}

function estimateTokenCount(value?: string) {
  if (!value?.trim()) {
    return 0;
  }

  return Math.max(1, Math.ceil(value.length * 0.25));
}

function getLogRequestUrl(log: RequestLog) {
  if (log.baseUrl || log.path) {
    return `${log.baseUrl ?? ""}${log.path ?? ""}` || log.endpoint;
  }

  return log.endpoint;
}

function getLogPlatformGroup(log: RequestLog): string {
  if (log.platformId === "openclaw-default" || log.platformId.startsWith("openclaw-agent-") || log.platformId.startsWith("openclaw-runtime-")) {
    return "OpenClaw";
  }
  return log.platformName;
}

function getLogAgentId(log: RequestLog): string | null {
  if (log.platformId.startsWith("openclaw-agent-")) {
    return log.platformId.slice("openclaw-agent-".length);
  }
  if (log.platformId.startsWith("openclaw-runtime-")) {
    const tail = log.platformId.slice("openclaw-runtime-".length);
    const sep = tail.indexOf("-");
    return sep > 0 ? tail.slice(0, sep) : tail;
  }
  return null;
}

function getLogAgentDisplayName(log: RequestLog): string {
  if (log.platformName.includes(" / ")) {
    return log.platformName.split(" / ").slice(1).join(" / ");
  }
  return getLogAgentId(log) ?? log.platformName;
}

function findPlatformById(platformId: string | null | undefined): PlatformConfig | null {
  if (!platformId) {
    return null;
  }

  return platforms.value.find((item) => item.id === platformId) ?? null;
}

function getPlatformPillClass(platformId: string | null | undefined) {
  const platform = findPlatformById(platformId);

  if (!platformId || platformId === "openclaw-default") {
    return "platform-pill--default";
  }

  if (!platform) {
    return "platform-pill--unknown";
  }

  if (!platform.enabled) {
    return "platform-pill--disabled";
  }

  if (platform.id === activePlatformId.value) {
    return "platform-pill--active";
  }

  return "platform-pill--normal";
}

function getPlatformOriginLabel(platformId: string | null | undefined) {
  if (!platformId || platformId === "openclaw-default") {
    return "默认通道";
  }

  if (platformId.startsWith("openclaw-runtime-")) {
    return "运行时";
  }

  const platform = findPlatformById(platformId);
  if (!platform) {
    return "外部记录";
  }

  return "已配置平台";
}

function getPlatformInitials(platformName: string | null | undefined) {
  const words = (platformName ?? "")
    .split(/[\s/·\-_.]+/)
    .map((item) => item.trim())
    .filter(Boolean);

  if (words.length >= 2) {
    return `${words[0][0] ?? ""}${words[1][0] ?? ""}`.toUpperCase();
  }

  const compact = (platformName ?? "").replace(/[\s/·\-_.]+/g, "");
  return compact.slice(0, 2).toUpperCase() || "PT";
}

function getPlatformIdentityToneClass(platformId: string | null | undefined, platformName: string | null | undefined) {
  if (!platformId || platformId === "openclaw-default") {
    return "platform-identity--default";
  }

  if (platformId.startsWith("openclaw-runtime-")) {
    return "platform-identity--runtime";
  }

  const source = `${platformId ?? ""}:${platformName ?? ""}`;
  const hash = Array.from(source).reduce((sum, char) => sum + char.charCodeAt(0), 0);
  const tones = ["platform-identity--amber", "platform-identity--sky", "platform-identity--mint", "platform-identity--rose"];
  return tones[hash % tones.length];
}

function getPlatformCardAccentClass(
  platformId: string | null | undefined,
  prefix: "log-card" | "session-card" | "mini-log-card"
): string {
  const platform = findPlatformById(platformId);
  if (!platformId || platformId === "openclaw-default") {
    return `${prefix}--platform-default`;
  }
  if (!platform) {
    return "";
  }
  if (platform.id === activePlatformId.value) {
    return `${prefix}--platform-active`;
  }
  return "";
}

function getPlatformMetaLabel(
  platformId: string | null | undefined,
  protocol: PlatformProtocol | null | undefined
): string {
  const platform = findPlatformById(platformId);
  const protocolLabel = protocol ? protocol.toUpperCase() : "";

  if (platform && platform.id === activePlatformId.value) {
    return protocolLabel ? `${protocolLabel} · 默认` : "默认";
  }

  return protocolLabel;
}

function getDefaultPreviewSection(log: RequestLog): PreviewSection {
  if (log.streamSummary?.trim()) {
    return "stream";
  }
  if (log.responseBody?.trim()) {
    return "response";
  }
  if (log.requestBody?.trim()) {
    return "request";
  }
  return "raw";
}

function formatJsonView(value?: string): JsonViewResult {
  if (!value?.trim()) {
    return { text: "", language: "text" };
  }

  try {
    return {
      text: JSON.stringify(JSON.parse(value), null, 2),
      language: "json"
    };
  } catch {
    return { text: value, language: "text" };
  }
}

function buildPreviewSections(log: RequestLog) {
  const sections: Array<{ id: PreviewSection; label: string; view: JsonViewResult }> = [];
  if (log.requestBody?.trim()) {
    sections.push({ id: "request", label: "请求体", view: formatJsonView(log.requestBody) });
  }
  if (log.responseBody?.trim()) {
    sections.push({ id: "response", label: "响应体", view: formatJsonView(log.responseBody) });
  }
  if (log.streamSummary?.trim()) {
    sections.push({ id: "stream", label: "流式汇总", view: formatJsonView(log.streamSummary) });
  }

  const rawText = [log.error, log.responseBody].filter(Boolean).join("\n\n").trim();
  if (rawText) {
    sections.push({ id: "raw", label: "原始响应", view: formatJsonView(rawText) });
  }

  if (!sections.length) {
    sections.push({ id: "raw", label: "原始响应", view: { text: "暂无可预览内容", language: "text" } });
  }

  return sections;
}

function getActivePreviewSection(log: RequestLog, section: PreviewSection) {
  const sections = buildPreviewSections(log);
  return sections.find((item) => item.id === section) ?? sections[0];
}

function maskSensitiveHeaders(headers?: Record<string, string>) {
  if (!headers) {
    return {};
  }

  return Object.fromEntries(
    Object.entries(headers).map(([key, value]) => {
      if (/authorization|x-api-key|api-key/i.test(key)) {
        return [key, value.length > 10 ? `${value.slice(0, 6)}******` : "******"];
      }

      return [key, value];
    })
  );
}

async function copyText(text: string, successMessage: string) {
  const value = text.trim();
  if (!value) {
    statusText.value = "当前没有可复制的内容。";
    return;
  }

  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(value);
    } else {
      const textarea = document.createElement("textarea");
      textarea.value = value;
      textarea.setAttribute("readonly", "true");
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
    }
    statusText.value = successMessage;
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "复制失败，请稍后再试。";
  }
}

function buildRequestHeaders(protocol: PlatformProtocol, apiKey?: string) {
  const headers: Record<string, string> = {
    "Content-Type": "application/json"
  };

  if (protocol === "anthropic") {
    headers["anthropic-version"] = "2023-06-01";
    if (apiKey?.trim()) {
      headers["x-api-key"] = apiKey.trim();
    }
    return headers;
  }

  if (apiKey?.trim()) {
    headers.Authorization = `Bearer ${apiKey.trim()}`;
  }

  return headers;
}

function summarizeLogText(log: RequestLog) {
  const source = log.error || log.responseBody || log.requestBody;
  const compact = source.replace(/\s+/g, " ").trim();
  return compact.length > 120 ? `${compact.slice(0, 120)}...` : compact || "暂无摘要";
}

function extractLogPrimaryOutput(log: RequestLog) {
  const source = log.streamSummary?.trim() || log.responseBody?.trim() || log.error?.trim() || "";
  if (!source) {
    return "";
  }

  const compact = source.replace(/\s+/g, " ").trim();
  return compact.length > 400 ? `${compact.slice(0, 400)}...` : compact;
}

function buildSessionOutput(logs: RequestLog[]) {
  const output = logs
    .map((log) => extractLogPrimaryOutput(log))
    .filter(Boolean)
    .join("\n\n");

  return output || "当前会话没有提取到完整输出内容。";
}

function maskApiKey(value: string) {
  if (!value.trim()) {
    return "未填写";
  }

  if (value.length <= 8) {
    return "已填写";
  }

  return `${value.slice(0, 4)}••••${value.slice(-4)}`;
}

function isFailedLog(log: RequestLog) {
  return log.responseStatus === 0 || log.responseStatus >= 400 || Boolean(log.error);
}

function normalizeFailureTitle(log: RequestLog) {
  const text = (log.error || log.responseBody || "").toLowerCase();

  if (log.responseStatus === 401 || text.includes("api key") || text.includes("unauthorized")) {
    return "鉴权失败";
  }
  if (log.responseStatus === 429 || text.includes("rate limit")) {
    return "请求限流";
  }
  if (log.responseStatus >= 500) {
    return "上游服务异常";
  }
  if (log.responseStatus === 0 || text.includes("failed to fetch") || text.includes("network")) {
    return "网络或跨域失败";
  }

  return "请求参数或协议不匹配";
}

function getFailureNextStep(log: RequestLog) {
  const title = normalizeFailureTitle(log);
  if (title === "鉴权失败") {
    return "检查平台 API Key、请求头类型和平台协议是否匹配。";
  }
  if (title === "请求限流") {
    return "降低发送频率，或切换到另一个平台继续调用。";
  }
  if (title === "上游服务异常") {
    return "稍后重试，并在时间线中确认是否为平台侧波动。";
  }
  if (title === "网络或跨域失败") {
    return "检查 base URL、网络连通性，以及是否需要通过本地网关转发。";
  }

  return "核对模型名、API 路径和请求协议，再对照原始请求体排查。";
}

watch(currentAnimationName, (name) => {
  if (animations[name].loop && !queuedAnimationName) {
    queueIdleAction();
  }
});

watch([isChatOpen, isConsoleOpen, isDragging, isSending], () => {
  applyBaseAnimation();
});

watch(chatInput, () => {
  applyBaseAnimation();
});

watch(draftAppLocale, (nextLocale) => {
  if (!isSystemSettingsOpen.value) {
    return;
  }
  setAppLocale(nextLocale, { persist: false });
});

watch(draftAppTheme, (nextTheme) => {
  if (!isSystemSettingsOpen.value) {
    return;
  }
  setAppTheme(nextTheme, { persist: false });
});

watch(
  () => contextMenu.value.visible,
  (visible) => {
    if (!visible) {
      return;
    }

    void nextTick(() => {
      adjustContextMenuToViewport();
    });
  }
);

watch(
  chatMessages,
  () => {
    persistChatHistory(activeChatAgentId.value);
  },
  { deep: true }
);

watch(
  roleWorkflowOverrides,
  () => {
    persistRoleWorkflowOverrides();
  },
  { deep: true }
);

watch(proxyPort, (value) => {
  safeLocalStorageSetItem("keai.desktop-pet.proxy-port", String(value));
});

watch(
  [platforms, proxyPort],
  () => {
    persistPlatformProxyEnabledStates(platforms.value);
    void syncLocalProxyServer();
  },
  { deep: true }
);

watch(
  activeSection,
  (section) => {
    if (panelPlacement.value.mode !== "manual") {
      return;
    }

    const bounds = stage.value?.getBoundingClientRect();
    if (!bounds) {
      return;
    }

    const prefersWide = true;
    const availableWidth = Math.max(320, bounds.width - 32);
    const availableHeight = Math.max(320, bounds.height - 32);
    const minWidth = Math.min(getPanelMinWidth(prefersWide), availableWidth);
    const minHeight = Math.min(420, availableHeight);
    panelPlacement.value = {
      ...panelPlacement.value,
      width: Math.min(Math.max(minWidth, panelPlacement.value.width), availableWidth),
      height: Math.min(Math.max(minHeight, panelPlacement.value.height || 0), availableHeight)
    };
  }
);

watch(
  [skillMarketCurrentTotalPages, () => skillMarketSearch.value],
  ([totalPages, keyword], [, prevKeyword = ""]) => {
    if (keyword !== prevKeyword) {
      skillMarketPage.value = 1;
      if (keyword.trim() !== "") {
        void refreshSkillMarket();
      } else if (prevKeyword.trim() !== "") {
        void refreshSkillMarket();
      }
      return;
    }
    if (skillMarketPage.value > totalPages) {
      skillMarketPage.value = totalPages;
    }
  }
);

watch(activeLogAnalysisView, (view) => {
  if (activePanelMode.value === "logs") {
    updateLogAnalysisStatus(view);
  }
});

watch(
  requestLogs,
  (logs) => {
    if (!selectedLogId.value && logs.length > 0) {
      selectedLogId.value = logs[0].id;
    }
    if (!selectedSessionId.value && sessionSummaries.value.length > 0) {
      selectedSessionId.value = sessionSummaries.value[0].id;
    }
    if (!selectedFailureKey.value && failureSummaries.value.length > 0) {
      selectedFailureKey.value = failureSummaries.value[0].key;
    }
  },
  { deep: true, immediate: true }
);

watch(
  activeMemorySelectionRecords,
  (records) => {
    if (activeResourceModal.value !== "memory") {
      return;
    }

    if (!records.length) {
      if (selectedMemoryId.value && !memoryRecords.value.some((record) => record.id === selectedMemoryId.value)) {
        resetMemoryDraft();
      }
      return;
    }

    if (!records.some((record) => record.id === selectedMemoryId.value)) {
      handleSelectMemory(records[0]);
    }
  },
  { immediate: true }
);

watch(
  activeDocumentSelectionRecords,
  (records) => {
    if (!activeResourceModal.value || activeResourceModal.value === "memory") {
      return;
    }

    if (!records.length) {
      if (selectedDocumentId.value && !documentRecords.value.some((record) => record.id === selectedDocumentId.value)) {
        resetDocumentDraft();
      }
      return;
    }

    if (!records.some((record) => record.id === selectedDocumentId.value)) {
      handleSelectDocument(records[0]);
    }
  },
  { immediate: true }
);

watch(
  () => selectedTimelineLog.value?.id ?? null,
  (logId) => {
    const log = selectedTimelineLog.value;
    if (!log) {
      return;
    }

    timelinePreviewSection.value = getDefaultPreviewSection(log);
  },
  { immediate: true }
);

watch(
  () => selectedSession.value?.id ?? null,
  (sessionId, prevSessionId) => {
    if (!sessionId) {
      sessionOverlayLogId.value = null;
      return;
    }

    const session = selectedSession.value;
    selectedSessionLogId.value = session?.logs[0]?.id ?? null;
    if (sessionId !== prevSessionId) {
      sessionOverlayLogId.value = null;
    }
  },
  { immediate: true }
);

watch(
  () => selectedSessionLog.value?.id ?? null,
  (logId) => {
    const log = selectedSessionLog.value;
    if (!log) {
      return;
    }

    sessionPreviewSection.value = getDefaultPreviewSection(log);
  },
  { immediate: true }
);

watch(
  activePlatform,
  (platform) => {
    if (activePlatformId.value && !platform) {
      activePlatformId.value = null;
      setActivePlatform(null);
    }
  },
  { immediate: true }
);

onMounted(async () => {
  if (isConsoleWindowMode) {
    activePanelMode.value = "console";
    activeSection.value = initialConsoleSection ?? "platforms";
    isConsoleOpen.value = true;
    panelMotionValue.value = 1;
    statusText.value = "代理配置窗口已独立打开。";
    void nextTick(() => {
      void revealConsoleWindowIfNeeded();
    });
  }

  const listen = getTauriApi()?.event?.listen;
  if (listen && isConsoleWindowMode) {
    try {
      const unlisten = await listen("clawpet://console-open", (event) => {
        const payload = event.payload as { section?: string } | null;
        const section = parseConsoleSection(payload?.section ?? null);
        if (!section) {
          return;
        }
        activePanelMode.value = "console";
        activeSection.value = section;
        isConsoleOpen.value = true;
        panelMotionValue.value = 1;
      });
      unlistenConsoleOpenEvent = unlisten;
    } catch {
      unlistenConsoleOpenEvent = null;
    }
  }

  chatMessages.value = loadChatHistory();
  currentSessionId.value = loadStoredSessionId();
  proxyPort.value = loadProxyPort();
  const openClawPlatforms = await loadPlatformsFromOpenClawConfig();
  platforms.value = openClawPlatforms ?? loadPlatforms();
  localRequestLogs.value = loadRequestLogs(platforms.value);
  void loadAvailableMonitors();
  void refreshOpenClawMessageLogs();
  void refreshStaffSnapshot();
  void refreshMemorySnapshot();
  void refreshDocumentSnapshot();
  void refreshOpenClawSkillSnapshot();
  void refreshOpenClawSkillsList();
  void refreshTaskSnapshot();
  if (activeSection.value === "channels") {
    void refreshMessageChannelSnapshot();
  }
  if (activeSection.value === "skill-market") {
    void refreshSkillMarket();
  }
  void refreshLobsterSnapshot();
  setAppTheme(appTheme.value, { persist: false });
  bindSystemThemeListener();
  void refreshLaunchOnLoginState();
  void applyAlwaysOnTop(isConsoleWindowMode ? false : petAlwaysOnTop.value);
  const storedActivePlatformId = loadActivePlatformId();
  const storedActivePlatform =
    platforms.value.find((platform) => platform.id === storedActivePlatformId) ?? null;
  const nextActivePlatformId =
    storedActivePlatformId && !isImplicitSeededOpenAiPlatform(storedActivePlatform) ? storedActivePlatformId : null;
  activePlatformId.value = nextActivePlatformId;
  setActivePlatform(nextActivePlatformId);
  void syncLocalProxyServer();
  void refreshGatewayMonitor();
  centerPet();
  lastInteractionAt = performance.now();
  animationStartedAt = performance.now();
  rafId = window.requestAnimationFrame(tick);
  applyBaseAnimation(true);
  void syncCursorPassThrough();
  cursorPassThroughTimer = window.setInterval(() => {
    void syncCursorPassThrough();
  }, 120);
  gatewayMonitorTimer = window.setInterval(() => {
    void refreshGatewayMonitor();
  }, 30000);
  runtimeLogTimer = window.setInterval(() => {
    void refreshOpenClawMessageLogs();
  }, runtimeLogPollIntervalMs);
  staffSnapshotPollTimer = window.setInterval(() => {
    if (!shouldAutoRefreshStaffSnapshot()) {
      return;
    }
    void refreshStaffSnapshot();
  }, staffSnapshotPollIntervalMs);
  windowPointerMoveListener = (event: PointerEvent) => {
    handlePointerMove(event);
  };
  windowPointerUpListener = (event: PointerEvent) => {
    finishDrag(event);
  };
  window.addEventListener("pointermove", windowPointerMoveListener);
  window.addEventListener("pointerup", windowPointerUpListener);
  window.addEventListener("pointercancel", windowPointerUpListener);
  window.addEventListener("pointermove", handleWindowChatPointerMove);
  window.addEventListener("pointerup", handleWindowChatPointerUp);
  window.addEventListener("pointercancel", handleWindowChatPointerUp);
  window.addEventListener("pointermove", handleWindowPanelPointerMove);
  window.addEventListener("pointerup", handleWindowPanelPointerUp);
  window.addEventListener("pointercancel", handleWindowPanelPointerUp);
  window.addEventListener("pointerdown", handleWindowPointerDown);
  window.addEventListener("keydown", handleEscape);
  window.addEventListener("focus", handleFocus);
  window.addEventListener("blur", handleBlur);
  window.addEventListener("contextmenu", handleContextMenu);
  document.addEventListener("visibilitychange", handleVisibilityChange);
  window.addEventListener("resize", handleResize);
});

onBeforeUnmount(() => {
  unbindSystemThemeListener();
  if (unlistenConsoleOpenEvent) {
    unlistenConsoleOpenEvent();
    unlistenConsoleOpenEvent = null;
  }
  stopVoicePlayback();
  window.cancelAnimationFrame(rafId);
  window.cancelAnimationFrame(chatAnimationFrame);
  window.cancelAnimationFrame(panelAnimationFrame);
  window.cancelAnimationFrame(bubbleAnimationFrame);
  window.clearTimeout(idleTimer);
  window.clearTimeout(runtimeLogFollowTimer);
  window.clearInterval(cursorPassThroughTimer);
  window.clearInterval(gatewayMonitorTimer);
  window.clearInterval(runtimeLogTimer);
  window.clearInterval(staffSnapshotPollTimer);
  window.clearInterval(lobsterInstallProgressTimer);
  if (windowPointerMoveListener) {
    window.removeEventListener("pointermove", windowPointerMoveListener);
  }
  if (windowPointerUpListener) {
    window.removeEventListener("pointerup", windowPointerUpListener);
    window.removeEventListener("pointercancel", windowPointerUpListener);
  }
  window.removeEventListener("pointermove", handleWindowChatPointerMove);
  window.removeEventListener("pointerup", handleWindowChatPointerUp);
  window.removeEventListener("pointercancel", handleWindowChatPointerUp);
  window.removeEventListener("pointermove", handleWindowPanelPointerMove);
  window.removeEventListener("pointerup", handleWindowPanelPointerUp);
  window.removeEventListener("pointercancel", handleWindowPanelPointerUp);
  window.removeEventListener("pointerdown", handleWindowPointerDown);
  window.removeEventListener("keydown", handleEscape);
  window.removeEventListener("focus", handleFocus);
  window.removeEventListener("blur", handleBlur);
  window.removeEventListener("contextmenu", handleContextMenu);
  document.removeEventListener("visibilitychange", handleVisibilityChange);
  window.removeEventListener("resize", handleResize);
});
</script>

<template>
  <main
    ref="stage"
    class="desktop-pet-stage"
    :class="{ 'desktop-pet-stage--console': isConsoleWindowMode, 'desktop-pet-stage--windows': isWindowsRuntime }"
  >
<div v-if="!isConsoleWindowMode && shouldShowHint" class="desktop-pet-hint" :style="hintStyle">
      <span class="desktop-pet-hint-title">{{ activeAnimation.label }}</span>
      <p>{{ statusText }}</p>
    </div>

    <button
      v-if="!isConsoleWindowMode"
      ref="pet"
      class="sprite-pet"
      :class="{ dragging: isDragging }"
      :style="petStyle"
      type="button"
      :aria-label="activeAnimation.label"
      @click="handlePetClick"
      @pointerdown="handlePointerDown"
    />

    <section
      v-if="!isConsoleWindowMode"
      v-show="isChatOpen || chatMotionValue > 0"
      ref="chatPanelRef"
      class="desktop-console-panel desktop-chat-window"
      :style="chatPanelStyle"
    >
      <header
        class="desktop-console-panel__dragbar chat-header"
        @pointerdown="handleChatDragStart"
      >
        <div class="chat-header__bar">
          <span class="chat-header__title">
            {{ chatHeaderTitle }}
          </span>
          <div class="chat-header__actions">
            <button
              class="chat-header__btn"
              type="button"
              aria-label="新建会话"
              title="新会话"
              @click="handleNewConversation"
            >
              <svg viewBox="0 0 20 20" aria-hidden="true"><path d="M10 4v12M4 10h12" /></svg>
            </button>
            <button
              class="chat-header__btn"
              type="button"
              aria-label="收起对话窗口"
              title="收起"
              @click="toggleChatPanel(false)"
            >
              <svg viewBox="0 0 20 20" aria-hidden="true"><path d="M5 10h10" /></svg>
            </button>
          </div>
        </div>
        <nav class="chat-tags" @pointerdown.stop>
          <button
            class="chat-tag"
            :class="{ 'chat-tag--active': activeChatAgentId === null }"
            type="button"
            @click="switchChatAgent(null)"
          >
            <svg class="chat-tag__icon" viewBox="0 0 20 20" aria-hidden="true">
              <path d="M3 5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H8l-4 3v-3a2 2 0 0 1-1-1.73V5Z" />
            </svg>
            主对话
          </button>
          <button
            v-for="member in staffMembers"
            :key="member.agentId"
            class="chat-tag"
            :class="{ 'chat-tag--active': activeChatAgentId === member.agentId }"
            type="button"
            :title="member.roleLabel"
            @click="switchChatAgent(member.agentId)"
          >
            <span class="chat-tag__dot" aria-hidden="true">{{ member.displayName.charAt(0) }}</span>
            {{ stripRoleLabel(member.displayName) }}
          </button>
          <button
            v-for="petPeer in boundPets"
            :key="petPeer.id"
            class="chat-tag"
            :class="{ 'chat-tag--active': activeChatAgentId === buildBoundPetChatAgentId(petPeer.id) }"
            type="button"
            :title="`绑定码 ${petPeer.bindingCode}`"
            @click="switchChatAgent(buildBoundPetChatAgentId(petPeer.id))"
          >
            <span class="chat-tag__dot" aria-hidden="true">{{ petPeer.petName.charAt(0) }}</span>
            {{ petPeer.petName }}
          </button>
        </nav>
      </header>

      <div class="desktop-console-body desktop-console-body--chat">
        <section class="assistant-column">
          <div ref="messageScrollerRef" class="desktop-chat-panel__messages">
            <article
              v-for="(message, index) in chatMessages"
              :key="message.id"
              class="chat-message"
              :class="[
                `chat-message--${message.role}`,
                `chat-message--${message.status}`,
                { 'chat-message--audio': isAudioMessage(message) }
              ]"
              :style="getBubbleStyle(index)"
            >
              <div
                v-if="message.role !== 'user' && message.role !== 'system'"
                class="chat-message__avatar"
                aria-hidden="true"
              >
                <svg viewBox="0 0 20 20">
                  <path d="M10 2.8l1.5 3.3 3.4 1.5-3.4 1.5L10 12.4 8.5 9.1 5 7.6l3.5-1.5z" />
                </svg>
              </div>
              <div class="chat-message__body">
                <div
                  class="chat-bubble"
                  :class="[
                    `chat-bubble--${message.role}`,
                    `chat-bubble--${message.status}`,
                    { 'chat-bubble--audio': isAudioMessage(message) }
                  ]"
                >
                  <button
                    v-if="isAudioMessage(message)"
                    class="voice-message"
                    :class="{ 'is-playing': isAudioMessagePlaying(message.id) }"
                    type="button"
                    @click="handleAudioMessageClick(message)"
                  >
                    <span
                      class="voice-message__icon"
                      :class="isAudioMessagePlaying(message.id) ? 'is-pause' : 'is-play'"
                      aria-hidden="true"
                    />
                    <span class="voice-message__body">
                      <strong>{{ getAudioMessageLabel(message) }}</strong>
                      <small>{{ isAudioMessagePlaying(message.id) ? "点击停止播放" : "点击播放语音" }}</small>
                    </span>
                  </button>
                  <template v-else>
                    <div v-if="message.attachments && message.attachments.length > 0" class="bubble-attachments">
                      <div v-for="att in message.attachments" :key="att.id" class="bubble-attachment">
                        <img v-if="isImageAttachment(att)" class="bubble-attachment__img" :src="att.dataUrl" :alt="att.name" />
                        <span v-else class="bubble-attachment__file">
                          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><path d="M14 2v6h6" /></svg>
                          {{ att.name }}
                        </span>
                      </div>
                    </div>
                    <p v-if="message.text && message.text !== '(附件)'">{{ message.text }}</p>
                  </template>
                </div>
                <div class="chat-message__meta">
                  <span class="chat-message__meta-role">{{ getMessageRoleLabel(message) }}</span>
                  <span v-if="getMessageTimeLabel(message)">{{ getMessageTimeLabel(message) }}</span>
                </div>
              </div>
            </article>
            <div v-if="chatMessages.length === 0" class="empty-state empty-state--small">
              当前会话还没有消息，发送第一句开始对话吧。
            </div>
          </div>

          <footer
            class="composer"
            :class="{ 'is-dragover': isDragOver }"
            @dragenter="handleComposerDragEnter"
            @dragover="handleComposerDragOver"
            @dragleave="handleComposerDragLeave"
            @drop="handleComposerDrop"
          >
            <div v-if="isDragOver" class="composer__drop-overlay">
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 16V4m0 0-4 4m4-4 4 4M4 18h16" /></svg>
              <span>松开以添加附件</span>
            </div>
            <template v-else>
              <div v-if="chatAttachments.length > 0" class="composer__attachments">
                <div
                  v-for="att in chatAttachments"
                  :key="att.id"
                  class="composer__chip"
                  :title="att.name + ' (' + formatFileSize(att.size) + ')'"
                >
                  <img
                    v-if="isImageAttachment(att)"
                    class="composer__chip-thumb"
                    :src="att.dataUrl"
                    :alt="att.name"
                  />
                  <svg v-else class="composer__chip-file" viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                    <path d="M14 2v6h6" />
                  </svg>
                  <span class="composer__chip-name">{{ att.name }}</span>
                  <button
                    class="composer__chip-remove"
                    type="button"
                    aria-label="移除附件"
                    @click="removeAttachment(att.id)"
                  >
                    <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg>
                  </button>
                </div>
              </div>
              <textarea
                v-model="chatInput"
                class="composer__input"
                rows="2"
                :placeholder="activeBoundPet ? '输入消息，或用 /agent <agentId> <任务> 调用已授权 Agent' : '输入你想让 OpenClaw 帮你做的事'"
                @keydown="handleComposerKeydown"
                @paste="handlePaste"
              />
              <div class="composer__toolbar">
                <button
                  class="composer__tool-btn"
                  type="button"
                  aria-label="添加附件"
                  title="添加附件"
                  @click="triggerFileInput"
                >
                  <svg viewBox="0 0 20 20" aria-hidden="true">
                    <path d="M15.5 10l-5.5 5.5a4.24 4.24 0 0 1-6-6L10 3.5a2.83 2.83 0 0 1 4 4l-6 6a1.41 1.41 0 0 1-2-2l5.5-5.5" />
                  </svg>
                </button>
                <button
                  class="composer__send-btn"
                  type="button"
                  aria-label="发送消息"
                  title="发送"
                  :disabled="isSending || (!chatInput.trim() && chatAttachments.length === 0)"
                  @click="submitChat"
                >
                  <svg viewBox="0 0 20 20" aria-hidden="true">
                    <path d="M5 10l8-5-2.5 5L13 15z" />
                  </svg>
                </button>
              </div>
              <div class="composer__hint">
                <span>{{ isSending ? "正在等待回复…" : "Enter 发送 · Shift + Enter 换行" }}</span>
                <span>
                  当前会话：
                  {{
                    activeBoundPet
                      ? activeBoundPet.petName
                      : (activeChatAgent ? stripRoleLabel(activeChatAgent.displayName) : "主对话")
                  }}
                </span>
              </div>
            </template>
            <input
              ref="fileInputRef"
              type="file"
              multiple
              class="composer__file-input"
              @change="handleFileInputChange"
            />
          </footer>
        </section>
      </div>

      <div class="desktop-console-panel__resize-handle" @pointerdown="handleChatResizeStart" />
    </section>

    <section
      v-show="isConsoleOpen || panelMotionValue > 0"
      ref="consolePanelRef"
      class="desktop-console-panel"
      :style="consolePanelStyle"
      @pointerdown="handleConsolePanelPointerDown"
    >
      <header class="desktop-console-panel__header desktop-console-panel__dragbar" @pointerdown="handleConsoleHeaderPointerDown">
        <div v-if="activePanelMode === 'console'">
          <p class="desktop-console-panel__eyebrow">ClawPet Command Deck</p>
          <strong>运营控制台</strong>
          <p class="desktop-console-panel__intro">
            当前默认平台
            <span class="desktop-console-panel__platform">{{ activePlatform?.name ?? openClawDefaultPlatformName }}</span>
            ，这里已经扩展为平台、员工与任务统一管理台。
          </p>
        </div>
        <div v-else-if="activePanelMode === 'logs'">
          <p class="desktop-console-panel__eyebrow">ClawPet Command Deck</p>
          <strong>日志分析</strong>
          <p class="desktop-console-panel__intro">集中查看时间线、会话视图和失败分析，不再混入控制台导航。</p>
        </div>
        <div v-else-if="activePanelMode === 'lobster'">
          <p class="desktop-console-panel__eyebrow">ClawPet Command Deck</p>
          <strong>龙虾配置</strong>
          <p class="desktop-console-panel__intro">包含龙虾安装、网关重启、自动修复、备份恢复与版本升级。</p>
        </div>
        <div v-else>
          <p class="desktop-console-panel__eyebrow">ClawPet Command Deck</p>
          <strong>订阅推荐</strong>
        </div>
        <div class="desktop-console-panel__actions">
          <button
            v-if="activePanelMode === 'console'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openLobsterConfig()"
          >
            龙虾配置
          </button>
          <button
            v-if="activePanelMode === 'console'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openLogAnalysis()"
          >
            日志分析
          </button>
          <button
            v-if="activePanelMode === 'console'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openSubscriptionRecommendations()"
          >
            订阅推荐
          </button>
          <button
            v-if="activePanelMode === 'console'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openSystemSettings()"
          >
            系统设置
          </button>
          <button
            v-if="activePanelMode !== 'console'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openConsole('platforms')"
          >
            代理配置
          </button>
          <button
            v-if="activePanelMode === 'subscriptions' || activePanelMode === 'lobster'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openLobsterConfig()"
          >
            龙虾配置
          </button>
          <button
            v-if="activePanelMode === 'subscriptions' || activePanelMode === 'lobster'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openLogAnalysis()"
          >
            日志分析
          </button>
          <button class="desktop-console-panel__action" type="button" @pointerdown.stop @click.stop.prevent="closeConsoleWindow">关闭</button>
        </div>
      </header>

      <nav v-if="activePanelMode === 'console'" class="desktop-console-nav">
        <button
          v-for="item in consoleSections"
          :key="item.id"
          class="desktop-console-nav__item"
          :class="{ active: activeSection === item.id }"
          type="button"
          @click="openConsole(item.id)"
        >
          {{ item.label }}
        </button>
      </nav>

      <nav v-else-if="activePanelMode === 'logs'" class="desktop-console-nav">
        <button
          class="desktop-console-nav__item"
          :class="{ active: activeLogAnalysisView === 'timeline' }"
          type="button"
          @click="activeLogAnalysisView = 'timeline'"
        >
          时间线
        </button>
        <button
          class="desktop-console-nav__item"
          :class="{ active: activeLogAnalysisView === 'sessions' }"
          type="button"
          @click="activeLogAnalysisView = 'sessions'"
        >
          会话视图
        </button>
        <button
          class="desktop-console-nav__item"
          :class="{ active: activeLogAnalysisView === 'failures' }"
          type="button"
          @click="activeLogAnalysisView = 'failures'"
        >
          失败分析
        </button>
      </nav>

      <div v-if="activePanelMode === 'logs'" class="log-filter-bar">
        <select v-model="logFilterPlatform" class="log-filter-select" aria-label="按平台筛选日志">
          <option :value="null">全部平台</option>
          <option v-for="name in availableLogPlatformGroups" :key="name" :value="name">{{ name }}</option>
        </select>
        <select v-model="logFilterAgent" class="log-filter-select" aria-label="按代理筛选日志">
          <option :value="null">全部代理</option>
          <option v-for="agent in availableLogAgentOptions" :key="agent.id" :value="agent.id">{{ agent.name }}</option>
        </select>
        <button
          v-if="hasActiveLogFilter"
          class="log-filter-clear"
          type="button"
          @click="clearLogFilters"
        >
          清除筛选
        </button>
      </div>

      <div v-if="activePanelMode === 'logs'" class="desktop-console-body desktop-console-body--split">
        <template v-if="activeLogAnalysisView === 'timeline'">
        <section class="section-block">
          <header class="section-block__header">
            <div>
              <h3>调用时间线</h3>
              <p>按时间倒序查看所有请求，便于回放最近发生了什么。</p>
            </div>
            <div class="toolbar-actions">
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleExportLogs">
                导出
              </button>
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleClearLogs">
                清空
              </button>
            </div>
          </header>

          <div class="log-list">
            <button
              v-for="log in timelineEntries"
              :key="log.id"
              class="log-card"
              :class="[
                { active: selectedTimelineLog?.id === log.id },
                getPlatformCardAccentClass(log.platformId, 'log-card')
              ]"
              type="button"
              @click="selectedLogId = log.id"
            >
              <div class="log-card__headline">
                <span class="platform-compact" :class="getPlatformIdentityToneClass(log.platformId, log.platformName)">
                  <span class="platform-compact__dot"></span>
                  <span class="platform-compact__name">{{ log.platformName }}</span>
                  <span class="platform-compact__meta">{{ getPlatformMetaLabel(log.platformId, log.protocol) }}</span>
                </span>
                <span>{{ formatTime(log.createdAt) }}</span>
              </div>
              <p>{{ summarizeLogText(log) }}</p>
              <small :data-status="isFailedLog(log) ? 'error' : 'success'">
                {{ log.method }} · {{ isFailedLog(log) ? "失败" : "成功" }} · {{ formatDuration(log.duration) }}
              </small>
            </button>
            <div v-if="timelineEntries.length === 0" class="empty-state">
              {{ hasActiveLogFilter ? "当前筛选条件下没有匹配的调用记录。" : "还没有调用记录，先去和桌宠聊两句吧。" }}
            </div>
          </div>
        </section>

        <aside class="section-block detail-panel">
          <header class="section-block__header">
            <div>
              <h3>请求详情</h3>
              <p>选中左侧条目后，可直接查看状态卡片、请求头和完整内容分段。</p>
            </div>
          </header>

          <template v-if="selectedTimelineLog">
            <div class="detail-stat-grid">
              <article class="detail-stat-card detail-stat-card--primary">
                <span>状态码</span>
                <strong>{{ selectedTimelineLog.responseStatus || "未返回" }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>耗时</span>
                <strong>{{ formatLatencyStat(selectedTimelineLog.duration) }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>TTFT</span>
                <strong>{{ formatLatencyStat(selectedTimelineLog.firstTokenTime) }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>输出速度</span>
                <strong>{{ formatSpeed(selectedTimelineLog.tokensPerSecond) }}</strong>
              </article>
            </div>

            <div class="detail-summary-card">
              <div class="detail-summary-grid">
                <div>
                  <span>平台</span>
                  <div class="platform-pill" :class="getPlatformPillClass(selectedTimelineLog.platformId)">
                    <span class="platform-pill__name">{{ selectedTimelineLog.platformName }}</span>
                    <span class="platform-pill__meta">
                      {{ getPlatformMetaLabel(selectedTimelineLog.platformId, selectedTimelineLog.protocol) }}
                    </span>
                  </div>
                </div>
                <div>
                  <span>时间</span>
                  <strong>{{ formatTime(selectedTimelineLog.createdAt) }}</strong>
                </div>
                <div>
                  <span>输入/输出 Token</span>
                  <strong>{{ formatTokenPair(selectedTimelineLog) }}</strong>
                </div>
                <div>
                  <span>缓存读取 Token</span>
                  <strong>{{ formatCacheTokens(selectedTimelineLog) }}</strong>
                </div>
              </div>
              <div class="detail-copy-row">
                <div class="detail-endpoint">{{ getLogRequestUrl(selectedTimelineLog) }}</div>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(getLogRequestUrl(selectedTimelineLog), '已复制请求地址。')"
                >
                  复制
                </button>
              </div>
            </div>

            <div v-if="selectedTimelineLog.requestHeaders && Object.keys(selectedTimelineLog.requestHeaders).length > 0" class="detail-code">
              <div class="detail-code__header">
                <h4>请求头</h4>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(JSON.stringify(maskSensitiveHeaders(selectedTimelineLog.requestHeaders), null, 2), '已复制请求头。')"
                >
                  复制
                </button>
              </div>
              <pre>{{ JSON.stringify(maskSensitiveHeaders(selectedTimelineLog.requestHeaders), null, 2) }}</pre>
            </div>

            <div class="detail-tab-row">
              <button
                v-for="section in buildPreviewSections(selectedTimelineLog)"
                :key="section.id"
                class="detail-tab"
                :class="{ active: getActivePreviewSection(selectedTimelineLog, timelinePreviewSection).id === section.id }"
                type="button"
                @click="timelinePreviewSection = section.id"
              >
                {{ section.label }}
              </button>
            </div>

            <div class="detail-code">
              <div class="detail-code__header">
                <h4>{{ getActivePreviewSection(selectedTimelineLog, timelinePreviewSection).label }}</h4>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(getActivePreviewSection(selectedTimelineLog, timelinePreviewSection).view.text, '已复制请求内容。')"
                >
                  复制
                </button>
              </div>
              <pre>{{ getActivePreviewSection(selectedTimelineLog, timelinePreviewSection).view.text }}</pre>
            </div>
          </template>
          <div v-else class="empty-state">暂无详情</div>
        </aside>
        </template>

        <template v-else-if="activeLogAnalysisView === 'sessions'">
        <section class="section-block">
          <header class="section-block__header">
            <div>
              <h3>会话视图</h3>
              <p>把一段连续请求聚合成会话，适合回看完整上下文。</p>
            </div>
          </header>

          <div class="session-list">
            <button
              v-for="session in sessionSummaries"
              :key="session.id"
              class="session-card"
              :class="[
                { active: selectedSession?.id === session.id },
                getPlatformCardAccentClass(session.logs[0]?.platformId ?? null, 'session-card')
              ]"
              type="button"
              @click="selectedSessionId = session.id"
            >
              <div class="session-card__headline">
                <span class="platform-compact" :class="getPlatformIdentityToneClass(session.logs[0]?.platformId ?? null, session.platformName)">
                  <span class="platform-compact__dot"></span>
                  <span class="platform-compact__name">{{ session.platformName }}</span>
                  <span class="platform-compact__meta">{{ getPlatformMetaLabel(session.logs[0]?.platformId ?? null, session.logs[0]?.protocol) }}</span>
                </span>
                <span>{{ formatTime(session.lastAt) }}</span>
              </div>
              <p>{{ session.previewText }}</p>
              <small>
                {{ session.requestCount }} 次调用 · {{ session.failureCount }} 次失败 · {{ formatDuration(session.totalDuration) }}
              </small>
            </button>
            <div v-if="sessionSummaries.length === 0" class="empty-state">
              {{ hasActiveLogFilter ? "当前筛选条件下没有匹配的会话记录。" : "还没有形成会话记录。" }}
            </div>
          </div>
        </section>

        <aside class="section-block detail-panel">
          <header class="section-block__header">
            <div>
              <h3>会话详情</h3>
              <p>先看会话统计，再继续查看会话内某一次请求的完整信息。</p>
            </div>
          </header>

          <template v-if="selectedSession && selectedSessionLog">
            <div class="detail-stat-grid">
              <article class="detail-stat-card detail-stat-card--primary">
                <span>请求数</span>
                <strong>{{ selectedSession.requestCount }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>失败数</span>
                <strong>{{ selectedSession.failureCount }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>总耗时</span>
                <strong>{{ formatLatencyStat(selectedSession.totalDuration) }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>总 Token</span>
                <strong>{{ selectedSession.totalTokens }}</strong>
              </article>
            </div>

            <div class="detail-summary-card">
              <div class="detail-summary-grid">
                <div>
                  <span>平台</span>
                  <strong>{{ selectedSession.platformName }}</strong>
                </div>
                <div>
                  <span>开始时间</span>
                  <strong>{{ formatTime(selectedSession.startedAt) }}</strong>
                </div>
                <div>
                  <span>最近时间</span>
                  <strong>{{ formatTime(selectedSession.lastAt) }}</strong>
                </div>
                <div>
                  <span>输入/输出 Token</span>
                  <strong>{{ selectedSession.promptTokens }}/{{ selectedSession.completionTokens }}</strong>
                </div>
              </div>
              <div class="detail-copy-row">
                <div class="detail-endpoint detail-endpoint--soft">{{ selectedSession.previewText }}</div>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(selectedSession.previewText, '已复制会话摘要。')"
                >
                  复制
                </button>
              </div>
            </div>

            <div class="detail-code session-output-card">
              <div class="detail-code__header">
                <h4>会话输出</h4>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(selectedSession.fullOutput, '已复制会话输出。')"
                >
                  复制
                </button>
              </div>
              <pre>{{ selectedSession.fullOutput }}</pre>
            </div>

            <div v-if="selectedSession.latestError" class="detail-code detail-code--danger">
              <div class="detail-code__header">
                <h4>最近一次失败</h4>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(selectedSession.latestError, '已复制失败信息。')"
                >
                  复制
                </button>
              </div>
              <pre>{{ selectedSession.latestError }}</pre>
            </div>

            <div class="session-timeline-section">
              <div class="section-block__header section-block__header--compact">
                <div>
                  <h3>会话时间线</h3>
                  <p>按时间回看本次会话里的每一次请求，点击任意条目查看它的完整详情。</p>
                </div>
              </div>

              <div class="mini-log-list">
              <button
                v-for="log in selectedSession.logs"
                :key="log.id"
                class="mini-log-card"
                :class="[
                  { active: selectedSessionLog?.id === log.id },
                  getPlatformCardAccentClass(log.platformId, 'mini-log-card')
                ]"
                type="button"
                @click="handleOpenSessionLog(log)"
              >
                <div>
                  <strong>{{ log.method }} {{ log.path || log.endpoint }}</strong>
                  <span>{{ formatTime(log.createdAt) }}</span>
                </div>
                <p>{{ summarizeLogText(log) }}</p>
                <small :data-status="isFailedLog(log) ? 'error' : 'success'">
                  {{ log.responseStatus || "未返回" }} · {{ formatLatencyStat(log.duration) }} ·
                  {{ sessionOverlayLog?.id === log.id ? "详情已打开" : "查看详情" }}
                </small>
              </button>
            </div>
            </div>

            <div class="section-block__header section-block__header--compact">
              <div>
                <h3>当前请求详情</h3>
                <p>正在查看 {{ selectedSessionLog.method }} {{ selectedSessionLog.path || selectedSessionLog.endpoint }}</p>
              </div>
            </div>

            <div class="detail-stat-grid">
              <article class="detail-stat-card detail-stat-card--primary">
                <span>状态码</span>
                <strong>{{ selectedSessionLog.responseStatus || "未返回" }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>耗时</span>
                <strong>{{ formatLatencyStat(selectedSessionLog.duration) }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>TTFT</span>
                <strong>{{ formatLatencyStat(selectedSessionLog.firstTokenTime) }}</strong>
              </article>
              <article class="detail-stat-card">
                <span>输出速度</span>
                <strong>{{ formatSpeed(selectedSessionLog.tokensPerSecond) }}</strong>
              </article>
            </div>

            <div class="detail-summary-card">
              <div class="detail-summary-grid">
                <div>
                  <span>平台</span>
                  <strong>{{ selectedSessionLog.platformName }}</strong>
                </div>
                <div>
                  <span>时间</span>
                  <strong>{{ formatTime(selectedSessionLog.createdAt) }}</strong>
                </div>
                <div>
                  <span>输入/输出 Token</span>
                  <strong>{{ formatTokenPair(selectedSessionLog) }}</strong>
                </div>
                <div>
                  <span>缓存读取 Token</span>
                  <strong>{{ formatCacheTokens(selectedSessionLog) }}</strong>
                </div>
              </div>
              <div class="detail-copy-row">
                <div class="detail-endpoint">{{ getLogRequestUrl(selectedSessionLog) }}</div>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(getLogRequestUrl(selectedSessionLog), '已复制请求地址。')"
                >
                  复制
                </button>
              </div>
            </div>

            <div v-if="selectedSessionLog.requestHeaders && Object.keys(selectedSessionLog.requestHeaders).length > 0" class="detail-code">
              <div class="detail-code__header">
                <h4>请求头</h4>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(JSON.stringify(maskSensitiveHeaders(selectedSessionLog.requestHeaders), null, 2), '已复制请求头。')"
                >
                  复制
                </button>
              </div>
              <pre>{{ JSON.stringify(maskSensitiveHeaders(selectedSessionLog.requestHeaders), null, 2) }}</pre>
            </div>

            <div class="detail-tab-row">
              <button
                v-for="section in buildPreviewSections(selectedSessionLog)"
                :key="section.id"
                class="detail-tab"
                :class="{ active: getActivePreviewSection(selectedSessionLog, sessionPreviewSection).id === section.id }"
                type="button"
                @click="sessionPreviewSection = section.id"
              >
                {{ section.label }}
              </button>
            </div>

            <div class="detail-code">
              <div class="detail-code__header">
                <h4>{{ getActivePreviewSection(selectedSessionLog, sessionPreviewSection).label }}</h4>
                <span>{{ getLogRequestUrl(selectedSessionLog) }}</span>
                <button
                  class="detail-copy-button"
                  type="button"
                  @click="copyText(getActivePreviewSection(selectedSessionLog, sessionPreviewSection).view.text, '已复制请求内容。')"
                >
                  复制
                </button>
              </div>
              <pre>{{ getActivePreviewSection(selectedSessionLog, sessionPreviewSection).view.text }}</pre>
            </div>
          </template>
          <div v-else class="empty-state">暂无会话详情</div>
        </aside>
        </template>

        <template v-else>
        <section class="section-block">
          <header class="section-block__header">
            <div>
              <h3>失败分析</h3>
              <p>自动把错误按原因聚合，优先看最频繁、最近发生的问题。</p>
            </div>
          </header>

          <div class="failure-list">
            <button
              v-for="failure in failureSummaries"
              :key="failure.key"
              class="failure-card"
              :class="{ active: selectedFailure?.key === failure.key }"
              type="button"
              @click="selectedFailureKey = failure.key"
            >
              <div class="failure-card__headline">
                <strong>{{ failure.title }}</strong>
                <span>{{ failure.count }} 次</span>
              </div>
              <p>{{ failure.platformNames.join(' / ') }}</p>
              <small>{{ formatTime(failure.latestAt) }}</small>
            </button>
            <div v-if="failureSummaries.length === 0" class="empty-state">目前没有失败请求，状态很好。</div>
          </div>
        </section>

        <aside class="section-block detail-panel">
          <header class="section-block__header">
            <div>
              <h3>处置建议</h3>
              <p>根据失败类型给出下一步排查建议。</p>
            </div>
          </header>

          <template v-if="selectedFailure && selectedFailureLog">
            <div class="detail-meta">
              <span>{{ selectedFailure.title }}</span>
              <span>{{ selectedFailure.platformNames.join(' / ') }}</span>
            </div>
            <div class="detail-code">
              <h4>建议</h4>
              <pre>{{ selectedFailure.nextStep }}</pre>
            </div>
            <div class="detail-code">
              <h4>样例错误</h4>
              <pre>{{ selectedFailureLog.error || selectedFailureLog.responseBody || "暂无错误详情" }}</pre>
            </div>
          </template>
          <div v-else class="empty-state">暂无失败详情</div>
        </aside>
        </template>
      </div>

      <div v-else-if="activePanelMode === 'lobster'" class="desktop-console-body desktop-console-body--overview lobster-console">
        <section class="section-block overview-section lobster-operations">
          <div class="lobster-toolbar">
            <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshLobsterSnapshot()">
              刷新状态
            </button>
          </div>

          <div class="lobster-action-grid">
            <article v-for="item in lobsterActionCards" :key="item.id" class="lobster-action-card">
              <div class="lobster-action-card__header">
                <strong>{{ item.title }}</strong>
                <p>{{ item.description }}</p>
              </div>

              <label v-if="item.id === 'restore'" class="lobster-restore-picker">
                <span>恢复目标备份</span>
                <select v-model="selectedLobsterBackupPath">
                  <option :value="null">自动选择最近备份</option>
                  <option v-for="backup in lobsterSnapshot?.backups ?? []" :key="backup.path" :value="backup.path">
                    {{ backup.name }} · {{ formatTime(backup.createdAtMs) }} · {{ formatFileSize(backup.sizeBytes) }}
                  </option>
                </select>
              </label>
              <p v-if="item.id === 'restore' && !(lobsterSnapshot?.backups?.length)" class="lobster-action-card__hint">
                暂无可恢复备份，请先执行“龙虾备份”。
              </p>

              <button
                class="desktop-console-panel__action"
                :class="{ 'desktop-console-panel__action--danger': item.danger }"
                type="button"
                :disabled="lobsterActionRunning !== null || (item.id === 'restore' && !(lobsterSnapshot?.backups?.length))"
                @click="runLobsterAction(item.id)"
              >
                {{ lobsterActionRunning === item.id ? "执行中..." : item.buttonLabel }}
              </button>

              <p
                v-if="lobsterActionResult && lobsterActionResult.action === item.id"
                class="lobster-action-card__result"
                :class="{ 'is-success': lobsterActionResult.success, 'is-failed': !lobsterActionResult.success }"
              >
                <span>{{ lobsterActionResult.success ? "最近执行成功" : "最近执行失败" }}</span>
                <small>耗时 {{ formatDuration(lobsterActionResult.durationMs) }}</small>
                <em>{{ lobsterActionResult.detail }}</em>
              </p>
            </article>
          </div>
        </section>
      </div>

      <div v-else-if="activePanelMode === 'subscriptions'" class="desktop-console-body desktop-console-body--overview">
        <section class="section-block overview-section">
          <div class="subscription-insight-card">
            <div class="subscription-insight-card__header">
              <div>
                <strong>订阅推荐更新</strong>
                <p>根据你提供的实测截图整理，更新时间 {{ subscriptionDataUpdatedAt }}。</p>
              </div>
              <button
                class="desktop-console-panel__action desktop-console-panel__action--ghost"
                type="button"
                @click="openCodingPlanPlatform(subscriptionReferenceUrl)"
              >
                查看参考原文
              </button>
            </div>
            <p class="subscription-insight-card__hint">
              维度包含：首响速度、总输出速度、Token 消耗、三类题型准确性和综合稳定性。
            </p>
          </div>

          <div class="subscription-table-card">
            <div class="subscription-table-card__header">
              <strong>按场景推荐</strong>
              <span>5 个高频使用场景</span>
            </div>
            <div v-if="subscriptionScenarioRecommendations.length > 0" class="subscription-table-scroll">
              <table class="subscription-table">
                <thead>
                  <tr>
                    <th>使用场景</th>
                    <th>首选推荐</th>
                    <th>次选</th>
                    <th>避坑提醒</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="item in subscriptionScenarioRecommendations" :key="item.scene">
                    <td>{{ item.scene }}</td>
                    <td class="subscription-table__recommend">{{ item.primary }}</td>
                    <td>{{ item.secondary }}</td>
                    <td class="subscription-table__warning">{{ item.caution }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-else class="empty-state">暂无场景推荐数据。</div>
          </div>

          <div class="subscription-table-card">
            <div class="subscription-table-card__header">
              <strong>平台实测对比</strong>
              <span>6 个平台横向评估</span>
            </div>
            <div v-if="subscriptionPlatformBenchmarks.length > 0" class="subscription-table-scroll">
              <table class="subscription-table subscription-table--benchmark">
                <colgroup>
                  <col class="subscription-col-platform" />
                  <col class="subscription-col-first" />
                  <col class="subscription-col-output" />
                  <col class="subscription-col-token" />
                  <col class="subscription-col-status-simple" />
                  <col class="subscription-col-status-logic" />
                  <col class="subscription-col-status-spatial" />
                  <col class="subscription-col-stability" />
                  <col class="subscription-col-note" />
                </colgroup>
                <thead>
                  <tr>
                    <th class="subscription-table__sticky-col">平台</th>
                    <th>首响速度</th>
                    <th>输出速度（总耗时）</th>
                    <th>Token 消耗</th>
                    <th>简单问答</th>
                    <th>逻辑题</th>
                    <th>空间题</th>
                    <th>综合稳定性</th>
                    <th>关键备注</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="item in subscriptionPlatformBenchmarks" :key="item.platform">
                    <td class="subscription-table__sticky-col">
                      <button
                        v-if="subscriptionPlatformUrlMap[item.platform]"
                        class="subscription-platform-link"
                        type="button"
                        @click="openSubscriptionPlatformByName(item.platform)"
                      >
                        {{ item.platform }}
                      </button>
                      <span v-else>{{ item.platform }}</span>
                    </td>
                    <td class="subscription-table__metric">
                      <template v-if="getSubscriptionStarCount(item.firstResponse) > 0">
                        <span class="subscription-rating">
                          <span class="subscription-rating__stars">
                            <span
                              v-for="index in getSubscriptionStarCount(item.firstResponse)"
                              :key="`first-${item.platform}-${index}`"
                              class="subscription-rating__star"
                            >
                              ★
                            </span>
                          </span>
                          <span v-if="getSubscriptionMetricNote(item.firstResponse)" class="subscription-rating__note">{{ getSubscriptionMetricNote(item.firstResponse) }}</span>
                        </span>
                      </template>
                      <span v-else class="subscription-rating__text" :class="{ 'subscription-rating__text--bad': isSubscriptionMetricFailure(item.firstResponse) }">{{ item.firstResponse }}</span>
                    </td>
                    <td class="subscription-table__metric">
                      <template v-if="getSubscriptionStarCount(item.outputSpeed) > 0">
                        <span class="subscription-rating">
                          <span class="subscription-rating__stars">
                            <span
                              v-for="index in getSubscriptionStarCount(item.outputSpeed)"
                              :key="`output-${item.platform}-${index}`"
                              class="subscription-rating__star"
                            >
                              ★
                            </span>
                          </span>
                          <span v-if="getSubscriptionMetricNote(item.outputSpeed)" class="subscription-rating__note">{{ getSubscriptionMetricNote(item.outputSpeed) }}</span>
                        </span>
                      </template>
                      <span v-else class="subscription-rating__text" :class="{ 'subscription-rating__text--bad': isSubscriptionMetricFailure(item.outputSpeed) }">{{ item.outputSpeed }}</span>
                    </td>
                    <td class="subscription-table__metric">
                      <template v-if="getSubscriptionStarCount(item.tokenUsage) > 0">
                        <span class="subscription-rating">
                          <span class="subscription-rating__stars">
                            <span
                              v-for="index in getSubscriptionStarCount(item.tokenUsage)"
                              :key="`token-${item.platform}-${index}`"
                              class="subscription-rating__star"
                            >
                              ★
                            </span>
                          </span>
                          <span v-if="getSubscriptionMetricNote(item.tokenUsage)" class="subscription-rating__note">{{ getSubscriptionMetricNote(item.tokenUsage) }}</span>
                        </span>
                      </template>
                      <span v-else class="subscription-rating__text" :class="{ 'subscription-rating__text--bad': isSubscriptionMetricFailure(item.tokenUsage) }">{{ item.tokenUsage }}</span>
                    </td>
                    <td>
                      <span class="subscription-status" :class="`subscription-status--${getSubscriptionStatusTone(item.simpleAccuracy)}`">
                        <span class="subscription-status__icon">{{ getSubscriptionStatusIcon(item.simpleAccuracy) }}</span>
                        <span>{{ getSubscriptionStatusLabel(item.simpleAccuracy) }}</span>
                      </span>
                    </td>
                    <td>
                      <span class="subscription-status" :class="`subscription-status--${getSubscriptionStatusTone(item.logicAccuracy)}`">
                        <span class="subscription-status__icon">{{ getSubscriptionStatusIcon(item.logicAccuracy) }}</span>
                        <span>{{ getSubscriptionStatusLabel(item.logicAccuracy) }}</span>
                      </span>
                    </td>
                    <td>
                      <span
                        class="subscription-status subscription-status--spatial"
                        :class="`subscription-status--${getSubscriptionStatusTone(item.spatialAccuracy)}`"
                      >
                        <span class="subscription-status__icon">{{ getSubscriptionStatusIcon(item.spatialAccuracy) }}</span>
                        <span>{{ item.spatialAccuracy }}</span>
                      </span>
                    </td>
                    <td class="subscription-table__metric">
                      <span class="subscription-rating">
                        <span class="subscription-rating__stars">
                          <span
                            v-for="index in getSubscriptionStarCount(item.stability)"
                            :key="`stable-${item.platform}-${index}`"
                            class="subscription-rating__star"
                          >
                            ★
                          </span>
                        </span>
                      </span>
                    </td>
                    <td class="subscription-table__note">{{ item.note }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-else class="empty-state">暂无平台对比数据。</div>
          </div>

          <div class="coding-plan-section">
            <div class="coding-plan-section__header">
              <strong>云厂商平台入口</strong>
              <span class="coding-plan-section__count">{{ cloudCodingPlans.length }} 项</span>
            </div>

            <div v-if="cloudCodingPlans.length > 0" class="coding-plan-grid">
              <article
                v-for="plan in cloudCodingPlans"
                :key="plan.id"
                class="coding-plan-card"
                :class="`coding-plan-card--${plan.accent}`"
                role="button"
                tabindex="0"
                :aria-label="`打开 ${plan.name}`"
                @click="openCodingPlanPlatform(plan.platformUrl)"
                @keydown.enter.prevent="openCodingPlanPlatform(plan.platformUrl)"
                @keydown.space.prevent="openCodingPlanPlatform(plan.platformUrl)"
              >
                <div class="coding-plan-card__topline">
                  <div class="coding-plan-card__title">
                    <strong>{{ plan.name }}</strong>
                  </div>
                  <span class="coding-plan-card__badge">云平台</span>
                </div>

                <p class="coding-plan-card__summary">{{ plan.summary }}</p>

                <div class="coding-plan-card__spotlight">
                  <span>最新动态</span>
                  <p>{{ plan.latest }}</p>
                </div>

                <div class="preset-tags coding-plan-card__tags">
                  <span v-for="highlight in plan.highlights" :key="highlight">{{ highlight }}</span>
                </div>

                <div class="coding-plan-card__price">
                  <span>价格</span>
                  <strong>{{ plan.pricing }}</strong>
                  <small v-if="plan.pricingNote">{{ plan.pricingNote }}</small>
                </div>
              </article>
            </div>
            <div v-else class="empty-state">暂无可展示的云厂商平台。</div>
          </div>

          <div class="coding-plan-section">
            <div class="coding-plan-section__header">
              <strong>模型厂商平台入口</strong>
              <span class="coding-plan-section__count">{{ modelCodingPlans.length }} 项</span>
            </div>

            <div v-if="modelCodingPlans.length > 0" class="coding-plan-grid">
              <article
                v-for="plan in modelCodingPlans"
                :key="plan.id"
                class="coding-plan-card"
                :class="`coding-plan-card--${plan.accent}`"
                role="button"
                tabindex="0"
                :aria-label="`打开 ${plan.name}`"
                @click="openCodingPlanPlatform(plan.platformUrl)"
                @keydown.enter.prevent="openCodingPlanPlatform(plan.platformUrl)"
                @keydown.space.prevent="openCodingPlanPlatform(plan.platformUrl)"
              >
                <div class="coding-plan-card__topline">
                  <div class="coding-plan-card__title">
                    <strong>{{ plan.name }}</strong>
                  </div>
                  <span class="coding-plan-card__badge coding-plan-card__badge--model">模型平台</span>
                </div>

                <p class="coding-plan-card__summary">{{ plan.summary }}</p>

                <div class="coding-plan-card__spotlight">
                  <span>最新动态</span>
                  <p>{{ plan.latest }}</p>
                </div>

                <div class="preset-tags coding-plan-card__tags">
                  <span v-for="highlight in plan.highlights" :key="highlight">{{ highlight }}</span>
                </div>

                <div class="coding-plan-card__price">
                  <span>价格</span>
                  <strong>{{ plan.pricing }}</strong>
                  <small v-if="plan.pricingNote">{{ plan.pricingNote }}</small>
                </div>
              </article>
            </div>
            <div v-else class="empty-state">暂无可展示的模型厂商平台。</div>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'overview'" class="desktop-console-body desktop-console-body--overview">
        <section class="section-block overview-section">
          <header class="section-block__header">
            <div>
              <h3>统计信息</h3>
              <p>保留原控制台核心指标，集中展示平台、调用、网关和 Token 情况。</p>
            </div>
          </header>

          <div class="desktop-console-panel__metrics overview-metrics-grid">
            <article v-for="metric in metrics" :key="metric.label" class="desktop-metric-card">
              <span>{{ metric.label }}</span>
              <strong>{{ metric.value }}</strong>
            </article>
          </div>
        </section>

        <section class="section-block overview-section">
          <header class="section-block__header">
            <div>
              <h3>当前状态</h3>
              <p>聚焦今天最常看的运行状态，避免在多个页签之间来回切换。</p>
            </div>
          </header>

            <div class="overview-status-grid">
              <article v-for="card in overviewStatusCards" :key="card.label" class="overview-status-card">
                <span>{{ card.label }}</span>
                <strong>{{ card.value }}</strong>
                <p>{{ card.description }}</p>
              </article>
            </div>
          </section>

          <section class="section-block overview-section">
            <header class="section-block__header">
              <div>
                <h3>控制中心模块</h3>
                <p>参照 openclaw-control-center 的组织方式，把人员、记忆、文档和任务收拢到同一个控制台。</p>
              </div>
            </header>

            <div class="overview-status-grid">
              <article v-for="card in controlCenterCards" :key="card.label" class="overview-status-card">
                <span>{{ card.label }}</span>
                <strong>{{ card.value }}</strong>
                <p>{{ card.description }}</p>
              </article>
            </div>
          </section>
        </div>

      <div v-else-if="activeSection === 'platforms'" class="desktop-console-body desktop-console-body--platforms">
        <section class="section-block section-block--platforms">
          <header class="section-block__header">
            <div>
              <h3>代理配置</h3>
              <p>统一维护已接入的平台配置，并切换聊天窗口默认使用的目标。</p>
            </div>
            <div class="toolbar-actions">
              <button class="platform-tips-trigger" type="button" @click="showPlatformTips = !showPlatformTips">
                {{ showPlatformTips ? "收起说明" : "接入说明" }}
              </button>
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="openSubscriptionRecommendations">
                订阅推荐
              </button>
              <button class="desktop-console-panel__action" type="button" @click="handleCreatePlatform">新增平台</button>
            </div>
          </header>

          <div v-if="showPlatformTips" class="platform-inline-note">
            <p>OpenAI 兼容协议通常使用 `/v1/chat/completions`；Anthropic 原生协议通常使用 `/v1/messages`。</p>
            <p>平台列表优先读取 `~/.openclaw/openclaw.json` 的 `models.providers`；开启开关会切换到本地代理链接。</p>
          </div>

          <section class="platform-preset-section">
            <div class="platform-preset-section__header">
              <div>
                <h4>已配置平台</h4>
                <p>默认平台会直接用于聊天窗口，请在这里统一维护。</p>
              </div>
            </div>

            <div v-if="platforms.length > 0" class="platform-list platform-list--grid">
              <article v-for="platform in platforms" :key="platform.id" class="platform-card">
                <div class="platform-card__header">
                  <div>
                    <strong>{{ platform.name }}</strong>
                    <span>{{ platform.protocol.toUpperCase() }}</span>
                  </div>
                  <label class="toggle-switch">
                    <input
                      :checked="platform.enabled"
                      type="checkbox"
                      @change="handleTogglePlatform(platform.id, ($event.target as HTMLInputElement).checked)"
                    />
                    <span />
                  </label>
                </div>
                <p class="platform-card__endpoint">{{ getPlatformDisplayBaseUrl(platform) }}</p>
                <div class="platform-card__meta">
                  <span>前缀 {{ platform.pathPrefix }}</span>
                  <span>{{ platform.enabled ? "本地代理" : "默认直连" }}</span>
                </div>
                <div class="platform-card__actions">
                  <button
                    class="section-block__link"
                    type="button"
                    :disabled="activePlatformId === platform.id"
                    @click="handleSetActivePlatform(platform.id)"
                  >
                    {{ activePlatformId === platform.id ? "当前默认" : "设为默认" }}
                  </button>
                  <button class="section-block__link" type="button" @click="handleEditPlatform(platform)">编辑</button>
                  <button class="section-block__link section-block__link--danger" type="button" @click="handleDeletePlatform(platform.id)">
                    删除
                  </button>
                </div>
              </article>
            </div>
            <div v-else class="empty-state">暂无平台配置，点击“新增平台”开始接入。</div>
          </section>
        </section>
      </div>

      <div v-else-if="activeSection === 'staff'" class="desktop-console-body desktop-console-body--overview staff-layout">
        <section class="section-block overview-section">
          <header class="section-block__header staff-overview-header">
            <div>
              <h3>员工总览</h3>
            </div>
            <div class="toolbar-actions">
              <button
                class="desktop-console-panel__action desktop-console-panel__action--ghost"
                type="button"
                :disabled="isStaffSnapshotRefreshing"
                @click="handleRefreshStaffSnapshot()"
              >
                {{ isStaffSnapshotRefreshing ? "刷新中..." : "刷新" }}
              </button>
            </div>
          </header>

          <div class="staff-brief-grid">
            <article v-for="member in staffMembers" :key="member.agentId" class="staff-brief-card">
              <button
                class="staff-brief-card__delete"
                type="button"
                title="删除角色"
                :aria-label="`删除角色 ${member.displayName}`"
                @click.stop="openStaffDeleteConfirm(member)"
              >
                <svg viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M9 3h6l1 2h4v2H4V5h4l1-2Zm1 7h2v8h-2v-8Zm4 0h2v8h-2v-8ZM7 10h2v8H7v-8Zm-1 10h12l1-13H5l1 13Z" />
                </svg>
              </button>
              <div class="staff-brief-head">
                <div class="staff-avatar">
                  <div class="staff-avatar__badge" :class="[getStaffStatusClass(member), getStaffAvatarColorClass(member.agentId)]">
                    {{ getStaffInitials(member.displayName) }}
                  </div>
                </div>
                <div class="staff-brief-identity">
                  <strong>{{ member.displayName }}</strong>
                  <p>{{ getStaffRoleLabel(member) }}</p>
                  <div class="staff-chip-row">
                    <span class="staff-status-chip" :class="getStaffStatusClass(member)">{{ member.statusLabel }}</span>
                    <span class="staff-soft-chip">{{ member.scheduledLabel }}</span>
                    <span v-if="member.channel" class="staff-soft-chip">{{ member.channel }}</span>
                  </div>
                </div>
              </div>

              <dl class="staff-brief-list">
                <div class="staff-brief-row">
                  <dt>工具权限</dt>
                  <dd>{{ member.toolsProfile || "—" }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>模型</dt>
                  <dd>{{ member.model || "—" }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>工作目录</dt>
                  <dd>{{ member.workspace || "—" }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>所属渠道</dt>
                  <dd>{{ member.channel || "—" }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>当前状态</dt>
                  <dd>{{ member.statusLabel }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>{{ member.currentWorkLabel }}</dt>
                  <dd class="staff-brief-row__current-work">{{ member.currentWork }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>最近产出</dt>
                  <dd class="staff-brief-row__recent-output">
                    <div
                      class="staff-markdown-output"
                      v-html="renderStaffRecentOutputMarkdown(member.recentOutput)"
                    />
                    <button
                      v-if="hasRecentOutput(member)"
                      class="staff-brief-row__toggle"
                      type="button"
                      @click="openRecentOutputModal(member)"
                    >
                      查看完整产出
                    </button>
                  </dd>
                </div>
                <div class="staff-brief-row">
                  <dt>是否在排班里</dt>
                  <dd>{{ member.scheduledLabel }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>关联资源</dt>
                  <dd>
                    <div class="staff-linked-actions staff-linked-actions--compact">
                      <button class="staff-linked-actions__button" type="button" title="该员工的记忆文件" @click="handleOpenMemberMemory(member)">
                        <span>记忆</span>
                        <strong>{{ getStaffLinkedResourceCounts(member).memory }}</strong>
                      </button>
                      <button class="staff-linked-actions__button" type="button" title="OpenClaw 技能库（全员共享），~/.openclaw/skills 下各 SKILL.md" @click="handleOpenMemberSkill(member)">
                        <span>技能库</span>
                        <strong>{{ getStaffLinkedResourceCounts(member).skill }}</strong>
                      </button>
                      <button class="staff-linked-actions__button" type="button" title="该员工的 OpenClaw 工具权限配置（profile / allow / deny）" @click="handleOpenMemberTool(member)">
                        <span>TOOLS</span>
                        <strong>{{ getStaffLinkedResourceCounts(member).tool }}</strong>
                      </button>
                    </div>
                  </dd>
                </div>
              </dl>
            </article>
            <div v-if="staffMembers.length === 0" class="empty-state">暂无可显示的员工信息。请确认 `~/.openclaw/openclaw.json` 或运行时员工目录存在。</div>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'role-workflow'" class="desktop-console-body desktop-console-body--overview role-workflow-layout">
        <section class="section-block overview-section role-workflow-section">
          <header class="section-block__header role-workflow-header">
            <h3>角色工作流</h3>
            <input
              v-model="roleWorkflowKeyword"
              class="management-filter-input role-workflow-search"
              type="search"
              placeholder="搜索角色 / 分组 / 场景"
            />
          </header>

          <p class="role-workflow-summary">
            共 {{ roleWorkflowTotalCount }} 个角色，当前显示 {{ roleWorkflowVisibleCount }} 个。
          </p>

          <div v-if="roleWorkflowVisibleCount === 0" class="empty-state">未找到匹配角色，请调整关键词。</div>

          <div v-else class="role-workflow-division-list">
            <section v-for="division in roleWorkflowDivisions" :key="division.id" class="role-workflow-division">
              <header class="role-workflow-division__header">
                <h4>{{ division.titleZh }}</h4>
                <span>{{ division.count }} 个角色</span>
              </header>

              <div v-for="group in division.groups" :key="group.id" class="role-workflow-group">
                <h5 v-if="group.titleZh && division.groups.length > 1">{{ group.titleZh }}</h5>
                <div class="role-workflow-grid">
                  <article
                    v-for="role in group.roles"
                    :key="role.id"
                    class="role-workflow-card"
                    role="button"
                    tabindex="0"
                    @click="openRoleWorkflowDetail(role.id)"
                    @keydown.enter.prevent="openRoleWorkflowDetail(role.id)"
                    @keydown.space.prevent="openRoleWorkflowDetail(role.id)"
                  >
                    <strong>{{ role.nameZh }}</strong>
                    <p>{{ role.workflowZh }}</p>
                  </article>
                </div>
              </div>
            </section>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'skill-market'" class="desktop-console-body desktop-console-body--overview skill-market-layout">
        <section class="section-block overview-section skill-market-section">
          <div class="skill-market-category-row" role="tablist" aria-label="技能分类">
            <button
              v-for="category in skillMarketCategories"
              :key="`skill-market-category-${category.id}`"
              class="skill-market-category-card"
              :class="{
                active: activeSkillMarketCategory === category.id,
                'skill-market-category-card--top': category.id === 'top'
              }"
              type="button"
              @click="selectSkillMarketCategory(category.id)"
            >
              <span class="skill-market-category-card__icon">{{ category.icon }}</span>
              <strong>{{ category.label }}</strong>
              <small>{{ category.hint }}</small>
            </button>
          </div>

          <div class="skill-market-toolbar">
            <input
              v-model="skillMarketSearch"
              class="management-filter-input skill-market-toolbar__search"
              type="search"
              placeholder="搜索关键词..."
              @keydown.enter.prevent="refreshSkillMarket(true)"
            />
            <button class="desktop-console-panel__action desktop-console-panel__action--ghost skill-market-toolbar__refresh" type="button" @click="refreshSkillMarket(true)">
              搜索
            </button>
          </div>

          <p class="skill-market-summary">{{ skillMarketSummaryText }}</p>

          <div v-if="skillMarketLoading" class="empty-state">正在加载技能市场数据...</div>
          <div v-else-if="skillMarketError" class="empty-state">{{ skillMarketError }}</div>
          <div v-else-if="pagedSkillMarketSkills.length === 0" class="empty-state">没有匹配的技能，请调整关键词或分类。</div>
          <div v-else class="skill-market-grid">
            <article
              v-for="(skill, index) in pagedSkillMarketSkills"
              :key="`${skill.slug || skill.name}-${index}`"
              class="skill-market-card"
              role="button"
              tabindex="0"
              @click="openSkillMarketDetailModal(skill)"
              @keydown.enter.prevent="openSkillMarketDetailModal(skill)"
              @keydown.space.prevent="openSkillMarketDetailModal(skill)"
            >
              <div class="skill-market-card__head">
                <div class="skill-market-card__avatar">{{ getSkillMarketInitial(skill.name) }}</div>
                <div class="skill-market-card__title">
                  <strong>{{ skill.name }}</strong>
                  <p>{{ getSkillMarketDescription(skill) }}</p>
                </div>
              </div>
              <div class="skill-market-card__chips">
                <span>{{ getSkillMarketCategoryLabel(skill.category) }}</span>
                <span v-if="skill.ownerName">@{{ skill.ownerName }}</span>
              </div>
              <div class="skill-market-card__meta">
                <span>↓ {{ formatSkillMarketCount(skill.downloads) }}</span>
                <span>☆ {{ formatSkillMarketCount(skill.stars) }}</span>
                <span>{{ formatSkillMarketVersion(skill.version) }}</span>
              </div>
            </article>
          </div>
          <div v-if="!skillMarketLoading && !skillMarketError && skillMarketCurrentTotalPages > 1" class="skill-market-pagination">
            <button
              class="skill-market-pagination__button"
              type="button"
              :disabled="!skillMarketCanPrevPage"
              @click="goPrevSkillMarketPage"
            >
              上一页
            </button>
            <button
              v-for="page in skillMarketPageNumbers"
              :key="`skill-market-page-${page}`"
              class="skill-market-pagination__page"
              :class="{ active: page === skillMarketPage }"
              type="button"
              @click="goToSkillMarketPage(page)"
            >
              {{ page }}
            </button>
            <button
              class="skill-market-pagination__button"
              type="button"
              :disabled="!skillMarketCanNextPage"
              @click="goNextSkillMarketPage"
            >
              下一页
            </button>
          </div>
          <p class="skill-market-copyright">
            数据来源于
            <button
              class="skill-market-copyright__link"
              type="button"
              @click="openCodingPlanPlatform('https://skillhub.tencent.com/')"
            >
              腾讯云 SkillHub
            </button>
          </p>
        </section>
      </div>

      <div v-else-if="activeSection === 'channels'" class="desktop-console-body desktop-console-body--overview">
        <section class="section-block overview-section channels-section">
          <header class="section-block__header channels-config-header">
            <div>
              <h3>已配置频道</h3>
              <p>可直接管理账号、绑定 Agent，并随时编辑或删除频道配置。</p>
            </div>
            <div class="toolbar-actions">
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshMessageChannelSnapshot()">
                刷新
              </button>
            </div>
          </header>

          <div v-if="channelGroups.length > 0" class="configured-channel-list">
            <article v-for="group in channelGroups" :key="`configured-${group.channelType}`" class="configured-channel-card">
              <div class="configured-channel-card__head">
                <div class="configured-channel-card__identity">
                  <div class="channel-card__icon-shell configured-channel-card__icon">
                    <img
                      v-if="getMessageChannelIcon(group.channelType)"
                      :src="getMessageChannelIcon(group.channelType)"
                      :alt="getMessageChannelDisplayName(group.channelType)"
                    />
                    <span v-else>{{ group.channelType.charAt(0).toUpperCase() }}</span>
                  </div>
                  <div class="configured-channel-card__title">
                    <strong>{{ getMessageChannelDisplayName(group.channelType) }}</strong>
                    <p>{{ group.channelType }}</p>
                  </div>
                  <span class="channel-status-dot" :class="{ 'is-on': group.status === 'connected' }" />
                </div>
                <div class="configured-channel-card__actions">
                  <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleAddChannelAccount(group.channelType)">
                    + 添加账号
                  </button>
                  <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleDeleteChannel(group.channelType)">
                    删除频道
                  </button>
                </div>
              </div>

              <div class="configured-channel-account-list">
                <div
                  v-for="account in group.accounts"
                  :key="`account-${group.channelType}-${account.accountId}`"
                  class="configured-channel-account-row"
                >
                  <strong class="configured-channel-account-row__name">{{ account.name || account.accountId }}</strong>
                  <label class="configured-channel-account-row__binding">
                    <span>绑定 Agent</span>
                    <select
                      :value="account.agentId ?? ''"
                      @change="handleBindChannelAccount(group.channelType, account.accountId, ($event.target as HTMLSelectElement).value)"
                    >
                      <option value="">未绑定</option>
                      <option v-for="member in staffMembers" :key="`channel-agent-${member.agentId}`" :value="member.agentId">
                        {{ member.displayName }}（{{ member.agentId }}）
                      </option>
                    </select>
                  </label>
                  <div class="configured-channel-account-row__actions">
                    <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleEditChannelAccount(group.channelType, account.accountId)">
                      编辑
                    </button>
                    <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleDeleteChannelAccount(group.channelType, account.accountId)">
                      删除
                    </button>
                  </div>
                </div>
                <div v-if="group.accounts.length === 0" class="empty-state empty-state--small">
                  当前频道暂无账号，点击“添加账号”开始配置。
                </div>
              </div>
            </article>
          </div>
          <div v-else class="empty-state">暂无已配置频道。可在下方点击频道卡片开始配置。</div>

          <div class="channel-group">
            <h4>支持的频道</h4>
            <div v-if="visibleMessageChannels.length > 0" class="channel-grid">
              <button
                v-for="channel in visibleMessageChannels"
                :key="`supported-${channel.id}`"
                class="channel-card channel-card--interactive"
                :class="{ 'channel-card--configured': isMessageChannelConfigured(channel.id) }"
                type="button"
                @click="handleOpenChannelConfigFromCard(channel.id)"
              >
                <div class="channel-card__icon-shell">
                  <img :src="channel.icon" :alt="channel.name" />
                </div>
                <div class="channel-card__content">
                  <div class="channel-card__title-row">
                    <strong>{{ channel.name }}</strong>
                    <span v-if="channel.plugin" class="channel-pill">插件</span>
                    <span class="channel-status-dot" :class="{ 'is-on': isMessageChannelConfigured(channel.id) }" />
                  </div>
                  <p>{{ channel.description }}</p>
                </div>
              </button>
            </div>
            <div v-else class="empty-state empty-state--small">暂无可用频道。</div>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'bindings'" class="desktop-console-body desktop-console-body--overview">
        <section class="section-block overview-section bindings-section">
          <header class="section-block__header">
            <div>
              <h3>宠物绑定中心</h3>
              <p>在这里管理绑定码、远程多宠物连接和 Agent 授权。</p>
            </div>
          </header>

          <div class="bindings-config-grid">
            <article class="bindings-config-card">
              <h4>我的宠物绑定码</h4>
              <p>将绑定码分享给对方，即可在远程设备完成连接。</p>
              <div class="bindings-config-card__value">{{ petBindingCode || "未生成" }}</div>
              <label class="bindings-config-field">
                <span>编辑绑定码</span>
                <input
                  v-model="bindingCodeDraft"
                  type="text"
                  placeholder="ABCD-EFGH"
                  @blur="bindingCodeDraft = normalizeBindingCode(bindingCodeDraft)"
                />
              </label>
              <div class="bindings-config-actions">
                <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="savePetBindingCode">
                  保存绑定码
                </button>
                <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="copyPetBindingCode">
                  复制绑定码
                </button>
                <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="regeneratePetBindingCode">
                  重新生成
                </button>
              </div>
            </article>

            <article class="bindings-config-card">
              <h4>添加远程宠物</h4>
              <p>输入对方绑定码并确认后，对方宠物将出现在聊天标签页。</p>
              <label class="bindings-config-field">
                <span>对方绑定码</span>
                <input
                  v-model="incomingBindingCode"
                  type="text"
                  placeholder="输入对方绑定码"
                  @blur="incomingBindingCode = normalizeBindingCode(incomingBindingCode)"
                  @keydown.enter.prevent="bindRemotePetByCode"
                />
              </label>
              <div class="bindings-config-actions bindings-config-actions--right">
                <button class="desktop-console-panel__action" type="button" @click="bindRemotePetByCode">
                  通过绑定码添加
                </button>
              </div>
            </article>
          </div>
        </section>

        <section class="section-block overview-section">
          <header class="section-block__header">
            <div>
              <h3>远程宠物列表</h3>
              <p>按宠物逐条配置授权能力，可随时打开互聊或解除绑定。</p>
            </div>
          </header>
          <div v-if="boundPets.length > 0" class="staff-brief-grid">
            <article v-for="petPeer in boundPets" :key="`bound-${petPeer.id}`" class="staff-brief-card">
              <div class="staff-brief-head">
                <div class="staff-avatar">
                  <div class="staff-avatar__badge">{{ petPeer.petName.charAt(0) }}</div>
                </div>
                <div class="staff-brief-identity">
                  <strong>{{ petPeer.petName }}</strong>
                  <p>{{ petPeer.ownerLabel }}</p>
                </div>
              </div>
              <dl class="staff-brief-list">
                <div class="staff-brief-row">
                  <dt>绑定码</dt>
                  <dd>{{ petPeer.bindingCode }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>绑定时间</dt>
                  <dd>{{ formatTime(petPeer.linkedAt) }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>昵称</dt>
                  <dd>
                    <input
                      :value="petPeer.petName"
                      type="text"
                      class="management-filter-input"
                      placeholder="远程宠物昵称"
                      @change="updateBoundPetName(petPeer.id, ($event.target as HTMLInputElement).value)"
                    />
                  </dd>
                </div>
                <div class="staff-brief-row">
                  <dt>Agent 授权</dt>
                  <dd>
                    <label v-for="capability in petPeer.capabilities" :key="`${petPeer.id}-${capability.id}`" class="task-switch">
                      <input
                        type="checkbox"
                        :checked="capability.enabled"
                        @change="setBoundPetCapability(petPeer.id, capability.id, ($event.target as HTMLInputElement).checked)"
                      />
                      <span>{{ capability.label }}</span>
                    </label>
                    <div v-if="petPeer.capabilities.length === 0" class="empty-state empty-state--small">
                      当前远程宠物暂无可配置能力。
                    </div>
                  </dd>
                </div>
              </dl>
              <div class="platform-modal__actions">
                <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="openBoundPetChat(petPeer.id)">
                  打开互聊
                </button>
                <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="removeBoundPet(petPeer.id)">
                  解除绑定
                </button>
              </div>
            </article>
          </div>
          <div v-else class="empty-state">还没有远程绑定宠物。先在上方配置绑定码并输入对方绑定码完成连接。</div>
        </section>
      </div>

      <div v-else-if="activeSection === 'tasks'" class="desktop-console-body desktop-console-body--overview">
        <section class="section-block overview-section ptask-section">
          <header class="section-block__header ptask-header">
            <div>
              <h3>计划任务</h3>
              <p>来自 OpenClaw cron/jobs.json 的调度任务，按 Agent 与状态管理。</p>
            </div>
            <div class="ptask-header__actions">
              <label class="ptask-toggle">
                <span>计划任务</span>
                <span class="ptask-toggle__badge">已启用</span>
              </label>
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshTaskSnapshot()">刷新</button>
            </div>
          </header>

          <div class="tasks-kpi-grid">
            <article class="tasks-kpi-card tasks-kpi-card--primary">
              <span>任务总数</span>
              <strong>{{ taskBoardMetrics.total }}</strong>
              <small>当前看板累计条目</small>
            </article>
            <article class="tasks-kpi-card">
              <span>已启用</span>
              <strong>{{ taskBoardMetrics.pending }}</strong>
              <small>将在 cron 中继续运行</small>
            </article>
            <article class="tasks-kpi-card">
              <span>12 小时内</span>
              <strong>{{ taskBoardMetrics.dueSoon }}</strong>
              <small>即将到期</small>
            </article>
            <article class="tasks-kpi-card">
              <span>待执行</span>
              <strong>{{ taskBoardMetrics.overdue }}</strong>
              <small>运行时间已到或已过</small>
            </article>
          </div>

          <div class="ptask-toolbar">
            <div class="ptask-tabs">
              <button
                class="ptask-tab"
                :class="{ active: cronTaskTab === 'late' }"
                type="button"
                @click="cronTaskTab = 'late'"
              >
                执行中 <em>{{ cronTaskStatusCounts.late }}</em>
              </button>
              <button
                class="ptask-tab"
                :class="{ active: cronTaskTab === 'scheduled' }"
                type="button"
                @click="cronTaskTab = 'scheduled'"
              >
                待执行 <em>{{ cronTaskStatusCounts.scheduled }}</em>
              </button>
              <button
                class="ptask-tab"
                :class="{ active: cronTaskTab === 'disabled' }"
                type="button"
                @click="cronTaskTab = 'disabled'"
              >
                已完成 <em>{{ cronTaskStatusCounts.disabled }}</em>
              </button>
              <button
                class="ptask-tab"
                :class="{ active: cronTaskTab === 'all' }"
                type="button"
                @click="cronTaskTab = 'all'"
              >
                全部 <em>{{ cronTaskStatusCounts.all }}</em>
              </button>
            </div>
            <div class="ptask-toolbar__right">
              <select v-model="cronTaskAgentFilter" class="ptask-agent-select">
                <option value="all">全部 Agent</option>
                <option v-for="agent in cronTaskAgents" :key="agent" :value="agent">{{ agent }}</option>
              </select>
            </div>
          </div>

          <div class="ptask-list">
            <article v-for="task in filteredCronTasks" :key="task.id" class="ptask-card" :class="[getCronTaskCardClass(task.statusKind)]">
              <div class="ptask-card__head">
                <div class="ptask-card__indicator" :class="getTaskStatusClass(task.statusKind)" />
                <div class="ptask-card__title-row">
                  <strong>{{ task.name }}</strong>
                  <div class="ptask-card__tags">
                    <span class="task-status-pill" :class="getTaskStatusClass(task.statusKind)">{{ formatTaskStatus(task.statusKind) }}</span>
                    <span class="task-priority-pill" :class="getTaskScheduleClass(task.scheduleKind, task.deleteAfterRun)">{{ formatTaskScheduleKind(task.scheduleKind, task.deleteAfterRun) }}</span>
                    <span v-if="task.agentId && task.agentId !== '未标注'" class="ptask-tag ptask-tag--agent">{{ task.agentId }}</span>
                    <span v-if="task.sessionTarget" class="ptask-tag ptask-tag--session">{{ task.sessionTarget }}</span>
                  </div>
                </div>
                <div class="ptask-card__actions">
                  <button
                    v-if="task.enabled"
                    class="ptask-action-btn ptask-action-btn--pause"
                    type="button"
                    title="暂停任务"
                    @click="toggleCronTaskEnabled(task.id, false)"
                  >⏸</button>
                  <button
                    v-else
                    class="ptask-action-btn ptask-action-btn--play"
                    type="button"
                    title="启动任务"
                    @click="toggleCronTaskEnabled(task.id, true)"
                  >▶</button>
                </div>
              </div>

              <div class="ptask-card__body">
                <div class="ptask-card__meta">
                  <div v-if="task.agentId" class="ptask-meta-item">
                    <span>Agent</span>
                    <strong>{{ task.agentId }}</strong>
                  </div>
                  <div class="ptask-meta-item">
                    <span>执行方式</span>
                    <strong>{{ task.statusLabel }}</strong>
                  </div>
                  <div class="ptask-meta-item">
                    <span>下次运行</span>
                    <strong>{{ task.nextRunAtMs ? formatTime(task.nextRunAtMs) : '—' }}</strong>
                  </div>
                  <div v-if="task.nextRunAtMs" class="ptask-meta-item">
                    <span>距运行</span>
                    <strong>{{ formatCronNextRun(task.nextRunAtMs) }}</strong>
                  </div>
                  <div class="ptask-meta-item">
                    <span>创建时间</span>
                    <strong>{{ formatCronTimestamp(task.createdAtMs) }}</strong>
                  </div>
                  <div class="ptask-meta-item">
                    <span>上次更新</span>
                    <strong>{{ formatCronTimestamp(task.updatedAtMs) }}</strong>
                  </div>
                </div>
                <p v-if="task.summary" class="ptask-card__summary">{{ task.summary }}</p>
              </div>
            </article>

            <div v-if="filteredCronTasks.length === 0" class="empty-state ptask-empty">
              {{ cronTaskTab === 'all' ? '暂无任务。请确认 cron/jobs.json 文件存在，或点击「刷新」重新读取。' : '当前筛选条件下没有任务。' }}
            </div>
          </div>

        </section>
      </div>

    <div v-if="!isConsoleWindowMode" class="desktop-console-panel__resize-handle" @pointerdown="handlePanelResizeStart" />
  </section>

    <div v-if="activeSkillMarketDetail" class="platform-modal-backdrop" @click.self="closeSkillMarketDetailModal">
      <section class="platform-modal skill-market-detail-modal" role="dialog" aria-modal="true" aria-label="技能详情">
        <header class="platform-modal__header skill-market-detail-modal__header">
          <div class="skill-market-detail-modal__identity">
            <div class="skill-market-detail-modal__avatar">{{ getSkillMarketInitial(activeSkillMarketDetail.name) }}</div>
            <div>
              <strong>{{ activeSkillMarketDetail.name }}</strong>
              <p>{{ activeSkillMarketDetail.slug || "skill" }}</p>
            </div>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop="closeSkillMarketDetailModal">×</button>
        </header>

        <div class="skill-market-detail-modal__body">
          <div class="skill-market-detail-modal__chips">
            <span>{{ formatSkillMarketVersion(activeSkillMarketDetail.version) }}</span>
            <span>{{ getSkillMarketCategoryLabel(activeSkillMarketDetail.category) }}</span>
          </div>

          <p class="skill-market-detail-modal__description">{{ getSkillMarketDescription(activeSkillMarketDetail) }}</p>

          <div class="skill-market-detail-modal__stats">
            <article class="skill-market-detail-modal__stat">
              <span>下载量</span>
              <strong>{{ formatSkillMarketCount(activeSkillMarketDetail.downloads) }}</strong>
            </article>
            <article class="skill-market-detail-modal__stat">
              <span>收藏</span>
              <strong>{{ formatSkillMarketCount(activeSkillMarketDetail.stars) }}</strong>
            </article>
            <article class="skill-market-detail-modal__stat">
              <span>安装量</span>
              <strong>{{ formatSkillMarketCount(activeSkillMarketDetail.installs) }}</strong>
            </article>
          </div>

          <p class="skill-market-detail-modal__source">
            数据来源于腾讯云 SkillHub，作者 {{ activeSkillMarketDetail.ownerName || "未知" }}
          </p>
        </div>

        <div class="platform-modal__actions skill-market-detail-modal__actions">
          <button
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openSkillMarketHomepage(activeSkillMarketDetail.homepage)"
          >
            访问 SkillHub
          </button>
        </div>
      </section>
    </div>

    <div v-if="activeRecentOutputMember" class="platform-modal-backdrop" @click.self="closeRecentOutputModal">
      <section class="platform-modal recent-output-modal" role="dialog" aria-modal="true" :aria-label="recentOutputModalTitle">
        <header class="platform-modal__header">
          <div>
            <strong>{{ recentOutputModalTitle }}</strong>
            <p>完整 Markdown 产出内容</p>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop="closeRecentOutputModal">×</button>
        </header>
        <div class="recent-output-modal__content">
          <div
            class="staff-markdown-output staff-markdown-output--modal is-expanded"
            v-html="renderStaffRecentOutputMarkdown(activeRecentOutputMember.recentOutput)"
          />
        </div>
      </section>
    </div>

    <div v-if="sessionOverlayLog" class="platform-modal-backdrop" @click.self="closeSessionLogOverlay">
      <section class="platform-modal session-log-overlay">
        <header class="platform-modal__header">
          <div>
            <strong>会话请求详情</strong>
            <p>{{ sessionOverlayLog.platformName }} · {{ sessionOverlayLog.method }} {{ sessionOverlayLog.path || sessionOverlayLog.endpoint }}</p>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop="closeSessionLogOverlay">×</button>
        </header>

        <div class="platform-modal__form">
          <div class="detail-stat-grid">
            <article class="detail-stat-card detail-stat-card--primary">
              <span>状态码</span>
              <strong>{{ sessionOverlayLog.responseStatus || "未返回" }}</strong>
            </article>
            <article class="detail-stat-card">
              <span>耗时</span>
              <strong>{{ formatLatencyStat(sessionOverlayLog.duration) }}</strong>
            </article>
            <article class="detail-stat-card">
              <span>TTFT</span>
              <strong>{{ formatLatencyStat(sessionOverlayLog.firstTokenTime) }}</strong>
            </article>
            <article class="detail-stat-card">
              <span>输出速度</span>
              <strong>{{ formatSpeed(sessionOverlayLog.tokensPerSecond) }}</strong>
            </article>
          </div>

          <div class="detail-summary-card">
            <div class="detail-summary-grid">
              <div>
                <span>平台</span>
                <strong>{{ sessionOverlayLog.platformName }}</strong>
              </div>
              <div>
                <span>时间</span>
                <strong>{{ formatTime(sessionOverlayLog.createdAt) }}</strong>
              </div>
              <div>
                <span>输入/输出 Token</span>
                <strong>{{ formatTokenPair(sessionOverlayLog) }}</strong>
              </div>
              <div>
                <span>缓存读取 Token</span>
                <strong>{{ formatCacheTokens(sessionOverlayLog) }}</strong>
              </div>
            </div>
            <div class="detail-endpoint">{{ getLogRequestUrl(sessionOverlayLog) }}</div>
          </div>

          <div v-if="sessionOverlayLog.requestHeaders && Object.keys(sessionOverlayLog.requestHeaders).length > 0" class="detail-code">
            <div class="detail-code__header">
              <h4>请求头</h4>
            </div>
            <pre>{{ JSON.stringify(maskSensitiveHeaders(sessionOverlayLog.requestHeaders), null, 2) }}</pre>
          </div>

          <div class="detail-tab-row">
            <button
              v-for="section in buildPreviewSections(sessionOverlayLog)"
              :key="section.id"
              class="detail-tab"
              :class="{ active: getActivePreviewSection(sessionOverlayLog, sessionPreviewSection).id === section.id }"
              type="button"
              @click="sessionPreviewSection = section.id"
            >
              {{ section.label }}
            </button>
          </div>

          <div class="detail-code">
            <div class="detail-code__header">
              <h4>{{ getActivePreviewSection(sessionOverlayLog, sessionPreviewSection).label }}</h4>
              <span>{{ getLogRequestUrl(sessionOverlayLog) }}</span>
            </div>
            <pre>{{ getActivePreviewSection(sessionOverlayLog, sessionPreviewSection).view.text }}</pre>
          </div>
        </div>
      </section>
    </div>

    <div v-if="activeResourceModal && activeResourceMember" class="platform-modal-backdrop" @click.self="closeResourceModal">
      <section class="platform-modal resource-modal">
        <header class="platform-modal__header">
          <div>
            <strong>{{ resourceModalHeaderTitle }}</strong>
            <p>{{ activeResourceModalDescription }}</p>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop.prevent="closeResourceModal">×</button>
        </header>

        <div class="resource-modal__content" :class="{ 'resource-modal__content--full': activeResourceModal === 'skill' || activeResourceModal === 'tool' }">
          <!-- 技能：内置 + 安装，可单独开启/禁用 -->
          <template v-if="activeResourceModal === 'skill'">
            <section class="openclaw-list-panel">
              <div class="openclaw-list-panel__toolbar">
                <input
                  v-model="resourceModalFilterText"
                  class="management-filter-input"
                  type="text"
                  placeholder="筛选技能名称或描述"
                />
                <span class="openclaw-list-panel__count">{{ openClawSkillsTotalCount }} 项（内置 {{ openClawSkillsList.builtIn?.length ?? 0 }} + 安装 {{ openClawSkillsList.installed?.length ?? 0 }}）</span>
                <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshOpenClawSkillsList()">重新读取</button>
              </div>

              <div class="openclaw-skill-switch" role="tablist" aria-label="技能分类切换">
                <button
                  class="openclaw-skill-switch__button"
                  :class="{ 'is-active': openClawSkillCategory === 'builtIn' }"
                  type="button"
                  role="tab"
                  :aria-selected="openClawSkillCategory === 'builtIn'"
                  @click="openClawSkillCategory = 'builtIn'"
                >
                  内置技能
                  <em>{{ filteredOpenClawBuiltInSkills.length }}</em>
                </button>
                <button
                  class="openclaw-skill-switch__button"
                  :class="{ 'is-active': openClawSkillCategory === 'installed' }"
                  type="button"
                  role="tab"
                  :aria-selected="openClawSkillCategory === 'installed'"
                  @click="openClawSkillCategory = 'installed'"
                >
                  安装技能
                  <em>{{ filteredOpenClawInstalledSkills.length }}</em>
                </button>
              </div>

              <section class="openclaw-skills-section">
                <h5 class="openclaw-tools-category__title">{{ activeOpenClawSkillsTitle }}</h5>
                <div v-if="activeOpenClawSkills.length === 0" class="empty-state management-empty-state empty-state--small">
                  {{ activeOpenClawSkillsEmptyText }}
                </div>
                <ul v-else class="openclaw-skill-cards">
                  <li v-for="skill in activeOpenClawSkills" :key="skill.id" class="openclaw-skill-card">
                    <div class="openclaw-skill-card__head">
                      <strong>{{ skill.name }}</strong>
                      <label class="openclaw-skill-card__toggle" :aria-label="`${skill.enabled ? '禁用' : '启用'}技能 ${skill.name}`">
                        <input type="checkbox" :checked="skill.enabled" @change="setOpenClawSkillEnabled(skill.id, ($event.target as HTMLInputElement).checked)" />
                        <span class="openclaw-skill-card__toggle-slider" />
                      </label>
                    </div>
                    <p class="openclaw-skill-card__desc">{{ skill.description }}</p>
                    <small v-if="openClawSkillCategory === 'installed' && skill.relativePath" class="openclaw-skill-card__path">{{ skill.relativePath }}</small>
                  </li>
                </ul>
              </section>
            </section>
          </template>

          <!-- 工具：支持权限范围、Profile 与单工具开关 -->
          <template v-else-if="activeResourceModal === 'tool'">
            <section class="openclaw-list-panel">
              <div class="openclaw-list-panel__toolbar openclaw-tools-toolbar">
                <div class="openclaw-tools-toolbar__controls">
                  <label class="openclaw-tools-control">
                    <span>权限范围</span>
                    <select
                      class="openclaw-tools-control__select"
                      :value="openClawToolsScope"
                      :disabled="isOpenClawToolsSaving"
                      @change="handleOpenClawToolsScopeChange(($event.target as HTMLSelectElement).value)"
                    >
                      <option value="agent">当前员工</option>
                      <option value="global">全局默认</option>
                    </select>
                  </label>
                  <label class="openclaw-tools-control">
                    <span>Profile</span>
                    <select
                      class="openclaw-tools-control__select"
                      :value="openClawToolsList.profile || 'default'"
                      :disabled="isOpenClawToolsSaving"
                      @change="setOpenClawToolsProfile(($event.target as HTMLSelectElement).value)"
                    >
                      <option v-for="option in openClawToolsProfileOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
                    </select>
                  </label>
                </div>
                <div class="openclaw-tools-toolbar__meta">
                  <span class="openclaw-list-panel__count">已启用 {{ openClawToolsEnabledCount }} / {{ openClawToolsList.tools.length }}</span>
                  <button
                    class="desktop-console-panel__action desktop-console-panel__action--ghost openclaw-tools-toolbar__refresh"
                    type="button"
                    :disabled="isOpenClawToolsSaving"
                    @click="refreshOpenClawToolsList(activeResourceMember?.agentId ?? null, openClawToolsScope)"
                  >
                    重新读取
                  </button>
                </div>
              </div>
              <div v-if="openClawToolsByCategory.length === 0" class="empty-state management-empty-state empty-state--small">
                当前没有可展示的工具配置。
              </div>
              <div v-for="group in openClawToolsByCategory" :key="group.category" class="openclaw-tools-category">
                <h5 class="openclaw-tools-category__title">{{ group.category }}</h5>
                <ul class="openclaw-tool-cards">
                  <li v-for="tool in group.tools" :key="tool.id" class="openclaw-tool-card">
                    <div class="openclaw-tool-card__head">
                      <code class="openclaw-tool-card__id">{{ tool.id }}</code>
                      <label class="openclaw-skill-card__toggle" :aria-label="`${tool.enabled ? '禁用' : '启用'}工具 ${tool.id}`">
                        <input
                          type="checkbox"
                          :checked="tool.enabled"
                          :disabled="isOpenClawToolsSaving"
                          @change="setOpenClawToolEnabled(tool.id, ($event.target as HTMLInputElement).checked)"
                        />
                        <span class="openclaw-skill-card__toggle-slider" />
                      </label>
                      <span class="openclaw-tool-card__badge" :class="{ 'openclaw-tool-card__badge--on': tool.enabled }">{{ tool.enabled ? '已启用' : '未启用' }}</span>
                    </div>
                    <p class="openclaw-tool-card__desc">{{ tool.description }}</p>
                  </li>
                </ul>
              </div>
            </section>
          </template>

          <!-- 记忆：保留原有侧边栏 + 编辑区 -->
          <template v-else>
            <aside class="management-sidebar">
              <div class="management-sidebar__tools">
                <div class="management-sidebar__headline">
                  <strong>{{ resourceSidebarHeadline }}</strong>
                  <small>{{ activeResourceTotalCount }} 条</small>
                </div>
                <input
                  v-model="resourceModalFilterText"
                  class="management-filter-input"
                  type="text"
                  placeholder="筛选标题、摘要或路径"
                />
              </div>
              <div class="management-nav">
                <button
                  v-for="record in filteredMemberMemoryRecords"
                  :key="record.id"
                  class="management-nav-item"
                  :class="{ active: selectedMemoryRecord?.id === record.id }"
                  type="button"
                  @click="handleSelectMemory(record)"
                >
                  <div class="management-nav-item__topline">
                    <strong>{{ record.title }}</strong>
                  </div>
                  <p>{{ record.summary }}</p>
                  <small>{{ record.scope }} · {{ record.relativePath }} · {{ formatTime(record.updatedAt) }}</small>
                </button>
                <div v-if="filteredMemberMemoryRecords.length === 0" class="empty-state management-empty-state">当前员工暂无可显示的记忆资料。</div>
              </div>
            </aside>
            <section class="management-editor management-editor--memory">
              <div class="management-editor__header">
                <div>
                  <strong>{{ activeResourceSelectedLabel }}</strong>
                  <p>{{ selectedMemoryPurposeDescription }}</p>
                </div>
                <div class="management-editor__actions">
                  <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshMemorySnapshot()">重新读取</button>
                </div>
              </div>
              <form class="management-editor__form management-editor__form--memory" @submit.prevent="handleSaveMemory">
                <div class="memory-meta-strip">
                  <div class="memory-meta-strip__item" v-if="memoryDraft.title">
                    <span>标题</span>
                    <strong>{{ memoryDraft.title }}</strong>
                  </div>
                  <div class="memory-meta-strip__item" v-if="memoryDraft.owner">
                    <span>归属</span>
                    <strong>{{ memoryDraft.owner }}</strong>
                  </div>
                  <div class="memory-meta-strip__item" v-if="memoryDraft.scope">
                    <span>分类</span>
                    <strong>{{ memoryDraft.scope }}</strong>
                  </div>
                  <div class="memory-meta-strip__item memory-meta-strip__item--wide" v-if="memoryDraft.relativePath">
                    <span>路径</span>
                    <strong>{{ memoryDraft.relativePath }}</strong>
                  </div>
                  <div class="memory-meta-strip__item memory-meta-strip__item--accent">
                    <span>{{ memoryEditorModeLabel }}</span>
                    <strong>{{ selectedMemoryId ? (selectedMemoryRecord?.exists === false ? "保存时创建" : "保存时写回") : "未选择" }}</strong>
                  </div>
                </div>
                <div class="memory-editor-textarea-wrap">
                  <textarea v-model="memoryDraft.content" class="memory-editor-textarea" placeholder="记忆文件内容" />
                </div>
                <div class="management-form-grid__actions management-form-grid__actions--editor">
                  <small v-if="memoryDraft.sourcePath" class="memory-meta-source-hint" :title="memoryDraft.sourcePath">{{ memoryDraft.sourcePath }}</small>
                  <button class="desktop-console-panel__action" type="submit" :disabled="!memoryDraft.sourcePath">{{ memoryEditorModeLabel }}</button>
                </div>
              </form>
            </section>
          </template>
        </div>
      </section>
    </div>

    <div
      v-if="isLobsterInstallWizardPrimed"
      class="platform-modal-backdrop lobster-install-wizard-backdrop"
      :class="{ 'is-open': isLobsterInstallWizardOpen }"
      :aria-hidden="!isLobsterInstallWizardOpen"
      :inert="!isLobsterInstallWizardOpen"
      @click.self="closeLobsterInstallWizard"
    >
      <section
        class="platform-modal lobster-install-wizard-modal"
        :class="{ 'lobster-install-wizard-modal--step3': lobsterInstallWizardStep === 3 }"
        role="dialog"
        aria-modal="true"
        aria-label="龙虾安装引导"
      >
        <button
          class="platform-modal__close lobster-install-wizard__close"
          type="button"
          aria-label="关闭"
          :disabled="lobsterInstallRunning"
          @click.stop="closeLobsterInstallWizard"
        >
          ×
        </button>

        <div class="lobster-install-wizard__steps">
          <div
            v-for="step in lobsterInstallWizardSteps"
            :key="step.id"
            class="lobster-install-wizard__step"
            :class="{
              'is-active': lobsterInstallWizardStep === step.id,
              'is-done': lobsterInstallWizardStep > step.id
            }"
          >
            <span>{{ step.id }}</span>
            <i v-if="step.id < lobsterInstallWizardSteps.length" />
          </div>
        </div>

        <header class="lobster-install-wizard__header">
          <h3>{{ lobsterInstallStepTitle }}</h3>
          <p>{{ lobsterInstallStepDescription }}</p>
        </header>

        <section class="lobster-install-wizard__panel">
          <template v-if="lobsterInstallWizardStep === 1">
            <div class="lobster-install-wizard__welcome">
              <section class="lobster-install-wizard__welcome-card">
                <div class="lobster-install-wizard__lang-row" role="tablist" aria-label="语言选择">
                  <button
                    v-for="localeOption in appLocaleOptions"
                    :key="`wizard-locale-${localeOption.value}`"
                    type="button"
                    :class="{ 'is-active': appLocale === localeOption.value }"
                    @click="setAppLocale(localeOption.value)"
                  >
                    {{ localeOption.label }}
                  </button>
                </div>
                <ul>
                  <li>{{ tr("wizard.welcome.feature1") }}</li>
                  <li>{{ tr("wizard.welcome.feature2") }}</li>
                  <li>{{ tr("wizard.welcome.feature3") }}</li>
                  <li>{{ tr("wizard.welcome.feature4") }}</li>
                </ul>
              </section>

              <section class="lobster-install-wizard__risk-card">
                <strong>{{ tr("wizard.risk.title") }}</strong>
                <p>{{ tr("wizard.risk.body") }}</p>
                <ul>
                  <li>{{ tr("wizard.risk.item1") }}</li>
                  <li>{{ tr("wizard.risk.item2") }}</li>
                  <li>{{ tr("wizard.risk.item3") }}</li>
                  <li>{{ tr("wizard.risk.item4") }}</li>
                  <li>{{ tr("wizard.risk.item5") }}</li>
                </ul>
              </section>

              <label class="lobster-install-wizard__risk-check">
                <input v-model="lobsterInstallRiskAccepted" type="checkbox" />
                <span>{{ tr("wizard.risk.accept") }}</span>
              </label>
            </div>
          </template>

          <template v-else-if="lobsterInstallWizardStep === 2">
            <div class="lobster-install-wizard__runtime">
              <div class="lobster-install-wizard__runtime-top">
                <div>
                  <h4>{{ tr("wizard.runtime.title") }}</h4>
                  <p class="lobster-install-wizard__lead">{{ tr("wizard.runtime.lead") }}</p>
                </div>
                <button
                  class="desktop-console-panel__action desktop-console-panel__action--ghost"
                  type="button"
                  :disabled="lobsterInstallGuideLoading"
                  @click="refreshLobsterInstallGuide()"
                >
                  {{ lobsterInstallGuideLoading ? tr("wizard.runtime.checking") : tr("wizard.runtime.recheck") }}
                </button>
              </div>
              <div
                class="lobster-install-wizard__runtime-summary"
                :class="{
                  'is-ready': lobsterInstallGuide?.ready && !lobsterInstallGuideLoading,
                  'is-blocked': lobsterInstallGuide && !lobsterInstallGuide.ready && !lobsterInstallGuideLoading,
                  'is-checking': lobsterInstallGuideLoading
                }"
              >
                <span>{{ lobsterInstallGuideSummaryText }}</span>
                <small>{{ lobsterInstallGuideCheckedCount }}/{{ lobsterInstallGuideCheckTotal }} {{ tr("wizard.runtime.check_count_unit") }}</small>
              </div>

              <div class="lobster-install-wizard__check-list">
                <article
                  v-for="item in lobsterInstallGuide?.checks ?? []"
                  :key="item.id"
                  class="lobster-install-wizard__check-item"
                  :class="[`is-${item.status}`]"
                >
                  <div class="lobster-install-wizard__check-title">
                    <strong>{{ item.title }}</strong>
                    <span>{{ getLobsterInstallCheckStatusLabel(item.status) }}</span>
                  </div>
                  <p>{{ item.detail }}</p>
                </article>
                <div v-if="!lobsterInstallGuide && !lobsterInstallGuideLoading" class="empty-state">
                  {{ tr("wizard.runtime.empty") }}
                </div>
              </div>
            </div>
          </template>

          <template v-else-if="lobsterInstallWizardStep === 3">
            <div class="lobster-install-wizard__provider">
              <div class="lobster-install-wizard__provider-panel">
                <label class="lobster-install-wizard__field lobster-install-wizard__field--span2">
                  <div class="lobster-install-wizard__field-head">
                    <span>{{ tr("wizard.provider.field.provider") }}</span>
                    <button
                      v-if="lobsterEffectiveProviderOption?.docsUrl"
                      type="button"
                      class="lobster-install-wizard__provider-doc-link"
                      @click="openLobsterProviderDocs()"
                    >
                      {{ tr("wizard.provider.docs") }}
                    </button>
                  </div>
                  <div v-if="isWindowsRuntime" class="lobster-provider-picker" role="group" :aria-label="tr('wizard.provider.field.provider')">
                    <div class="lobster-provider-picker__options">
                      <button
                        type="button"
                        class="lobster-provider-picker__option"
                        :class="{ 'is-active': lobsterProviderPresetKey === '' }"
                        @click="lobsterProviderPresetKey = ''; handleLobsterProviderPresetChange()"
                      >
                        {{ tr("wizard.provider.keep_current") }}
                      </button>
                    </div>
                    <template v-for="group in lobsterProviderGroupOptions" :key="`wizard-provider-group-${group.key}`">
                      <div class="lobster-provider-picker__group">
                        <p v-if="group.key !== 'custom'" class="lobster-provider-picker__title">{{ group.label }}</p>
                        <div class="lobster-provider-picker__options">
                          <button
                            v-for="preset in group.options"
                            :key="`wizard-provider-${preset.id}`"
                            type="button"
                            class="lobster-provider-picker__option"
                            :class="{ 'is-active': lobsterProviderPresetKey === preset.id }"
                            @click="lobsterProviderPresetKey = preset.id; handleLobsterProviderPresetChange()"
                          >
                            {{ getLobsterProviderOptionLabel(preset) }}
                          </button>
                        </div>
                      </div>
                    </template>
                  </div>
                  <div v-else class="lobster-install-wizard__provider-select-wrap">
                    <span class="lobster-install-wizard__provider-select-icon" aria-hidden="true">
                      {{ lobsterProviderSelectedIcon }}
                    </span>
                    <select
                      v-model="lobsterProviderPresetKey"
                      class="lobster-install-wizard__provider-select"
                      @change="handleLobsterProviderPresetChange()"
                    >
                      <option value="">{{ tr("wizard.provider.keep_current") }}</option>
                      <template v-for="group in lobsterProviderGroupOptions" :key="`wizard-provider-group-${group.key}`">
                        <option v-if="group.key === 'custom'" disabled value="__divider__">────────────</option>
                        <optgroup v-if="group.key !== 'custom'" :label="group.label">
                          <option v-for="preset in group.options" :key="`wizard-provider-${preset.id}`" :value="preset.id">
                            {{ getLobsterProviderOptionDisplayName(preset) }}
                          </option>
                        </optgroup>
                        <template v-else>
                          <option v-for="preset in group.options" :key="`wizard-provider-${preset.id}`" :value="preset.id">
                            {{ getLobsterProviderOptionDisplayName(preset) }}
                          </option>
                        </template>
                      </template>
                    </select>
                    <span class="lobster-install-wizard__provider-select-caret" aria-hidden="true" />
                  </div>
                </label>

                <label class="lobster-install-wizard__field lobster-install-wizard__field--span2">
                  <span>{{ tr("wizard.provider.field.base_url") }}</span>
                  <input
                    v-model="lobsterProviderForm.baseUrl"
                    type="text"
                    placeholder="https://api.openai.com"
                    @input="lobsterProviderConfigured = false"
                  />
                </label>

                <label class="lobster-install-wizard__field lobster-install-wizard__field--span2">
                  <span>{{ tr("wizard.provider.field.model") }}</span>
                  <input
                    v-model="lobsterProviderForm.model"
                    type="text"
                    :placeholder="lobsterProviderModelPlaceholder"
                    @input="lobsterProviderConfigured = false"
                  />
                  <small class="lobster-install-wizard__field-hint">{{ tr("wizard.provider.model_hint") }}</small>
                </label>

                <label
                  v-if="lobsterShowCustomProtocolPicker"
                  class="lobster-install-wizard__field lobster-install-wizard__field--span2"
                >
                  <span>{{ tr("wizard.provider.protocol") }}</span>
                  <div class="lobster-install-wizard__protocol-toggle" role="group" :aria-label="tr('wizard.provider.protocol')">
                    <button
                      v-for="option in lobsterProviderApiKindOptions"
                      :key="`wizard-provider-api-kind-${option.value}`"
                      type="button"
                      class="lobster-install-wizard__protocol-option"
                      :class="{ 'is-active': lobsterProviderApiKind === option.value }"
                      @click="applyLobsterProviderApiKind(option.value)"
                    >
                      {{ option.label }}
                    </button>
                  </div>
                </label>

                <label class="lobster-install-wizard__field lobster-install-wizard__field--span2">
                  <span>{{ tr("wizard.provider.field.api_key") }}</span>
                  <div class="lobster-install-wizard__secret">
                    <input
                      v-model="lobsterProviderForm.apiKey"
                      :type="lobsterProviderShowKey ? 'text' : 'password'"
                      :placeholder="lobsterProviderApiKeyPlaceholder"
                      @input="lobsterProviderConfigured = false"
                    />
                    <button
                      type="button"
                      class="desktop-console-panel__action desktop-console-panel__action--ghost lobster-install-wizard__secret-toggle"
                      @click="lobsterProviderShowKey = !lobsterProviderShowKey"
                    >
                      {{ lobsterProviderShowKey ? "🙈" : "👁" }}
                    </button>
                  </div>
                  <small v-if="!lobsterProviderRequiresApiKey" class="lobster-install-wizard__provider-no-key">
                    {{ tr("wizard.provider.no_key_hint") }}
                  </small>
                </label>

                <button
                  class="desktop-console-panel__action lobster-install-wizard__provider-save"
                  type="button"
                  :disabled="lobsterProviderSaving || !lobsterProviderCanSave"
                  @click="saveLobsterProviderFromWizard()"
                >
                  {{
                    lobsterProviderSaving
                      ? tr("wizard.provider.saving")
                      : lobsterProviderConfigured
                        ? tr("wizard.provider.saved")
                        : lobsterProviderRequiresApiKey
                          ? tr("wizard.provider.save")
                          : tr("wizard.provider.save_simple")
                  }}
                </button>
                <small class="lobster-install-wizard__provider-tip">{{ tr("wizard.provider.key_tip") }}</small>
              </div>
            </div>
          </template>

          <template v-else-if="lobsterInstallWizardStep === 4">
            <div class="lobster-install-wizard__installing">
              <div class="lobster-install-wizard__installing-head">
                <div class="lobster-install-wizard__installing-mark">⚙️</div>
                <h4>{{ tr("wizard.installing.title") }}</h4>
                <p>{{ tr("wizard.installing.subtitle") }}</p>
              </div>

              <div class="lobster-install-wizard__installing-progress">
                <div class="lobster-install-wizard__installing-progress-head">
                  <span>{{ tr("wizard.installing.progress") }}</span>
                  <em>{{ lobsterInstallProgressDisplay }}%</em>
                </div>
                <div class="lobster-install-wizard__installing-progress-track">
                  <i :style="{ width: `${lobsterInstallProgressDisplay}%` }" />
                </div>
              </div>

              <div class="lobster-install-wizard__installing-list">
                <article
                  v-for="item in lobsterInstallComponentStates"
                  :key="`wizard-installing-item-${item.id}`"
                  class="lobster-install-wizard__installing-item"
                >
                  <div class="lobster-install-wizard__installing-item-main">
                    <span class="lobster-install-wizard__installing-item-icon" :class="`is-${item.status}`" />
                    <div class="lobster-install-wizard__installing-item-copy">
                      <strong>{{ item.name }}</strong>
                      <small>{{ item.description }}</small>
                    </div>
                  </div>
                  <em class="lobster-install-wizard__installing-item-status" :class="`is-${item.status}`">
                    {{ getLobsterInstallComponentStatusLabel(item.status) }}
                  </em>
                </article>
              </div>

              <small class="lobster-install-wizard__installing-wait">{{ tr("wizard.installing.wait") }}</small>
            </div>
          </template>

          <template v-else>
            <div
              v-if="lobsterInstallFinishedResult?.success"
              class="lobster-install-wizard__complete"
            >
              <div class="lobster-install-wizard__complete-head">
                <div class="lobster-install-wizard__complete-mark">🎉</div>
                <strong>{{ tr("wizard.complete.title") }}</strong>
                <p>{{ tr("wizard.complete.subtitle") }}</p>
              </div>

              <div class="lobster-install-wizard__complete-list">
                <div class="lobster-install-wizard__complete-item">
                  <span>{{ tr("wizard.complete.provider") }}</span>
                  <em>{{ lobsterCompleteProviderSummary }}</em>
                </div>
                <div class="lobster-install-wizard__complete-item">
                  <span>{{ tr("wizard.complete.components") }}</span>
                  <em>{{ lobsterCompleteComponentsSummary }}</em>
                </div>
                <div class="lobster-install-wizard__complete-item">
                  <span>{{ tr("wizard.complete.gateway") }}</span>
                  <em :class="{ 'is-running': lobsterCompleteGatewayRunning }">{{ lobsterCompleteGatewaySummary }}</em>
                </div>
              </div>

              <small>{{ tr("wizard.complete.footer") }}</small>
            </div>

            <div
              v-else
              class="lobster-install-wizard__result"
              :class="{ 'is-success': lobsterInstallFinishedResult?.success, 'is-failed': lobsterInstallFinishedResult && !lobsterInstallFinishedResult.success }"
            >
              <strong>{{ lobsterInstallFinishedResult?.success ? "安装完成" : "安装失败" }}</strong>
              <p>{{ lobsterInstallFinishedResult?.detail || "暂无安装结果" }}</p>
              <small v-if="lobsterInstallFinishedResult">耗时 {{ formatDuration(lobsterInstallFinishedResult.durationMs) }}</small>
              <div v-if="lobsterInstallRuntimeLogs" class="detail-code">
                <h4>执行输出</h4>
                <pre>{{ lobsterInstallRuntimeLogs }}</pre>
              </div>
            </div>
          </template>
        </section>

        <footer class="lobster-install-wizard__actions">
          <button
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            :disabled="lobsterInstallRunning"
            @click="(lobsterInstallWizardStep === 1 || lobsterInstallWizardStep === 5) ? closeLobsterInstallWizard() : handleLobsterInstallWizardBack()"
          >
            {{ lobsterInstallWizardStep === 1 ? tr("wizard.button.cancel") : tr("wizard.button.back") }}
          </button>
          <button
            v-if="lobsterInstallWizardStep === 3"
            class="lobster-install-wizard__skip"
            type="button"
            :disabled="lobsterInstallRunning || lobsterProviderSaving"
            @click="handleLobsterProviderSkip()"
          >
            {{ tr("wizard.provider.skip") }}
          </button>
          <span v-else class="lobster-install-wizard__actions-spacer" />
          <button
            class="desktop-console-panel__action"
            type="button"
            :disabled="lobsterInstallRunning || lobsterInstallGuideLoading || !lobsterInstallCanGoNext"
            @click="handleLobsterInstallWizardNext()"
          >
            {{
              lobsterInstallWizardStep === 5
                ? (lobsterInstallFinishedResult?.success ? tr("wizard.button.get_started") : tr("wizard.button.finish"))
                : tr("wizard.button.next")
            }}
          </button>
        </footer>
      </section>
    </div>

    <div v-if="isSystemSettingsOpen" class="platform-modal-backdrop" @click.self="closeSystemSettings">
      <div class="platform-modal system-settings-modal" role="dialog" aria-modal="true" aria-label="系统设置">
        <header class="platform-modal__header">
          <h3>{{ tr("system.settings.title") }}</h3>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop="closeSystemSettings">×</button>
        </header>
        <div class="system-settings-body">
          <section class="system-settings-section">
            <h4 class="system-settings-panel-title">{{ tr("system.settings.general") }}</h4>
            <div class="system-settings-stack">
              <div class="system-settings-field">
                <h5 class="system-settings-section__title">{{ tr("system.settings.theme") }}</h5>
                <div class="system-settings-choice-row" role="tablist" aria-label="theme">
                  <button
                    v-for="themeOption in appThemeOptions"
                    :key="`settings-theme-${themeOption.value}`"
                    type="button"
                    class="system-settings-choice"
                    :class="{ 'is-active': draftAppTheme === themeOption.value }"
                    @click="draftAppTheme = themeOption.value"
                  >
                    {{ themeOption.label }}
                  </button>
                </div>
              </div>

              <div class="system-settings-field">
                <h5 class="system-settings-section__title">{{ tr("system.settings.language") }}</h5>
                <div class="system-settings-choice-row" role="tablist" aria-label="language">
                  <button
                    v-for="localeOption in appLocaleOptions"
                    :key="`settings-locale-${localeOption.value}`"
                    type="button"
                    class="system-settings-choice"
                    :class="{ 'is-active': draftAppLocale === localeOption.value }"
                    @click="draftAppLocale = localeOption.value"
                  >
                    {{ localeOption.label }}
                  </button>
                </div>
              </div>

              <div class="system-settings-field">
                <h5 class="system-settings-section__title">{{ tr("system.settings.startup") }}</h5>
                <div
                  class="system-settings-toggle-row"
                  :class="{ 'is-disabled': !launchOnLoginSupported }"
                  @click="launchOnLoginSupported && (draftLaunchOnLoginEnabled = !draftLaunchOnLoginEnabled)"
                >
                  <div class="system-settings-toggle-row__label">
                    <strong>{{ tr("system.settings.startup") }}</strong>
                    <span>{{ launchOnLoginSupported ? tr("system.settings.startup.hint") : tr("system.settings.startup.unsupported") }}</span>
                  </div>
                  <div
                    class="sys-toggle"
                    :class="{ 'sys-toggle--on': draftLaunchOnLoginEnabled, 'sys-toggle--disabled': !launchOnLoginSupported }"
                    role="switch"
                    :aria-label="draftLaunchOnLoginEnabled ? '自启动已开启' : '自启动已关闭'"
                    :aria-checked="draftLaunchOnLoginEnabled"
                    tabindex="0"
                    @click.stop="launchOnLoginSupported && (draftLaunchOnLoginEnabled = !draftLaunchOnLoginEnabled)"
                    @keydown.enter.space.prevent="launchOnLoginSupported && (draftLaunchOnLoginEnabled = !draftLaunchOnLoginEnabled)"
                  >
                    <div class="sys-toggle__thumb" />
                  </div>
                </div>
              </div>
            </div>
          </section>

          <section class="system-settings-section">
            <h4 class="system-settings-section__title">{{ tr("system.settings.pet_size") }}</h4>
            <div class="system-settings-size-options">
              <label
                v-for="opt in ([
                  { value: 'small', label: tr('size.small'), px: 28 },
                  { value: 'medium', label: tr('size.medium'), px: 44 },
                  { value: 'large', label: tr('size.large'), px: 60 }
                ] as const)"
                :key="opt.value"
                class="system-settings-size-option"
                :class="{ 'is-selected': draftSizeLevel === opt.value }"
              >
                <input
                  type="radio"
                  name="pet-size"
                  :value="opt.value"
                  :checked="draftSizeLevel === opt.value"
                  class="system-settings-size-option__radio"
                  @change="draftSizeLevel = opt.value"
                />
                <div class="system-settings-size-option__preview">
                  <div class="system-settings-size-option__dot" :style="{ width: `${opt.px}px`, height: `${opt.px}px` }" />
                </div>
                <span class="system-settings-size-option__label">{{ opt.label }}</span>
              </label>
            </div>
          </section>

          <section class="system-settings-section">
            <h4 class="system-settings-section__title">{{ tr("system.settings.window_behavior") }}</h4>
            <div class="system-settings-toggle-row" @click="draftAlwaysOnTop = !draftAlwaysOnTop">
              <div class="system-settings-toggle-row__label">
                <strong>{{ tr("system.settings.always_top") }}</strong>
                <span>{{ tr("system.settings.always_top.hint") }}</span>
              </div>
              <div
                class="sys-toggle"
                :class="{ 'sys-toggle--on': draftAlwaysOnTop }"
                role="switch"
                :aria-label="draftAlwaysOnTop ? '置顶已开启' : '置顶已关闭'"
                :aria-checked="draftAlwaysOnTop"
                tabindex="0"
                @click.stop="draftAlwaysOnTop = !draftAlwaysOnTop"
                @keydown.enter.space.prevent="draftAlwaysOnTop = !draftAlwaysOnTop"
              >
                <div class="sys-toggle__thumb" />
              </div>
            </div>
          </section>
        </div>
        <footer class="system-settings-footer">
          <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="closeSystemSettings">{{ tr("system.settings.cancel") }}</button>
          <button class="desktop-console-panel__action" type="button" @click="handleSystemSettingsSave">{{ tr("system.settings.save") }}</button>
        </footer>
      </div>
    </div>

    <div v-if="isPlatformModalOpen" class="platform-modal-backdrop" @click.self="handleCancelPlatformEdit">
      <section class="platform-modal">
        <header class="platform-modal__header">
          <div>
            <strong>{{ editingPlatformId ? "编辑平台" : "新增平台" }}</strong>
            <p>可先从预设平台快速填充，再补全基础协议和 URL。</p>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop="handleCancelPlatformEdit">×</button>
        </header>

        <form class="platform-modal__form" @submit.prevent="handleSavePlatform">
          <section v-if="!editingPlatformId" class="platform-modal__preset-groups">
            <section class="platform-preset-section">
              <div class="platform-preset-section__header">
                <div>
                  <h4>平台预设</h4>
                  <p>展开一次即可直接选择目标平台，按国内外分组显示，选中后立即填充草稿。</p>
                </div>
              </div>

              <div class="platform-select-row">
                <select v-model="selectedPresetKey" class="platform-select" @change="handlePresetSelect">
                  <option value="">选择预设平台</option>
                  <optgroup label="国外平台">
                    <option v-for="preset in globalPlatformPresets" :key="`global-${preset.name}`" :value="`${preset.region}:${preset.name}`">
                      {{ preset.name }} · {{ preset.protocol.toUpperCase() }}
                    </option>
                  </optgroup>
                  <optgroup label="国内平台">
                    <option v-for="preset in chinaPlatformPresets" :key="`china-${preset.name}`" :value="`${preset.region}:${preset.name}`">
                      {{ preset.name }} · {{ preset.protocol.toUpperCase() }}
                    </option>
                  </optgroup>
                </select>
              </div>
            </section>
          </section>

          <div class="platform-modal__grid">
            <label>
              <span>平台名称</span>
              <input v-model="platformForm.name" type="text" placeholder="阿里" />
            </label>
            <label>
              <span>协议类型</span>
              <div v-if="isWindowsRuntime" class="platform-protocol-toggle" role="group" aria-label="协议类型">
                <button
                  type="button"
                  class="platform-protocol-toggle__option"
                  :class="{ 'is-active': platformForm.protocol === 'openai' }"
                  @click="platformForm.protocol = 'openai'"
                >
                  OpenAI 兼容
                </button>
                <button
                  type="button"
                  class="platform-protocol-toggle__option"
                  :class="{ 'is-active': platformForm.protocol === 'anthropic' }"
                  @click="platformForm.protocol = 'anthropic'"
                >
                  Anthropic Messages
                </button>
              </div>
              <select v-else v-model="platformForm.protocol">
                <option value="openai">OpenAI 兼容</option>
                <option value="anthropic">Anthropic Messages</option>
              </select>
            </label>
          </div>
          <label>
            <span>API Base URL</span>
            <input v-model="platformForm.baseUrl" type="text" placeholder="https://coding.dashscope.aliyuncs.com/coding" />
          </label>
          <label>
            <span>路径前缀</span>
            <input v-model="platformForm.pathPrefix" type="text" placeholder="/coding" />
            <small>本地代理通过此前缀区分平台，例如 /openai、/deepseek。</small>
          </label>
          <label>
            <span>本地代理端口</span>
            <input v-model.number="proxyPort" type="number" min="1" max="65535" placeholder="5005" />
            <small>代理地址示例：`http://127.0.0.1:{{ proxyPort }}{{ platformForm.pathPrefix || "/platform" }}`。</small>
          </label>
          <div class="platform-modal__actions">
            <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleCancelPlatformEdit">
              取消
            </button>
            <button class="desktop-console-panel__action" type="submit">保存变更</button>
          </div>
        </form>
      </section>
    </div>

    <div v-if="isChannelConfigModalOpen && activeChannelConfigMeta" class="platform-modal-backdrop" @click.self="closeChannelConfigModal">
      <section class="platform-modal channel-config-modal">
        <header class="platform-modal__header">
          <div>
            <strong>{{ channelConfigAllowEditAccountId ? `新增 ${activeChannelConfigMeta.name} 账号` : `配置 ${activeChannelConfigMeta.name}` }}</strong>
            <p>{{ activeChannelConfigMeta.description }}</p>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop="closeChannelConfigModal">×</button>
        </header>

        <form class="platform-modal__form channel-config-modal__form" @submit.prevent="handleSaveChannelConfig">
          <section class="channel-config-guide">
            <div class="channel-config-guide__head">
              <div>
                <h4>如何连接</h4>
                <p>{{ activeChannelConfigMeta.description }}</p>
              </div>
              <button
                v-if="activeChannelConfigMeta.docsUrl"
                class="channel-config-guide__docs"
                type="button"
                @click="handleOpenChannelConfigDocs"
              >
                查看文档 ↗
              </button>
            </div>
            <ol v-if="(activeChannelConfigMeta.instructions ?? []).length > 0" class="channel-config-guide__steps">
              <li v-for="step in activeChannelConfigMeta.instructions ?? []" :key="`channel-step-${activeChannelConfigMeta.id}-${step}`">
                {{ step }}
              </li>
            </ol>
            <p v-else class="channel-config-guide__fallback">保存配置后会自动启用该频道，后续可在“已配置频道”继续维护账号与绑定关系。</p>
          </section>

          <label class="channel-config-field">
            <span class="channel-config-field__label">账号 ID</span>
            <input
              v-model="channelConfigEditingAccountId"
              class="channel-config-field__input"
              type="text"
              :disabled="!channelConfigAllowEditAccountId"
              placeholder="default"
            />
            <small class="channel-config-field__hint">可自定义账号 ID，用于区分同一频道下的多个账号。</small>
          </label>

          <template v-for="field in activeChannelConfigMeta.fields ?? []" :key="`channel-field-${field.key}`">
            <div class="channel-config-field">
              <span class="channel-config-field__label">
                {{ field.label }}
                <em v-if="field.required" class="channel-config-field__required">*</em>
              </span>
              <div class="channel-config-field__input-row" :class="{ 'channel-config-field__input-row--secret': field.secret }">
                <input
                  v-model="channelConfigForm[field.key]"
                  class="channel-config-field__input"
                  :type="field.secret && !isChannelConfigSecretVisible(field.key) ? 'password' : 'text'"
                  :placeholder="field.placeholder"
                />
                <button
                  v-if="field.secret"
                  class="channel-config-field__secret-toggle"
                  type="button"
                  @click="toggleChannelConfigSecretVisibility(field.key)"
                >
                  {{ isChannelConfigSecretVisible(field.key) ? "隐藏" : "显示" }}
                </button>
              </div>
              <small v-if="field.description" class="channel-config-field__hint">{{ field.description }}</small>
              <small v-if="field.envVar" class="channel-config-field__env">环境变量：{{ field.envVar }}</small>
            </div>
          </template>

          <div v-if="(activeChannelConfigMeta.fields ?? []).length === 0" class="empty-state channel-config-empty">
            当前频道无需手动填写参数，保存后会直接启用该频道。
          </div>

          <div class="channel-config-modal__footer">
            <div v-if="channelConfigError" class="platform-form-error channel-config-modal__error">{{ channelConfigError }}</div>
            <div class="platform-modal__actions channel-config-modal__actions">
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="closeChannelConfigModal">
                取消
              </button>
              <button class="desktop-console-panel__action channel-config-modal__submit" type="submit" :disabled="isChannelConfigSaving">
                {{ isChannelConfigSaving ? "保存中..." : channelConfigAllowEditAccountId ? "✓ 保存并连接" : "✓ 保存并更新" }}
              </button>
            </div>
          </div>
        </form>
      </section>
    </div>

    <div v-if="activeRoleWorkflowBase" class="platform-modal-backdrop" @click.self="closeRoleWorkflowDetail">
      <section class="platform-modal role-workflow-detail-modal">
        <header class="platform-modal__header role-workflow-detail-modal__header">
          <div>
            <strong>角色详情</strong>
            <p>
              {{ activeRoleWorkflowBase.divisionTitleZh }}
              <span v-if="activeRoleWorkflowBase.groupTitleZh"> / {{ activeRoleWorkflowBase.groupTitleZh }}</span>
            </p>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" @click.stop="closeRoleWorkflowDetail">×</button>
        </header>

        <div class="role-workflow-detail-modal__body">
          <div
            v-if="roleWorkflowInstallNotice"
            class="role-workflow-detail-modal__notice"
            :class="`role-workflow-detail-modal__notice--${roleWorkflowInstallNotice.tone}`"
          >
            {{ roleWorkflowInstallNotice.text }}
          </div>

          <label class="role-workflow-detail-modal__field role-workflow-detail-modal__field--name">
            <span>角色中文名称</span>
            <input
              v-model="roleWorkflowNameZhDraft"
              class="role-workflow-detail-modal__name-input"
              type="text"
              placeholder="请输入角色中文名称"
            />
          </label>

          <label class="role-workflow-detail-modal__field">
            <span>详情内容（Markdown，可编辑）</span>
            <textarea
              v-model="roleWorkflowDetailDraft.contentZh"
              class="role-workflow-detail-modal__editor"
              rows="18"
              placeholder="在这里编辑角色详情 Markdown"
            />
          </label>

          <section class="role-workflow-detail-modal__versions">
            <header class="role-workflow-detail-modal__versions-header">
              <strong>已保存版本（最多 3 个）</strong>
            </header>
            <p v-if="roleWorkflowDetailSavedVersions.length === 0" class="role-workflow-detail-modal__versions-empty">
              暂无已保存版本，点击“保存修改”后会自动保留历史。
            </p>
            <ul v-else class="role-workflow-detail-modal__versions-list">
              <li v-for="version in roleWorkflowDetailSavedVersions" :key="version.id" class="role-workflow-detail-modal__version-item">
                <span class="role-workflow-detail-modal__version-time">{{ formatTime(version.savedAt) }}</span>
                <div class="role-workflow-detail-modal__version-actions">
                  <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="applyRoleWorkflowSavedVersion(version.id)">
                    载入
                  </button>
                  <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="deleteRoleWorkflowSavedVersion(version.id)">
                    删除
                  </button>
                </div>
              </li>
            </ul>
          </section>
        </div>

        <div class="platform-modal__actions role-workflow-detail-modal__actions">
          <button
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            :disabled="!isRoleWorkflowDraftChanged"
            @click="restoreRoleWorkflowOriginalContent"
          >
            恢复原始内容
          </button>
          <button
            class="desktop-console-panel__action"
            type="button"
            :disabled="!isRoleWorkflowDraftChanged"
            @click="saveRoleWorkflowDetail"
          >
            保存修改
          </button>
          <button
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            :disabled="isRoleWorkflowInstalling"
            @click="installRoleWorkflowRole"
          >
            {{ isRoleWorkflowInstalling ? "安装中..." : "安装角色" }}
          </button>
        </div>
      </section>
    </div>

    <div v-if="staffDeleteTargetMember" class="platform-modal-backdrop" @click.self="closeStaffDeleteConfirm">
      <section class="platform-modal staff-delete-modal" role="dialog" aria-modal="true" aria-label="删除角色确认">
        <header class="platform-modal__header">
          <div>
            <strong>移除角色</strong>
            <p>将「{{ staffDeleteTargetMember.displayName }}」从员工列表移除？</p>
          </div>
          <button class="platform-modal__close" type="button" aria-label="关闭" :disabled="isStaffDeleting" @click.stop="closeStaffDeleteConfirm">×</button>
        </header>

        <div class="staff-delete-modal__body">
          <label class="staff-delete-modal__option">
            <input v-model="staffDeleteRemoveFiles" type="checkbox" />
            <span>同时删除本地配置文件（工作区目录）</span>
          </label>
          <p v-if="staffDeleteError" class="platform-form-error">{{ staffDeleteError }}</p>
        </div>

        <div class="platform-modal__actions">
          <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" :disabled="isStaffDeleting" @click="closeStaffDeleteConfirm">
            取消
          </button>
          <button class="desktop-console-panel__action desktop-console-panel__action--danger" type="button" :disabled="isStaffDeleting" @click="confirmDeleteStaffMember">
            {{ isStaffDeleting ? "移除中..." : "确认移除" }}
          </button>
        </div>
      </section>
    </div>

    <div
      v-if="!isConsoleWindowMode && contextMenu.visible"
      ref="contextMenuRef"
      class="desktop-context-menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
    >
      <button class="desktop-context-menu__item" type="button" @click="openChatPanel()">{{ tr("context.chat") }}</button>
      <button class="desktop-context-menu__item" type="button" @click="openConsole('platforms')">{{ tr("context.platforms") }}</button>
      <button class="desktop-context-menu__item" type="button" @click="openConsole('channels')">{{ tr("context.channels") }}</button>
      <button class="desktop-context-menu__item" type="button" @click="openConsole('bindings')">{{ tr("context.bindings") }}</button>
      <button class="desktop-context-menu__item" type="button" @click="openLobsterConfig()">{{ tr("context.lobster") }}</button>
      <button class="desktop-context-menu__item" type="button" @click="openLogAnalysis('timeline')">{{ tr("context.logs") }}</button>
      <button class="desktop-context-menu__item" type="button" @click="openSubscriptionRecommendations()">{{ tr("context.subscription") }}</button>
      <button class="desktop-context-menu__item" type="button" @click="openSystemSettings()">{{ tr("context.system") }}</button>
      <button class="desktop-context-menu__item desktop-context-menu__item--danger" type="button" @click="handleQuitClick">
        {{ tr("context.quit") }}
      </button>
    </div>
  </main>
</template>
