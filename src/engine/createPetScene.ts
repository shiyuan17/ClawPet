import { Application, BlurFilter, Container, Graphics, Text } from "pixi.js";
import { gsap } from "gsap";
import type { PetAction, PetMood, SkinDefinition } from "../stores/pet";

type InteractionHandlers = {
  onTap: () => void;
  onHover: (elapsedMs: number) => void;
  onDragStart: () => void;
  onDragMove: (speed: number) => void;
  onDragEnd: () => void;
};

type SceneOptions = {
  host: HTMLDivElement;
  handlers: InteractionHandlers;
};

type RenderState = {
  mood: PetMood;
  action: PetAction;
  skin: SkinDefinition;
  energy: number;
  intimacy: number;
  dragging: boolean;
};

export async function createPetScene({ host, handlers }: SceneOptions) {
  const app = new Application();
  await app.init({
    width: 340,
    height: 340,
    antialias: true,
    backgroundAlpha: 0,
    resolution: window.devicePixelRatio || 1
  });

  host.replaceChildren(app.canvas);
  const stage = app.stage;
  stage.eventMode = "static";
  stage.sortableChildren = true;

  const anchor = new Container();
  anchor.position.set(app.screen.width / 2, app.screen.height / 2);
  stage.addChild(anchor);

  const shadow = new Graphics().ellipse(0, 94, 92, 24).fill({ color: 0x5d4b3d, alpha: 0.16 });
  shadow.filters = [new BlurFilter({ strength: 6 })];
  shadow.zIndex = 0;
  anchor.addChild(shadow);

  const floatRig = new Container();
  floatRig.zIndex = 1;
  anchor.addChild(floatRig);

  const motionRig = new Container();
  motionRig.sortableChildren = true;
  floatRig.addChild(motionRig);

  const glow = new Graphics();
  glow.alpha = 0.85;
  glow.zIndex = 1;
  motionRig.addChild(glow);

  const body = new Graphics();
  body.zIndex = 2;
  motionRig.addChild(body);

  const cheekLeft = new Graphics();
  const cheekRight = new Graphics();
  cheekLeft.zIndex = 3;
  cheekRight.zIndex = 3;
  motionRig.addChild(cheekLeft, cheekRight);

  const eyeLeft = new Graphics();
  const eyeRight = new Graphics();
  eyeLeft.zIndex = 4;
  eyeRight.zIndex = 4;
  motionRig.addChild(eyeLeft, eyeRight);

  const mouth = new Graphics();
  mouth.zIndex = 5;
  motionRig.addChild(mouth);

  const sparkle = new Graphics();
  sparkle.alpha = 0;
  sparkle.zIndex = 6;
  motionRig.addChild(sparkle);

  const bubble = new Graphics().roundRect(-110, -136, 220, 46, 23).fill({ color: 0xffffff, alpha: 0.68 });
  bubble.alpha = 0;
  bubble.zIndex = 7;
  motionRig.addChild(bubble);

  const bubbleText = new Text({
    text: "",
    style: {
      fill: 0x664320,
      fontFamily: "Avenir Next, PingFang SC, sans-serif",
      fontSize: 15,
      fontWeight: "600"
    }
  });
  bubbleText.anchor.set(0.5);
  bubbleText.position.set(0, -113);
  bubbleText.alpha = 0;
  bubbleText.zIndex = 8;
  motionRig.addChild(bubbleText);

  let hoverStart = 0;
  let dragging = false;
  let dragPointerId: number | null = null;
  let lastDragPoint = { x: 0, y: 0, t: 0 };

  app.canvas.style.width = "100%";
  app.canvas.style.height = "100%";
  app.canvas.style.touchAction = "none";

  stage.hitArea = app.screen;

  stage.on("pointertap", () => {
    handlers.onTap();
    pulse(1.03, 0.18);
  });

  stage.on("pointerover", () => {
    hoverStart = performance.now();
  });

  stage.on("pointermove", (event) => {
    const point = event.global;
    const elapsed = hoverStart ? performance.now() - hoverStart : 80;
    handlers.onHover(elapsed);

    const nx = (point.x - app.screen.width / 2) / 110;
    const ny = (point.y - app.screen.height / 2) / 110;
    gsap.to([eyeLeft, eyeRight], {
      x: gsap.utils.clamp(-7, 7, nx * 6),
      y: gsap.utils.clamp(-5, 5, ny * 4),
      duration: 0.16,
      overwrite: true
    });
    gsap.to(motionRig, {
      rotation: gsap.utils.clamp(-0.12, 0.12, nx * 0.08),
      x: gsap.utils.clamp(-10, 10, nx * 8),
      y: gsap.utils.clamp(-8, 8, ny * 3),
      duration: 0.25,
      overwrite: "auto"
    });

    if (!dragging || dragPointerId !== event.pointerId) {
      return;
    }

    const now = performance.now();
    const deltaX = point.x - lastDragPoint.x;
    const deltaY = point.y - lastDragPoint.y;
    const deltaT = Math.max(16, now - lastDragPoint.t);
    const speed = Math.sqrt(deltaX * deltaX + deltaY * deltaY) / (deltaT / 16);
    handlers.onDragMove(speed);

    gsap.to(motionRig, {
      x: gsap.utils.clamp(-32, 32, point.x - app.screen.width / 2),
      y: gsap.utils.clamp(-26, 26, point.y - app.screen.height / 2),
      duration: 0.12,
      overwrite: true
    });

    lastDragPoint = { x: point.x, y: point.y, t: now };
  });

  stage.on("pointerdown", (event) => {
    dragging = true;
    dragPointerId = event.pointerId;
    lastDragPoint = { x: event.global.x, y: event.global.y, t: performance.now() };
    handlers.onDragStart();
    pulse(0.98, 0.14);
  });

  const endDrag = () => {
    if (!dragging) {
      return;
    }
    dragging = false;
    dragPointerId = null;
    handlers.onDragEnd();
    gsap.to(motionRig, {
      x: 0,
      y: 0,
      rotation: 0,
      duration: 0.45,
      ease: "elastic.out(1, 0.6)"
    });
  };

  stage.on("pointerup", endDrag);
  stage.on("pointerupoutside", endDrag);
  stage.on("pointerleave", endDrag);

  gsap.to(floatRig, {
    y: "-=8",
    repeat: -1,
    yoyo: true,
    ease: "sine.inOut",
    duration: 1.7
  });

  function pulse(scale: number, duration: number) {
    gsap.fromTo(
      motionRig.scale,
      { x: scale, y: scale },
      { x: 1, y: 1, duration, ease: "power2.out", overwrite: true }
    );
  }

  function drawStar(target: Graphics, color: number) {
    target.clear();
    target.star(0, 0, 5, 11, 5).fill({ color, alpha: 0.95 });
  }

  function showBubble(text: string) {
    bubbleText.text = text;
    gsap.killTweensOf([bubble, bubbleText]);
    gsap.fromTo([bubble, bubbleText], { alpha: 0, y: -6 }, { alpha: 1, y: 0, duration: 0.2 });
    gsap.to([bubble, bubbleText], { alpha: 0, duration: 0.35, delay: 1.5 });
  }

  function render(state: RenderState, utterance?: string) {
    const { skin, mood, action, energy, intimacy } = state;
    const excitement = intimacy / 100;
    const sleepy = mood === "sleepy";

    glow.clear();
    glow.circle(0, 0, 122).fill({ color: skin.glow, alpha: 0.26 + excitement * 0.16 });

    body.clear();
    body.circle(0, 0, 98).fill({
      color: skin.shellBottom
    });
    body.circle(-12, -18, 84).fill({
      color: skin.shellMid,
      alpha: 0.92
    });
    body.circle(-22, -28, 46).fill({
      color: skin.shellTop,
      alpha: 0.76
    });

    cheekLeft.clear().circle(-48, 22, 18).fill({ color: skin.blush, alpha: sleepy ? 0.2 : 0.52 });
    cheekRight.clear().circle(48, 22, 18).fill({ color: skin.blush, alpha: sleepy ? 0.2 : 0.52 });

    eyeLeft.clear();
    eyeRight.clear();
    const eyeWidth = sleepy ? 22 : mood === "excited" ? 28 : 24;
    const eyeHeight = sleepy ? 4 : mood === "shy" ? 18 : 24;
    const eyeAlpha = clamp(0.65 + energy / 180, 0.5, 1);
    eyeLeft.roundRect(-58, -26, eyeWidth, eyeHeight, 12).fill({ color: 0x161312, alpha: eyeAlpha });
    eyeRight.roundRect(34, -26, eyeWidth, eyeHeight, 12).fill({ color: 0x161312, alpha: eyeAlpha });

    if (mood === "excited") {
      drawStar(sparkle, skin.accent);
      sparkle.position.set(0, -64);
      gsap.to(sparkle, { alpha: 1, rotation: Math.PI, duration: 0.32, overwrite: true });
    } else if (action === "sing") {
      sparkle.clear();
      sparkle.circle(0, 0, 8).fill({ color: 0xffffff, alpha: 0.85 });
      sparkle.position.set(62, -52);
      gsap.to(sparkle, { alpha: 0.78, y: "-=10", duration: 0.45, overwrite: true });
    } else {
      gsap.to(sparkle, { alpha: 0, duration: 0.24, overwrite: true });
    }

    mouth.clear();
    mouth.moveTo(-24, 34);
    if (mood === "sleepy") {
      mouth.arc(0, 30, 12, Math.PI * 0.2, Math.PI * 0.8);
    } else if (mood === "shy") {
      mouth.bezierCurveTo(-10, 48, 10, 48, 24, 34);
    } else if (mood === "mischief") {
      mouth.bezierCurveTo(-6, 46, 10, 28, 26, 34);
    } else {
      mouth.bezierCurveTo(-10, 60, 12, 60, 24, 34);
    }
    mouth.stroke({ color: 0x8f4d1c, width: mood === "excited" ? 6 : 5, cap: "round" });

    const squish = action === "tap" ? 1.06 : action === "drag" ? 0.94 : 1;
    gsap.to(motionRig.scale, {
      x: squish,
      y: 2 - squish,
      duration: 0.18,
      overwrite: true
    });

    gsap.to(shadow.scale, {
      x: sleepy ? 0.85 : dragging ? 0.76 : 1,
      y: sleepy ? 0.72 : dragging ? 0.62 : 1,
      duration: 0.25,
      overwrite: true
    });

    if (utterance) {
      showBubble(utterance);
    }
  }

  function destroy() {
    gsap.killTweensOf([
      floatRig,
      motionRig,
      motionRig.scale,
      shadow.scale,
      eyeLeft,
      eyeRight,
      sparkle,
      bubble,
      bubbleText
    ]);
    app.destroy(true, { children: true });
  }

  return {
    destroy,
    render
  };
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
