# AGENTS.md — 全局记忆 / Global Memory

## 🌐 语言规则（最高优先级）

- **思考与互动一律使用中文**：内部推理、与用户对话、回复总结都用中文。
- 例外：代码、标识符、文件内容、引用/报错原文保持原样，不翻译。
- 用户用中文提问时，必须用中文回答。
- Language rule: think and respond in Chinese by default; keep code, identifiers, and quoted error messages verbatim.

---

## 📦 项目要点

- **包管理器：Bun**。禁止使用 `npm install` / `npm run` / `npx`；一律用 `bun install`、`bun run <script>`、`bun run tauri ...`。
- **前端**：Vue 3 + TypeScript + Varlet UI，位于 `src/`（视图在 `src/views/`，组件在 `src/components/`）。
- **后端**：Rust + Tauri 2，位于 `src-tauri/`；命令模块在 `src-tauri/src/commands/`，核心逻辑在 `src-tauri/src/core/`。
- **数据库**：SQLite（rusqlite），路径与表结构见 `src-tauri/src/core/db.rs`；共享连接封装在 `db_state.rs`。
- **表情图(sticker)机制**：后台异步生成 240×240 GIF，磁盘缓存 `.sticker_cache/<相对路径哈希>.gif` 为主，`images.sticker_data` BLOB 为恢复兜底；缓存键用相对路径 FNV-1a 哈希（不用自增 id），`full_refresh` 会清理孤儿缓存。
- **双端差异**：Android（`tauri-plugin-share`、原生分享）与桌面端（`tauri-plugin-share`、剪贴板）的分享/复制路径不同，改动需同时考虑。

---

## 🛠 工程原则

- **外科手术式修改**：只改必须改的，匹配现有风格；不重构无关的完好代码。
- **简单优先**：写解决问题的最少代码，不做投机性功能或过度抽象。
- **先澄清再动手**：存在多种合理解读时，先明确假设或询问用户。
- **注释讲"为什么"**：注释解释意图与设计决策，不复述代码本身。
- **命名用英文**：不用拼音或无意义占位名；类 `PascalCase`，变量/函数 `camelCase`。
- **行宽 ≤ 100 字符，空格缩进（不用 Tab）**。
- **收尾清理**：删除自己改动产生的无用 import/变量/依赖。