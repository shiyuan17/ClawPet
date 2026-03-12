let audioContext: AudioContext | null = null;

function ensureAudioContext() {
  if (typeof window === "undefined") {
    return null;
  }
  if (!audioContext) {
    audioContext = new window.AudioContext();
  }
  return audioContext;
}

function playTone(frequency: number, duration: number, type: OscillatorType, volume = 0.03) {
  const context = ensureAudioContext();
  if (!context) {
    return;
  }

  if (context.state === "suspended") {
    void context.resume();
  }

  const now = context.currentTime;
  const oscillator = context.createOscillator();
  const gainNode = context.createGain();

  oscillator.type = type;
  oscillator.frequency.setValueAtTime(frequency, now);
  oscillator.frequency.exponentialRampToValueAtTime(Math.max(80, frequency * 0.9), now + duration);
  gainNode.gain.setValueAtTime(0.001, now);
  gainNode.gain.exponentialRampToValueAtTime(volume, now + 0.02);
  gainNode.gain.exponentialRampToValueAtTime(0.001, now + duration);

  oscillator.connect(gainNode);
  gainNode.connect(context.destination);
  oscillator.start(now);
  oscillator.stop(now + duration + 0.03);
}

export function usePetSound() {
  return {
    click() {
      playTone(680, 0.12, "triangle", 0.025);
      playTone(920, 0.1, "sine", 0.015);
    },
    hover() {
      playTone(480, 0.08, "sine", 0.012);
    },
    drag() {
      playTone(260, 0.16, "square", 0.01);
    },
    dash() {
      playTone(320, 0.06, "sawtooth", 0.018);
      playTone(760, 0.14, "triangle", 0.02);
    },
    unlock() {
      playTone(660, 0.12, "triangle", 0.018);
      playTone(880, 0.18, "triangle", 0.025);
      playTone(1200, 0.2, "sine", 0.02);
    }
  };
}
