import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Force all Univer facade modules to share the same pre-bundled chunks.
  // Without this, Vite pre-bundles @univerjs/sheets/facade and
  // @univerjs/sheets-drawing-ui/facade into separate files with duplicated
  // FWorksheet classes, causing the drawing-ui facade's .extend() call to
  // modify a different prototype than the one used at runtime.
  optimizeDeps: {
    include: [
      "@univerjs/core",
      "@univerjs/core/facade",
      "@univerjs/sheets",
      "@univerjs/sheets/facade",
      "@univerjs/sheets-ui",
      "@univerjs/sheets-ui/facade",
      "@univerjs/sheets-drawing-ui",
      "@univerjs/sheets-drawing-ui/facade",
      "@univerjs/drawing",
      "@univerjs/drawing-ui",
      "@univerjs/sheets-drawing",
      "@univerjs/engine-render",
      "@univerjs/engine-formula",
      "@univerjs/engine-formula/facade",
    ],
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
}));
