<script setup lang="ts">
export type ControlIconName =
  | "close"
  | "copy"
  | "nav-chat"
  | "nav-dashboard"
  | "nav-market"
  | "nav-recruitment"
  | "nav-skills"
  | "nav-tasks"
  | "theme"
  | "proxy"
  | "logs"
  | "settings"
  | "web"
  | "search"
  | "plus"
  | "chevron-down"
  | "expand"
  | "trash"
  | "history"
  | "runtime"
  | "chat"
  | "tool"
  | "edit"
  | "bolt"
  | "send"
  | "eye"
  | "eye-off";

withDefaults(
  defineProps<{
    name: ControlIconName;
  }>(),
  {
    name: "settings"
  }
);
</script>

<template>
  <svg class="control-icon" viewBox="0 0 24 24" fill="none" aria-hidden="true" focusable="false">
    <g v-if="name === 'close'">
      <path d="M7 7l10 10M17 7 7 17" />
    </g>

    <g v-else-if="name === 'copy'">
      <rect x="9" y="9" width="11" height="11" rx="2.2" />
      <path d="M7.5 15.2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h7.2a2 2 0 0 1 2 2v1.5" />
    </g>

    <g v-else-if="name === 'nav-chat'">
      <path d="M7 5.5h10A3 3 0 0 1 20 8.5v6a3 3 0 0 1-3 3h-6.4L6 20.8v-3.3H7a3 3 0 0 1-3-3v-6a3 3 0 0 1 3-3Z" />
      <circle class="accent-fill" cx="16.8" cy="8.8" r="1.1" />
    </g>

    <g v-else-if="name === 'nav-dashboard'">
      <rect x="4" y="4" width="7" height="7" rx="1.5" />
      <rect x="13" y="4" width="7" height="5" rx="1.5" />
      <rect x="4" y="13" width="7" height="7" rx="1.5" />
      <rect x="13" y="11" width="7" height="9" rx="1.5" />
      <path class="accent-stroke" d="M14.4 14.8h4.2" />
    </g>

    <g v-else-if="name === 'nav-market'">
      <path d="M4.2 9h15.6l-1 10.2H5.2L4.2 9Z" />
      <path d="M8 9V7.1a4 4 0 0 1 8 0V9" />
      <path class="accent-stroke" d="M10 12h.01M14 12h.01" />
    </g>

    <g v-else-if="name === 'nav-recruitment'">
      <circle cx="10.2" cy="10.2" r="5.6" />
      <path d="m14.4 14.4 5 5" />
      <path class="accent-stroke" d="M8.2 11.2c.9-.9 2.9-.9 3.8 0" />
      <circle class="accent-fill" cx="10.1" cy="8.3" r="1" />
    </g>

    <g v-else-if="name === 'nav-skills'">
      <path d="m5 19 9.2-9.2" />
      <path d="m11.8 7.6 4.6 4.6" />
      <path d="m14.2 4 1.1-2 1.1 2 2 1.1-2 1.1-1.1 2-1.1-2-2-1.1Z" />
      <path class="accent-stroke" d="M4.6 18.2h2.6" />
    </g>

    <g v-else-if="name === 'nav-tasks'">
      <rect x="6" y="4.5" width="12" height="15" rx="2" />
      <path d="M9 4.5h6" />
      <path d="M10 10h5M10 14h5" />
      <path class="accent-stroke" d="m8 9.8 1 1 1.3-1.4M8 13.8l1 1 1.3-1.4" />
    </g>

    <g v-else-if="name === 'theme'">
      <path d="M12 4a8 8 0 1 0 0 16h1.1a2.4 2.4 0 1 0 0-4.8h-1a2.2 2.2 0 1 1 0-4.4h3.5A4.4 4.4 0 0 0 20 6.4 8 8 0 0 0 12 4Z" />
      <circle class="accent-fill" cx="8" cy="8.8" r="1.1" />
    </g>

    <g v-else-if="name === 'proxy'">
      <rect x="4" y="8" width="16" height="11" rx="2" />
      <path d="M8 8V6.6a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2V8" />
      <path class="accent-stroke" d="M9 12.5h6" />
    </g>

    <g v-else-if="name === 'logs'">
      <path d="M7 3.8h8l4 4v12.4a1.8 1.8 0 0 1-1.8 1.8H7a1.8 1.8 0 0 1-1.8-1.8V5.6A1.8 1.8 0 0 1 7 3.8Z" />
      <path d="M15 3.8V8h4" />
      <path d="M9 12h6M9 15.2h4.8" />
      <path class="accent-stroke" d="m9 18.2 2.1-2.1 1.8 1.8 2.5-2.8" />
    </g>

    <g v-else-if="name === 'settings'">
      <circle cx="12" cy="12" r="3.2" />
      <path d="M12 3.2v2.1M12 18.7v2.1M3.2 12h2.1M18.7 12h2.1M5.9 5.9l1.5 1.5M16.6 16.6l1.5 1.5M18.1 5.9l-1.5 1.5M7.4 16.6l-1.5 1.5" />
      <circle class="accent-fill" cx="18.9" cy="12" r="0.9" />
    </g>

    <g v-else-if="name === 'web'">
      <rect x="3.5" y="5" width="17" height="14" rx="2.2" />
      <path d="M3.5 9h17" />
      <circle class="accent-fill" cx="7" cy="7" r="0.95" />
      <circle cx="10" cy="7" r="0.95" />
    </g>

    <g v-else-if="name === 'search'">
      <circle cx="10.2" cy="10.2" r="5.8" />
      <path d="m14.7 14.7 4.8 4.8" />
      <circle class="accent-fill" cx="10.2" cy="10.2" r="1" />
    </g>

    <g v-else-if="name === 'plus'">
      <path d="M12 5v14M5 12h14" />
    </g>

    <g v-else-if="name === 'chevron-down'">
      <path d="m7 10 5 5 5-5" />
    </g>

    <g v-else-if="name === 'expand'">
      <path d="M14 4h6v6" />
      <path d="m20 4-7 7" />
      <path d="M10 20H4v-6" />
      <path d="m4 20 7-7" />
    </g>

    <g v-else-if="name === 'trash'">
      <path d="M4.8 6h14.4" />
      <path d="M8.7 6V4.5a1.5 1.5 0 0 1 1.5-1.5h3.6a1.5 1.5 0 0 1 1.5 1.5V6" />
      <path d="m6.2 6 1 13a2 2 0 0 0 2 1.8h5.6a2 2 0 0 0 2-1.8l1-13" />
      <path class="accent-stroke" d="M10 10.2v7M14 10.2v7" />
    </g>

    <g v-else-if="name === 'history'">
      <circle cx="12" cy="12" r="8" />
      <path d="M12 8v4.6l3 1.8" />
      <path class="accent-stroke" d="M8.2 5.6H5.8" />
    </g>

    <g v-else-if="name === 'runtime'">
      <rect x="6" y="4.5" width="12" height="15" rx="2" />
      <path d="M9 4.5h6" />
      <path d="m8.2 15.7 2.6-2.6 2.2 2.2 2.8-3.2" />
      <path class="accent-stroke" d="M8.2 10.2h7.6" />
    </g>

    <g v-else-if="name === 'chat'">
      <path d="M7 5.5h10A3 3 0 0 1 20 8.5v6a3 3 0 0 1-3 3H9.5L6 20.6v-3.1H7a3 3 0 0 1-3-3v-6a3 3 0 0 1 3-3Z" />
      <path class="accent-stroke" d="M9 11h.01M12 11h.01M15 11h.01" />
    </g>

    <g v-else-if="name === 'tool'">
      <path d="M14.7 4.3a4 4 0 0 0 4.8 4.8L15.4 13l-2.4-2.4-6.3 6.3a1.7 1.7 0 1 1-2.4-2.4l6.3-6.3L8.2 5.8l3.9-4.1a4 4 0 0 0 2.6 2.6Z" />
      <circle class="accent-fill" cx="18.4" cy="5.7" r="0.9" />
    </g>

    <g v-else-if="name === 'edit'">
      <path d="M4.5 19.5h4l9.7-9.7a1.8 1.8 0 0 0 0-2.6l-1.4-1.4a1.8 1.8 0 0 0-2.6 0L4.5 15.5v4Z" />
      <path class="accent-stroke" d="m12.6 7.4 4 4" />
    </g>

    <g v-else-if="name === 'bolt'">
      <path d="M13.2 2.8 6.2 12h4.2l-1.2 9.2 8.6-11.2h-4.4z" />
      <path class="accent-stroke" d="M9.4 12h3.1" />
    </g>

    <g v-else-if="name === 'send'">
      <path d="M12 19V5" />
      <path d="m7 10 5-5 5 5" />
      <path class="accent-stroke" d="M12 19h.01" />
    </g>

    <g v-else-if="name === 'eye'">
      <path d="M2.8 12s3.4-5.8 9.2-5.8 9.2 5.8 9.2 5.8-3.4 5.8-9.2 5.8-9.2-5.8-9.2-5.8Z" />
      <circle class="accent-fill" cx="12" cy="12" r="2.2" />
    </g>

    <g v-else-if="name === 'eye-off'">
      <path d="M2.8 12s3.4-5.8 9.2-5.8c2.1 0 3.8.7 5.2 1.7" />
      <path d="M21.2 12s-3.4 5.8-9.2 5.8c-2.1 0-3.9-.8-5.3-1.8" />
      <circle cx="12" cy="12" r="2.2" />
      <path d="M4 4l16 16" />
    </g>
  </svg>
</template>

<style scoped>
.control-icon {
  width: 1em;
  height: 1em;
  stroke: currentColor;
  stroke-width: 1.9;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.control-icon :is(path, circle, rect, line, polyline, ellipse) {
  vector-effect: non-scaling-stroke;
}

.accent-stroke {
  stroke: var(--control-icon-accent, #35b87d);
}

.accent-fill {
  fill: var(--control-icon-accent, #35b87d);
  stroke: none;
}
</style>
