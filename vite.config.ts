import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
// @ts-expect-error process is a nodejs global
const cacheDir = process.env.VITE_CACHE_DIR || "node_modules/.vite";

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  cacheDir,
  build: {
    manifest: true,
    rolldownOptions: {
      output: {
        codeSplitting: {
          // Avoid pulling the chart library into React's static startup graph.
          // Priorities keep shared React dependencies in the core group.
          includeDependenciesRecursively: false,
          groups: [
            {
              name: "vendor-react",
              test: /[\\/]node_modules[\\/](react|react-dom|react-router|react-router-dom|zustand|scheduler)[\\/]/,
              priority: 100,
            },
            {
              name: "vendor-charts",
              test: /[\\/]node_modules[\\/]recharts[\\/]/,
              includeDependenciesRecursively: true,
              priority: 10,
            },
            {
              name: "vendor-tauri",
              test: /[\\/]node_modules[\\/]@tauri-apps[\\/]api[\\/]/,
              priority: 50,
            },
          ],
        },
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // Vitest config
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
}));
