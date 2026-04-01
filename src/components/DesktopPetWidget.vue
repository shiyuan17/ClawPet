<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { usePetStore } from "../stores/pet";
import { createPetScene } from "../engine/createPetScene";
import { usePetSound } from "../composables/usePetSound";

const host = ref<HTMLDivElement | null>(null);
const petStore = usePetStore();
const sound = usePetSound();
const { action, activeSkin, availableSkins, energy, hint, intimacy, isDragging, level, mood, moodLabel } =
  storeToRefs(petStore);

const presence = computed(() => Math.round((energy.value * 0.4 + intimacy.value * 0.6) * 10) / 10);
const statChips = computed(() => [
  { label: "情绪", value: moodLabel.value },
  { label: "陪伴值", value: `${presence.value}` },
  { label: "动作", value: action.value }
]);

let scene: Awaited<ReturnType<typeof createPetScene>> | null = null;
let autonomyTimer = 0;
let pointerMoveListener: ((event: PointerEvent) => void) | null = null;
let lastWindowPointer = { x: 0, y: 0, t: 0 };
let previousUnlockedCount = petStore.unlockedSkinIds.length;

const utterances: Record<string, string> = {
  calm: "我在",
  happy: "继续互动呀",
  excited: "再来一次",
  sleepy: "让我缓缓",
  mischief: "追到你了",
  shy: "别一直看"
};

onMounted(async () => {
  if (!host.value) {
    return;
  }

  scene = await createPetScene({
    host: host.value,
    handlers: {
      onTap() {
        petStore.registerTap(1);
        sound.click();
      },
      onHover(elapsedMs) {
        petStore.registerHover(elapsedMs);
        if (elapsedMs > 260) {
          sound.hover();
        }
      },
      onDragStart() {
        petStore.registerDragStart();
        sound.drag();
      },
      onDragMove(speed) {
        petStore.registerDragMove(speed);
      },
      onDragEnd() {
        petStore.registerDragEnd();
      }
    }
  });

  pointerMoveListener = (event: PointerEvent) => {
    const now = performance.now();
    if (lastWindowPointer.t > 0) {
      const dx = event.clientX - lastWindowPointer.x;
      const dy = event.clientY - lastWindowPointer.y;
      const dt = Math.max(16, now - lastWindowPointer.t);
      const speed = Math.sqrt(dx * dx + dy * dy) / (dt / 16);
      if (!isDragging.value) {
        petStore.registerPointerDash(speed);
        if (speed > 20) {
          sound.dash();
        }
      }
    }
    lastWindowPointer = { x: event.clientX, y: event.clientY, t: now };
  };
  window.addEventListener("pointermove", pointerMoveListener, { passive: true });

  autonomyTimer = window.setInterval(() => {
    petStore.tickAutonomy(Date.now());
  }, 1400);

  scene.render({
    action: action.value,
    dragging: isDragging.value,
    energy: energy.value,
    intimacy: intimacy.value,
    mood: mood.value,
    skin: activeSkin.value
  });
});

watch(
  () => ({
    action: action.value,
    dragging: isDragging.value,
    energy: energy.value,
    intimacy: intimacy.value,
    mood: mood.value,
    skin: activeSkin.value
  }),
  (state) => {
    scene?.render(
      {
        action: state.action,
        dragging: state.dragging,
        energy: state.energy,
        intimacy: state.intimacy,
        mood: state.mood,
        skin: state.skin
      },
      utterances[state.mood]
    );
  },
  { immediate: true, deep: true }
);

watch(
  () => petStore.unlockedSkinIds.length,
  (nextCount) => {
    if (nextCount > previousUnlockedCount) {
      sound.unlock();
    }
    previousUnlockedCount = nextCount;
  }
);

onBeforeUnmount(() => {
  if (pointerMoveListener) {
    window.removeEventListener("pointermove", pointerMoveListener);
  }
  if (autonomyTimer) {
    window.clearInterval(autonomyTimer);
  }
  scene?.destroy();
});
</script>

<template>
  <section class="pet-shell" :style="{ '--pet-aura': activeSkin.aura }">
    <div class="pet-stage">
      <div ref="host" class="pixi-host" />
      <div class="pet-badge">Lv.{{ level }}</div>
      <div class="pet-growth">
        <div class="pet-growth-bar" :style="{ transform: `scaleX(${petStore.growthProgress})` }" />
      </div>
    </div>

    <aside class="pet-panel">
      <header class="panel-header">
        <div>
          <p class="eyebrow">Embeddable Companion</p>
          <h1>DragonClaw 桌宠系统</h1>
        </div>
        <div class="status-pill" :data-mood="mood">{{ moodLabel }}</div>
      </header>

      <p class="panel-copy">
        高互动反馈由 <strong>PixiJS</strong> 驱动表现层，<strong>GSAP</strong> 负责弹性动画，
        <strong>Pinia</strong> 管理成长、情绪和主动行为。
      </p>

      <div class="chip-grid">
        <div v-for="chip in statChips" :key="chip.label" class="chip">
          <span>{{ chip.label }}</span>
          <strong>{{ chip.value }}</strong>
        </div>
      </div>

      <div class="hint-card">
        <span class="hint-dot" />
        <p>{{ hint }}</p>
      </div>

      <div class="interaction-list">
        <div class="interaction-item">悬停：眼神跟随、轻微偏头、羞涩状态累积</div>
        <div class="interaction-item">点击：弹性挤压、表情强化、连击奖励</div>
        <div class="interaction-item">拖拽：抓取晃动、释放回弹、速度敏感反馈</div>
        <div class="interaction-item">快速移动：追光冲刺、好奇值抬升、音效增强</div>
        <div class="interaction-item">待机主动：眨眼、轻唱、打盹、呼吸浮动</div>
      </div>

      <div class="skin-section">
        <div class="section-title">
          <span>皮肤扩展</span>
          <strong>{{ availableSkins.length }}/3</strong>
        </div>
        <div class="skin-row">
          <button
            v-for="skin in availableSkins"
            :key="skin.id"
            class="skin-pill"
            :class="{ active: skin.id === activeSkin.id }"
            @click="petStore.setSkin(skin.id)"
          >
            {{ skin.name }}
          </button>
        </div>
      </div>
    </aside>
  </section>
</template>
