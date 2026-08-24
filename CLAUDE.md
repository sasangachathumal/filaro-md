# Filaro

A lightweight, view-only markdown previewer desktop app.

**Repo:** `filaro-md`
**Stack:** Tauri v2 (Rust backend + web frontend)
**Targets:** macOS and Windows for v1. Linux is deferred (Tauri builds it from the same code — it's a checkbox later, not a rewrite).

## Why this exists

Personal daily-use tool first, portfolio piece second. The problem being solved: markdown files scattered across machines, and needing to open VS Code with an extension every time just to *read* one. Filaro replaces that.

The markdown editor market is saturated with free competitors. **We are not competing there.** Don't propose features on the basis of "competitors have this."

## Core principle: v1 is a previewer, NOT an editor

Reading only. No writing, no saving, no dirty-state tracking, no undo/redo, no autosave prompts. Editing roughly triples the scope and solves none of the stated pain.

If a request seems to imply editing, flag it rather than silently building toward it.

## v1 scope — IN

- View-only preview of `.md` / `.markdown` files
- Rendered as **GitHub Flavored Markdown** (tables, task lists, fenced code blocks with syntax highlighting)
- Open a **single file** via native dialog
- Open a **folder** — shown as a tree filtered to markdown files only. Not a general file manager.
- **Sidebar** with two sections:
  - "Now previewing" — pinned at top, the current file
  - "Recent" — list below it, showing file **name + full path**
- Recent list **persists across restarts** (local JSON config file)
- Moved/deleted recent entries are shown **greyed out / marked missing**, not silently dropped
- Packaged installers for Windows and macOS

## v1 scope — OUT (this list is the discipline)

Do not build, scaffold, or leave hooks for any of these unless explicitly asked:

- Editing / writing / saving
- Whole-computer file scanning
- Table of contents / heading outline *(strong v1.1 candidate)*
- In-document search (Cmd/Ctrl+F) *(strong v1.1 candidate)*
- Mermaid diagrams
- LaTeX / math rendering
- Themes / theming system
- Multi-tab
- Export (PDF, HTML, etc.)
- Any AI features

## Architecture

Keep a hard line between:

1. **UI layer** — markdown rendering, sidebar, file tree, preview pane. Frontend.
2. **File-access layer** — opening dialogs, reading files, walking folders, persisting the recent list. Rust / Tauri commands.

Rationale: if these stay decoupled, the file-access layer is the only thing that gets swapped if this ever targets web (File System Access API) or mobile. Costs nothing now, saves a rewrite later. Don't let file-access concerns leak into UI components or vice versa.

**Rust surface is deliberately thin** — only what touches the OS. Everything else is frontend.

Relevant Rust/Tauri pieces for v1:
- Tauri **commands** (the Rust ↔ frontend bridge)
- Tauri **dialog plugin** (file/folder picker)
- Tauri **fs plugin** (reading files)
- **`walkdir`** crate (folder traversal)
- `Result` / error handling — surface errors to the frontend properly, don't `unwrap()` in command handlers

Use **Tauri v2** docs and APIs throughout. v1 patterns do not apply and will silently mislead.

## Rendering safety

Markdown is rendered as HTML. Sanitize it. Files come from the local disk but that's not a reason to inject raw HTML unfiltered.

## Working style

- Scope conservatively. Build the smallest thing that works, then layer.
- Prefer honest trade-off framing over hedged recommendations. If something is a bad idea, say so and say why.
- When a decision has a real fork in it, surface the fork instead of picking silently.
- Don't add dependencies casually — lightweight footprint is the whole reason Tauri was chosen over Electron (small binary, native webview, low memory).

## Roadmap (post-v1, not now)

Outline/TOC → in-document search → themes → Mermaid/LaTeX → multi-tab → export → editing.
Longer term, an embeddable developer SDK is a more plausible monetization path than the consumer market.
