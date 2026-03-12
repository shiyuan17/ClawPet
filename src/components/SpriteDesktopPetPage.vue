<script setup lang="ts">
import { interpolate } from "remotion/no-react";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import smileBlinkConfig from "../images/animate/smile_blink/index.json";
import smileBlinkSprite from "../images/animate/smile_blink/sprite.png";
import stompFeetConfig from "../images/animate/stomp_feet/index.json";
import stompFeetSprite from "../images/animate/stomp_feet/sprite.png";
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

type AnimationName = "smile_blink" | "stomp_feet";
type ConsoleSection = "platforms" | "timeline" | "sessions" | "failures";

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
  logs: RequestLog[];
  previewText: string;
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

const animations: Record<AnimationName, AnimationDefinition> = {
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
  }
};

const actionTips: Record<AnimationName, string> = {
  smile_blink: "今天状态不错，适合放在页面右下角陪你工作。",
  stomp_feet: "你刚刚戳到它了，它正在跺脚表达情绪。"
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
const activeSection = ref<ConsoleSection>("platforms");
const isSending = ref(false);
const chatInput = ref("");
const chatMessages = ref<ChatMessage[]>([...defaultChatMessages]);
const chatMotionValue = ref(0);
const panelMotionValue = ref(0);
const bubbleMotionValue = ref(1);
const requestLogs = ref<RequestLog[]>([]);
const platforms = ref<PlatformConfig[]>([]);
const activePlatformId = ref<string | null>(null);
const isEditingPlatform = ref(false);
const editingPlatformId = ref<string | null>(null);
const platformForm = ref<PlatformDraft>(createPlatformDraft());
const showPlatformTips = ref(false);
const isPlatformModalOpen = ref(false);
const selectedFeaturedPreset = ref("");
const selectedMorePreset = ref("");
const selectedLogId = ref<string | null>(null);
const selectedSessionId = ref<string | null>(null);
const selectedFailureKey = ref<string | null>(null);
const currentSessionId = ref("");
const proxyPort = ref(5005);
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
const platformPresets = getPlatformPresets();
const featuredPlatformPresets = computed(() => platformPresets.filter((preset) => preset.featured));
const morePlatformPresets = computed(() => platformPresets.filter((preset) => !preset.featured));

const activeAnimation = computed(() => animations[currentAnimationName.value]);
const currentFrame = computed(() => activeAnimation.value.config.frames[currentFrameIndex.value]);
const activePlatform = computed(
  () =>
    platforms.value.find((platform) => platform.id === activePlatformId.value && platform.enabled) ||
    platforms.value.find((platform) => platform.enabled) ||
    platforms.value[0] ||
    null
);
const enabledPlatformCount = computed(() => platforms.value.filter((platform) => platform.enabled).length);
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
  const defaultWidth = Math.min(prefersWide ? 980 : 620, Math.max(prefersWide ? 720 : 420, viewportWidth - 32));
  const defaultHeight = Math.min(prefersWide ? 700 : 620, Math.max(420, viewportHeight - 32));
  const panelWidth =
    panelPlacement.value.mode === "manual" && panelPlacement.value.width > 0
      ? Math.min(Math.max(getPanelMinWidth(prefersWide), panelPlacement.value.width), viewportWidth - 32)
      : defaultWidth;
  const panelHeight =
    panelPlacement.value.mode === "manual" && panelPlacement.value.height > 0
      ? Math.min(Math.max(420, panelPlacement.value.height), viewportHeight - 32)
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
    panelPlacement.value.mode === "manual"
      ? Math.min(Math.max(16, panelPlacement.value.x), Math.max(16, viewportWidth - panelWidth - 16))
      : autoLeft;
  const top =
    panelPlacement.value.mode === "manual"
      ? Math.min(Math.max(16, panelPlacement.value.y), Math.max(16, viewportHeight - panelHeight - 16))
      : autoTop;
  const progress = panelMotionValue.value;
  const originX = canPlaceLeft ? "right" : canPlaceRight ? "left" : "center";

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

  return [
    { label: "启用平台", value: `${enabledPlatformCount.value}` },
    { label: "调用总数", value: `${requestLogs.value.length}` },
    { label: "失败请求", value: `${failures}` },
    { label: "平均耗时", value: `${averageDuration} ms` }
  ];
});
const timelineEntries = computed(() => requestLogs.value);
const sessionSummaries = computed<SessionSummary[]>(() => {
  const map = new Map<string, SessionSummary>();

  for (const log of requestLogs.value) {
    const current = map.get(log.sessionId);
    const preview = summarizeLogText(log);

    if (current) {
      current.lastAt = Math.max(current.lastAt, log.createdAt);
      current.requestCount += 1;
      current.failureCount += isFailedLog(log) ? 1 : 0;
      current.totalDuration += log.duration;
      current.logs.push(log);
      if (preview.length > current.previewText.length) {
        current.previewText = preview;
      }
    } else {
      map.set(log.sessionId, {
        id: log.sessionId,
        startedAt: log.createdAt,
        lastAt: log.createdAt,
        platformName: log.platformName,
        requestCount: 1,
        failureCount: isFailedLog(log) ? 1 : 0,
        totalDuration: log.duration,
        logs: [log],
        previewText: preview
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
const selectedSessionLog = computed(() => selectedSession.value?.logs[0] ?? null);
const selectedFailureLog = computed(() => selectedFailure.value?.logs[0] ?? null);

let rafId = 0;
let idleTimer = 0;
let animationStartedAt = 0;
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

type TauriWindowApi = {
  close: () => Promise<void> | void;
  destroy: () => Promise<void> | void;
  setIgnoreCursorEvents: (value: boolean) => Promise<void> | void;
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

function setAnimation(name: AnimationName) {
  currentAnimationName.value = name;
  currentFrameIndex.value = 0;
  animationStartedAt = performance.now();
  statusText.value = actionTips[name];
  window.clearTimeout(idleTimer);

  if (animations[name].loop) {
    queueIdleAction();
  }
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

  if (!isChatOpen.value) {
    isChatOpen.value = true;
    startChatAnimation();
  }

  statusText.value = "OpenClaw 对话窗口已打开。";
  startBubbleAnimation();
  scrollMessagesToBottom();
}

function toggleChatPanel(nextValue?: boolean) {
  const finalValue = nextValue ?? !isChatOpen.value;
  if (finalValue === isChatOpen.value) {
    return;
  }

  isChatOpen.value = finalValue;
  startChatAnimation();
  statusText.value = finalValue ? "对话窗口已打开。" : "对话窗口已收起。";
}

function openConsole(section: ConsoleSection) {
  activeSection.value = section;
  hideContextMenu();

  if (!isConsoleOpen.value) {
    if (panelPlacement.value.mode === "auto") {
      resetPanelPlacement();
    }
    isConsoleOpen.value = true;
    startPanelAnimation();
  }

  if (section === "platforms") {
    statusText.value = "平台管理已展开，可以新增、切换默认平台或修改接口配置。";
  } else if (section === "timeline") {
    statusText.value = "调用时间线已展开，最近请求会按时间倒序显示。";
  } else if (section === "sessions") {
    statusText.value = "会话视图已展开，适合回看一段连续请求。";
  } else {
    statusText.value = "失败分析已展开，我帮你把问题聚合到一起了。";
  }
}

function toggleConsolePanel(nextValue?: boolean) {
  const finalValue = nextValue ?? !isConsoleOpen.value;
  if (finalValue === isConsoleOpen.value) {
    return;
  }

  isConsoleOpen.value = finalValue;
  startPanelAnimation();
  statusText.value = finalValue ? "控制台面板已打开。" : "控制台已收起，我继续在这里陪你。";
}

function queueIdleAction() {
  window.clearTimeout(idleTimer);
  idleTimer = window.setTimeout(() => {
    if (!isDragging.value && currentAnimationName.value === "smile_blink") {
      setAnimation("stomp_feet");
    } else {
      queueIdleAction();
    }
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
    setAnimation("smile_blink");
  }

  rafId = window.requestAnimationFrame(tick);
}

function handlePetClick() {
  if (dragDistance.value > 6) {
    dragDistance.value = 0;
    return;
  }

  setAnimation("stomp_feet");
  openChatPanel();
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
  statusText.value = "位置记住了，继续待机陪伴。";
  queueIdleAction();
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

    panelPlacement.value = {
      ...panelPlacement.value,
      width: Math.min(panelPlacement.value.width, bounds.width - 32),
      height: Math.min(panelPlacement.value.height, bounds.height - 32),
      x: Math.min(Math.max(16, panelPlacement.value.x), Math.max(16, bounds.width - panelPlacement.value.width - 16)),
      y: Math.min(Math.max(16, panelPlacement.value.y), Math.max(16, bounds.height - panelPlacement.value.height - 16))
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

async function setWindowIgnoreCursorEvents(nextValue: boolean) {
  if (ignoreCursorEvents === nextValue) {
    return;
  }

  const tauriApi = getTauriApi();
  const currentWindow = tauriApi?.window?.getCurrentWindow?.();

  if (!currentWindow?.setIgnoreCursorEvents) {
    return;
  }

  await currentWindow.setIgnoreCursorEvents(nextValue);
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

function createRequestPayload(messages: OpenClawMessage[], platform: PlatformConfig) {
  if (platform.protocol === "anthropic") {
    const system = messages
      .filter((message) => message.role === "system")
      .map((message) => message.content)
      .join("\n\n");

    return {
      ...(platform.model.trim() ? { model: platform.model.trim() } : {}),
      max_tokens: 1024,
      ...(system ? { system } : {}),
      messages: messages
        .filter((message) => message.role !== "system")
        .map((message) => ({
          role: message.role === "assistant" ? "assistant" : "user",
          content: message.content
        }))
    };
  }

  return {
    messages
  };
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
  const requestBody = safeJson({ messages });
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
  statusText.value = "消息已经发给 OpenClaw 默认通道，正在等待回复。";
  startBubbleAnimation();
  scrollMessagesToBottom();

  try {
    const response = await sendOpenClawChat(messages);
    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = response.text;
      pendingMessage.status = "done";
    }

    requestLogs.value = appendRequestLog({
      sessionId: currentSessionId.value,
      platformId: "openclaw-default",
      platformName: "OpenClaw 默认通道",
      protocol: "openai",
      method: "POST",
      endpoint,
      requestBody,
      responseStatus: response.status ?? 200,
      responseBody: response.raw ?? response.text,
      duration: Math.round(performance.now() - startedAt)
    });

    statusText.value = "OpenClaw 已回复，你可以继续追问。";
  } catch (error) {
    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    const errorText = error instanceof Error ? error.message : "OpenClaw 调用失败，请稍后再试。";
    if (pendingMessage) {
      pendingMessage.text = errorText;
      pendingMessage.status = "error";
    }

    requestLogs.value = appendRequestLog({
      sessionId: currentSessionId.value,
      platformId: "openclaw-default",
      platformName: "OpenClaw 默认通道",
      protocol: "openai",
      method: "POST",
      endpoint,
      requestBody,
      responseStatus: 0,
      responseBody: "",
      duration: Math.round(performance.now() - startedAt),
      error: errorText
    });

    statusText.value = "这次没有连上目标平台，我已经把失败原因记到日志里了。";
    openConsole("failures");
  } finally {
    isSending.value = false;
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

function handlePresetSelect(kind: "featured" | "more") {
  const source = kind === "featured" ? featuredPlatformPresets.value : morePlatformPresets.value;
  const selectedName = kind === "featured" ? selectedFeaturedPreset.value : selectedMorePreset.value;
  const preset = source.find((item) => item.name === selectedName);
  if (!preset) {
    return;
  }

  handleUsePreset(preset);
  if (kind === "featured") {
    selectedFeaturedPreset.value = "";
  } else {
    selectedMorePreset.value = "";
  }
}

function handleCreatePlatform() {
  platformForm.value = createPlatformDraft();
  editingPlatformId.value = null;
  isEditingPlatform.value = true;
  isPlatformModalOpen.value = true;
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
  activePlatformId.value = loadActivePlatformId() ?? platforms.value.find((item) => item.enabled)?.id ?? platforms.value[0]?.id ?? null;
  statusText.value = `${target.name} 已删除。`;
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
    const fallback = platforms.value.find((item) => item.enabled && item.id !== platformId) ?? null;
    activePlatformId.value = fallback?.id ?? null;
    setActivePlatform(fallback?.id ?? null);
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
  const confirmed = window.confirm("确认清空全部调用日志吗？");
  if (!confirmed) {
    return;
  }

  requestLogs.value = clearRequestLogs();
  selectedLogId.value = null;
  selectedSessionId.value = null;
  selectedFailureKey.value = null;
  statusText.value = "调用日志已清空。";
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
    panelPlacement.value = {
      ...panelPlacement.value,
      width: Math.min(Math.max(getPanelMinWidth(prefersWide), nextWidth), bounds.width - 32),
      height: Math.min(Math.max(420, nextHeight), bounds.height - 32),
      x: Math.min(Math.max(16, panelPlacement.value.x), Math.max(16, bounds.width - Math.min(Math.max(getPanelMinWidth(prefersWide), nextWidth), bounds.width - 32) - 16)),
      y: Math.min(Math.max(16, panelPlacement.value.y), Math.max(16, bounds.height - Math.min(Math.max(420, nextHeight), bounds.height - 32) - 16))
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

function summarizeLogText(log: RequestLog) {
  const source = log.error || log.responseBody || log.requestBody;
  const compact = source.replace(/\s+/g, " ").trim();
  return compact.length > 120 ? `${compact.slice(0, 120)}...` : compact || "暂无摘要";
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
  if (name === "smile_blink" && !isDragging.value) {
    queueIdleAction();
  }
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
    panelPlacement.value = {
      ...panelPlacement.value,
      width: Math.min(Math.max(getPanelMinWidth(prefersWide), panelPlacement.value.width), bounds.width - 32),
      height: Math.min(Math.max(420, panelPlacement.value.height || 0), bounds.height - 32)
    };
  }
);

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
  activePlatform,
  (platform) => {
    if (platform && !activePlatformId.value) {
      activePlatformId.value = platform.id;
      setActivePlatform(platform.id);
    }
  },
  { immediate: true }
);

onMounted(() => {
  chatMessages.value = loadChatHistory();
  currentSessionId.value = loadStoredSessionId();
  proxyPort.value = loadProxyPort();
  platforms.value = loadPlatforms();
  requestLogs.value = loadRequestLogs();
  activePlatformId.value =
    loadActivePlatformId() ?? platforms.value.find((platform) => platform.enabled)?.id ?? platforms.value[0]?.id ?? null;
  void syncLocalProxyServer();
  centerPet();
  animationStartedAt = performance.now();
  rafId = window.requestAnimationFrame(tick);
  queueIdleAction();
  void syncCursorPassThrough();
  cursorPassThroughTimer = window.setInterval(() => {
    void syncCursorPassThrough();
  }, 120);
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
  window.cancelAnimationFrame(panelAnimationFrame);
  window.cancelAnimationFrame(bubbleAnimationFrame);
  window.clearTimeout(idleTimer);
  window.clearInterval(cursorPassThroughTimer);
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
      <header class="desktop-console-panel__header desktop-console-panel__dragbar" @pointerdown="handleChatDragStart">
        <div>
          <p class="desktop-console-panel__eyebrow">OpenClaw</p>
          <strong>对话窗口</strong>
        </div>
        <div class="desktop-console-panel__actions">
          <button class="desktop-console-panel__action" type="button" @click="toggleChatPanel(false)">收起</button>
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
              <span class="chat-bubble__role">{{ message.role === "user" ? "你" : "OpenClaw" }}</span>
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
              class="desktop-console-panel__action"
              type="button"
              :disabled="isSending || !chatInput.trim()"
              @click="submitChat"
            >
              {{ isSending ? "发送中..." : "发送" }}
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
        <div>
          <p class="desktop-console-panel__eyebrow">ClawPet Command Deck</p>
          <strong>运营控制台</strong>
          <p class="desktop-console-panel__intro">
            当前默认平台
            <span class="desktop-console-panel__platform">{{ activePlatform?.name ?? "未选择" }}</span>
            ，这里只负责平台管理和调用分析。
          </p>
        </div>
        <div class="desktop-console-panel__actions">
          <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="handleNewConversation">
            新会话
          </button>
          <button class="desktop-console-panel__action desktop-console-panel__action--ghost" type="button" @click="resetPanelPlacement">
            复位窗口
          </button>
          <button class="desktop-console-panel__action" type="button" @click="toggleConsolePanel(false)">收起</button>
        </div>
      </header>

      <div class="desktop-console-panel__metrics">
        <article v-for="metric in metrics" :key="metric.label" class="desktop-metric-card">
          <span>{{ metric.label }}</span>
          <strong>{{ metric.value }}</strong>
        </article>
      </div>

      <nav class="desktop-console-nav">
        <button
          v-for="item in [
            { id: 'platforms', label: '平台管理' },
            { id: 'timeline', label: '时间线' },
            { id: 'sessions', label: '会话视图' },
            { id: 'failures', label: '失败分析' }
          ]"
          :key="item.id"
          class="desktop-console-nav__item"
          :class="{ active: activeSection === item.id }"
          type="button"
          @click="openConsole(item.id as ConsoleSection)"
        >
          {{ item.label }}
        </button>
      </nav>

      <div v-if="activeSection === 'platforms'" class="desktop-console-body desktop-console-body--platforms">
        <section class="section-block section-block--platforms">
          <header class="section-block__header">
            <div>
              <h3>平台管理</h3>
              <p>预置主流 AI 平台，支持快速接入、编辑配置和切换默认目标。</p>
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
            <p>建议先从预置卡片生成草稿，再按你的模型名、密钥和网关路径做细调。</p>
          </div>

          <section class="platform-preset-section">
            <div class="platform-preset-section__header">
              <div>
                <h4>热门平台</h4>
                <p>从下拉列表选择最常用的平台并快速生成草稿。</p>
              </div>
            </div>

            <div class="platform-select-row">
              <select v-model="selectedFeaturedPreset" class="platform-select">
                <option value="">选择热门平台</option>
                <option v-for="preset in featuredPlatformPresets" :key="preset.name" :value="preset.name">
                  {{ preset.name }} · {{ preset.protocol.toUpperCase() }}
                </option>
              </select>
              <button
                class="desktop-console-panel__action"
                type="button"
                :disabled="!selectedFeaturedPreset"
                @click="handlePresetSelect('featured')"
              >
                添加
              </button>
            </div>
          </section>

          <section class="platform-preset-section">
            <div class="platform-preset-section__header">
              <div>
                <h4>更多可选平台</h4>
                <p>补齐常见国内外模型供应商，按需从列表中添加。</p>
              </div>
            </div>

            <div class="platform-select-row">
              <select v-model="selectedMorePreset" class="platform-select">
                <option value="">选择更多平台</option>
                <option v-for="preset in morePlatformPresets" :key="preset.name" :value="preset.name">
                  {{ preset.name }} · {{ preset.protocol.toUpperCase() }}
                </option>
              </select>
              <button
                class="desktop-console-panel__action"
                type="button"
                :disabled="!selectedMorePreset"
                @click="handlePresetSelect('more')"
              >
                添加
              </button>
            </div>
          </section>

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

        <section class="section-block platform-editor-block">
          <header class="section-block__header">
            <div>
              <h3>配置提示</h3>
              <p>平台卡片已缩小展示；点击“新增平台”或“编辑”会弹出独立配置窗口。</p>
            </div>
          </header>

          <div class="platform-help platform-help--compact">
            <p>弹窗中的配置项已改成和 cc-look 一致的四项：平台名称、协议类型、API Base URL、路径前缀。</p>
            <p>本地代理会优先使用你配置的路径前缀，例如 `/openai`、`/deepseek`。</p>
          </div>
        </section>
      </div>

      <div v-else-if="activeSection === 'timeline'" class="desktop-console-body desktop-console-body--split">
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
              :class="{ active: selectedTimelineLog?.id === log.id }"
              type="button"
              @click="selectedLogId = log.id"
            >
              <div class="log-card__headline">
                <strong>{{ log.platformName }}</strong>
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
              <p>选中左侧条目后，可直接查看请求体和响应体。</p>
            </div>
          </header>

          <template v-if="selectedTimelineLog">
            <div class="detail-meta">
              <span>{{ selectedTimelineLog.platformName }}</span>
              <span>{{ selectedTimelineLog.endpoint }}</span>
              <span>{{ selectedTimelineLog.responseStatus || "未返回状态" }}</span>
            </div>
            <div class="detail-code">
              <h4>Request</h4>
              <pre>{{ selectedTimelineLog.requestBody }}</pre>
            </div>
            <div class="detail-code">
              <h4>Response</h4>
              <pre>{{ selectedTimelineLog.error || selectedTimelineLog.responseBody || "暂无响应体" }}</pre>
            </div>
          </template>
          <div v-else class="empty-state">暂无详情</div>
        </aside>
      </div>

      <div v-else-if="activeSection === 'sessions'" class="desktop-console-body desktop-console-body--split">
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
              :class="{ active: selectedSession?.id === session.id }"
              type="button"
              @click="selectedSessionId = session.id"
            >
              <div class="session-card__headline">
                <strong>{{ session.platformName }}</strong>
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
              <h3>会话摘要</h3>
              <p>这里展示当前会话里的第一条关键记录，方便快速定位。</p>
            </div>
          </header>

          <template v-if="selectedSession && selectedSessionLog">
            <div class="detail-meta">
              <span>开始 {{ formatTime(selectedSession.startedAt) }}</span>
              <span>最近 {{ formatTime(selectedSession.lastAt) }}</span>
              <span>{{ selectedSession.platformName }}</span>
            </div>
            <div class="detail-code">
              <h4>会话摘要</h4>
              <pre>{{ selectedSession.previewText }}</pre>
            </div>
            <div class="detail-code">
              <h4>关键记录</h4>
              <pre>{{ selectedSessionLog.error || selectedSessionLog.responseBody || selectedSessionLog.requestBody }}</pre>
            </div>
          </template>
          <div v-else class="empty-state">暂无会话详情</div>
        </aside>
      </div>

      <div v-else class="desktop-console-body desktop-console-body--split">
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
              <p>{{ failure.platformNames.join(" / ") }}</p>
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
              <span>{{ selectedFailure.platformNames.join(" / ") }}</span>
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
      </div>

      <div class="desktop-console-panel__resize-handle" @pointerdown="handlePanelResizeStart" />
    </section>

    <div v-if="isPlatformModalOpen" class="platform-modal-backdrop" @click.self="handleCancelPlatformEdit">
      <section class="platform-modal">
        <header class="platform-modal__header">
          <div>
            <strong>编辑平台</strong>
            <p>填写基础协议和 URL 即可接入本地代理。</p>
          </div>
          <button class="platform-modal__close" type="button" @click="handleCancelPlatformEdit">×</button>
        </header>

        <form class="platform-modal__form" @submit.prevent="handleSavePlatform">
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
      <button class="desktop-context-menu__item desktop-context-menu__item--danger" type="button" @click="handleQuitClick">
        退出
      </button>
    </div>
  </main>
</template>
