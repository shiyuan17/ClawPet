<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import ChatAppPage from "./components/ChatAppPage.vue";
import LoginPage from "./components/LoginPage.vue";

const LOGIN_STORAGE_KEY = "dragonclaw.auth.session";
const SIDEBAR_SETTINGS_APPEARANCE_STORAGE_KEY = "DragonClaw.desktop-pet.sidebar-settings.appearance";
const SIDEBAR_THEME_PRESET_STORAGE_KEY = "DragonClaw.desktop-pet.sidebar-theme.preset";
const SKIP_LOGIN_PAGE = true;
const isAuthenticated = ref(false);
type SidebarSettingsAppearance = "system" | "light" | "dark";
type SidebarThemeMode = "day" | "night";
type SidebarThemePreset = "elegant" | "frosted" | "pure-white";

let cleanupSystemThemeModeListener: (() => void) | null = null;

function hasPersistedSession() {
  if (typeof window === "undefined") {
    return false;
  }

  try {
    const sessionText = window.localStorage.getItem(LOGIN_STORAGE_KEY)?.trim();
    return Boolean(sessionText);
  } catch {
    return false;
  }
}

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

function normalizeSidebarSettingsAppearance(raw: string | null): SidebarSettingsAppearance {
  if (raw === "light" || raw === "dark" || raw === "system") {
    return raw;
  }
  return "system";
}

function normalizeSidebarThemePreset(raw: string | null): SidebarThemePreset {
  if (raw === "elegant" || raw === "frosted" || raw === "pure-white") {
    return raw;
  }
  return "elegant";
}

function resolveSystemThemeMode(): SidebarThemeMode {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return "day";
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "night" : "day";
}

function applySidebarThemeToDocument(mode: SidebarThemeMode, preset: SidebarThemePreset) {
  if (typeof document === "undefined") {
    return;
  }
  const root = document.documentElement;
  const resolved = mode === "night" ? "dark" : "light";
  root.setAttribute("data-app-theme-mode", mode);
  root.setAttribute("data-app-theme-resolved", resolved);
  root.setAttribute("data-app-theme-preset", preset);
  root.style.colorScheme = resolved;
}

function applyThemeFromStorage() {
  const appearance = normalizeSidebarSettingsAppearance(safeStorageGet(SIDEBAR_SETTINGS_APPEARANCE_STORAGE_KEY));
  const preset = normalizeSidebarThemePreset(safeStorageGet(SIDEBAR_THEME_PRESET_STORAGE_KEY));

  const nextMode: SidebarThemeMode =
    appearance === "system" ? resolveSystemThemeMode() : appearance === "dark" ? "night" : "day";

  applySidebarThemeToDocument(nextMode, preset);
}

function stopSystemThemeModeListener() {
  cleanupSystemThemeModeListener?.();
  cleanupSystemThemeModeListener = null;
}

function ensureSystemThemeModeListener() {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function" || cleanupSystemThemeModeListener) {
    return;
  }

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = () => {
    const appearance = normalizeSidebarSettingsAppearance(safeStorageGet(SIDEBAR_SETTINGS_APPEARANCE_STORAGE_KEY));
    if (appearance !== "system") {
      return;
    }
    const preset = normalizeSidebarThemePreset(safeStorageGet(SIDEBAR_THEME_PRESET_STORAGE_KEY));
    applySidebarThemeToDocument(mediaQuery.matches ? "night" : "day", preset);
  };

  if (typeof mediaQuery.addEventListener === "function") {
    mediaQuery.addEventListener("change", handler);
    cleanupSystemThemeModeListener = () => mediaQuery.removeEventListener("change", handler);
  } else {
    mediaQuery.addListener(handler);
    cleanupSystemThemeModeListener = () => mediaQuery.removeListener(handler);
  }
}

function handleLoginSuccess(payload: { phone: string; sessionToken: string }) {
  if (typeof window !== "undefined") {
    const sessionSnapshot = {
      phone: payload.phone,
      sessionToken: payload.sessionToken,
      loggedInAt: Date.now()
    };
    try {
      window.localStorage.setItem(LOGIN_STORAGE_KEY, JSON.stringify(sessionSnapshot));
    } catch {
      // ignore localStorage errors and still allow this runtime session to continue
    }
  }

  isAuthenticated.value = true;
}

applyThemeFromStorage();

onMounted(() => {
  applyThemeFromStorage();
  ensureSystemThemeModeListener();

  if (SKIP_LOGIN_PAGE) {
    isAuthenticated.value = true;
    return;
  }
  isAuthenticated.value = hasPersistedSession();
});

onBeforeUnmount(() => {
  stopSystemThemeModeListener();
});
</script>

<template>
  <ChatAppPage v-if="SKIP_LOGIN_PAGE || isAuthenticated" />
  <LoginPage v-else @login-success="handleLoginSuccess" />
</template>
