// Tabs store for Phase 3.
//
// Mirrors the mockup's tab row: every open page is a tab. A tab can be
// pinned to "left" or "right" — when both pins are set, the desk shows
// the two pages side-by-side. Otherwise the active tab fills the desk.
//
// Persistence: in-memory only for v1. Restored open tabs across runs is
// a separate decision left for later.

import type { Page } from "./vault.js";
import {
  readPage,
  writePage,
  listLoosePages,
  listPagesInBook,
} from "./vault.js";

/** "user" — opened explicitly (click, drag, search hit). Sticky.
 *  "auto" — loaded as a sibling for navigation when its book became the
 *  current context. Removed when the context shifts (unless dirty/pinned). */
export type TabKind = "user" | "auto";

export interface Tab {
  rel_path: string;
  title: string;
  body: string;
  saved: string;
  pin: "left" | "right" | null;
  loading: boolean;
  kind: TabKind;
}

/** Infer the book name from a vault-relative page path. Loose pages return
 * null. Books are one level deep, per the design. */
function bookOf(relPath: string): string | null {
  const i = relPath.indexOf("/");
  return i === -1 ? null : relPath.slice(0, i);
}

let bookContext: string | null = null;

export const tabsState: { tabs: Tab[]; activeId: string | null } = $state({
  tabs: [],
  activeId: null,
});

const SAVE_DEBOUNCE_MS = 750;
const saveTimers = new Map<string, ReturnType<typeof setTimeout>>();

export function isDirty(tab: Tab): boolean {
  return tab.body !== tab.saved;
}

function findIndex(relPath: string): number {
  return tabsState.tabs.findIndex((t) => t.rel_path === relPath);
}

export async function openTab(
  page: Pick<Page, "rel_path" | "title">,
  kind: TabKind = "user",
) {
  const existing = findIndex(page.rel_path);
  if (existing !== -1) {
    // Promote auto → user if this page is now being explicitly chosen.
    if (kind === "user" && tabsState.tabs[existing].kind === "auto") {
      tabsState.tabs[existing].kind = "user";
    }
    if (kind === "user") tabsState.activeId = page.rel_path;
  } else {
    const placeholder: Tab = {
      rel_path: page.rel_path,
      title: page.title,
      body: "",
      saved: "",
      pin: null,
      loading: true,
      kind,
    };
    tabsState.tabs = [...tabsState.tabs, placeholder];
    if (kind === "user") tabsState.activeId = page.rel_path;
    try {
      const body = await readPage(page.rel_path);
      const i = findIndex(page.rel_path);
      if (i === -1) return;
      const t = tabsState.tabs[i];
      t.body = body;
      t.saved = body;
      t.loading = false;
    } catch (e) {
      const i = findIndex(page.rel_path);
      if (i !== -1) {
        tabsState.tabs[i].loading = false;
        tabsState.tabs[i].body = `# Error\n\n${String(e)}`;
      }
    }
  }
  // Only an explicit user-driven open shifts the book context. Auto-opens
  // recurse from inside populateBookContext and must not re-trigger it.
  if (kind === "user") {
    const newBook = bookOf(page.rel_path);
    if (newBook !== bookContext) {
      await populateBookContext(newBook);
    }
  }
}

/** Load every page in `book` (or every loose page when null) as auto tabs
 * so the user can navigate siblings without leaving the editor. Drops any
 * lingering auto tabs from the previous context that aren't pinned/dirty. */
export async function populateBookContext(book: string | null) {
  // Drop stale auto tabs from prior contexts.
  for (const t of [...tabsState.tabs]) {
    if (
      t.kind === "auto" &&
      t.pin == null &&
      !isDirty(t) &&
      bookOf(t.rel_path) !== book
    ) {
      tabsState.tabs = tabsState.tabs.filter((tt) => tt !== t);
    }
  }
  bookContext = book;
  try {
    const pages = book == null ? await listLoosePages() : await listPagesInBook(book);
    for (const p of pages) {
      // openTab no-ops if already open; for auto siblings we don't change
      // the active tab — that stays on the user's clicked page.
      await openTab({ rel_path: p.rel_path, title: p.title }, "auto");
    }
  } catch {
    // Listing failed (book deleted under us, etc.) — drop the context.
  }
}

export function closeTab(relPath: string) {
  const t = saveTimers.get(relPath);
  if (t) {
    clearTimeout(t);
    saveTimers.delete(relPath);
  }
  // Best-effort flush: if dirty, kick off an immediate save.
  const tab = tabsState.tabs.find((tt) => tt.rel_path === relPath);
  if (tab && isDirty(tab)) {
    void writePage(relPath, tab.body).catch(() => {});
  }
  tabsState.tabs = tabsState.tabs.filter((tt) => tt.rel_path !== relPath);
  if (tabsState.activeId === relPath) {
    tabsState.activeId = tabsState.tabs[tabsState.tabs.length - 1]?.rel_path ?? null;
  }
}

export function setActive(relPath: string) {
  if (findIndex(relPath) !== -1) tabsState.activeId = relPath;
}

export function togglePin(relPath: string, side: "left" | "right") {
  const i = findIndex(relPath);
  if (i === -1) return;
  const tab = tabsState.tabs[i];
  // Clear any other tab pinned to the same side.
  for (const other of tabsState.tabs) {
    if (other !== tab && other.pin === side) other.pin = null;
  }
  tab.pin = tab.pin === side ? null : side;
}

/** "Smart" pin used by the pin button on each tab. If the tab is already
 * pinned, unpin it. Otherwise pick whichever side is currently empty —
 * defaulting to left when both sides are empty *or* both are taken.
 * Avoids the original always-cycle-to-left bug where pinning a second
 * tab would silently bump the first off the left side. */
export function cyclePin(relPath: string) {
  const i = findIndex(relPath);
  if (i === -1) return;
  const tab = tabsState.tabs[i];
  if (tab.pin) {
    tab.pin = null;
    return;
  }
  const leftTaken = tabsState.tabs.some((t) => t !== tab && t.pin === "left");
  const rightTaken = tabsState.tabs.some((t) => t !== tab && t.pin === "right");
  const side: "left" | "right" =
    !leftTaken && rightTaken ? "left" :
    leftTaken && !rightTaken ? "right" :
    "left";
  for (const other of tabsState.tabs) {
    if (other !== tab && other.pin === side) other.pin = null;
  }
  tab.pin = side;
}

export function unpin(relPath: string) {
  const i = findIndex(relPath);
  if (i !== -1) tabsState.tabs[i].pin = null;
}

/** Open `page` (if not already open) and pin it to `side`, replacing any
 * other tab currently pinned there. Used by drag-onto-pinned-pane. */
export async function replaceAtPin(
  side: "left" | "right",
  page: Pick<Page, "rel_path" | "title">,
) {
  await openTab(page);
  // Clear the existing pin on this side, then pin the new tab.
  for (const t of tabsState.tabs) {
    if (t.pin === side && t.rel_path !== page.rel_path) t.pin = null;
  }
  const i = findIndex(page.rel_path);
  if (i !== -1) {
    tabsState.tabs[i].pin = side;
    tabsState.activeId = page.rel_path;
  }
}

export function setBody(relPath: string, body: string) {
  const i = findIndex(relPath);
  if (i === -1) return;
  tabsState.tabs[i].body = body;
  scheduleSave(relPath);
}

function scheduleSave(relPath: string) {
  const existing = saveTimers.get(relPath);
  if (existing) clearTimeout(existing);
  const timer = setTimeout(async () => {
    saveTimers.delete(relPath);
    const i = findIndex(relPath);
    if (i === -1) return;
    const tab = tabsState.tabs[i];
    if (!isDirty(tab)) return;
    try {
      await writePage(relPath, tab.body);
      tab.saved = tab.body;
    } catch {
      // leave dirty; user will see the indicator
    }
  }, SAVE_DEBOUNCE_MS);
  saveTimers.set(relPath, timer);
}

export function pinned(side: "left" | "right"): Tab | undefined {
  return tabsState.tabs.find((t) => t.pin === side);
}

export function activeTab(): Tab | undefined {
  return tabsState.tabs.find((t) => t.rel_path === tabsState.activeId);
}

/** External edit reconciliation. Called from vault.svelte.ts on watcher events. */
export async function reconcileExternal() {
  for (const tab of tabsState.tabs) {
    if (isDirty(tab)) continue;
    try {
      const body = await readPage(tab.rel_path);
      if (body !== tab.body) {
        tab.body = body;
        tab.saved = body;
      }
    } catch {
      // file may have been deleted; leave the tab and let the user notice
    }
  }
}
