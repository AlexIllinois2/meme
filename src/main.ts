import { createApp } from "vue";
import App from "./App.vue";
import Varlet from '@varlet/ui';
import '@varlet/ui/es/style';
import './styles/theme.css';

const app = createApp(App);
app.use(Varlet);
app.mount("#app");

// 仅在开发环境引入 vconsole
if (import.meta.env.DEV) {
  import('vconsole').then((mod) => {
    const VConsole = mod.default || (mod as any).VConsole || mod;
    if (typeof VConsole === 'function') {
      new VConsole();
      console.log('[vConsole] 已启用');
    }
  }).catch((e) => {
    console.error('[vConsole] 加载失败:', e);
  });
}
