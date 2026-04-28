# Skein

A local note-taking app with full-text + semantic search, auto-tagging, image embeds, markdown, import/export, and an embedded Claude chat. Linux and Windows.

The visual metaphor is a study: a bookshelf of "books" (folders of pages) above a desk where you spread out the pages you're working on, with an assistant seated at your right elbow.

## Status

Phase 0 — bootstrap. See [`design.md`](./design.md) for the full design and phased implementation plan.

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
└── .github/workflows/       # CI
```

## Scripts

- `npm run tauri dev` — run the app in dev mode (Vite + Tauri).
- `npm run tauri build` — build a release bundle.
- `npm run check` — type-check the Svelte/TS code.
- `npm run lint` — ESLint.
- `npm run format` — Prettier.

## License

MIT — see [LICENSE](./LICENSE).
