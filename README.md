# Meme Manager - 本地表情包管理工具

一个基于 Tauri 2、Vue 3 和 TypeScript 的跨平台本地表情包分享和管理工具。

## 功能特性

- ✅ **模式管理**：支持创建多个表情包模式（分类）
- ✅ **分组管理**：在每个模式下创建多个分组
- ✅ **关键词管理**：为分组添加关键词，支持拼音和首字母缩写搜索
- ✅ **智能搜索**：通过关键词、拼音或首字母缩写快速查找表情包
- ✅ **批量操作**：支持批量删除、移动表情包
- ✅ **导入导出**：支持配置和数据的导入导出
- ✅ **主题切换**：支持浅色/深色/跟随系统三种主题模式
- ✅ **跨平台**：支持 Linux (x86_64) 和 Android (aarch64)

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **后端**: Rust + Tauri 2
- **数据库**: SQLite (rusqlite)
- **状态管理**: Vue Composition API

## 开发环境要求

- Node.js 18+ 
- Rust 1.70+
- 操作系统特定的依赖：
  - **Linux**: webkit2gtk, libappindicator3, librsvg2-dev
  - **Android**: Android SDK & NDK

## 安装依赖

```bash
npm install
```

## 开发运行

```bash
npm run tauri dev
```

这将同时启动 Vite 开发服务器和 Tauri 应用。

## 构建生产版本

```bash
npm run tauri build
```

构建后的应用将位于 `src-tauri/target/release/bundle/` 目录下。

## 项目结构

```
meme/
├── src/                      # 前端源码
│   ├── components/          # Vue 组件
│   │   └── SideMenu.vue    # 侧边菜单组件
│   ├── views/              # 页面视图
│   │   ├── ModeManagement.vue      # 模式管理
│   │   ├── GroupManagement.vue     # 分组管理
│   │   ├── KeywordManagement.vue   # 关键词管理
│   │   └── Settings.vue           # 设置页面
│   ├── App.vue             # 主应用组件
│   └── main.ts             # 入口文件
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── lib.rs         # 主要业务逻辑
│   │   └── main.rs        # 应用入口
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
└── package.json           # Node.js 依赖配置
```

## 数据库结构

应用使用 SQLite 数据库存储以下数据：

- **modes**: 模式表（表情包的顶级分类）
- **groups**: 分组表（模式下的子分类）
- **keywords**: 关键词表（用于搜索）
- **images**: 图片表（表情包文件信息）
- **keyword_group_links**: 关键词-分组关联表
- **keyword_mode_links**: 关键词-模式关联表
- **config**: 配置表（用户设置）

## 配置文件

应用的配置文件位于：
- **Linux**: `~/.local/share/meme/meme.db`
- **Android**: `<应用数据目录>/meme.db`

用户可以通过设置页面导出/导入配置文件（JSON 格式）。

## 使用说明

### 首次启动

1. 应用会提示选择表情包存储目录
2. 如果目录为空，会自动创建初始结构
3. 如果目录已有数据，会尝试导入现有数据

### 主界面

- **顶部 Tab**: 切换不同的模式
- **中部 Tab**: 切换当前模式下的分组
- **网格区域**: 显示表情包缩略图
- **底部搜索栏**: 搜索表情包

### 编辑模式

- 长按图片或点击编辑按钮进入编辑模式
- 可以选择多张图片进行批量删除或移动
- 可以新增分组或导入表情包

### 侧边菜单

- **模式管理**: 创建、编辑、删除模式
- **分组管理**: 管理所有分组，支持搜索和过滤
- **关键词管理**: 管理搜索关键词及其关联
- **设置**: 修改应用配置和数据管理

## 注意事项

1. **图片格式**: 支持 JPG、PNG、GIF、WebP 等常见格式
2. **缩略图**: 应用会自动生成缩略图以加快加载速度
3. **分享统计**: 每次分享或复制图片都会自动更新分享次数
4. **数据安全**: 建议定期备份数据以防丢失

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！