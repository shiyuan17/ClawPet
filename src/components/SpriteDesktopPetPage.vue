<script setup lang="ts">
import { interpolate } from "remotion/no-react";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import smileBlinkConfig from "../images/animate/smile_blink/index.json";
import smileBlinkSprite from "../images/animate/smile_blink/sprite.png";
import stompFeetConfig from "../images/animate/stomp_feet/index.json";
import stompFeetSprite from "../images/animate/stomp_feet/sprite.png";
import { sendOpenClawChat, type OpenClawMessage } from "../services/openclaw";

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
const isSending = ref(false);
const chatInput = ref("");
const chatMessages = ref<ChatMessage[]>([...defaultChatMessages]);
const panelMotionValue = ref(0);
const bubbleMotionValue = ref(1);

const viewportSize = 280;
const autoplayDelayMs = 9000;
const playbackRate = 3;

const activeAnimation = computed(() => animations[currentAnimationName.value]);
const currentFrame = computed(() => activeAnimation.value.config.frames[currentFrameIndex.value]);
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
  const bubbleWidth = Math.min(320, viewportWidth - 40);
  const petCenterX = petPosition.value.x + viewportSize / 2;
  const preferredLeft = petCenterX - bubbleWidth / 2;
  const left = Math.min(Math.max(16, preferredLeft), Math.max(16, viewportWidth - bubbleWidth - 16));
  const topAbovePet = petPosition.value.y - 96;
  const topBelowPet = petPosition.value.y + viewportSize + 12;
  const top = topAbovePet >= 16 ? topAbovePet : Math.min(topBelowPet, Math.max(16, viewportHeight - 84));

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
const panelStyle = computed(() => {
  const bounds = stage.value?.getBoundingClientRect();
  const viewportWidth = bounds?.width ?? (typeof window === "undefined" ? 360 : window.innerWidth);
  const viewportHeight = bounds?.height ?? (typeof window === "undefined" ? 640 : window.innerHeight);
  const panelWidth = Math.min(380, Math.max(300, viewportWidth - 32));
  const panelHeight = Math.min(460, Math.max(360, viewportHeight - 32));
  const gap = 18;
  const petLeft = petPosition.value.x;
  const petRight = petPosition.value.x + viewportSize;
  const petCenterY = petPosition.value.y + viewportSize / 2;
  const leftSpace = petLeft - gap - 16;
  const rightSpace = viewportWidth - petRight - gap - 16;
  const canPlaceLeft = leftSpace >= panelWidth;
  const canPlaceRight = rightSpace >= panelWidth;
  const left = canPlaceLeft
    ? petLeft - panelWidth - gap
    : canPlaceRight
      ? petRight + gap
      : Math.min(Math.max(16, viewportWidth - panelWidth - 16), Math.max(16, petLeft - panelWidth * 0.5));
  const top = canPlaceLeft || canPlaceRight
    ? Math.min(Math.max(16, petCenterY - panelHeight / 2), Math.max(16, viewportHeight - panelHeight - 16))
    : Math.min(
        Math.max(16, petPosition.value.y - panelHeight - gap),
        Math.max(16, viewportHeight - panelHeight - viewportSize - gap - 16)
      );
  const progress = panelMotionValue.value;
  const originX = canPlaceLeft ? "right" : canPlaceRight ? "left" : "center";

  return {
    width: `${panelWidth}px`,
    left: `${left}px`,
    top: `${top}px`,
    maxHeight: `${panelHeight}px`,
    opacity: `${interpolate(progress, [0, 1], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: easeOutCubic
    })}`,
    transform: `translateY(${interpolate(progress, [0, 1], [20, 0])}px) scale(${interpolate(progress, [0, 1], [0.94, 1])})`,
    transformOrigin: `${originX} center`
  };
});

let rafId = 0;
let idleTimer = 0;
let animationStartedAt = 0;
let dragPointerId: number | null = null;
let dragStart = { x: 0, y: 0, petX: 0, petY: 0 };
let windowPointerMoveListener: ((event: PointerEvent) => void) | null = null;
let windowPointerUpListener: ((event: PointerEvent) => void) | null = null;
let cursorPassThroughTimer = 0;
let ignoreCursorEvents = false;
let panelAnimationFrame = 0;
let panelAnimationStartedAt = 0;
let bubbleAnimationFrame = 0;
let bubbleAnimationStartedAt = 0;

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

function persistChatHistory() {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }

  try {
    window.localStorage.setItem(chatStorageKey, JSON.stringify(getStableChatMessages(chatMessages.value)));
  } catch {
    // Ignore storage errors so chat remains usable even in restricted environments.
  }
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

  panelMotionValue.value = isChatOpen.value ? motion : 1 - motion;

  const shouldContinue = isChatOpen.value ? panelMotionValue.value < 0.995 : panelMotionValue.value > 0.005;
  if (shouldContinue) {
    panelAnimationFrame = window.requestAnimationFrame(animatePanel);
    return;
  }

  panelMotionValue.value = isChatOpen.value ? 1 : 0;
  panelAnimationFrame = 0;
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

function toggleChatPanel(nextValue?: boolean) {
  const finalValue = nextValue ?? !isChatOpen.value;
  if (finalValue === isChatOpen.value) {
    return;
  }

  isChatOpen.value = finalValue;
  startPanelAnimation();

  if (finalValue) {
    statusText.value = "OpenClaw 聊天窗口已打开，直接输入就可以开始。";
    startBubbleAnimation();
    scrollMessagesToBottom();
  } else {
    statusText.value = "聊天窗口已收起，我继续在这里陪你。";
  }
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
  hideContextMenu();
  toggleChatPanel(true);
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
}

function hideContextMenu() {
  contextMenu.value.visible = false;
}

function getTauriApi() {
  return (window as Window & { __TAURI__?: TauriNamespace }).__TAURI__;
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

  if (currentWindow) {
    if (currentWindow.close) {
      await currentWindow.close();
      return;
    }
  }

  window.close();
}

function handleEscape(event: KeyboardEvent) {
  if (event.key !== "Escape" || !isWindowActive.value) {
    return;
  }

  hideContextMenu();
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
  const menuWidth = 148;
  const menuHeight = 56;
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
    (!event.target.closest(".desktop-context-menu") && !event.target.closest(".desktop-chat-panel"))
  ) {
    hideContextMenu();
  }
}

async function submitChat() {
  const text = chatInput.value.trim();
  if (!text || isSending.value) {
    return;
  }

  const pendingId = createMessageId("assistant");
  const conversationHistory = [...openClawMessages.value];

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
  statusText.value = "消息已经发给 OpenClaw，正在等待回复。";
  startBubbleAnimation();
  scrollMessagesToBottom();

  try {
    const response = await sendOpenClawChat([
      {
        role: "system",
        content: "你是桌宠里的 OpenClaw 助手，请使用简洁自然的中文回复。"
      },
      ...conversationHistory,
      {
        role: "user",
        content: text
      }
    ]);
    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = response.text;
      pendingMessage.status = "done";
    }
    statusText.value = "OpenClaw 已回复，你可以继续追问。";
  } catch (error) {
    const pendingMessage = chatMessages.value.find((message) => message.id === pendingId);
    if (pendingMessage) {
      pendingMessage.text = error instanceof Error ? error.message : "OpenClaw 调用失败，请稍后再试。";
      pendingMessage.status = "error";
    }
    statusText.value = "这次没有连上 OpenClaw，我把错误信息显示在气泡里了。";
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
  const panelRect = chatPanelRef.value?.getBoundingClientRect();

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

  await setWindowIgnoreCursorEvents(!(isInPet || isInMenu || isInPanel));
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

onMounted(() => {
  chatMessages.value = loadChatHistory();
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
      v-show="isChatOpen || panelMotionValue > 0"
      ref="chatPanelRef"
      class="desktop-chat-panel"
      :style="panelStyle"
    >
      <header class="desktop-chat-panel__header">
        <div>
          <strong>OpenClaw</strong>
          <p>点击宠物即可唤起对话，返回内容会以文字气泡显示。</p>
        </div>
        <button class="desktop-chat-panel__close" type="button" @click="toggleChatPanel(false)">收起</button>
      </header>

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
          class="desktop-chat-panel__send"
          type="button"
          :disabled="isSending || !chatInput.trim()"
          @click="submitChat"
        >
          {{ isSending ? "发送中..." : "发送" }}
        </button>
      </footer>
    </section>

    <div
      v-if="contextMenu.visible"
      ref="contextMenuRef"
      class="desktop-context-menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
    >
      <button class="desktop-context-menu__item" type="button" @click="handleQuitClick">退出程序</button>
    </div>
  </main>
</template>
