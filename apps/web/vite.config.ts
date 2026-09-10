import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": resolve(import.meta.dirname, "src"),
    },
  },
  // WASM ファイルを正しくロードするための設定
  optimizeDeps: {
    exclude: ["harmonic-core"],
  },
  server: {
    port: 5173,
    fs: {
      // WASM パッケージへのアクセスを許可
      allow: ["../../packages/harmonic-core/pkg", "."],
    },
  },
  preview: {
    // Docker上のk6からローカルpreviewへ到達させる。任意hostは許可しない。
    allowedHosts: ["host.docker.internal"],
  },
});
