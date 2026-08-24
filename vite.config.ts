import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  build: {
    // Tauri's webviews (WebView2 / WKWebView) are modern; ES2022 avoids
    // transpiling syntax they natively support.
    target: "es2022",
    rollupOptions: {
      output: {
        // The editor stack is heavy; split it out of the entry chunk so the
        // app shell paints without parsing ProseMirror + KaTeX first.
        manualChunks: {
          milkdown: ["@milkdown/crepe", "@milkdown/kit", "@milkdown/vue"],
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
});
