import { computed, ref } from "vue";
import { defineStore } from "pinia";

export type PetMood = "calm" | "happy" | "excited" | "sleepy" | "mischief" | "shy";
export type PetAction =
  | "idle"
  | "hover"
  | "tap"
  | "drag"
  | "dash"
  | "blink"
  | "sing"
  | "float"
  | "nap";

export interface SkinDefinition {
  id: string;
  name: string;
  shellTop: number;
  shellMid: number;
  shellBottom: number;
  blush: number;
  accent: number;
  glow: number;
  aura: string;
}

const skins: SkinDefinition[] = [
  {
    id: "sunrise",
    name: "晨曦",
    shellTop: 0xfffcf1,
    shellMid: 0xffd894,
    shellBottom: 0xff9a58,
    blush: 0xff89b2,
    accent: 0xff7f32,
    glow: 0xffe3b2,
    aura: "linear-gradient(135deg, rgba(255,237,199,.9), rgba(255,140,86,.65))"
  },
  {
    id: "mint",
    name: "薄荷汽水",
    shellTop: 0xf2fff8,
    shellMid: 0xaaf7d6,
    shellBottom: 0x64d5be,
    blush: 0xffb2d1,
    accent: 0x28b7a4,
    glow: 0xbafbed,
    aura: "linear-gradient(135deg, rgba(240,255,248,.88), rgba(102,225,197,.6))"
  },
  {
    id: "violet",
    name: "夜空莓果",
    shellTop: 0xfff7ff,
    shellMid: 0xd1b9ff,
    shellBottom: 0x8f6af8,
    blush: 0xff9fbd,
    accent: 0x6d43ed,
    glow: 0xcfbeff,
    aura: "linear-gradient(135deg, rgba(255,245,255,.9), rgba(151,119,255,.62))"
  }
];

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

export const usePetStore = defineStore("pet", () => {
  const mood = ref<PetMood>("calm");
  const action = ref<PetAction>("idle");
  const level = ref(1);
  const xp = ref(16);
  const intimacy = ref(22);
  const energy = ref(88);
  const curiosity = ref(35);
  const unlockedSkinIds = ref<string[]>(["sunrise"]);
  const activeSkinId = ref("sunrise");
  const interactionChain = ref(0);
  const hoverCharge = ref(0);
  const isDragging = ref(false);
  const lastInteractionAt = ref(Date.now());
  const hint = ref("摸摸我，我会记住你的节奏");

  const availableSkins = computed(() =>
    skins.filter((skin) => unlockedSkinIds.value.includes(skin.id))
  );
  const activeSkin = computed(
    () => skins.find((skin) => skin.id === activeSkinId.value) ?? skins[0]
  );
  const growthProgress = computed(() => xp.value / 100);
  const moodLabel = computed(() => {
    const labels: Record<PetMood, string> = {
      calm: "安静陪伴",
      happy: "开心冒泡",
      excited: "超想互动",
      sleepy: "打个小盹",
      mischief: "调皮试探",
      shy: "被你逗到了"
    };
    return labels[mood.value];
  });

  function updateMood(nextMood: PetMood, nextHint?: string) {
    mood.value = nextMood;
    if (nextHint) {
      hint.value = nextHint;
    }
  }

  function gainBond(amount: number) {
    intimacy.value = clamp(intimacy.value + amount, 0, 100);
    xp.value += amount * 1.4;
    while (xp.value >= 100) {
      xp.value -= 100;
      level.value += 1;
      hint.value = `成长到 Lv.${level.value}，解锁了新的表达能力`;
      if (level.value === 2 && !unlockedSkinIds.value.includes("mint")) {
        unlockedSkinIds.value.push("mint");
      }
      if (level.value === 4 && !unlockedSkinIds.value.includes("violet")) {
        unlockedSkinIds.value.push("violet");
      }
    }
  }

  function setSkin(id: string) {
    if (!unlockedSkinIds.value.includes(id)) {
      return;
    }
    activeSkinId.value = id;
    updateMood("happy", `换上了「${skins.find((skin) => skin.id === id)?.name}」皮肤`);
  }

  function registerHover(durationMs: number) {
    hoverCharge.value = clamp(hoverCharge.value + durationMs / 1200, 0, 100);
    curiosity.value = clamp(curiosity.value + 0.4, 0, 100);
    action.value = "hover";
    energy.value = clamp(energy.value - 0.2, 10, 100);
    lastInteractionAt.value = Date.now();

    if (hoverCharge.value > 18) {
      updateMood("shy", "一直盯着我看，我会害羞地偏头");
    } else {
      updateMood("happy", "有目光跟着我，我会更有精神");
    }
  }

  function registerTap(intensity = 1) {
    interactionChain.value += 1;
    action.value = "tap";
    gainBond(4 * intensity);
    energy.value = clamp(energy.value - 1.5 + intensity, 0, 100);
    curiosity.value = clamp(curiosity.value + 3, 0, 100);
    lastInteractionAt.value = Date.now();

    if (interactionChain.value >= 5) {
      updateMood("excited", "连击命中，我准备给你一套夸张反馈");
      gainBond(8);
      interactionChain.value = 0;
    } else {
      updateMood("happy", "这一下戳得刚刚好");
    }
  }

  function registerDragStart() {
    isDragging.value = true;
    action.value = "drag";
    updateMood("mischief", "被抓住啦，我会顺着你的手势晃动");
    gainBond(2);
    lastInteractionAt.value = Date.now();
  }

  function registerDragMove(speed: number) {
    curiosity.value = clamp(curiosity.value + speed * 0.02, 0, 100);
    energy.value = clamp(energy.value - speed * 0.004, 0, 100);
    if (speed > 18) {
      updateMood("excited", "飞起来了，尾迹和音效会更明显");
    }
  }

  function registerDragEnd() {
    isDragging.value = false;
    action.value = "float";
    updateMood("happy", "放手后我会轻轻弹回去");
    lastInteractionAt.value = Date.now();
  }

  function registerPointerDash(speed: number) {
    if (speed < 18) {
      return;
    }
    action.value = "dash";
    curiosity.value = clamp(curiosity.value + speed * 0.08, 0, 100);
    gainBond(3);
    updateMood(speed > 32 ? "excited" : "mischief", "你移动得很快，我会追一下并亮起来");
    lastInteractionAt.value = Date.now();
  }

  function tickAutonomy(now: number) {
    const idleFor = now - lastInteractionAt.value;
    hoverCharge.value = clamp(hoverCharge.value - 0.18, 0, 100);
    interactionChain.value = 0;

    if (idleFor > 18000) {
      action.value = "nap";
      updateMood("sleepy", "太久没人理我，我会慢慢进入省电待机");
      energy.value = clamp(energy.value + 0.3, 0, 100);
      return;
    }

    if (idleFor > 8500) {
      action.value = curiosity.value > 52 ? "sing" : "blink";
      updateMood(curiosity.value > 52 ? "mischief" : "calm", "我会自己眨眼、哼唱或探头看看你在不在");
      energy.value = clamp(energy.value + 0.08, 0, 100);
      return;
    }

    if (!isDragging.value) {
      action.value = "idle";
    }
  }

  return {
    action,
    activeSkin,
    availableSkins,
    energy,
    growthProgress,
    hint,
    intimacy,
    isDragging,
    level,
    mood,
    moodLabel,
    setSkin,
    tickAutonomy,
    registerDragEnd,
    registerDragMove,
    registerDragStart,
    registerHover,
    registerPointerDash,
    registerTap,
    unlockedSkinIds
  };
});
