# Contributing to 咪萌 (Meme Manager)

## Development Setup

This project uses **Bun** as the package manager. Do NOT use npm.

```bash
# Install frontend dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

### Android Build

```bash
bun run android:build
```

## Code Style

- Rust: run `cargo clippy` and `cargo fmt` before committing
- TypeScript/Vue: run `vue-tsc --noEmit` to verify types
- Line width: 100 characters max
- Indentation: spaces, not tabs (2 for TypeScript, 4 for Rust)

## Pull Request Process

1. Ensure all static checks pass
2. Update the README or docs if functionality changes
3. Add a note to CHANGELOG.md under "Unreleased"
4. PRs require at least one review

## Commit Messages

Use clear, concise imperative English (e.g., "Fix image loading crash" not "Fixed bug").
