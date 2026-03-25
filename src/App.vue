<script setup lang="ts">
import { defineAsyncComponent } from "vue";
import ChatAppPage from "./components/ChatAppPage.vue";

const SpriteDesktopPetPage = defineAsyncComponent(() => import("./components/SpriteDesktopPetPage.vue"));

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
</script>

<template>
  <SpriteDesktopPetPage v-if="isLegacyConsoleMode" />
  <ChatAppPage v-else />
</template>
