import { defineConfig, Plugin } from "vite";
import react from "@vitejs/plugin-react";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

/**
 * Patches RxJS's `bufferWhen` operator to prevent infinite synchronous
 * recursion. Univer's data-validation and formula-engine plugins use
 * `bufferWhen(() => lifecycle$.pipe(filter(x => x === Rendered)))`. After the
 * lifecycle reaches Rendered, each new subscription to `lifecycle$` replays
 * past stages (including Rendered) synchronously, causing `openBuffer` to call
 * itself infinitely and overflow the stack.
 *
 * This plugin adds a reentrancy guard so recursive `openBuffer` calls during
 * the same synchronous subscription are suppressed.
 */
function fixBufferWhenRecursion(): Plugin {
  const guardPattern =
    /(var|let|const)\s+openBuffer\s*=\s*(?:function\s*\(\)|(?:\(\)\s*=>))\s*\{/g;
  const closingPattern =
    /(innerFrom\(closingSelector\(\)\)\.subscribe\([^;]*openBuffer[^;]*\);)\s*\};/g;

  function applyGuard(code: string): string {
    let result = code;
    result = result.replace(guardPattern, (match, keyword) => {
      const varKw = keyword === "const" ? "let" : keyword;
      return `${varKw} _inOpenBuffer = false; ${match} if (_inOpenBuffer) return; _inOpenBuffer = true; try {`;
    });
    result = result.replace(closingPattern, (match, innerFromLine) => {
      return `${innerFromLine} } finally { _inOpenBuffer = false; } };`;
    });
    return result;
  }

  return {
    name: "fix-rxjs-bufferWhen-recursion",
    enforce: "pre",
    transform(code, id) {
      const isRxjsSource = id.includes("rxjs") && !id.includes("node_modules/.vite");
      if (!isRxjsSource) return null;
      if (!code.includes("openBuffer")) return null;
      const patched = applyGuard(code);
      if (patched !== code) return { code: patched, map: null };
      return null;
    },
    config() {
      return {
        optimizeDeps: {
          esbuildOptions: {
            plugins: [
              {
                name: "fix-bufferWhen-esbuild",
                setup(build) {
                  build.onLoad(
                    { filter: /rxjs.*bufferWhen/ },
                    async (args) => {
                      const fs = await import("fs");
                      const original = fs.readFileSync(args.path, "utf8");
                      const patched = applyGuard(original);
                      if (patched !== original) {
                        return { contents: patched, loader: "js" };
                      }
                      return undefined;
                    }
                  );
                },
              },
            ],
          },
        },
      };
    },
  };
}

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [fixBufferWhenRecursion(), react()],

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
