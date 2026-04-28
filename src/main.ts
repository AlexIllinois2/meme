import { createApp } from "vue";
import App from "./App.vue";
import Varlet from '@varlet/ui';
import '@varlet/ui/es/style';
import 'remixicon/fonts/remixicon.css';
import './styles/theme.css';

const app = createApp(App);
app.use(Varlet);
app.mount("#app");
