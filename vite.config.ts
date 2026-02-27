import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// @ts-expect-error process 是 Node.js 全局对象
const host = process.env.TAURI_DEV_HOST;

// Vite 配置：https://vite.dev/config/
export default defineConfig(async () => ({
  // 插件配置：使用 Vue 插件
  plugins: [vue()],

  // 专为 Tauri 开发优化的 Vite 选项，仅在 `tauri dev` 或 `tauri build` 时应用
  //
  // 1. 防止 Vite 清屏，避免隐藏 Rust 编译错误
  clearScreen: false,
  
  // 2. 开发服务器配置：Tauri 要求固定端口，如果端口不可用则失败
  server: {
    // 开发服务器端口号
    port: 1420,
    // 严格端口模式：端口被占用时报错而不是自动切换
    strictPort: true,
    // 服务器监听的主机地址（从环境变量获取，用于远程调试）
    host: host || false,
    // 热模块替换（HMR）配置
    hmr: host
      ? {
          // HMR 协议
          protocol: "ws",
          // HMR 主机地址
          host,
          // HMR 端口号
          port: 1421,
        }
      : undefined,
    // 文件监听配置
    watch: {
      // 3. 告诉 Vite 忽略监听 `src-tauri` 目录（避免监听 Rust 代码变化）
      ignored: ["**/src-tauri/**"],
    },
  },
  
  // 路径解析配置
  resolve: {
    // 路径别名配置
    alias: {
      // @ 符号指向 src 目录
      "@": "/src",
    },
  },
}));
