import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // IMPORTANT: Vite pre-bundling can duplicate Univer class prototypes across
  // separate chunks. This breaks the facade mixin pattern (FWorksheet.extend())
  // and causes "X is not a function" errors at runtime. Listing packages here
  // encourages Vite to share code across pre-bundled chunks. Even with this
  // setting, the facade may still break — see image_cell_renderer.ts for the
  // command-based workaround and appendix_d_univer_integration.md for details.
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
