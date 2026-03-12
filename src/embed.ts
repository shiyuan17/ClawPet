import { createApp, type Component } from "vue";
import { createPinia } from "pinia";
import SpriteDesktopPetPage from "./components/SpriteDesktopPetPage.vue";
import "./style.css";

export function createDesktopPet(target: string | Element, component: Component = SpriteDesktopPetPage) {
  const host = typeof target === "string" ? document.querySelector(target) : target;
  if (!host) {
    throw new Error("Desktop pet mount target not found.");
  }

  const app = createApp(component);
  app.use(createPinia());
  app.mount(host);

  return {
    destroy() {
      app.unmount();
    }
  };
}

export { SpriteDesktopPetPage };
