// Typed in-page Tauri mock. This file is type-checked against the real
// Vault / Book / Page / Settings / GitStatus interfaces from src/lib/skein,
// so the mock drifts loudly when those types change.
//
// At test time the body of `installMock` is serialized via Function.toString()
// and shipped to the browser via Playwright's addInitScript. The function
// body therefore can NOT reference outer-scope variables — config is passed
// in as the only parameter and JSON-serialized into the wrapper script.

import type {
  Book,
  Page,
  PageTitle,
  SearchHit,
  Vault,
  EmbeddingModelStatus,
  DailyResult,
  RelatedHit,
  BacklinkHit,
  DeleteBookResult,
} from "../../../src/lib/skein/vault";
import type { Settings, GitStatus, GitPullResult } from "../../../src/lib/skein/settings";

export interface MockConfig {
  hasVault: boolean;
  vaultName: string;
  vaultRoot: string;
}

export interface MockState {
  vault: Vault | null;
  books: Book[];
  bookOrder: string[];
  pages: Map<string, Page>;
  pageBodies: Map<string, string>;
  settings: Settings;
  secrets: Set<string>;
  eventListeners: Map<string, Map<number, (e: { event: string; payload: unknown; id: number }) => void>>;
  nextEventId: number;
  callbacks: Map<number, (v: unknown) => void>;
  nextCallback: number;
}

// The browser side. Lives in this file so TypeScript checks every line —
// the return types match the real Tauri command signatures.
export function installMock(cfg: MockConfig): void {
  const state: MockState = {
    vault: cfg.hasVault ? { root: cfg.vaultRoot, name: cfg.vaultName } : null,
    books: [
      { name: "Research", rel_path: "Research", page_count: 2 },
      { name: "Daily", rel_path: "Daily", page_count: 3 },
      { name: "Recipes", rel_path: "Recipes", page_count: 1 },
    ],
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
    eventListeners: new Map(),
    nextEventId: 1,
    callbacks: new Map(),
    nextCallback: 1,
  };

  function emit(event: string, payload: unknown): void {
    const listeners = state.eventListeners.get(event);
    if (!listeners) return;
    for (const cb of listeners.values()) cb({ event, payload, id: 0 });
  }

  // Argument type used by every handler. Each command picks what it needs.
  type Args = Record<string, unknown>;

  const handlers: Record<string, (args: Args) => unknown> = {
    current_vault: () => state.vault,
    open_vault: ({ path }) => {
      const p = String(path);
      state.vault = { root: p, name: p.split("/").pop() || "vault" };
      return state.vault;
    },
    close_vault: () => {
      state.vault = null;
    },
    list_books: (): Book[] =>
      state.bookOrder
        .map((n) => state.books.find((b) => b.name === n))
        .filter((b): b is Book => b !== undefined),
    create_book: ({ name }) => {
      const n = String(name);
      if (!state.books.find((b) => b.name === n)) {
        state.books.push({ name: n, rel_path: n, page_count: 0 });
        state.bookOrder.push(n);
      }
      emit("vault-changed", null);
    },
    rename_book: ({ oldName, newName }) => {
      const o = String(oldName);
      const nn = String(newName);
      const b = state.books.find((b) => b.name === o);
      if (b) {
        b.name = nn;
        b.rel_path = nn;
        const i = state.bookOrder.indexOf(o);
        if (i >= 0) state.bookOrder[i] = nn;
        for (const p of state.pages.values()) {
          if (p.book === o) {
            p.book = nn;
            const newRel = p.rel_path.replace(o + "/", nn + "/");
            state.pages.delete(p.rel_path);
            p.rel_path = newRel;
            state.pages.set(newRel, p);
          }
        }
      }
      emit("vault-changed", null);
    },
    delete_book: ({ name, alsoDeletePages }): DeleteBookResult => {
      const n = String(name);
      const drop = !!alsoDeletePages;
      const deleted: string[] = [];
      const moved: [string, string][] = [];
      for (const p of [...state.pages.values()]) {
        if (p.book === n) {
          if (drop) {
            state.pages.delete(p.rel_path);
            deleted.push(p.rel_path);
          } else {
            const newRel = p.rel_path.split("/").pop()!;
            state.pages.delete(p.rel_path);
            const oldRel = p.rel_path;
            p.book = null;
            p.rel_path = newRel;
            state.pages.set(newRel, p);
            moved.push([oldRel, newRel]);
          }
        }
      }
      state.books = state.books.filter((b) => b.name !== n);
      state.bookOrder = state.bookOrder.filter((x) => x !== n);
      emit("vault-changed", null);
      return { deleted_rel_paths: deleted, moved };
    },
    set_book_order: ({ names }) => {
      state.bookOrder = (names as string[]).slice();
      emit("vault-changed", null);
    },
    create_page: ({ book, title }): string => {
      const b = book == null ? null : String(book);
      const t = String(title);
      const safe = t.replace(/[^a-zA-Z0-9 _-]/g, "");
      const rel = b ? `${b}/${safe}.md` : `${safe}.md`;
      state.pages.set(rel, {
        rel_path: rel,
        title: safe,
        book: b,
        tags: [],
        modified: Date.now() / 1000,
      });
      const found = state.books.find((bb) => bb.name === b);
      if (found) found.page_count += 1;
      emit("vault-changed", null);
      return rel;
    },
    rename_page: ({ relPath, newTitle }): string => {
      const r = String(relPath);
      const nt = String(newTitle);
      const p = state.pages.get(r);
      if (!p) throw new Error("not found");
      const dir = r.includes("/") ? r.split("/").slice(0, -1).join("/") + "/" : "";
      const newRel = `${dir}${nt}.md`;
      state.pages.delete(r);
      p.rel_path = newRel;
      p.title = nt;
      state.pages.set(newRel, p);
      emit("vault-changed", null);
      return newRel;
    },
    delete_page_command: ({ relPath }) => {
      const r = String(relPath);
      const p = state.pages.get(r);
      if (p?.book) {
        const b = state.books.find((bb) => bb.name === p.book);
        if (b) b.page_count = Math.max(0, b.page_count - 1);
      }
      state.pages.delete(r);
      emit("vault-changed", null);
    },
    list_loose_pages: (): Page[] =>
      [...state.pages.values()].filter((p) => p.book === null),
    list_pages_in_book: ({ book }): Page[] =>
      [...state.pages.values()].filter((p) => p.book === String(book)),
    read_page: ({ relPath }): string => state.pageBodies.get(String(relPath)) ?? "",
    write_page: ({ relPath, body }) => {
      state.pageBodies.set(String(relPath), String(body));
    },
    list_page_titles: (): PageTitle[] =>
      [...state.pages.values()].map((p) => ({ rel_path: p.rel_path, title: p.title })),
    find_backlinks: (): BacklinkHit[] => [],
    search_pages: ({ query }): SearchHit[] => {
      const q = String(query).toLowerCase();
      return [...state.pages.values()]
        .filter(
          (p) =>
            p.title.toLowerCase().includes(q) ||
            (state.pageBodies.get(p.rel_path) ?? "").toLowerCase().includes(q),
        )
        .map((p) => ({
          rel_path: p.rel_path,
          title: p.title,
          book: p.book,
          snippet: p.title,
        }));
    },
    find_related: (): RelatedHit[] => [],
    rebuild_index: (): number => state.pages.size,
    embedding_model_status: (): EmbeddingModelStatus => ({ name: "bge-small", local: false }),
    download_embedding_model: (): EmbeddingModelStatus => ({ name: "bge-small", local: true }),
    suggest_tags: (): string[] => [],
    apply_tag: ({ relPath, tag }) => {
      const p = state.pages.get(String(relPath));
      const t = String(tag);
      if (p && !p.tags.includes(t)) p.tags.push(t);
    },
    dismiss_tag: ({ relPath, tag }) => {
      const p = state.pages.get(String(relPath));
      const t = String(tag);
      if (p) p.tags = p.tags.filter((x) => x !== t);
    },
    open_today_daily: (): DailyResult => {
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
    save_attachment: (): string => "attachments/x.bin",
    save_attachment_from_path: (): string => "attachments/x.bin",
    export_vault: () => undefined,
    open_vault_from_archive: ({ destDir }): Vault => {
      state.vault = { root: String(destDir), name: "imported" };
      return state.vault;
    },

    get_settings: (): Settings => ({ ...state.settings }),
    set_settings: ({ patch }): Settings => {
      Object.assign(state.settings, patch as Partial<Settings>);
      return { ...state.settings };
    },
    has_secret: ({ name }): boolean => state.secrets.has(String(name)),
    set_secret: ({ name }) => {
      state.secrets.add(String(name));
    },
    clear_secret: ({ name }) => {
      state.secrets.delete(String(name));
    },

    git_status: (): GitStatus => ({
      initialized: false,
      branch: null,
      remote_url: null,
      ahead: 0,
      behind: 0,
      dirty: [],
      conflicted: [],
    }),
    git_set_remote: () => undefined,
    git_pull: (): GitPullResult => ({ kind: "up-to-date", conflicted: [] }),
    git_push: () => undefined,
    git_commit_all: (): boolean => true,

    chat_send: (): string => {
      const id = "turn-" + Math.random().toString(36).slice(2);
      setTimeout(() => emit(`chat-turn-${id}`, { kind: "done", text: "Mock response" }), 50);
      return id;
    },

    "plugin:event|listen": ({ event, handler }): number => {
      const id = state.nextEventId++;
      const h = state.callbacks.get(handler as number);
      if (h) {
        let listeners = state.eventListeners.get(String(event));
        if (!listeners) {
          listeners = new Map();
          state.eventListeners.set(String(event), listeners);
        }
        listeners.set(id, h as (e: { event: string; payload: unknown; id: number }) => void);
      }
      return id;
    },
    "plugin:event|unlisten": ({ event, eventId }) => {
      const listeners = state.eventListeners.get(String(event));
      if (listeners) listeners.delete(eventId as number);
    },
    "plugin:event|emit": ({ event, payload }) => {
      emit(String(event), payload);
    },
  };

  interface TauriInternals {
    invoke: (cmd: string, args?: unknown, opts?: unknown) => Promise<unknown>;
    transformCallback: (cb: (v: unknown) => void, once: boolean) => number;
    unregisterCallback: (id: number) => void;
    convertFileSrc: (path: string, protocol?: string) => string;
    metadata: { currentWindow: { label: string }; currentWebview: { label: string } };
  }

  const internals: TauriInternals = {
    invoke: async (cmd, args) => {
      const h = handlers[cmd];
      if (!h) {
        // eslint-disable-next-line no-console
        console.warn("[tauri-mock] unhandled command:", cmd, args);
        throw new Error(`Mock: command "${cmd}" not implemented`);
      }
      return h((args ?? {}) as Args);
    },
    transformCallback: (cb) => {
      const id = state.nextCallback++;
      state.callbacks.set(id, cb);
      return id;
    },
    unregisterCallback: (id) => {
      state.callbacks.delete(id);
    },
    convertFileSrc: (path) => `mock://${path}`,
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { label: "main" },
    },
  };

  const w = window as unknown as { __TAURI_INTERNALS__: TauriInternals; __SKEIN_MOCK__: { state: MockState; handlers: typeof handlers; emit: typeof emit } };
  w.__TAURI_INTERNALS__ = internals;
  w.__SKEIN_MOCK__ = { state, handlers, emit };
}
