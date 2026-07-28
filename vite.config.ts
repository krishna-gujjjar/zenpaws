import path from "node:path";

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const host = loadEnv(mode, ".", "").TAURI_DEV_HOST;

  return {
    clearScreen: false,
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@": path.resolve(process.cwd(), "src"),
      },
    },
    server: {
      hmr: host
        ? {
            host,
            port: 1421,
            protocol: "ws",
          }
        : undefined,
      host,
      port: 1420,
      strictPort: true,
      watch: {
        ignored: [
          "**/.bun/**",
          "**/.cargo/**",
          "**/.local/**",
          "**/.rustup/**",
          "**/node_modules/**",
          "**/src-tauri/**",
        ],
      },
    },
  };
});