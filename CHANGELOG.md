# Changelog

## Unreleased

An adversarial review pass — visual design, security, and usability
reviewers working against the running app, each finding independently
checked by two skeptics before being acted on — followed by the fixes and
the features the gaps pointed at.

### Security

- **Vault containment.** One `resolve_in_vault` / `check_rel_path` helper now
  guards every filesystem command. It rejects traversal, absolute paths and
  symlinks — at the leaf *and* at every ancestor — before anything is created,
  so a refused write no longer leaves stray directories behind. Previously the
  write path canonicalized only the parent, so a vault cloned from a hostile
  git remote carrying `Note.md -> ~/.bashrc` would have been written through.
- `delete_book` skipped name validation entirely, so a book named
  `../Documents` reached `remove_dir_all`.
- **Attachments** are identified by magic bytes rather than a caller-supplied
  extension, capped at 64 MB, and refused unless they are real images. The
  vault is a git working tree and a shared zip.
- **Asset protocol** scoped to the open vault (was `**` — every file on the
  machine) and granted at vault-install time; image refs that are absolute or
  climb out with `..` are refused in the editor.
- An explicit **Content-Security-Policy** (was `null`).
- The **embeddings sidecar** in a restored zip is treated as untrusted: read
  through a read-only connection instead of `ATTACH`, integrity-checked, and
  rows with the wrong model or vector dimension dropped. A poisoned vector
  otherwise chooses which notes are fed to the chat.
- The **git token** is only offered to `https` endpoints, and credentials
  embedded in a pasted remote URL are moved to the keychain instead of being
  written to `settings.json` and `.git/config` in cleartext.
- **Retrieved note text** is fenced in per-request nonce tags and framed as
  data, so an imported note cannot issue instructions to the model.

New `src-tauri/tests/containment.rs` covers each escape.

### Fixed

- **A failed page read could destroy the file.** The error message was written
  into the tab's body while `saved` stayed empty, so the tab counted as dirty
  and autosave wrote that text over the real page. Tabs now carry the error
  separately, render read-only with a retry, and refuse to save.
- **Failed saves were silent.** A new toast bus reports them, with a retry.
- **The watcher wiped a whole book from the index on every edit** — found by
  running the real binary against a real vault. Any path that wasn't a file was
  treated as deleted, and a directory isn't a file, so writing a page swept its
  book out of search until a manual rebuild.
- **The chat transcript couldn't scroll**; everything past one screenful was
  unreachable.
- **The empty desk showed five fabricated notes** from the design mockup — notes
  the user never wrote, which did nothing when clicked.
- **Small functional text failed contrast** at roughly 1.8:1. `--ink-3` now
  clears WCAG AA 4.5:1 in both themes and carries that text; `--ink-4` is
  decorative only.
- **Spine titles were clipped mid-letter** and ran across the cloth band; the
  shelf reserved a second row of bare wood regardless of content (~28% of an
  800px window). Titles now fit, and the second row appears only when needed.
- The Sync fields rendered as **native white boxes** inside the dark Settings
  modal; the add-book slot was invisible on the light shelf.
- The model and context **pills carried dropdown chevrons but silently cycled**
  on click — a click meant to see the options could land on Opus.
- **Palette arrow keys didn't scroll** the selection into view.
- **Escape stopped closing Settings** after clicking any button that disables
  itself, because focus moved to `<body>` and the handler was on the overlay.
- Help pointed at the wrong repository and used `window.open`, a no-op in the
  Tauri webview.
- The **DevControls mockup panel** and the first-run "design preview" link
  shipped to release builds; the preview had no way back.

### Added

- **Trash with undo.** Deletes move to `.skein/trash/` (30-day retention)
  instead of being permanent, with an Undo prompt and a Settings panel.
- **Auto-tagging is opt-in.** It previously uploaded a page's text to Anthropic
  a few seconds after you stopped typing, activated by nothing more than having
  a key configured for chat, with no way to turn it off. Now off by default and
  enforced in the backend, alongside a Privacy panel stating what leaves the
  machine.
- **The command palette opens on recent pages** instead of a blank box, and
  explains the `:` and `#` prefixes on a fresh vault.
- **Chat: start a new conversation, or save one as a page** — the transcript
  becomes an ordinary note that indexes, embeds and links like any other.
- **A keyboard-shortcut overlay** (<kbd>Ctrl</kbd>+<kbd>/</kbd>) generated from
  one registry, replacing a `window.alert` list that had drifted.
- **A themed new-page dialog** replacing `window.prompt` at three call sites.
- **An unsaved-changes guard** on Settings, which used to discard Sync and
  Daily-note edits without a word.
- `scripts/rust-env.sh` builds the Rust crate against a GNOME flatpak SDK when
  distro GTK dev packages aren't available.
