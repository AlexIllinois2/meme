# 咪萌 - 本地表情包管理工具

一个本地表情包分享和管理工具，基于 Tauri 2、Vue 3 和 Rust 打造，支持 Linux 桌面端和 Android 移动端。

应用采用 **"模式-分组"** 二级分类体系，帮助用户高效组织海量表情包。内置智能关键词系统，支持中文、拼音及首字母缩写多维度搜索。独特的分享次数排序功能，自动将常用表情置顶。

## 功能特性

- ✅ **模式管理** — 一级分类（如"工作"、"闲聊"）
- ✅ **分组管理** — 二级分类，基于文件夹的自动同步
- ✅ **关键词管理** — 为分组添加关键词，支持拼音和首字母缩写搜索
- ✅ **智能搜索** — 关键词、拼音、首字母缩写即时检索
- ✅ **批量操作** — 批量删除、移动表情包
- ✅ **主题切换** — 浅色 / 深色 / 跟随系统
- ✅ **跨平台** — Linux (x86_64) + Android (aarch64)
- ✅ **剪贴板粘贴** — 桌面端一键粘贴图片
- ✅ **自定义分享** — Android 自定义分享到指定 App

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Vue 3 + TypeScript + Vite + Varlet UI |
| 后端 | Rust + Tauri 2 |
| 数据库 | SQLite (rusqlite, bundled) |
| 图片处理 | image-rs |
| 拼音 | pinyin-rs |

## 开发环境

### 前置要求

- **Node.js** 18+（通过 mise 管理）
- **Rust** 1.70+
- **Bun**（包管理器，不要用 npm）
- **Linux**: webkit2gtk, libappindicator3, librsvg2-dev
- **Android**: Android SDK & NDK（可选，仅构建 Android 时需要）

### 安装与运行

```bash
# 安装前端依赖
bun install

# 开发模式
bun run tauri dev

# 构建生产版本
bun run tauri build

# 构建 Android apk
bun run android:build
```

### 代码检查

```bash
cargo clippy        # Rust lint
vue-tsc --noEmit    # TypeScript 类型检查
```

## 项目结构

```
meme/
├── src/                        # 前端源码 (Vue 3 + TS)
│   ├── components/             # 公共组件
│   │   ├── ContextMenu.vue     # 右键/长按菜单
│   │   ├── FloatingSearchButton.vue  # 悬浮搜索按钮
│   │   ├── FolderPicker.vue    # 目录选择器
│   │   ├── Icon.vue            # 图标组件
│   │   ├── KeywordManager.vue  # 关键词管理器
│   │   └── ThemeProvider.vue   # 主题提供者
│   ├── composables/            # Vue 组合式函数
│   │   ├── useConfig.ts        # 配置与主题
│   │   ├── useEditMode.ts      # 编辑模式
│   │   ├── useMemeData.ts      # 数据加载/切换
│   │   ├── useSearch.ts        # 搜索逻辑
│   │   └── useSwipe.ts         # 手势滑动
│   ├── views/                  # 页面视图
│   │   ├── Settings.vue        # 设置
│   │   ├── SettingsAbout.vue   # 关于
│   │   ├── Support.vue         # 支持
│   │   ├── UserAgreement.vue   # 用户协议
│   │   └── PrivacyPolicy.vue   # 隐私政策
│   ├── types/                  # TypeScript 类型定义
│   ├── utils/                  # 工具函数
│   ├── App.vue                 # 主应用组件
│   └── main.ts                 # 入口
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # Tauri 入口 (invoke_handler)
│   │   ├── main.rs             # 系统入口
│   │   ├── core/               # 核心基础设施
│   │   │   ├── db.rs           # 数据库初始化与迁移
│   │   │   ├── config.rs       # 配置读写
│   │   │   ├── db_state.rs     # 全局连接状态
│   │   │   ├── error.rs        # 统一错误类型
│   │   │   ├── models.rs       # 数据结构
│   │   │   └── meme_fs.rs      # 文件系统路径解析
│   │   └── commands/           # Tauri 命令
│   │       ├── mode.rs         # 模式 CRUD
│   │       ├── group.rs        # 分组 CRUD + 搜索
│   │       ├── image.rs        # 图片查询/搜索/管理
│   │       ├── keyword.rs      # 关键词生成/同步
│   │       ├── clipboard.rs    # 剪贴板操作
│   │       ├── upload.rs       # 图片上传
│   │       ├── save_image.rs   # 保存到相册
│   │       ├── custom_share.rs # 自定义分享 App
│   │       └── android_picker.rs # Android 目录选择
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json                # 前端依赖 (bun)
├── AGENTS.md                   # 项目级 AI 助手指令
└── README.md
```

## 数据库

SQLite 数据库存储在：

- **Linux**: `~/.local/share/meme/meme.db`
- **Android**: `/data/data/com.v.meme/files/meme.db`

### 表结构

| 表 | 说明 |
|---|---|
| `modes` | 模式（一级分类） |
| `groups` | 分组（二级分类，关联 mode） |
| `images` | 图片记录（相对路径存储） |
| `keywords` | 关键词（含拼音/首字母缩写） |
| `keyword_group_links` | 关键词-分组多对多关联 |
| `config` | 用户配置（单行表） |

## 使用说明

首次启动应用会提示选择表情包存储目录。目录结构约定：

```
<meme_dir>/
├── <模式名>/
│   ├── <分组名>/          ← 分组名支持逗号/顿号分隔关键词
│   │   ├── image1.png
│   │   └── image2.jpg
│   └── <另一个分组名>/
└── <另一个模式名>/
```

支持格式：PNG, JPG, JPEG, GIF, WebP, BMP

## 许可证

MIT License — 详见 [LICENSE](./LICENSE)
