import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Vite is configured for Tauri 2: fixed dev port, no auto-open, file system
// poll fallback for the src-tauri/ folder so backend rebuilds trigger reloads.
// See https://tauri.app/v1/guides/getting-started/setup/vite for the canonical
// shape; mirrors the gitehr GUI config.
export default defineConfig({
  plugins: [react()],

  // Tauri serves the production frontend from the bundled app resources, not
  // from an HTTP origin root. Keep all generated asset URLs relative so the
  // webview can resolve JS/CSS/images correctly in packaged builds.
  base: "./",

  // Prevent Vite from clearing the screen so we can see Rust compiler errors.
  clearScreen: false,

  server: {
    port: 5173,
    strictPort: true,
    host: "127.0.0.1",
    watch: {
      // Tell Vite to ignore watching the Rust source tree; tauri's own dev
      // watcher rebuilds the backend separately.
      ignored: ["**/src-tauri/**"],
    },
  },

  // For Tauri's bundled build: emit ES2022 (Tauri webviews on supported
  // platforms all handle it) and use Vite's debug-friendly source maps in
  // dev only.
  build: {
    target: "es2022",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
