# Skein — Design Notes

A local note-taking app with full-text + semantic search, auto-tagging, image embeds, markdown, import/export, and an embedded Claude chat. Linux and Windows.

## Core stack

**Decision: Tauri (Rust core) + Svelte frontend.**

- Tauri gives ~10MB installers, low RAM, native filesystem access via Rust, and a clean Linux+Windows story.
- Svelte chosen for the UI: small runtime, fast, plays nicely with Tauri's IPC model, and the editor libraries we'll likely use (CodeMirror 6) are framework-agnostic so Svelte is no obstacle.
- Accept the known tradeoff: webview drift between WebKitGTK (Linux) and WebView2 (Windows). Plan to QA both per release; avoid bleeding-edge CSS.
- Rejected: Electron (binary size, RAM); native Qt/GTK (dev cost).

### Storage shape

The most consequential decision. Markdown files on disk as the source of truth + a disposable SQLite index. Obsidian/Logseq pattern.

**Vault layout — books and pages:**

- A **page** is a single `.md` file.
- A **book** is a folder in the vault containing pages (and optionally attachments).
- Pages may live at the vault root (top-level, not in any book) or inside a book folder.
- One level of nesting in v1 — books do not contain books. Revisit if users ask for it.

**Attachments:**

- Live **beside the page they were first inserted into**: vault root for top-level pages, inside the book folder for pages in a book.
- Filenames are content-addressed (hash-prefixed) to dedupe within a folder.
- If the same image is pasted into two pages in different books, it gets stored twice. Acceptable cost for the locality benefit (moving/deleting a book takes its assets with it).
- Reference cleanup: when a page is deleted, scan remaining pages in the same folder for refs to its attachments; if none reference an attachment, delete it.

**Index location:**

- SQLite index lives in the **OS app data dir** (e.g. `~/.local/share/skein/` on Linux, `%APPDATA%\skein\` on Windows), not inside the vault.
- Per-machine, disposable, rebuilt from the vault on demand. Keeps the vault clean for git/Dropbox/etc.

**File watcher:**

- Reactive reload via `notify` (Rust) — external edits (user opens a page in another editor, or sync writes a file) trigger re-index of just the changed file.
- Debounced (e.g. 500ms) to coalesce rapid writes.

**Metadata format:**

- YAML frontmatter at the top of each `.md` for tags, created/updated timestamps, and other metadata. Obsidian-compatible, portable, standard.
- Body of the file is plain markdown — no Skein-specific extensions that would break in other editors.

## Embeddings & auto-tagging

**Embeddings model:**

- Default local model: **BGE-small-en-v1.5** via ONNX (~130MB, 384-dim, CPU). Better benchmark quality than MiniLM at similar size.
- Optional remote backend: **Voyage** (user provides API key in settings). Higher quality vectors when available; falls back to local BGE when offline or no key.
- Vectors are stored in `sqlite-vec` with a column tracking which model produced them — switching models triggers a reindex rather than mixing dimensionalities.

**Chunking:**

- Pages are chunked **by markdown section** (split on `#`/`##`/`###` headings). Each section becomes one embedded chunk; the whole-page vector is the average of its chunks.
- This gives both: page-level "related notes" (via the page-level vector) and precise RAG retrieval ("quote the paragraph about X") for the Claude chat box.
- Sections without headings get a single page-level chunk. Very long sections get further split at paragraph boundaries with a soft cap (~500 tokens).

**Auto-tagging:**

- Powered by **Claude Haiku** on save, debounced ~3s after last keystroke.
- Suggests 1–5 tags as chips the user accepts with a click. Existing vault tags are preferred to keep the namespace tight.
- Activates only when an Anthropic API key is present in settings. No key = no auto-tagging (manual tagging always works).

**API key model:**

- App is fully usable with **zero API keys**: local BGE embeddings power search and related-notes.
- **Voyage API key** unlocks higher-quality embeddings.
- **Anthropic API key** unlocks auto-tagging and the Claude chat box.
- Keys stored in OS keychain (libsecret on Linux, Credential Manager on Windows), never in plaintext on disk.

**Related-notes UX:**

- Sidebar pane on each page showing top-K similar pages by cosine similarity, refreshed on save.
- Click to navigate; pin to keep a related note visible alongside the editor.

## Claude chat box

**Activation:** requires an Anthropic API key in settings. Without a key, the chat sidebar is hidden/disabled.

**Context modes** (selectable per-conversation, default in bold):

- _Current note only_ — narrow scope, cheap.
- **_Current note + top-K related chunks via vector search (RAG)_** — default. Earns the chunked embeddings their keep.
- _Whole vault_ — power users; burns tokens but answers cross-cutting questions.

**RAG parameters:**

- Top 8 chunks retrieved across the vault (not just the current page), tunable in settings.
- Retrieved context block is assembled into a single system message section so it caches cleanly.

**Prompt caching:**

- Anthropic SDK with prompt caching on the system prompt + retrieved context block, scoped per-conversation.
- Vault-wide caching rejected: context changes too often to be worth the complexity.

**Models:**

- Default: **Haiku 4.5**.
- Selectable per-conversation from a dropdown in the chat header: Haiku 4.5, Sonnet 4.6, Opus 4.7. Users can escalate mid-thread.

**Streaming:** token streaming in the UI via Anthropic SDK streaming.

**Persistence:**

- Chats are **not** auto-saved as notes. Conversations live in their own ephemeral chat log; the user explicitly drags text out into a note when they want to keep it (see UI section).
- Chat history per session is retained in the app data dir for resume, but is not part of the vault and not indexed for search.

**v1 scope cut:** no tool use / agentic features. Chat reads via RAG only — it does not write or edit notes. Revisit in v2 once undo/safety story is designed.

## UI / app shell

The window is divided into four regions:

1. **Top menu bar** — file/edit/view/help, settings, vault switcher, command palette trigger.
2. **Bookshelf** (below menu bar) — two rows of labeled book spines representing the books in the vault. Empty slots remain visible where rows aren't full, reinforcing the bookshelf metaphor. Click a spine to open/expand the book; spines show the book's title vertically.
3. **The desk** (center, below the shelf) — the working surface. Tabs at the top of the desk for currently open pages. Tabs can be pinned to **left** or **right**, allowing up to two pages displayed side-by-side (split view). Markdown editor with live preview in each tab.
4. **Right sidebar — Claude chat** — conversation log on top, input box pinned at the bottom (VSCode LLM chat pattern). Model picker and context-mode selector in the chat header. Sidebar can be collapsed when no API key is set or when the user wants more desk real estate.

**Loose pages — the Folio:**

- A pseudo-book on the shelf labeled **"Folio"** (or similar) that contains all top-level pages — pages living at the vault root, not bound to any book. Opens like any other book.
- Empty-state behavior: when no book is open and no page is active, loose pages appear **scattered on the desk** as visible page-cards the user can click to open. Reinforces the "papers on a desk" metaphor.

**Bookshelf overflow:**

- More than two rows is fine — the shelf expands into additional shelves. Vertical scroll on the shelf area when rows exceed the visible space. Mirrors a real bookshelf with more shelves below.

**Cross-pane interaction:**

- **Drag-select-to-insert:** when a note is open and the user click-drag-selects text in the chat log, releasing the drag inserts that text into the open note at the typing cursor's current position. This replaces auto-saving chats — it makes the user's curation explicit and frictionless.
- Selection in the chat log otherwise behaves like normal text selection (copy, etc.). The drag-to-insert behavior triggers only when the drop target is the editor pane.
- Visual affordance during drag: cursor changes, editor shows an insertion indicator at the current caret.
- With split view, the drop target is whichever pane the cursor hovers over on release.

### Visual design system (locked)

The canonical visual reference is the mockup bundle at `design/mockups/Skein Mockups.html` (and the JSX components it loads). Implementation must match these mockups pixel-faithfully; the Tauri/Svelte port should treat the JSX as a spec, not a structure to clone.

**Headline mockups** in the bundle:

1. **Populated · split view · dark** — two pages pinned (left + right) on the desk, full bookshelf, chat sidebar open with an in-progress conversation.
2. **Drag-to-insert · mid-drag** — chat sentence selected (blue text-selection), drag preview floating near the cursor with an `↳ insert at caret` ribbon, accent-amber caret indicator showing the insertion point in the open page.
3. **Empty state · loose pages on the desk** — no tabs open, scattered page-cards (slightly rotated, varied positions) over a soft radial vignette, with the hint "_The desk is clear._ Open a book from the shelf, or pick a loose page below."

**Locked aesthetic decisions:**

- **Platform chrome:** GNOME-style headerbar (close button on the right, traffic lights hidden). Same component on Windows.
- **Default theme:** dark (warm graphite paper, umber wood). Light theme (cream paper, oak wood) is a first-class equal, toggled in settings.
- **Shelf realism:** **suggestive** — soft wood gradient with flat spines and a thin colored cloth band. Abstract (chrome bar) and tactile (planked wood, deeper shadow) are available as settings, but suggestive is the default.
- **Folio:** stack of papers tied with twine, sitting at the _start_ of the shelf (first slot, before the first book).
- **Spine treatment:** varied heights (74–84px), warm-wood backgrounds shifted by hue, a thin cloth-bound color band ~14px tall placed near the top, vertical title rendered below the band in `Source Serif 4`, uppercase, letter-spaced.
- **Cloth-band palette:** six muted hues sharing chroma 0.06–0.09 — terracotta, moss, slate, ochre, oxblood, dusty teal. Rotated across books deterministically (e.g. by title hash) so reordering doesn't reshuffle colors.
- **Active book affordance:** spine pulled forward (translateY −6px) with a soft amber glow.

**Type pairing:**

- **UI chrome:** Inter (400/500/600/700). Screen-tuned, clean.
- **Page content:** Source Serif 4 (modern literary serif, screen-optimized) by default. Settings let users swap to Iowan Old Style, Spectral, or Lora.
- **Code / monospace:** JetBrains Mono.

**Color tokens** (paste these verbatim into the Svelte port; full definitions in `design/mockups/skein-app.jsx`, top of `SKEIN_CSS`):

- Wood: `--wood-1`, `--wood-2`, `--wood-3`, `--wood-edge`, `--wood-shadow`.
- Paper / page: `--paper`, `--paper-2`, `--page`, `--page-edge`.
- Ink: `--ink`, `--ink-2`, `--ink-3`, `--ink-4` (descending emphasis).
- Chrome: `--chrome`, `--chrome-2`, `--chrome-edge`, `--chrome-ink`, `--chrome-ink-2`.
- Accent (warm amber, used sparingly): `--accent`, `--accent-soft`, `--accent-edge`.
- Chat bubbles: `--user-msg`, `--asst-msg`.
- Page shadow: `--shadow-page`.

**Dimension constants** (also from the bundle CSS):

- Window: 1480×920 reference artboards. App scales fluidly; these are the design canvas size.
- Titlebar: 38px tall.
- Sidebar (open): 340px wide. Collapsed: 36px. Hidden: 0.
- Tabs row: 34px tall, max tab width 200px.
- Shelf: rows 92px tall with an 18px gap and a planked plinth between/below each row. Two visible rows by default; vertical scroll reveals more.
- Spine: ~28px wide (22px in narrow mode), 74–84px tall.
- Folio: 38×80px.
- Empty-state cards: 200×132px, with a `-webkit-line-clamp: 5` body and a meta footer (tag + age).

**Component inventory** (Svelte components to mirror the JSX):

`Titlebar` · `Bookshelf` · `Spine` · `Folio` · `Desk` · `Tabs` · `Tab` · `Page` (with `live-preview` editor inside) · `EmptyDesk` (cards) · `Sidebar` (open / collapsed / hidden) · `ChatHeader` · `ChatLog` · `ChatMessage` · `ChatInput` · `DragOverlay` (drag-preview + cursor icon) · `Caret` (regular blink + accent insert).

**Settings-exposed tweaks** (mirroring the design's Tweaks panel):

- Theme: dark / light.
- Shelf realism: abstract / suggestive / tactile.
- Sidebar: open / collapsed / hidden.
- Page font: Source Serif 4 / Iowan Old Style / Spectral / Lora.

The mockup also exposes a "Scenario" tweak (populated / dragging / empty) — that is for design preview only; the app derives the live state from real vault contents and user interaction.

## Features that compound well

**In v1:**

- **Wiki-style `[[backlinks]]`** — type `[[` to autocomplete page titles. Linked page gets a "Linked from" panel showing all references. Pairs naturally with vector similarity: explicit links + implicit (embedding-based) links shown side-by-side in the related-notes pane.
- **Daily notes / journal** — templated page per day, lives in a `Daily/` book by default. Configurable template (frontmatter + body skeleton). Time-zone-aware date rollover.
  - **OS notifications/reminders:** users can configure reminder times for daily notes (e.g. "remind me at 9am to write today's entry") and ad-hoc reminders attached to a page or task. Delivered via native OS notifications (Tauri's notification API, libnotify on Linux, Windows toast).
- **Paste-image-from-clipboard** — auto-saved as a hash-named file in the appropriate folder per the attachments rules (vault root for top-level pages, book folder for pages in a book).
- **Git-backed sync** — optional, off by default. Settings UI to configure a remote, view sync status, and trigger pull/push manually. Conflict resolution: leave the file with conflict markers and surface it in the UI for the user to resolve. No auto-merge magic.

**Cut from v1:**

- **Global hotkey quick capture** — owner is not a hotkey-heavy user. Defer.

**To be discussed (not yet decided):**

- **Optional encryption at rest** — desired so users can keep passwords/API keys/sensitive notes in the vault. Candidate approach: passphrase-secured GPG, or age with a passphrase. Open questions: per-page or whole-vault encryption, key derivation/recovery story, how encrypted pages interact with search and embeddings (likely: skipped from index unless unlocked in-session). Defer detailed design to a later pass.

**Cut entirely (not v1, not roadmapped):**

- Graph view — flashy, low utility for the bookshelf metaphor.
- Web clipper — separate browser extension scope.
- Voice notes / transcription — out of scope.

## Backup / export / import

**Export:**

- Primary export: **zipped vault** — markdown files + attachments preserved as-is.
- **Embeddings sidecar:** the archive also includes a portable embeddings file (e.g. `.skein/vectors.db` inside the zip — a stripped SQLite file containing only the `vectors` table, keyed by **content hash** rather than file path). Vectors cost real money via Voyage and meaningful CPU time via local BGE; preserving them across machines and restores is worth the extra ~10–20% archive size.
- The full SQLite index (FTS5, tag tables, etc.) is **not** exported — it's cheaply rebuilt from the markdown. Only embeddings are precious enough to capture.
- On import/restore, the indexer hashes each page's content; if the hash matches a vector in the sidecar, reuse it. Otherwise re-embed only the changed/new pages. Cheap restores, resilient to offline edits.

**Import sources:**

- **Obsidian** — point the app at the vault folder, copy-or-link, run the indexer. Frontmatter and `[[wikilinks]]` are already compatible.
- **Plain folder of markdown** — pick any folder; subdirectories become books, top-level `.md` files become loose pages. Covers random `.md` dumps and exports from tools we don't explicitly support.
- **OneNote** — _not in v1._ See note below.
- Notion, Bear, etc. — deferred to v2+.

**Import collision handling:**

- If a page already exists in the target vault, **rename the incoming file** (`page (1).md`) by default, with a per-file override (skip / overwrite / rename) in the import dialog.

**Restore from archive:**

- One-click **"Open Vault from Archive"** flow: select `.zip`, app unzips to a chosen location, loads the embeddings sidecar, and rebuilds the rest of the index in the background. Single action, no manual rebuild step.

**On OneNote import (deferred):**

- OneNote uses a proprietary format (`.one` files, or hosted-only on OneDrive for the modern app). Two viable paths, both meaningful work:
  1. **Microsoft Graph API** — OAuth flow, fetch notebooks/sections/pages as HTML, convert HTML→markdown, download embedded images. Best fidelity, requires registering an Azure app and handling auth.
  2. **User-driven HTML/PDF export from OneNote first**, then import as a folder of markdown via the plain-folder importer. Lower fidelity (loses some structure), but no auth dance.
- Realistic answer: **possible, but a v2 feature**. Path 1 is the right approach when we get there. Flagged for a future roadmap pass.

## Naming

**Skein** — a coil of yarn. Tactile, unusual, evokes the threaded/woven nature of an interlinked note vault. Short, pronounceable, memorable.

## Implementation plan

A phased build, each phase ending in something runnable. Phases land in order; later phases assume the earlier ones. Each phase should ship behind a real demo: open the app, click around, see the new capability work end-to-end.

### Phase 0 — Bootstrap

- Tauri 2.x project (Rust backend) + Svelte 5 + Vite + TypeScript front-end.
- Workspace layout: `src-tauri/` (Rust), `src/` (Svelte/TS), `design/` (mockups + this doc), `docs/` (user-facing later).
- Tooling: `cargo fmt` + `clippy`, `prettier` + `eslint` for TS/Svelte, `pnpm` (or `npm`) for JS deps. Pre-commit hooks via `lefthook` or similar.
- License: MIT (`LICENSE` + SPDX headers in source files).
- README skeleton describing what Skein is and how to run it locally.
- CI: GitHub Actions building debug bundles for **Linux (Ubuntu 22.04+)** and **Windows (windows-latest)** on push and PR. No release artifacts yet.
- Acceptance: `pnpm tauri dev` opens an empty Tauri window on both platforms.

### Phase 1 — UI shell (visual mockup, no real data)

- Port `design/mockups/skein-app.jsx` to Svelte components, matching the locked visual design system above pixel-faithfully.
- All four regions: Titlebar, Bookshelf (suggestive style by default), Desk (with tabs + split view), Sidebar (open/collapsed/hidden modes).
- All three scenarios reachable via a temporary dev menu: populated split, drag-to-insert mid-state, empty desk with cards.
- Theme toggle (dark / light) wired to a Svelte store. Page font selectable.
- Mock data only — no filesystem reads yet. Books, pages, chat, cards are hardcoded fixtures.
- Acceptance: side-by-side comparison with the mockup bundle shows no visible drift.

### Phase 2 — Vault model + filesystem layer

- Settings flow: pick a vault folder on first run, persist the path.
- Rust scanner walks the vault: top-level dir = vault root; sub-dirs = books; `.md` files = pages. Top-level `.md` files become loose pages (Folio).
- YAML frontmatter parser (`serde_yaml` or `gray_matter`).
- Reactive file watcher (`notify` crate) → debounced (~500ms) → emit IPC events to the front-end on create / modify / delete.
- Tauri commands: `list_books`, `list_pages`, `read_page`, `write_page`, `create_page`, `delete_page`, `create_book`, `move_page`.
- Bookshelf and tabs now render real vault contents. Pages open as read-only previews.
- Acceptance: create a folder of `.md` files outside the app; opening it as a vault populates the bookshelf, and edits made in another editor reflect live.

### Phase 3 — Editor (CodeMirror 6 + live preview)

- CodeMirror 6 integrated as a Svelte component, one editor instance per open tab.
- Markdown language pack, Obsidian-style live-preview overlay (raw markdown around the cursor; rendered output everywhere else).
- Wire to the filesystem layer: load on tab open, save on debounced change (~750ms after last keystroke), also save on tab close / blur.
- Dirty indicator on tabs (the `.dirty` dot in the mockup).
- Pin-to-left / pin-to-right tab actions; max two panes shown.
- Acceptance: edit a note in-app, alt-tab to a terminal, see the file content updated. Vice versa.

### Phase 4 — Indexing (FTS5)

- SQLite database in OS app data dir (`~/.local/share/skein/index.db` on Linux, `%APPDATA%\skein\index.db` on Windows). Disposable.
- Schema: `pages` (path, hash, title, frontmatter JSON, mtime), `tags`, `links`, `pages_fts` (FTS5 virtual table).
- Initial scan on vault open; incremental updates from the file watcher.
- Search command palette in the titlebar (`Ctrl+P`-style, or click the search icon).
- Acceptance: type in the search bar, results stream in; FTS matches highlighted.

### Phase 5 — Embeddings + related notes

- Bundle ONNX runtime + **BGE-small-en-v1.5** model (~130MB) shipped with the app.
- Section-based chunker (split on `#`/`##`/`###`; soft cap ~500 tokens per chunk; further-split overlong sections at paragraph boundaries).
- `sqlite-vec` extension loaded into the SQLite DB. `vectors` table keyed by content hash, stores chunk vectors + page-level vector (mean of chunks).
- Reindex flow: on page write, hash content; if hash unchanged, skip; else re-chunk, re-embed, replace vectors.
- Settings toggle for **Voyage** as a remote backend (uses key from keychain, see Phase 7). Switching backends triggers a full reindex with the new dimensionality.
- Related-notes pane next to the editor: top-K cosine matches refreshed on save.
- Acceptance: open a note, see semantically related notes in the sidebar; edit and save, list updates.

### Phase 6 — Settings + secrets

- Settings page UI: vault path, theme, shelf style, sidebar default, page font, embeddings backend, API keys (Anthropic, Voyage).
- API keys stored via the OS keychain (`keyring` crate: libsecret on Linux, Windows Credential Manager on Windows). Never written to plaintext config.
- Non-secret config in a TOML file at the OS config dir.
- Acceptance: keys set in settings persist across restarts and aren't visible in any plaintext file.

### Phase 7 — Claude chat sidebar

- Anthropic SDK (Rust client or HTTP via `reqwest`) with **prompt caching** on the system prompt + retrieved context block.
- Three context modes: current note, current + RAG (top-8 chunks vault-wide), whole vault.
- Streaming responses rendered token-by-token in the chat panel.
- Model picker pill (Haiku 4.5 default; Sonnet 4.6 / Opus 4.7 selectable, per-conversation).
- **Drag-to-insert**: chat-log selection + drag → drop on editor pane → text inserted at caret. Honor split view (drop target = pane under cursor on release).
- Conversation persisted per-session in app data dir for resume; not part of the vault, not indexed.
- Acceptance: ask a question grounded in vault content, drag a paragraph from the answer into a note.

### Phase 8 — Auto-tagging

- Debounced (~3s after last keystroke) call to **Claude Haiku** with a small prompt that includes the page body and the vault's existing tag list.
- Returns 1–5 tag suggestions, shown as chips above the editor; click to accept (writes to frontmatter).
- Disabled cleanly when no Anthropic key is set.
- Acceptance: write a note about cooking, see `#recipes` and similar suggestions.

### Phase 9 — Backlinks

- `[[wikilink]]` parser + autocomplete on `[[` (uses page index from Phase 4).
- "Linked from" panel on each page: live list of pages whose body contains a wikilink to this one.
- Combined with Phase 5 in the side pane: explicit links above, implicit (embedding) related below.
- Acceptance: type `[[`, autocomplete; visit the linked page, see the source listed under "Linked from".

### Phase 10 — Daily notes + reminders

- Daily-note creation: open today's daily page (creates from template if missing). Lives in a `Daily/` book by default; configurable.
- Template editor in settings (frontmatter + body skeleton with `{{date}}`, `{{weekday}}` variables).
- OS notifications via Tauri's notification API: configurable reminder times for daily notes; ad-hoc reminders attachable to any page.
- Time-zone-aware date rollover.
- Acceptance: schedule a 9am reminder, receive a native OS notification.

### Phase 11 — Paste image

- Clipboard-image handler in the editor: paste → save as `<hash>.<ext>` to the page's folder per the attachment rules → insert markdown ref at caret.
- Drag-drop image files into the editor: same path.
- Acceptance: paste a screenshot into a page; the file lands beside the page and renders in live preview.

### Phase 12 — Backup / export / import

- Export: zip vault directory + a content-hash-keyed embeddings sidecar (`.skein/vectors.db` inside the archive). One-click "Export Vault" in settings.
- "Open Vault from Archive": pick `.zip`, choose destination, app unzips, loads the embeddings sidecar, kicks off background indexing for non-vector data.
- Import: **Obsidian** (point at folder, no transformation needed) and **plain folder of markdown** (subdirs → books, top-level `.md` → loose pages). Collision: rename incoming with `(1)` suffix; per-file override in import dialog.
- Acceptance: round-trip a vault through export → restore, on a fresh machine, with no re-embedding cost.

### Phase 13 — Git sync

- Embed `git2` (libgit2) for in-process git ops.
- Settings: configure remote URL, auth (SSH key or token via keychain), choose branch.
- Buttons: Pull, Push, Status. No auto-merge magic.
- On conflicts: leave the file with conflict markers, surface conflicted files in a panel; user resolves in-editor.
- Acceptance: connect to a remote, edit on machine A, pull on machine B, see the change.

### Phase 14 — Polish + packaging

- Linux: AppImage (primary), `.deb`, `.rpm`. Wayland + X11 tested on GNOME and KDE.
- Windows: MSI via Tauri's WiX bundler.
- Code signing: **defer** until we decide whether to publish (cost / certificate management). Builds will be unsigned during dev.
- Auto-update: Tauri updater pointed at GitHub Releases.
- Crash reporting: opt-in, local log to app data dir; remote reporting deferred.
- Onboarding: first-run flow guides vault creation, optional API key setup.
- Acceptance: a friend on stock Ubuntu and a friend on Windows 11 can install from a GitHub release and run through the daily-notes happy path without help.

### Cross-cutting work (lands incrementally)

- **Accessibility:** keyboard navigation through tabs, shelf, chat. Screen-reader labels on icon-only controls. Color contrast verified in both themes (target WCAG AA on UI chrome).
- **Performance budgets:** indexing 10k pages should complete in < 2 minutes on a mid-range laptop; incremental updates < 100ms. Editor open-to-typeable < 250ms. Chat first-token < 1.5s when the cache is warm.
- **Telemetry:** none in v1. The app is local-first and offline-capable; phone-home is opt-in only and deferred past v1.
- **Testing:** Rust unit tests for the indexer/chunker/parser; Playwright (or WebDriver-via-Tauri) smoke tests for the major flows; visual regression tests against the Phase 1 mockup.

### Out of plan (deferred features parked elsewhere in this doc)

- Encryption at rest — separate design pass.
- OneNote / Notion / Bear importers — v2.
- Tool-use / agentic chat — v2 with an undo/safety story.
- Global hotkey quick capture — owner doesn't want it.
- Graph view / web clipper / voice notes — cut.

## Resolved decisions (v1)

- **Editor view**: tabs at top of the desk, with left/right pin allowing up to two side-by-side panes. Split view in v1; no further multi-pane.
- **Editor surface**: **CodeMirror 6** with a live-preview overlay (Obsidian-style: formatted text rendered, raw markdown revealed around the cursor). Pairs well with markdown-on-disk and handles large files cheaply.
- **Indexer**: incremental updates via the file watcher (debounced ~500ms). Full reindex available as a manual command for recovery / model swaps. No batch-mode-by-default; the watcher handles the steady state.
- **API key storage**: Anthropic and Voyage API keys stored in **app settings** (encrypted via the OS keychain — libsecret on Linux, Credential Manager on Windows). Plain-text settings file holds non-secret config only.
- **Encrypted-notes auth** (passphrase / GPG / etc.): out of scope here; covered by the deferred "encryption at rest" pass.
- **License**: **MIT.**

## Still to design (later passes)

- Encryption at rest: per-page vs whole-vault, key derivation, recovery, search/embedding interaction.
- v2 importers: OneNote (via Microsoft Graph), Notion, Bear.
- v2 chat features: tool use / agentic editing, with an undo + safety story.
- Bookshelf visual design: spine colors/labels, default book ordering, drag-to-reorder.
- Daily-notes template editor and reminder configuration UI.
- Git sync conflict-resolution UI.
