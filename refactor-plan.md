# 大胆重构计划

## Phase 1: App.vue 继续拆分
- `useSwipe` — 滑动手势（~60行，干净独立）
- `useShare` — 分享功能（~200行）
- `useAgreement` — 协议弹窗（~30行）
- `handleAndroidBack` 简化（100+行顺序if-else→可配置化）

## Phase 2: Settings.vue 拆分（976行）
- 提取子组件：SettingsAppearance, SettingsStorage, SettingsAbout
- 或提取 composable: useSettings

## Phase 3: Rust image.rs 继续拆（837行）
- 提取 upload 模块 (upload_images, upload_images_android)
- 剩下的 image.rs 专注查询/搜索
