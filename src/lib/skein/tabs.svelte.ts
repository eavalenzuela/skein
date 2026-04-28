// Tabs store for Phase 3.
//
// Mirrors the mockup's tab row: every open page is a tab. A tab can be
// pinned to "left" or "right" — when both pins are set, the desk shows
// the two pages side-by-side. Otherwise the active tab fills the desk.
//
// Persistence: in-memory only for v1. Restored open tabs across runs is
// a separate decision left for later.

import type { Page } from "./vault.js";
import { readPage, writePage } from "./vault.js";

export interface Tab {
  rel_path: string;
  title: string;
  body: string;
  saved: string;
  pin: "left" | "right" | null;
  loading: boolean;
}

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

export async function openTab(page: Pick<Page, "rel_path" | "title">) {
  const existing = findIndex(page.rel_path);
  if (existing !== -1) {
    tabsState.activeId = page.rel_path;
    return;
  }
  const placeholder: Tab = {
    rel_path: page.rel_path,
    title: page.title,
    body: "",
    saved: "",
    pin: null,
    loading: true,
  };
  tabsState.tabs = [...tabsState.tabs, placeholder];
  tabsState.activeId = page.rel_path;
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
