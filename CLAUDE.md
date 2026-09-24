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
- **Window** opens at roughly half the screen (70% of each side of the monitor's work area, never below 800×600), centered
- **Sidebar** with two sections:
  - "Now previewing" — pinned at top, the current file
  - "Recent" — list below it, showing file **name + location**. The location is the containing folder relative to the home folder (`/Users/sam/Downloads/notes.md` → `Downloads`; a file directly in home → `Home`; outside home → full folder path). Hovering the location shows the full path.
  - Long names and locations are cut to one line with `…`; hovering shows the full text
- Recent list **persists across restarts** (local JSON config file)
- Moved/deleted recent entries are shown **greyed out / marked missing**, not silently dropped
- Recent list rules (enforced in Rust, so no caller can get them wrong):
  - Capped at **20** entries; the oldest drop off
  - Reopening a file **moves it to the top** — never a duplicate entry
  - **Files only** — opening a folder never adds anything to Recent
  - The existence check runs **on every read** (one stat per entry). No file watcher — that's a whole subsystem for a cosmetic grey-out.
  - A missing or corrupt config file means an empty list, not an error
- **"App resource usage"** indicator at the bottom of the sidebar: CPU and RAM as percentages, colored green (<50%), yellow (50–80%), red (≥80%), refreshed every 2s. CPU is the share of total machine capacity; RAM is the share of system RAM (size in MB on hover).
  - It measures the app's **main process only**. The webview renders in separate OS processes that aren't counted (on macOS they can't be reliably traced back to the app), so the numbers understate the real footprint. The hover text says so — keep it honest.
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

1. **UI layer** — markdown rendering, sidebar, file tree, preview pane, resource indicator, display formatting. Frontend.
2. **File-access layer** — opening dialogs, reading files, walking folders, persisting the recent list. Rust / Tauri commands.

Rationale: if these stay decoupled, the file-access layer is the only thing that gets swapped if this ever targets web (File System Access API) or mobile. Costs nothing now, saves a rewrite later. Don't let file-access concerns leak into UI components or vice versa.

**Rust surface is deliberately thin** — only what touches the OS. Everything else is frontend.

### Rust side (`src-tauri/src/`)

- `file_access/` — everything file-related. `commands.rs` holds the thin `#[tauri::command]` wrappers; the logic lives in plain functions that take a `&Path` so they can be unit-tested without Tauri (`walk.rs` for folder traversal, `recent.rs` for the recent list). `error.rs` has `FileAccessError`, which serializes to a readable string for the frontend.
- `resource_usage.rs` — the CPU/RAM sampler. It touches the OS but isn't file access, so it stays out of `file_access/`.
- `lib.rs` — plugin and command registration, plus sizing and centering the window at startup.
- The recent list is stored as a JSON array of paths in `recent-files.json` in Tauri's app config folder (`app.path().app_config_dir()` — never hardcode a path). Only paths are stored; the name and `exists` flag are worked out on each read.

Relevant Rust/Tauri pieces for v1:
- Tauri **commands** (the Rust ↔ frontend bridge)
- Tauri **dialog plugin** (file/folder picker)
- File reads use **`std::fs` inside our own commands**, not the Tauri fs plugin. The fs plugin's permission scopes don't fit "read whatever file the user just picked." (`tauri-plugin-fs` is only pulled in indirectly by the dialog plugin.)
- **`walkdir`** crate (folder traversal)
- **`sysinfo`** crate, with only its `system` feature (resource indicator)
- `Result` / error handling — surface errors to the frontend properly, don't `unwrap()` in command handlers

Use **Tauri v2** docs and APIs throughout. v1 patterns do not apply and will silently mislead.

### Frontend side (`src/`, plain TypeScript + Vite, no framework)

- **Only `file-access.ts` and `resource-usage.ts` talk to Tauri** (`invoke` and the Tauri JS APIs). They're thin wrappers.
- **Components take plain data, never Tauri.** `MarkdownPreview`, `Sidebar` and `ResourceMeter` receive plain data as props, plus callbacks for user actions. They don't know where the data came from and never call Tauri.
- `main.ts` is the only place that wires the wrappers to the components.
- Pure helpers like `path-display.ts` do string work only, with no OS access. The home folder they need is fetched by `file-access.ts` and passed down.
- Rendering libraries: `marked` + `marked-highlight`, `highlight.js`, `DOMPurify`. Load highlight.js via **`highlight.js/lib/core` with a hand-picked set of languages**. The full bundle is ~1 MB of JS versus ~190 KB.

### Known pitfalls

- **Dialog commands must be `async fn`.** A regular (non-async) command runs on the main thread. There, `blocking_pick_file()` / `blocking_pick_folder()` deadlock: the picker opens but can't be clicked, and the cursor spins.
- **Register commands by their defining module** in `generate_handler!` (e.g. `file_access::commands::read_markdown_file`). A `pub use` re-export doesn't carry the extra items the `#[tauri::command]` macro generates, so it fails to compile.
- **`write_recent_files` enforces the rules but doesn't reorder.** Callers pass the list most-recent-first, i.e. with the just-opened path prepended to the current list.
- **The window starts hidden** (`"visible": false` in `tauri.conf.json`). The setup code sizes it and then calls `show()`, which avoids a visible jump. If you change the startup code, make sure `show()` is still called, or the app launches with no window.

## Rendering safety

Markdown is rendered as HTML. Sanitize it. Files come from the local disk but that's not a reason to inject raw HTML unfiltered.

Every render goes through `DOMPurify.sanitize()` before touching the DOM. The config's only extra allowance is `<input type="checkbox" checked disabled>`, for GFM task lists; DOMPurify strips `<input>` by default. Task-list checkboxes are always disabled (read-only).

## Working style

- Scope conservatively. Build the smallest thing that works, then layer.
- Prefer honest trade-off framing over hedged recommendations. If something is a bad idea, say so and say why.
- When a decision has a real fork in it, surface the fork instead of picking silently.
- Don't add dependencies casually — lightweight footprint is the whole reason Tauri was chosen over Electron (small binary, native webview, low memory).

## Roadmap (post-v1, not now)

Outline/TOC → in-document search → themes → Mermaid/LaTeX → multi-tab → export → editing.
Longer term, an embeddable developer SDK is a more plausible monetization path than the consumer market.
