import { createApp } from "vue";
import App from "./App.vue";
import { Button, Icon, Input, Image, Popup, Tabs, Tab, TabsItems, TabItem, Switch, Card, Loading, Result, AppBar } from '@varlet/ui';
import '@varlet/ui/es/button/style';
import '@varlet/ui/es/icon/style';
import '@varlet/ui/es/input/style';
import '@varlet/ui/es/image/style';
import '@varlet/ui/es/popup/style';
import '@varlet/ui/es/tabs/style';
import '@varlet/ui/es/tab/style';
import '@varlet/ui/es/tabs-items/style';
import '@varlet/ui/es/tab-item/style';
import '@varlet/ui/es/switch/style';
import '@varlet/ui/es/card/style';
import '@varlet/ui/es/loading/style';
import '@varlet/ui/es/result/style';
import '@varlet/ui/es/app-bar/style';
import '@varlet/ui/es/snackbar/style';
import '@varlet/ui/es/dialog/style';
import './styles/theme.css';

const app = createApp(App);
app.use(Button);
app.use(Icon);
app.use(Input);
app.use(Image);
app.use(Popup);
app.use(Tabs);
app.use(Tab);
app.use(TabsItems);
app.use(TabItem);
app.use(Switch);
app.use(Card);
app.use(Loading);
app.use(Result);
app.use(AppBar);
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
