/**
 * 构建渠道区分：开源版（oss，默认）与商店版（store）
 *
 * 渠道在构建期由 Vite 环境变量 VITE_APP_CHANNEL 注入：
 *   - bun run build / bun run build:oss  → oss（默认渠道，.env.oss）
 *   - bun run build:store                → store（需显式指定，.env.store）
 *   - bun run tauri build                → oss（tauri.conf.json 默认 beforeBuildCommand）
 *   - bun run tauri:build:store          → store（tauri.store.conf.json 覆盖 beforeBuildCommand）
 *
 * 未注入变量或取值非 store 时一律按开源版处理，避免误打包成商店版。
 */

export type AppChannel = 'store' | 'oss';

const channel: AppChannel = import.meta.env.VITE_APP_CHANNEL === 'store' ? 'store' : 'oss';

/** 开源版：不弹用户协议弹窗，设置页不展示用户协议/隐私政策入口 */
export const isOpenSourceVersion = channel === 'oss';

/** 商店版：协议弹窗与协议入口均保留 */
export const isStoreVersion = channel === 'store';
