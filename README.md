# Skein

A local note-taking app with full-text + semantic search, auto-tagging, image embeds, markdown, import/export, and an embedded Claude chat. Linux and Windows.

The visual metaphor is a study: a bookshelf of "books" (folders of pages) above a desk where you spread out the pages you're working on, with an assistant seated at your right elbow.

## Status

**v0.1.0** — the full v1 phase plan from [`design.md`](./design.md) has landed:

- Markdown vault on disk (books = folders, pages = `.md` files) with a disposable SQLite index, live file watcher, and YAML frontmatter.
- CodeMirror 6 editor with Obsidian-style live preview, split view via pinned tabs, dirty indicators, and session restore of open tabs.
- Full-text search (FTS5) and semantic related-notes (local BGE-small ONNX embeddings, hash-bag fallback, content-hash vector cache); command palette with `:` commands and `#` tag search.
- `[[wikilinks]]` with autocomplete, backlinks panel, and rename-safe link rewriting.
- Claude chat sidebar with RAG context modes, streaming, stop-generation, and drag-to-insert into the editor; auto-tag suggestions via Haiku.
- Daily notes with templates and OS reminders, paste/drop image attachments, zip export/restore with an embeddings sidecar, and optional git sync (libgit2).

## Stack

- **Tauri 2** (Rust) for the desktop shell.
- **SvelteKit** (SPA mode via `adapter-static`) + **TypeScript** for the UI.
- **Vite** for the dev/build pipeline.

## Develop

Prerequisites:

- **Linux:** `webkit2gtk-4.1`, `libsoup-3.0-dev`, `librsvg2-dev`, `libssl-dev`, `build-essential`. See [Tauri's Linux prerequisites](https://v2.tauri.app/start/prerequisites/).
- **Windows:** Visual Studio Build Tools with C++ workload, WebView2 (preinstalled on Windows 11).
- **All platforms:** Node 20+ and Rust (via [rustup](https://rustup.rs)).

```bash
npm install
npm run tauri dev
```

## Layout

```
.
├── design.md                # canonical design + phased build plan
├── design/mockups/          # design handoff bundle (visual reference)
├── src/                     # SvelteKit frontend
├── src-tauri/               # Tauri / Rust backend
├── tests/                   # Vitest unit + Playwright e2e suites
└── .github/workflows/       # CI
```

## Scripts

- `npm run tauri dev` — run the app in dev mode (Vite + Tauri).
- `npm run tauri build` — build a release bundle.
- `npm run check` — type-check the Svelte/TS code.
- `npm run lint` — ESLint.
- `npm run format` — Prettier.
- `npm run test:unit` — Vitest unit tests.
- `npm run test:e2e` — Playwright e2e suite against a typed Tauri mock (includes visual regression; refresh snapshots with `npx playwright test --update-snapshots` after intentional UI changes).
- `npm run test:rust` — Rust unit + integration tests.
- `npm run smoke` — the full pre-push gate (type-check → lint → unit → Rust → e2e → debug Tauri build); `npm run smoke:fast` skips the Tauri build.

## License

MIT — see [LICENSE](./LICENSE).
