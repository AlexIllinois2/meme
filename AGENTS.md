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
- Android: `bun run android1` (dev), `bun run android:build` (production)

## Tauri Backend Structure
Rust modules in `src-tauri/src/`:
- `lib.rs` - Entry point, registers all Tauri commands
- `db.rs` - Database initialization
- `config.rs` - App configuration
- `mode.rs`, `group.rs`, `keyword.rs`, `image.rs` - CRUD operations
- `trash.rs` - Soft delete support

## Varlet UI Components
Use these Varlet components (not other libraries):
- Tabs: `var-tabs`
- Grid: `var-grid`
- Images: `var-image fit="cover"`
- Preview: `var-image-preview`
- Menus/dialogs/drawers: Use Varlet equivalents

## References
- AI_DEVELOPMENT.md - Detailed refactoring requirements
- .vscode/rules/project_rules.md - UI/UX specifications
- QUICKSTART.md - Setup guide with system dependencies
