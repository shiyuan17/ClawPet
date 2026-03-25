<script setup lang="ts">
import { defineAsyncComponent, onMounted, ref } from "vue";
import ChatAppPage from "./components/ChatAppPage.vue";
import LoginPage from "./components/LoginPage.vue";

const SpriteDesktopPetPage = defineAsyncComponent(() => import("./components/SpriteDesktopPetPage.vue"));
const LOGIN_STORAGE_KEY = "clawpet.auth.session";
const SKIP_LOGIN_PAGE = true;

function resolveLegacyConsoleMode() {
  if (typeof window === "undefined") {
    return false;
  }

  const tauriWindow = window as Window & { __CLAWPET_CONSOLE_MODE?: boolean };
  if (tauriWindow.__CLAWPET_CONSOLE_MODE) {
    return true;
  }

  try {
    const url = new URL(window.location.href);
    const windowParam = url.searchParams.get("window")?.trim().toLowerCase();
    const legacyParam = url.searchParams.get("legacy_console")?.trim().toLowerCase();
    return windowParam === "console" || legacyParam === "1" || legacyParam === "true";
  } catch {
    return false;
  }
}

const isLegacyConsoleMode = resolveLegacyConsoleMode();
const isAuthenticated = ref(false);

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

onMounted(() => {
  if (SKIP_LOGIN_PAGE) {
    isAuthenticated.value = true;
    return;
  }
  isAuthenticated.value = hasPersistedSession();
});
</script>

<template>
  <SpriteDesktopPetPage v-if="isLegacyConsoleMode" />
  <ChatAppPage v-else-if="SKIP_LOGIN_PAGE || isAuthenticated" />
  <LoginPage v-else @login-success="handleLoginSuccess" />
</template>
