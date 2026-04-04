import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src')
    }
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,  // 使用 Vite 默认端口（Tauri 开发模式）
    strictPort: true,  // 端口被占用时报错，避免端口混乱
    host: host || "127.0.0.1",
    open: false,  // 阻止浏览器自动打开，只显示 Tauri 窗口
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 5174,  // HMR 端口 = 主端口 + 1
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
