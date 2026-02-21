# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

**Agent Skill & MCP Inventory** — a macOS desktop app (Tauri v1 + React 18 + TypeScript + Vite) that scans local AI agent config files (Codex, Claude, Gemini, Antigravity) and displays their MCP servers and Skills in a unified dashboard.

## Commands

```bash
# Install dependencies
pnpm install

# Dev server (browser preview only, no Tauri features)
pnpm run dev                      # → http://127.0.0.1:5175

# Desktop dev (hot reload with Tauri window)
pnpm run tauri dev

# Type check (TypeScript only)
pnpm run typecheck

# Full type check (TypeScript + Rust cargo check)
pnpm run typecheck:full

# Production macOS build
pnpm run deploy:mac               # runs: tsc -b && vite build && tauri build

# Pre-flight verification (env → deps → ts → rust)
pnpm run verify:all

# Inspect CI/PR failures
pnpm run ci:inspect-pr -- --pr <PR_NUMBER>
```

## Architecture

### IPC Boundary: Rust ↔ TypeScript

All frontend–backend communication goes through Tauri's `invoke()` bridge.

**Tauri commands** (registered in `src-tauri/src/main.rs`):
| Command | Purpose |
|---|---|
| `discover_all` | Scan all agents, use cache if unchanged |
| `refresh(agent?)` | Force rescan, optionally filter by agent |
| `group_by(records, key)` | Group records by `"agent"`, `"scope"`, or `"location"` |
| `resolve_duplicates(records)` | Find records sharing the same `canonical_group_id` |
| `open_path(path)` | Open file in macOS (`open` command) |

**Frontend wrappers** live in `src/api.ts` — typed wrappers around `invoke<T>()`. Never call `invoke()` directly from components; go through `api.ts`.

**Shared type contract**: `src/types.ts` mirrors Rust structs in `src-tauri/src/discovery.rs`. When adding fields to `DiscoveryRecord` in Rust, update `types.ts` as well.

### Rust Backend (`src-tauri/src/`)

- **`discovery.rs`** — all logic: scanning, parsing, caching, grouping.
  - `AgentType`: `codex | claude | gemini | antigravity`
  - `ItemKind`: `mcp | skill`
  - `ScopeKind`: `global | personal | project | managed | system | session | antigravity-config | unknown`
  - `DiscoveryRecord`: canonical normalized record emitted for every detected item
  - `discover_all(filter, force_refresh)`: entry point; checks SHA-256+mtime cache before rescanning
  - `scan_codex/claude/gemini/antigravity()`: per-agent scanner functions
  - Skills are detected by finding `SKILL.md` files; MCP entries come from JSON/TOML config files
  - Cache stored at `~/Library/Caches/agent-skill-inventory/inventory-cache.json`

- **`main.rs`** — thin Tauri builder; just registers command handlers, no business logic.

### React Frontend (`src/`)

- **Single component** `App.tsx` — all UI state lives here (no external state library).
  - `records`: raw `DiscoveryRecord[]` from Rust
  - `filtered`: derived list after applying agent/scope/kind/status/query filters
  - `groups`: `filtered` records grouped via `groupBy` Tauri command (with `localGroup` as fallback)
  - `hierarchy`: tree structure built client-side from `filtered` for the hierarchy view
  - Two view modes: `gallery` (card grid grouped by agent/scope/location) and `hierarchy` (collapsible tree)
- **`styles.css`** — all styles, no CSS framework.

### Scan Rules (config files read at runtime)

| Agent | Files scanned |
|---|---|
| Codex | `~/.codex/config.toml`, `~/.codex/skills/`, `~/.codex/vendor_imports/skills/`, per-project `.codex/` |
| Claude | `~/.claude.json`, `~/.claude/settings.json`, `~/.claude/projects/**/*.json`, `/Library/Application Support/ClaudeCode/managed-mcp.json`, per-project `.mcp.json` and `.claude/skills/` |
| Gemini | `~/.gemini/settings.json`, `~/.gemini/skills/`, `~/.gemini/antigravity/mcp_config.json`, per-project `.gemini/` |
| Antigravity | `~/.gemini/antigravity/mcp_config.json`, `~/.gemini/antigravity/code_tracker/**/*_mcp.json`, `~/.gemini/antigravity/skills/` |

Project roots are discovered from `~/.claude.json` top-level keys (absolute paths) plus `$CWD`.

## Agent CLI Documentation

Reference links for each supported agent's official docs — use these when updating scan rules or adding new config paths.

| Agent | Docs URL |
|---|---|
| **Claude Code** | https://docs.anthropic.com/en/docs/claude-code |
| **OpenAI Codex** | https://platform.openai.com/docs/codex |
| **Gemini CLI** | https://github.com/google-gemini/gemini-cli |
| **Antigravity** | Internal — part of Gemini CLI; config lives under `~/.gemini/antigravity/` |

### Key config paths per agent (for scanner updates)

- **Claude**: `~/.claude.json` · `~/.claude/settings.json` · `~/.claude/CLAUDE.md` · per-project `CLAUDE.md` / `.mcp.json` / `.claude/`
- **Codex**: `~/.codex/config.toml` · per-project `CODEX.md` / `.codex/`
- **Gemini**: `~/.gemini/settings.json` · `~/.gemini/GEMINI.md` · per-project `GEMINI.md` / `.gemini/`
- **Antigravity**: `~/.gemini/antigravity/mcp_config.json` · `~/.gemini/antigravity/code_tracker/` · `~/.gemini/antigravity/skills/`
- **Agent-agnostic soul file**: `AGENTS.md` (OpenAI convention, recognized by all agents)

## Key Constraints

- **Tauri v1**, not v2. APIs are `@tauri-apps/api` v1.x. Do not use v2 APIs.
- `pnpm` only — no npm/yarn.
- TypeScript `strict` mode is required; `verify:typescript` will fail otherwise.
- The frontend is a single `App.tsx`. Keep it that way unless there is a strong reason to split.
