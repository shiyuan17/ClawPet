<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";
import appLogoUrl from "../../images/xia-logo.png";

type TauriInvoke = (command: string, args?: Record<string, unknown>) => Promise<unknown>;
type TauriWindowApi = {
  close?: () => Promise<void> | void;
  minimize?: () => Promise<void> | void;
  maximize?: () => Promise<void> | void;
  unmaximize?: () => Promise<void> | void;
  toggleMaximize?: () => Promise<void> | void;
  isMaximized?: () => Promise<boolean> | boolean;
};
type TauriNamespace = {
  core?: {
    invoke?: TauriInvoke;
  };
  window?: {
    getCurrentWindow?: () => TauriWindowApi;
  };
};

type SmsSendResponse = {
  detail?: string;
  cooldownSeconds?: number;
};

type SmsVerifyResponse = {
  detail?: string;
  sessionToken?: string;
};

const emit = defineEmits<{
  (event: "login-success", payload: { phone: string; sessionToken: string }): void;
}>();

const phone = ref("");
const code = ref("");
const agreed = ref(true);
const sending = ref(false);
const loggingIn = ref(false);
const countdown = ref(0);
const noticeText = ref("");
const noticeTone = ref<"error" | "success" | "">("");

let countdownTimer: number | null = null;

const isPhoneValid = computed(() => /^1[3-9]\d{9}$/.test(phone.value));
const isCodeValid = computed(() => /^\d{6}$/.test(code.value));
const canSendCode = computed(() => isPhoneValid.value && !sending.value && countdown.value <= 0);
const canLogin = computed(() => isPhoneValid.value && isCodeValid.value && agreed.value && !loggingIn.value);
const sendCodeLabel = computed(() => (countdown.value > 0 ? `${countdown.value}s` : sending.value ? "发送中..." : "获取验证码"));

function toErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") {
    return error.message;
  }
  return "请求失败，请稍后重试。";
}

function setNotice(tone: "error" | "success", text: string) {
  noticeTone.value = tone;
  noticeText.value = text;
}

function getTauriNamespace(): TauriNamespace | null {
  if (typeof window === "undefined") {
    return null;
  }

  const runtime = window as Window & { __TAURI__?: TauriNamespace };
  return runtime.__TAURI__ ?? null;
}

function getTauriInvoke(): TauriInvoke | null {
  return getTauriNamespace()?.core?.invoke ?? null;
}

function getTauriWindow(): TauriWindowApi | null {
  return getTauriNamespace()?.window?.getCurrentWindow?.() ?? null;
}

function sanitizePhoneInput() {
  phone.value = phone.value.replace(/\D/g, "").slice(0, 11);
}

function sanitizeCodeInput() {
  code.value = code.value.replace(/\D/g, "").slice(0, 6);
}

function stopCountdownTimer() {
  if (countdownTimer !== null) {
    window.clearInterval(countdownTimer);
    countdownTimer = null;
  }
}

function startCountdown(seconds: number) {
  stopCountdownTimer();
  countdown.value = Math.max(0, Math.floor(seconds));
  if (countdown.value <= 0) {
    return;
  }
  countdownTimer = window.setInterval(() => {
    if (countdown.value <= 1) {
      countdown.value = 0;
      stopCountdownTimer();
      return;
    }
    countdown.value -= 1;
  }, 1000);
}

async function handleSendCode() {
  if (!canSendCode.value) {
    return;
  }

  const invoke = getTauriInvoke();
  if (!invoke) {
    setNotice("error", "当前环境不支持短信发送，请在桌面客户端中操作。");
    return;
  }

  sending.value = true;
  noticeText.value = "";
  noticeTone.value = "";

  try {
    const result = (await invoke("send_sms_code", { phone: phone.value })) as SmsSendResponse;
    const cooldownSeconds =
      typeof result?.cooldownSeconds === "number" && Number.isFinite(result.cooldownSeconds) && result.cooldownSeconds > 0
        ? result.cooldownSeconds
        : 60;
    startCountdown(cooldownSeconds);
    setNotice("success", result?.detail?.trim() || "验证码已发送，请注意查收短信。");
  } catch (error) {
    setNotice("error", toErrorMessage(error));
  } finally {
    sending.value = false;
  }
}

async function handleLogin() {
  if (!canLogin.value) {
    return;
  }

  const invoke = getTauriInvoke();
  if (!invoke) {
    setNotice("error", "当前环境不支持手机验证码登录，请在桌面客户端中操作。");
    return;
  }

  loggingIn.value = true;
  noticeText.value = "";
  noticeTone.value = "";

  try {
    const result = (await invoke("verify_sms_code", {
      phone: phone.value,
      code: code.value
    })) as SmsVerifyResponse;
    const sessionToken = result?.sessionToken?.trim();
    if (!sessionToken) {
      throw new Error("登录失败，请重新获取验证码后再试。");
    }
    setNotice("success", result?.detail?.trim() || "登录成功，正在进入工作台。");
    emit("login-success", { phone: phone.value, sessionToken });
  } catch (error) {
    setNotice("error", toErrorMessage(error));
  } finally {
    loggingIn.value = false;
  }
}

async function handleWindowClose() {
  const tauriWindow = getTauriWindow();
  if (tauriWindow?.close) {
    try {
      await tauriWindow.close();
      return;
    } catch {
      // fall through
    }
  }

  const invoke = getTauriInvoke();
  if (!invoke) {
    return;
  }
  try {
    await invoke("quit_app");
  } catch {
    // ignore
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
    // ignore
  }
}

async function handleWindowToggleMaximize() {
  const tauriWindow = getTauriWindow();
  if (!tauriWindow) {
    return;
  }

  try {
    if (tauriWindow.toggleMaximize) {
      await tauriWindow.toggleMaximize();
      return;
    }
    if (tauriWindow.isMaximized && tauriWindow.maximize && tauriWindow.unmaximize) {
      const isMaximized = await tauriWindow.isMaximized();
      if (isMaximized) {
        await tauriWindow.unmaximize();
      } else {
        await tauriWindow.maximize();
      }
    }
  } catch {
    // ignore
  }
}

async function handleWindowDrag(event: MouseEvent) {
  if (event.button !== 0) {
    return;
  }
  const invoke = getTauriInvoke();
  if (!invoke) {
    return;
  }
  try {
    await invoke("start_main_window_drag");
  } catch {
    // ignore
  }
}

onBeforeUnmount(() => {
  stopCountdownTimer();
});
</script>

<template>
  <main class="login-shell">
    <section class="login-card">
      <header class="login-titlebar" data-tauri-drag-region @mousedown="handleWindowDrag">
        <div class="window-controls">
          <button class="window-control is-close" type="button" aria-label="关闭窗口" @click.stop="handleWindowClose" />
          <button class="window-control is-min" type="button" aria-label="最小化窗口" @click.stop="handleWindowMinimize" />
          <button class="window-control is-max" type="button" aria-label="最大化窗口" @click.stop="handleWindowToggleMaximize" />
        </div>
      </header>

      <div class="login-content">
        <aside class="brand-panel">
          <header class="brand-header">
            <div class="brand-logo-wrap">
              <img :src="appLogoUrl" alt="ClawPet Logo" />
            </div>
            <div class="brand-title-row">
              <strong>ClawPet</strong>
              <span class="brand-badge">AI 协作工作台</span>
            </div>
          </header>
          <h2>
            让 AI 在一条线程里
            <br />
            规划、执行、回报
          </h2>
          <p class="brand-subtitle">一个工作台接住 Agent、IM 渠道和定时任务，不再只是聊天。</p>
          <ul class="brand-list">
            <li>
              <span>01</span>
              <div>
                <strong>线程式回报</strong>
                <p>进度、工具调用和结果回到同一条会话。</p>
              </div>
            </li>
            <li>
              <span>02</span>
              <div>
                <strong>接入你的 IM</strong>
                <p>接入 IM 渠道，让 AI 助手直接在线程协作。</p>
              </div>
            </li>
            <li>
              <span>03</span>
              <div>
                <strong>沉淀成自动化</strong>
                <p>把一次成功执行转成可重复运行的任务。</p>
              </div>
            </li>
          </ul>
        </aside>

        <section class="form-panel">
          <span class="form-badge">手机验证码</span>
          <h1>登录</h1>
          <p class="form-subtitle">输入手机号和验证码，继续进入你的工作台。</p>

          <label for="phone">手机号</label>
          <input
            id="phone"
            v-model="phone"
            type="text"
            inputmode="numeric"
            autocomplete="tel"
            placeholder="请输入手机号"
            @input="sanitizePhoneInput"
          />

          <label for="code">验证码</label>
          <div class="code-row">
            <input
              id="code"
              v-model="code"
              type="text"
              inputmode="numeric"
              autocomplete="one-time-code"
              placeholder="请输入验证码"
              @input="sanitizeCodeInput"
            />
            <button type="button" :disabled="!canSendCode" @click="handleSendCode">{{ sendCodeLabel }}</button>
          </div>

          <button class="submit-btn" type="button" :disabled="!canLogin" @click="handleLogin">
            {{ loggingIn ? "登录中..." : "登录" }}
          </button>

          <label class="agreement">
            <input v-model="agreed" type="checkbox" />
            <span>已阅读并同意 <a href="javascript:void(0)">《用户协议》</a> 和 <a href="javascript:void(0)">《隐私政策》</a></span>
          </label>
          <p class="hint">首次登录会为你自动创建默认工作区。</p>
          <p v-if="noticeText" class="notice" :class="noticeTone === 'error' ? 'is-error' : 'is-success'">{{ noticeText }}</p>
        </section>
      </div>
    </section>
  </main>
</template>

<style scoped>
.login-shell {
  --theme-accent: #f35a1e;
  --theme-accent-soft: rgba(243, 90, 30, 0.14);
  --theme-border: rgba(39, 36, 44, 0.08);
  --theme-text: #2f2d33;
  --theme-muted: rgba(72, 69, 78, 0.58);
  position: relative;
  width: 100%;
  min-height: 100%;
  padding: clamp(14px, 2.8vw, 24px);
  display: grid;
  place-items: center;
  overflow: auto;
  font-family: "Avenir Next", "PingFang SC", "Hiragino Sans GB", sans-serif;
  background: transparent;
}

.login-card {
  position: relative;
  z-index: 1;
  width: min(1040px, 100%);
  border-radius: 24px;
  border: 1px solid var(--theme-border);
  box-shadow: 0 12px 28px rgba(40, 37, 46, 0.1);
  background: rgba(255, 255, 255, 0.92);
  overflow: hidden;
  backdrop-filter: blur(4px);
}

.login-titlebar {
  height: 38px;
  display: flex;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid rgba(39, 36, 44, 0.08);
  cursor: grab;
  user-select: none;
  -webkit-user-select: none;
  -webkit-app-region: drag;
}

.login-titlebar:active {
  cursor: grabbing;
}

.window-controls {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.window-control {
  width: 12px;
  height: 12px;
  border: 0;
  border-radius: 999px;
  cursor: pointer;
  opacity: 0.92;
  transition:
    transform 0.15s ease,
    opacity 0.15s ease,
    filter 0.15s ease;
}

.window-control:hover {
  transform: scale(1.04);
  opacity: 1;
  filter: saturate(1.08);
}

.window-control.is-close {
  background: #ef6a5a;
}

.window-control.is-min {
  background: #e8b43f;
}

.window-control.is-max {
  background: #5eb768;
}

.login-content {
  display: grid;
  grid-template-columns: minmax(360px, 1.35fr) minmax(330px, 1fr);
}

.brand-panel {
  padding: clamp(20px, 2.8vw, 34px);
  border-right: 1px solid rgba(39, 36, 44, 0.08);
}

.brand-header {
  display: flex;
  align-items: center;
  gap: 9px;
  min-height: 34px;
}

.brand-title-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.brand-title-row strong {
  font-size: 16px;
  font-weight: 700;
  color: var(--theme-text);
  line-height: 1.1;
  letter-spacing: 0.01em;
}

.brand-logo-wrap {
  width: 34px;
  height: 34px;
  border-radius: 10px;
  display: grid;
  place-items: center;
  border: 1px solid rgba(39, 36, 44, 0.08);
  background: #fff;
  overflow: hidden;
}

.brand-logo-wrap img {
  width: 80%;
  height: 80%;
  object-fit: contain;
}

.brand-badge {
  display: inline-flex;
  padding: 5px 10px;
  border-radius: 999px;
  font-size: 11px;
  line-height: 1.15;
  color: var(--theme-accent);
  background: var(--theme-accent-soft);
}

.brand-panel h2 {
  margin: 14px 0 10px;
  font-size: clamp(22px, 2.2vw, 34px);
  line-height: 1.24;
  letter-spacing: 0.012em;
  color: var(--theme-text);
}

.brand-subtitle {
  margin: 0;
  font-size: 13px;
  line-height: 1.72;
  letter-spacing: 0.01em;
  color: var(--theme-muted);
}

.brand-list {
  margin: 18px 0 0;
  padding: 0;
  list-style: none;
}

.brand-list li {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  padding: 10px 0;
  border-bottom: 1px solid rgba(39, 36, 44, 0.08);
}

.brand-list li span {
  flex: 0 0 30px;
  width: 30px;
  height: 30px;
  border-radius: 999px;
  display: grid;
  place-items: center;
  font-size: 11px;
  font-weight: 600;
  color: var(--theme-accent);
  border: 1px solid rgba(39, 36, 44, 0.08);
  background: rgba(255, 255, 255, 0.92);
}

.brand-list li strong {
  font-size: 14px;
  line-height: 1.3;
  letter-spacing: 0.01em;
  color: var(--theme-text);
}

.brand-list li p {
  margin: 4px 0 0;
  font-size: 12px;
  line-height: 1.65;
  letter-spacing: 0.008em;
  color: rgba(72, 69, 78, 0.56);
}

.form-panel {
  display: grid;
  align-content: center;
  gap: 8px;
  padding: clamp(20px, 2.8vw, 34px);
}

.form-badge {
  justify-self: start;
  padding: 5px 10px;
  border-radius: 999px;
  font-size: 11px;
  color: var(--theme-accent);
  background: var(--theme-accent-soft);
}

.form-panel h1 {
  margin: 0;
  font-size: clamp(20px, 2vw, 30px);
  line-height: 1;
  letter-spacing: 0.01em;
  color: var(--theme-text);
}

.form-subtitle {
  margin: 0 0 3px;
  font-size: 13px;
  line-height: 1.68;
  letter-spacing: 0.008em;
  color: var(--theme-muted);
}

.form-panel label {
  font-size: 13px;
  color: rgba(52, 49, 58, 0.74);
}

.form-panel input[type="text"] {
  width: 100%;
  height: 42px;
  padding: 0 12px;
  border-radius: 12px;
  border: 1px solid rgba(39, 36, 44, 0.1);
  background: rgba(255, 255, 255, 0.9);
  font-size: 14px;
  color: var(--theme-text);
  outline: none;
  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    background-color 0.2s ease;
}

.form-panel input[type="text"]:focus {
  border-color: rgba(243, 90, 30, 0.56);
  box-shadow: 0 0 0 3px rgba(243, 90, 30, 0.16);
  background: #fff;
}

.code-row {
  display: grid;
  grid-template-columns: 1fr 112px;
  gap: 8px;
}

.code-row button {
  height: 42px;
  border: 1px solid rgba(39, 36, 44, 0.1);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.88);
  color: rgba(72, 69, 78, 0.58);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.code-row button:disabled {
  cursor: not-allowed;
  opacity: 0.62;
}

.code-row button:not(:disabled):hover {
  border-color: rgba(243, 90, 30, 0.32);
  color: var(--theme-accent);
  background: rgba(255, 255, 255, 0.94);
}

.submit-btn {
  margin-top: 2px;
  height: 44px;
  border: 0;
  border-radius: 12px;
  background: linear-gradient(180deg, #ff7442, #ef4f14);
  color: #fff;
  font-size: 16px;
  font-weight: 700;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    opacity 0.2s ease;
}

.submit-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 10px 20px rgba(239, 79, 20, 0.24);
}

.submit-btn:disabled {
  cursor: not-allowed;
  opacity: 0.52;
}

.agreement {
  margin-top: 2px;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 12px;
  line-height: 1.5;
  color: rgba(72, 69, 78, 0.64);
}

.agreement input {
  margin-top: 3px;
  accent-color: #f35a1e;
}

.agreement a {
  color: var(--theme-accent);
  text-decoration: none;
}

.hint {
  margin: 0;
  font-size: 12px;
  color: rgba(72, 69, 78, 0.5);
}

.notice {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
}

.notice.is-success {
  color: #2e8a58;
}

.notice.is-error {
  color: #d3532f;
}

@media (max-width: 980px) {
  .login-shell {
    align-items: start;
  }

  .login-card {
    border-radius: 18px;
  }

  .login-content {
    grid-template-columns: 1fr;
    border-radius: 26px;
  }

  .brand-panel {
    border-right: 0;
    border-bottom: 1px solid rgba(39, 36, 44, 0.08);
  }

  .brand-panel h2 {
    font-size: clamp(20px, 7vw, 30px);
  }

  .brand-subtitle {
    font-size: 12px;
  }

  .brand-list li strong {
    font-size: 13px;
  }

  .brand-list li p {
    font-size: 12px;
  }

  .form-subtitle {
    font-size: 12px;
  }

  .form-panel label {
    font-size: 12px;
  }

  .form-panel input[type="text"],
  .code-row button,
  .submit-btn {
    height: 40px;
    font-size: 13px;
  }
}

@media (max-width: 560px) {
  .login-shell {
    padding: 12px;
  }

  .login-card {
    border-radius: 16px;
  }

  .login-titlebar {
    height: 34px;
  }

  .code-row {
    grid-template-columns: 1fr 94px;
  }
}
</style>
