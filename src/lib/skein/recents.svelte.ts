// Most-recently-opened pages, per vault.
//
// Ctrl+K with an empty query used to show a blank panel, which wasted the
// app's main entry point and left the `:` and `#` prefixes undiscoverable.
// The palette now opens on this list.

export interface RecentPage {
  rel_path: string;
  title: string;
}

const PREFIX = "skein.recents.";
const LIMIT = 8;

export const recentsState: { items: RecentPage[] } = $state({ items: [] });

let root: string | null = null;

function persist() {
  if (!root) return;
  try {
    localStorage.setItem(PREFIX + root, JSON.stringify(recentsState.items));
  } catch {
    // Storage full or unavailable — recents are a convenience, not data.
  }
}

/** Point the store at a vault and load its list. */
export function startRecents(vaultRoot: string) {
  root = vaultRoot;
  try {
    const raw = JSON.parse(localStorage.getItem(PREFIX + vaultRoot) ?? "null");
    recentsState.items = Array.isArray(raw)
      ? raw
          .filter(
            (r): r is RecentPage =>
              !!r && typeof r.rel_path === "string" && typeof r.title === "string",
          )
          .slice(0, LIMIT)
      : [];
  } catch {
    recentsState.items = [];
  }
}

export function endRecents() {
  root = null;
  recentsState.items = [];
}

/** Record a page as just-opened, moving it to the front. */
export function noteRecent(page: RecentPage) {
  const rest = recentsState.items.filter((r) => r.rel_path !== page.rel_path);
  recentsState.items = [{ rel_path: page.rel_path, title: page.title }, ...rest].slice(0, LIMIT);
  persist();
}

/** Drop a page that no longer exists (deleted, or renamed away). */
export function forgetRecent(relPath: string) {
  const next = recentsState.items.filter((r) => r.rel_path !== relPath);
  if (next.length !== recentsState.items.length) {
    recentsState.items = next;
    persist();
  }
}
