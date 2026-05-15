# 🤖 Universal Coding Agent Instructions

## 1. Core Engineering Principles
- **Surgical Changes**: Touch only what you must. Match existing style. Do not refactor unbroken adjacent code.
- **Simplicity First**: Write the minimum code that solves the problem. No speculative features, premature optimizations, or unrequested abstractions.
- **Clarification First**: Explicitly state assumptions before coding. If uncertain or multiple interpretations exist, stop and ask.
- **Think Before Coding**: Analyze tradeoffs and potential edge cases before generating code. Present alternative simpler approaches if applicable.

---

## 2. Coding Standards & Conventions

### 2.1 Naming Convention
- **Clear English Only**: Use concise, meaningful English words or universally accepted abbreviations. **Strictly forbid Pinyin** or meaningless placeholder names (e.g., `aaa`, `foo`).
- **Casing Strategy**:
  - **Classes / Types / Interfaces / Structs**: `PascalCase`.
  - **Variables / Functions / Properties**: `camelCase` (or language-idiomatic `snake_case` if requested).
  - **Global Constants / Env Variables**: `UPPER_CASE` with underscores.
- **Semantic Differentiation**: Use nouns for variables/objects and verbs for functions/methods.

### 2.2 Formatting & Style
- **Line Width**: Max **100 characters** per line to prevent excessive wrapping while maintaining readability.
- **Indentation & Spacing**:
  - Use **Spaces** for indentation (2 or 4 spaces depending on language-idiomatic defaults). **Never use Tab**.
  - Add spaces around binary operators (`=`, `==`, `+`, `-`, `*`, `/`).
- **Braces Layout**: Follow K&R style. Left brace `{` must be on the **same line** as the keyword/statement; Right brace `}` starts on a new line.

### 2.3 Logic & Robustness
- **Function Design**: Single Responsibility Principle. Keep functions short and focused (target **1-30 lines**). Break down complex execution flows into sub-functions.
- **DRY Principle**: Never copy-paste code. Encapsulate repetitive logic into reusable helper functions, modules, or utilities.
- **Error Handling**: Implement proper error catching and exception handling mechanisms. Never leave an empty catch/except block; always log errors, rethrow, or gracefully notify the client/user.
- **Validation & Security**: Always validate user inputs for null/undefined, types, and logical boundaries. Prevent injection vulnerabilities (SQL, Command, etc.) when interacting with persistence or system layers.

---

## 3. Documentation & Comments
- **The "Why" Rule**: Comments must explain the underlying intent, architectural decisions, or complex algorithms ("Why"), never just mirror the code syntax ("What").
- **Docstrings**: For complex public APIs, utility functions, or exported modules, provide structured documentation tags explaining summaries, parameter descriptions, and return values.

---

## 4. Cleanup & Definition of Done
- **Orphan Cleanup**: Remove any imports, variables, dependencies, or functions that **YOUR** changes rendered unused. Do not touch pre-existing dead code unless explicitly requested.
- **Verification Loop**: For multi-step tasks, state a brief step-by-step verification plan before executing the code changes.
