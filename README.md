# Folder Manager App *(Name TBD)*

> ⚠️ **Pre-release software** — Active development on the `dev` branch. Not yet feature-complete.

A desktop file manager built for power users who need more than folders. Tag files across directories, track work status, search massive directories in real time, and manage the Recycle Bin — all in a fast, native UI.

Built with **Tauri 2 + Svelte 5 + Rust + SQLite**.

---

## What Makes This Different

Standard file explorers give you folders. This gives you:

- **Tags** — label files across folder boundaries (e.g. `#client`, `#urgent`)
- **Status tracking** — mark files as *In Progress*, *Done*, *Review*, or any custom status
- **Real-time search** — streams results progressively as it scans; large directories without freezing
- **Recycle Bin integration** — browse, restore, or permanently delete from the Windows Recycle Bin across all drives
- **Conflict resolution** — copy/move operations let you skip, replace, or keep both files
- **Virtual scrolling** — renders 16K+ files smoothly without loading everything into the DOM

---

## Features

| Feature | Status | Notes |
|---------|--------|-------|
| Directory browsing | ✅ | Back/forward history, root directory selection |
| Virtual scrolling | ✅ | 16K+ files at a fixed row height |
| Multi-select | ✅ | Shift-click, Ctrl-click, drag selection |
| Sortable columns | ✅ | Name, date modified, type, size, tags, status |
| Resizable columns | ✅ | Drag column headers |
| Tags | ✅ | Create and assign tags per file, stored in SQLite |
| Status labels | ✅ | Create and assign statuses per file, stored in SQLite |
| Copy / Cut / Paste | ✅ | Ctrl+C / Ctrl+X / Ctrl+V + context menu |
| Drag-and-drop | ✅ | Move or copy files between directories |
| Conflict resolution | ✅ | Skip / Keep Both / Replace dialog |
| Real-time search | ✅ | Batched streaming results, case-insensitive |
| Recycle Bin | ✅ | Browse, restore, permanently delete — all drives |
| UAC elevation | ✅ | Prompts for admin rights when needed |
| Context menu | ✅ | Right-click for all file operations |
| Extended delete options | 🚧 | Additional delete flows in progress |
| UI / style polish | 🚧 | Active design iteration |
| Smart groups / virtual folders | 📋 | Phase 1 |
| Saved workspaces | 📋 | Phase 1 |
| Auto-tag rules | 📋 | Phase 1 |
| Batch rename | 📋 | Phase 1 |
| Advanced search filters | 📋 | Phase 1 |
| Keyboard shortcut system | 📋 | Phase 1 |
| Inline file notes | 📋 | Phase 1 |
| Dual-pane mode | 💡 | Considering |
| Quick preview panel | 💡 | Considering |
| Git status indicators | 💡 | Considering |
| File comparison | 💡 | Considering |
| Automation engine | 💡 | Considering |

**Legend:** ✅ Done • 🚧 In Progress • 📋 Planned • 💡 Considering

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://rustup.rs/)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) — on Windows: Microsoft C++ Build Tools + WebView2

### Install

```bash
npm install
```

### Run in development

```bash
npm run tauri dev
```

### Build a production installer

```bash
npm run tauri build
```

### Other scripts

```bash
npm run check        # TypeScript type checking
npm run check:watch  # Watch mode
```

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| UI Framework | Svelte 5 (runes) + SvelteKit 2 |
| Desktop Shell | Tauri 2 |
| Backend | Rust (2021 edition) |
| Database | SQLite via `rusqlite` |
| Styling | Tailwind CSS 4 |
| Language | TypeScript 5 |
| Icons | Material Icon Theme |

---

## Project Structure

```
Folder Manager App/
├── src/                          # Frontend (Svelte/TypeScript)
│   ├── lib/
│   │   ├── components/           # UI components
│   │   │   ├── FileTable.svelte  # Main file list with virtual scroll
│   │   │   ├── ContextMenu.svelte
│   │   │   ├── ConflictDialog.svelte
│   │   │   └── navbar/
│   │   ├── state/                # Reactive state (Svelte runes)
│   │   │   ├── FileExplorerState.svelte.ts
│   │   │   ├── SelectionState.svelte.ts
│   │   │   ├── ClipboardState.svelte.ts
│   │   │   ├── TagManager.svelte.ts
│   │   │   ├── StatusManager.svelte.ts
│   │   │   └── TrashState.svelte.ts
│   │   ├── runes/
│   │   │   ├── virtualScroll.svelte.ts
│   │   │   └── keyboardShortcuts.svelte.ts
│   │   └── types.ts
│   └── routes/
│       └── +page.svelte
│
└── src-tauri/                    # Backend (Rust)
    └── src/
        ├── commands/             # Tauri IPC handlers
        ├── db/                   # Repository layer
        ├── platform/             # Windows-specific (SID, drive enumeration)
        ├── migration.rs          # SQLite schema migrations
        ├── model.rs              # Shared data models
        └── error.rs              # Custom error types
```

---

## Platform Notes

Currently **Windows-focused**. Windows-specific features include:

- Recycle Bin access via `$Recycle.Bin` and Windows SID resolution
- UAC elevation for privileged file operations
- Drive enumeration for multi-drive Recycle Bin support

Cross-platform support (macOS/Linux) is not a current priority, but the Tauri foundation supports it.

---

## Branches & Issues

| Branch | Purpose |
|--------|---------|
| `main` | Stable snapshots |
| `dev` | Active development — latest features land here first |

Bug reports and feature requests welcome via [GitHub Issues](https://github.com/FranzAlexander/Folder-Manager-App/issues).
