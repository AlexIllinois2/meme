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

// 判断开发环境：仅在 Vite dev 模式或 localhost 下启用 vConsole
// release 版本（生产构建）不显示 vConsole
const isDev = import.meta.env.DEV || window.location.hostname === 'localhost';

if (isDev) {
  initVConsole();
}

const app = createApp(App);
app.use(Varlet);
app.mount("#app");