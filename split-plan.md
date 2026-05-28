# 代码拆分计划

## 1. image.rs (1008 行) → image.rs + clipboard.rs
拆分策略: 将剪贴板相关函数抽到独立模块 clipboard.rs
- paste_image_from_clipboard
- paste_image_from_clipboard_raw (desktop + android)
- read_clipboard_with_system_command
- ClipboardImage struct

## 2. App.vue (4360 行) → App.vue + composables/
拆分策略: 提取 3 个 composable：
- useMemeData.ts — modes/groups/images 加载管理、搜索
- useAndroidBack.ts — Android 返回键处理、全局编辑模式
- useAppConfig.ts — 配置加载/更新、主题同步

## 3. Settings.vue (976 行) → Settings.vue + Settings*.vue
拆分策略: 提取子视图组件
- 分享设置 → SettingsShare.vue
- 权限设置 → SettingsPermissions.vue
