let audioContext: AudioContext | null = null;
const eventLastPlayedAt: Record<string, number> = {};
const MASTER_VOLUME_GAIN = 1.9;
const MAX_TONE_VOLUME = 0.08;

function ensureAudioContext() {
  if (globalThis.window === undefined) {
    return null;
  }
  audioContext ??= new globalThis.AudioContext();
  return audioContext;
}

type ToneOptions = {
  volume?: number;
  delay?: number;
  endFrequency?: number;
};

function playTone(frequency: number, duration: number, type: OscillatorType, options: ToneOptions = {}) {
  const context = ensureAudioContext();
  if (!context) {
    return;
  }

  if (context.state === "suspended") {
    void context.resume();
  }

  const now = context.currentTime + Math.max(0, options.delay ?? 0);
  const volume = Math.min(MAX_TONE_VOLUME, (options.volume ?? 0.03) * MASTER_VOLUME_GAIN);
  const endFrequency = Math.max(80, options.endFrequency ?? frequency * 0.9);
  const oscillator = context.createOscillator();
  const gainNode = context.createGain();

  oscillator.type = type;
  oscillator.frequency.setValueAtTime(frequency, now);
  oscillator.frequency.exponentialRampToValueAtTime(endFrequency, now + duration);
  gainNode.gain.setValueAtTime(0.001, now);
  gainNode.gain.exponentialRampToValueAtTime(volume, now + 0.02);
  gainNode.gain.exponentialRampToValueAtTime(0.001, now + duration);

  oscillator.connect(gainNode);
  gainNode.connect(context.destination);
  oscillator.start(now);
  oscillator.stop(now + duration + 0.03);
}

function canPlay(event: string, cooldownMs: number) {
  const now = globalThis.performance?.now?.() ?? Date.now();
  const last = eventLastPlayedAt[event] ?? 0;
  if (now - last < cooldownMs) {
    return false;
  }
  eventLastPlayedAt[event] = now;
  return true;
}

function playAnimationSound(name: string, loop: boolean) {
  const event = `animation:${name}`;
  const cooldown = loop ? 1300 : 420;
  if (!canPlay(event, cooldown)) {
    return;
  }

  switch (name) {
    case "stomp_feet":
      playTone(160, 0.07, "square", { volume: 0.016, endFrequency: 120 });
      playTone(130, 0.1, "triangle", { volume: 0.012, delay: 0.04, endFrequency: 100 });
      break;
    case "applause_to_celebrate":
      playTone(720, 0.08, "triangle", { volume: 0.022 });
      playTone(980, 0.08, "sine", { volume: 0.016, delay: 0.06 });
      playTone(1240, 0.1, "triangle", { volume: 0.013, delay: 0.13 });
      break;
    case "confusion":
      playTone(420, 0.12, "sine", { volume: 0.014, endFrequency: 300 });
      playTone(260, 0.14, "triangle", { volume: 0.012, delay: 0.07, endFrequency: 200 });
      break;
    case "have_meal":
      playTone(360, 0.1, "square", { volume: 0.013, endFrequency: 320 });
      playTone(520, 0.08, "sine", { volume: 0.012, delay: 0.08 });
      break;
    case "chat_typing":
      playTone(900, 0.04, "square", { volume: 0.008, endFrequency: 760 });
      playTone(760, 0.04, "square", { volume: 0.007, delay: 0.04, endFrequency: 650 });
      break;
    case "sleep":
      playTone(180, 0.2, "sine", { volume: 0.009, endFrequency: 140 });
      break;
    case "stretch_yawn_and_rub_your_eyes":
      playTone(240, 0.14, "triangle", { volume: 0.013, endFrequency: 180 });
      playTone(320, 0.12, "sine", { volume: 0.01, delay: 0.09, endFrequency: 250 });
      break;
    case "stretch_body":
      playTone(260, 0.1, "sawtooth", { volume: 0.012, endFrequency: 300 });
      playTone(340, 0.09, "triangle", { volume: 0.011, delay: 0.05, endFrequency: 300 });
      break;
    case "rub_your_eyes":
      playTone(300, 0.09, "sine", { volume: 0.011, endFrequency: 260 });
      playTone(260, 0.08, "sine", { volume: 0.01, delay: 0.07, endFrequency: 220 });
      break;
    case "act_cute_rotation":
      playTone(520, 0.08, "triangle", { volume: 0.01, endFrequency: 640 });
      break;
    case "smile_and_blink":
    case "smile_blink":
      playTone(560, 0.07, "sine", { volume: 0.01, endFrequency: 620 });
      break;
    case "the_body_rises_and_falls":
      playTone(220, 0.12, "sine", { volume: 0.008, endFrequency: 210 });
      break;
    case "think":
      playTone(450, 0.1, "triangle", { volume: 0.011, endFrequency: 520 });
      playTone(620, 0.08, "sine", { volume: 0.009, delay: 0.06, endFrequency: 580 });
      break;
    case "wink_quietly":
      playTone(700, 0.06, "sine", { volume: 0.009, endFrequency: 760 });
      break;
    default:
      playTone(480, 0.06, "sine", { volume: 0.008 });
      break;
  }
}

export function usePetSound() {
  return {
    click() {
      playTone(680, 0.12, "triangle", { volume: 0.025 });
      playTone(920, 0.1, "sine", { volume: 0.015 });
    },
    hover() {
      playTone(480, 0.08, "sine", { volume: 0.012 });
    },
    drag() {
      playTone(260, 0.16, "square", { volume: 0.01 });
    },
    dash() {
      playTone(320, 0.06, "sawtooth", { volume: 0.018 });
      playTone(760, 0.14, "triangle", { volume: 0.02 });
    },
    unlock() {
      playTone(660, 0.12, "triangle", { volume: 0.018 });
      playTone(880, 0.18, "triangle", { volume: 0.025 });
      playTone(1200, 0.2, "sine", { volume: 0.02 });
    },
    animation(name: string, loop = false) {
      playAnimationSound(name, loop);
    }
  };
}
