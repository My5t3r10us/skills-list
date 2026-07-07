import { defineConfig } from "vite";
import { join } from "node:path";

const cacheDir =
  process.env.VITE_CACHE_DIR ??
  join(process.env.LOCALAPPDATA ?? ".", "skills-list", "vite-cache");

export default defineConfig({
  clearScreen: false,
  cacheDir,
  server: {
    strictPort: true,
    port: 5174,
  },
  envPrefix: ["VITE_", "TAURI_"],
});
