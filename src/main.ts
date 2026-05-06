import { createApp } from "vue";
import App from "./App.vue";
import Varlet from '@varlet/ui';
import '@varlet/ui/es/style';
import 'remixicon/fonts/remixicon.css';
import './styles/theme.css';

// 开发环境启用 vConsole（调试工具）
async function initVConsole() {
  try {
    // 使用 * as 导入整个模块
    const mod = await import('vconsole');
    console.log('[vConsole] 模块内容:', Object.keys(mod));
    
    // vconsole 可能有不同的导出方式
    const VConsole = mod.default || (mod as any).VConsole || mod;
    if (typeof VConsole === 'function') {
      new VConsole();
      console.log('[vConsole] 已启用');
    } else {
      console.error('[vConsole] 不是构造函数:', VConsole);
    }
  } catch (e) {
    console.error('[vConsole] 加载失败:', e);
  }
}

// 判断开发环境：
// 1. Vite dev 模式 (import.meta.env.DEV)
// 2. 本地调试 (localhost)
// 3. Android Tauri WebView (有 __TAURI__ 且不是生产构建)
const isTauriAndroid = typeof (window as any).__TAURI__ !== 'undefined' && /android/i.test(navigator.userAgent);
const isDev = import.meta.env.DEV || window.location.hostname === 'localhost' || isTauriAndroid;

if (isDev) {
  initVConsole();
}

const app = createApp(App);
app.use(Varlet);
app.mount("#app");