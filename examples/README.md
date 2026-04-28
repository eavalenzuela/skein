# Example vault

A small sample vault for kicking the tyres on Skein during development.

## Contents

```
vault/
├── Daily/                   # 2 daily notes
├── Reading Notes/           # 2 reading-note pages
├── Recipes/                 # 1 recipe
├── Project Ideas/           # 1 idea
├── Garden/                  # 1 garden plan
├── Skein — naming.md        # loose page (Folio)
└── Walk, Sat morning.md     # loose page (Folio)
```

Pages have YAML frontmatter (`title`, `tags`, etc.) and use `[[wikilinks]]` to cross-reference. The intent is to exercise:

- Books (subdirectories) and loose top-level pages.
- Frontmatter parsing (titles, tag lists).
- Pages with the design's literary voice so the typography choices are honest.

## Use it

```bash
npm run tauri dev
```

Then in the vault picker, choose `examples/vault/` from this repo.

The vault is plain markdown. Edit any file in another editor and the app will refresh live (the file watcher emits a `vault-changed` event with a 500 ms debounce).
