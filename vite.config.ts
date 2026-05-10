import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import compression from "vite-plugin-compression";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [
    vue(),
    compression({
      algorithm: "gzip",
      threshold: 1024,
      minRatio: 0.8,
    }),
  ],
  define: {
    __VUE_PROD_DEVTOOLS__: false,
    __DEV__: JSON.stringify(process.env.NODE_ENV !== 'production'),
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
    minify: "terser",
    terserOptions: {
      compress: {
        drop_console: false,
        drop_debugger: true,
        pure_funcs: ["console.log", "console.info", "console.warn", "console.debug", "console.trace"],
      },
      format: {
        comments: false,
      },
    },
    assetsInlineLimit: 4096,
    cssCodeSplit: true,
    sourcemap: false,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["vue", "@varlet/ui"],
          tauri: ["@tauri-apps/api", "@tauri-apps/plugin-clipboard-manager", "@tauri-apps/plugin-dialog", "@tauri-apps/plugin-fs", "@tauri-apps/plugin-opener"],
          utils: ["pinyin-pro"],
        },
      },
    },
  },
}));
