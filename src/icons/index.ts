import { addCollection } from '@iconify/vue/offline';
import collection from './remix-icons.json';

// 注册 Remix Icon 离线子集：应用内所有 ri: 图标均在此数据中解析，
// 不依赖 api.iconify.design 在线请求，弱网/离线环境下图标也能完整渲染
addCollection(collection as Parameters<typeof addCollection>[0]);
