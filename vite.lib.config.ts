import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [vue()],
  build: {
    lib: {
      entry: resolve(__dirname, "src/embed.ts"),
      name: "KeAIDesktopPet",
      fileName: "keai-desktop-pet",
      formats: ["es", "umd"]
    },
    rollupOptions: {
      external: [],
      output: {
        exports: "named"
      }
    }
  }
});
