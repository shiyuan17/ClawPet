import { createApp, type Component } from "vue";
import { createPinia } from "pinia";
import DesktopPetWidget from "./components/DesktopPetWidget.vue";
import "./style.css";

export function createDesktopPet(target: string | Element, component: Component = DesktopPetWidget) {
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

export { DesktopPetWidget };
