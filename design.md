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

- *Current note only* — narrow scope, cheap.
- ***Current note + top-K related chunks via vector search (RAG)*** — default. Earns the chunked embeddings their keep.
- *Whole vault* — power users; burns tokens but answers cross-cutting questions.

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
- **OneNote** — *not in v1.* See note below.
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
