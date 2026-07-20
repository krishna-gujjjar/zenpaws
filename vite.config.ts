import react from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const host = loadEnv(mode, ".", "").TAURI_DEV_HOST;

  return {
    clearScreen: false,
    plugins: [react()],
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
      watch: { ignored: ["**/src-tauri/**"] },
    },
  };
});
