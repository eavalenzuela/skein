// Browser-side Tauri mock. Installed via page.addInitScript before
// SvelteKit/Tauri code runs. Provides an in-memory vault and stubs the
// invoke + event surfaces that the Skein frontend uses.

export interface MockOptions {
  hasVault?: boolean;
  vaultName?: string;
  vaultRoot?: string;
}

export function tauriMockScript(opts: MockOptions = {}): string {
  const cfg = {
    hasVault: opts.hasVault ?? true,
    vaultName: opts.vaultName ?? "Test Vault",
    vaultRoot: opts.vaultRoot ?? "/tmp/test-vault",
  };
  // The body below runs in the browser. It receives `__CFG__` injected
  // via string replacement.
  const body = String(installMock).replace(/__CFG__/g, JSON.stringify(cfg));
  return `(${body})();`;
}

// This function is serialized to the page; it must be self-contained.
// It cannot reference outer-scope variables.
function installMock() {
  const cfg: any = __CFG__;

  type Book = { name: string; rel_path: string; page_count: number };
  type Page = {
    rel_path: string;
    title: string;
    book: string | null;
    tags: string[];
    modified: number;
  };

  const state = {
    vault: cfg.hasVault
      ? { root: cfg.vaultRoot, name: cfg.vaultName }
      : (null as { root: string; name: string } | null),
    books: [
      { name: "Research", rel_path: "Research", page_count: 2 },
      { name: "Daily", rel_path: "Daily", page_count: 3 },
      { name: "Recipes", rel_path: "Recipes", page_count: 1 },
    ] as Book[],
    bookOrder: ["Research", "Daily", "Recipes"],
    pages: new Map<string, Page>([
      [
        "Research/alpha.md",
        {
          rel_path: "Research/alpha.md",
          title: "Alpha note",
          book: "Research",
          tags: ["tag-a"],
          modified: 1700000000,
        },
      ],
      [
        "Research/beta.md",
        {
          rel_path: "Research/beta.md",
          title: "Beta note",
          book: "Research",
          tags: [],
          modified: 1700000100,
        },
      ],
      [
        "Daily/2026-04-29.md",
        {
          rel_path: "Daily/2026-04-29.md",
          title: "2026-04-29",
          book: "Daily",
          tags: [],
          modified: 1714000000,
        },
      ],
      [
        "Daily/2026-04-28.md",
        {
          rel_path: "Daily/2026-04-28.md",
          title: "2026-04-28",
          book: "Daily",
          tags: [],
          modified: 1713000000,
        },
      ],
      [
        "Daily/2026-04-27.md",
        {
          rel_path: "Daily/2026-04-27.md",
          title: "2026-04-27",
          book: "Daily",
          tags: [],
          modified: 1712000000,
        },
      ],
      [
        "Recipes/cake.md",
        {
          rel_path: "Recipes/cake.md",
          title: "Cake",
          book: "Recipes",
          tags: [],
          modified: 1710000000,
        },
      ],
      [
        "loose-thought.md",
        {
          rel_path: "loose-thought.md",
          title: "Loose thought",
          book: null,
          tags: [],
          modified: 1709000000,
        },
      ],
    ]),
    pageBodies: new Map<string, string>(),
    settings: {
      vault_path: cfg.vaultRoot,
      theme: "warm",
      shelf_style: "leather",
      sidebar: "left",
      page_font: "serif",
      daily_book: "Daily",
      daily_template: null,
      daily_reminder_time: null,
      git_remote_url: null,
      git_branch: null,
      git_auth_kind: null,
      chat_model: "sonnet-4.6",
      chat_context_mode: "auto",
    },
    secrets: new Set<string>(),
    eventListeners: new Map<string, Map<number, (e: any) => void>>(),
    nextEventId: 1,
    callbacks: new Map<number, (v: any) => void>(),
    nextCallback: 1,
  };

  function emit(event: string, payload: any) {
    const listeners = state.eventListeners.get(event);
    if (!listeners) return;
    for (const cb of listeners.values()) cb({ event, payload, id: 0 });
  }

  const handlers: Record<string, (args: any) => any> = {
    // Vault
    current_vault: () => state.vault,
    open_vault: ({ path }) => {
      state.vault = { root: path, name: path.split("/").pop() || "vault" };
      return state.vault;
    },
    close_vault: () => {
      state.vault = null;
    },
    list_books: () =>
      state.bookOrder
        .map((n) => state.books.find((b) => b.name === n))
        .filter(Boolean),
    create_book: ({ name }) => {
      if (!state.books.find((b) => b.name === name)) {
        state.books.push({ name, rel_path: name, page_count: 0 });
        state.bookOrder.push(name);
      }
      emit("vault-changed", null);
    },
    rename_book: ({ oldName, newName }) => {
      const b = state.books.find((b) => b.name === oldName);
      if (b) {
        b.name = newName;
        b.rel_path = newName;
        const i = state.bookOrder.indexOf(oldName);
        if (i >= 0) state.bookOrder[i] = newName;
        // rewrite page rel_paths
        for (const p of state.pages.values()) {
          if (p.book === oldName) {
            p.book = newName;
            const newRel = p.rel_path.replace(`${oldName}/`, `${newName}/`);
            state.pages.delete(p.rel_path);
            p.rel_path = newRel;
            state.pages.set(newRel, p);
          }
        }
      }
      emit("vault-changed", null);
    },
    delete_book: ({ name, alsoDeletePages }) => {
      const deleted: string[] = [];
      const moved: [string, string][] = [];
      for (const p of [...state.pages.values()]) {
        if (p.book === name) {
          if (alsoDeletePages) {
            state.pages.delete(p.rel_path);
            deleted.push(p.rel_path);
          } else {
            const newRel = p.rel_path.split("/").pop()!;
            state.pages.delete(p.rel_path);
            p.book = null;
            const oldRel = p.rel_path;
            p.rel_path = newRel;
            state.pages.set(newRel, p);
            moved.push([oldRel, newRel]);
          }
        }
      }
      state.books = state.books.filter((b) => b.name !== name);
      state.bookOrder = state.bookOrder.filter((n) => n !== name);
      emit("vault-changed", null);
      return { deleted_rel_paths: deleted, moved };
    },
    set_book_order: ({ names }) => {
      state.bookOrder = names;
      emit("vault-changed", null);
    },
    create_page: ({ book, title }) => {
      const safe = title.replace(/[^a-zA-Z0-9 _-]/g, "");
      const rel = book ? `${book}/${safe}.md` : `${safe}.md`;
      state.pages.set(rel, {
        rel_path: rel,
        title: safe,
        book,
        tags: [],
        modified: Date.now() / 1000,
      });
      const b = state.books.find((bb) => bb.name === book);
      if (b) b.page_count += 1;
      emit("vault-changed", null);
      return rel;
    },
    rename_page: ({ relPath, newTitle }) => {
      const p = state.pages.get(relPath);
      if (!p) throw new Error("not found");
      const dir = relPath.includes("/") ? relPath.split("/").slice(0, -1).join("/") + "/" : "";
      const newRel = `${dir}${newTitle}.md`;
      state.pages.delete(relPath);
      p.rel_path = newRel;
      p.title = newTitle;
      state.pages.set(newRel, p);
      emit("vault-changed", null);
      return newRel;
    },
    delete_page_command: ({ relPath }) => {
      const p = state.pages.get(relPath);
      if (p?.book) {
        const b = state.books.find((bb) => bb.name === p.book);
        if (b) b.page_count = Math.max(0, b.page_count - 1);
      }
      state.pages.delete(relPath);
      emit("vault-changed", null);
    },
    list_loose_pages: () =>
      [...state.pages.values()].filter((p) => p.book === null),
    list_pages_in_book: ({ book }) =>
      [...state.pages.values()].filter((p) => p.book === book),
    read_page: ({ relPath }) => state.pageBodies.get(relPath) ?? "",
    write_page: ({ relPath, body }) => {
      state.pageBodies.set(relPath, body);
    },
    list_page_titles: () =>
      [...state.pages.values()].map((p) => ({
        rel_path: p.rel_path,
        title: p.title,
      })),
    find_backlinks: () => [],
    search_pages: ({ query }) => {
      const q = query.toLowerCase();
      return [...state.pages.values()]
        .filter(
          (p) =>
            p.title.toLowerCase().includes(q) ||
            (state.pageBodies.get(p.rel_path) || "")
              .toLowerCase()
              .includes(q),
        )
        .map((p) => ({
          rel_path: p.rel_path,
          title: p.title,
          book: p.book,
          snippet: p.title,
        }));
    },
    find_related: () => [],
    rebuild_index: () => state.pages.size,
    embedding_model_status: () => ({ name: "bge-small", local: false }),
    download_embedding_model: () => ({ name: "bge-small", local: true }),
    suggest_tags: () => [],
    apply_tag: ({ relPath, tag }) => {
      const p = state.pages.get(relPath);
      if (p && !p.tags.includes(tag)) p.tags.push(tag);
    },
    dismiss_tag: ({ relPath, tag }) => {
      const p = state.pages.get(relPath);
      if (p) p.tags = p.tags.filter((t: string) => t !== tag);
    },
    open_today_daily: () => {
      const rel = "Daily/today.md";
      let created = false;
      if (!state.pages.has(rel)) {
        state.pages.set(rel, {
          rel_path: rel,
          title: "today",
          book: "Daily",
          tags: [],
          modified: Date.now() / 1000,
        });
        created = true;
      }
      return { rel_path: rel, created };
    },
    save_attachment: () => "attachments/x.bin",
    save_attachment_from_path: () => "attachments/x.bin",
    export_vault: () => undefined,
    open_vault_from_archive: ({ archivePath, destDir }) => {
      state.vault = { root: destDir, name: "imported" };
      return state.vault;
    },

    // Settings
    get_settings: () => ({ ...state.settings }),
    set_settings: ({ patch }) => {
      Object.assign(state.settings, patch);
      return { ...state.settings };
    },
    has_secret: ({ name }) => state.secrets.has(name),
    set_secret: ({ name }) => {
      state.secrets.add(name);
    },
    clear_secret: ({ name }) => {
      state.secrets.delete(name);
    },

    // Git
    git_status: () => ({
      initialized: false,
      branch: null,
      remote_url: null,
      ahead: 0,
      behind: 0,
      dirty: [],
      conflicted: [],
    }),
    git_set_remote: () => undefined,
    git_pull: () => ({ kind: "up-to-date", conflicted: [] }),
    git_push: () => undefined,
    git_commit_all: () => true,

    // Chat
    chat_send: () => {
      const id = "turn-" + Math.random().toString(36).slice(2);
      setTimeout(() => emit(`chat-turn-${id}`, { kind: "done", text: "Mock response" }), 50);
      return id;
    },

    // Tauri event plugin
    "plugin:event|listen": ({ event, handler }: any) => {
      const id = state.nextEventId++;
      const cb = state.callbacks.get(handler);
      if (cb) {
        let listeners = state.eventListeners.get(event);
        if (!listeners) {
          listeners = new Map();
          state.eventListeners.set(event, listeners);
        }
        listeners.set(id, cb);
      }
      return id;
    },
    "plugin:event|unlisten": ({ event, eventId }: any) => {
      const listeners = state.eventListeners.get(event);
      if (listeners) listeners.delete(eventId);
    },
    "plugin:event|emit": ({ event, payload }: any) => {
      emit(event, payload);
    },
  };

  const internals = {
    invoke: async (cmd: string, args: any = {}, _opts?: any) => {
      const h = handlers[cmd];
      if (!h) {
        console.warn("[tauri-mock] unhandled command:", cmd, args);
        throw new Error(`Mock: command "${cmd}" not implemented`);
      }
      try {
        return await h(args);
      } catch (e) {
        throw e;
      }
    },
    transformCallback: (cb: (v: any) => void, _once: boolean) => {
      const id = state.nextCallback++;
      state.callbacks.set(id, cb);
      return id;
    },
    unregisterCallback: (id: number) => {
      state.callbacks.delete(id);
    },
    convertFileSrc: (path: string) => `mock://${path}`,
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
  };

  (window as any).__TAURI_INTERNALS__ = internals;
  (window as any).__SKEIN_MOCK__ = { state, handlers, emit };
}
