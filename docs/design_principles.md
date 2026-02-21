# UI Design Principles

This document outlines the core design principles applied to the Agent UI project. The goal is to maintain a highly minimal, developer-focused, and information-dense interface (inspired by tools like the Codex desktop application).

## 1. Minimal & Functional Aesthetic
- **Flat UI:** Avoid heavy drop shadows, excessive border radiuses, and complex gradients.
- **Subtle Borders:** UI panes should be separated by very light, 1px borders (`var(--panel-border)`).
- **Whitespace:** Use margins and padding purposefully to establish a clear visual hierarchy without the need for drawing explicit container boxes everywhere.

## 2. Information Hierarchy
- Focus on answering the user's primary questions immediately:
  1. What is the name of the tool?
  2. What category does it belong to (Soul, Subagent, MCP, Skill)?
- Detailed information (raw JSON, exact paths, verbose descriptions) should be deferred to the right-hand Detail Pane or tooltips.

## 3. Iconography over Text
- Redundant text labels take up space and increase cognitive load.
- Use `lucide-react` for clean, unopinionated SVG icons.
- Common actions (Refresh, Settings, Open External, Toggle Theme) MUST use icons instead of text buttons.
- Always provide helpful hover context using the standard HTML `title` attribute for these icons.

## 4. Unified Theming System
- **No Hardcoded Colors:** All colors must be drawn from the semantic CSS CSS variables (`var(--bg-color)`, `var(--text-main)`, `var(--accent-primary)`, etc.).
- **Light / Dark Mode Parity:** New components must support both Light and Dark modes. The default configuration should read `--color-x` variables from `:root` (Light mode) and `.dark` scoped overrides.

## 5. Standardized Categorization
- Emphasize the four core Agent categories using consistent badge styling:
  - **MCP** (`--bg-mcp`, `--color-mcp`)
  - **Skill** (`--bg-skill`, `--color-skill`)
  - **Subagent** (`--bg-subagent`, `--color-subagent`)
  - **Soul** (`--bg-soul`, `--color-soul`)
- Ensure these badges are compact and use semantic background/text color pairings.
