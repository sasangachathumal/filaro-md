# Filaro

A lightweight, view-only markdown previewer for your desktop.

Open a `.md` file and read it, nicely rendered, without starting up a code editor. Filaro doesn't edit files and never writes to them.

---

## For users

### What it does

- **Opens markdown files** (`.md`, `.markdown`) from a native file picker.
- **Renders GitHub Flavored Markdown**: tables, task lists, and fenced code blocks with syntax highlighting.
- **Keeps a Recent list** in the sidebar:
  - **Now previewing** at the top shows the file you're reading.
  - **Recent** lists up to 20 files you've opened, newest first. Each shows the file name and the folder it's in, relative to your home folder (for example `Downloads` or `Documents/notes`). Hover over a name or folder to see it in full.
  - Click a recent file to open it again. It moves back to the top.
  - A file that has been moved or deleted stays in the list, greyed out and marked **missing**, so you can see what's gone. It can't be opened.
  - The list is kept between restarts.
- **Shows its own resource usage** at the bottom of the sidebar: CPU and RAM as percentages, colored green (under 50%), yellow (50–80%) and red (80% or more).
- Follows your system's **light or dark mode**.

### How to use it

1. Click **Open Markdown File…** and choose a file.
2. The file appears in the preview pane and at the top of **Recent**.
3. Next time, click it in **Recent** instead of browsing for it.

### Good to know

- **Filaro is read-only.** There's no editing, saving, or unsaved-changes prompts.
- **Opening a whole folder** (shown as a tree of its markdown files) is planned for v1 but not available yet.
- **The resource numbers cover Filaro's main process only.** The part of the app that draws the page runs as a separate system process and isn't counted, so real usage is somewhat higher than shown.
- **To clear the Recent list**, quit Filaro and delete `recent-files.json`:
  - macOS: `~/Library/Application Support/com.smsc.filaro-md/`
  - Windows: `%APPDATA%\com.smsc.filaro-md\`

### Installing

There are no pre-built installers yet. Until there are, build Filaro from source (see [Building installers](#building-installers) below). Filaro targets macOS and Windows, and so far it has been developed and tested on macOS.

---

## For developers

Filaro is a [Tauri v2](https://v2.tauri.app/) app: a thin Rust backend for anything that touches the operating system, and a plain TypeScript + Vite frontend (no UI framework) for everything else.

### Prerequisites

- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS (Rust toolchain plus platform build tools)
- Node.js 20 or newer, with npm

### Getting started

```bash
npm install
npm run tauri dev      # run the app with hot reload
```

### Checks

```bash
npx tsc --noEmit                    # typecheck the frontend
(cd src-tauri && cargo test --lib)  # Rust unit tests
```

### Building installers

```bash
npm run tauri build
```

Installers are written to `src-tauri/target/release/bundle/`.

### Project layout

```text
src/                       Frontend (TypeScript)
  main.ts                  Wires everything together
  file-access.ts           Wrappers for the file commands and home-folder lookup
  resource-usage.ts        Wrapper for the resource-usage command
  markdown-preview.ts      Renders markdown → sanitized HTML
  sidebar.ts               "Now previewing" and "Recent" sections
  resource-meter.ts        CPU / RAM indicator
  path-display.ts          Shortens paths for display (pure string logic)
  styles.css

src-tauri/src/             Backend (Rust)
  lib.rs                   Plugin and command registration, window sizing
  resource_usage.rs        CPU / RAM sampling for the app's own process
  file_access/
    commands.rs            The #[tauri::command] functions
    walk.rs                Folder traversal, filtered to markdown files
    recent.rs              Recent-list storage and rules
    model.rs, error.rs     Shared types and the error sent to the frontend
```

### Architecture

The code keeps a hard line between two layers:

1. **UI layer (frontend):** rendering, sidebar, preview pane, display formatting.
2. **File-access layer (Rust):** dialogs, reading files, walking folders, storing the recent list.

On the frontend, only `file-access.ts` and `resource-usage.ts` talk to Tauri. The components (`MarkdownPreview`, `Sidebar`, `ResourceMeter`) receive plain data and callbacks and never call Tauri themselves. Keeping it this way means the file-access layer is the only part to replace if Filaro ever targets the web or mobile.

### Tauri commands

| Command | Purpose |
| --- | --- |
| `open_markdown_file` | Native file picker, filtered to markdown. Returns the chosen path, or `null` if cancelled. |
| `open_markdown_folder` | Native folder picker; returns a tree of the markdown files inside. *(Not used by the UI yet.)* |
| `read_markdown_file` | Reads a markdown file as text. |
| `read_recent_files` | Returns the recent list, each entry flagged with whether the file still exists. |
| `write_recent_files` | Saves the recent list. Enforces the rules: at most 20 entries, no duplicates, files only. |
| `read_resource_usage` | CPU and RAM usage of the app's main process. |

Errors come back to the frontend as readable strings, and command handlers never `unwrap()`.

### Main dependencies

- **Rust:** `tauri`, `tauri-plugin-dialog`, `walkdir`, `sysinfo` (with only its `system` feature)
- **Frontend:** `marked` + `marked-highlight`, `highlight.js` (core plus a hand-picked set of languages), `DOMPurify`

All rendered HTML goes through DOMPurify before it reaches the page.

### Contributing notes

[`CLAUDE.md`](CLAUDE.md) holds the project's scope rules (what's in and out of v1), design decisions, and known pitfalls. For example, commands that open a dialog must be `async`, or the picker freezes. Read it before adding features.

### Recommended IDE setup

[VS Code](https://code.visualstudio.com/) with the [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) and [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extensions.
