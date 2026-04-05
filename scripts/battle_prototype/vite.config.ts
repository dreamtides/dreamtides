import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      "/connect": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
      "/perform_action": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
      "/poll": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
      "/log": {
        target: "http://localhost:26598",
        changeOrigin: true,
      },
    },
  },
});
