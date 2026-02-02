import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

export default defineConfig({
  plugins: [react()],
  root: resolve(__dirname, "src"),
  build: {
    outDir: resolve(__dirname, "dist-web"),
    emptyOutDir: true,
  },
  server: {
    port: 8080,
    proxy: {
      "/api": {
        target: "http://localhost:3030",
        changeOrigin: true,
      },
    },
  },
  resolve: {
    alias: {
      "@paperclip/common": resolve(__dirname, "../common/src"),
    },
  },
});
