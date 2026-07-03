# Planned improvements & features — 2026-07-03

Plan for one maintenance + enhancement pass over Skein (Tauri 2 / SvelteKit 5).
Each item is scoped, matched to existing code style, and verified via
svelte-check / eslint / vitest / playwright plus compile-and-test of the pure
Rust modules (the full Tauri crate needs GTK dev libs not present on this
machine, so Tauri-coupled Rust edits get careful manual review instead).

## Improvements (10)

1. **Fix UTF-8 panic in the chunker's soft-cap split** (`src-tauri/src/chunker.rs`)
   — `current_text[..SOFT_CAP_CHARS]` byte-slices at 2000 and panics when that
   byte falls inside a multi-byte character; any long non-ASCII note crashes indexing.
2. **Fix UTF-8 panic in auto-tag body truncation** (`src-tauri/src/autotag.rs`)
   — `body[..6000]` has the same mid-character slice panic; truncate at a char boundary.
3. **Skip re-embedding when a page's content hash is unchanged** (`src-tauri/src/index.rs`)
   — `upsert_page` re-chunks and re-embeds every page even when nothing changed;
   rename/delete flows rebuild the whole vault, so the design-doc's "if hash
   unchanged, skip" early-out makes those flows near-instant.
4. **Surface mid-stream Anthropic SSE `error` events** (`src-tauri/src/chat.rs`)
   — `event: error` frames (e.g. overloaded_error) are currently dropped on the
   floor, leaving the chat bubble hanging; parse them and emit a chat error.
5. **Exclude `.git/` from vault export** (`src-tauri/src/archive.rs`)
   — a git-synced vault currently ships its entire git history inside the
   export zip; skip it like `.skein/`.
6. **Drop stale index rows when a book folder vanishes externally** (`src-tauri/src/watcher.rs`)
   — deleting/renaming a book folder outside the app only emits an event for the
   directory path, so its pages linger in search until a manual rebuild.
7. **Handle CRLF frontmatter in the chunker** (`src-tauri/src/chunker.rs`)
   — `strip_frontmatter` only matches `---\n`, so Windows-edited pages get their
   YAML embedded into chunk text (autotag's splitter already handles `\r\n`).
8. **Escape HTML in search-result snippets** (`src/lib/skein/components/CommandPalette.svelte`)
   — FTS snippets are injected via `{@html}` with raw page text; a note containing
   markup (e.g. an imported vault) can inject arbitrary HTML/script into the app.
   Escape everything except the FTS `<mark>` highlights; unit-test the helper.
9. **Ctrl+D shortcut for today's daily note** (`src/lib/skein/VaultWindow.svelte`, `Titlebar.svelte`)
   — the daily note has a titlebar button and menu entry but no keybinding;
   add Ctrl/Cmd+D and document it in Help → Keyboard shortcuts.
10. **README refresh** (`README.md`) — still says "Phase 0 — bootstrap" though
    v0.1.0 shipped the full phase plan; describe current features and document
    the test/smoke scripts.

## New features (5)

11. **Move page to another book** — new `move_page` Rust command (collision-safe
    rename into the target folder, index update) + `Move to…` context-menu in
    `PageList.svelte`; wikilinks keep resolving since the file stem is unchanged.
12. **Tag search in the command palette** — `#` prefix queries a new
    `pages_with_tag` command (indexed `tags` table, prefix match) so tags are
    finally reachable from the keyboard; palette hint row updated.
13. **Stop generation for chat** — `chat_cancel` command with a cancelled-turn
    registry checked in the SSE loop, `cancelled` chat-event kind, and a Stop
    button in the sidebar while a turn is streaming.
14. **Restore open tabs per vault across sessions** — persist user-opened tabs,
    pins and the active tab to `localStorage` keyed by vault root; restore on
    bootstrap, skipping pages that no longer exist.
15. **Word count + reading time in the editor** — footer on each editor pane
    showing words / characters / est. reading minutes for the current body
    (frontmatter excluded), with a unit-tested counting helper.
