// Reactive vault state for Phase 2.
// Listens to the Rust file watcher's "vault-changed" event and re-fetches.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  currentVault,
  openVault,
  openVaultFromArchive,
  closeVault,
  listBooks,
  listLoosePages,
  listPagesInBook,
  type Vault,
  type Book,
  type Page,
} from "./vault.js";
import { reconcileExternal } from "./tabs.svelte.js";

export const vaultState: {
  vault: Vault | null;
  books: Book[];
  loosePages: Page[];
  activeBook: string | null;
  pagesInActiveBook: Page[];
  loading: boolean;
  error: string | null;
} = $state({
  vault: null,
  books: [],
  loosePages: [],
  activeBook: null,
  pagesInActiveBook: [],
  loading: false,
  error: null,
});

let unlisten: UnlistenFn | null = null;

async function refreshVaultLists() {
  if (!vaultState.vault) return;
  try {
    const [books, loose] = await Promise.all([listBooks(), listLoosePages()]);
    vaultState.books = books;
    vaultState.loosePages = loose;
    if (vaultState.activeBook) {
      vaultState.pagesInActiveBook = await listPagesInBook(vaultState.activeBook);
    }
    await reconcileExternal();
  } catch (e) {
    vaultState.error = String(e);
  }
}

async function attachWatcher() {
  if (unlisten) return;
  unlisten = await listen("vault-changed", () => {
    refreshVaultLists();
  });
}

export async function bootstrap() {
  vaultState.loading = true;
  try {
    const v = await currentVault();
    if (v) {
      vaultState.vault = v;
      await refreshVaultLists();
      await attachWatcher();
    }
  } catch (e) {
    vaultState.error = String(e);
  } finally {
    vaultState.loading = false;
  }
}

export async function open(path: string) {
  vaultState.loading = true;
  vaultState.error = null;
  try {
    const v = await openVault(path);
    vaultState.vault = v;
    vaultState.activeBook = null;
    vaultState.pagesInActiveBook = [];
    await refreshVaultLists();
    await attachWatcher();
  } catch (e) {
    vaultState.error = String(e);
  } finally {
    vaultState.loading = false;
  }
}

export async function openFromArchive(archivePath: string, destDir: string) {
  vaultState.loading = true;
  vaultState.error = null;
  try {
    const v = await openVaultFromArchive(archivePath, destDir);
    vaultState.vault = v;
    vaultState.activeBook = null;
    vaultState.pagesInActiveBook = [];
    await refreshVaultLists();
    await attachWatcher();
  } catch (e) {
    vaultState.error = String(e);
    throw e;
  } finally {
    vaultState.loading = false;
  }
}

export async function close() {
  await closeVault();
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  vaultState.vault = null;
  vaultState.books = [];
  vaultState.loosePages = [];
  vaultState.activeBook = null;
  vaultState.pagesInActiveBook = [];
}

export async function selectBook(name: string | null) {
  vaultState.activeBook = name;
  if (name) {
    vaultState.pagesInActiveBook = await listPagesInBook(name);
  } else {
    vaultState.pagesInActiveBook = [];
  }
}
