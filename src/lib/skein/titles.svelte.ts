// Reactive cache of vault page titles for the [[wikilink]] autocomplete.
// Refreshed on vault-changed so newly-created or renamed pages show up.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { listPageTitles, type PageTitle } from "./vault.js";

export const titlesState: { items: PageTitle[] } = $state({ items: [] });

let unlisten: UnlistenFn | null = null;

export async function bootstrap() {
  await refresh();
  if (!unlisten) {
    unlisten = await listen("vault-changed", () => {
      void refresh();
    });
  }
}

export async function refresh() {
  try {
    titlesState.items = await listPageTitles();
  } catch {
    // index not ready yet — leave the previous list in place
  }
}
