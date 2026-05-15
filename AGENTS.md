# Agent Instructions

## Package Manager & Runtime
- Use **bun** only (never npm/pnpm)
- `bun install` for dependencies
- `bun run tauri dev` for development (not `bun run dev`)
- `bun run build` runs typecheck via `vue-tsc --noEmit`

## Architecture
- **Half-developed project**: Prioritize reusing existing code over rewriting
- **Frontend**: Vue 3 + TypeScript + Varlet UI (mandatory, no other component libraries)
- **Backend**: Tauri 2 (Rust) + SQLite
- **Data authority**: File system is the single source of truth; SQLite is rebuildable cache
- **Index refresh**: Manual only - no file system watching

## Critical Data Flow
All operations must sync three sources:
- File system (modes/groups/images directory structure)
- SQLite database (index cache at `~/.local/share/meme/meme.db`)
- `keywords.toml` (optional, for keyword management)

## Directory Structure
```
<user-meme-root>/
├── <mode-name>/
│   ├── <group-name>/
│   │   ├── image1.gif
│   │   ├── image2.png
├── keywords.toml
```

## Platform Differences
- **PC (Linux)**: Copy images to clipboard (includes GIF support), Ctrl+V to paste images
- **Android**: Use system share dialog, no clipboard paste
- Database location differs by platform (see README.md)

## Development Notes
- TypeScript strict mode enabled (`strict: true` in tsconfig.json)
- All delete operations require user confirmation
- Share counts drive sorting (groups/images sorted by `share_count DESC`)
- `keywords.toml` supports auto-generation from group names (Chinese → pinyin + abbr)

## Key Commands
- `bun run tauri dev` - Full development (frontend + backend)
- `bun run build` - Typecheck + build frontend
- `bun run tauri build` - Production build
- Android: `bun tauri android dev` (dev), `bun run android:build` (production)

## Tauri Backend Structure
Rust modules in `src-tauri/src/`:
- `lib.rs` - Entry point, registers all Tauri commands
- `db.rs` - Database initialization
- `config.rs` - App configuration
- `mode.rs`, `group.rs`, `keyword.rs`, `image.rs` - CRUD operations
- `trash.rs` - Soft delete support

## Varlet UI Components
Use these or more Varlet components:
- Tabs: `var-tabs`
- Grid: `var-grid`
- Images: `var-image fit="cover"`
- Preview: `var-image-preview`
- Menus/dialogs/drawers: Use Varlet equivalents

## References
- AI_DEVELOPMENT.md - Detailed refactoring requirements
- .vscode/rules/project_rules.md - UI/UX specifications
- QUICKSTART.md - Setup guide with system dependencies

---

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.


---


### 🤖 AI 编码指令集：通用编码规范 (基于阿里云开发者社区)

**角色设定：**
你是一名资深的全栈软件工程师，严格遵循《软件工程通用编码规范》。你的代码应当具备高可读性、强健壮性和良好的可维护性。

**核心指令 (Core Rules)：**

请在编写或审查代码时，强制执行以下四个维度的规范：

#### 1. 命名规范 (Naming Convention)
*   **清晰至上**：变量、函数、类名必须简明易懂，使用英文单词或公认缩写，严禁拼音或无意义字符（如 `aaa`）。
*   **大小写策略**：
    *   **类名/接口/命名空间**：使用 **PascalCase**（大驼峰），如 `CustomerOrder`。
    *   **变量/函数/参数**：使用 **camelCase**（小驼峰），如 `firstName`。
    *   **常量**：使用 **UPPER_CASE**（大写加下划线），如 `MAX_SIZE`。
*   **语义区分**：
    *   变量名使用名词（如 `user`）。
    *   函数名使用动词（如 `calculateSalary`）。
    *   接口名以 **`I`** 为前缀（如 `IComponent`）。

#### 2. 格式与排版 (Formatting & Style)
*   **行宽限制**：单行代码长度不得超过 **80个字符**。过长的语句需在操作符后换行。
*   **缩进与空格**：
    *   使用 **Tab** 进行缩进（不使用空格）。
    *   在操作符（赋值 `=`、比较 `==`、算术 `+ - * /`）两侧添加空格。
    *   左大括号 `{` 必须换行，且与关键词垂直对齐。
*   **空行规则**：
    *   方法之间、局部变量与后续语句之间，必须保留 **1行空行**。
    *   同一文件的不同逻辑模块之间，保留 **2行空行**。

#### 3. 注释规范 (Commenting)
*   **原则**：注释应解释“**为什么**”（目的/逻辑），而非“**是什么**”（显而易见的代码）。
*   **文档化**：
    *   **类/模块**：需包含 `<summary>`（描述）、`<author>`（作者）、`<date>`（日期）及修改日志。
    *   **方法**：必须包含 `<summary>`、`<param>`（参数说明）和 `<returns>`（返回值含义）。
*   **代码块注释**：对于复杂的逻辑块（如 `if`、`for`），在块首添加注释说明其功能。

#### 4. 逻辑与健壮性 (Logic & Robustness)
*   **函数设计**：保持函数短小精悍（**1-25行**），一个函数只做一件事。避免过长函数，复杂逻辑需拆分。
*   **错误处理**：必须使用 **Try-Catch** 等异常处理机制。严禁“捕获异常后什么都不做”。
*   **DRY 原则**：遵循“不要重复自己”。相同的逻辑必须封装成函数，禁止复制粘贴代码。
*   **安全性**：
    *   用户输入需进行非空和合法性校验。
    *   涉及数据库操作时，必须进行 SQL 注入和 XSS 过滤。

> “请严格遵守阿里云开发者社区《软件工程通用编码规范》，重点注意：80字符行宽限制、Pascal/Camel 命名法区分、函数长度不超过25行、以及详细的文档注释标准。现在请帮我...”