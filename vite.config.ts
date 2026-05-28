import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from 'path'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [
    vue(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  define: {
    __VUE_PROD_DEVTOOLS__: false,
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || true,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : false,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    minify: "esbuild",
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["vue", "@varlet/ui"],
          tauri: ["@tauri-apps/api", "@tauri-apps/plugin-clipboard-manager", "@tauri-apps/plugin-dialog", "@tauri-apps/plugin-fs", "@tauri-apps/plugin-opener"],
        },
      },
    },
  },
}));
