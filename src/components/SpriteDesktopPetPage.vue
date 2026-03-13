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
import { sendOpenClawChat, type OpenClawMessage } from "../services/openclaw";
import {
  appendRequestLog,
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
  | "chat_typing"
  | "have_meal"
  | "sleep"
  | "smile_and_blink"
  | "smile_blink"
  | "stomp_feet"
  | "stretch_yawn_and_rub_your_eyes"
  | "think";
type ConsoleSection =
  | "overview"
  | "platforms"
  | "staff"
  | "memory"
  | "docs"
  | "tasks";

type LogAnalysisView = "timeline" | "sessions" | "failures";
type PanelMode = "console" | "logs";

type AnimationDefinition = {
  name: AnimationName;
  label: string;
  description: string;
  loop: boolean;
  sprite: string;
  config: AnimationConfig;
};

type ChatMessage = {
  id: string;
  role: "assistant" | "user" | "system";
  text: string;
  status: "pending" | "done" | "error";
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
  model: string;
  workspace: string;
  toolsProfile: string;
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
  chat_typing: {
    name: "chat_typing",
    label: "打字回复",
    description: "等待 OpenClaw 回复时保持输入状态。",
    loop: true,
    sprite: chatTypingSprite,
    config: chatTypingConfig as AnimationConfig
  },
  have_meal: {
    name: "have_meal",
    label: "开心进食",
    description: "收到回复或被投喂灵感后会满足地吃一口。",
    loop: false,
    sprite: haveMealSprite,
    config: haveMealConfig as AnimationConfig
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
  stretch_yawn_and_rub_your_eyes: {
    name: "stretch_yawn_and_rub_your_eyes",
    label: "伸懒腰醒神",
    description: "从睡眠中被唤醒时先打个哈欠，再恢复互动。",
    loop: false,
    sprite: stretchYawnSprite,
    config: stretchYawnConfig as AnimationConfig
  },
  think: {
    name: "think",
    label: "歪头思考",
    description: "查看控制台或进入空闲思考时会露出思索表情。",
    loop: true,
    sprite: thinkSprite,
    config: thinkConfig as AnimationConfig
  }
};

const actionTips: Record<AnimationName, string> = {
  act_cute_rotation: "被你拖起来后，它开始原地转圈卖萌。",
  chat_typing: "正在替你盯着回复，等 OpenClaw 把字打完。",
  have_meal: "像被投喂到一样，收到回应后会满足地吃一口。",
  sleep: "太久没有新互动，它已经安心睡着了。",
  smile_and_blink: "聊天窗口打开时，它会保持更专注的陪伴表情。",
  smile_blink: "今天状态不错，适合放在页面右下角陪你工作。",
  stomp_feet: "你刚刚戳到它了，它正在跺脚表达情绪。",
  stretch_yawn_and_rub_your_eyes: "刚被你叫醒，先伸个懒腰再继续营业。",
  think: "它在认真琢磨眼前的信息，像在陪你一起排查。"
};

const chatStorageKey = "keai.desktop-pet.openclaw.chat-history";
const sessionStorageKey = "keai.desktop-pet.openclaw.session-id";
const defaultChatMessages: ChatMessage[] = [
  {
    id: "welcome",
    role: "assistant",
    text: "点一下我就会展开 OpenClaw 对话框，回复会用文字气泡显示。",
    status: "done"
  }
];

const stage = ref<HTMLDivElement | null>(null);
const pet = ref<HTMLButtonElement | null>(null);
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
const chatMessages = ref<ChatMessage[]>([...defaultChatMessages]);
const chatMotionValue = ref(0);
const panelMotionValue = ref(0);
const bubbleMotionValue = ref(1);
const localRequestLogs = ref<RequestLog[]>([]);
const runtimeRequestLogs = ref<RequestLog[]>([]);
const platforms = ref<PlatformConfig[]>([]);
const staffMembers = ref<StaffMemberSnapshot[]>([]);
const memoryRecords = ref<MemoryRecord[]>([]);
const documentRecords = ref<DocumentRecord[]>([]);
const taskRecords = ref<TaskSnapshotItem[]>([]);
const activePlatformId = ref<string | null>(null);
const isEditingPlatform = ref(false);
const editingPlatformId = ref<string | null>(null);
const platformForm = ref<PlatformDraft>(createPlatformDraft());
const showPlatformTips = ref(false);
const isPlatformModalOpen = ref(false);
const selectedPresetKey = ref("");
const selectedLogId = ref<string | null>(null);
const selectedSessionId = ref<string | null>(null);
const selectedSessionLogId = ref<string | null>(null);
const selectedFailureKey = ref<string | null>(null);
const timelinePreviewSection = ref<PreviewSection>("request");
const sessionPreviewSection = ref<PreviewSection>("response");
const sessionOverlayLogId = ref<string | null>(null);
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
const staffSnapshotDetail = ref("正在读取员工配置...");
const staffSnapshotSourcePath = ref("");
const staffMissionStatement = ref("构建可持续自治的 AI 员工体系，持续完成高价值任务。");
const taskSnapshotDetail = ref("正在读取任务调度...");
const taskSnapshotSourcePath = ref("");
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

const viewportSize = 280;
const autoplayDelayMs = 9000;
const playbackRate = 3;
const sleepDelayMs = 24000;
const idleShowcaseSequence: AnimationName[] = ["think", "smile_and_blink", "have_meal", "act_cute_rotation"];
const platformPresets = getPlatformPresets();
const globalPlatformPresets = computed(() => platformPresets.filter((preset) => preset.region === "global"));
const chinaPlatformPresets = computed(() => platformPresets.filter((preset) => preset.region === "china"));
const openClawDefaultPlatformName = "OpenClaw 默认通道";
const consoleSections: Array<{ id: ConsoleSection; label: string }> = [
  { id: "overview", label: "总览" },
  { id: "platforms", label: "平台管理" },
  { id: "staff", label: "员工管理" },
  { id: "memory", label: "记忆管理" },
  { id: "docs", label: "文档管理" },
  { id: "tasks", label: "任务管理" }
];

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
const currentFrame = computed(() => activeAnimation.value.config.frames[currentFrameIndex.value]);
const activePlatform = computed(
  () => platforms.value.find((platform) => platform.id === activePlatformId.value && platform.enabled) || null
);
const requestLogs = computed<RequestLog[]>(() =>
  [...localRequestLogs.value, ...runtimeRequestLogs.value]
    .sort((left, right) => right.createdAt - left.createdAt)
    .filter((log, index, items) => items.findIndex((candidate) => candidate.id === log.id) === index)
);
const enabledPlatformCount = computed(() => platforms.value.filter((platform) => platform.enabled).length);
const configuredSubscriptionCount = computed(() => platforms.value.filter((platform) => platform.apiKey.trim()).length);
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
  const petCenterX = petPosition.value.x + viewportSize / 2;
  const preferredLeft = petCenterX - bubbleWidth / 2;
  const left = Math.min(Math.max(16, preferredLeft), Math.max(16, viewportWidth - bubbleWidth - 16));
  const topAbovePet = petPosition.value.y - 108;
  const topBelowPet = petPosition.value.y + viewportSize + 14;
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
  const scale = viewportSize / animation.config.frame_size.w;

  return {
    width: `${viewportSize}px`,
    height: `${viewportSize}px`,
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
  const petRight = petPosition.value.x + viewportSize;
  const petCenterY = petPosition.value.y + viewportSize / 2;
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
  const petCenterX = petPosition.value.x + viewportSize / 2;
  const petCenterY = petPosition.value.y + viewportSize / 2;
  const petSafeLeft = Math.max(0, petPosition.value.x - petClearance);
  const petSafeTop = Math.max(0, petPosition.value.y - petClearance);
  const petSafeRight = Math.min(viewportWidth, petPosition.value.x + viewportSize + petClearance);
  const petSafeBottom = Math.min(viewportHeight, petPosition.value.y + viewportSize + petClearance);
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
    clampAutoPosition(petPosition.value.x + viewportSize + gap, petCenterY - panelHeight / 2),
    clampAutoPosition(petCenterX - panelWidth / 2, petPosition.value.y - panelHeight - gap),
    clampAutoPosition(petCenterX - panelWidth / 2, petPosition.value.y + viewportSize + gap),
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
    { label: "启用平台", value: `${enabledPlatformCount.value}` },
    { label: "调用总数", value: `${requestLogs.value.length}` },
    { label: "网关状态", value: gatewayStatusValue },
    { label: "平均耗时", value: `${averageDuration} ms` },
    { label: "今日 Token", value: `${todayTokens}` },
    { label: "7 天 Token", value: `${sevenDayTokens}` },
    { label: "累计 Token", value: `${totalTokens}` },
    { label: "失败请求", value: `${failures}` }
  ];
});
const timelineEntries = computed(() => requestLogs.value);
const scheduledTaskCount = computed(() => 0);
const memoryStatusSummary = computed(() => ({
  main: memoryRecords.value.filter((item) => item.owner === "Main").length,
  agents: memoryRecords.value.filter((item) => item.owner !== "Main").length
}));
const documentStatusSummary = computed(() => ({
  main: documentRecords.value.filter((item) => item.owner === "Main").length,
  agents: documentRecords.value.filter((item) => item.owner !== "Main").length
}));
const memoryScopeOptions = computed(() => [
  { key: "all", label: "全部范围" },
  ...Array.from(new Set(memoryRecords.value.map((item) => item.scope.trim()).filter(Boolean))).map((scope) => ({
    key: scope,
    label: scope
  }))
]);
const documentCategoryOptions = computed(() => [
  { key: "all", label: "全部分类" },
  ...Array.from(new Set(documentRecords.value.map((item) => item.category.trim()).filter(Boolean))).map((category) => ({
    key: category,
    label: category
  }))
]);
const filteredMemoryRecords = computed(() => {
  const keyword = memoryFilterText.value.trim().toLowerCase();

  return memoryRecords.value.filter((record) => {
    const matchesScope = activeMemoryScope.value === "all" || record.scope === activeMemoryScope.value;
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
const selectedMemoryRecord = computed(
  () => filteredMemoryRecords.value.find((record) => record.id === selectedMemoryId.value) ?? filteredMemoryRecords.value[0] ?? null
);
const selectedDocumentRecord = computed(
  () =>
    filteredDocumentRecords.value.find((record) => record.id === selectedDocumentId.value) ?? filteredDocumentRecords.value[0] ?? null
);
const memoryEditorModeLabel = computed(() =>
  memoryDraft.value.sourcePath ? "保存记忆文件" : "选择记忆文件"
);
const documentEditorModeLabel = computed(() =>
  documentDraft.value.source ? "保存文档文件" : "选择文档文件"
);
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

  for (const log of requestLogs.value) {
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

  for (const log of requestLogs.value.filter((item) => isFailedLog(item))) {
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
        ? `已配置密钥 ${configuredSubscriptionCount.value} 个，启用平台 ${enabledPlatformCount.value} 个。`
        : "还没有接入平台，暂时没有可统计的订阅。"
  },
  {
    label: "默认平台",
    value: activePlatform.value?.name ?? openClawDefaultPlatformName,
    description: activePlatform.value
      ? `${activePlatform.value.protocol.toUpperCase()} · ${activePlatform.value.model}`
      : "可在平台管理中设置默认接入平台。"
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

type TauriWindowApi = {
  close: () => Promise<void> | void;
  destroy: () => Promise<void> | void;
  setIgnoreCursorEvents: (value: boolean, options?: { forward?: boolean }) => Promise<void> | void;
};

type TauriNamespace = {
  app?: {
    exit?: (code?: number) => Promise<void> | void;
  };
  core?: {
    invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
  };
  window?: {
    getCurrentWindow?: () => TauriWindowApi;
    cursorPosition?: () => Promise<{ x: number; y: number }>;
  };
};

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
    status
  };
}

function getStableChatMessages(messages: ChatMessage[]) {
  return messages.filter((message) => message.status !== "pending");
}

function loadChatHistory() {
  if (typeof window === "undefined" || !window.localStorage) {
    return [...defaultChatMessages];
  }

  try {
    const raw = window.localStorage.getItem(chatStorageKey);
    if (!raw) {
      return [...defaultChatMessages];
    }

    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [...defaultChatMessages];
    }

    const messages = parsed
      .map((item) => normalizeChatMessage(item))
      .filter((message): message is ChatMessage => message !== null)
      .filter((message) => message.status !== "pending");

    return messages.length > 0 ? messages : [...defaultChatMessages];
  } catch {
    return [...defaultChatMessages];
  }
}

function loadStoredSessionId() {
  if (typeof window === "undefined" || !window.localStorage) {
    return createSessionId();
  }

  const value = window.localStorage.getItem(sessionStorageKey);
  if (value) {
    return value;
  }

  const next = createSessionId();
  window.localStorage.setItem(sessionStorageKey, next);
  return next;
}

function persistChatHistory() {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }

  try {
    window.localStorage.setItem(chatStorageKey, JSON.stringify(getStableChatMessages(chatMessages.value)));
    window.localStorage.setItem(sessionStorageKey, currentSessionId.value);
  } catch {
    // Ignore storage errors so chat remains usable even in restricted environments.
  }
}

function loadProxyPort() {
  if (typeof window === "undefined" || !window.localStorage) {
    return 5005;
  }

  const raw = window.localStorage.getItem("keai.desktop-pet.proxy-port");
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 5005;
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

function resolveBaseAnimationName(now = performance.now()): AnimationName {
  if (isSending.value) {
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

function openConsole(section: ConsoleSection) {
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
    statusText.value = "平台管理已展开，可以新增、切换默认平台或修改接口配置。";
  } else if (section === "staff") {
    statusText.value = "员工管理已展开，适合维护角色、职责和轮值状态。";
    void refreshStaffSnapshot();
  } else if (section === "memory") {
    statusText.value = "记忆管理已展开，当前读取的是 OpenClaw 真实记忆文件。";
    void refreshMemorySnapshot();
  } else if (section === "docs") {
    statusText.value = "文档管理已展开，当前读取的是 OpenClaw 核心文档文件。";
    void refreshDocumentSnapshot();
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

function toggleConsolePanel(nextValue?: boolean) {
  const finalValue = nextValue ?? !isConsoleOpen.value;
  if (finalValue === isConsoleOpen.value) {
    return;
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

  const maxX = Math.max(0, bounds.width - viewportSize);
  const maxY = Math.max(0, bounds.height - viewportSize);

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
    x: Math.max(0, bounds.width - viewportSize - 48),
    y: Math.max(0, bounds.height - viewportSize - 56)
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

function handlePointerDown(event: PointerEvent) {
  const petEl = pet.value;
  if (!petEl) {
    return;
  }

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

function handlePointerMove(event: PointerEvent) {
  if (!isDragging.value || dragPointerId !== event.pointerId) {
    return;
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

  isDragging.value = false;
  dragPointerId = null;
  noteInteraction();
  statusText.value = "位置记住了，继续待机陪伴。";
  applyBaseAnimation(true);
}

function handleResize() {
  petPosition.value = clampPetPosition(petPosition.value.x, petPosition.value.y);
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

async function syncLocalProxyServer() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;
  if (!invoke) {
    return;
  }

  const payload: LocalProxyPlatformPayload[] = platforms.value.map((platform) => ({
    protocol: platform.protocol,
    baseUrl: platform.baseUrl,
    pathPrefix: platform.pathPrefix,
    apiKey: platform.apiKey
  }));

  try {
    await invoke("sync_local_proxy", { port: proxyPort.value, platforms: payload });
  } catch (error) {
    statusText.value = error instanceof Error ? error.message : "本地代理启动失败。";
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

  if (sessionOverlayLog.value) {
    sessionOverlayLogId.value = null;
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
}

function handleContextMenu(event: MouseEvent) {
  event.preventDefault();
  const menuWidth = 188;
  const menuHeight = 128;
  const maxX = Math.max(8, window.innerWidth - menuWidth - 8);
  const maxY = Math.max(8, window.innerHeight - menuHeight - 8);

  contextMenu.value = {
    visible: true,
    x: Math.min(event.clientX, maxX),
    y: Math.min(event.clientY, maxY)
  };
}

function handleWindowPointerDown(event: PointerEvent) {
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
  if (!text || isSending.value) {
    return;
  }

  const platform = activePlatform.value;
  const pendingId = createMessageId("assistant");
  const conversationHistory = [...openClawMessages.value];
  const messages: OpenClawMessage[] = [
    {
      role: "system",
      content: "你是桌宠里的 OpenClaw 助手，请使用简洁自然的中文回复。"
    },
    ...conversationHistory,
    {
      role: "user",
      content: text
    }
  ];
  const effectivePlatform = platform?.enabled ? platform : null;
  const endpoint = "openclaw://default";
  const protocol = "openai";
  const payload = { messages };
  const requestBody = safeJson(payload);
  const requestHeaders = buildRequestHeaders(protocol);
  const baseUrl = "openclaw://default";
  const path = "";
  const platformId = effectivePlatform?.id ?? "openclaw-default";
  const platformName = effectivePlatform?.name ?? "OpenClaw 默认通道";
  const startedAt = performance.now();

  chatMessages.value.push({
    id: createMessageId("user"),
    role: "user",
    text,
    status: "done"
  });
  chatMessages.value.push({
    id: pendingId,
    role: "assistant",
    text: "OpenClaw 正在思考中...",
    status: "pending"
  });
  chatInput.value = "";
  isSending.value = true;
  noteInteraction();
  applyBaseAnimation(true);
  statusText.value = `消息已经发给 ${platformName}，正在等待回复。`;
  startBubbleAnimation();
  scrollMessagesToBottom();

  try {
    const response = await sendOpenClawChat(messages);
    const completedAt = performance.now();
    const duration = Math.round(completedAt - startedAt);
    const promptTokens = response.usage?.promptTokens ?? estimateTokenCount(requestBody);
    const completionTokens = response.usage?.completionTokens ?? estimateTokenCount(response.text);
    const totalTokens = response.usage?.totalTokens ?? promptTokens + completionTokens;
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
  await invoke("save_source_file", {
    kind: "document",
    sourcePath: documentDraft.value.source,
    content: documentDraft.value.content
  });
  statusText.value = `文档文件“${documentDraft.value.title}”已保存。`;
  await refreshDocumentSnapshot();
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

async function refreshStaffSnapshot() {
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    staffMembers.value = [];
    staffSnapshotSourcePath.value = "";
    staffSnapshotDetail.value = "当前环境不支持读取 openclaw.json。";
    return;
  }

  try {
    const result = (await invoke("load_staff_snapshot")) as StaffSnapshotResponse;
    staffMembers.value = Array.isArray(result.members) ? result.members : [];
    staffSnapshotSourcePath.value = result.sourcePath ?? "";
    staffSnapshotDetail.value = result.detail ?? "员工配置读取完成。";
    staffMissionStatement.value = result.missionStatement || staffMissionStatement.value;
  } catch (error) {
    staffMembers.value = [];
    staffSnapshotSourcePath.value = "";
    staffSnapshotDetail.value = error instanceof Error ? error.message : "员工配置读取失败。";
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
    updatedAt: item.updatedAtMs
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
    updatedAt: item.updatedAtMs
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
  } catch (error) {
    memoryRecords.value = [];
    memorySnapshotSourcePath.value = "";
    memorySnapshotDetail.value = error instanceof Error ? error.message : "记忆文件读取失败。";
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
  const tauriApi = getTauriApi();
  const invoke = tauriApi?.core?.invoke;

  if (!invoke) {
    runtimeRequestLogs.value = [];
    runtimeLogDetail.value = "当前环境不支持读取 OpenClaw 运行时消息。";
    return;
  }

  try {
    const result = (await invoke("load_openclaw_message_logs")) as OpenClawMessageLogResponse;
    runtimeRequestLogs.value = Array.isArray(result.logs) ? result.logs : [];
    runtimeLogDetail.value = result.detail ?? "OpenClaw 运行时消息读取完成。";
  } catch (error) {
    runtimeRequestLogs.value = [];
    runtimeLogDetail.value = error instanceof Error ? error.message : "OpenClaw 运行时消息读取失败。";
  }
}

function formatMemoryStatus(_: string) {
  return "源文件";
}

function formatDocumentStatus(_: string) {
  return "源文件";
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

function handleTogglePlatform(platformId: string, enabled: boolean) {
  platforms.value = setPlatformEnabled(platforms.value, platformId, enabled);
  const current = platforms.value.find((item) => item.id === platformId);

  if (!current) {
    return;
  }

  if (enabled && !activePlatformId.value) {
    activePlatformId.value = platformId;
    setActivePlatform(platformId);
  }

  if (!enabled && activePlatformId.value === platformId) {
    activePlatformId.value = null;
    setActivePlatform(null);
  }

  statusText.value = enabled ? `${current.name} 已启用。` : `${current.name} 已停用。`;
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
  chatMessages.value = [...defaultChatMessages];
  chatInput.value = "";
  persistChatHistory();
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

  if (!cursorPosition) {
    return;
  }

  if (isDragging.value || contextMenu.value.visible) {
    await setWindowIgnoreCursorEvents(false);
    return;
  }

  const cursor = await cursorPosition();
  const scale = window.devicePixelRatio || 1;
  const cursorX = cursor.x / scale;
  const cursorY = cursor.y / scale;
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

watch(
  chatMessages,
  () => {
    persistChatHistory();
  },
  { deep: true }
);

watch(proxyPort, (value) => {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }

  window.localStorage.setItem("keai.desktop-pet.proxy-port", String(value));
});

watch(
  [platforms, proxyPort],
  () => {
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
  filteredMemoryRecords,
  (records) => {
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
  filteredDocumentRecords,
  (records) => {
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
  selectedTimelineLog,
  (log) => {
    if (!log) {
      return;
    }

    timelinePreviewSection.value = getDefaultPreviewSection(log);
  },
  { immediate: true }
);

watch(
  selectedSession,
  (session) => {
    if (!session) {
      sessionOverlayLogId.value = null;
      return;
    }

    selectedSessionLogId.value = session.logs[0]?.id ?? null;
    sessionOverlayLogId.value = null;
  },
  { immediate: true }
);

watch(
  selectedSessionLog,
  (log) => {
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

onMounted(() => {
  chatMessages.value = loadChatHistory();
  currentSessionId.value = loadStoredSessionId();
  proxyPort.value = loadProxyPort();
  platforms.value = loadPlatforms();
  localRequestLogs.value = loadRequestLogs(platforms.value);
  void refreshOpenClawMessageLogs();
  void refreshStaffSnapshot();
  void refreshMemorySnapshot();
  void refreshDocumentSnapshot();
  void refreshTaskSnapshot();
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
  }, 15000);
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
  window.cancelAnimationFrame(rafId);
  window.cancelAnimationFrame(chatAnimationFrame);
  window.cancelAnimationFrame(panelAnimationFrame);
  window.cancelAnimationFrame(bubbleAnimationFrame);
  window.clearTimeout(idleTimer);
  window.clearInterval(cursorPassThroughTimer);
  window.clearInterval(gatewayMonitorTimer);
  window.clearInterval(runtimeLogTimer);
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
  <main ref="stage" class="desktop-pet-stage">
    <div class="desktop-pet-hint" :style="hintStyle">
      <span class="desktop-pet-hint-title">{{ activeAnimation.label }}</span>
      <p>{{ statusText }}</p>
    </div>

    <button
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
      v-show="isChatOpen || chatMotionValue > 0"
      ref="chatPanelRef"
      class="desktop-console-panel desktop-chat-window"
      :style="chatPanelStyle"
    >
      <header
        class="desktop-console-panel__header desktop-console-panel__dragbar desktop-chat-window__header"
        @pointerdown="handleChatDragStart"
      >
        <div class="desktop-chat-window__title">
          <p class="desktop-console-panel__eyebrow">OpenClaw</p>
          <strong>对话</strong>
        </div>
        <div class="desktop-console-panel__actions">
          <button
            class="desktop-chat-window__icon-button desktop-chat-window__icon-button--ghost"
            type="button"
            aria-label="新建会话"
            title="新会话"
            @click="handleNewConversation"
          >
            <svg viewBox="0 0 20 20" aria-hidden="true">
              <path d="M10 4v12" />
              <path d="M4 10h12" />
            </svg>
          </button>
          <button
            class="desktop-chat-window__icon-button desktop-chat-window__icon-button--ghost"
            type="button"
            aria-label="收起对话窗口"
            title="收起"
            @click="toggleChatPanel(false)"
          >
            <svg viewBox="0 0 20 20" aria-hidden="true">
              <path d="M5 10h10" />
            </svg>
          </button>
        </div>
      </header>

      <div class="desktop-console-body desktop-console-body--chat">
        <section class="assistant-column">
          <div ref="messageScrollerRef" class="desktop-chat-panel__messages">
            <article
              v-for="(message, index) in chatMessages"
              :key="message.id"
              class="chat-bubble"
              :class="[`chat-bubble--${message.role}`, `chat-bubble--${message.status}`]"
              :style="getBubbleStyle(index)"
            >
              <p>{{ message.text }}</p>
            </article>
          </div>

          <footer class="desktop-chat-panel__composer">
            <textarea
              v-model="chatInput"
              class="desktop-chat-panel__input"
              rows="3"
              placeholder="输入你想让 OpenClaw 帮你做的事"
              @keydown="handleComposerKeydown"
            />
            <button
              class="desktop-chat-window__icon-button"
              type="button"
              aria-label="发送消息"
              title="发送"
              :disabled="isSending || !chatInput.trim()"
              @click="submitChat"
            >
              <svg viewBox="0 0 20 20" aria-hidden="true">
                <path d="M4 10.5 15.5 5l-3 10-2.5-3-3 2z" />
              </svg>
            </button>
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
    >
      <header class="desktop-console-panel__header desktop-console-panel__dragbar" @pointerdown="handlePanelDragStart">
        <div v-if="activePanelMode === 'console'">
          <p class="desktop-console-panel__eyebrow">ClawPet Command Deck</p>
          <strong>运营控制台</strong>
          <p class="desktop-console-panel__intro">
            当前默认平台
            <span class="desktop-console-panel__platform">{{ activePlatform?.name ?? openClawDefaultPlatformName }}</span>
            ，这里已经扩展为平台、员工、记忆、文档、任务统一管理台。
          </p>
        </div>
        <div v-else>
          <p class="desktop-console-panel__eyebrow">ClawPet Command Deck</p>
          <strong>日志分析</strong>
          <p class="desktop-console-panel__intro">集中查看时间线、会话视图和失败分析，不再混入控制台导航。</p>
        </div>
        <div class="desktop-console-panel__actions">
          <button
            v-if="activePanelMode === 'console'"
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openLogAnalysis()"
          >
            日志分析
          </button>
          <button
            v-else
            class="desktop-console-panel__action desktop-console-panel__action--ghost"
            type="button"
            @click="openConsole('platforms')"
          >
            平台管理
          </button>
          <button class="desktop-console-panel__action" type="button" @click="toggleConsolePanel(false)">收起</button>
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

      <nav v-else class="desktop-console-nav">
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
                <div
                  class="platform-identity"
                  :class="[getPlatformPillClass(log.platformId), getPlatformIdentityToneClass(log.platformId, log.platformName)]"
                >
                  <span class="platform-identity__avatar">{{ getPlatformInitials(log.platformName) }}</span>
                  <span class="platform-identity__body">
                    <span class="platform-identity__eyebrow">
                      <span>{{ getPlatformOriginLabel(log.platformId) }}</span>
                      <span>{{ getPlatformMetaLabel(log.platformId, log.protocol) || "协议未知" }}</span>
                    </span>
                    <span class="platform-identity__name">{{ log.platformName }}</span>
                  </span>
                </div>
                <span>{{ formatTime(log.createdAt) }}</span>
              </div>
              <p>{{ summarizeLogText(log) }}</p>
              <small :data-status="isFailedLog(log) ? 'error' : 'success'">
                {{ log.method }} · {{ isFailedLog(log) ? "失败" : "成功" }} · {{ formatDuration(log.duration) }}
              </small>
            </button>
            <div v-if="timelineEntries.length === 0" class="empty-state">还没有调用记录，先去和桌宠聊两句吧。</div>
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
              <div class="detail-endpoint">{{ getLogRequestUrl(selectedTimelineLog) }}</div>
            </div>

            <div v-if="selectedTimelineLog.requestHeaders && Object.keys(selectedTimelineLog.requestHeaders).length > 0" class="detail-code">
              <div class="detail-code__header">
                <h4>请求头</h4>
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
                <div
                  class="platform-identity"
                  :class="[
                    getPlatformPillClass(session.logs[0]?.platformId ?? null),
                    getPlatformIdentityToneClass(session.logs[0]?.platformId ?? null, session.platformName)
                  ]"
                >
                  <span class="platform-identity__avatar">{{ getPlatformInitials(session.platformName) }}</span>
                  <span class="platform-identity__body">
                    <span class="platform-identity__eyebrow">
                      <span>{{ getPlatformOriginLabel(session.logs[0]?.platformId ?? null) }}</span>
                      <span>{{ getPlatformMetaLabel(session.logs[0]?.platformId ?? null, session.logs[0]?.protocol) || "协议未知" }}</span>
                    </span>
                    <span class="platform-identity__name">{{ session.platformName }}</span>
                  </span>
                </div>
                <span>{{ formatTime(session.lastAt) }}</span>
              </div>
              <p>{{ session.previewText }}</p>
              <small>
                {{ session.requestCount }} 次调用 · {{ session.failureCount }} 次失败 · {{ formatDuration(session.totalDuration) }}
              </small>
            </button>
            <div v-if="sessionSummaries.length === 0" class="empty-state">还没有形成会话记录。</div>
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
              <div class="detail-endpoint detail-endpoint--soft">{{ selectedSession.previewText }}</div>
            </div>

            <div class="detail-code session-output-card">
              <div class="detail-code__header">
                <h4>会话输出</h4>
              </div>
              <pre>{{ selectedSession.fullOutput }}</pre>
            </div>

            <div v-if="selectedSession.latestError" class="detail-code detail-code--danger">
              <div class="detail-code__header">
                <h4>最近一次失败</h4>
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
              <div class="detail-endpoint">{{ getLogRequestUrl(selectedSessionLog) }}</div>
            </div>

            <div v-if="selectedSessionLog.requestHeaders && Object.keys(selectedSessionLog.requestHeaders).length > 0" class="detail-code">
              <div class="detail-code__header">
                <h4>请求头</h4>
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
              <h3>平台管理</h3>
              <p>统一维护已接入的平台配置，并切换聊天窗口默认使用的目标。</p>
            </div>
            <div class="toolbar-actions">
              <button class="platform-tips-trigger" type="button" @click="showPlatformTips = !showPlatformTips">
                {{ showPlatformTips ? "收起说明" : "接入说明" }}
              </button>
              <button class="desktop-console-panel__action" type="button" @click="handleCreatePlatform">新增平台</button>
            </div>
          </header>

          <div v-if="showPlatformTips" class="platform-inline-note">
            <p>OpenAI 兼容协议通常使用 `/v1/chat/completions`；Anthropic 原生协议通常使用 `/v1/messages`。</p>
            <p>点击“新增平台”后，可直接从国外平台或国内平台预设中填充草稿，再微调模型名、密钥和路径。</p>
          </div>

          <section class="platform-preset-section">
            <div class="platform-preset-section__header">
              <div>
                <h4>已配置平台</h4>
                <p>默认平台会直接用于聊天窗口，请在这里统一维护。</p>
              </div>
            </div>

            <div class="platform-list platform-list--grid">
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
                <p class="platform-card__endpoint">{{ platform.baseUrl }}</p>
                <div class="platform-card__meta">
                  <span>前缀 {{ platform.pathPrefix }}</span>
                  <span>{{ activePlatformId === platform.id ? "默认平台" : "可切换" }}</span>
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
          </section>
        </section>
      </div>

      <div v-else-if="activeSection === 'staff'" class="desktop-console-body desktop-console-body--overview staff-layout">
        <section class="section-block overview-section">
          <header class="section-block__header">
            <div>
              <h3>员工总览</h3>
              <p>员工信息按 openclaw-control-center 的方式读取，展示角色定位、当前状态、正在处理什么、最近产出，以及是否在排班里。</p>
            </div>
          </header>

          <div class="staff-brief-grid">
            <article v-for="member in staffMembers" :key="member.agentId" class="staff-brief-card">
              <div class="staff-brief-head">
                <div class="staff-avatar">
                  <div class="staff-avatar__badge" :class="getStaffStatusClass(member)">
                    {{ getStaffInitials(member.displayName) }}
                  </div>
                </div>
                <div class="staff-brief-identity">
                  <strong>{{ member.displayName }}</strong>
                  <p>{{ getStaffRoleLabel(member) }}</p>
                  <div class="staff-chip-row">
                    <span class="staff-status-chip" :class="getStaffStatusClass(member)">{{ member.statusLabel }}</span>
                    <span class="staff-soft-chip">{{ member.scheduledLabel }}</span>
                  </div>
                </div>
              </div>

              <dl class="staff-brief-list">
                <div class="staff-brief-row">
                  <dt>当前状态</dt>
                  <dd>{{ member.statusLabel }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>{{ member.currentWorkLabel }}</dt>
                  <dd>{{ member.currentWork }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>最近产出</dt>
                  <dd>{{ member.recentOutput }}</dd>
                </div>
                <div class="staff-brief-row">
                  <dt>是否在排班里</dt>
                  <dd>{{ member.scheduledLabel }}</dd>
                </div>
              </dl>
            </article>
            <div v-if="staffMembers.length === 0" class="empty-state">暂无可显示的员工信息。请确认 `~/.openclaw/openclaw.json` 或运行时员工目录存在。</div>
          </div>
        </section>

        <section class="section-block compact-details">
          <h4 class="compact-details__summary">员工共同目标</h4>
          <div class="compact-details__body">
            <div class="mission-banner">{{ staffMissionStatement }}</div>
            <p class="compact-details__meta">来源：{{ staffSnapshotSourcePath || "未定位 openclaw.json" }}</p>
            <p class="compact-details__meta">{{ staffSnapshotDetail }}</p>
          </div>
        </section>

        <section class="section-block compact-details">
          <h4 class="compact-details__summary">员工配置明细</h4>
          <div class="compact-details__body">
            <div class="staff-config-table">
              <table>
                <thead>
                  <tr>
                    <th>名称</th>
                    <th>agentId</th>
                    <th>工作目录</th>
                    <th>职责焦点</th>
                    <th>模型</th>
                    <th>工具权限</th>
                    <th>状态</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="member in staffMembers" :key="`${member.agentId}-detail`">
                    <td>{{ member.displayName }}</td>
                    <td>{{ member.agentId }}</td>
                    <td>{{ member.workspace }}</td>
                    <td>{{ getStaffRoleLabel(member) }}</td>
                    <td>{{ member.model }}</td>
                    <td>{{ member.toolsProfile }}</td>
                    <td>{{ member.statusLabel }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'memory'" class="desktop-console-body desktop-console-body--overview">
        <section class="section-block overview-section">
          <header class="section-block__header section-block__header--compact">
            <div>
              <h3>记忆管理</h3>
              <p>参照 openclaw-control-center，直接浏览和编辑 OpenClaw 的真实记忆文件。</p>
            </div>
            <div class="toolbar-actions">
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshMemorySnapshot">
                刷新
              </button>
            </div>
          </header>

          <div class="platform-inline-note">
            <p>{{ memorySnapshotDetail }}</p>
            <p>来源：{{ memorySnapshotSourcePath || "未定位记忆工作区" }}</p>
          </div>

          <div class="management-workbench management-workbench--dense">
            <aside class="management-sidebar">
              <div class="management-sidebar__tools">
                <div class="management-sidebar__headline">
                  <strong>记忆目录</strong>
                  <small>{{ filteredMemoryRecords.length }} / {{ memoryRecords.length }} 条</small>
                </div>
                <div class="management-segmented">
                  <button
                    v-for="scope in memoryScopeOptions"
                    :key="scope.key"
                    class="management-segmented__item"
                    :class="{ active: activeMemoryScope === scope.key }"
                    type="button"
                    @click="activeMemoryScope = scope.key"
                  >
                    {{ scope.label }}
                  </button>
                </div>
                <input v-model="memoryFilterText" class="management-filter-input" type="text" placeholder="筛选标题、归属人、范围或摘要" />
              </div>

              <div class="management-nav">
                <button
                  v-for="record in filteredMemoryRecords"
                  :key="record.id"
                  class="management-nav-item"
                  :class="{ active: selectedMemoryRecord?.id === record.id }"
                  type="button"
                  @click="handleSelectMemory(record)"
                >
                  <div class="management-nav-item__topline">
                    <strong>{{ record.title }}</strong>
                    <span>{{ formatMemoryStatus(record.scope) }}</span>
                  </div>
                  <p>{{ record.summary }}</p>
                  <small>{{ record.scope }} · {{ record.relativePath }} · {{ formatTime(record.updatedAt) }}</small>
                </button>
                <div v-if="filteredMemoryRecords.length === 0" class="empty-state management-empty-state">当前筛选下没有记忆条目。</div>
              </div>
            </aside>

            <section class="management-editor">
              <div class="management-editor__header">
                <div>
                  <strong>{{ selectedMemoryId ? "编辑记忆文件" : "选择记忆文件" }}</strong>
                  <p>
                    {{
                      selectedMemoryRecord
                        ? `${selectedMemoryRecord.scope} · ${selectedMemoryRecord.relativePath} · 最近更新 ${formatTime(selectedMemoryRecord.updatedAt)}`
                        : "这里只展示 Main 和记忆工作区里的真实源文件。"
                    }}
                  </p>
                </div>
                <div class="management-editor__actions">
                  <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshMemorySnapshot">
                    重新读取
                  </button>
                </div>
              </div>

              <form class="management-editor__form" @submit.prevent="handleSaveMemory">
                <div class="management-form-grid management-form-grid--workbench">
                  <input v-model="memoryDraft.title" type="text" placeholder="记忆标题" readonly />
                  <input v-model="memoryDraft.owner" type="text" placeholder="归属员工" readonly />
                  <input v-model="memoryDraft.scope" type="text" placeholder="记忆分类" readonly />
                  <input v-model="memoryDraft.relativePath" type="text" placeholder="相对路径" readonly />
                  <input v-model="memoryDraft.sourcePath" type="text" placeholder="源文件路径" readonly />
                  <div class="management-editor__meta-card">
                    <span>当前来源</span>
                    <strong>{{ memoryEditorModeLabel }}</strong>
                    <small>{{ selectedMemoryId ? "保存会直接写回 OpenClaw 真实记忆文件" : "先从左侧选择记忆文件" }}</small>
                  </div>
                  <textarea v-model="memoryDraft.content" rows="12" placeholder="记忆文件内容" />
                </div>

                <div class="management-form-grid__actions management-form-grid__actions--editor">
                  <button class="desktop-console-panel__action" type="submit" :disabled="!memoryDraft.sourcePath">{{ memoryEditorModeLabel }}</button>
                </div>
              </form>
            </section>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'docs'" class="desktop-console-body desktop-console-body--overview">
        <section class="section-block overview-section">
          <header class="section-block__header section-block__header--compact">
            <div>
              <h3>文档管理</h3>
              <p>参照 openclaw-control-center，直接读取 Main 和各 agent 工作区里的核心文档。</p>
            </div>
            <div class="toolbar-actions">
              <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshDocumentSnapshot">
                刷新
              </button>
            </div>
          </header>

          <div class="management-workbench management-workbench--dense">
            <aside class="management-sidebar">
              <div class="management-sidebar__tools">
                <div class="management-sidebar__headline">
                  <strong>文档目录</strong>
                  <small>{{ filteredDocumentRecords.length }} / {{ documentRecords.length }} 份</small>
                </div>
                <div class="management-segmented">
                  <button
                    v-for="category in documentCategoryOptions"
                    :key="category.key"
                    class="management-segmented__item"
                    :class="{ active: activeDocumentCategory === category.key }"
                    type="button"
                    @click="activeDocumentCategory = category.key"
                  >
                    {{ category.label }}
                  </button>
                </div>
                <input
                  v-model="documentFilterText"
                  class="management-filter-input"
                  type="text"
                  placeholder="筛选标题、分类、负责人、来源或摘要"
                />
              </div>

              <div class="management-nav">
                <button
                  v-for="record in filteredDocumentRecords"
                  :key="record.id"
                  class="management-nav-item management-nav-item--document"
                  :class="{ active: selectedDocumentRecord?.id === record.id }"
                  type="button"
                  @click="handleSelectDocument(record)"
                >
                  <div class="management-nav-item__topline">
                    <strong>{{ record.title }}</strong>
                    <span class="management-nav-item__badge">{{ formatDocumentStatus(record.category) }}</span>
                  </div>
                  <p>{{ record.summary }}</p>
                  <small>{{ record.category }} · {{ record.relativePath }} · {{ formatTime(record.updatedAt) }}</small>
                </button>
                <div v-if="filteredDocumentRecords.length === 0" class="empty-state management-empty-state">当前筛选下没有文档。</div>
              </div>
            </aside>

            <section class="management-editor">
              <div class="management-editor__header">
                <div>
                  <strong>{{ selectedDocumentId ? "编辑核心文档" : "选择核心文档" }}</strong>
                  <p>
                    {{
                      selectedDocumentRecord
                        ? `${selectedDocumentRecord.category} · ${selectedDocumentRecord.relativePath} · 最近更新 ${formatTime(selectedDocumentRecord.updatedAt)}`
                        : "这里只展示 Main 和各 agent 工作区的核心文档文件。"
                    }}
                  </p>
                </div>
                <div class="management-editor__actions">
                  <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="refreshDocumentSnapshot">
                    重新读取
                  </button>
                </div>
              </div>

              <form class="management-editor__form" @submit.prevent="handleSaveDocument">
                <div class="management-form-grid management-form-grid--workbench">
                  <input v-model="documentDraft.title" type="text" placeholder="文档标题" readonly />
                  <input v-model="documentDraft.category" type="text" placeholder="文档分类" readonly />
                  <input v-model="documentDraft.owner" type="text" placeholder="负责人" readonly />
                  <input v-model="documentDraft.relativePath" type="text" placeholder="相对路径" readonly />
                  <input v-model="documentDraft.source" type="text" placeholder="来源路径" readonly />
                  <div class="management-editor__meta-card">
                    <span>当前来源</span>
                    <strong>{{ documentEditorModeLabel }}</strong>
                    <small>{{ selectedDocumentId ? "保存会直接写回 OpenClaw 真实文档文件" : "先从左侧选择文档文件" }}</small>
                  </div>
                  <textarea v-model="documentDraft.content" rows="12" placeholder="文档内容" />
                </div>

                <div class="management-form-grid__actions management-form-grid__actions--editor">
                  <button class="desktop-console-panel__action" type="submit" :disabled="!documentDraft.source">{{ documentEditorModeLabel }}</button>
                </div>
              </form>
            </section>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'tasks'" class="desktop-console-body desktop-console-body--overview">
        <section class="tasks-dashboard">
          <section class="section-block overview-section tasks-hero">
            <header class="section-block__header tasks-hero__header">
              <div>
                <h3>今日与下一批排程</h3>
                <p>直接读取 openclaw 的 `cron/jobs.json`，把真实调度任务按运行时间和启用状态集中展示。</p>
              </div>
              <div class="tasks-hero__badge">
                <span>下一截止</span>
                <strong>{{ nextTaskDueRecord?.nextRunAtMs ? formatTaskRelativeDueAt(nextTaskDueRecord.nextRunAtMs) : "暂无排程" }}</strong>
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
                <span>未标注代理</span>
                <strong>{{ taskBoardMetrics.unassigned }}</strong>
                <small>agentId 待确认</small>
              </article>
            </div>

            <div class="tasks-schedule-grid">
              <article v-for="group in taskScheduleCards" :key="group.id" class="tasks-schedule-card" :class="`tasks-schedule-card--${group.tone}`">
                <div class="tasks-schedule-card__header">
                  <div>
                    <strong>{{ group.title }}</strong>
                    <p>{{ group.subtitle }}</p>
                  </div>
                  <span>{{ group.records.length }} 条</span>
                </div>

                <div v-if="group.records.length" class="tasks-schedule-list">
                  <article v-for="record in group.records.slice(0, 3)" :key="record.id" class="tasks-schedule-item">
                    <div class="tasks-schedule-item__topline">
                      <strong>{{ record.name }}</strong>
                      <span class="task-status-pill" :class="getTaskStatusClass(record.statusKind)">{{ formatTaskStatus(record.statusKind) }}</span>
                    </div>
                    <p>{{ record.summary }}</p>
                    <div class="tasks-schedule-item__meta">
                      <span>Agent {{ record.agentId }}</span>
                      <span>Target {{ record.sessionTarget }}</span>
                      <span>{{ record.nextRunAtMs ? formatDueAt(record.nextRunAtMs) : "未提供下次执行时间" }}</span>
                    </div>
                  </article>
                </div>
                <div v-else class="empty-state">当前分组没有任务</div>
              </article>
            </div>

          </section>

          <section class="section-block overview-section tasks-board">
            <header class="section-block__header tasks-board__header">
              <div>
                <h3>Cron 执行看板</h3>
                <p>按状态分组展示当前任务、任务目的以及下一次执行时间，风格向 control-center 靠拢。</p>
              </div>
              <div class="tasks-board__summary">
                <span>待执行 {{ taskBoardMetrics.overdue }}</span>
                <strong>{{ taskStatusSummary.scheduled + taskStatusSummary.late }} 个启用任务</strong>
              </div>
            </header>

            <div class="tasks-board-groups">
              <section v-for="group in taskBoardGroups" :key="group.key" class="tasks-board-group">
                <div class="tasks-board-group__header">
                  <div>
                    <strong>{{ group.label }}</strong>
                    <p>{{ group.summary }}</p>
                  </div>
                  <span>{{ group.count }} 个任务</span>
                </div>

                <div v-if="group.records.length" class="tasks-board-list">
                  <article v-for="record in group.records" :key="record.id" class="tasks-board-card">
                    <div class="tasks-board-card__topline">
                      <div>
                        <strong>{{ record.name }}</strong>
                        <p>{{ record.summary }}</p>
                      </div>
                      <div class="tasks-board-card__tags">
                        <span class="task-priority-pill" :class="getTaskScheduleClass(record.scheduleKind, record.deleteAfterRun)">
                          {{ formatTaskScheduleKind(record.scheduleKind, record.deleteAfterRun) }}
                        </span>
                        <span class="task-status-pill" :class="getTaskStatusClass(record.statusKind)">{{ formatTaskStatus(record.statusKind) }}</span>
                      </div>
                    </div>
                    <div class="tasks-board-card__meta">
                      <span>Agent {{ record.agentId }}</span>
                      <span>Target {{ record.sessionTarget }}</span>
                      <span>{{ record.nextRunAtMs ? `下次: ${formatDueAt(record.nextRunAtMs)}` : "未提供下次执行时间" }}</span>
                      <span>{{ record.nextRunAtMs ? formatTaskRelativeDueAt(record.nextRunAtMs) : record.statusLabel }}</span>
                    </div>
                  </article>
                </div>
                <div v-else class="empty-state">当前分组没有任务</div>
              </section>
            </div>
          </section>
        </section>
      </div>

      <div class="desktop-console-panel__resize-handle" @pointerdown="handlePanelResizeStart" />
    </section>

    <div v-if="sessionOverlayLog" class="platform-modal-backdrop" @click.self="closeSessionLogOverlay">
      <section class="platform-modal session-log-overlay">
        <header class="platform-modal__header">
          <div>
            <strong>会话请求详情</strong>
            <p>{{ sessionOverlayLog.platformName }} · {{ sessionOverlayLog.method }} {{ sessionOverlayLog.path || sessionOverlayLog.endpoint }}</p>
          </div>
          <button class="platform-modal__close" type="button" @click="closeSessionLogOverlay">×</button>
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

    <div v-if="isPlatformModalOpen" class="platform-modal-backdrop" @click.self="handleCancelPlatformEdit">
      <section class="platform-modal">
        <header class="platform-modal__header">
          <div>
            <strong>{{ editingPlatformId ? "编辑平台" : "新增平台" }}</strong>
            <p>可先从预设平台快速填充，再补全基础协议和 URL。</p>
          </div>
          <button class="platform-modal__close" type="button" @click="handleCancelPlatformEdit">×</button>
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
              <select v-model="platformForm.protocol">
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

    <div
      v-if="contextMenu.visible"
      ref="contextMenuRef"
      class="desktop-context-menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
    >
      <button class="desktop-context-menu__item" type="button" @click="openChatPanel()">聊天</button>
      <button class="desktop-context-menu__item" type="button" @click="openConsole('platforms')">平台管理</button>
      <button class="desktop-context-menu__item" type="button" @click="openConsole('tasks')">任务管理</button>
      <button class="desktop-context-menu__item desktop-context-menu__item--danger" type="button" @click="handleQuitClick">
        退出
      </button>
    </div>
  </main>
</template>
